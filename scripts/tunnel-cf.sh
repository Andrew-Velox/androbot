#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

if ! command -v cloudflared >/dev/null 2>&1; then
  echo "cloudflared not found. Try: pkg install -y cloudflared"
  exit 1
fi

echo "Starting cloudflared tunnel to http://127.0.0.1:3000"
cloudflared tunnel --url http://127.0.0.1:3000
