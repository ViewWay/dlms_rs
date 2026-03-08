//! DLMS/COSEM Security Layer
//!
//! This crate provides security functionality for DLMS/COSEM protocol including
//! encryption, authentication, and key management.
//!
//! # Overview
//!
//! DLMS/COSEM supports multiple security mechanisms defined in the Blue Book:
//!
//! - **AES-128-GCM**: Encryption and authentication for ciphered connections
//! - **GMAC**: Authentication-only using Galois Message Authentication Code
//! - **HLS5-GMAC**: High-level security with challenge-response authentication
//! - **Low-Level Authentication**: Simple password-based authentication
//! - **Key Wrapping**: RFC 3394 key wrapping for secure key transport
//!
//! # Security Suites
//!
//! Security suites combine authentication and encryption mechanisms:
//!
//! ```rust,ignore
//! use dlms_security::{SecuritySuite, SecuritySuiteBuilder, SecurityPolicy};
//!
//! // Suite 0: No security
//! let suite_0 = SecuritySuite::new();
//!
//! // Suite 1: Low-level authentication only
//! let suite_1 = SecuritySuiteBuilder::new()
//!     .with_low_level_auth(b"secret_password")
//!     .build()?;
//!
//! // Suite 2: HLS5-GMAC authentication
//! let suite_2 = SecuritySuiteBuilder::new()
//!     .with_hls5_gmac(b"client_key", b"server_key")
//!     .build()?;
//!
//! // Suite 12: AES-GCM encryption with authentication
//! let suite_12 = SecuritySuiteBuilder::new()
//!     .with_encryption(
//!         &client_enc_key,
//!         &client_auth_key,
//!         &server_system_title,
//!     )
//!     .build()?;
//! ```
//!
//! # Encryption
//!
//! AES-GCM encryption provides both confidentiality and integrity:
//!
//! ```rust,ignore
//! use dlms_security::{AesGcmEncryption, SecurityControl};
//!
//! let encryption = AesGcmEncryption::new(&key)?;
//!
//! // Encrypt data
//! let security_control = SecurityControl::new(12, 0, 0);
//! let ciphertext = encryption.encrypt(
//!     &system_title,
//!     &frame_counter,
//!     security_control,
//!     &plaintext,
//! )?;
//!
//! // Decrypt data
//! let decrypted = encryption.decrypt(
//!     &system_title,
//!     &frame_counter,
//!     security_control,
//!     &ciphertext,
//! )?;
//! ```
//!
//! # Authentication
//!
//! Multiple authentication mechanisms are supported:
//!
//! ```rust,ignore
//! use dlms_security::{AuthenticationFlow, AuthenticationMechanism, GmacAuth};
//!
//! // GMAC authentication
//! let gmac = GmacAuth::new(&key)?;
//! let tag = gmac.generate_tag(&system_title, &frame_counter, &data)?;
//!
//! // HLS5 challenge-response
//! let flow = AuthenticationFlow::new_hls5_client(
//!     &client_title,
//!     &client_key,
//!     &server_title,
//! )?;
//!
//! let challenge = flow.generate_challenge()?;
//! // Send challenge to server, receive response...
//! let verified = flow.verify_response(&response)?;
//! ```
//!
//! # Key Management
//!
//! Utilities for secure key handling:
//!
//! ```rust,ignore
//! use dlms_security::{
//!     generate_aes128_key,
//!     wrap_aes_rfc3394_key,
//!     unwrap_aes_rfc3394_key,
//!     KeyId,
//! };
//!
//! // Generate random AES-128 key
//! let key = generate_aes128_key();
//!
//! // Wrap key for transport (RFC 3394)
//! let wrapping_key = generate_aes128_key();
//! let (wrapped_key, kek_id) = wrap_aes_rfc3394_key(&wrapping_key, &key)?;
//!
//! // Unwrap key
//! let unwrapped = unwrap_aes_rfc3394_key(&wrapping_key, &wrapped_key, &kek_id)?;
//! ```
//!
//! # xDLMS Context
//!
//! The xDLMS context manages system titles and frame counters for encrypted connections:
//!
//! ```rust,ignore
//! use dlms_security::{XdlmsContext, SystemTitle, FrameCounter};
//!
//! let mut context = XdlmsContext::new();
//!
//! // Set system titles
//! context.set_client_title(
//!     SystemTitle::from_bytes(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08])
//! );
//! context.set_server_title(
//!     SystemTitle::from_bytes(&[0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18])
//! );
//!
//! // Frame counter management
//! context.increment_frame_counter();
//! let fc = context.frame_counter();
//! ```
//!
//! # Module Structure
//!
//! - [`suite`] - Security suite and policy definitions
//! - [`encryption`] - AES-GCM encryption implementation
//! - [`authentication`] - Authentication mechanisms (GMAC, HLS, Low-Level)
//! - [`auth_flow`] - Authentication flow orchestration
//! - [`utils`] - Key generation and wrapping utilities
//! - [`constants`] - Security-related constants
//! - [`xdlms`] - xDLMS context management
//! - [`xdlms_frame`] - Encrypted frame building and parsing
//! - [`suite_negotiation`] - Security suite negotiation
//! - [`validation`] - Security parameter validation
//! - [`key_management`] - Key management and lifecycle
//! - [`key_agreement`] - Key agreement protocols
//!
//! # Implementation Status
//!
//! ## Encryption
//! - [x] AES-128-GCM encryption/decryption
//! - [x] Security Control byte handling
//! - [x] Encrypted frame construction/parsing
//! - [x] System Title integration
//! - [x] Frame Counter integration
//!
//! ## Authentication
//! - [x] GMAC authentication
//! - [x] Low-Level authentication (password-based)
//! - [x] HLS5-GMAC challenge-response
//! - [x] Authentication flow orchestration
//! - [x] Authentication state management
//!
//! ## Key Management
//! - [x] AES-128 key generation
//! - [x] RFC 3394 key wrapping/unwrapping
//! - [x] Key Derivation Function (KDF)
//! - [x] System Title management
//! - [x] Frame Counter management
//! - [x] xDLMS context management
//!
//! ## Security Suites
//! - [x] Security Suite 0: No security
//! - [x] Security Suite 1: Low-Level authentication
//! - [x] Security Suite 2: HLS5-GMAC
//! - [x] Security Suite 12: AES-GCM encryption
//!
//! # References
//!
//! - DLMS Blue Book: DLMS/COSEM Architecture and Protocols
//! - DLMS Green Book: DLMS/COSEM Object Identification System
//! - DLMS White Book: DLMS/COSEM Security
//! - RFC 3394: Advanced Encryption Standard (AES) Key Wrap Algorithm

