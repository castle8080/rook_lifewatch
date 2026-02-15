#!/bin/bash
set -e

# Change to script directory
cd "$(dirname "$0")"

# Setup Python environment
source ./setup-env.sh

# Execute notebook with visible output
echo "Executing export_yolov8_with_embeddings.ipynb..."
jupyter nbconvert --to notebook --execute --inplace \
    --ExecutePreprocessor.timeout=600 \
    export_yolov8_with_embeddings.ipynb

jupyter nbconvert --to notebook --execute --inplace \
    --ExecutePreprocessor.timeout=600 \
    download_prebuilt_models.ipynb

echo "✓ Model export complete"
