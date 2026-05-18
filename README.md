# Androbot (Termux Friendly)

This project runs a Meta webhook server and forwards messages to a local LLM HTTP server (llama.cpp).

## Quick Start (Termux)

1) Setup dependencies and build:

```bash
./scripts/setup.sh
```

2) (Optional) Install llama.cpp and download a model:

```bash
./scripts/llm-setup.sh
```

3) Create your env file (interactive):

```bash
./scripts/config.sh
```

The script only requires `PAGE_ACCESS_TOKEN`. It will reuse existing values, auto-generate `VERIFY_TOKEN` if blank, and keep sensible defaults for `LLM_URL` and `BIND_ADDR`.

4) Run the services in separate Termux sessions:

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

Copy the HTTPS URL from the tunnel output and set your Meta webhook URL to:

```
https://YOUR_TUNNEL_URL/webhook
```

## Notes

- Never commit `.env`. If it was ever committed or shared, rotate tokens immediately.
- The webhook must be publicly reachable over HTTPS. The tunnel scripts provide this.
- The LLM server can run on the same device (Termux) or another host; update `LLM_URL` accordingly.
