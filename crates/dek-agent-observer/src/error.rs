use thiserror::Error;

#[derive(Error, Debug)]
pub enum ObserverError {
    #[error("Cost calculation failed: {0}")]
    CostCalculation(String),
    #[error("Ingestion error: {0}")]
    Ingestion(String),
}
