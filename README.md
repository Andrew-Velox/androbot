# Androbot

Runs a Meta/Telegram webhook server and forwards messages to a local LLM (llama.cpp) or any OpenAI-compatible API.

## Quick Start (Termux)

```bash
git clone https://github.com/Andrew-Velox/androbot.git
cd androbot
./scripts/run.sh
```

First run creates `.env` automatically. Open the setup page to fill in your tokens:

```
http://127.0.0.1:3000/setup
```

## Run (Each Time)

```bash
./scripts/run.sh          # webhook server
./scripts/llm-run.sh      # local LLM (separate session, if using on-device GGUF)
tunnel                    # public HTTPS tunnel (needed for Meta/WhatsApp webhooks)
```

Set your Meta webhook URL to:

```
https://YOUR_TUNNEL_URL/webhook
```

Webhook verify token: `ANDROBOT_VERIFY_TOKEN`

Telegram: starts automatically when `TELEGRAM_BOT_TOKEN` is set in `.env`.

## Local LLM Setup (Optional)

To run a GGUF model on-device with llama.cpp:

```bash
./scripts/llm-setup.sh    # build llama.cpp (one time)
./scripts/llm-run.sh      # start the server
```

Drop your `.gguf` model into the `models/` folder. The largest `.gguf` found there is used automatically.

## Live Data via Database URL

Paste a Postgres URL into `/setup` → "Database URL":

```
postgres://user:pass@host:5432/dbname
```

Androbot auto-introspects the schema and exposes a `query_database` tool to the AI, so it can answer customer questions using real product/order/inventory data.

**Safety guardrails (always on):**

- Only `SELECT` and `WITH` queries — `INSERT`/`UPDATE`/`DELETE`/`DROP`/etc. are rejected at validation time.
- Multi-statement queries are rejected (no `;` in the middle).
- 5 s query timeout, 2 connections max, results capped at 50 rows / 2000 chars.
- For extra safety, connect with a Postgres role that has only `SELECT` grants.

Works with Supabase, Neon, RDS, or any Postgres-compatible DB. Requires `LLM_API_KEY` (local Gemma 2B is too small for reliable tool calls).

## Use a Hosted API Instead

Set these in `.env` or the setup page to skip the local model entirely:

```
LLM_API_BASE_URL=https://api.openai.com/v1
LLM_API_MODEL=gpt-4o-mini
LLM_API_KEY=your-key
```

When `LLM_API_KEY` is set, the local `LLM_URL` is ignored.

## Notes

- Never commit `.env`. If it was ever shared, rotate your tokens immediately.
- The webhook server must be reachable over HTTPS — use the `tunnel` script for this.
