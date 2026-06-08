# Pollen DEK Installation Guide

## System Requirements
- OS: Windows 10/11, macOS 12+, or Ubuntu 20.04+
- Storage: 100MB free space
- Privileges: Administrator/root access required

## Windows Installation
1. Download `pollen-dek-windows-msi` from Releases.
2. Run the MSI installer.
3. The DEK Core Service will automatically start.

## Linux Installation
1. Download the `.deb` or `.tar.gz` release.
2. For `.deb`: `sudo dpkg -i pollen-dek_1.0.0_amd64.deb`
3. The systemd service will automatically start.

## macOS Installation
1. Download the `.pkg` release.
2. Run the installer package.
3. The launchd agent will load automatically.

## Verification
Run `dek-core --version` to verify installation.
