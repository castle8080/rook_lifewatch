#!/bin/python3
import os
from urllib.request import urlopen
from urllib.parse import urlparse

download_dir = os.path.join("var", "downloads")
model_dir = os.path.join("dist", "models")

model_files = [
    "https://huggingface.co/Kalray/yolov4-tiny/resolve/d4625044dd555c909ad2ee396efa0d85b2ece68a/yolov4-tiny.onnx?download=true",
    "https://github.com/AlexeyAB/darknet/releases/download/yolov4/yolov4-tiny.weights",
    "https://raw.githubusercontent.com/AlexeyAB/darknet/master/cfg/yolov4-tiny.cfg",
    "https://raw.githubusercontent.com/pjreddie/darknet/master/data/coco.names"
]

def download(url, dest_file, timeout=10):
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

def download_url_cached(url):
    os.makedirs(download_dir, exist_ok=True)

    path = urlparse(url).path
    file_name = os.path.basename(path)
    download_file = os.path.join(download_dir, file_name)

    if os.path.isfile(download_file):
        print(f"Using cached file {download_file}")
        return download_file
    
    download(url, download_file)
    return download_file

def download_install(url):
    download_file = download_url_cached(url)
    os.makedirs(model_dir, exist_ok=True)
    dest_file = os.path.join(model_dir, os.path.basename(download_file))
    if not os.path.isfile(dest_file):
        print(f"Installing model file to {dest_file}")
        os.link(download_file, dest_file)
    else:
        print(f"Model file already installed at {dest_file}")

def main():
    for url in model_files:
        print(f"Processing URL: {url}")
        download_install(url)

if __name__ == "__main__":
    main()