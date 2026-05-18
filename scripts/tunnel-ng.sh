#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

if ! command -v ngrok >/dev/null 2>&1; then
  echo "ngrok not found. Install from https://ngrok.com/download"
  exit 1
fi

echo "Starting ngrok tunnel to http://127.0.0.1:3000"
ngrok http 3000
