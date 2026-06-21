# Pollen DEK Beta - Known Limitations

## Current Limitations (v1.0.0-beta)
- **eBPF Compatibility**: Currently only supported on Linux kernels 5.8+. Windows and macOS use user-space proxies for enforcement.
- **Auto-Updater Rollbacks**: Rollbacks currently only restore the immediate previous version (`.bak`). Nested or historical rollbacks are not yet implemented.
- **GUI Ext-Authz**: The external authorization UI (`dek-ext-authz`) requires Edge WebView2 runtime on Windows. It is currently experimental on Linux and macOS.
- **Offline Analytics**: Telemetry batching for long-term offline environments (e.g. >30 days without connectivity) might lead to unbounded memory growth. A hard limit will be introduced in RC1.
