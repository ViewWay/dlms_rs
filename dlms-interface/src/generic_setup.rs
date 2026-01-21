//! Generic Setup interface class (Class ID: 26)
//!
//! The Generic Setup interface class manages generic configuration parameters
//! that can be used for various custom setup needs.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: parameter_1 - First configurable parameter
//! - Attribute 3: parameter_2 - Second configurable parameter
//! - Attribute 3: parameter_3 - Third configurable parameter
//! - Attribute 5: parameter_4 - Fourth configurable parameter
//! - Attribute 6: parameter_5 - Fifth configurable parameter
//! - Attribute 7: parameter_6 - Sixth configurable parameter
//! - Attribute 8: parameter_7 - Seventh configurable parameter
//! - Attribute 9: parameter_8 - Eighth configurable parameter

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Generic Setup interface class (Class ID: 26)
///
/// Default OBIS: 0-0:26.0.0.255
///
/// This class manages generic configuration parameters that can be used
/// for various custom setup needs in smart metering applications.
#[derive(Debug, Clone)]
pub struct GenericSetup {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Parameter 1 (unsigned 8-bit)
    parameter_1: Arc<RwLock<u8>>,

    /// Parameter 2 (unsigned 16-bit)
    parameter_2: Arc<RwLock<u16>>,

    /// Parameter 3 (unsigned 32-bit)
    parameter_3: Arc<RwLock<u32>>,

    /// Parameter 4 (signed 8-bit)
    parameter_4: Arc<RwLock<i8>>,

    /// Parameter 5 (signed 16-bit)
    parameter_5: Arc<RwLock<i16>>,

    /// Parameter 6 (signed 32-bit)
    parameter_6: Arc<RwLock<i32>>,

    /// Parameter 7 (boolean)
    parameter_7: Arc<RwLock<bool>>,

    /// Parameter 8 (octet string)
    parameter_8: Arc<RwLock<Vec<u8>>>,
}

impl GenericSetup {
    /// Class ID for GenericSetup
    pub const CLASS_ID: u16 = 26;

    /// Default OBIS code for GenericSetup (0-0:26.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 26, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_PARAMETER_1: u8 = 2;
    pub const ATTR_PARAMETER_2: u8 = 3;
    pub const ATTR_PARAMETER_3: u8 = 4;
    pub const ATTR_PARAMETER_4: u8 = 5;
    pub const ATTR_PARAMETER_5: u8 = 6;
    pub const ATTR_PARAMETER_6: u8 = 7;
    pub const ATTR_PARAMETER_7: u8 = 8;
    pub const ATTR_PARAMETER_8: u8 = 9;

    /// Create a new GenericSetup object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            parameter_1: Arc::new(RwLock::new(0)),
            parameter_2: Arc::new(RwLock::new(0)),
            parameter_3: Arc::new(RwLock::new(0)),
            parameter_4: Arc::new(RwLock::new(0)),
            parameter_5: Arc::new(RwLock::new(0)),
            parameter_6: Arc::new(RwLock::new(0)),
            parameter_7: Arc::new(RwLock::new(false)),
            parameter_8: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get parameter 1 (unsigned 8-bit)
    pub async fn parameter_1(&self) -> u8 {
        *self.parameter_1.read().await
    }

    /// Set parameter 1
    pub async fn set_parameter_1(&self, value: u8) {
        *self.parameter_1.write().await = value;
    }

    /// Get parameter 2 (unsigned 16-bit)
    pub async fn parameter_2(&self) -> u16 {
        *self.parameter_2.read().await
    }

    /// Set parameter 2
    pub async fn set_parameter_2(&self, value: u16) {
        *self.parameter_2.write().await = value;
    }

    /// Get parameter 3 (unsigned 32-bit)
    pub async fn parameter_3(&self) -> u32 {
        *self.parameter_3.read().await
    }

    /// Set parameter 3
    pub async fn set_parameter_3(&self, value: u32) {
        *self.parameter_3.write().await = value;
    }

    /// Get parameter 4 (signed 8-bit)
    pub async fn parameter_4(&self) -> i8 {
        *self.parameter_4.read().await
    }

    /// Set parameter 4
    pub async fn set_parameter_4(&self, value: i8) {
        *self.parameter_4.write().await = value;
    }

