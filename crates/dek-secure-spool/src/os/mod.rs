#[cfg(windows)]
pub mod windows_dpapi;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub mod linux_keyring;

#[cfg(windows)]
pub use windows_dpapi::WindowsDpapiStore as DefaultOsKeyStore;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub use linux_keyring::LinuxFileFallbackStore as DefaultOsKeyStore;
