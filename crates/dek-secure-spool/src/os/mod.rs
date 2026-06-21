#[cfg(windows)]
pub mod windows_dpapi;

#[cfg(target_os = "macos")]
pub mod macos_keychain;

#[cfg(target_os = "linux")]
pub mod linux_keyring;

#[cfg(windows)]
pub use windows_dpapi::WindowsDpapiStore as DefaultOsKeyStore;

#[cfg(target_os = "macos")]
pub use macos_keychain::MacOsKeychainStore as DefaultOsKeyStore;

#[cfg(target_os = "linux")]
pub use linux_keyring::LinuxFileFallbackStore as DefaultOsKeyStore;
