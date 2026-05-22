use reqwest::Client;
use serde_json::{json, Value};

use crate::config::DEFAULT_LLM_URL;
use crate::services::database;
use crate::services::env_store::{read_env_value, read_system_prompt};
use crate::services::rag;

const MAX_TOOL_ITERATIONS: usize = 3;
const DEFAULT_API_BASE_URL: &str = "https://api.groq.com/openai/v1";
const DEFAULT_API_MODEL: &str = "llama-3.1-8b-instant";

fn env_or(key: &str, default: &str) -> String {
    match read_env_value(key) {
        Some(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => default.to_string(),
    }
}

fn env_opt(key: &str) -> Option<String> {
    read_env_value(key).and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

async fn build_messages(prompt: &str, db_url: Option<&str>) -> Vec<Value> {
    let mut system_prompt = read_system_prompt().trim().to_string();

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
    messages.push(json!({"role": "user", "content": prompt}));
    messages
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

async fn run_tool_calls(tool_calls: &[Value], db_url: Option<&str>, messages: &mut Vec<Value>) {
    for call in tool_calls {
        let id = call["id"].as_str().unwrap_or("").to_string();
        let name = call["function"]["name"].as_str().unwrap_or("");
        let args = call["function"]["arguments"].as_str().unwrap_or("{}");

        let result = if name == "query_database" {
            match db_url {
                Some(url) => {
                    let parsed: Value = serde_json::from_str(args).unwrap_or(Value::Null);
                    let sql = parsed["sql"].as_str().unwrap_or("");
                    println!("→ db: {}", sql);
                    database::execute_query(url, sql).await
                }
                None => "Error: database not configured".to_string(),
            }
        } else {
            format!("Error: unknown tool '{}'", name)
        };

        messages.push(json!({"role": "tool", "tool_call_id": id, "content": result}));
    }
}

async fn query_openai_compat(
    api_base_url: &str,
    api_key: &str,
    api_model: &str,
    db_url: Option<&str>,
    mut messages: Vec<Value>,
) -> String {
    let base = api_base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return "Error: LLM_API_BASE_URL is empty.".to_string();
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

        let response = match client
            .post(&url)
            .bearer_auth(api_key.trim())
            .json(&payload)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => return format!("Error: Failed to reach the LLM API. ({})", e),
        };
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        let parsed: Value = match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(_) => {
                return format!(
                    "Error: couldn't parse response. (status: {}, body: {})",
                    status, body
                )
            }
        };

        let err_msg = parsed["error"]["message"]
            .as_str()
            .or_else(|| parsed[0]["error"]["message"].as_str());
        if let Some(err_msg) = err_msg {
            return format!("Error: {}", err_msg);
        }

        let message = &parsed["choices"][0]["message"];
        let tool_calls = message["tool_calls"].as_array().cloned().unwrap_or_default();

        if tool_calls.is_empty() {
            return message["content"]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    format!(
                        "Error: response missing content. (status: {}, body: {})",
                        status, body
                    )
                });
        }

        messages.push(message.clone());
        run_tool_calls(&tool_calls, db_url, &mut messages).await;
    }

    "Error: tool-call loop exceeded maximum iterations.".to_string()
}

async fn query_local_llm(llm_url: &str, messages: Vec<Value>) -> String {
    let client = Client::new();
    let payload = json!({
        "messages": messages,
        "temperature": 0.7,
        "max_tokens": 150
    });

    let response = match client.post(llm_url).json(&payload).send().await {
        Ok(r) => r,
        Err(e) => return format!("Error: Failed to reach the local LLM server at {}. ({})", llm_url, e),
    };
    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    if let Ok(json) = serde_json::from_str::<Value>(&body) {
        if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
            return content.to_string();
        }
        if let Some(err) = json["error"]["message"].as_str() {
            return format!("Error: {}", err);
        }
    }
    format!("Error: couldn't parse response. (status: {}, body: {})", status, body)
}

pub async fn query_llm(prompt: &str) -> String {
    let api_key = env_opt("LLM_API_KEY");
    let db_url = env_opt("DATABASE_URL");

    let db_for_messages = api_key.as_ref().and(db_url.as_deref());
    let messages = build_messages(prompt, db_for_messages).await;

    let reply = if let Some(key) = api_key {
        let base = env_or("LLM_API_BASE_URL", DEFAULT_API_BASE_URL);
        let model = env_or("LLM_API_MODEL", DEFAULT_API_MODEL);
        query_openai_compat(&base, &key, &model, db_url.as_deref(), messages).await
    } else {
        let url = env_or("LLM_URL", DEFAULT_LLM_URL);
        query_local_llm(&url, messages).await
    };

    println!("LLM reply: {}", reply);
    reply
}
