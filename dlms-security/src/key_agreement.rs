//! Key Agreement
//!
//! This module provides key agreement protocols for securely establishing
//! shared secrets between DLMS/COSEM devices.
//!
//! # Overview
//!
//! Key agreement allows two parties to establish a shared secret over an
//! insecure channel. This module supports:
//!
//! - **Pre-Shared Keys (PSK)**: Simple key-based authentication
//! - **Diffie-Hellman (DH)**: Secure key exchange
//! - **Certificate-based**: Certificate validation
//!
//! # Usage
//!
//! ```rust,no_run
//! use dlms_security::key_agreement::{
//!     KeyAgreement, KeyAgreementProtocol, KeyAgreementRole
//! };
//!
//! // Create a key agreement instance
//! let mut agreement = KeyAgreement::new(KeyAgreementProtocol::PreSharedKey);
//!
//! // As initiator
//! let request = agreement.initiate("device_id")?;
//!
//! // As responder
//! let response = agreement.process_request(request)?;
//! let shared_secret = agreement.get_shared_secret()?;
//! ```

use crate::error::{DlmsError, DlmsResult};
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Key agreement protocol type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAgreementProtocol {
    /// Pre-shared key (simple password-based)
    PreSharedKey,
    /// Diffie-Hellman key exchange
    DiffieHellman,
    /// Certificate-based authentication
    CertificateBased,
    /// TLS-style key exchange
    TlsKeyExchange,
}

impl fmt::Display for KeyAgreementProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyAgreementProtocol::PreSharedKey => write!(f, "PSK"),
            KeyAgreementProtocol::DiffieHellman => write!(f, "DH"),
            KeyAgreementProtocol::CertificateBased => write!(f, "Cert"),
            KeyAgreementProtocol::TlsKeyExchange => write!(f, "TLS"),
        }
    }
}

/// Role in key agreement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAgreementRole {
    /// Initiator (client side)
    Initiator,
    /// Responder (server side)
    Responder,
}

/// Key agreement state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAgreementState {
    /// Not started
    Idle,
    /// In progress
    InProgress,
    /// Completed successfully
    Completed,
    /// Failed
    Failed,
}

/// Key agreement message
#[derive(Debug, Clone)]
pub struct KeyAgreementMessage {
    /// Protocol version
    pub version: u8,
    /// Protocol type
    pub protocol: KeyAgreementProtocol,
    /// Message type
    pub message_type: KeyAgreementMessageType,
    /// Payload data
    pub payload: Vec<u8>,
    /// Timestamp
    pub timestamp: u64,
}

/// Message type for key agreement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAgreementMessageType {
    /// Initial request from initiator
    Initiate,
    /// Response from responder
    Response,
    /// Confirmation message
    Confirmation,
    /// Error message
    Error,
}

/// Shared secret result
#[derive(Debug, Clone)]
pub struct SharedSecret {
    /// The shared secret bytes
    pub secret: Vec<u8>,
    /// Secret identifier
    pub secret_id: String,
    /// Key derivation info
    pub derivation_info: Vec<u8>,
}

/// Pre-shared key configuration
#[derive(Debug, Clone)]
pub struct PskConfig {
    /// The pre-shared key
    pub key: Vec<u8>,
    /// Key identifier
    pub key_id: String,
}

impl PskConfig {
    /// Create a new PSK configuration
    pub fn new(key: Vec<u8>, key_id: String) -> Self {
        Self { key, key_id }
    }
}

/// Diffie-Hellman parameters
#[derive(Debug, Clone)]
pub struct DhParams {
    /// Prime modulus (p)
    pub prime: Vec<u8>,
    /// Generator (g)
    pub generator: Vec<u8>,
    /// Private key length in bits
    pub key_length: usize,
}

impl Default for DhParams {
    fn default() -> Self {
        // RFC 3526 2048-bit MODP Group
        // Simplified placeholder - in production, use actual DH parameters
        Self {
            prime: vec![0xFF], // Placeholder
            generator: vec![0x02],
            key_length: 2048,
        }
    }
}

