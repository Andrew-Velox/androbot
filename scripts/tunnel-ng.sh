#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

NGROK_BIN="$HOME/.local/bin/ngrok"
mkdir -p "$HOME/.local/bin"

if ! command -v ngrok >/dev/null 2>&1 && [[ ! -x "$NGROK_BIN" ]]; then
  echo "ngrok not found. Downloading ARM64 binary..."
  TMP=$(mktemp -d)
  curl -fL "https://bin.equinox.io/c/bNyj1mQVY4c/ngrok-v3-stable-linux-arm64.tgz" -o "$TMP/ngrok.tgz"
  tar -xzf "$TMP/ngrok.tgz" -C "$TMP"
  mv "$TMP/ngrok" "$NGROK_BIN"
  chmod +x "$NGROK_BIN"
  rm -rf "$TMP"
  echo "ngrok installed to $NGROK_BIN"
fi

export PATH="$HOME/.local/bin:$PATH"

if ! command -v ngrok >/dev/null 2>&1; then
  echo "ngrok still not available. Aborting."
  exit 1
fi

echo "Starting ngrok tunnel to http://127.0.0.1:3000"
echo "Note: free ngrok requires a free account — sign up at https://ngrok.com and run: ngrok authtoken YOUR_TOKEN"
echo ""
ngrok http 3000
