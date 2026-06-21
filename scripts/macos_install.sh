#!/usr/bin/env bash
set -e

INSTALL_DIR="/usr/local/bin"
DATA_DIR="/Library/Application Support/PollenDEK"
PLIST_DIR="/Library/LaunchDaemons"

if [ "$EUID" -ne 0 ]; then
  echo "Please run as root"
  exit 1
fi

echo "Installing Pollen DEK..."

mkdir -p "$DATA_DIR"
chmod 700 "$DATA_DIR"
chown root:wheel "$DATA_DIR"
mkdir -p "$INSTALL_DIR"

# Copy binaries
cp dek-core "$INSTALL_DIR/"
cp dek-updater "$INSTALL_DIR/"
cp dek-mcp-proxy "$INSTALL_DIR/"

chmod +x "$INSTALL_DIR/dek-core" "$INSTALL_DIR/dek-updater" "$INSTALL_DIR/dek-mcp-proxy"

# Create launchd plist
cat << 'EOF' > "$PLIST_DIR/com.pollen.dek.core.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.pollen.dek.core</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/dek-core</string>
    </array>
    <key>EnvironmentVariables</key>
    <dict>
        <key>DEK_BOOTSTRAP_PATH</key>
        <string>/Library/Application Support/PollenDEK/bootstrap.json</string>
    </dict>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardErrorPath</key>
    <string>/Library/Logs/PollenDEK-Core.err.log</string>
    <key>StandardOutPath</key>
    <string>/Library/Logs/PollenDEK-Core.out.log</string>
</dict>
</plist>
EOF

launchctl load "$PLIST_DIR/com.pollen.dek.core.plist"

echo "Installation complete. Service com.pollen.dek.core is loaded."
