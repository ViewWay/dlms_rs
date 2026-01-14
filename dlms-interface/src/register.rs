//! Register interface class (Class ID: 3)
//!
//! The Register interface class represents a single register value with
//! scaling factor and unit information.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: value - The register value (Integer, Long, DoubleLong, etc.)
//! - Attribute 3: scaler_unit - ScalerUnit structure (scaler and unit)
//! - Attribute 4: status - Optional status value (Unsigned8)
//!
//! # Methods
//!
//! None
//!
//! # Usage
//!
//! ```rust,no_run
//! use dlms_interface::{Register, ScalerUnit};
//! use dlms_core::{ObisCode, DataObject};
//!
//! // Create a Register for energy (Wh)
//! let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
//! let value = DataObject::Unsigned32(12345);
//! let scaler_unit = ScalerUnit::new(0, 0x1E); // Wh
//! let register = Register::new(obis, value, scaler_unit, None);
//!
//! // Get the value
//! let current_value = register.value().await;
//! ```

use crate::scaler_unit::ScalerUnit;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_server::CosemObject;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Register interface class (Class ID: 3)
///
/// Represents a single register value with scaling factor and unit information.
/// This is one of the most commonly used interface classes in DLMS/COSEM.
#[derive(Debug, Clone)]
pub struct Register {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,
    /// The register value (Integer, Long, DoubleLong, etc.)
    value: Arc<RwLock<DataObject>>,
    /// Scaler and unit information
    scaler_unit: Arc<RwLock<ScalerUnit>>,
    /// Optional status value
    status: Arc<RwLock<Option<u8>>>,
}

impl Register {
    /// Create a new Register object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `value` - Initial register value (must be a numeric type)
    /// * `scaler_unit` - ScalerUnit structure
    /// * `status` - Optional initial status value
    ///
    /// # Returns
    /// A new Register instance
    pub fn new(
        logical_name: ObisCode,
        value: DataObject,
        scaler_unit: ScalerUnit,
        status: Option<u8>,
    ) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(value)),
            scaler_unit: Arc::new(RwLock::new(scaler_unit)),
            status: Arc::new(RwLock::new(status)),
        }
    }

    /// Get the current value
    ///
    /// # Returns
    /// A copy of the current register value
    pub async fn value(&self) -> DataObject {
        self.value.read().await.clone()
    }

    /// Set the value
    ///
    /// # Arguments
    /// * `new_value` - New register value to set
    pub async fn set_value(&self, new_value: DataObject) {
        *self.value.write().await = new_value;
    }

    /// Get the scaler unit
    pub async fn scaler_unit(&self) -> ScalerUnit {
        *self.scaler_unit.read().await
    }

    /// Set the scaler unit
    pub async fn set_scaler_unit(&self, scaler_unit: ScalerUnit) {
        *self.scaler_unit.write().await = scaler_unit;
    }

    /// Get the status
    ///
    /// # Returns
    /// Optional status value
    pub async fn status(&self) -> Option<u8> {
        *self.status.read().await
    }

    /// Set the status
    ///
    /// # Arguments
    /// * `new_status` - New status value (None to clear)
    pub async fn set_status(&self, new_status: Option<u8>) {
        *self.status.write().await = new_status;
    }

    /// Get the logical name (OBIS code)
    pub fn logical_name(&self) -> ObisCode {
        self.logical_name
    }

    /// Get the scaled value as f64
    ///
    /// This applies the scaling factor to the register value.
    ///
    /// # Returns
    /// The scaled value, or error if value is not numeric
    pub async fn scaled_value(&self) -> DlmsResult<f64> {
        let value = self.value().await;
        let numeric_value = match value {
            DataObject::Integer8(v) => v as f64,
            DataObject::Integer16(v) => v as f64,
            DataObject::Integer32(v) => v as f64,
            DataObject::Integer64(v) => v as f64,
            DataObject::Unsigned8(v) => v as f64,
            DataObject::Unsigned16(v) => v as f64,
            DataObject::Unsigned32(v) => v as f64,
            DataObject::Unsigned64(v) => v as f64,
            DataObject::Float32(v) => v as f64,
            DataObject::Float64(v) => v,
            _ => {
                return Err(DlmsError::InvalidData(
                    "Register value must be numeric".to_string(),
                ));
            }
        };
        Ok(self.scaler_unit.scale_value(numeric_value))
    }
}

