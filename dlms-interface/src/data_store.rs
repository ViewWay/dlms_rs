//! Data Store interface class (Class ID: 2)
//!
//! The Data Store interface class manages storage of arbitrary data.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: data_value - The stored data value
//! - Attribute 3: data_size - Maximum size of data in bytes
//! - Attribute 4: data_type - Type identifier for the stored data

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Data Type for Data Store
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DataStoreType {
    /// Binary data
    Binary = 0,
    /// Text data (UTF-8)
    Text = 1,
    /// JSON data
    Json = 2,
    /// XML data
    Xml = 3,
    /// Custom data type
    Custom = 255,
}

impl DataStoreType {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Binary,
            1 => Self::Text,
            2 => Self::Json,
            3 => Self::Xml,
            _ => Self::Custom,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Data Store interface class (Class ID: 2)
///
/// Default OBIS: 0-0:2.0.0.255
///
/// This class manages storage of arbitrary data.
#[derive(Debug, Clone)]
pub struct DataStore {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// The stored data value
    data_value: Arc<RwLock<Vec<u8>>>,

    /// Maximum size of data in bytes
    data_size: Arc<RwLock<u32>>,

    /// Type identifier for the stored data
    data_type: Arc<RwLock<DataStoreType>>,
}

impl DataStore {
    /// Class ID for DataStore
    pub const CLASS_ID: u16 = 2;

    /// Default OBIS code for DataStore (0-0:2.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 2, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_DATA_VALUE: u8 = 2;
    pub const ATTR_DATA_SIZE: u8 = 3;
    pub const ATTR_DATA_TYPE: u8 = 4;

