# Androbot (Termux Friendly)

This project runs a Meta webhook server and forwards messages to a local LLM HTTP server (llama.cpp).


## First-Time Setup (Termux)

1) Clone the repo:

```bash
git clone https://github.com/Andrew-Velox/androbot.git
cd androbot
```

2) Setup dependencies and build:

```bash
./scripts/setup.sh
```

3) Install llama.cpp and download a model:

```bash
./scripts/llm-setup.sh
```

4) Create your env file (interactive):

```bash
./scripts/config.sh
```

The script only requires `PAGE_ACCESS_TOKEN`. It will reuse existing values and keep sensible defaults for `LLM_URL` and `BIND_ADDR`.

Optional web setup (after the server is running):

```
http://127.0.0.1:3000/setup
```

This page lets you set `PAGE_ACCESS_TOKEN` and `TELEGRAM_BOT_TOKEN` without editing files.

Optional: add Telegram by setting `TELEGRAM_BOT_TOKEN` in `.env` (from @BotFather). If set, the bot runs alongside Facebook and uses the same LLM.

Webhook verify token (fixed):

```
ANDROBOT_VERIFY_TOKEN
```

## Run (Each Time)

1) Run the services in separate Termux sessions:

Terminal A (llama.cpp [local LLM server]):

```bash
./scripts/llm-run.sh
```

Terminal B (tunnel [http server]):

```bash
./scripts/tunnel-cf.sh
```

<!-- or

```bash
./scripts/tunnel-ng.sh
``` -->

Terminal C (webhook server):

```bash
./scripts/run.sh
```

Tip: start the webhook server first, then start the tunnel. This avoids brief 502s while the tunnel connects.

Copy the HTTPS URL from the tunnel output and set your Meta webhook URL to:

```
https://YOUR_TUNNEL_URL/webhook
```

Telegram note: if `TELEGRAM_BOT_TOKEN` is set, the Telegram bot will start automatically when you run `./scripts/run.sh`.

## Notes

- Never commit `.env`. If it was ever committed or shared, rotate tokens immediately.
- The webhook must be publicly reachable over HTTPS. The tunnel scripts provide this.
- The LLM server can run on the same device (Termux) or another host; update `LLM_URL` accordingly.
