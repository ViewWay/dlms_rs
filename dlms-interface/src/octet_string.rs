//! Octet String interface class (Class ID: 89)
//!
//! The Octet String interface class manages octet string (byte array) data.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: value - The octet string value
//! - Attribute 3: max_length - Maximum length of the octet string

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Octet String interface class (Class ID: 89)
///
/// Default OBIS: 0-0:89.0.0.255
///
/// This class manages octet string (byte array) data.
#[derive(Debug, Clone)]
pub struct OctetString {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// The octet string value
    value: Arc<RwLock<Vec<u8>>>,

    /// Maximum length of the octet string
    max_length: Arc<RwLock<usize>>,
}

impl OctetString {
    /// Class ID for OctetString
    pub const CLASS_ID: u16 = 89;

    /// Default OBIS code for OctetString (0-0:89.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 89, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_VALUE: u8 = 2;
    pub const ATTR_MAX_LENGTH: u8 = 3;

    /// Create a new OctetString object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(Vec::new())),
            max_length: Arc::new(RwLock::new(255)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Create with specific max length
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `max_length` - Maximum length of the string
    pub fn with_max_length(logical_name: ObisCode, max_length: usize) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(Vec::new())),
            max_length: Arc::new(RwLock::new(max_length)),
        }
    }

    /// Get the octet string value
    pub async fn value(&self) -> Vec<u8> {
        self.value.read().await.clone()
    }

    /// Set the octet string value
    pub async fn set_value(&self, value: Vec<u8>) -> DlmsResult<()> {
        let max_len = self.max_length().await;
        if value.len() > max_len {
            return Err(DlmsError::InvalidData(format!(
                "Octet string length {} exceeds maximum {}",
                value.len(),
                max_len
            )));
        }
        *self.value.write().await = value;
        Ok(())
    }

    /// Get the max length
    pub async fn max_length(&self) -> usize {
        *self.max_length.read().await
    }

    /// Set the max length
    pub async fn set_max_length(&self, max_length: usize) {
        *self.max_length.write().await = max_length;
    }

    /// Get the current length
    pub async fn len(&self) -> usize {
        self.value.read().await.len()
    }

    /// Check if the octet string is empty
    pub async fn is_empty(&self) -> bool {
        self.value.read().await.is_empty()
    }

    /// Clear the octet string
    pub async fn clear(&self) {
        self.value.write().await.clear();
    }

    /// Append bytes to the octet string
    pub async fn append(&self, bytes: &[u8]) -> DlmsResult<()> {
        let max_len = self.max_length().await;
        let current_len = self.len().await;
        if current_len + bytes.len() > max_len {
            return Err(DlmsError::InvalidData(format!(
                "Appending would exceed maximum length {}",
                max_len
            )));
        }
        self.value.write().await.extend_from_slice(bytes);
        Ok(())
    }

    /// Get the remaining capacity
    pub async fn remaining_capacity(&self) -> usize {
        let max = self.max_length().await;
        let current = self.len().await;
        max.saturating_sub(current)
    }

    /// Get the value as a hex string
    pub async fn to_hex_string(&self) -> String {
        let value = self.value().await;
        value.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Set from a hex string
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

        self.set_value(bytes).await
    }

    /// Set from a UTF-8 string
    pub async fn from_utf8_string(&self, s: &str) -> DlmsResult<()> {
        self.set_value(s.as_bytes().to_vec()).await
    }

    /// Get as a UTF-8 string (if valid)
    pub async fn to_utf8_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.value().await)
    }
}

