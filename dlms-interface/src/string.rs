//! String interface class (Class ID: 90)
//!
//! The String interface class manages UTF-8 string data.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: value - The string value
//! - Attribute 3: max_length - Maximum length of the string (in characters)

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// String interface class (Class ID: 90)
///
/// Default OBIS: 0-0:90.0.0.255
///
/// This class manages UTF-8 string data.
#[derive(Debug, Clone)]
pub struct StringInterface {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// The string value
    value: Arc<RwLock<String>>,

    /// Maximum length of the string (in characters)
    max_length: Arc<RwLock<usize>>,
}

impl StringInterface {
    /// Class ID for StringInterface
    pub const CLASS_ID: u16 = 90;

    /// Default OBIS code for StringInterface (0-0:90.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 90, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_VALUE: u8 = 2;
    pub const ATTR_MAX_LENGTH: u8 = 3;

    /// Create a new StringInterface object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(String::new())),
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
            value: Arc::new(RwLock::new(String::new())),
            max_length: Arc::new(RwLock::new(max_length)),
        }
    }

    /// Create with initial value
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `initial_value` - Initial string value
    pub fn with_value(logical_name: ObisCode, initial_value: String) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(initial_value)),
            max_length: Arc::new(RwLock::new(255)),
        }
    }

    /// Get the string value
    pub async fn value(&self) -> String {
        self.value.read().await.clone()
    }

    /// Set the string value
    pub async fn set_value(&self, value: String) -> DlmsResult<()> {
        let max_len = self.max_length().await;
        if value.chars().count() > max_len {
            return Err(DlmsError::InvalidData(format!(
                "String length {} exceeds maximum {}",
                value.chars().count(),
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

    /// Get the current character count
    pub async fn len(&self) -> usize {
        self.value.read().await.chars().count()
    }

    /// Get the current byte count
    pub async fn byte_len(&self) -> usize {
        self.value.read().await.len()
    }

    /// Check if the string is empty
    pub async fn is_empty(&self) -> bool {
        self.value.read().await.is_empty()
    }

    /// Clear the string
    pub async fn clear(&self) {
        self.value.write().await.clear();
    }

    /// Append text to the string
    pub async fn append(&self, text: &str) -> DlmsResult<()> {
        let max_len = self.max_length().await;
        let current_len = self.len().await;
        let append_len = text.chars().count();
        if current_len + append_len > max_len {
            return Err(DlmsError::InvalidData(format!(
                "Appending would exceed maximum length {}",
                max_len
            )));
        }
        self.value.write().await.push_str(text);
        Ok(())
    }

    /// Get the remaining capacity (in characters)
    pub async fn remaining_capacity(&self) -> usize {
        let max = self.max_length().await;
        let current = self.len().await;
        max.saturating_sub(current)
    }

    /// Check if the string contains a substring
    pub async fn contains(&self, pattern: &str) -> bool {
        self.value.read().await.contains(pattern)
    }

    /// Get a substring (by character indices)
    pub async fn substring(&self, start: usize, end: usize) -> String {
        let value = self.value.read().await;
        value.chars().skip(start).take(end - start).collect()
    }

    /// Trim whitespace from the string
    pub async fn trim(&self) -> String {
        let value = self.value.read().await;
        value.trim().to_string()
    }

    /// Convert to uppercase
    pub async fn to_uppercase(&self) -> String {
        let value = self.value.read().await;
        value.to_uppercase()
    }

    /// Convert to lowercase
    pub async fn to_lowercase(&self) -> String {
        let value = self.value.read().await;
        value.to_lowercase()
    }

    /// Split the string by a delimiter
    pub async fn split(&self, delimiter: char) -> Vec<String> {
        let value = self.value.read().await;
        value.split(delimiter).map(|s| s.to_string()).collect()
    }
}

#[async_trait]
impl CosemObject for StringInterface {
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
                Ok(DataObject::OctetString(self.value().await.into_bytes()))
            }
            Self::ATTR_MAX_LENGTH => {
                Ok(DataObject::Unsigned16(self.max_length().await as u16))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "StringInterface has no attribute {}",
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
                        let string_value = String::from_utf8_lossy(&bytes).to_string();
                        self.set_value(string_value).await?;
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
                "StringInterface has no attribute {}",
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
            "StringInterface has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_string_class_id() {
        let s = StringInterface::with_default_obis();
        assert_eq!(s.class_id(), 90);
    }

    #[tokio::test]
    async fn test_string_obis_code() {
        let s = StringInterface::with_default_obis();
        assert_eq!(s.obis_code(), StringInterface::default_obis());
    }

    #[tokio::test]
    async fn test_string_initial_state() {
        let s = StringInterface::with_default_obis();
        assert!(s.is_empty().await);
        assert_eq!(s.len().await, 0);
        assert_eq!(s.byte_len().await, 0);
        assert_eq!(s.max_length().await, 255);
    }

    #[tokio::test]
    async fn test_string_set_value() {
        let s = StringInterface::with_default_obis();
        s.set_value("Hello".to_string()).await.unwrap();
        assert_eq!(s.value().await, "Hello");
        assert_eq!(s.len().await, 5);
        assert_eq!(s.byte_len().await, 5);
    }

    #[tokio::test]
    async fn test_string_set_value_unicode() {
        let s = StringInterface::with_default_obis();
        s.set_value("你好世界".to_string()).await.unwrap();
        assert_eq!(s.value().await, "你好世界");
        assert_eq!(s.len().await, 4);  // 4 characters
        assert_eq!(s.byte_len().await, 12);  // 12 bytes in UTF-8
    }

    #[tokio::test]
    async fn test_string_set_value_too_long() {
        let s = StringInterface::with_max_length(ObisCode::new(0, 0, 90, 0, 0, 255), 10);
        let result = s.set_value("This is way too long".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_string_clear() {
        let s = StringInterface::with_default_obis();
        s.set_value("Hello".to_string()).await.unwrap();
        assert!(!s.is_empty().await);

        s.clear().await;
        assert!(s.is_empty().await);
    }

    #[tokio::test]
    async fn test_string_append() {
        let s = StringInterface::with_default_obis();
        s.set_value("Hello".to_string()).await.unwrap();
        s.append(" World").await.unwrap();
        assert_eq!(s.value().await, "Hello World");
    }

    #[tokio::test]
    async fn test_string_append_too_much() {
        let s = StringInterface::with_max_length(ObisCode::new(0, 0, 90, 0, 0, 255), 5);
        s.set_value("Hi".to_string()).await.unwrap();
        let result = s.append("World").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_string_remaining_capacity() {
        let s = StringInterface::with_max_length(ObisCode::new(0, 0, 90, 0, 0, 255), 100);
        assert_eq!(s.remaining_capacity().await, 100);

        s.set_value("Hello".to_string()).await.unwrap();
        assert_eq!(s.remaining_capacity().await, 95);
    }

    #[tokio::test]
    async fn test_string_contains() {
        let s = StringInterface::with_default_obis();
        s.set_value("Hello World".to_string()).await.unwrap();
        assert!(s.contains("World").await);
        assert!(!s.contains("world").await);  // Case sensitive
    }

    #[tokio::test]
    async fn test_string_substring() {
        let s = StringInterface::with_default_obis();
        s.set_value("Hello World".to_string()).await.unwrap();
        assert_eq!(s.substring(0, 5).await, "Hello");
        assert_eq!(s.substring(6, 11).await, "World");
    }

    #[tokio::test]
    async fn test_string_trim() {
        let s = StringInterface::with_default_obis();
        s.set_value("  Hello  ".to_string()).await.unwrap();
        assert_eq!(s.trim().await, "Hello");
    }

    #[tokio::test]
    async fn test_string_to_uppercase() {
        let s = StringInterface::with_default_obis();
        s.set_value("hello".to_string()).await.unwrap();
        assert_eq!(s.to_uppercase().await, "HELLO");
    }

    #[tokio::test]
    async fn test_string_to_lowercase() {
        let s = StringInterface::with_default_obis();
        s.set_value("HELLO".to_string()).await.unwrap();
        assert_eq!(s.to_lowercase().await, "hello");
    }

    #[tokio::test]
    async fn test_string_split() {
        let s = StringInterface::with_default_obis();
        s.set_value("a,b,c".to_string()).await.unwrap();
        let result = s.split(',').await;
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[tokio::test]
    async fn test_string_with_value() {
        let s = StringInterface::with_value(ObisCode::new(0, 0, 90, 0, 0, 255), "Initial".to_string());
        assert_eq!(s.value().await, "Initial");
    }

    #[tokio::test]
    async fn test_string_get_attributes() {
        let s = StringInterface::with_value(ObisCode::new(0, 0, 90, 0, 0, 255), "Test".to_string());

        // Test value
        let result = s.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(String::from_utf8_lossy(&bytes), "Test");
            }
            _ => panic!("Expected OctetString"),
        }

        // Test max_length
        let result = s.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Unsigned16(max_len) => assert_eq!(max_len, 255),
            _ => panic!("Expected Unsigned16"),
        }
    }

    #[tokio::test]
    async fn test_string_set_attributes() {
        let s = StringInterface::with_default_obis();

        s.set_attribute(2, DataObject::OctetString(b"Hello".to_vec()), None)
            .await
            .unwrap();
        assert_eq!(s.value().await, "Hello");

        s.set_attribute(3, DataObject::Unsigned16(512), None)
            .await
            .unwrap();
        assert_eq!(s.max_length().await, 512);
    }

    #[tokio::test]
    async fn test_string_set_max_length_u8() {
        let s = StringInterface::with_default_obis();
        s.set_attribute(3, DataObject::Unsigned8(200), None)
            .await
            .unwrap();
        assert_eq!(s.max_length().await, 200);
    }

    #[tokio::test]
    async fn test_string_set_max_length_direct() {
        let s = StringInterface::with_default_obis();
        s.set_max_length(1024).await;
        assert_eq!(s.max_length().await, 1024);
    }

    #[tokio::test]
    async fn test_string_read_only_logical_name() {
        let s = StringInterface::with_default_obis();
        let result = s
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 90, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_string_invalid_attribute() {
        let s = StringInterface::with_default_obis();
        let result = s.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_string_invalid_method() {
        let s = StringInterface::with_default_obis();
        let result = s.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_string_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 90, 0, 0, 1);
        let s = StringInterface::new(obis);
        assert_eq!(s.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_string_with_max_length() {
        let s = StringInterface::with_max_length(ObisCode::new(0, 0, 90, 0, 0, 255), 100);
        assert_eq!(s.max_length().await, 100);
    }

    #[tokio::test]
    async fn test_string_unicode_substring() {
        let s = StringInterface::with_default_obis();
        s.set_value("你好世界".to_string()).await.unwrap();
        assert_eq!(s.substring(0, 2).await, "你好");
    }

    #[tokio::test]
    async fn test_string_unicode_capacity() {
        let s = StringInterface::with_max_length(ObisCode::new(0, 0, 90, 0, 0, 255), 10);
        s.set_value("你好世界".to_string()).await.unwrap();
        assert_eq!(s.remaining_capacity().await, 6);  // 10 - 4 characters
    }
}
