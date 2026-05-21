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
