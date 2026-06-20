use thiserror::Error;

#[derive(Error, Debug)]
pub enum SuggesterError {
    #[error("Rule evaluation failed: {0}")]
    Evaluation(String),
    #[error("Invalid suggestion format: {0}")]
    InvalidFormat(String),
}
