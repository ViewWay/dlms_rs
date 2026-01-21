//! Key Table interface class (Class ID: 101)
//!
//! The Key Table interface class manages encryption keys for DLMS/COSEM security.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: key_id - Current key identifier
//! - Attribute 3: key_version - Key version number
//! - Attribute 4: key_value - The key value (encrypted or protected)
//! - Attribute 5: key_type - Type of key (encryption, authentication, etc.)

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Key Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum KeyType {
    /// Unspecified
    Unspecified = 0,
    /// Encryption key
    Encryption = 1,
    /// Authentication key
    Authentication = 2,
    /// Broadcast key
    Broadcast = 3,
    /// Global key
    Global = 4,
    /// Dedicated key
    Dedicated = 5,
}

impl KeyType {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Unspecified,
            1 => Self::Encryption,
            2 => Self::Authentication,
            3 => Self::Broadcast,
            4 => Self::Global,
            5 => Self::Dedicated,
            _ => Self::Unspecified,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if this is an encryption key
    pub fn is_encryption(self) -> bool {
        matches!(self, Self::Encryption)
    }

    /// Check if this is an authentication key
    pub fn is_authentication(self) -> bool {
        matches!(self, Self::Authentication)
    }
}

/// Key Table interface class (Class ID: 101)
///
/// Default OBIS: 0-0:101.0.0.255
///
/// This class manages encryption keys for DLMS/COSEM security.
#[derive(Debug, Clone)]
pub struct KeyTable {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Current key identifier
    key_id: Arc<RwLock<u8>>,

    /// Key version number
    key_version: Arc<RwLock<u16>>,

    /// The key value (stored as bytes)
    key_value: Arc<RwLock<Vec<u8>>>,

    /// Type of key
    key_type: Arc<RwLock<KeyType>>,

    /// Maximum key size in bytes
    max_key_size: Arc<RwLock<usize>>,
}

impl KeyTable {
    /// Class ID for KeyTable
    pub const CLASS_ID: u16 = 101;

    /// Default OBIS code for KeyTable (0-0:101.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 101, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_KEY_ID: u8 = 2;
    pub const ATTR_KEY_VERSION: u8 = 3;
    pub const ATTR_KEY_VALUE: u8 = 4;
    pub const ATTR_KEY_TYPE: u8 = 5;

