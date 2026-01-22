#!/bin/python3
import sys
import subprocess
import shutil
import os
import platform

install_dir = "../dist"
bin_dir = os.path.join(install_dir, "bin")
build_dir = "target/release"

windows_opencv_dir = os.environ.get('OpenCV_DIR', 'C:\\opencv\\build')

def _setup_opencv():
    if platform.system() == 'Windows':
        # Download binaries from: https://opencv.org/releases/
        # and place them under C:\\opencv
        
        # Need to setup environment variables telling well opencv is.
        if not os.path.isdir(windows_opencv_dir):
            raise Exception(f"No opencv build directory found.")
        
        # Get include dir
        opencv_inc_dir = os.path.join(windows_opencv_dir, 'include')
        
        opencv_lib_dir = None
        opencv_bin_dir = None
        print(windows_opencv_dir)
        for f_ent in os.listdir(os.path.join(windows_opencv_dir, 'x64')):
            if f_ent.lower().startswith("vc"):
                print(f_ent)
                _bin_dir = os.path.join(windows_opencv_dir, 'x64', f_ent, 'bin')
                _lib_dir = os.path.join(windows_opencv_dir, 'x64', f_ent, 'lib')
                if os.path.isdir(_bin_dir) and opencv_bin_dir is None:
                    opencv_bin_dir = _bin_dir
                if os.path.isdir(_lib_dir) and opencv_lib_dir is None:
                    opencv_lib_dir = _lib_dir
        if opencv_lib_dir is None or opencv_bin_dir is None:
            raise Exception(f"No opencv lib/bin directory found.")
        
        os.environ['PATH'] = opencv_bin_dir + ';' + os.environ['PATH']

        os.environ['OPENCV_LINK_LIBS'] = 'opencv_world4120'
        os.environ['OPENCV_INCLUDE_PATHS'] = opencv_inc_dir
        os.environ['OPENCV_LINK_PATHS'] = opencv_lib_dir

        # Needed by rust crate on windows to skip trying pkg-config
        os.environ['OPENCV_DISABLE_PROBES'] = 'pkg_config,cmake,vcpkg_cmake,vcpkg'
    else:
        # Nothing to do expect that pkg-config finds it.
        pass

def _get_features():
    if platform.system() == 'Windows':
        # Libcamera is linux.
        return []
    else:
        return ["--features", "libcamera"]
    
def clean():
    subprocess.run(["cargo", "clean"], check=True)

def test():
    _setup_opencv()
    subprocess.run(["cargo", "test", "--release"] + _get_features(), check=True)

def build():
    _setup_opencv()
    subprocess.run(["cargo", "build", "--release"] + _get_features(), check=True)

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
