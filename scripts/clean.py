#!/usr/bin/env python3
import os
import sys
import subprocess

project_dirs = [
    "rook_lw_libcamera_capture",
    "rook_lw_daemon",
    "rook_lw_admin",
    "rook_lw_admin_fe",
    "rook_lw_models",
    "rook_lw_image_repo",
]

def cmd(dir, *command):
    print(f"Running command: {' '.join(command)}")
    if len(command) >= 0 and command[0].endswith(".py"):
        command = [sys.executable] + list(command)
    subprocess.run(command, cwd=dir, check=True)

def run_project_cleans():
    for project in project_dirs:
        print(f"Cleaning project: {project}")
        cmd(project, "make.py", "clean")

def main():
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    os.chdir(base_dir)
    run_project_cleans()

main()