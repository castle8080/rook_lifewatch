#!/usr/bin/env python3
import subprocess
import os
import shutil
import platform

cert_dir = "certs"

def find_openssl():
    # Standard path search
    openssl_exe = shutil.which('openssl')
    if openssl_exe is not None:
        return openssl_exe
    
    # Annoying search on windows
    # Many openssl installs don't add to path on windows.
    if platform.system() == 'Windows':
        for pf_key in ['PROGRAMFILES', 'PROGRAMFILES(X86)', 'PROGRAMW6432']:
            pf_dir = os.environ.get(pf_key)
            if pf_dir:
                for d_ent in os.listdir(pf_dir):
                    if d_ent.lower().startswith("openssl"):
                        openssl_exe = os.path.join(pf_dir, d_ent, "bin", "openssl.exe")
                        if os.path.isfile(openssl_exe):
                            return openssl_exe
    
    return None

def generate_self_signed_certs():
    os.makedirs(cert_dir, exist_ok=True)

    key_file = os.path.join(cert_dir, "key.pem")
    cert_file = os.path.join(cert_dir, "cert.pem")

    if os.path.exists(cert_file) and os.path.exists(key_file):
        print("Certificates already exist. Skipping generation.")
        return

    openssl_exe = find_openssl()
    if openssl_exe is None:
        openssl_exe = "openssl"

    subprocess.run([
        openssl_exe, "req", "-x509", "-nodes", "-days", "365", "-newkey", "rsa:2048",
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