    /// Create a new DataStore object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            data_value: Arc::new(RwLock::new(Vec::new())),
            data_size: Arc::new(RwLock::new(4096)),
            data_type: Arc::new(RwLock::new(DataStoreType::Binary)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Create with specific size limit
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `size` - Maximum data size in bytes
    pub fn with_size(logical_name: ObisCode, size: u32) -> Self {
        Self {
            logical_name,
            data_value: Arc::new(RwLock::new(Vec::new())),
            data_size: Arc::new(RwLock::new(size)),
            data_type: Arc::new(RwLock::new(DataStoreType::Binary)),
        }
    }

    /// Get the stored data
    pub async fn data(&self) -> Vec<u8> {
        self.data_value.read().await.clone()
    }

    /// Set the stored data
    pub async fn set_data(&self, data: Vec<u8>) -> DlmsResult<()> {
        let max_size = self.data_size().await;
        if data.len() as u32 > max_size {
            return Err(DlmsError::InvalidData(format!(
                "Data size {} exceeds maximum {}",
                data.len(),
                max_size
            )));
        }
        *self.data_value.write().await = data;
        Ok(())
    }

    /// Append data to the store
    pub async fn append_data(&self, data: &[u8]) -> DlmsResult<()> {
        let max_size = self.data_size().await;
        let current_size = self.data_value.read().await.len() as u32;
        if current_size + data.len() as u32 > max_size {
            return Err(DlmsError::InvalidData(format!(
                "Appending would exceed maximum size {}",
                max_size
            )));
        }
        self.data_value.write().await.extend_from_slice(data);
        Ok(())
    }

    /// Clear the stored data
    pub async fn clear(&self) {
        self.data_value.write().await.clear();
    }

    /// Get the current data size
    pub async fn current_size(&self) -> usize {
        self.data_value.read().await.len()
    }

    /// Get the maximum data size
    pub async fn data_size(&self) -> u32 {
        *self.data_size.read().await
    }

    /// Set the maximum data size
    pub async fn set_data_size(&self, size: u32) {
        *self.data_size.write().await = size;
    }

    /// Get the data type
    pub async fn data_type(&self) -> DataStoreType {
        *self.data_type.read().await
    }

    /// Set the data type
    pub async fn set_data_type(&self, data_type: DataStoreType) {
        *self.data_type.write().await = data_type;
    }

    /// Check if the store is empty
    pub async fn is_empty(&self) -> bool {
        self.data_value.read().await.is_empty()
    }

    /// Get the remaining capacity
    pub async fn remaining_capacity(&self) -> u32 {
        let max = self.data_size().await;
        let current = self.current_size().await as u32;
        max.saturating_sub(current)
    }
}

#[async_trait]
impl CosemObject for DataStore {
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
            Self::ATTR_DATA_VALUE => {
                Ok(DataObject::OctetString(self.data().await))
            }
            Self::ATTR_DATA_SIZE => {
                Ok(DataObject::Unsigned32(self.data_size().await))
            }
            Self::ATTR_DATA_TYPE => {
                Ok(DataObject::Enumerate(self.data_type().await.to_u8()))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "DataStore has no attribute {}",
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
            Self::ATTR_DATA_VALUE => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_data(bytes).await?;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for data_value".to_string(),
                    )),
                }
            }
            Self::ATTR_DATA_SIZE => {
                match value {
                    DataObject::Unsigned32(size) => {
                        self.set_data_size(size).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned32 for data_size".to_string(),
                    )),
                }
            }
            Self::ATTR_DATA_TYPE => {
                match value {
                    DataObject::Enumerate(data_type) => {
                        self.set_data_type(DataStoreType::from_u8(data_type)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for data_type".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "DataStore has no attribute {}",
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
            "DataStore has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_data_store_class_id() {
        let store = DataStore::with_default_obis();
        assert_eq!(store.class_id(), 2);
    }

    #[tokio::test]
    async fn test_data_store_obis_code() {
        let store = DataStore::with_default_obis();
        assert_eq!(store.obis_code(), DataStore::default_obis());
    }

    #[tokio::test]
    async fn test_data_store_type_from_u8() {
        assert_eq!(DataStoreType::from_u8(0), DataStoreType::Binary);
        assert_eq!(DataStoreType::from_u8(1), DataStoreType::Text);
        assert_eq!(DataStoreType::from_u8(2), DataStoreType::Json);
        assert_eq!(DataStoreType::from_u8(3), DataStoreType::Xml);
        assert_eq!(DataStoreType::from_u8(255), DataStoreType::Custom);
        assert_eq!(DataStoreType::from_u8(99), DataStoreType::Custom);
    }

    #[tokio::test]
    async fn test_data_store_initial_state() {
        let store = DataStore::with_default_obis();
        assert!(store.is_empty().await);
        assert_eq!(store.data_size().await, 4096);
        assert_eq!(store.data_type().await, DataStoreType::Binary);
    }

    #[tokio::test]
    async fn test_data_store_set_data() {
        let store = DataStore::with_default_obis();
        let data = vec![1, 2, 3, 4, 5];
        store.set_data(data.clone()).await.unwrap();
        assert_eq!(store.data().await, data);
        assert_eq!(store.current_size().await, 5);
    }

    #[tokio::test]
    async fn test_data_store_set_data_too_large() {
        let store = DataStore::with_size(ObisCode::new(0, 0, 2, 0, 0, 255), 10);
        let data = vec![1u8; 20];
        let result = store.set_data(data).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_data_store_append_data() {
        let store = DataStore::with_default_obis();
        store.set_data(vec![1, 2, 3]).await.unwrap();
        store.append_data(&[4, 5]).await.unwrap();
        assert_eq!(store.data().await, vec![1, 2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn test_data_store_append_too_much() {
        let store = DataStore::with_size(ObisCode::new(0, 0, 2, 0, 0, 255), 10);
        store.set_data(vec![1, 2, 3, 4, 5]).await.unwrap();
        let result = store.append_data(&[6, 7, 8, 9, 10, 11]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_data_store_clear() {
        let store = DataStore::with_default_obis();
        store.set_data(vec![1, 2, 3]).await.unwrap();
        assert!(!store.is_empty().await);

        store.clear().await;
        assert!(store.is_empty().await);
    }

    #[tokio::test]
    async fn test_data_store_set_data_size() {
        let store = DataStore::with_default_obis();
        store.set_data_size(8192).await;
        assert_eq!(store.data_size().await, 8192);
    }

    #[tokio::test]
    async fn test_data_store_set_data_type() {
        let store = DataStore::with_default_obis();
        store.set_data_type(DataStoreType::Text).await;
        assert_eq!(store.data_type().await, DataStoreType::Text);

        store.set_data_type(DataStoreType::Json).await;
        assert_eq!(store.data_type().await, DataStoreType::Json);
    }

    #[tokio::test]
    async fn test_data_store_remaining_capacity() {
        let store = DataStore::with_size(ObisCode::new(0, 0, 2, 0, 0, 255), 100);
        assert_eq!(store.remaining_capacity().await, 100);

        store.set_data(vec![1u8; 30]).await.unwrap();
        assert_eq!(store.remaining_capacity().await, 70);

        store.set_data(vec![1u8; 100]).await.unwrap();
        assert_eq!(store.remaining_capacity().await, 0);
    }

    #[tokio::test]
    async fn test_data_store_get_attributes() {
        let store = DataStore::with_default_obis();

        // Test data_size
        let result = store.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Unsigned32(size) => assert_eq!(size, 4096),
            _ => panic!("Expected Unsigned32"),
        }

        // Test data_type
        let result = store.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Enumerate(dt) => assert_eq!(dt, 0), // Binary
            _ => panic!("Expected Enumerate"),
        }
    }

    #[tokio::test]
    async fn test_data_store_set_attributes() {
        let store = DataStore::with_default_obis();

        store.set_attribute(2, DataObject::OctetString(vec![10, 20, 30]), None)
            .await
            .unwrap();
        assert_eq!(store.data().await, vec![10, 20, 30]);

        store.set_attribute(3, DataObject::Unsigned32(2048), None)
            .await
            .unwrap();
        assert_eq!(store.data_size().await, 2048);

        store.set_attribute(4, DataObject::Enumerate(1), None) // Text
            .await
            .unwrap();
        assert_eq!(store.data_type().await, DataStoreType::Text);
    }

    #[tokio::test]
    async fn test_data_store_read_only_logical_name() {
        let store = DataStore::with_default_obis();
        let result = store
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 2, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_data_store_invalid_attribute() {
        let store = DataStore::with_default_obis();
        let result = store.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_data_store_invalid_method() {
        let store = DataStore::with_default_obis();
        let result = store.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_data_store_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 2, 0, 0, 1);
        let store = DataStore::new(obis);
        assert_eq!(store.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_data_store_with_size_limit() {
        let store = DataStore::with_size(ObisCode::new(0, 0, 2, 0, 0, 255), 100);
        assert_eq!(store.data_size().await, 100);
        assert_eq!(store.remaining_capacity().await, 100);
    }

    #[tokio::test]
    async fn test_data_store_invalid_data_type_for_value() {
        let store = DataStore::with_default_obis();
        let result = store.set_attribute(2, DataObject::Boolean(true), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_data_store_invalid_data_type_for_size() {
        let store = DataStore::with_default_obis();
        let result = store.set_attribute(3, DataObject::Boolean(true), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_data_store_store_and_retrieve_text() {
        let store = DataStore::with_default_obis();
        store.set_data_type(DataStoreType::Text).await;

        let text = "Hello, World!".as_bytes().to_vec();
        store.set_data(text.clone()).await.unwrap();

        assert_eq!(store.data().await, text);
        assert_eq!(store.current_size().await, 13);
    }
}
