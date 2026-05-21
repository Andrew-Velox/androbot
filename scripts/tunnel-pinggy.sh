#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

# Pinggy.io free SSH tunnel — works on networks that block Cloudflare/ngrok.
# Uses SSH over port 443, so it looks like normal HTTPS traffic to your carrier.
# Free tier: 60-minute sessions, anonymous (no signup).

if ! command -v ssh >/dev/null 2>&1; then
  echo "ssh not found. Install: pkg install -y openssh"
  exit 1
fi

PORT="${PORT:-3000}"

echo "Starting Pinggy SSH tunnel to http://127.0.0.1:${PORT}"
echo "Your public HTTPS URL will appear below in a few seconds."
echo ""

ssh -p 443 \
  -o ServerAliveInterval=30 \
  -o StrictHostKeyChecking=no \
  -o UserKnownHostsFile=/dev/null \
  -R "0:localhost:${PORT}" \
  a.pinggy.io
