#!/bin/python3
import sys
import subprocess
import shutil
import os

def clean():
    shutil.rmtree("var")

def build():
    # TODO: Could invoke model build.
    pass

def install():
    os.makedirs("../dist/models", exist_ok=True)
    for f in os.listdir("models"):
        print(f"Copying: {f}")
        shutil.copy(os.path.join("models", f), "../dist/models")

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
