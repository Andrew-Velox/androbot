use axum::{extract::Form, response::Html};
use serde::Deserialize;

use crate::config::{DEFAULT_BIND_ADDR, DEFAULT_LLM_URL};
use crate::services::env_store::{read_env_value, write_env_file};

#[derive(Deserialize)]
pub struct SetupForm {
    page_access_token: String,
    telegram_bot_token: Option<String>,
}

pub async fn setup_page() -> Html<String> {
    let page_access_token = read_env_value("PAGE_ACCESS_TOKEN").unwrap_or_default();
    let telegram_bot_token = read_env_value("TELEGRAM_BOT_TOKEN").unwrap_or_default();

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

    .notice {{
      display: flex;
      gap: 0.6rem;
      align-items: flex-start;
      background: #1e2433;
      border: 1px solid #2d3a4a;
      border-radius: 8px;
      padding: 0.75rem 1rem;
      margin-bottom: 1.75rem;
      font-size: 0.82rem;
      color: #7dd3fc;
      line-height: 1.5;
    }}

    .notice-icon {{ flex-shrink: 0; margin-top: 0.05rem; }}

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

    @media (max-width: 480px) {{
      .card {{ padding: 1.75rem 1.25rem; }}
      h1 {{ font-size: 1.3rem; }}
      .subtitle {{ padding-left: 0; }}
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
          type="text"
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
          type="text"
          value="{tg_token}"
          placeholder="123456:ABCdef..."
          spellcheck="false"
          autocomplete="off"
        >
      </div>

      <div class="notice">
        <span class="notice-icon">&#8505;&#65039;</span>
        <span>The Telegram bot is started on server launch. If you add or change the token here, <strong>restart the server</strong> for it to take effect.</span>
      </div>

      <button type="submit">Save Configuration</button>
    </form>
  </div>
</body>
</html>"#,
        page_token = page_access_token,
        tg_token = telegram_bot_token,
    );

    Html(html)
}

pub async fn save_setup(Form(form): Form<SetupForm>) -> Html<String> {
    let llm_url = read_env_value("LLM_URL").unwrap_or_else(|| DEFAULT_LLM_URL.to_string());
    let bind_addr = read_env_value("BIND_ADDR").unwrap_or_else(|| DEFAULT_BIND_ADDR.to_string());

    let _ = write_env_file(
        &form.page_access_token,
        form.telegram_bot_token.as_deref(),
        &llm_url,
        &bind_addr,
    );

    Html(r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Androbot — Saved</title>
  <style>
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      min-height: 100vh;
      display: flex;
      align-items: center;
      justify-content: center;
      background: #0f1117;
      font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
      color: #e2e8f0;
      padding: 1.5rem;
    }
    .card {
      width: 100%;
      max-width: 420px;
      background: #1a1d27;
      border: 1px solid #2d3148;
      border-radius: 16px;
      padding: 2.5rem 2rem;
      text-align: center;
      box-shadow: 0 24px 48px rgba(0,0,0,0.4);
    }
    .icon { font-size: 3rem; margin-bottom: 1rem; }
    h2 { font-size: 1.4rem; font-weight: 700; color: #f8fafc; margin-bottom: 0.5rem; }
    p { color: #64748b; font-size: 0.9rem; line-height: 1.6; margin-bottom: 1.5rem; }
    a {
      display: inline-block;
      padding: 0.7rem 1.5rem;
      background: linear-gradient(135deg, #6366f1, #8b5cf6);
      border-radius: 8px;
      color: #fff;
      font-weight: 600;
      text-decoration: none;
      font-size: 0.95rem;
      transition: opacity 0.2s;
    }
    a:hover { opacity: 0.85; }
  </style>
</head>
<body>
  <div class="card">
    <div class="icon">&#9989;</div>
    <h2>Configuration Saved</h2>
    <p>Your settings have been written.<br>If you changed the Telegram token, restart the server for it to take effect.</p>
    <a href="/setup">Back to Setup</a>
  </div>
</body>
</html>"#.to_string())
}
