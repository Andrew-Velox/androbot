#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

pkg update -y
pkg install -y rust clang cmake pkg-config git

cargo build --release

echo "Done. Next: cp .env.example .env and run ./scripts/run-termux.sh"