    /// Create a new KeyTable object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            key_id: Arc::new(RwLock::new(0)),
            key_version: Arc::new(RwLock::new(1)),
            key_value: Arc::new(RwLock::new(Vec::new())),
            key_type: Arc::new(RwLock::new(KeyType::Unspecified)),
            max_key_size: Arc::new(RwLock::new(32)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Create with specific key type
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `key_type` - Type of key
    pub fn with_key_type(logical_name: ObisCode, key_type: KeyType) -> Self {
        Self {
            logical_name,
            key_id: Arc::new(RwLock::new(0)),
            key_version: Arc::new(RwLock::new(1)),
            key_value: Arc::new(RwLock::new(Vec::new())),
            key_type: Arc::new(RwLock::new(key_type)),
            max_key_size: Arc::new(RwLock::new(32)),
        }
    }

    /// Get the key ID
    pub async fn key_id(&self) -> u8 {
        *self.key_id.read().await
    }

    /// Set the key ID
    pub async fn set_key_id(&self, id: u8) {
        *self.key_id.write().await = id;
    }

    /// Get the key version
    pub async fn key_version(&self) -> u16 {
        *self.key_version.read().await
    }

    /// Set the key version
    pub async fn set_key_version(&self, version: u16) {
        *self.key_version.write().await = version;
    }

    /// Increment the key version
    pub async fn increment_key_version(&self) {
        let version = self.key_version().await;
        self.set_key_version(version.wrapping_add(1)).await;
    }

    /// Get the key value
    pub async fn key_value(&self) -> Vec<u8> {
        self.key_value.read().await.clone()
    }

    /// Set the key value
    pub async fn set_key_value(&self, value: Vec<u8>) -> DlmsResult<()> {
        let max_size = self.max_key_size().await;
        if value.len() > max_size {
            return Err(DlmsError::InvalidData(format!(
                "Key size {} exceeds maximum {}",
                value.len(),
                max_size
            )));
        }
        *self.key_value.write().await = value;
        Ok(())
    }

    /// Get the key type
    pub async fn key_type(&self) -> KeyType {
        *self.key_type.read().await
    }

    /// Set the key type
    pub async fn set_key_type(&self, key_type: KeyType) {
        *self.key_type.write().await = key_type;
    }

    /// Get the max key size
    pub async fn max_key_size(&self) -> usize {
        *self.max_key_size.read().await
    }

    /// Set the max key size
    pub async fn set_max_key_size(&self, size: usize) {
        *self.max_key_size.write().await = size;
    }

    /// Get the key length
    pub async fn key_length(&self) -> usize {
        self.key_value.read().await.len()
    }

    /// Check if the key is set
    pub async fn is_key_set(&self) -> bool {
        !self.key_value.read().await.is_empty()
    }

    /// Clear the key value
    pub async fn clear_key(&self) {
        self.key_value.write().await.clear();
    }

    /// Check if this is an encryption key
    pub async fn is_encryption_key(&self) -> bool {
        self.key_type().await.is_encryption()
    }

    /// Check if this is an authentication key
    pub async fn is_authentication_key(&self) -> bool {
        self.key_type().await.is_authentication()
    }

    /// Get key as hex string
    pub async fn to_hex_string(&self) -> String {
        let key = self.key_value().await;
        key.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Set key from hex string
    pub async fn from_hex_string(&self, hex: &str) -> DlmsResult<()> {
        let hex = hex.trim_start_matches("0x");
        if hex.len() % 2 != 0 {
            return Err(DlmsError::InvalidData(
                "Hex string must have even length".to_string(),
            ));
        }

        let mut bytes = Vec::new();
        for i in (0..hex.len()).step_by(2) {
            let byte_str = &hex[i..i + 2];
            let byte = u8::from_str_radix(byte_str, 16).map_err(|_| {
                DlmsError::InvalidData(format!("Invalid hex string: {}", byte_str))
            })?;
            bytes.push(byte);
        }

        self.set_key_value(bytes).await
    }

    /// Check if key version matches
    pub async fn version_matches(&self, version: u16) -> bool {
        self.key_version().await == version
    }
}

#[async_trait]
impl CosemObject for KeyTable {
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
            Self::ATTR_KEY_ID => {
                Ok(DataObject::Unsigned8(self.key_id().await))
            }
            Self::ATTR_KEY_VERSION => {
                Ok(DataObject::Unsigned16(self.key_version().await))
            }
            Self::ATTR_KEY_VALUE => {
                Ok(DataObject::OctetString(self.key_value().await))
            }
            Self::ATTR_KEY_TYPE => {
                Ok(DataObject::Enumerate(self.key_type().await.to_u8()))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "KeyTable has no attribute {}",
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
                Err(DlmsError::AccessDenied(
                    "Attribute 1 (logical_name) is read-only".to_string(),
                ))
            }
            Self::ATTR_KEY_ID => {
                match value {
                    DataObject::Unsigned8(id) => {
                        self.set_key_id(id).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for key_id".to_string(),
                    )),
                }
            }
            Self::ATTR_KEY_VERSION => {
                match value {
                    DataObject::Unsigned16(version) => {
                        self.set_key_version(version).await;
                        Ok(())
                    }
                    DataObject::Unsigned8(version) => {
                        self.set_key_version(version as u16).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned16/Unsigned8 for key_version".to_string(),
                    )),
                }
            }
            Self::ATTR_KEY_VALUE => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_key_value(bytes).await?;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for key_value".to_string(),
                    )),
                }
            }
            Self::ATTR_KEY_TYPE => {
                match value {
                    DataObject::Enumerate(key_type) => {
                        self.set_key_type(KeyType::from_u8(key_type)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for key_type".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "KeyTable has no attribute {}",
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
        Err(DlmsError::InvalidData(format!(
            "KeyTable has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_key_table_class_id() {
        let kt = KeyTable::with_default_obis();
        assert_eq!(kt.class_id(), 101);
    }

    #[tokio::test]
    async fn test_key_table_obis_code() {
        let kt = KeyTable::with_default_obis();
        assert_eq!(kt.obis_code(), KeyTable::default_obis());
    }

    #[tokio::test]
    async fn test_key_table_initial_state() {
        let kt = KeyTable::with_default_obis();
        assert_eq!(kt.key_id().await, 0);
        assert_eq!(kt.key_version().await, 1);
        assert!(!kt.is_key_set().await);
        assert_eq!(kt.key_type().await, KeyType::Unspecified);
        assert_eq!(kt.max_key_size().await, 32);
    }

    #[tokio::test]
    async fn test_key_table_with_key_type() {
        let kt = KeyTable::with_key_type(ObisCode::new(0, 0, 101, 0, 0, 255), KeyType::Encryption);
        assert_eq!(kt.key_type().await, KeyType::Encryption);
    }

    #[tokio::test]
    async fn test_key_table_set_key_id() {
        let kt = KeyTable::with_default_obis();
        kt.set_key_id(5).await;
        assert_eq!(kt.key_id().await, 5);
    }

    #[tokio::test]
    async fn test_key_table_set_key_version() {
        let kt = KeyTable::with_default_obis();
        kt.set_key_version(10).await;
        assert_eq!(kt.key_version().await, 10);
    }

    #[tokio::test]
    async fn test_key_table_increment_key_version() {
        let kt = KeyTable::with_default_obis();
        assert_eq!(kt.key_version().await, 1);
        kt.increment_key_version().await;
        assert_eq!(kt.key_version().await, 2);
    }

    #[tokio::test]
    async fn test_key_table_set_key_value() {
        let kt = KeyTable::with_default_obis();
        let key = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        kt.set_key_value(key.clone()).await.unwrap();
        assert_eq!(kt.key_value().await, key);
        assert_eq!(kt.key_length().await, 8);
    }

    #[tokio::test]
    async fn test_key_table_set_key_value_too_large() {
        let kt = KeyTable::with_default_obis();
        let key = vec![1u8; 64]; // Exceeds max of 32
        let result = kt.set_key_value(key).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_key_table_set_key_type() {
        let kt = KeyTable::with_default_obis();
        kt.set_key_type(KeyType::Authentication).await;
        assert_eq!(kt.key_type().await, KeyType::Authentication);
    }

    #[tokio::test]
    async fn test_key_table_is_encryption_key() {
        let kt = KeyTable::with_key_type(ObisCode::new(0, 0, 101, 0, 0, 255), KeyType::Encryption);
        assert!(kt.is_encryption_key().await);
        assert!(!kt.is_authentication_key().await);
    }

    #[tokio::test]
    async fn test_key_table_is_authentication_key() {
        let kt = KeyTable::with_key_type(ObisCode::new(0, 0, 101, 0, 0, 255), KeyType::Authentication);
        assert!(kt.is_authentication_key().await);
        assert!(!kt.is_encryption_key().await);
    }

    #[tokio::test]
    async fn test_key_table_clear_key() {
        let kt = KeyTable::with_default_obis();
        kt.set_key_value(vec![1, 2, 3, 4]).await.unwrap();
        assert!(kt.is_key_set().await);

        kt.clear_key().await;
        assert!(!kt.is_key_set().await);
    }

    #[tokio::test]
    async fn test_key_table_to_hex_string() {
        let kt = KeyTable::with_default_obis();
        kt.set_key_value(vec![0x01, 0x02, 0xAB, 0xFF]).await.unwrap();
        assert_eq!(kt.to_hex_string().await, "0102abff");
    }

    #[tokio::test]
    async fn test_key_table_from_hex_string() {
        let kt = KeyTable::with_default_obis();
        kt.from_hex_string("0102ABFF").await.unwrap();
        assert_eq!(kt.key_value().await, vec![0x01, 0x02, 0xAB, 0xFF]);
    }

    #[tokio::test]
    async fn test_key_table_from_hex_string_with_prefix() {
        let kt = KeyTable::with_default_obis();
        kt.from_hex_string("0x0102ABFF").await.unwrap();
        assert_eq!(kt.key_value().await, vec![0x01, 0x02, 0xAB, 0xFF]);
    }

    #[tokio::test]
    async fn test_key_table_from_hex_string_invalid_length() {
        let kt = KeyTable::with_default_obis();
        let result = kt.from_hex_string("0102ABF").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_key_table_version_matches() {
        let kt = KeyTable::with_default_obis();
        assert!(kt.version_matches(1).await);
        assert!(!kt.version_matches(2).await);
    }

    #[tokio::test]
    async fn test_key_type_from_u8() {
        assert_eq!(KeyType::from_u8(0), KeyType::Unspecified);
        assert_eq!(KeyType::from_u8(1), KeyType::Encryption);
        assert_eq!(KeyType::from_u8(2), KeyType::Authentication);
        assert_eq!(KeyType::from_u8(3), KeyType::Broadcast);
        assert_eq!(KeyType::from_u8(4), KeyType::Global);
        assert_eq!(KeyType::from_u8(5), KeyType::Dedicated);
    }

    #[tokio::test]
    async fn test_key_type_is_encryption() {
        assert!(KeyType::Encryption.is_encryption());
        assert!(!KeyType::Authentication.is_encryption());
    }

    #[tokio::test]
    async fn test_key_type_is_authentication() {
        assert!(KeyType::Authentication.is_authentication());
        assert!(!KeyType::Encryption.is_authentication());
    }

    #[tokio::test]
    async fn test_key_table_get_attributes() {
        let kt = KeyTable::with_default_obis();

        // Test key_id
        let result = kt.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Unsigned8(id) => assert_eq!(id, 0),
            _ => panic!("Expected Unsigned8"),
        }

        // Test key_version
        let result = kt.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Unsigned16(version) => assert_eq!(version, 1),
            _ => panic!("Expected Unsigned16"),
        }

        // Test key_type
        let result = kt.get_attribute(5, None).await.unwrap();
        match result {
            DataObject::Enumerate(key_type) => assert_eq!(key_type, 0), // Unspecified
            _ => panic!("Expected Enumerate"),
        }
    }

    #[tokio::test]
    async fn test_key_table_set_attributes() {
        let kt = KeyTable::with_default_obis();

        kt.set_attribute(2, DataObject::Unsigned8(5), None)
            .await
            .unwrap();
        assert_eq!(kt.key_id().await, 5);

        kt.set_attribute(3, DataObject::Unsigned16(10), None)
            .await
            .unwrap();
        assert_eq!(kt.key_version().await, 10);

        kt.set_attribute(5, DataObject::Enumerate(1), None)
            .await
            .unwrap();
        assert_eq!(kt.key_type().await, KeyType::Encryption);
    }

    #[tokio::test]
    async fn test_key_table_set_key_version_u8() {
        let kt = KeyTable::with_default_obis();
        kt.set_attribute(3, DataObject::Unsigned8(20), None)
            .await
            .unwrap();
        assert_eq!(kt.key_version().await, 20);
    }

    #[tokio::test]
    async fn test_key_table_read_only_logical_name() {
        let kt = KeyTable::with_default_obis();
        let result = kt
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 101, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_key_table_invalid_attribute() {
        let kt = KeyTable::with_default_obis();
        let result = kt.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_key_table_invalid_method() {
        let kt = KeyTable::with_default_obis();
        let result = kt.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_key_table_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 101, 0, 0, 1);
        let kt = KeyTable::new(obis);
        assert_eq!(kt.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_key_table_set_max_key_size() {
        let kt = KeyTable::with_default_obis();
        kt.set_max_key_size(64).await;
        assert_eq!(kt.max_key_size().await, 64);

        // Now we can set a larger key
        let key = vec![1u8; 48];
        kt.set_key_value(key).await.unwrap();
    }
}
