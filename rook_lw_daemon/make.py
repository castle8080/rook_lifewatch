#!/bin/python3
import sys
import subprocess
import shutil
import os

install_dir = "../dist"
bin_dir = os.path.join(install_dir, "bin")
build_dir = "target/release"

def clean():
    subprocess.run(["cargo", "clean"], check=True)

def test():
    subprocess.run(["cargo", "test", "--release", "--features", "libcamera"], check=True)

def build():
    subprocess.run(["cargo", "build", "--release", "--features", "libcamera"], check=True)

def _install_executables():
    os.makedirs(bin_dir, exist_ok=True)

    # Install all executable files from build_dir to bin_dir
    for file in os.listdir(build_dir):
        build_file_path = os.path.join(build_dir, file)
        file_path = os.path.join(bin_dir, file)
        if os.path.isfile(build_file_path) and os.access(build_file_path, os.X_OK):
            print(f"Installing {file} to {file_path}")
            shutil.copy(build_file_path, file_path)

def install():
    build()
    _install_executables()

if __name__ == "__main__":
    try:
        main = sys.modules["__main__"]
        if len(sys.argv) <= 1:
            targets = ["build"]
        else:
            targets = sys.argv[1:]

        for target in targets:
            getattr(main, target)()
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
