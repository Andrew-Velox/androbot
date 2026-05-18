#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

if ! command -v cloudflared >/dev/null 2>&1; then
  echo "cloudflared not found."
  read -r -p "Install cloudflared now? (y/N): " ans
  if [[ "$ans" == "y" || "$ans" == "Y" ]]; then
    pkg update -y
    pkg install -y cloudflared
  else
    echo "Install it first: pkg install -y cloudflared"
    exit 1
  fi
fi

if ! command -v cloudflared >/dev/null 2>&1; then
  echo "cloudflared still not available. Aborting."
  exit 1
fi

echo "Starting cloudflared tunnel to http://127.0.0.1:3000"
cloudflared tunnel --url http://127.0.0.1:3000
