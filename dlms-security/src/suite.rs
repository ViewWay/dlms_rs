//! Security suite configuration for DLMS/COSEM

use crate::error::{DlmsError, DlmsResult};
use std::fmt;

/// Security policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityPolicy {
    /// No encryption and authentication
    Nothing = 0,
    /// All messages to be authenticated
    Authenticated = 1,
    /// All messages to be encrypted
    Encrypted = 2,
    /// All messages to be authenticated and encrypted
    AuthenticatedAndEncrypted = 3,
}

impl SecurityPolicy {
    /// Get policy ID
    pub fn id(&self) -> u8 {
        *self as u8
    }

    /// Get policy from ID
    pub fn from_id(id: u8) -> DlmsResult<Self> {
        match id {
            0 => Ok(SecurityPolicy::Nothing),
            1 => Ok(SecurityPolicy::Authenticated),
            2 => Ok(SecurityPolicy::Encrypted),
            3 => Ok(SecurityPolicy::AuthenticatedAndEncrypted),
            _ => Err(DlmsError::Security(format!("Invalid security policy ID: {}", id))),
        }
    }

    /// Check if policy requires authentication
    pub fn is_authenticated(&self) -> bool {
        matches!(self, SecurityPolicy::Authenticated | SecurityPolicy::AuthenticatedAndEncrypted)
    }

    /// Check if policy requires encryption
    pub fn is_encrypted(&self) -> bool {
        matches!(self, SecurityPolicy::Encrypted | SecurityPolicy::AuthenticatedAndEncrypted)
    }
}

/// Encryption mechanism
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionMechanism {
    /// Do not encrypt transport
    None = -1,
    /// Use AES-128-GCM
    AesGcm128 = 0,
}

impl EncryptionMechanism {
    /// Get mechanism ID
    pub fn id(&self) -> i32 {
        *self as i32
    }

    /// Get mechanism from ID
    pub fn from_id(id: i32) -> DlmsResult<Self> {
        match id {
            -1 => Ok(EncryptionMechanism::None),
            0 => Ok(EncryptionMechanism::AesGcm128),
            _ => Err(DlmsError::Security(format!("Invalid encryption mechanism ID: {}", id))),
        }
    }

    /// Get key length in bits
    pub fn key_length_bits(&self) -> Option<usize> {
        match self {
            EncryptionMechanism::None => None,
            EncryptionMechanism::AesGcm128 => Some(128),
        }
    }

    /// Get key length in bytes
    pub fn key_length_bytes(&self) -> Option<usize> {
        self.key_length_bits().map(|bits| bits / 8)
    }

    /// Validate key length
    pub fn validate_key_length(&self, key: &[u8]) -> DlmsResult<()> {
        if let Some(expected_len) = self.key_length_bytes() {
            if key.len() != expected_len {
                return Err(DlmsError::Security(format!(
                    "Invalid key length: expected {} bytes, got {}",
                    expected_len,
                    key.len()
                )));
            }
        }
        Ok(())
    }
}

/// Authentication mechanism
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthenticationMechanism {
    /// No authentication used (no mechanism presented in AARQ message)
    Absent = -1,
    /// No authentication used
    None = 0,
    /// Authentication of the client by sending a shared password as secret
    Low = 1,
    /// Authentication of both client and smart meter using GMAC and a pre shared secret password
    Hls5Gmac = 5,
}

impl AuthenticationMechanism {
    /// Get mechanism ID
    pub fn id(&self) -> i32 {
        *self as i32
    }

    /// Get mechanism from ID
    pub fn from_id(id: i32) -> DlmsResult<Self> {
        match id {
            -1 => Ok(AuthenticationMechanism::Absent),
            0 => Ok(AuthenticationMechanism::None),
            1 => Ok(AuthenticationMechanism::Low),
            5 => Ok(AuthenticationMechanism::Hls5Gmac),
            _ => Err(DlmsError::Security(format!("Invalid authentication mechanism ID: {}", id))),
        }
    }

