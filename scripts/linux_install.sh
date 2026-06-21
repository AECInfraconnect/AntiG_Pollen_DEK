#!/usr/bin/env bash
set -e

INSTALL_DIR="/usr/local/bin"
DATA_DIR="/etc/pollen-dek"
SYSTEMD_DIR="/etc/systemd/system"

if [ "$EUID" -ne 0 ]; then
  echo "Please run as root"
  exit 1
fi

echo "Installing Pollen DEK..."

mkdir -p "$DATA_DIR"

# Copy binaries
cp dek-core "$INSTALL_DIR/"
cp dek-updater "$INSTALL_DIR/"
cp dek-mcp-proxy "$INSTALL_DIR/"

chmod +x "$INSTALL_DIR/dek-core" "$INSTALL_DIR/dek-updater" "$INSTALL_DIR/dek-mcp-proxy"

# Create systemd service
cat << 'EOF' > "$SYSTEMD_DIR/pollen-dek-core.service"
[Unit]
Description=Pollen Distributed Enforcement Kernel Core
After=network.target

[Service]
ExecStart=/usr/local/bin/dek-core
Restart=always
RestartSec=5
User=root
# We run as root to support eBPF and firewall manipulation
Environment=DEK_BOOTSTRAP_PATH=/etc/pollen-dek/bootstrap.json
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable pollen-dek-core.service
systemctl start pollen-dek-core.service

echo "Installation complete. Service pollen-dek-core is enabled and started."