pub mod error;
pub mod suite;
pub mod encryption;
pub mod authentication;
pub mod auth_flow;
pub mod utils;
pub mod constants;
pub mod xdlms;
pub mod xdlms_frame;
pub mod suite_negotiation;
pub mod validation;
pub mod key_management;
pub mod key_agreement;

pub use error::{DlmsError, DlmsResult};
pub use suite::{
    SecuritySuite, SecuritySuiteBuilder, SecurityPolicy, EncryptionMechanism,
};
pub use encryption::{AesGcmEncryption, SecurityControl};
pub use authentication::{GmacAuth, LowAuth, Hls5GmacAuth};
pub use auth_flow::{AuthenticationFlow, AuthenticationMechanism, AuthenticationState};
pub use utils::{KeyId, generate_aes128_key, wrap_aes_rfc3394_key, unwrap_aes_rfc3394_key};
pub use constants::*;
pub use xdlms::{SystemTitle, FrameCounter, KeyDerivationFunction, XdlmsContext};
pub use xdlms_frame::{EncryptedFrameBuilder, EncryptedFrameParser};
pub use suite_negotiation::{
    SecuritySuiteNegotiator, SuiteId, SuiteProposal, NegotiationState,
    NegotiationTimeout, NegotiationError, NegotiationParameters,
    create_common_suites, suite_id_from_ids, suite_id_to_ids,
};
pub use validation::{
    SecurityParameterValidator, ValidationResult, ValidationError, ValidationWarning,
    ErrorCode, WarningCode, ValidationRule, validate_key_vector,
};
pub use key_management::{
    KeyManager, ProtectedKey, KeyStorage, KeyType, KeyRotationPolicy,
    InMemoryKeyStorage, SessionKeys, KeyGenerator,
};
pub use key_agreement::{
    KeyAgreement, KeyAgreementProtocol, KeyAgreementRole, KeyAgreementState,
    KeyAgreementMessage, SharedSecret, PskConfig, PskKeyAgreement, KeyAgreementResult,
};
