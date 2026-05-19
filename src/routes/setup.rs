use axum::{extract::{Form, Query, State}, response::{Html, Redirect}};
use serde::Deserialize;

use crate::config::{DEFAULT_BIND_ADDR, DEFAULT_LLM_URL};
use crate::services::telegram::{run_telegram_bot, validate_telegram_token};
use crate::services::env_store::{read_env_value, write_env_file, read_system_prompt, write_system_prompt};
use crate::AppState;

#[derive(Deserialize)]
pub struct SetupForm {
    page_access_token: String,
    telegram_bot_token: Option<String>,
    system_prompt: Option<String>,
}

#[derive(Deserialize)]
pub struct SetupQuery {
    toast: Option<String>,
}

pub async fn setup_page(Query(q): Query<SetupQuery>) -> Html<String> {
    let page_access_token = read_env_value("PAGE_ACCESS_TOKEN").unwrap_or_default();
    let telegram_bot_token = read_env_value("TELEGRAM_BOT_TOKEN").unwrap_or_default();
    let system_prompt = read_system_prompt();

    let toast_html = match q.toast.as_deref() {
        Some("saved")      => r#"<div class="toast">&#10003;&nbsp; Configuration saved</div>"#,
        Some("tg_started") => r#"<div class="toast">&#10003;&nbsp; Configuration saved &mdash; Telegram bot started</div>"#,
        Some("tg_updated") => r#"<div class="toast">&#10003;&nbsp; Configuration saved &mdash; Telegram bot restarted</div>"#,
        Some("tg_stopped") => r#"<div class="toast">&#10003;&nbsp; Configuration saved &mdash; Telegram bot stopped</div>"#,
        Some("tg_invalid") => r#"<div class="toast toast-error">&#10007;&nbsp; Invalid Telegram token &mdash; bot not started</div>"#,
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

      

      <button type="submit">Save Configuration</button>
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

    let new_tg = form.telegram_bot_token.as_deref().unwrap_or("").trim().to_string();
    let new_prompt = form.system_prompt.as_deref().unwrap_or("").to_string();
    let _ = write_system_prompt(&new_prompt);

    let _ = write_env_file(
        &form.page_access_token,
        if new_tg.is_empty() { None } else { Some(new_tg.as_str()) },
        &llm_url,
        &bind_addr,
    );

    let toast = if new_tg != previous_tg {
        if !new_tg.is_empty() {
            if !validate_telegram_token(&new_tg).await {
                println!("⚠️  Setup: invalid Telegram token submitted — bot not started");
                "tg_invalid"
            } else {
                let mut task = state.telegram_task.lock().await;
                if let Some(handle) = task.take() {
                    handle.abort();
                }
                let llm_url = state.llm_url.clone();
                let token = new_tg.clone();
                *task = Some(tokio::spawn(async move {
                    run_telegram_bot(token, llm_url).await;
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
