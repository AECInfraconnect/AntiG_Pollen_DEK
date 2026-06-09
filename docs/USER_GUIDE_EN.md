# Pollen DEK User Guide

## Overview
Pollen DEK (Distributed Enforcement Kernel) is an endpoint security and policy enforcement tool.

## Key Components
- **Pollen DEK Core (`pollen-dek`)**: The background service that manages identity, downloads policies, and controls enforcement.
- **Pollen DEK CLI (`pollen-dekctl`)**: Command-line tool to enroll, manage, and troubleshoot the DEK.
- **Pollen MCP Proxy (`pollen-mcp-proxy`)**: A local proxy for Model Context Protocol (MCP) tool usage, authorizing requests before they reach the tools.
- **Mock Cloud (`pollen-mock-cloud`)**: Local simulation of Pollen Cloud for development and beta testing.

## Configuration
Configuration is located at `~/.pollen/dek/` by default during the beta phase, utilizing `bootstrap.json`.

## Logs
Logs can be viewed using the `pollen-dekctl logs` command, or found in the `~/.pollen/dek/logs/` directory.