    /// Get parameter 5 (signed 16-bit)
    pub async fn parameter_5(&self) -> i16 {
        *self.parameter_5.read().await
    }

    /// Set parameter 5
    pub async fn set_parameter_5(&self, value: i16) {
        *self.parameter_5.write().await = value;
    }

    /// Get parameter 6 (signed 32-bit)
    pub async fn parameter_6(&self) -> i32 {
        *self.parameter_6.read().await
    }

    /// Set parameter 6
    pub async fn set_parameter_6(&self, value: i32) {
        *self.parameter_6.write().await = value;
    }

    /// Get parameter 7 (boolean)
    pub async fn parameter_7(&self) -> bool {
        *self.parameter_7.read().await
    }

    /// Set parameter 7
    pub async fn set_parameter_7(&self, value: bool) {
        *self.parameter_7.write().await = value;
    }

    /// Get parameter 8 (octet string)
    pub async fn parameter_8(&self) -> Vec<u8> {
        self.parameter_8.read().await.clone()
    }

    /// Set parameter 8
    pub async fn set_parameter_8(&self, value: Vec<u8>) {
        *self.parameter_8.write().await = value;
    }

    /// Reset all parameters to default values
    pub async fn reset_all(&self) {
        *self.parameter_1.write().await = 0;
        *self.parameter_2.write().await = 0;
        *self.parameter_3.write().await = 0;
        *self.parameter_4.write().await = 0;
        *self.parameter_5.write().await = 0;
        *self.parameter_6.write().await = 0;
        *self.parameter_7.write().await = false;
        *self.parameter_8.write().await = Vec::new();
    }
}

