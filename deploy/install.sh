#!/usr/bin/env bash
set -euo pipefail

# Pollen DEK Installation Script

echo "--- Installing Pollen DEK ---"

if [ "$EUID" -ne 0 ]; then
  echo "Please run as root"
  exit 1
fi

DEK_VERSION=${1:-"latest"}
BIN_URL="https://github.com/pollen-cloud/pollen-dek/releases/download/${DEK_VERSION}/dek-linux-x86_64"

echo "1. Downloading DEK version ${DEK_VERSION}..."
curl -sL "${BIN_URL}" -o /usr/local/bin/pollen-dek
chmod +x /usr/local/bin/pollen-dek

echo "2. Setting up configuration directories..."
mkdir -p /etc/pollen
mkdir -p /var/lib/pollen

# If configuring via environment variables during install
if [ -n "${POLLEN_ENROLLMENT_TOKEN:-}" ]; then
  echo "Found enrollment token, generating initial config..."
  cat <<EOF > /etc/pollen/dek.yml
control_plane:
  endpoint: "https://cloud.pollen.internal:8443"
  tenant_id: "default"
enrollment:
  token: "${POLLEN_ENROLLMENT_TOKEN}"
EOF
fi

echo "3. Installing systemd service..."
cp dek.service /etc/systemd/system/dek.service
systemctl daemon-reload
systemctl enable dek.service
systemctl start dek.service

echo "--- Installation Complete ---"
echo "Check status with: systemctl status dek.service"
