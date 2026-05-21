use reqwest::Client;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::DEFAULT_LLM_URL;
use crate::services::database;
use crate::services::env_store::{read_env_value, read_system_prompt};
use crate::services::rag;

use serde_json::Value;

const MAX_TOOL_ITERATIONS: usize = 3;

const DEFAULT_API_BASE_URL: &str = "https://api.groq.com/openai/v1";
const DEFAULT_API_MODEL: &str = "llama-3.1-8b-instant";

fn current_datetime_string() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;

    // Compute year/month/day from days since Unix epoch
    let mut remaining = days_since_epoch;
    let mut year = 1970u64;
    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 366 } else { 365 };
        if remaining < days_in_year { break; }
        remaining -= days_in_year;
        year += 1;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = [31u64, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let month_names = ["January","February","March","April","May","June",
                       "July","August","September","October","November","December"];
    let mut month = 0usize;
    for (i, &d) in month_days.iter().enumerate() {
        if remaining < d { month = i; break; }
        remaining -= d;
    }
    let day = remaining + 1;

    format!(
        "{} {:02}, {} {:02}:{:02}:{:02} UTC",
        month_names[month], day, year, hour, minute, second
    )
}

async fn build_messages(prompt: &str, db_url: Option<&str>) -> Vec<serde_json::Value> {
    let mut system_prompt = read_system_prompt().trim().to_string();
    let datetime = current_datetime_string();

    let snippets = rag::retrieve_default(prompt);
    if !snippets.is_empty() {
        if !system_prompt.is_empty() {
            system_prompt.push_str("\n\n");
        }
        system_prompt.push_str(
            "Reference snippets retrieved from the uploaded file (these are the most relevant \
             sections for the customer's question — quote details verbatim when relevant):\n",
        );
        for (i, snip) in snippets.iter().enumerate() {
            system_prompt.push_str(&format!("\n[{}]\n{}\n", i + 1, snip));
        }
    }

    if let Some(url) = db_url {
        let schema = database::get_or_fetch_schema(url).await;
        if !schema.is_empty() {
            if !system_prompt.is_empty() {
                system_prompt.push_str("\n\n");
            }
            system_prompt.push_str(
                "You have read-only access to a Postgres database via the `query_database` tool. \
                 Only SELECT and WITH queries are allowed. Cast non-text columns to ::text when needed. \
                 Available tables:\n",
            );
            system_prompt.push_str(&schema);
        }
    }

    let mut messages = vec![];
    if !system_prompt.is_empty() {
        messages.push(json!({"role": "system", "content": system_prompt}));
    }

    let user_content = format!("[Current date and time: {}]\n{}", datetime, prompt);
    messages.push(json!({"role": "user", "content": user_content}));
    messages
}

async fn query_openai_compat(
    api_base_url: &str,
    api_key: &str,
    api_model: &str,
    db_url: Option<&str>,
    mut messages: Vec<serde_json::Value>,
) -> String {
    let base = api_base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return "Error: LLM_API_BASE_URL is empty. Set it to your provider base URL (e.g. https://api.openai.com/v1).".to_string();
    }
    if api_model.trim().is_empty() {
        return "Error: LLM_API_MODEL is required when using an API key.".to_string();
    }

    let url = format!("{}/chat/completions", base);
    let tools_schema = build_tools_schema(db_url.is_some());
    let client = Client::new();

    for _ in 0..=MAX_TOOL_ITERATIONS {
        let mut payload = json!({
            "model": api_model.trim(),
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": 400
        });
        if let Some(schema) = &tools_schema {
            payload["tools"] = schema.clone();
        }

        let res = client
            .post(&url)
            .bearer_auth(api_key.trim())
            .json(&payload)
            .send()
            .await;

        let response = match res {
            Ok(r) => r,
            Err(e) => return format!("Error: Failed to reach the LLM API. ({})", e),
        };
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        let parsed: serde_json::Value = match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(_) => return format!(
                "Error: I received a response, but couldn't parse the text. (status: {}, body: {})",
                status, body
            ),
        };

        if let Some(err_msg) = parsed["error"]["message"].as_str() {
            return format!("Error: {}", err_msg);
        }

        let message = &parsed["choices"][0]["message"];
        let tool_calls = message["tool_calls"].as_array().cloned().unwrap_or_default();

        if tool_calls.is_empty() {
            if let Some(content) = message["content"].as_str() {
                return content.to_string();
            }
            return format!(
                "Error: response missing content. (status: {}, body: {})",
                status, body
            );
        }

        messages.push(message.clone());
        run_tool_calls(&tool_calls, db_url, &mut messages).await;
    }

    "Error: tool-call loop exceeded maximum iterations.".to_string()
}

