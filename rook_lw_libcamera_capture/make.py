#!/bin/python3
import sys
import subprocess
import shutil
import os

install_dir = "../dist"

def clean():
    subprocess.run(["cmake", "--build", "--preset", "default", "--target", "clean"], check=True)

def test():
    build()
    pass  # No test steps defined.

def build():
    subprocess.run(["cmake", "--preset", "default"], check=True)
    subprocess.run(["cmake", "--build", "--preset", "default"], check=True)

def install():
    build()
    subprocess.run(["cmake", "--install", "build/default", "--prefix", install_dir], check=True)

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
