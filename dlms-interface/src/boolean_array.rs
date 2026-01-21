//! Boolean Array interface class (Class ID: 91)
//!
//! The Boolean Array interface class manages arrays of boolean values.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: value - The boolean array value
//! - Attribute 3: max_size - Maximum size of the array

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Boolean Array interface class (Class ID: 91)
///
/// Default OBIS: 0-0:91.0.0.255
///
/// This class manages arrays of boolean values.
#[derive(Debug, Clone)]
pub struct BooleanArray {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// The boolean array value
    value: Arc<RwLock<Vec<bool>>>,

    /// Maximum size of the array
    max_size: Arc<RwLock<usize>>,
}

impl BooleanArray {
    /// Class ID for BooleanArray
    pub const CLASS_ID: u16 = 91;

    /// Default OBIS code for BooleanArray (0-0:91.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 91, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_VALUE: u8 = 2;
    pub const ATTR_MAX_SIZE: u8 = 3;

    /// Create a new BooleanArray object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(Vec::new())),
            max_size: Arc::new(RwLock::new(255)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Create with specific max size
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `max_size` - Maximum size of the array
    pub fn with_max_size(logical_name: ObisCode, max_size: usize) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(Vec::new())),
            max_size: Arc::new(RwLock::new(max_size)),
        }
    }

    /// Create with initial value
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `initial_value` - Initial boolean array value
    pub fn with_value(logical_name: ObisCode, initial_value: Vec<bool>) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(initial_value)),
            max_size: Arc::new(RwLock::new(255)),
        }
    }

    /// Get the boolean array value
    pub async fn value(&self) -> Vec<bool> {
        self.value.read().await.clone()
    }

    /// Set the boolean array value
    pub async fn set_value(&self, value: Vec<bool>) -> DlmsResult<()> {
        let max_size = self.max_size().await;
        if value.len() > max_size {
            return Err(DlmsError::InvalidData(format!(
                "Boolean array size {} exceeds maximum {}",
                value.len(),
                max_size
            )));
        }
        *self.value.write().await = value;
        Ok(())
    }

    /// Get the max size
    pub async fn max_size(&self) -> usize {
        *self.max_size.read().await
    }

    /// Set the max size
    pub async fn set_max_size(&self, max_size: usize) {
        *self.max_size.write().await = max_size;
    }

    /// Get the current size
    pub async fn len(&self) -> usize {
        self.value.read().await.len()
    }

    /// Check if the array is empty
    pub async fn is_empty(&self) -> bool {
        self.value.read().await.is_empty()
    }

    /// Clear the array
    pub async fn clear(&self) {
        self.value.write().await.clear();
    }

    /// Get a boolean value at the specified index
    pub async fn get(&self, index: usize) -> DlmsResult<bool> {
        let value = self.value.read().await;
        if index >= value.len() {
            return Err(DlmsError::InvalidData(format!(
                "Index {} out of bounds (size: {})",
                index,
                value.len()
            )));
        }
        Ok(value[index])
    }

    /// Set a boolean value at the specified index
    pub async fn set(&self, index: usize, val: bool) -> DlmsResult<()> {
        let mut value = self.value.write().await;
        if index >= value.len() {
            return Err(DlmsError::InvalidData(format!(
                "Index {} out of bounds (size: {})",
                index,
                value.len()
            )));
        }
        value[index] = val;
        Ok(())
    }

    /// Append a boolean value to the array
    pub async fn push(&self, val: bool) -> DlmsResult<()> {
        let max_size = self.max_size().await;
        let current_len = self.len().await;
        if current_len >= max_size {
            return Err(DlmsError::InvalidData(format!(
                "Cannot push: array is at maximum size {}",
                max_size
            )));
        }
        self.value.write().await.push(val);
        Ok(())
    }

    /// Remove the last element from the array
    pub async fn pop(&self) -> DlmsResult<bool> {
        let mut value = self.value.write().await;
        if value.is_empty() {
            return Err(DlmsError::InvalidData(
                "Cannot pop: array is empty".to_string(),
            ));
        }
        Ok(value.pop().unwrap())
    }

    /// Get the remaining capacity
    pub async fn remaining_capacity(&self) -> usize {
        let max = self.max_size().await;
        let current = self.len().await;
        max.saturating_sub(current)
    }

    /// Count the number of `true` values
    pub async fn count_true(&self) -> usize {
        self.value.read().await.iter().filter(|&&v| v).count()
    }

    /// Count the number of `false` values
    pub async fn count_false(&self) -> usize {
        self.value.read().await.iter().filter(|&&v| !v).count()
    }

    /// Check if all values are true
    pub async fn all_true(&self) -> bool {
        let value = self.value.read().await;
        value.iter().all(|&v| v)
    }

    /// Check if all values are false
    pub async fn all_false(&self) -> bool {
        let value = self.value.read().await;
        value.iter().all(|&v| !v)
    }

    /// Check if any value is true
    pub async fn any_true(&self) -> bool {
        let value = self.value.read().await;
        value.iter().any(|&v| v)
    }

    /// Toggle all values (true -> false, false -> true)
    pub async fn toggle_all(&self) {
        let mut value = self.value.write().await;
        for v in value.iter_mut() {
            *v = !*v;
        }
    }

    /// Toggle a value at the specified index
    pub async fn toggle(&self, index: usize) -> DlmsResult<()> {
        let mut value = self.value.write().await;
        if index >= value.len() {
            return Err(DlmsError::InvalidData(format!(
                "Index {} out of bounds (size: {})",
                index,
                value.len()
            )));
        }
        value[index] = !value[index];
        Ok(())
    }

    /// Set all values to true
    pub async fn set_all_true(&self) {
        let mut value = self.value.write().await;
        for v in value.iter_mut() {
            *v = true;
        }
    }

    /// Set all values to false
    pub async fn set_all_false(&self) {
        let mut value = self.value.write().await;
        for v in value.iter_mut() {
            *v = false;
        }
    }

    /// Convert to a byte vector (bit packing)
    pub async fn to_bytes(&self) -> Vec<u8> {
        let value = self.value.read().await;
        let len = value.len();
        let byte_count = (len + 7) / 8;
        let mut bytes = vec![0u8; byte_count];

        for (i, &v) in value.iter().enumerate() {
            if v {
                bytes[i / 8] |= 1 << (i % 8);
            }
        }
        bytes
    }

    /// Set from a byte vector (bit unpacking)
    pub async fn from_bytes(&self, bytes: &[u8], bit_count: usize) -> DlmsResult<()> {
        let max_size = self.max_size().await;
        if bit_count > max_size {
            return Err(DlmsError::InvalidData(format!(
                "Bit count {} exceeds maximum {}",
                bit_count,
                max_size
            )));
        }

        let mut value = Vec::with_capacity(bit_count);
        for i in 0..bit_count {
            let byte_index = i / 8;
            let bit_index = i % 8;
            let bit = if byte_index < bytes.len() {
                (bytes[byte_index] >> bit_index) & 1 == 1
            } else {
                false
            };
            value.push(bit);
        }

        self.set_value(value).await
    }
}

