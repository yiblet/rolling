use thiserror::Error;

#[derive(Debug, Error)]
pub enum RollingError {
    #[error("couldn't parse input")]
    ParseError,
    #[error("failed to write result")]
    WriterResult(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, RollingError>;
