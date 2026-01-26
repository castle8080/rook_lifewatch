#!/bin/sh
#
# Configures and installs the admin service.
#
set -e

admin_base_dir="$1"

if [ -z "$admin_base_dir" ]; then
    admin_base_dir="$(cd "$(dirname "$0")/../" && pwd)"
else
    admin_base_dir="$(cd "$admin_base_dir" && pwd)"
fi

start_script_path="$admin_base_dir/bin/start_rook_lw_admin.sh"
stop_script_path="$admin_base_dir/bin/stop_rook_lw_admin.sh"

if [ ! -f "$start_script_path" ]; then
    echo "Error: start script not found at: $start_script_path"
    exit 1
fi

if [ ! -f "$stop_script_path" ]; then
    echo "Error: stop script not found at: $stop_script_path"
    exit 1
fi

echo "Installing admin service from base dir: $admin_base_dir"

user="$(whoami)"
group="$(id -gn "$user")"

if [ -z "$user" ] || [ -z "$group" ]; then
    echo "Error: Unable to determine current user/group."
    exit 1
fi

if [ "$user" = "root" ]; then
    echo "Error: Refusing to install service as root user."
    exit 1
fi

service_name="rook_lw_admin"
service_file_path="/etc/systemd/system/$service_name.service"
service_file_local_path="$admin_base_dir/etc/$service_name.service"

mkdir -p "$admin_base_dir/etc"

{
    echo "[Unit]"
    echo "Description=Rook Lifewatch Admin Service"
    echo "After=network.target"
    echo ""
    echo "[Service]"
    echo "Type=simple"
    echo "User=$user"
    echo "Group=$group"
    echo "WorkingDirectory=$admin_base_dir"
    echo "ExecStart=$start_script_path"
    echo "ExecStop=$stop_script_path"
    echo "Restart=on-failure"
    echo "RestartSec=5"
    echo "StandardOutput=journal"
    echo "StandardError=journal"
    echo "TimeoutStopSec=15"
    echo "Environment=RUST_LOG=info"
    echo ""
    echo "[Install]"
    echo "WantedBy=multi-user.target"
    echo ""
} > "$service_file_local_path"

echo "Installing service file to: $service_file_path"
sudo cp -fv "$service_file_local_path" "$service_file_path"
sudo chown root:root "$service_file_path"
sudo chmod 644 "$service_file_path"

echo "Reloading systemd daemon and enabling service..."
sudo systemctl daemon-reexec
sudo systemctl daemon-reload
sudo systemctl enable $service_name.service

echo "Service '$service_name' installed and enabled."
echo "You can start the service with: sudo systemctl start $service_name"