#[async_trait]
impl CosemObject for BooleanArray {
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
                // Convert boolean array to DLMS format
                let value = self.value().await;
                Ok(DataObject::Array(
                    value.iter().map(|&b| DataObject::Boolean(b)).collect()
                ))
            }
            Self::ATTR_MAX_SIZE => {
                Ok(DataObject::Unsigned16(self.max_size().await as u16))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "BooleanArray has no attribute {}",
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
                    DataObject::Array(items) => {
                        let mut bool_array = Vec::new();
                        for item in items {
                            match item {
                                DataObject::Boolean(b) => bool_array.push(b),
                                _ => return Err(DlmsError::InvalidData(
                                    "Expected Boolean values in array".to_string(),
                                )),
                            }
                        }
                        self.set_value(bool_array).await?;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Array for value".to_string(),
                    )),
                }
            }
            Self::ATTR_MAX_SIZE => {
                match value {
                    DataObject::Unsigned16(max_size) => {
                        self.set_max_size(max_size as usize).await;
                        Ok(())
                    }
                    DataObject::Unsigned8(max_size) => {
                        self.set_max_size(max_size as usize).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned16/Unsigned8 for max_size".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "BooleanArray has no attribute {}",
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
            "BooleanArray has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_boolean_array_class_id() {
        let ba = BooleanArray::with_default_obis();
        assert_eq!(ba.class_id(), 91);
    }

    #[tokio::test]
    async fn test_boolean_array_obis_code() {
        let ba = BooleanArray::with_default_obis();
        assert_eq!(ba.obis_code(), BooleanArray::default_obis());
    }

    #[tokio::test]
    async fn test_boolean_array_initial_state() {
        let ba = BooleanArray::with_default_obis();
        assert!(ba.is_empty().await);
        assert_eq!(ba.len().await, 0);
        assert_eq!(ba.max_size().await, 255);
    }

    #[tokio::test]
    async fn test_boolean_array_set_value() {
        let ba = BooleanArray::with_default_obis();
        let data = vec![true, false, true, true, false];
        ba.set_value(data.clone()).await.unwrap();
        assert_eq!(ba.value().await, data);
        assert_eq!(ba.len().await, 5);
    }

    #[tokio::test]
    async fn test_boolean_array_set_value_too_large() {
        let ba = BooleanArray::with_max_size(ObisCode::new(0, 0, 91, 0, 0, 255), 5);
        let data = vec![true; 10];
        let result = ba.set_value(data).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_boolean_array_clear() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![true, false, true]).await.unwrap();
        assert!(!ba.is_empty().await);

        ba.clear().await;
        assert!(ba.is_empty().await);
    }

    #[tokio::test]
    async fn test_boolean_array_get() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![true, false, true]).await.unwrap();
        assert_eq!(ba.get(0).await.unwrap(), true);
        assert_eq!(ba.get(1).await.unwrap(), false);
        assert_eq!(ba.get(2).await.unwrap(), true);
    }

    #[tokio::test]
    async fn test_boolean_array_get_out_of_bounds() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![true, false]).await.unwrap();
        let result = ba.get(5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_boolean_array_set() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![true, false, true]).await.unwrap();
        ba.set(1, true).await.unwrap();
        assert_eq!(ba.get(1).await.unwrap(), true);
    }

    #[tokio::test]
    async fn test_boolean_array_set_out_of_bounds() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![true, false]).await.unwrap();
        let result = ba.set(5, true).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_boolean_array_push() {
        let ba = BooleanArray::with_default_obis();
        ba.push(true).await.unwrap();
        ba.push(false).await.unwrap();
        assert_eq!(ba.len().await, 2);
        assert_eq!(ba.value().await, vec![true, false]);
    }

    #[tokio::test]
    async fn test_boolean_array_push_too_much() {
        let ba = BooleanArray::with_max_size(ObisCode::new(0, 0, 91, 0, 0, 255), 3);
        ba.push(true).await.unwrap();
        ba.push(true).await.unwrap();
        ba.push(true).await.unwrap();
        let result = ba.push(true).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_boolean_array_pop() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![true, false, true]).await.unwrap();
        assert_eq!(ba.pop().await.unwrap(), true);
        assert_eq!(ba.len().await, 2);
    }

    #[tokio::test]
    async fn test_boolean_array_pop_empty() {
        let ba = BooleanArray::with_default_obis();
        let result = ba.pop().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_boolean_array_remaining_capacity() {
        let ba = BooleanArray::with_max_size(ObisCode::new(0, 0, 91, 0, 0, 255), 100);
        assert_eq!(ba.remaining_capacity().await, 100);

        ba.set_value(vec![true; 30]).await.unwrap();
        assert_eq!(ba.remaining_capacity().await, 70);
    }

    #[tokio::test]
    async fn test_boolean_array_count_true() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![true, false, true, true, false]).await.unwrap();
        assert_eq!(ba.count_true().await, 3);
    }

    #[tokio::test]
    async fn test_boolean_array_count_false() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![true, false, true, true, false]).await.unwrap();
        assert_eq!(ba.count_false().await, 2);
    }

    #[tokio::test]
    async fn test_boolean_array_all_true() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![true, true, true]).await.unwrap();
        assert!(ba.all_true().await);
    }

    #[tokio::test]
    async fn test_boolean_array_all_true_not_all() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![true, false, true]).await.unwrap();
        assert!(!ba.all_true().await);
    }

    #[tokio::test]
    async fn test_boolean_array_all_false() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![false, false, false]).await.unwrap();
        assert!(ba.all_false().await);
    }

    #[tokio::test]
    async fn test_boolean_array_any_true() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![false, true, false]).await.unwrap();
        assert!(ba.any_true().await);
    }

    #[tokio::test]
    async fn test_boolean_array_any_true_none() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![false, false, false]).await.unwrap();
        assert!(!ba.any_true().await);
    }

    #[tokio::test]
    async fn test_boolean_array_toggle_all() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![true, false, true]).await.unwrap();
        ba.toggle_all().await;
        assert_eq!(ba.value().await, vec![false, true, false]);
    }

    #[tokio::test]
    async fn test_boolean_array_toggle() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![true, false, true]).await.unwrap();
        ba.toggle(1).await.unwrap();
        assert_eq!(ba.get(1).await.unwrap(), true);
    }

    #[tokio::test]
    async fn test_boolean_array_toggle_out_of_bounds() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![true, false]).await.unwrap();
        let result = ba.toggle(5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_boolean_array_set_all_true() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![true, false, true]).await.unwrap();
        ba.set_all_true().await;
        assert!(ba.all_true().await);
    }

    #[tokio::test]
    async fn test_boolean_array_set_all_false() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![true, false, true]).await.unwrap();
        ba.set_all_false().await;
        assert!(ba.all_false().await);
    }

    #[tokio::test]
    async fn test_boolean_array_to_bytes() {
        let ba = BooleanArray::with_default_obis();
        ba.set_value(vec![true, false, true, true, false, true, false, true]).await.unwrap();
        let bytes = ba.to_bytes().await;
        // Index 0 = bit 0, Index 1 = bit 1, etc.
        // [true, false, true, true, false, true, false, true]
        // Bits: 0=1, 1=0, 2=1, 3=1, 4=0, 5=1, 6=0, 7=1
        // Byte: 10101101 = 0xAD
        assert_eq!(bytes[0], 0b10101101);
    }

    #[tokio::test]
    async fn test_boolean_array_from_bytes() {
        let ba = BooleanArray::with_max_size(ObisCode::new(0, 0, 91, 0, 0, 255), 16);
        // Byte 0xAB = 0b10101011
        // Bits: 0=1, 1=1, 2=0, 3=1, 4=0, 5=1, 6=0, 7=1
        ba.from_bytes(&[0xAB, 0x55], 16).await.unwrap();
        let value = ba.value().await;
        assert_eq!(value[0], true);   // bit 0 of 0xAB
        assert_eq!(value[1], true);   // bit 1 of 0xAB
        assert_eq!(value[2], false);  // bit 2 of 0xAB
        assert_eq!(value[3], true);   // bit 3 of 0xAB
    }

    #[tokio::test]
    async fn test_boolean_array_with_value() {
        let ba = BooleanArray::with_value(ObisCode::new(0, 0, 91, 0, 0, 255), vec![true, false, true]);
        assert_eq!(ba.value().await, vec![true, false, true]);
    }

    #[tokio::test]
    async fn test_boolean_array_get_attributes() {
        let ba = BooleanArray::with_value(ObisCode::new(0, 0, 91, 0, 0, 255), vec![true, false]);

        // Test value
        let result = ba.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Array(items) => {
                assert_eq!(items.len(), 2);
                match &items[0] {
                    DataObject::Boolean(b) => assert_eq!(*b, true),
                    _ => panic!("Expected Boolean"),
                }
            }
            _ => panic!("Expected Array"),
        }

        // Test max_size
        let result = ba.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Unsigned16(max_size) => assert_eq!(max_size, 255),
            _ => panic!("Expected Unsigned16"),
        }
    }

    #[tokio::test]
    async fn test_boolean_array_set_attributes() {
        let ba = BooleanArray::with_default_obis();

        ba.set_attribute(2, DataObject::Array(vec![
            DataObject::Boolean(true),
            DataObject::Boolean(false),
            DataObject::Boolean(true),
        ]), None).await.unwrap();
        assert_eq!(ba.value().await, vec![true, false, true]);

        ba.set_attribute(3, DataObject::Unsigned16(512), None)
            .await
            .unwrap();
        assert_eq!(ba.max_size().await, 512);
    }

    #[tokio::test]
    async fn test_boolean_array_set_max_size_u8() {
        let ba = BooleanArray::with_default_obis();
        ba.set_attribute(3, DataObject::Unsigned8(200), None)
            .await
            .unwrap();
        assert_eq!(ba.max_size().await, 200);
    }

    #[tokio::test]
    async fn test_boolean_array_read_only_logical_name() {
        let ba = BooleanArray::with_default_obis();
        let result = ba
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 91, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_boolean_array_invalid_attribute() {
        let ba = BooleanArray::with_default_obis();
        let result = ba.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_boolean_array_invalid_method() {
        let ba = BooleanArray::with_default_obis();
        let result = ba.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_boolean_array_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 91, 0, 0, 1);
        let ba = BooleanArray::new(obis);
        assert_eq!(ba.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_boolean_array_with_max_size() {
        let ba = BooleanArray::with_max_size(ObisCode::new(0, 0, 91, 0, 0, 255), 100);
        assert_eq!(ba.max_size().await, 100);
    }

    #[tokio::test]
    async fn test_boolean_array_set_max_size_direct() {
        let ba = BooleanArray::with_default_obis();
        ba.set_max_size(1024).await;
        assert_eq!(ba.max_size().await, 1024);
    }

    #[tokio::test]
    async fn test_boolean_array_invalid_value_type() {
        let ba = BooleanArray::with_default_obis();
        let result = ba.set_attribute(2, DataObject::OctetString(vec![1, 2]), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_boolean_array_invalid_array_element() {
        let ba = BooleanArray::with_default_obis();
        let result = ba.set_attribute(2, DataObject::Array(vec![
            DataObject::Boolean(true),
            DataObject::Unsigned8(42),
        ]), None).await;
        assert!(result.is_err());
    }
}
