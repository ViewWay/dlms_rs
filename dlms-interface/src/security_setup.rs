//! COSEM Security Setup interface class (Class ID: 64)
//!
//! This interface class manages security parameters for DLMS/COSEM communication.
//! It provides configuration for encryption, authentication, and security policies.

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DataObject, DlmsError, DlmsResult, ObisCode};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Security Setup interface class (Class ID: 64)
///
/// Default OBIS: 0-0:43.0.0.255
///
/// This class manages security parameters for DLMS/COSEM communication including:
/// - Security suite configuration
/// - System titles (client and server)
/// - Security policy
/// - Global and dedicated keys
#[derive(Debug, Clone)]
pub struct SecuritySetup {
    /// Logical name (OBIS code)
    logical_name: ObisCode,

    /// Security suite containing all security parameters
    security_suite: Arc<RwLock<dlms_security::SecuritySuite>>,
}

impl SecuritySetup {
    /// Class ID for Security Setup
    pub const CLASS_ID: u16 = 64;

    /// Default OBIS code for Security Setup (0-0:43.0.0.255)
    /// Note: Use `default_obis()` method to get the default OBIS code
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 43, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_SECURITY_SUITE: u8 = 2;
    pub const ATTR_CLIENT_SYSTEM_TITLE: u8 = 3;
    pub const ATTR_SERVER_SYSTEM_TITLE: u8 = 4;
    pub const ATTR_SECURITY_POLICY: u8 = 5;
    pub const ATTR_GLOBAL_KEY: u8 = 6;
    pub const ATTR_DEDICATED_KEY: u8 = 7;

    /// Create a new Security Setup object
    pub fn new(
        logical_name: ObisCode,
        security_suite: dlms_security::SecuritySuite,
    ) -> Self {
        Self {
            logical_name,
            security_suite: Arc::new(RwLock::new(security_suite)),
        }
    }

    /// Create with default OBIS code and default security suite
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis(), dlms_security::SecuritySuite::default())
    }

    /// Get the security suite
    pub async fn get_security_suite(&self) -> dlms_security::SecuritySuite {
        self.security_suite.read().await.clone()
    }

    /// Set the security suite
    pub async fn set_security_suite(&self, suite: dlms_security::SecuritySuite) {
        let mut ss = self.security_suite.write().await;
        *ss = suite;
    }

    /// Get the client system title
    pub async fn get_client_system_title(&self) -> Vec<u8> {
        // Note: System titles are managed through the security suite
        // This is a placeholder that returns an empty byte array
        // In a full implementation, the SecuritySuite would include system titles
        Vec::new()
    }

    /// Get the server system title
    pub async fn get_server_system_title(&self) -> Vec<u8> {
        // Note: System titles are managed through the security suite
        // This is a placeholder that returns an empty byte array
        // In a full implementation, the SecuritySuite would include system titles
        Vec::new()
    }

    /// Get the security policy as a DataObject
    async fn encode_security_policy(&self) -> DataObject {
        let suite = self.security_suite.read().await;
        let policy = suite.security_policy();
        DataObject::Unsigned8(policy.id())
    }

    /// Encode the security suite as a DataObject (structure)
    async fn encode_security_suite(&self) -> DataObject {
        let suite = self.security_suite.read().await;
        let mut fields = Vec::new();

        // Security policy
        fields.push(DataObject::Unsigned8(suite.security_policy().id()));

        // Encryption mechanism (id returns i32, so use Integer32)
        fields.push(DataObject::Integer32(suite.encryption_mechanism().id()));

        // Authentication mechanism (id returns i32, so use Integer32)
        fields.push(DataObject::Integer32(suite.authentication_mechanism().id()));

        DataObject::Structure(fields)
    }
}

#[async_trait]
impl CosemObject for SecuritySetup {
    fn class_id(&self) -> u16 {
        Self::CLASS_ID
    }

    fn obis_code(&self) -> ObisCode {
        self.logical_name
    }

