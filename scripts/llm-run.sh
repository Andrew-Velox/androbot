#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_dir="$(dirname "$script_dir")"
repo_dir="$project_dir/llama.cpp"
models_dir="$project_dir/models"

if [[ ! -x "$repo_dir/build/bin/llama-server" ]]; then
  echo "llama-server not built. Run: ./scripts/llm-setup.sh"
  exit 1
fi

model_file=$(find "$models_dir" "$repo_dir/models" -maxdepth 2 -type f -name "*.gguf" \
  ! -name "ggml-vocab-*" -printf "%s %p\n" 2>/dev/null | sort -nr | head -n1 | awk '{print $2}')

if [[ -z "$model_file" ]]; then
  read -r -p "Path to model (.gguf): " model_file
fi

if [[ ! -f "$model_file" ]]; then
  echo "Model not found: $model_file"
  exit 1
fi

host=${LLM_HOST:-0.0.0.0}
port=${LLM_PORT:-8080}

echo "Model : $model_file"
echo "URL   : http://127.0.0.1:${port}"

exec "$repo_dir/build/bin/llama-server" \
  -m "$model_file" \
  --host "$host" \
  --port "$port" \
  --cache-ram 256
