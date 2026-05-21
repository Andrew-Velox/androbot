use axum::{extract::{Form, Multipart, Query, State}, response::{Html, Redirect}};
use serde::Deserialize;

use crate::config::{DEFAULT_BIND_ADDR, DEFAULT_LLM_URL};
use crate::services::rag;
use crate::services::telegram::{run_telegram_bot, validate_telegram_token};
use crate::services::env_store::{read_env_value, write_env_file, read_system_prompt, write_system_prompt};
use crate::AppState;

#[derive(Deserialize)]
pub struct SetupForm {
    page_access_token: String,
    telegram_bot_token: Option<String>,
    system_prompt: Option<String>,
  llm_api_base_url: Option<String>,
  llm_api_model: Option<String>,
  llm_api_key: Option<String>,
  database_url: Option<String>,
  database_allowed_tables: Option<String>,
  database_blocked_columns: Option<String>,
}

#[derive(Deserialize)]
pub struct SetupQuery {
    toast: Option<String>,
}

pub async fn setup_page(Query(q): Query<SetupQuery>) -> Html<String> {
    let page_access_token = read_env_value("PAGE_ACCESS_TOKEN").unwrap_or_default();
    let telegram_bot_token = read_env_value("TELEGRAM_BOT_TOKEN").unwrap_or_default();
    let system_prompt = read_system_prompt();
    let llm_api_base_url = read_env_value("LLM_API_BASE_URL").unwrap_or_default();
    let llm_api_model = read_env_value("LLM_API_MODEL").unwrap_or_default();
    let llm_api_key = read_env_value("LLM_API_KEY").unwrap_or_default();
    let database_url = read_env_value("DATABASE_URL").unwrap_or_default();
    let database_allowed_tables = read_env_value("DATABASE_ALLOWED_TABLES").unwrap_or_default();
    let database_blocked_columns = read_env_value("DATABASE_BLOCKED_COLUMNS").unwrap_or_default();
    let uploaded = rag::info();
    let upload_status_html = match &uploaded {
        Some((name, n_chunks)) => format!(
            r#"<div style="background:#0f1117;border:1px solid #2d3148;border-radius:8px;padding:0.75rem 1rem;display:flex;align-items:center;justify-content:space-between;gap:0.75rem;margin-bottom:0.75rem;">
              <div style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">
                <span style="color:#6ee7b7;">&#10003;</span>
                <span style="color:#e2e8f0;font-family:'Courier New',monospace;font-size:0.85rem;">{}</span>
                <span style="color:#64748b;font-size:0.8rem;">&middot; {} chunks indexed</span>
              </div>
              <form method="post" action="/setup/delete-upload" style="margin:0;">
                <button type="submit" style="background:#450a0a;color:#fca5a5;border:1px solid #7f1d1d;border-radius:6px;padding:0.35rem 0.75rem;font-size:0.8rem;cursor:pointer;">Remove</button>
              </form>
            </div>"#,
            html_escape(name),
            n_chunks
        ),
        None => String::new(),
    };

    let toast_html = match q.toast.as_deref() {
        Some("saved")      => r#"<div class="toast">&#10003;&nbsp; Configuration saved</div>"#,
        Some("tg_started") => r#"<div class="toast">&#10003;&nbsp; Configuration saved &mdash; Telegram bot started</div>"#,
        Some("tg_updated") => r#"<div class="toast">&#10003;&nbsp; Configuration saved &mdash; Telegram bot restarted</div>"#,
        Some("tg_stopped") => r#"<div class="toast">&#10003;&nbsp; Configuration saved &mdash; Telegram bot stopped</div>"#,
        Some("tg_invalid") => r#"<div class="toast toast-error">&#10007;&nbsp; Invalid Telegram token &mdash; bot not started</div>"#,
        Some("db_bad")     => r#"<div class="toast toast-error">&#10007;&nbsp; Database URL must start with postgres:// &mdash; not saved</div>"#,
        Some("uploaded")   => r#"<div class="toast">&#10003;&nbsp; File uploaded and parsed</div>"#,
        Some("upload_bad") => r#"<div class="toast toast-error">&#10007;&nbsp; Unsupported file type or parse failed</div>"#,
        Some("upload_empty") => r#"<div class="toast toast-error">&#10007;&nbsp; No file received</div>"#,
        Some("upload_deleted") => r#"<div class="toast">&#10003;&nbsp; Uploaded file removed</div>"#,
        _                  => "",
    };

    let html = format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Androbot Setup</title>
  <style>
    *, *::before, *::after {{ box-sizing: border-box; margin: 0; padding: 0; }}

    body {{
      min-height: 100vh;
      display: flex;
      align-items: center;
      justify-content: center;
      background: #0f1117;
      font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
      color: #e2e8f0;
      padding: 1.5rem;
    }}

    .card {{
      width: 100%;
      max-width: 560px;
      background: #1a1d27;
      border: 1px solid #2d3148;
      border-radius: 16px;
      padding: 2.5rem 2rem;
      box-shadow: 0 24px 48px rgba(0,0,0,0.4);
    }}

    .logo-row {{
      display: flex;
      align-items: center;
      gap: 0.75rem;
      margin-bottom: 0.5rem;
    }}

    .logo-icon {{
      width: 40px;
      height: 40px;
      background: linear-gradient(135deg, #6366f1, #8b5cf6);
      border-radius: 10px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 1.25rem;
      flex-shrink: 0;
    }}

    h1 {{
      font-size: 1.5rem;
      font-weight: 700;
      color: #f8fafc;
      letter-spacing: -0.02em;
    }}

    .subtitle {{
      color: #64748b;
      font-size: 0.9rem;
      margin-bottom: 2rem;
      padding-left: calc(40px + 0.75rem);
    }}

    .divider {{
      border: none;
      border-top: 1px solid #2d3148;
      margin-bottom: 2rem;
    }}

    .field {{
      margin-bottom: 1.5rem;
    }}

    label {{
      display: block;
      font-size: 0.85rem;
      font-weight: 600;
      color: #94a3b8;
      text-transform: uppercase;
      letter-spacing: 0.06em;
      margin-bottom: 0.5rem;
    }}

    .field-desc {{
      font-size: 0.8rem;
      color: #475569;
      margin-bottom: 0.5rem;
    }}

    input[type="text"], input[type="password"] {{
      width: 100%;
      padding: 0.75rem 1rem;
      background: #0f1117;
      border: 1px solid #2d3148;
      border-radius: 8px;
      color: #e2e8f0;
      font-size: 0.95rem;
      font-family: 'Courier New', monospace;
      transition: border-color 0.2s, box-shadow 0.2s;
      outline: none;
    }}

    input[type="text"]:focus, input[type="password"]:focus {{
      border-color: #6366f1;
      box-shadow: 0 0 0 3px rgba(99,102,241,0.15);
    }}

    input[type="text"]::placeholder, input[type="password"]::placeholder {{
      color: #334155;
    }}

    .badge {{
      display: inline-block;
      font-size: 0.7rem;
      font-weight: 600;
      padding: 0.15rem 0.5rem;
      border-radius: 4px;
      margin-left: 0.4rem;
      vertical-align: middle;
      text-transform: uppercase;
      letter-spacing: 0.05em;
    }}

    .badge-optional {{ background: #1e3a2e; color: #6ee7b7; }}

    textarea {{
      width: 100%;
      padding: 0.75rem 1rem;
      background: #0f1117;
      border: 1px solid #2d3148;
      border-radius: 8px;
      color: #e2e8f0;
      font-size: 0.85rem;
      font-family: 'Courier New', monospace;
      line-height: 1.5;
      resize: vertical;
      min-height: 140px;
      transition: border-color 0.2s, box-shadow 0.2s;
      outline: none;
    }}
    textarea:focus {{
      border-color: #6366f1;
      box-shadow: 0 0 0 3px rgba(99,102,241,0.15);
    }}
    textarea::placeholder {{ color: #334155; }}


    button[type="submit"] {{
      width: 100%;
      padding: 0.85rem;
      background: linear-gradient(135deg, #6366f1, #8b5cf6);
      border: none;
      border-radius: 8px;
      color: #fff;
      font-size: 1rem;
      font-weight: 600;
      cursor: pointer;
      transition: opacity 0.2s, transform 0.1s;
      letter-spacing: 0.01em;
    }}

    button[type="submit"]:hover {{ opacity: 0.9; }}
    button[type="submit"]:active {{ transform: scale(0.99); }}

    .toast {{
      position: fixed;
      bottom: 1.75rem;
      left: 50%;
      transform: translateX(-50%) translateY(0);
      background: #14532d;
      color: #bbf7d0;
      border: 1px solid #16a34a;
      border-radius: 10px;
      padding: 0.75rem 1.25rem;
      font-size: 0.9rem;
      font-weight: 500;
      white-space: nowrap;
      box-shadow: 0 8px 24px rgba(0,0,0,0.5);
      display: flex;
      align-items: center;
      gap: 0.5rem;
      z-index: 999;
      animation: toastIn 0.3s ease, toastOut 0.4s ease 3s forwards;
    }}
    .toast-error {{
      background: #450a0a;
      color: #fca5a5;
      border-color: #dc2626;
    }}

    @keyframes toastIn {{
      from {{ opacity: 0; transform: translateX(-50%) translateY(0.75rem); }}
      to   {{ opacity: 1; transform: translateX(-50%) translateY(0); }}
    }}
    @keyframes toastOut {{
      to   {{ opacity: 0; transform: translateX(-50%) translateY(0.75rem); }}
    }}

    @media (max-width: 480px) {{
      .card {{ padding: 1.75rem 1.25rem; }}
      h1 {{ font-size: 1.3rem; }}
      .subtitle {{ padding-left: 0; }}
      .toast {{ white-space: normal; text-align: center; width: calc(100% - 3rem); }}
    }}
  </style>
</head>
<body>
  <div class="card">
    <div class="logo-row">
      <div class="logo-icon">&#129302;</div>
      <h1>Androbot Setup</h1>
    </div>
    <p class="subtitle">Enable one or both integrations — each is independent.</p>
    <hr class="divider">

    <form method="post" action="/setup" autocomplete="off">

      <div class="field">
        <label for="api_key">
          LLM API Key
          <span class="badge badge-optional">Optional</span>
        </label>
        <p class="field-desc">If set, Groq API is used and the local GGUF model is skipped.</p>
        <input
          id="api_key"
          name="llm_api_key"
          type="password"
          value="{llm_api_key}"
          placeholder="sk-..."
          spellcheck="false"
          autocomplete="off"
        >
      </div>

      <details class="field">
        <summary style="cursor:pointer; color:#94a3b8; font-weight:600; letter-spacing:0.06em; text-transform:uppercase; font-size:0.8rem;">
          Advanced API Settings
        </summary>
        <div style="margin-top:1rem;">
          <div class="field">
            <label for="api_base">
              LLM API Base URL
              <span class="badge badge-optional">Optional</span>
            </label>
            <p class="field-desc">OpenAI-compatible base URL. Example: https://api.openai.com/v1 or https://api.groq.com/openai/v1</p>
            <input
              id="api_base"
              name="llm_api_base_url"
              type="text"
              value="{llm_api_base_url}"
              placeholder="https://api.groq.com/openai/v1"
              spellcheck="false"
              autocomplete="off"
            >
          </div>

          <div class="field">
            <label for="api_model">
              LLM API Model
              <span class="badge badge-optional">Optional</span>
            </label>
            <p class="field-desc">Model name for your provider. Example: llama3-8b-8192.</p>
            <input
              id="api_model"
              name="llm_api_model"
              type="text"
              value="{llm_api_model}"
              placeholder="llama3-8b-8192"
              spellcheck="false"
              autocomplete="off"
            >
          </div>
        </div>
      </details>

      <hr class="divider">

      <div class="field">
        <label for="pat">
          Facebook Page Access Token
          <span class="badge badge-optional">Optional</span>
        </label>
        <p class="field-desc">Your Meta / Facebook Page Access Token. Leave blank to disable Facebook Messenger.</p>
        <input
          id="pat"
          name="page_access_token"
          type="password"
          value="{page_token}"
          placeholder="EAAxxxxx..."
          spellcheck="false"
          autocomplete="off"
        >
      </div>

      <div class="field">
        <label for="tgt">
          Telegram Bot Token
          <span class="badge badge-optional">Optional</span>
        </label>
        <p class="field-desc">Obtained from @BotFather. Leave blank to disable Telegram.</p>
        <input
          id="tgt"
          name="telegram_bot_token"
          type="password"
          value="{tg_token}"
          placeholder="123456:ABCdef..."
          spellcheck="false"
          autocomplete="off"
        >
      </div>

      <div class="field">
        <label for="sp">System Prompt</label>
        <p class="field-desc">Defines the AI persona sent to the LLM on every message. Leave blank to use the model's default.</p>
        <textarea
          id="sp"
          name="system_prompt"
          placeholder="You are a helpful assistant..."
          spellcheck="false"
        >{system_prompt}</textarea>
      </div>

      <div class="field">
        <label for="db_url">
          Database URL (Postgres)
          <span class="badge badge-optional">Optional</span>
        </label>
        <p class="field-desc">Read-only AI access to your DB. Best practice: use a Postgres role with <code>GRANT SELECT</code> only on tables you want exposed (see README).</p>
        <input
          id="db_url"
          name="database_url"
          type="password"
          value="{database_url}"
          placeholder="postgres://readonly_user:pass@host:5432/dbname"
          spellcheck="false"
          autocomplete="off"
        >
      </div>

      <details class="field">
        <summary style="cursor:pointer; color:#94a3b8; font-weight:600; letter-spacing:0.06em; text-transform:uppercase; font-size:0.8rem;">
          Database Access Filters
        </summary>
        <div style="margin-top:1rem;">
          <div class="field">
            <label for="db_tables">
              Allowed Tables
              <span class="badge badge-optional">Optional</span>
            </label>
            <p class="field-desc">Comma-separated table names. If set, the AI only sees these in the schema. Empty = all public tables visible.</p>
            <input
              id="db_tables"
              name="database_allowed_tables"
              type="text"
              value="{database_allowed_tables}"
              placeholder="products, orders, categories"
              spellcheck="false"
              autocomplete="off"
            >
          </div>

          <div class="field">
            <label for="db_blocked">
              Blocked Column Patterns
              <span class="badge badge-optional">Optional</span>
            </label>
            <p class="field-desc">Comma-separated substrings. Columns whose name contains any of these are hidden from the AI. Example values block PII/secrets.</p>
            <input
              id="db_blocked"
              name="database_blocked_columns"
              type="text"
              value="{database_blocked_columns}"
              placeholder="password, secret, token, api_key, ssn"
              spellcheck="false"
              autocomplete="off"
            >
          </div>
        </div>
      </details>

      <button type="submit">Save Configuration</button>
    </form>

    <hr class="divider" style="margin-top:2rem;">

    <form method="post" action="/setup/upload" enctype="multipart/form-data" class="field" style="margin-bottom:0;">
      <label for="ctxfile">
        Store File
        <span class="badge badge-optional">Optional</span>
      </label>
      <p class="field-desc">Upload a product list, FAQ, or any reference file. Indexed with BM25 — AI retrieves the most relevant snippets per question. Supports .txt, .md, .csv, .tsv, .json, .xlsx, .xls, .ods. Max 10 MB.</p>
      {upload_status_html}
      <div style="display:flex; gap:0.5rem;">
        <input id="ctxfile" name="file" type="file" accept=".txt,.md,.csv,.tsv,.json,.xlsx,.xls,.ods" style="flex:1; color:#94a3b8;" required>
        <button type="submit" style="background:linear-gradient(135deg,#10b981,#059669);border:none;border-radius:8px;color:#fff;padding:0 1rem;font-weight:600;cursor:pointer;">Upload</button>
      </div>
    </form>
  </div>

  {toast_html}

  <script>
    const t = document.querySelector('.toast');
    if (t) {{
      history.replaceState(null, '', '/setup');
      setTimeout(() => t.remove(), 3400);
    }}
  </script>
</body>
</html>"#,
        page_token = page_access_token,
        tg_token = telegram_bot_token,
        system_prompt = system_prompt,
        llm_api_base_url = llm_api_base_url,
        llm_api_model = llm_api_model,
        llm_api_key = llm_api_key,
        database_url = database_url,
        database_allowed_tables = database_allowed_tables,
        database_blocked_columns = database_blocked_columns,
        upload_status_html = upload_status_html,
        toast_html = toast_html,
    );

    Html(html)
}

pub async fn save_setup(
    State(state): State<AppState>,
    Form(form): Form<SetupForm>,
) -> Redirect {
    let previous_tg = read_env_value("TELEGRAM_BOT_TOKEN").unwrap_or_default();
    let llm_url = read_env_value("LLM_URL").unwrap_or_else(|| DEFAULT_LLM_URL.to_string());
    let bind_addr = read_env_value("BIND_ADDR").unwrap_or_else(|| DEFAULT_BIND_ADDR.to_string());
    let llm_api_base_url = form.llm_api_base_url.as_deref().unwrap_or("").to_string();
    let llm_api_model = form.llm_api_model.as_deref().unwrap_or("").to_string();
    let llm_api_key = form.llm_api_key.as_deref().unwrap_or("").to_string();
    let database_url = form.database_url.as_deref().unwrap_or("").trim().to_string();
    let database_allowed_tables = form.database_allowed_tables.as_deref().unwrap_or("").trim().to_string();
    let database_blocked_columns = form.database_blocked_columns.as_deref().unwrap_or("").trim().to_string();
    let db_invalid = !database_url.is_empty()
        && !database_url.starts_with("postgres://")
        && !database_url.starts_with("postgresql://");

    let new_tg = form.telegram_bot_token.as_deref().unwrap_or("").trim().to_string();
    let new_prompt = form.system_prompt.as_deref().unwrap_or("").to_string();
    let _ = write_system_prompt(&new_prompt);

    let db_to_write = if db_invalid { "" } else { database_url.as_str() };
    let _ = write_env_file(&[
        ("PAGE_ACCESS_TOKEN", form.page_access_token.as_str()),
        ("TELEGRAM_BOT_TOKEN", new_tg.as_str()),
        ("LLM_URL", llm_url.as_str()),
        ("BIND_ADDR", bind_addr.as_str()),
        ("LLM_API_BASE_URL", llm_api_base_url.trim()),
        ("LLM_API_MODEL", llm_api_model.trim()),
        ("LLM_API_KEY", llm_api_key.trim()),
        ("DATABASE_URL", db_to_write),
        ("DATABASE_ALLOWED_TABLES", database_allowed_tables.as_str()),
        ("DATABASE_BLOCKED_COLUMNS", database_blocked_columns.as_str()),
    ]);

    let toast = if db_invalid {
        "db_bad"
    } else if new_tg != previous_tg {
        if !new_tg.is_empty() {
            if !validate_telegram_token(&new_tg).await {
                println!("⚠️  Setup: invalid Telegram token submitted — bot not started");
                "tg_invalid"
            } else {
                let mut task = state.telegram_task.lock().await;
                if let Some(handle) = task.take() {
                    handle.abort();
                }
                let token = new_tg.clone();
                *task = Some(tokio::spawn(async move {
                  run_telegram_bot(token).await;
                }));
                if previous_tg.is_empty() { "tg_started" } else { "tg_updated" }
            }
        } else {
            let mut task = state.telegram_task.lock().await;
            if let Some(handle) = task.take() {
                handle.abort();
            }
            "tg_stopped"
        }
    } else {
        "saved"
    };

    Redirect::to(&format!("/setup?toast={}", toast))
}

pub async fn upload_context(mut multipart: Multipart) -> Redirect {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name != "file" {
            continue;
        }
        let filename = field.file_name().unwrap_or("upload").to_string();
        let bytes = match field.bytes().await {
            Ok(b) => b,
            Err(e) => {
                eprintln!("upload read error: {}", e);
                return Redirect::to("/setup?toast=upload_bad");
            }
        };
        if bytes.is_empty() {
            return Redirect::to("/setup?toast=upload_empty");
        }
        let text = match rag::extract_text(&filename, &bytes) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("upload extract error: {}", e);
                return Redirect::to("/setup?toast=upload_bad");
            }
        };
        match rag::build_index(&filename, &text) {
            Ok(n) => {
                println!("RAG index built: {} chunks from {}", n, filename);
                return Redirect::to("/setup?toast=uploaded");
            }
            Err(e) => {
                eprintln!("RAG index error: {}", e);
                return Redirect::to("/setup?toast=upload_bad");
            }
        }
    }
    Redirect::to("/setup?toast=upload_empty")
}

pub async fn delete_upload() -> Redirect {
    rag::clear();
    Redirect::to("/setup?toast=upload_deleted")
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