    /// Check if this is an HLS mechanism
    pub fn is_hls_mechanism(&self) -> bool {
        matches!(self, AuthenticationMechanism::Hls5Gmac)
    }
}

/// Security suite builder
pub struct SecuritySuiteBuilder {
    encryption_mechanism: EncryptionMechanism,
    authentication_mechanism: AuthenticationMechanism,
    global_unicast_encryption_key: Option<Vec<u8>>,
    authentication_key: Option<Vec<u8>>,
    password: Option<Vec<u8>>,
    security_policy: Option<SecurityPolicy>,
}

impl SecuritySuiteBuilder {
    /// Create a new security suite builder with default config (no authentication and no encryption)
    pub fn new() -> Self {
        Self {
            encryption_mechanism: EncryptionMechanism::None,
            authentication_mechanism: AuthenticationMechanism::None,
            global_unicast_encryption_key: None,
            authentication_key: None,
            password: None,
            security_policy: None,
        }
    }

    /// Set the security policy
    pub fn set_security_policy(mut self, policy: SecurityPolicy) -> Self {
        self.security_policy = Some(policy);
        self
    }

    /// Set the encryption mechanism
    pub fn set_encryption_mechanism(mut self, mechanism: EncryptionMechanism) -> Self {
        self.encryption_mechanism = mechanism;
        self
    }

    /// Set the authentication mechanism
    pub fn set_authentication_mechanism(mut self, mechanism: AuthenticationMechanism) -> Self {
        self.authentication_mechanism = mechanism;
        self
    }

    /// Set the global unicast encryption key
    pub fn set_global_unicast_encryption_key(mut self, key: Vec<u8>) -> Self {
        self.global_unicast_encryption_key = Some(key);
        self
    }

    /// Set the authentication key
    pub fn set_authentication_key(mut self, key: Vec<u8>) -> Self {
        self.authentication_key = Some(key);
        self
    }

    /// Set the password (for LOW authentication)
    pub fn set_password(mut self, password: Vec<u8>) -> Self {
        self.password = Some(password);
        self.authentication_mechanism = AuthenticationMechanism::Low;
        self
    }

    /// Build the security suite
    pub fn build(self) -> DlmsResult<SecuritySuite> {
        let security_policy = self.security_policy.unwrap_or_else(|| {
            if self.authentication_mechanism.is_hls_mechanism() {
                if self.encryption_mechanism != EncryptionMechanism::None {
                    SecurityPolicy::AuthenticatedAndEncrypted
                } else {
                    SecurityPolicy::Authenticated
                }
            } else if self.encryption_mechanism != EncryptionMechanism::None {
                SecurityPolicy::Encrypted
            } else {
                SecurityPolicy::Nothing
            }
        });

        // Validate fields
        self.validate(&security_policy)?;

        Ok(SecuritySuite {
            global_unicast_encryption_key: self.global_unicast_encryption_key,
            authentication_key: self.authentication_key,
            password: self.password,
            encryption_mechanism: self.encryption_mechanism,
            authentication_mechanism: self.authentication_mechanism,
            security_policy,
        })
    }

