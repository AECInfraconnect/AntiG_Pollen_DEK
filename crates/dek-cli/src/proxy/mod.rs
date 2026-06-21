use anyhow::Result;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

pub fn enable_system_proxy() -> Result<()> {
    #[cfg(windows)]
    return windows::enable();

    #[cfg(target_os = "macos")]
    return macos::enable();

    #[cfg(target_os = "linux")]
    return linux::enable();

    #[allow(unreachable_code)]
    Ok(())
}

pub fn disable_system_proxy() -> Result<()> {
    #[cfg(windows)]
    return windows::disable();

    #[cfg(target_os = "macos")]
    return macos::disable();

    #[cfg(target_os = "linux")]
    return linux::disable();

    #[allow(unreachable_code)]
    Ok(())
}
