#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

repo_dir="llama.cpp"
models_dir="models"

if [[ ! -x "$repo_dir/build/bin/llama-server" ]]; then
  echo "llama-server not built. Run: ./scripts/llm-setup.sh"
  exit 1
fi

model_file=""

search_paths=(
  "$models_dir"
  "$repo_dir/models"
  "$HOME/llama.cpp"
  "$HOME/llama.cpp/models"
)

for dir in "${search_paths[@]}"; do
  if [[ -d "$dir" ]]; then
    model_file=$(find "$dir" -maxdepth 2 -type f -name "*.gguf" \
      ! -name "ggml-vocab-*" -printf "%s %p\n" 2>/dev/null | \
      sort -nr | head -n 1 | awk '{print $2}')
    if [[ -n "$model_file" ]]; then
      break
    fi
  fi
done

if [[ -z "$model_file" ]]; then
  read -r -p "Path to model (.gguf): " model_file
fi

if [[ ! -f "$model_file" ]]; then
  echo "Model not found: $model_file"
  exit 1
fi

echo "Using model: $model_file"

host=${LLM_HOST:-0.0.0.0}
port=${LLM_PORT:-8080}

local_url="http://127.0.0.1:${port}"
lan_ip=$(ip route get 1.1.1.1 2>/dev/null | awk '/src/ {print $7}' | head -n 1)
if [[ -z "$lan_ip" ]]; then
  lan_ip=$(ip -4 addr show 2>/dev/null | awk '/inet / {print $2}' | cut -d/ -f1 | grep -v '^127\.' | head -n 1)
fi
if [[ -z "$lan_ip" ]]; then
  lan_ip=$(hostname -I 2>/dev/null | awk '{print $1}')
fi
if [[ -n "$lan_ip" ]]; then
  lan_url="http://${lan_ip}:${port}"
  echo "LLM will listen on: ${local_url} (local), ${lan_url} (LAN)"
else
  echo "LLM will listen on: ${local_url} (local)"
fi

"$repo_dir/build/bin/llama-server" -m "$model_file" --host "$host" --port "$port"
