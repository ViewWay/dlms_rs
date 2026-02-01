//! DLMS/COSEM server error types

use std::fmt;

/// Server error codes for detailed error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerErrorCode {
    /// General initialization error
    InitializationFailed = 1,
    /// Binding to address failed
    BindFailed = 2,
    /// Listening failed
    ListenFailed = 3,
    /// Accept connection failed
    AcceptFailed = 4,
    /// I/O error
    IoError = 5,
    /// Authentication failed
    AuthenticationFailed = 6,
    /// Max connections exceeded
    MaxConnectionsExceeded = 7,
    /// Invalid configuration
    InvalidConfiguration = 8,
    /// Resource exhausted
    ResourceExhausted = 9,
    /// Operation timeout
    Timeout = 10,
}

impl ServerErrorCode {
    /// Get error code description
    pub fn description(&self) -> &'static str {
        match self {
            Self::InitializationFailed => "Server initialization failed",
            Self::BindFailed => "Failed to bind to address",
            Self::ListenFailed => "Failed to listen on socket",
            Self::AcceptFailed => "Failed to accept connection",
            Self::IoError => "I/O error occurred",
            Self::AuthenticationFailed => "Authentication failed",
            Self::MaxConnectionsExceeded => "Maximum connections exceeded",
            Self::InvalidConfiguration => "Invalid server configuration",
            Self::ResourceExhausted => "Server resources exhausted",
            Self::Timeout => "Operation timed out",
        }
    }

    /// Create from integer value
    pub fn from_value(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::InitializationFailed),
            2 => Some(Self::BindFailed),
            3 => Some(Self::ListenFailed),
            4 => Some(Self::AcceptFailed),
            5 => Some(Self::IoError),
            6 => Some(Self::AuthenticationFailed),
            7 => Some(Self::MaxConnectionsExceeded),
            8 => Some(Self::InvalidConfiguration),
            9 => Some(Self::ResourceExhausted),
            10 => Some(Self::Timeout),
            _ => None,
        }
    }

    /// Get integer value
    pub fn value(&self) -> u8 {
        *self as u8
    }
}

impl fmt::Display for ServerErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// DLMS/COSEM server-specific error type
#[derive(Debug)]
pub enum DlmsServerError {
    /// Invalid state transition attempted
    InvalidStateTransition {
        /// The source state
        from: String,
        /// The target state
        to: String,
    },
    /// Invalid state for operation
    InvalidState {
        /// Expected state
        expected: String,
        /// Actual state
        actual: String,
    },
    /// Operation not allowed in current state
    OperationNotAllowed {
        /// Current state
        state: String,
        /// Operation that was attempted
        operation: String,
    },
    /// Server error with code
    ServerError {
        /// Error code
        code: ServerErrorCode,
        /// Error message
        message: String,
    },
}

impl fmt::Display for DlmsServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStateTransition { from, to } => {
                write!(f, "Invalid state transition: {} -> {}", from, to)
            }
            Self::InvalidState { expected, actual } => {
                write!(f, "Invalid state: expected {}, got {}", expected, actual)
            }
            Self::OperationNotAllowed { state, operation } => {
                write!(f, "Operation '{}' not allowed in state '{}'", operation, state)
            }
            Self::ServerError { code, message } => {
                write!(f, "Server error ({}): {}", code, message)
            }
        }
    }
}

impl std::error::Error for DlmsServerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(ServerErrorCode::BindFailed.to_string(), "Failed to bind to address");
        assert_eq!(ServerErrorCode::Timeout.to_string(), "Operation timed out");
    }

    #[test]
    fn test_server_error_display() {
        let err = DlmsServerError::InvalidStateTransition {
            from: "Idle".to_string(),
            to: "Stopped".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid state transition: Idle -> Stopped");
    }
}
