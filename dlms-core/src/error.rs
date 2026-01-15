use thiserror::Error;

/// Main error type for jDLMS operations
#[derive(Error, Debug)]
pub enum DlmsError {
    #[error("Connection error: {0}")]
    Connection(#[from] std::io::Error),
    
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("Security error: {0}")]
    Security(String),
    
    #[error("Timeout")]
    Timeout,
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("ASN.1 encoding error: {0}")]
    Asn1Encoding(String),
    
    #[error("ASN.1 decoding error: {0}")]
    Asn1Decoding(String),
    
    #[error("Frame invalid: {0}")]
    FrameInvalid(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),
}

/// Result type alias for jDLMS operations
pub type DlmsResult<T> = Result<T, DlmsError>;