#[async_trait]
impl CosemObject for OctetString {
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
            Self::ATTR_VALUE => {
                Ok(DataObject::OctetString(self.value().await))
            }
            Self::ATTR_MAX_LENGTH => {
                Ok(DataObject::Unsigned16(self.max_length().await as u16))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "OctetString has no attribute {}",
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
            Self::ATTR_VALUE => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_value(bytes).await?;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for value".to_string(),
                    )),
                }
            }
            Self::ATTR_MAX_LENGTH => {
                match value {
                    DataObject::Unsigned16(max_len) => {
                        self.set_max_length(max_len as usize).await;
                        Ok(())
                    }
                    DataObject::Unsigned8(max_len) => {
                        self.set_max_length(max_len as usize).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned16/Unsigned8 for max_length".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "OctetString has no attribute {}",
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
            "OctetString has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_octet_string_class_id() {
        let os = OctetString::with_default_obis();
        assert_eq!(os.class_id(), 89);
    }

    #[tokio::test]
    async fn test_octet_string_obis_code() {
        let os = OctetString::with_default_obis();
        assert_eq!(os.obis_code(), OctetString::default_obis());
    }

    #[tokio::test]
    async fn test_octet_string_initial_state() {
        let os = OctetString::with_default_obis();
        assert!(os.is_empty().await);
        assert_eq!(os.len().await, 0);
        assert_eq!(os.max_length().await, 255);
    }

    #[tokio::test]
    async fn test_octet_string_set_value() {
        let os = OctetString::with_default_obis();
        let data = vec![1, 2, 3, 4, 5];
        os.set_value(data.clone()).await.unwrap();
        assert_eq!(os.value().await, data);
        assert_eq!(os.len().await, 5);
    }

    #[tokio::test]
    async fn test_octet_string_set_value_too_long() {
        let os = OctetString::with_max_length(ObisCode::new(0, 0, 89, 0, 0, 255), 10);
        let data = vec![1u8; 20];
        let result = os.set_value(data).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_octet_string_clear() {
        let os = OctetString::with_default_obis();
        os.set_value(vec![1, 2, 3]).await.unwrap();
        assert!(!os.is_empty().await);

        os.clear().await;
        assert!(os.is_empty().await);
    }

    #[tokio::test]
    async fn test_octet_string_append() {
        let os = OctetString::with_default_obis();
        os.set_value(vec![1, 2, 3]).await.unwrap();
        os.append(&[4, 5]).await.unwrap();
        assert_eq!(os.value().await, vec![1, 2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn test_octet_string_append_too_much() {
        let os = OctetString::with_max_length(ObisCode::new(0, 0, 89, 0, 0, 255), 5);
        os.set_value(vec![1, 2, 3]).await.unwrap();
        let result = os.append(&[4, 5, 6]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_octet_string_remaining_capacity() {
        let os = OctetString::with_max_length(ObisCode::new(0, 0, 89, 0, 0, 255), 100);
        assert_eq!(os.remaining_capacity().await, 100);

        os.set_value(vec![1u8; 30]).await.unwrap();
        assert_eq!(os.remaining_capacity().await, 70);
    }

    #[tokio::test]
    async fn test_octet_string_to_hex_string() {
        let os = OctetString::with_default_obis();
        os.set_value(vec![0x01, 0x02, 0xAB, 0xFF]).await.unwrap();
        assert_eq!(os.to_hex_string().await, "0102abff");
    }

    #[tokio::test]
    async fn test_octet_string_from_hex_string() {
        let os = OctetString::with_default_obis();
        os.from_hex_string("0102ABFF").await.unwrap();
        assert_eq!(os.value().await, vec![0x01, 0x02, 0xAB, 0xFF]);
    }

    #[tokio::test]
    async fn test_octet_string_from_hex_string_with_prefix() {
        let os = OctetString::with_default_obis();
        os.from_hex_string("0x0102ABFF").await.unwrap();
        assert_eq!(os.value().await, vec![0x01, 0x02, 0xAB, 0xFF]);
    }

    #[tokio::test]
    async fn test_octet_string_from_hex_string_invalid_length() {
        let os = OctetString::with_default_obis();
        // "0102ABF" has 7 hex digits (odd length) - should fail
        let result = os.from_hex_string("0102ABF").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_octet_string_from_utf8_string() {
        let os = OctetString::with_default_obis();
        os.from_utf8_string("Hello").await.unwrap();
        assert_eq!(os.value().await, b"Hello".to_vec());
    }

    #[tokio::test]
    async fn test_octet_string_to_utf8_string() {
        let os = OctetString::with_default_obis();
        os.from_utf8_string("Hello").await.unwrap();
        assert_eq!(os.to_utf8_string().await.unwrap(), "Hello");
    }

    #[tokio::test]
    async fn test_octet_string_to_utf8_string_invalid() {
        let os = OctetString::with_default_obis();
        os.set_value(vec![0xFF, 0xFE]).await.unwrap();
        assert!(os.to_utf8_string().await.is_err());
    }

    #[tokio::test]
    async fn test_octet_string_get_attributes() {
        let os = OctetString::with_max_length(ObisCode::new(0, 0, 89, 0, 0, 255), 512);

        // Test value
        let result = os.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::OctetString(bytes) => assert!(bytes.is_empty()),
            _ => panic!("Expected OctetString"),
        }

        // Test max_length
        let result = os.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Unsigned16(max_len) => assert_eq!(max_len, 512),
            _ => panic!("Expected Unsigned16"),
        }
    }

    #[tokio::test]
    async fn test_octet_string_set_attributes() {
        let os = OctetString::with_default_obis();

        os.set_attribute(2, DataObject::OctetString(vec![10, 20, 30]), None)
            .await
            .unwrap();
        assert_eq!(os.value().await, vec![10, 20, 30]);

        os.set_attribute(3, DataObject::Unsigned16(512), None)
            .await
            .unwrap();
        assert_eq!(os.max_length().await, 512);
    }

    #[tokio::test]
    async fn test_octet_string_set_max_length_u8() {
        let os = OctetString::with_default_obis();
        os.set_attribute(3, DataObject::Unsigned8(200), None)
            .await
            .unwrap();
        assert_eq!(os.max_length().await, 200);
    }

    #[tokio::test]
    async fn test_octet_string_read_only_logical_name() {
        let os = OctetString::with_default_obis();
        let result = os
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 89, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_octet_string_invalid_attribute() {
        let os = OctetString::with_default_obis();
        let result = os.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_octet_string_invalid_method() {
        let os = OctetString::with_default_obis();
        let result = os.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_octet_string_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 89, 0, 0, 1);
        let os = OctetString::new(obis);
        assert_eq!(os.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_octet_string_with_max_length() {
        let os = OctetString::with_max_length(ObisCode::new(0, 0, 89, 0, 0, 255), 100);
        assert_eq!(os.max_length().await, 100);
    }

    #[tokio::test]
    async fn test_octet_string_set_max_length_direct() {
        let os = OctetString::with_default_obis();
        os.set_max_length(1024).await;
        assert_eq!(os.max_length().await, 1024);
    }

    #[tokio::test]
    async fn test_octet_string_invalid_hex_string() {
        let os = OctetString::with_default_obis();
        let result = os.from_hex_string("01gh").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_octet_string_from_hex_string_empty() {
        let os = OctetString::with_default_obis();
        os.from_hex_string("").await.unwrap();
        assert!(os.is_empty().await);
    }
}
