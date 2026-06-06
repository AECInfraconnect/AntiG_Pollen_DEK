use anyhow::Result;

pub mod rollback;

pub trait ServiceManager {
    fn install(&self) -> Result<()>;
    fn uninstall(&self) -> Result<()>;
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn status(&self) -> Result<String>;
}

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::OsServiceManager;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::OsServiceManager;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::OsServiceManager;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod unsupported {
    use super::ServiceManager;
    use anyhow::Result;

    pub struct OsServiceManager;

    impl OsServiceManager {
        pub fn new() -> Self {
            Self
        }
    }

    impl ServiceManager for OsServiceManager {
        fn install(&self) -> Result<()> { anyhow::bail!("Unsupported OS") }
        fn uninstall(&self) -> Result<()> { anyhow::bail!("Unsupported OS") }
        fn start(&self) -> Result<()> { anyhow::bail!("Unsupported OS") }
        fn stop(&self) -> Result<()> { anyhow::bail!("Unsupported OS") }
        fn status(&self) -> Result<String> { anyhow::bail!("Unsupported OS") }
    }
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub use unsupported::OsServiceManager;
