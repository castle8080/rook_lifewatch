#!/bin/python3
import re
import os
import platform
import tarfile
import shutil
from urllib.request import urlopen
from urllib.parse import urlparse

download_dir = os.path.join("var", "downloads")
lib_dir = os.path.join("dist", "lib")

def download(url, dest_file, timeout=10):
    if os.path.isfile(dest_file):
        print(f"File already exists: {dest_file}")
        return

    tmp_file = dest_file + ".part"
    with urlopen(url, timeout=timeout) as resp:
        if resp.status != 200:
            raise RuntimeError(f"HTTP {resp.status} for {url}")

        try:
            with open(tmp_file, "wb") as f:
                for chunk in iter(lambda: resp.read(1024 * 8), b""):
                    f.write(chunk)

            os.replace(tmp_file, dest_file)  # atomic on POSIX + Windows
        except Exception:
            os.unlink(tmp_file)
            raise

def get_onnx_runtime_url():
    system = platform.system()
    machine = platform.machine()

    if system == "Linux" and machine == "x86_64":
        return "https://sourceforge.net/projects/onnx-runtime.mirror/files/v1.23.2/onnxruntime-linux-x64-1.23.2.tgz/download"
    elif system == "Linux" and machine == "aarch64":
        return "https://sourceforge.net/projects/onnx-runtime.mirror/files/v1.23.2/onnxruntime-linux-aarch64-1.23.2.tgz/download"
    else:
        raise RuntimeError(f"Unsupported platform: {system} {machine}")
    
def get_onnx_runtime_filename():
    url = get_onnx_runtime_url()
    path = urlparse(url).path
    path_parts = re.split(r'[\\/]+', path)

    for i in range(len(path_parts)-1, -1, -1):
        file_name = path_parts[i]
        if file_name != '' and file_name != 'download':
            return file_name

    raise RuntimeError(f"Could not determine filename from URL: {url}")

def extract_onnx_runtime(tar_path, extract_dir):
    # Open tar file detecting compression and in streaming mode.
    with tarfile.open(tar_path, "r|*") as tf:
        for m in tf:
            base_name = os.path.basename(m.name)
            if base_name.startswith("libonnxruntime.so") and m.isfile():
                simple_name = "libonnxruntime.so"
                dest_path = os.path.join(lib_dir, simple_name)
                print(f"Name: {base_name}, Extracting to: {dest_path}")
                with tf.extractfile(m) as fsrc, open(dest_path, "wb") as fdst:
                    shutil.copyfileobj(fsrc, fdst)
                    return

    raise RuntimeError("libonnxruntime.so not found in the ONNX Runtime tarball")

def main():
    url = get_onnx_runtime_url()
    filename = get_onnx_runtime_filename()
    os.makedirs(download_dir, exist_ok=True)
    download_path = os.path.join(download_dir, filename)
    print(f"Downloading ONNX Runtime from {url}")
    download(url, download_path)
    print(f"ONNX Runtime downloaded to {download_path}")
    extract_onnx_runtime(download_path, lib_dir)

main()