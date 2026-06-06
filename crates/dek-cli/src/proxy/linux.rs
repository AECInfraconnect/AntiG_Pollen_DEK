use anyhow::{Context, Result};
use std::process::Command;

pub fn enable() -> Result<()> {
    // Linux primarily uses eBPF for Layer 2 guardrails.
    // As a fallback app-layer redirect, we can attempt gsettings for GNOME.
    let _ = Command::new("gsettings")
        .args(&["set", "org.gnome.system.proxy", "mode", "'manual'"])
        .output();
    let _ = Command::new("gsettings")
        .args(&["set", "org.gnome.system.proxy.http", "host", "'127.0.0.1'"])
        .output();
    let _ = Command::new("gsettings")
        .args(&["set", "org.gnome.system.proxy.http", "port", "43890"])
        .output();

    Ok(())
}

pub fn disable() -> Result<()> {
    let _ = Command::new("gsettings")
        .args(&["set", "org.gnome.system.proxy", "mode", "'none'"])
        .output();
    Ok(())
}
