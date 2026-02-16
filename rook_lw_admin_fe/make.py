#!/bin/python3
import sys
import subprocess
import shutil
import os

install_dir = "../dist"
target_dir = os.path.join(install_dir, "www", "admin")

# trunk puts build files here.
build_dir = "dist"

def clean():
    subprocess.run(["cargo", "clean"], check=True)

def test():
    build()
    # Run native Rust tests
    subprocess.run(["cargo", "test", "--release"], check=True)
    # Run WASM tests in headless Firefox
    subprocess.run(["wasm-pack", "test", "--headless", "--firefox"], check=True)

# Makes development setup by installing trunk and rust targets.
def init_dev():
    subprocess.run(["cargo", "install", "trunk"], check=True)
    subprocess.run(["cargo", "install", "wasm-pack"], check=True)
    subprocess.run(["rustup", "target", "add", "wasm32-unknown-unknown"], check=True)

def run():
    init_dev()
    proxy_endpoint = os.environ.get("ROOK_LW_PROXY", "http://localhost:8080/api")
    print(f"Proxying to: {proxy_endpoint}")
    subprocess.run([
        "trunk", "serve", "--port", "8081", "--public-url", "/admin/",
        f"--proxy-backend={proxy_endpoint}"
    ], check=True)

def build():
    init_dev()
    subprocess.run(["trunk", "build", "--release", "--public-url", "/admin/"], check=True)

def install():
    if os.path.isdir(target_dir):
        shutil.rmtree(target_dir)

    os.makedirs(target_dir, exist_ok=True)

    print("Copying built files from", build_dir, "to", target_dir)
    shutil.copytree(build_dir, target_dir, dirs_exist_ok=True)

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
        raise
        sys.exit(1)