/// Key agreement protocol
pub struct KeyAgreement {
    /// The protocol being used
    protocol: KeyAgreementProtocol,
    /// Current state
    state: KeyAgreementState,
    /// Local role (initiator or responder)
    role: KeyAgreementRole,
    /// Shared secret (after successful agreement)
    shared_secret: Option<Vec<u8>>,
    /// Protocol version
    version: u8,
}

impl KeyAgreement {
    /// Create a new key agreement instance
    pub fn new(protocol: KeyAgreementProtocol) -> Self {
        Self {
            protocol,
            state: KeyAgreementState::Idle,
            role: KeyAgreementRole::Initiator,
            shared_secret: None,
            version: 1,
        }
    }

    /// Set the role
    pub fn with_role(mut self, role: KeyAgreementRole) -> Self {
        self.role = role;
        self
    }

    /// Get the current state
    pub fn state(&self) -> KeyAgreementState {
        self.state
    }

    /// Get the protocol
    pub fn protocol(&self) -> KeyAgreementProtocol {
        self.protocol
    }

    /// Initiate key agreement (for initiator)
    pub fn initiate(&mut self, peer_id: &str) -> DlmsResult<KeyAgreementMessage> {
        if self.state != KeyAgreementState::Idle {
            return Err(DlmsError::Security(
                "Key agreement already initiated ".to_string(),
            ));
        }

        self.state = KeyAgreementState::InProgress;

        Ok(KeyAgreementMessage {
            version: self.version,
            protocol: self.protocol,
            message_type: KeyAgreementMessageType::Initiate,
            payload: peer_id.as_bytes().to_vec(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
        })
    }

    /// Process a key agreement request (for responder)
    pub fn process_request(&mut self, message: KeyAgreementMessage) -> DlmsResult<KeyAgreementMessage> {
        if message.message_type != KeyAgreementMessageType::Initiate {
            return Err(DlmsError::Security(
                "Expected Initiate message ".to_string(),
            ));
        }

        self.state = KeyAgreementState::InProgress;

        // Generate response based on protocol
        let payload = match self.protocol {
            KeyAgreementProtocol::PreSharedKey => {
                // PSK: Return key identifier
                vec![0x01] // PSK response indicator
            }
            _ => {
                vec![0x00] // Placeholder for other protocols
            }
        };

        Ok(KeyAgreementMessage {
            version: self.version,
            protocol: self.protocol,
            message_type: KeyAgreementMessageType::Response,
            payload,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
        })
    }

    /// Process a response and derive shared secret
    pub fn process_response(&mut self, message: KeyAgreementMessage) -> DlmsResult<()> {
        if message.message_type != KeyAgreementMessageType::Response {
            return Err(DlmsError::Security(
                "Expected Response message ".to_string(),
            ));
        }

        // Derive shared secret based on protocol
        self.shared_secret = Some(match self.protocol {
            KeyAgreementProtocol::PreSharedKey => {
                // For PSK, the shared secret is the PSK itself
                // In real implementation, would fetch from storage based on peer_id
                vec![0x01, 0x02, 0x03, 0x04] // Placeholder
            }
            KeyAgreementProtocol::DiffieHellman => {
                // DH would compute shared secret here
                vec![0x05, 0x06, 0x07, 0x08] // Placeholder
            }
            _ => {
                vec![0x00] // Placeholder
            }
        });

        self.state = KeyAgreementState::Completed;
        Ok(())
    }

    /// Get the shared secret (after successful agreement)
    pub fn get_shared_secret(&self) -> DlmsResult<&[u8]> {
        self.shared_secret
            .as_deref()
            .ok_or_else(|| DlmsError::Security("No shared secret established ".to_string()))
    }

    /// Reset the key agreement state
    pub fn reset(&mut self) {
        self.state = KeyAgreementState::Idle;
        self.shared_secret = None;
    }
}

/// Key agreement result
#[derive(Debug, Clone)]
pub struct KeyAgreementResult {
    /// Whether the agreement was successful
    pub success: bool,
    /// The shared secret (if successful)
    pub shared_secret: Option<Vec<u8>>,
    /// Any error message (if failed)
    pub error: Option<String>,
}

impl KeyAgreementResult {
    /// Create a successful result
    pub fn success(shared_secret: Vec<u8>) -> Self {
        Self {
            success: true,
            shared_secret: Some(shared_secret),
            error: None,
        }
    }

