#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

# Canonical paths — always resolved relative to this script's parent directory
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_dir="$(dirname "$script_dir")"
repo_dir="$project_dir/llama.cpp"
models_dir="$project_dir/models"

# ── Dependencies & build ───────────────────────────────────────────────────────
pkg update -y
pkg install -y git clang cmake make wget

if [[ ! -d "$repo_dir" ]]; then
  git clone https://github.com/ggerganov/llama.cpp "$repo_dir"
fi

cmake -S "$repo_dir" -B "$repo_dir/build" -DLLAMA_BUILD_SERVER=ON
cmake --build "$repo_dir/build" --config Release -j "$(nproc)"

mkdir -p "$models_dir"

# ── Model catalog ──────────────────────────────────────────────────────────────
MODEL_NAMES=(
  "Qwen 2.5  0.5B  Q4_K_M  (~400 MB)  ← default, fastest"
  "Qwen 2.5  1.5B  Q4_K_M  (~1.0 GB)"
  "Qwen 2.5  3B    Q4_K_M  (~1.9 GB)"
  "Llama 3.2 1B    Q4_K_M  (~700 MB)"
  "Llama 3.2 3B    Q4_K_M  (~1.9 GB)"
  "Phi 3.5 Mini    Q4_K_M  (~2.4 GB)"
  "Gemma 2   2B    Q4_K_M  (~1.6 GB)"
  "SmolLM2   1.7B  Q4_K_M  (~1.1 GB)"
  "Mistral   7B    Q4_K_M  (~4.1 GB)"
  "DeepSeek-R1 1.5B Q4_K_M (~1.0 GB)"
)

MODEL_URLS=(
  "https://huggingface.co/bartowski/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/Qwen2.5-0.5B-Instruct-Q4_K_M.gguf"
  "https://huggingface.co/bartowski/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/Qwen2.5-1.5B-Instruct-Q4_K_M.gguf"
  "https://huggingface.co/bartowski/Qwen2.5-3B-Instruct-GGUF/resolve/main/Qwen2.5-3B-Instruct-Q4_K_M.gguf"
  "https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF/resolve/main/Llama-3.2-1B-Instruct-Q4_K_M.gguf"
  "https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF/resolve/main/Llama-3.2-3B-Instruct-Q4_K_M.gguf"
  "https://huggingface.co/bartowski/Phi-3.5-mini-instruct-GGUF/resolve/main/Phi-3.5-mini-instruct-Q4_K_M.gguf"
  "https://huggingface.co/bartowski/gemma-2-2b-it-GGUF/resolve/main/gemma-2-2b-it-Q4_K_M.gguf"
  "https://huggingface.co/bartowski/SmolLM2-1.7B-Instruct-GGUF/resolve/main/SmolLM2-1.7B-Instruct-Q4_K_M.gguf"
  "https://huggingface.co/bartowski/Mistral-7B-Instruct-v0.3-GGUF/resolve/main/Mistral-7B-Instruct-v0.3-Q4_K_M.gguf"
  "https://huggingface.co/bartowski/DeepSeek-R1-Distill-Qwen-1.5B-GGUF/resolve/main/DeepSeek-R1-Distill-Qwen-1.5B-Q4_K_M.gguf"
)

# ── Model picker ───────────────────────────────────────────────────────────────
echo ""
echo "  ┌──────────────────────────────────────────────────────┐"
echo "  │              Select a model to install               │"
echo "  ├────┬─────────────────────────────────────────────────┤"
for i in "${!MODEL_NAMES[@]}"; do
  printf "  │ %2d │ %s\n" "$((i + 1))" "${MODEL_NAMES[$i]}"
done
echo "  ├────┴─────────────────────────────────────────────────┤"
echo "  │  Enter a number, a direct .gguf URL, or press Enter  │"
echo "  │  with no input to install the default (1).           │"
echo "  └──────────────────────────────────────────────────────┘"
echo ""
read -r -p "  Choice: " choice
echo ""

count=${#MODEL_URLS[@]}

if [[ -z "$choice" ]]; then
  selected_url="${MODEL_URLS[0]}"
  selected_name="${MODEL_NAMES[0]}"
elif [[ "$choice" =~ ^[0-9]+$ ]] && (( choice >= 1 && choice <= count )); then
  idx=$((choice - 1))
  selected_url="${MODEL_URLS[$idx]}"
  selected_name="${MODEL_NAMES[$idx]}"
elif [[ "$choice" =~ ^https?:// && "$choice" == *.gguf ]]; then
  selected_url="$choice"
  selected_name="Custom ($choice)"
else
  echo "  Invalid input — falling back to default."
  selected_url="${MODEL_URLS[0]}"
  selected_name="${MODEL_NAMES[0]}"
fi

echo "  Model : $selected_name"

# Remove all previously downloaded models across known locations.
# Excludes ggml-vocab-* which are llama.cpp's own internal files.
echo "  Removing old model files..."
find "$HOME" -maxdepth 6 -type f -name "*.gguf" \
  ! -name "ggml-vocab-*" \
  -delete 2>/dev/null || true
echo "  Done."

file_name=$(basename "$selected_url")
echo "  Downloading $file_name ..."
echo ""
wget --show-progress -O "$models_dir/$file_name" "$selected_url"

echo ""
echo "  ✅ Installed: $models_dir/$file_name"
echo "  Next: ./scripts/llm-run.sh"
echo ""