    async fn get_attribute(
        &self,
        attribute_id: u8,
        _selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<DataObject> {
        match attribute_id {
            Self::ATTR_LOGICAL_NAME => {
                Ok(DataObject::OctetString(self.logical_name.to_bytes().to_vec()))
            }
            Self::ATTR_SECURITY_SUITE => {
                Ok(self.encode_security_suite().await)
            }
            Self::ATTR_CLIENT_SYSTEM_TITLE => {
                Ok(DataObject::OctetString(self.get_client_system_title().await))
            }
            Self::ATTR_SERVER_SYSTEM_TITLE => {
                Ok(DataObject::OctetString(self.get_server_system_title().await))
            }
            Self::ATTR_SECURITY_POLICY => {
                Ok(self.encode_security_policy().await)
            }
            Self::ATTR_GLOBAL_KEY => {
                let suite = self.security_suite.read().await;
                if let Some(key) = suite.global_unicast_encryption_key() {
                    Ok(DataObject::OctetString(key.to_vec()))
                } else {
                    Ok(DataObject::OctetString(Vec::new()))
                }
            }
            Self::ATTR_DEDICATED_KEY => {
                // In current implementation, dedicated_key is stored in the security suite
                // but not exposed through the public API
                // For now, return empty octet string
                Ok(DataObject::OctetString(Vec::new()))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Attribute not supported: {}",
                attribute_id
            ))),
        }
    }

    async fn set_attribute(
        &self,
        attribute_id: u8,
        value: DataObject,
        _selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<()> {
        match attribute_id {
            Self::ATTR_LOGICAL_NAME => {
                // Logical name is typically read-only
                Err(DlmsError::AccessDenied(format!(
                    "Attribute {} is read-only",
                    attribute_id
                )))
            }
            Self::ATTR_SECURITY_SUITE => {
                // Security suite should be set through the dedicated method
                Err(DlmsError::AccessDenied(format!(
                    "Attribute {} must be set through dedicated method",
                    attribute_id
                )))
            }
            Self::ATTR_CLIENT_SYSTEM_TITLE => {
                // System titles are typically managed through the security suite
                Err(DlmsError::AccessDenied(format!(
                    "Attribute {} is read-only",
                    attribute_id
                )))
            }
            Self::ATTR_SERVER_SYSTEM_TITLE => {
                // System titles are typically managed through the security suite
                Err(DlmsError::AccessDenied(format!(
                    "Attribute {} is read-only",
                    attribute_id
                )))
            }
            Self::ATTR_SECURITY_POLICY => {
                // Security policy should be set through the security suite
                Err(DlmsError::AccessDenied(format!(
                    "Attribute {} must be set through security suite",
                    attribute_id
                )))
            }
            Self::ATTR_GLOBAL_KEY => {
                if let DataObject::OctetString(_bytes) = value {
                    // Note: We can't directly modify the global key through the current API
                    // In a full implementation, we would use SecuritySuite::update_global_unicast_encryption_key
                    // For now, return an error indicating this needs to be done differently
                    Err(DlmsError::InvalidData(
                        "Global key must be set through security suite".to_string(),
                    ))
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected octet string for global key".to_string(),
                    ))
                }
            }
            Self::ATTR_DEDICATED_KEY => {
                if let DataObject::OctetString(_bytes) = value {
                    // Dedicated keys are managed through the security suite
                    Err(DlmsError::InvalidData(
                        "Dedicated key must be set through security suite".to_string(),
                    ))
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected octet string for dedicated key".to_string(),
                    ))
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Attribute not supported: {}",
                attribute_id
            ))),
        }
    }

    async fn invoke_method(
        &self,
        method_id: u8,
        _parameters: Option<DataObject>,
        _selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<Option<DataObject>> {
        match method_id {
            // Security Setup typically doesn't have methods in the standard
            _ => Err(DlmsError::InvalidData(format!(
                "Method not supported: {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_setup_class_id() {
        let setup = SecuritySetup::with_default_obis();
        assert_eq!(setup.class_id(), 64);
    }

    #[tokio::test]
    async fn test_security_setup_obis_code() {
        let setup = SecuritySetup::with_default_obis();
        assert_eq!(setup.obis_code(), SecuritySetup::default_obis());
    }

    #[tokio::test]
    async fn test_security_setup_get_logical_name() {
        let setup = SecuritySetup::with_default_obis();
        let result = setup.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
                assert_eq!(bytes, SecuritySetup::default_obis().to_bytes());
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_security_suite() {
        use dlms_security::suite::{AuthenticationMechanism, EncryptionMechanism, SecurityPolicy};

        let suite = dlms_security::SecuritySuite::builder()
            .set_security_policy(SecurityPolicy::AuthenticatedAndEncrypted)
            .set_encryption_mechanism(EncryptionMechanism::AesGcm128)
            .set_authentication_mechanism(AuthenticationMechanism::Hls5Gmac)
            .set_global_unicast_encryption_key(vec![0u8; 16])
            .set_authentication_key(vec![0u8; 16])
            .build()
            .unwrap();

        let setup = SecuritySetup::new(SecuritySetup::default_obis(), suite);

        let retrieved_suite = setup.get_security_suite().await;
        assert_eq!(
            retrieved_suite.security_policy(),
            SecurityPolicy::AuthenticatedAndEncrypted
        );
        assert_eq!(
            retrieved_suite.encryption_mechanism(),
            EncryptionMechanism::AesGcm128
        );
    }

    #[tokio::test]
    async fn test_security_setup_get_security_policy() {
        use dlms_security::SecurityPolicy;

        let setup = SecuritySetup::with_default_obis();
        let result = setup.get_attribute(5, None).await.unwrap();

        match result {
            DataObject::Unsigned8(policy_id) => {
                assert_eq!(policy_id, SecurityPolicy::Nothing.id());
            }
            _ => panic!("Expected Unsigned8"),
        }
    }

    #[tokio::test]
    async fn test_security_setup_get_global_key() {
        let suite = dlms_security::SecuritySuite::builder()
            .set_global_unicast_encryption_key(vec![0x01, 0x02, 0x03, 0x04])
            .build()
            .unwrap();

        let setup = SecuritySetup::new(SecuritySetup::default_obis(), suite);
        let result = setup.get_attribute(6, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes, vec![0x01, 0x02, 0x03, 0x04]);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_security_setup_get_security_suite_attribute() {
        let setup = SecuritySetup::with_default_obis();
        let result = setup.get_attribute(2, None).await.unwrap();

        match result {
            DataObject::Structure(fields) => {
                assert_eq!(fields.len(), 3);
                // Field 0: Security policy
                // Field 1: Encryption mechanism
                // Field 2: Authentication mechanism
            }
            _ => panic!("Expected Structure"),
        }
    }

    #[tokio::test]
    async fn test_security_setup_set_security_suite() {
        use dlms_security::suite::{EncryptionMechanism, SecurityPolicy};

        let setup = SecuritySetup::with_default_obis();

        let new_suite = dlms_security::SecuritySuite::builder()
            .set_security_policy(SecurityPolicy::Encrypted)
            .set_encryption_mechanism(EncryptionMechanism::AesGcm128)
            .set_global_unicast_encryption_key(vec![0u8; 16])
            .build()
            .unwrap();

        setup.set_security_suite(new_suite).await;

        let retrieved = setup.get_security_suite().await;
        assert_eq!(retrieved.security_policy(), SecurityPolicy::Encrypted);
    }
}