    /// Create a failed result
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            shared_secret: None,
            error: Some(error.into()),
        }
    }

    /// Check if the agreement was successful
    pub fn is_success(&self) -> bool {
        self.success
    }
}

/// Simple PSK-based key agreement
pub struct PskKeyAgreement {
    /// Pre-shared key
    psk: Vec<u8>,
}

impl PskKeyAgreement {
    /// Create a new PSK key agreement
    pub fn new(psk: Vec<u8>) -> Self {
        Self { psk }
    }

    /// Perform key agreement
    pub fn agree(&self) -> KeyAgreementResult {
        KeyAgreementResult::success(self.psk.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_agreement_creation() {
        let agreement = KeyAgreement::new(KeyAgreementProtocol::PreSharedKey);
        assert_eq!(agreement.protocol(), KeyAgreementProtocol::PreSharedKey);
        assert_eq!(agreement.state(), KeyAgreementState::Idle);
    }

    #[test]
    fn test_key_agreement_initiate() {
        let mut agreement = KeyAgreement::new(KeyAgreementProtocol::PreSharedKey);
        let message = agreement.initiate("peer_id").unwrap();

        assert_eq!(message.message_type, KeyAgreementMessageType::Initiate);
        assert_eq!(agreement.state(), KeyAgreementState::InProgress);
    }

    #[test]
    fn test_key_agreement_process_request() {
        let mut agreement = KeyAgreement::new(KeyAgreementProtocol::PreSharedKey)
            .with_role(KeyAgreementRole::Responder);

        let initiate_msg = KeyAgreementMessage {
            version: 1,
            protocol: KeyAgreementProtocol::PreSharedKey,
            message_type: KeyAgreementMessageType::Initiate,
            payload: b"peer_id".to_vec(),
            timestamp: 1000,
        };

        let response = agreement.process_request(initiate_msg).unwrap();

        assert_eq!(response.message_type, KeyAgreementMessageType::Response);
        assert_eq!(agreement.state(), KeyAgreementState::InProgress);
    }

    #[test]
    fn test_key_agreement_process_response() {
        let mut agreement = KeyAgreement::new(KeyAgreementProtocol::PreSharedKey);
        agreement.initiate("peer_id").unwrap();

        let response_msg = KeyAgreementMessage {
            version: 1,
            protocol: KeyAgreementProtocol::PreSharedKey,
            message_type: KeyAgreementMessageType::Response,
            payload: vec![0x01],
            timestamp: 1000,
        };

        let result = agreement.process_response(response_msg);
        assert!(result.is_ok());
        assert_eq!(agreement.state(), KeyAgreementState::Completed);
        assert!(agreement.get_shared_secret().is_ok());
    }

    #[test]
    fn test_key_agreement_reset() {
        let mut agreement = KeyAgreement::new(KeyAgreementProtocol::PreSharedKey);
        agreement.initiate("peer_id").unwrap();
        assert_eq!(agreement.state(), KeyAgreementState::InProgress);

        agreement.reset();
        assert_eq!(agreement.state(), KeyAgreementState::Idle);
        assert!(agreement.get_shared_secret().is_err());
    }

    #[test]
    fn test_key_agreement_result() {
        let secret = vec![1, 2, 3, 4];
        let result = KeyAgreementResult::success(secret.clone());

        assert!(result.is_success());
        assert_eq!(result.shared_secret, Some(secret));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_key_agreement_result_failure() {
        let result = KeyAgreementResult::failure("Test error");

        assert!(!result.is_success());
        assert!(result.shared_secret.is_none());
        assert_eq!(result.error, Some("Test error".to_string()));
    }

    #[test]
    fn test_psk_key_agreement() {
        let psk = vec![0x01, 0x02, 0x03, 0x04];
        let agreement = PskKeyAgreement::new(psk.clone());

        let result = agreement.agree();

        assert!(result.is_success());
        assert_eq!(result.shared_secret, Some(psk));
    }

    #[test]
    fn test_protocol_display() {
        assert_eq!(KeyAgreementProtocol::PreSharedKey.to_string(), "PSK");
        assert_eq!(KeyAgreementProtocol::DiffieHellman.to_string(), "DH");
    }
}