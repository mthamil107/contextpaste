#!/bin/bash
# Download ONNX embedding model for ContextPaste
# Model: all-MiniLM-L6-v2 (384-dim, ~80MB)

set -e

MODEL_DIR="src-tauri/models"
MODEL_FILE="$MODEL_DIR/all-MiniLM-L6-v2.onnx"
MODEL_URL="https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx"

mkdir -p "$MODEL_DIR"

if [ -f "$MODEL_FILE" ]; then
    echo "Model already exists at $MODEL_FILE"
    exit 0
fi

echo "Downloading all-MiniLM-L6-v2 ONNX model..."
echo "This is ~80MB and only needs to be downloaded once."
echo ""

curl -L -o "$MODEL_FILE" "$MODEL_URL" --progress-bar

if [ -f "$MODEL_FILE" ]; then
    SIZE=$(du -h "$MODEL_FILE" | cut -f1)
    echo ""
    echo "Download complete: $MODEL_FILE ($SIZE)"
else
    echo "ERROR: Download failed"
    exit 1
fi
