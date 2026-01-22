#!/bin/python3
import os
import sys
import subprocess
import re
import shutil

project_dirs = [
    "rook_lw_libcamera_capture",
    "rook_lw_daemon",
    "rook_lw_admin",
    "rook_lw_admin_fe",
]

doownlad_scripts = [
    "scripts/download-model-files.py",
    "scripts/install-onnxruntime.py",
]

install_script_re = re.compile(r"(start_rook_lw|stop_rook_lw|run_daemon|gen_self_signed_cert).*\.(py|sh|cmd)$")

def cmd(dir, *command):
    print(f"Running command: {' '.join(command)}")
    if len(command) >= 0 and command[0].endswith(".py"):
        command = [sys.executable] + list(command)
    subprocess.run(command, cwd=dir, check=True)

def build_install():
    for project in project_dirs:
        print(f"Building project: {project}")
        cmd(project, "make.py", "build")
        print(f"Installing project: {project}")
        cmd(project, "make.py", "install")

def download_resources():
    for script in doownlad_scripts:
        print(f"Running download script: {script}")
        cmd(".", script)

def install_scripts():
    for f in os.listdir("scripts"):
        if install_script_re.match(f):
            print(f"Installing script: {f}")
            shutil.copy(os.path.join("scripts", f), os.path.join("dist", "bin", f))

def main():
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    os.chdir(base_dir)
    build_install()
    download_resources()
    install_scripts()

main()
