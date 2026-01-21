#!/usr/bin/env python3
import subprocess
import os

cert_dir = "certs"

def generate_self_signed_certs():
    os.makedirs(cert_dir, exist_ok=True)

    key_file = os.path.join(cert_dir, "key.pem")
    cert_file = os.path.join(cert_dir, "cert.pem")

    if os.path.exists(cert_file) and os.path.exists(key_file):
        print("Certificates already exist. Skipping generation.")
        return

    subprocess.run([
        "openssl", "req", "-x509", "-nodes", "-days", "365", "-newkey", "rsa:2048",
        "-keyout", key_file, "-out", cert_file,
        "-subj", "/C=US/ST=State/L=City/O=Org/OU=Unit/CN=localhost"
    ], check=True)

    print(f"Self-signed certificate generated: {cert_file}, {key_file}")

def main():
    project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    print(f"Project root directory: {project_root}")
    os.chdir(project_root)
    generate_self_signed_certs()

if __name__ == "__main__":
    main()