fn build_tools_schema(has_db: bool) -> Option<Value> {
    if !has_db {
        return None;
    }
    Some(json!([{
        "type": "function",
        "function": {
            "name": "query_database",
            "description": "Run a read-only SQL query against the connected Postgres database. Use this to fetch real data — products, orders, customers, stock, etc. Only SELECT and WITH (CTE) statements are allowed; INSERT/UPDATE/DELETE/DDL will be rejected. Returns rows in a pipe-separated table; max 50 rows.",
            "parameters": {
                "type": "object",
                "properties": {
                    "sql": {
                        "type": "string",
                        "description": "A single SELECT or WITH query. Cast non-text columns to ::text if needed."
                    }
                },
                "required": ["sql"]
            }
        }
    }]))
}

async fn run_tool_calls(
    tool_calls: &[Value],
    db_url: Option<&str>,
    messages: &mut Vec<Value>,
) {
    for call in tool_calls {
        let id = call["id"].as_str().unwrap_or("").to_string();
        let name = call["function"]["name"].as_str().unwrap_or("").to_string();
        let args = call["function"]["arguments"].as_str().unwrap_or("{}").to_string();

        let result = if name == "query_database" {
            match db_url {
                Some(url) => {
                    let parsed: Value = serde_json::from_str(&args).unwrap_or(Value::Null);
                    let sql = parsed["sql"].as_str().unwrap_or("");
                    println!("→ db: {}", sql);
                    database::execute_query(url, sql).await
                }
                None => "Error: database not configured".to_string(),
            }
        } else {
            format!("Error: unknown tool '{}'", name)
        };

        messages.push(json!({
            "role": "tool",
            "tool_call_id": id,
            "content": result
        }));
    }
}

async fn query_local_llm(llm_url: &str, messages: Vec<serde_json::Value>) -> String {
    let client = Client::new();
    let payload = json!({
        "messages": messages,
        "temperature": 0.7,
        "max_tokens": 150
    });

    let res = client.post(llm_url)
        .json(&payload)
        .send()
        .await;

    match res {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                    return content.to_string();
                }
                if let Some(err_msg) = json["error"]["message"].as_str() {
                    return format!("Error: {}", err_msg);
                }
            }

            format!(
                "Error: I received a response, but couldn't parse the text. (status: {}, body: {})",
                status,
                body
            )
        }
        Err(e) => format!("Error: Failed to reach the local LLM server at {}. ({})", llm_url, e),
    }
}

pub async fn query_llm(prompt: &str) -> String {
    let api_key = read_env_value("LLM_API_KEY").unwrap_or_default();
    let db_url = read_env_value("DATABASE_URL").unwrap_or_default();
    let db_url = if db_url.trim().is_empty() {
        None
    } else {
        Some(db_url.trim().to_string())
    };

    let use_api = !api_key.trim().is_empty();
    let db_for_messages = if use_api { db_url.as_deref() } else { None };
    let messages = build_messages(prompt, db_for_messages).await;

    if use_api {
        let api_base_url = read_env_value("LLM_API_BASE_URL")
            .unwrap_or_else(|| DEFAULT_API_BASE_URL.to_string());
        let api_model = read_env_value("LLM_API_MODEL")
            .unwrap_or_else(|| DEFAULT_API_MODEL.to_string());
        let api_base_url = if api_base_url.trim().is_empty() {
            DEFAULT_API_BASE_URL.to_string()
        } else {
            api_base_url
        };
        let api_model = if api_model.trim().is_empty() {
            DEFAULT_API_MODEL.to_string()
        } else {
            api_model
        };
        let reply = query_openai_compat(
            &api_base_url,
            &api_key,
            &api_model,
            db_url.as_deref(),
            messages,
        )
        .await;
        println!("LLM reply: {}", reply);
        return reply;
    }

    let llm_url = read_env_value("LLM_URL").unwrap_or_else(|| DEFAULT_LLM_URL.to_string());
    let reply = query_local_llm(&llm_url, messages).await;
    println!("LLM reply: {}", reply);
    reply
}
