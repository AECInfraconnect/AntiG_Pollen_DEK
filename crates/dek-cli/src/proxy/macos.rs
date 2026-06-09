// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use anyhow::{Context, Result};
use std::process::Command;

pub fn enable() -> Result<()> {
    // Enable proxy for Wi-Fi interface as an example
    Command::new("networksetup")
        .args(&["-setwebproxy", "Wi-Fi", "127.0.0.1", "43890"])
        .output()
        .context("Failed to set web proxy on Mac")?;

    Command::new("networksetup")
        .args(&["-setsecurewebproxy", "Wi-Fi", "127.0.0.1", "43890"])
        .output()
        .context("Failed to set secure web proxy on Mac")?;

    Ok(())
}

pub fn disable() -> Result<()> {
    Command::new("networksetup")
        .args(&["-setwebproxystate", "Wi-Fi", "off"])
        .output()
        .context("Failed to disable web proxy on Mac")?;

    Command::new("networksetup")
        .args(&["-setsecurewebproxystate", "Wi-Fi", "off"])
        .output()
        .context("Failed to disable secure web proxy on Mac")?;

    Ok(())
}