#[async_trait]
impl CosemObject for GenericSetup {
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
            Self::ATTR_PARAMETER_1 => {
                Ok(DataObject::Unsigned8(self.parameter_1().await))
            }
            Self::ATTR_PARAMETER_2 => {
                Ok(DataObject::Unsigned16(self.parameter_2().await))
            }
            Self::ATTR_PARAMETER_3 => {
                Ok(DataObject::Unsigned32(self.parameter_3().await))
            }
            Self::ATTR_PARAMETER_4 => {
                Ok(DataObject::Integer8(self.parameter_4().await))
            }
            Self::ATTR_PARAMETER_5 => {
                Ok(DataObject::Integer16(self.parameter_5().await))
            }
            Self::ATTR_PARAMETER_6 => {
                Ok(DataObject::Integer32(self.parameter_6().await))
            }
            Self::ATTR_PARAMETER_7 => {
                Ok(DataObject::Boolean(self.parameter_7().await))
            }
            Self::ATTR_PARAMETER_8 => {
                Ok(DataObject::OctetString(self.parameter_8().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "GenericSetup has no attribute {}",
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
            Self::ATTR_PARAMETER_1 => {
                match value {
                    DataObject::Unsigned8(v) => {
                        self.set_parameter_1(v).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for parameter_1".to_string(),
                    )),
                }
            }
            Self::ATTR_PARAMETER_2 => {
                match value {
                    DataObject::Unsigned16(v) => {
                        self.set_parameter_2(v).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned16 for parameter_2".to_string(),
                    )),
                }
            }
            Self::ATTR_PARAMETER_3 => {
                match value {
                    DataObject::Unsigned32(v) => {
                        self.set_parameter_3(v).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned32 for parameter_3".to_string(),
                    )),
                }
            }
            Self::ATTR_PARAMETER_4 => {
                match value {
                    DataObject::Integer8(v) => {
                        self.set_parameter_4(v).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer8 for parameter_4".to_string(),
                    )),
                }
            }
            Self::ATTR_PARAMETER_5 => {
                match value {
                    DataObject::Integer16(v) => {
                        self.set_parameter_5(v).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer16 for parameter_5".to_string(),
                    )),
                }
            }
            Self::ATTR_PARAMETER_6 => {
                match value {
                    DataObject::Integer32(v) => {
                        self.set_parameter_6(v).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer32 for parameter_6".to_string(),
                    )),
                }
            }
            Self::ATTR_PARAMETER_7 => {
                match value {
                    DataObject::Boolean(v) => {
                        self.set_parameter_7(v).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Boolean for parameter_7".to_string(),
                    )),
                }
            }
            Self::ATTR_PARAMETER_8 => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_parameter_8(bytes).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for parameter_8".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "GenericSetup has no attribute {}",
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
            "GenericSetup has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generic_setup_class_id() {
        let setup = GenericSetup::with_default_obis();
        assert_eq!(setup.class_id(), 26);
    }

    #[tokio::test]
    async fn test_generic_setup_obis_code() {
        let setup = GenericSetup::with_default_obis();
        assert_eq!(setup.obis_code(), GenericSetup::default_obis());
    }

    #[tokio::test]
    async fn test_generic_setup_initial_state() {
        let setup = GenericSetup::with_default_obis();
        assert_eq!(setup.parameter_1().await, 0);
        assert_eq!(setup.parameter_2().await, 0);
        assert_eq!(setup.parameter_3().await, 0);
        assert_eq!(setup.parameter_4().await, 0);
        assert_eq!(setup.parameter_5().await, 0);
        assert_eq!(setup.parameter_6().await, 0);
        assert!(!setup.parameter_7().await);
        assert!(setup.parameter_8().await.is_empty());
    }

    #[tokio::test]
    async fn test_generic_setup_set_parameter_1() {
        let setup = GenericSetup::with_default_obis();
        setup.set_parameter_1(255).await;
        assert_eq!(setup.parameter_1().await, 255);
    }

    #[tokio::test]
    async fn test_generic_setup_set_parameter_2() {
        let setup = GenericSetup::with_default_obis();
        setup.set_parameter_2(65535).await;
        assert_eq!(setup.parameter_2().await, 65535);
    }

    #[tokio::test]
    async fn test_generic_setup_set_parameter_3() {
        let setup = GenericSetup::with_default_obis();
        setup.set_parameter_3(4294967295).await;
        assert_eq!(setup.parameter_3().await, 4294967295);
    }

    #[tokio::test]
    async fn test_generic_setup_set_parameter_4() {
        let setup = GenericSetup::with_default_obis();
        setup.set_parameter_4(-128).await;
        assert_eq!(setup.parameter_4().await, -128);
    }

    #[tokio::test]
    async fn test_generic_setup_set_parameter_5() {
        let setup = GenericSetup::with_default_obis();
        setup.set_parameter_5(-32768).await;
        assert_eq!(setup.parameter_5().await, -32768);
    }

    #[tokio::test]
    async fn test_generic_setup_set_parameter_6() {
        let setup = GenericSetup::with_default_obis();
        setup.set_parameter_6(-2147483648).await;
        assert_eq!(setup.parameter_6().await, -2147483648);
    }

    #[tokio::test]
    async fn test_generic_setup_set_parameter_7() {
        let setup = GenericSetup::with_default_obis();
        setup.set_parameter_7(true).await;
        assert!(setup.parameter_7().await);
    }

    #[tokio::test]
    async fn test_generic_setup_set_parameter_8() {
        let setup = GenericSetup::with_default_obis();
        let data = vec![1, 2, 3, 4, 5];
        setup.set_parameter_8(data.clone()).await;
        assert_eq!(setup.parameter_8().await, data);
    }

    #[tokio::test]
    async fn test_generic_setup_reset_all() {
        let setup = GenericSetup::with_default_obis();
        setup.set_parameter_1(100).await;
        setup.set_parameter_2(200).await;
        setup.set_parameter_3(300).await;
        setup.set_parameter_7(true).await;
        setup.set_parameter_8(vec![1, 2, 3]).await;

        setup.reset_all().await;

        assert_eq!(setup.parameter_1().await, 0);
        assert_eq!(setup.parameter_2().await, 0);
        assert_eq!(setup.parameter_3().await, 0);
        assert!(!setup.parameter_7().await);
        assert!(setup.parameter_8().await.is_empty());
    }

    #[tokio::test]
    async fn test_generic_setup_get_attributes() {
        let setup = GenericSetup::with_default_obis();

        // Test parameter_1
        let result = setup.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Unsigned8(v) => assert_eq!(v, 0),
            _ => panic!("Expected Unsigned8"),
        }

        // Test parameter_2
        let result = setup.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Unsigned16(v) => assert_eq!(v, 0),
            _ => panic!("Expected Unsigned16"),
        }

        // Test parameter_3
        let result = setup.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Unsigned32(v) => assert_eq!(v, 0),
            _ => panic!("Expected Unsigned32"),
        }

        // Test parameter_4
        let result = setup.get_attribute(5, None).await.unwrap();
        match result {
            DataObject::Integer8(v) => assert_eq!(v, 0),
            _ => panic!("Expected Integer8"),
        }

        // Test parameter_5
        let result = setup.get_attribute(6, None).await.unwrap();
        match result {
            DataObject::Integer16(v) => assert_eq!(v, 0),
            _ => panic!("Expected Integer16"),
        }

        // Test parameter_6
        let result = setup.get_attribute(7, None).await.unwrap();
        match result {
            DataObject::Integer32(v) => assert_eq!(v, 0),
            _ => panic!("Expected Integer32"),
        }

        // Test parameter_7
        let result = setup.get_attribute(8, None).await.unwrap();
        match result {
            DataObject::Boolean(v) => assert!(!v),
            _ => panic!("Expected Boolean"),
        }

        // Test parameter_8
        let result = setup.get_attribute(9, None).await.unwrap();
        match result {
            DataObject::OctetString(bytes) => assert!(bytes.is_empty()),
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_generic_setup_set_attributes() {
        let setup = GenericSetup::with_default_obis();

        setup.set_attribute(2, DataObject::Unsigned8(123), None)
            .await
            .unwrap();
        assert_eq!(setup.parameter_1().await, 123);

        setup.set_attribute(3, DataObject::Unsigned16(456), None)
            .await
            .unwrap();
        assert_eq!(setup.parameter_2().await, 456);

        setup.set_attribute(4, DataObject::Unsigned32(789), None)
            .await
            .unwrap();
        assert_eq!(setup.parameter_3().await, 789);

        setup.set_attribute(5, DataObject::Integer8(-50), None)
            .await
            .unwrap();
        assert_eq!(setup.parameter_4().await, -50);

        setup.set_attribute(6, DataObject::Integer16(-1000), None)
            .await
            .unwrap();
        assert_eq!(setup.parameter_5().await, -1000);

        setup.set_attribute(7, DataObject::Integer32(-5000), None)
            .await
            .unwrap();
        assert_eq!(setup.parameter_6().await, -5000);

        setup.set_attribute(8, DataObject::Boolean(true), None)
            .await
            .unwrap();
        assert!(setup.parameter_7().await);

        setup.set_attribute(9, DataObject::OctetString(vec![10, 20, 30]), None)
            .await
            .unwrap();
        assert_eq!(setup.parameter_8().await, vec![10, 20, 30]);
    }

    #[tokio::test]
    async fn test_generic_setup_read_only_logical_name() {
        let setup = GenericSetup::with_default_obis();
        let result = setup
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 26, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generic_setup_invalid_attribute() {
        let setup = GenericSetup::with_default_obis();
        let result = setup.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generic_setup_invalid_method() {
        let setup = GenericSetup::with_default_obis();
        let result = setup.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generic_setup_invalid_data_types() {
        let setup = GenericSetup::with_default_obis();

        // Wrong type for parameter_1
        let result = setup.set_attribute(2, DataObject::Boolean(true), None).await;
        assert!(result.is_err());

        // Wrong type for parameter_2
        let result = setup.set_attribute(3, DataObject::Unsigned8(1), None).await;
        assert!(result.is_err());

        // Wrong type for parameter_7
        let result = setup.set_attribute(8, DataObject::Unsigned8(1), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generic_setup_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 26, 0, 0, 1);
        let setup = GenericSetup::new(obis);
        assert_eq!(setup.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_generic_setup_signed_parameters_negative_values() {
        let setup = GenericSetup::with_default_obis();

        setup.set_parameter_4(-1).await;
        assert_eq!(setup.parameter_4().await, -1);

        setup.set_parameter_5(-1).await;
        assert_eq!(setup.parameter_5().await, -1);

        setup.set_parameter_6(-1).await;
        assert_eq!(setup.parameter_6().await, -1);
    }
}
