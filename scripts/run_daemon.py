#!/bin/python3
#
# Run a daemon process in the background, ensuring only one instance is running.
#
import os
import subprocess
import datetime as dt

def list_processes():
    """List all running processes."""
    # Todo: use tasklist on Windows
    result = subprocess.run(['ps', 'ax', '-o', 'cmd'], capture_output=True, text=True, check=True)
    return [line for line in result.stdout.splitlines() if line != ""]

# Determine home and data directories
script_dir = os.path.dirname(os.path.abspath(__file__))
home_dir = os.path.dirname(script_dir)

home_dir = os.environ.get("ROOK_LW_HOME", home_dir)
data_dir = os.environ.get("ROOK_LW_DATA", os.path.join(home_dir, "var"))

os.environ["ROOK_LW_HOME"] = home_dir
os.environ["ROOK_LW_DATA"] = data_dir

# Process args
if len(os.sys.argv) < 2:
    print("Usage: run_daemon.py <daemon_process> [args...]")
    os.sys.exit(1)

daemon_process = os.sys.argv[1]
daemon_args = os.sys.argv[2:]

# Resolve daemon process path
if not os.path.isabs(daemon_process):
    if not os.path.isfile(daemon_process):
        daemon_process = os.path.join(home_dir, "bin", daemon_process)
    if not os.path.isfile(daemon_process):
        print(f"Daemon process '{daemon_process}' not found in '{os.path.join(home_dir, 'bin')}'")
        os.sys.exit(1)
    daemon_process = os.path.abspath(daemon_process)

daemon_name = os.path.basename(daemon_process)

# Change working directory to home directory
os.chdir(home_dir)

# Find ORT_DYN_LIB
if os.environ.get("ORT_DYLIB_PATH") is None:
    lib_dir = os.path.join(home_dir, "lib")
    possible_lib_names = ["libonnxruntime.so", "onnxruntime.dll"]
    for pln in possible_lib_names:
        lib_path = os.path.join(lib_dir, pln)
        if os.path.isfile(lib_path):
            os.environ["ORT_DYLIB_PATH"] = lib_path
            break

print(f"data_dir: {data_dir}")

# See if the process is already running
processes = list_processes()
for proc in processes:
    if daemon_name in proc and "run_daemon.py" not in proc:
        print(f"Daemon process '{daemon_name}' is already running:")
        print(proc)
        os.sys.exit(0)

# Get start_timestamp
start_timestamp = dt.datetime.now().strftime("%Y%m%d_%H%M%S")

# Set up log file
log_dir = os.path.join(data_dir, "logs")
os.makedirs(log_dir, exist_ok=True)
log_file = os.path.join(log_dir, f"{daemon_name}_{start_timestamp}.log")

# Launch the daemon process in a detached subprocess
with open(log_file, 'a') as lf:
    print(f"Launching daemon process '{daemon_process}' with args: {daemon_args}")
    process = subprocess.Popen(
        [daemon_process] + daemon_args,
        stdout=lf,
        stderr=lf,
        preexec_fn=os.setsid
    )
    print(f"Started daemon process '{daemon_process}' with PID {process.pid}")
    print(f"Logs are being written to: {log_file}")
