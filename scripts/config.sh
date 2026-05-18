#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

default_llm_url="http://127.0.0.1:8080/v1/chat/completions"
default_bind_addr="0.0.0.0:3000"

# Reuse existing .env values if present
if [[ -f .env ]]; then
	set -a
	# shellcheck disable=SC1091
	source .env
	set +a
fi

page_access_token=${PAGE_ACCESS_TOKEN:-}
llm_url=${LLM_URL:-$default_llm_url}
bind_addr=${BIND_ADDR:-$default_bind_addr}

if [[ -z "$page_access_token" ]]; then
	read -r -p "PAGE_ACCESS_TOKEN: " page_access_token
fi

cat > .env <<EOF
PAGE_ACCESS_TOKEN=${page_access_token}
LLM_URL=${llm_url}
BIND_ADDR=${bind_addr}
EOF

echo "Wrote .env"
