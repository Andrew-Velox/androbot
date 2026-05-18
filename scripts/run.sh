#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

if [[ ! -f .env ]]; then
  echo "Missing .env. Run: cp .env.example .env"
  exit 1
fi

cargo run --release
