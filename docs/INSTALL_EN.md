# Pollen DEK Installation Guide (v1.0.0-beta)

## System Requirements

- OS: Windows 10/11, macOS 12+, or Ubuntu 20.04+
- Storage: 100MB free space
- Privileges: Administrator/root access required

## Windows Installation

1. Download `pollen-dek-x86_64-pc-windows-msvc.msi` from the GitHub Releases page.
2. Double-click the MSI installer and follow the prompts.
3. The `PollenDEKCore` service will be installed and started automatically in the background.

## Linux Installation

1. Download the `.deb` release matching your architecture (e.g., `pollen-dek-x86_64-unknown-linux-gnu.deb` or `aarch64`).
2. Install via dpkg: `sudo dpkg -i pollen-dek-*.deb`
3. The `pollen-dek.service` systemd service will be automatically enabled and started.

## macOS Installation

1. Download the `.pkg` release (e.g., `pollen-dek-x86_64-apple-darwin.pkg`).
2. Run the installer package.
3. The `ai.pollen.dek` launchd agent will load automatically.

## Verification

Run `pollen-dekctl status` to verify installation and service health.
