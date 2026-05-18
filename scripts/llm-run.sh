#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

repo_dir="llama.cpp"
models_dir="models"

if [[ ! -x "$repo_dir/build/bin/llama-server" ]]; then
  echo "llama-server not built. Run: ./scripts/llm-setup.sh"
  exit 1
fi

model_file=""
if ls "$models_dir"/*.gguf >/dev/null 2>&1; then
  model_file=$(ls "$models_dir"/*.gguf | head -n 1)
fi

if [[ -z "$model_file" ]]; then
  read -r -p "Path to model (.gguf): " model_file
fi

if [[ ! -f "$model_file" ]]; then
  echo "Model not found: $model_file"
  exit 1
fi

host=${LLM_HOST:-0.0.0.0}
port=${LLM_PORT:-8080}

"$repo_dir/build/bin/llama-server" -m "$model_file" --host "$host" --port "$port"
