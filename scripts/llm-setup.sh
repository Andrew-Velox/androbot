#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

repo_dir="llama.cpp"
models_dir="models"

pkg update -y
pkg install -y git clang cmake make wget

if [[ ! -d "$repo_dir" ]]; then
  git clone https://github.com/ggerganov/llama.cpp "$repo_dir"
fi

cmake -S "$repo_dir" -B "$repo_dir/build" -DLLAMA_BUILD_SERVER=ON
cmake --build "$repo_dir/build" --config Release

mkdir -p "$models_dir"

if ! ls "$models_dir"/*.gguf >/dev/null 2>&1; then
  read -r -p "Model URL (.gguf) to download (leave blank to skip): " model_url
  if [[ -n "$model_url" ]]; then
    file_name=$(basename "$model_url")
    wget -O "$models_dir/$file_name" "$model_url"
  fi
fi

echo "LLM setup complete. Next: ./scripts/llm-run.sh"
