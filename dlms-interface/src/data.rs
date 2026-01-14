//! Data interface class (Class ID: 1)
//!
//! The Data interface class is the simplest COSEM interface class.
//! It represents a single data value that can be read and optionally written.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: value (DataObject) - The data value
//!
//! # Methods
//!
//! None
//!
//! # Usage
//!
//! ```rust,no_run
//! use dlms_interface::data::Data;
//! use dlms_core::{ObisCode, DataObject};
//!
//! // Create a Data object
//! let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
//! let value = DataObject::Integer32(12345);
//! let data = Data::new(obis, value);
//!
//! // Get the value
//! let current_value = data.value();
//! ```

use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_server::CosemObject;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Data interface class (Class ID: 1)
///
/// The Data interface class represents a single data value.
/// It is the simplest COSEM interface class and serves as a foundation
/// for more complex interface classes.
#[derive(Debug, Clone)]
pub struct Data {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,
    /// The data value
    value: Arc<RwLock<DataObject>>,
}

impl Data {
    /// Create a new Data object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `value` - Initial data value
    ///
    /// # Returns
    /// A new Data instance
    pub fn new(logical_name: ObisCode, value: DataObject) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(value)),
        }
    }

    /// Get the current value
    ///
    /// # Returns
    /// A copy of the current data value
    pub async fn value(&self) -> DataObject {
        self.value.read().await.clone()
    }

    /// Set the value
    ///
    /// # Arguments
    /// * `new_value` - New data value to set
    pub async fn set_value(&self, new_value: DataObject) {
        *self.value.write().await = new_value;
    }

    /// Get the logical name (OBIS code)
    pub fn logical_name(&self) -> ObisCode {
        self.logical_name
    }
}

#[async_trait::async_trait]
impl CosemObject for Data {
    fn class_id(&self) -> u16 {
        1 // Data interface class ID
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
            1 => {
                // Attribute 1: logical_name (OBIS code)
                Ok(DataObject::OctetString(self.logical_name.to_bytes().to_vec()))
            }
            2 => {
                // Attribute 2: value
                Ok(self.value().await)
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Data interface class has no attribute {}",
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
            1 => {
                // Attribute 1: logical_name is read-only
                Err(DlmsError::AccessDenied(
                    "Attribute 1 (logical_name) is read-only".to_string(),
                ))
            }
            2 => {
                // Attribute 2: value
                self.set_value(value).await;
                Ok(())
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Data interface class has no attribute {}",
                attribute_id
            ))),
        }
    }

    async fn invoke_method(
        &self,
        method_id: u8,
        _parameters: Option<DataObject>,
    ) -> DlmsResult<Option<DataObject>> {
        Err(DlmsError::InvalidData(format!(
            "Data interface class has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_data_creation() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Integer32(12345);
        let data = Data::new(obis, value.clone());

        assert_eq!(data.class_id(), 1);
        assert_eq!(data.obis_code(), obis);
        assert_eq!(data.value().await, value);
    }

    #[tokio::test]
    async fn test_data_get_attribute() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Integer32(12345);
        let data = Data::new(obis, value.clone());

        // Get attribute 1 (logical_name)
        let attr1 = data.get_attribute(1, None).await.unwrap();
        if let DataObject::OctetString(bytes) = attr1 {
            assert_eq!(bytes, obis.to_bytes());
        } else {
            panic!("Attribute 1 should be OctetString");
        }

        // Get attribute 2 (value)
        let attr2 = data.get_attribute(2, None).await.unwrap();
        assert_eq!(attr2, value);
    }

    #[tokio::test]
    async fn test_data_set_attribute() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let initial_value = DataObject::Integer32(12345);
        let data = Data::new(obis, initial_value);

        // Set attribute 2 (value)
        let new_value = DataObject::Integer32(67890);
        data.set_attribute(2, new_value.clone(), None).await.unwrap();

        // Verify the value was updated
        let current_value = data.get_attribute(2, None).await.unwrap();
        assert_eq!(current_value, new_value);

        // Try to set attribute 1 (should fail - read-only)
        let result = data
            .set_attribute(1, DataObject::OctetString(vec![1, 2, 3, 4, 5, 6]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_data_invalid_attribute() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Integer32(12345);
        let data = Data::new(obis, value);

        // Try to get non-existent attribute
        let result = data.get_attribute(99, None).await;
        assert!(result.is_err());

        // Try to set non-existent attribute
        let result = data.set_attribute(99, DataObject::Integer32(0), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_data_no_methods() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Integer32(12345);
        let data = Data::new(obis, value);

        // Try to invoke a method (should fail - no methods)
        let result = data.invoke_method(1, None).await;
        assert!(result.is_err());
    }
}
