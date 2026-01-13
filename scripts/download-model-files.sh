#!/bin/sh

set -e

model_dir="var/models"
mkdir -p "${model_dir}"

download_model_file() {
    local model_url="$1"

    local file_name="${model_url##*/}"
    local full_file_name="${model_dir}/${file_name}"

    if [ ! -f "${full_file_name}" ]; then
        echo "Downloading ${file_name}..."
        curl --fail -L -o "${full_file_name}" "${model_url}"
    else
        echo "${file_name} already exists, skipping download."
    fi
}

# Download YOLOv4-tiny (lightweight and well-supported by OpenCV)
download_model_file "https://github.com/AlexeyAB/darknet/releases/download/yolov4/yolov4-tiny.weights"

download_model_file "https://raw.githubusercontent.com/AlexeyAB/darknet/master/cfg/yolov4-tiny.cfg"

# Download COCO class names (80 classes)
download_model_file "https://raw.githubusercontent.com/pjreddie/darknet/master/data/coco.names"

echo "Done! Model files:"
ls -lh "${model_dir}"
