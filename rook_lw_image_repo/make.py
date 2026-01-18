#!/bin/python3
import sys
import subprocess

def clean():
    subprocess.run(["cargo", "clean"], check=True)

def test():
    subprocess.run(["cargo", "test", "--release"], check=True)

def build():
    subprocess.run(["cargo", "build", "--release"], check=True)

def install():
    build()
    # No installation steps defined.
    pass

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
