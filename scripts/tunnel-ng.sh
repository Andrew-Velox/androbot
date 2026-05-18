#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

if ! command -v ngrok >/dev/null 2>&1; then
  echo "ngrok not found."
  read -r -p "Try to install via pkg? (y/N): " ans
  if [[ "$ans" == "y" || "$ans" == "Y" ]]; then
    pkg update -y
    if ! pkg install -y ngrok; then
      echo "pkg install ngrok failed. Install manually: https://ngrok.com/download"
      exit 1
    fi
  else
    echo "Install manually: https://ngrok.com/download"
    exit 1
  fi
fi

if ! command -v ngrok >/dev/null 2>&1; then
  echo "ngrok still not available. Aborting."
  exit 1
fi

echo "Starting ngrok tunnel to http://127.0.0.1:3000"
ngrok http 3000
