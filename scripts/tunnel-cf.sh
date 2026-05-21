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

echo "cloudflared $(cloudflared --version 2>&1 | head -n1)"
echo ""

# Diagnostic: check Cloudflare API reachability before starting tunnel
echo "Checking connectivity to api.trycloudflare.com..."
if ! curl -sS -o /dev/null -w "  HTTP %{http_code} in %{time_total}s\n" --max-time 8 https://api.trycloudflare.com/ 2>&1; then
  echo "  ✗ Cannot reach api.trycloudflare.com — your network may be blocking it."
  echo "    Try: switch WiFi/mobile data, use a VPN, or run ./scripts/tunnel-ng.sh"
fi
echo ""

# Try multiple flag combinations — the bare command often fails with 'unexpected EOF'
# when the network blocks QUIC or IPv6 paths to Cloudflare's edge.
attempts=(
  "--protocol http2 --edge-ip-version 4"
  "--protocol http2"
  "--edge-ip-version 4"
  ""
)

for flags in "${attempts[@]}"; do
  echo "Starting tunnel (flags: ${flags:-default})..."
  # shellcheck disable=SC2086
  if cloudflared tunnel $flags --url http://127.0.0.1:3000; then
    exit 0
  fi
  echo ""
  echo "→ attempt failed, trying next flag combination in 2s..."
  sleep 2
done

echo ""
echo "All cloudflared attempts failed."
echo "Options:"
echo "  1) pkg upgrade cloudflared    # may be an outdated version"
echo "  2) ./scripts/tunnel-ng.sh     # use ngrok instead"
echo "  3) Check if your carrier/WiFi blocks api.trycloudflare.com"
exit 1