    fn validate(&self, security_policy: &SecurityPolicy) -> DlmsResult<()> {
        // Validate security policy
        if security_policy.is_encrypted()
            && self.encryption_mechanism == EncryptionMechanism::None
        {
            return Err(DlmsError::Security(
                "Select a cryptographic algorithm to encrypt messages".to_string(),
            ));
        }

        if security_policy.is_authenticated()
            && !self.authentication_mechanism.is_hls_mechanism()
        {
            return Err(DlmsError::Security(
                "Select a HLS authentication to authenticate messages".to_string(),
            ));
        }

        // Validate encryption mechanism key length
        if let Some(ref key) = self.global_unicast_encryption_key {
            self.encryption_mechanism.validate_key_length(key)?;
        }

        // Validate authentication mechanism
        match self.authentication_mechanism {
            AuthenticationMechanism::Hls5Gmac => {
                if self.authentication_key.is_none() || self.global_unicast_encryption_key.is_none() {
                    return Err(DlmsError::Security(
                        "Authentication/Encryption key either not supplied or don't match in length"
                            .to_string(),
                    ));
                }
                if let (Some(auth_key), Some(enc_key)) =
                    (&self.authentication_key, &self.global_unicast_encryption_key)
                {
                    if auth_key.len() != enc_key.len() {
                        return Err(DlmsError::Security(
                            "Authentication key length does not match encryption key length"
                                .to_string(),
                        ));
                    }
                }
            }
            AuthenticationMechanism::Low => {
                if self.password.is_none() {
                    return Err(DlmsError::Security(
                        "Password is not set for the security level low".to_string(),
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl Default for SecuritySuiteBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Security suite
#[derive(Debug, Clone)]
pub struct SecuritySuite {
    global_unicast_encryption_key: Option<Vec<u8>>,
    authentication_key: Option<Vec<u8>>,
    password: Option<Vec<u8>>,
    encryption_mechanism: EncryptionMechanism,
    authentication_mechanism: AuthenticationMechanism,
    security_policy: SecurityPolicy,
}

impl SecuritySuite {
    /// Create a new security suite builder
    pub fn builder() -> SecuritySuiteBuilder {
        SecuritySuiteBuilder::new()
    }

    /// Get the global unicast encryption key
    pub fn global_unicast_encryption_key(&self) -> Option<&[u8]> {
        self.global_unicast_encryption_key.as_deref()
    }

    /// Get the authentication key
    pub fn authentication_key(&self) -> Option<&[u8]> {
        self.authentication_key.as_deref()
    }

    /// Get the password
    pub fn password(&self) -> Option<&[u8]> {
        self.password.as_deref()
    }

    /// Get the encryption mechanism
    pub fn encryption_mechanism(&self) -> EncryptionMechanism {
        self.encryption_mechanism
    }

    /// Get the authentication mechanism
    pub fn authentication_mechanism(&self) -> AuthenticationMechanism {
        self.authentication_mechanism
    }

    /// Get the security policy
    pub fn security_policy(&self) -> SecurityPolicy {
        self.security_policy
    }

    /// Update the global unicast encryption key
    pub fn update_global_unicast_encryption_key(&mut self, key: Vec<u8>) -> DlmsResult<()> {
        self.encryption_mechanism.validate_key_length(&key)?;
        self.global_unicast_encryption_key = Some(key);
        Ok(())
    }

    /// Update the authentication key
    pub fn update_authentication_key(&mut self, key: Vec<u8>) -> DlmsResult<()> {
        if let Some(ref enc_key) = self.global_unicast_encryption_key {
            if key.len() != enc_key.len() {
                return Err(DlmsError::Security(
                    "Authentication key length does not match encryption key length".to_string(),
                ));
            }
        }
        self.authentication_key = Some(key);
        Ok(())
    }
}

impl Default for SecuritySuite {
    fn default() -> Self {
        Self {
            global_unicast_encryption_key: None,
            authentication_key: None,
            password: None,
            encryption_mechanism: EncryptionMechanism::None,
            authentication_mechanism: AuthenticationMechanism::None,
            security_policy: SecurityPolicy::Nothing,
        }
    }
}

impl fmt::Display for SecuritySuite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SecuritySuite(encryption={:?}, auth={:?}, policy={:?})",
            self.encryption_mechanism, self.authentication_mechanism, self.security_policy
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_suite_builder() {
        let suite = SecuritySuite::builder()
            .set_encryption_mechanism(EncryptionMechanism::AesGcm128)
            .set_global_unicast_encryption_key(vec![0u8; 16])
            .build()
            .unwrap();
        assert_eq!(suite.encryption_mechanism(), EncryptionMechanism::AesGcm128);
    }
}
