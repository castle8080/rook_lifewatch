#!/bin/sh

set -e

# Check OS and architecture
arch=$(uname  -m)
echo "Detected architecture: $arch"

download_url=""
file_name=""
download_dir="var/downloads"

if [ "$(uname -s)" != "Linux" ]; then
    echo "[Warning] Not running on Linux"
    exit 0
fi

if [ "$arch" = "x86_64" ]; then
    download_url="https://sourceforge.net/projects/onnx-runtime.mirror/files/v1.23.2/onnxruntime-linux-x64-1.23.2.tgz/download"
    file_name="onnxruntime-linux-x64-1.23.2.tgz"
elif [ "$arch" = "aarch64" ]; then
    download_url="https://sourceforge.net/projects/onnx-runtime.mirror/files/v1.23.2/onnxruntime-linux-aarch64-1.23.2.tgz/download"
    file_name="onnxruntime-linux-aarch64-1.23.2.tgz"
else
    echo "[Warning] Unsupported architecture: $arch"
    exit 0
fi

# Download the appropriate ONNX Runtime package
mkdir -p "$download_dir"

download_path="$download_dir/$file_name"
if [ ! -f "$download_path" ]; then
    curl --fail -L -o "$download_path" "$download_url"
fi

# Extract the package
unpack_dir="var/tmp/onnxruntime"
if [ -d "$unpack_dir" ]; then
    rm -rf "$unpack_dir"
fi
mkdir -p "$unpack_dir"
tar -xzf "$download_path" -C "$unpack_dir" --wildcards '*/lib/libonnxruntime.so*' 

# find the library
onnxruntime_lib=$(find "$unpack_dir" -name "libonnxruntime.so" | head -1)
if [ -z "$onnxruntime_lib" ]; then
    echo "[Error] ONNX Runtime library not found after extraction"
    exit 1
fi

# Copy the library to the dist/lib directory
mkdir -p "dist/lib"
cp -vf "$onnxruntime_lib" "dist/lib/"

# Clean up
rm -rf "$unpack_dir"