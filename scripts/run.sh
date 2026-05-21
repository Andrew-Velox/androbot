#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

if [[ ! -f .env ]]; then
  cat > .env <<EOF
PAGE_ACCESS_TOKEN=
LLM_URL=http://127.0.0.1:8080/v1/chat/completions
BIND_ADDR=0.0.0.0:3000
LLM_API_BASE_URL=
LLM_API_MODEL=
LLM_API_KEY=
EOF
  echo "Created .env with defaults. Open /setup to add tokens."
fi

cargo run --release
