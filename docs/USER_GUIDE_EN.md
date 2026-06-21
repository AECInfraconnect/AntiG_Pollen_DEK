# Pollen DEK User Guide

## Overview
Pollen DEK (Distributed Enforcement Kernel) is an endpoint security and policy enforcement tool.

## Key Components
- **DEK Core**: The background service that manages identity, downloads policies, and controls enforcement.
- **DEK MCP Proxy**: A local proxy for Model Context Protocol (MCP) tool usage, authorizing requests before they reach the tools.
- **DEK Updater**: Auto-updater that verifies TUF metadata and cryptographic signatures before applying updates.

## Configuration
Configuration is located at `/opt/pollen` (Linux), `/Library/Application Support/PollenDEK` (macOS), or `C:\ProgramData\PollenDEK` (Windows).

## Logs
Logs can be found in the `logs` subdirectory within the configuration folder.
