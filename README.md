# Androbot (Termux Friendly)

This project runs a Meta webhook server and forwards messages to a local LLM HTTP server (llama.cpp).

Repository: https://github.com/Andrew-Velox/androbot

## Quick Start (Termux)

1) Clone the repo:

```bash
git clone https://github.com/Andrew-Velox/androbot.git
cd androbot
```

2) Setup dependencies and build:

```bash
./scripts/setup.sh
```

3) (Optional) Install llama.cpp and download a model:

```bash
./scripts/llm-setup.sh
```

4) Create your env file (interactive):

```bash
./scripts/config.sh
```

The script only requires `PAGE_ACCESS_TOKEN`. It will reuse existing values and keep sensible defaults for `LLM_URL` and `BIND_ADDR`.

Webhook verify token (fixed):

```
ANDROBOT_VERIFY_TOKEN
```

5) Run the services in separate Termux sessions:

Terminal A (llama.cpp):

```bash
./scripts/llm-run.sh
```

Terminal B (tunnel):

```bash
./scripts/tunnel-cf.sh
```

or

```bash
./scripts/tunnel-ng.sh
```

Terminal C (webhook server):

```bash
./scripts/run.sh
```

Tip: start the webhook server first, then start the tunnel. This avoids brief 502s while the tunnel connects.

Copy the HTTPS URL from the tunnel output and set your Meta webhook URL to:

```
https://YOUR_TUNNEL_URL/webhook
```

## Notes

- Never commit `.env`. If it was ever committed or shared, rotate tokens immediately.
- The webhook must be publicly reachable over HTTPS. The tunnel scripts provide this.
- The LLM server can run on the same device (Termux) or another host; update `LLM_URL` accordingly.
