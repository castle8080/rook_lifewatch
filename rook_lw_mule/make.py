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
    subprocess.run(["cargo", "test"], check=True)

def build():
    subprocess.run(["cargo", "tauri", "build"], check=True)

def run():
    subprocess.run([
        "cargo", "tauri", "dev"
    ], check=True)

def install():
    build()

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