#[async_trait::async_trait]
impl CosemObject for Register {
    fn class_id(&self) -> u16 {
        3 // Register interface class ID
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
            3 => {
                // Attribute 3: scaler_unit
                Ok(self.scaler_unit().await.to_data_object())
            }
            4 => {
                // Attribute 4: status (optional)
                match self.status().await {
                    Some(s) => Ok(DataObject::Unsigned8(s)),
                    None => Ok(DataObject::Null),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Register interface class has no attribute {}",
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
            3 => {
                // Attribute 3: scaler_unit
                let scaler_unit = ScalerUnit::from_data_object(&value)?;
                self.set_scaler_unit(scaler_unit).await;
                Ok(())
            }
            4 => {
                // Attribute 4: status
                let status = match value {
                    DataObject::Null => None,
                    DataObject::Unsigned8(s) => Some(s),
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Status must be Unsigned8 or Null".to_string(),
                        ));
                    }
                };
                self.set_status(status).await;
                Ok(())
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Register interface class has no attribute {}",
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
            "Register interface class has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_creation() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned32(12345);
        let scaler_unit = ScalerUnit::new(0, 0x1E); // Wh
        let register = Register::new(obis, value.clone(), scaler_unit, Some(0));

        assert_eq!(register.class_id(), 3);
        assert_eq!(register.obis_code(), obis);
        assert_eq!(register.value().await, value);
        assert_eq!(register.status().await, Some(0));
    }

    #[tokio::test]
    async fn test_register_get_attribute() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned32(12345);
        let scaler_unit = ScalerUnit::new(3, 0x1B); // kW
        let register = Register::new(obis, value.clone(), scaler_unit, Some(0));

        // Get attribute 1 (logical_name)
        let attr1 = register.get_attribute(1, None).await.unwrap();
        if let DataObject::OctetString(bytes) = attr1 {
            assert_eq!(bytes, obis.to_bytes());
        } else {
            panic!("Attribute 1 should be OctetString");
        }

        // Get attribute 2 (value)
        let attr2 = register.get_attribute(2, None).await.unwrap();
        assert_eq!(attr2, value);

        // Get attribute 3 (scaler_unit)
        let attr3 = register.get_attribute(3, None).await.unwrap();
        let decoded_scaler_unit = ScalerUnit::from_data_object(&attr3).unwrap();
        assert_eq!(decoded_scaler_unit, register.scaler_unit().await);

        // Get attribute 4 (status)
        let attr4 = register.get_attribute(4, None).await.unwrap();
        if let DataObject::Unsigned8(s) = attr4 {
            assert_eq!(s, 0);
        } else {
            panic!("Attribute 4 should be Unsigned8");
        }
    }

    #[tokio::test]
    async fn test_register_set_attribute() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let initial_value = DataObject::Unsigned32(12345);
        let scaler_unit = ScalerUnit::new(0, 0x1E);
        let register = Register::new(obis, initial_value, scaler_unit, None);

        // Set attribute 2 (value)
        let new_value = DataObject::Unsigned32(67890);
        register.set_attribute(2, new_value.clone(), None).await.unwrap();

        // Verify the value was updated
        let current_value = register.get_attribute(2, None).await.unwrap();
        assert_eq!(current_value, new_value);

        // Set attribute 4 (status)
        register
            .set_attribute(4, DataObject::Unsigned8(1), None)
            .await
            .unwrap();
        assert_eq!(register.status().await, Some(1));

        // Clear status
        register.set_attribute(4, DataObject::Null, None).await.unwrap();
        assert_eq!(register.status().await, None);
    }

    #[tokio::test]
    async fn test_register_scaled_value() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned32(12345);
        let scaler_unit = ScalerUnit::new(3, 0x1B); // kW (scale factor 3)
        let register = Register::new(obis, value, scaler_unit, None);

        let scaled = register.scaled_value().await.unwrap();
        // 12345 * 10^3 = 12345000
        assert!((scaled - 12345000.0).abs() < 0.001);
        
        // Test setting scaler_unit
        let new_scaler_unit = ScalerUnit::new(0, 0x1E);
        register.set_scaler_unit(new_scaler_unit).await;
        let scaled2 = register.scaled_value().await.unwrap();
        // 12345 * 10^0 = 12345
        assert!((scaled2 - 12345.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_register_invalid_attribute() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned32(12345);
        let scaler_unit = ScalerUnit::new(0, 0x1E);
        let register = Register::new(obis, value, scaler_unit, None);

        // Try to get non-existent attribute
        let result = register.get_attribute(99, None).await;
        assert!(result.is_err());

        // Try to set non-existent attribute
        let result = register.set_attribute(99, DataObject::Integer32(0), None).await;
        assert!(result.is_err());
    }
}
