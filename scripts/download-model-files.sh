#!/bin/sh

set -e

download_dir="var/downloads"
mkdir -p "${download_dir}"

model_dir="dist/models"
mkdir -p "${model_dir}"

download_model_file() {
    local model_url="$1"

    local file_name="${model_url##*/}"
    file_name="${file_name%%\?*}"  # Remove query parameters if any
    local full_file_name="${download_dir}/${file_name}"

    if [ ! -f "${full_file_name}" ]; then
        echo "Downloading ${file_name}..."
        curl --fail -L -o "${full_file_name}" "${model_url}"
    else
        echo "${file_name} already exists, skipping download."
    fi

    cp -vf "${full_file_name}" "${model_dir}/"
}

download_urls="
    https://huggingface.co/Kalray/yolov4-tiny/resolve/d4625044dd555c909ad2ee396efa0d85b2ece68a/yolov4-tiny.onnx?download=true
    https://github.com/AlexeyAB/darknet/releases/download/yolov4/yolov4-tiny.weights
    https://raw.githubusercontent.com/AlexeyAB/darknet/master/cfg/yolov4-tiny.cfg
    https://raw.githubusercontent.com/pjreddie/darknet/master/data/coco.names
"

for download_url in $download_urls; do
    download_model_file "${download_url}"
done

echo "Done! Model files:"
ls -lh "${model_dir}"
