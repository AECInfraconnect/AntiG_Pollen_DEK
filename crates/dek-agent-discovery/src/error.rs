use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiscoveryError {
    #[error("Process scan failed: {0}")]
    ProcessScan(#[from] std::io::Error),
    #[error("Agent validation failed: {0}")]
    Validation(String),
}
