//! Security module for DLMS/COSEM protocol
//!
//! This crate provides security functionality including encryption and authentication.

pub mod error;
pub mod suite;
pub mod encryption;
pub mod authentication;
pub mod utils;

pub use error::{DlmsError, DlmsResult};
pub use suite::{
    SecuritySuite, SecuritySuiteBuilder, SecurityPolicy, EncryptionMechanism, AuthenticationMechanism,
};
pub use encryption::{AesGcmEncryption, SecurityControl};
pub use authentication::{GmacAuth, LowAuth, Hls5GmacAuth};
pub use utils::{KeyId, generate_aes128_key, wrap_aes_rfc3394_key, unwrap_aes_rfc3394_key};
