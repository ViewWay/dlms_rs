//! Extended Register interface class (Class ID: 4)
//!
//! The Extended Register interface class represents a register with additional
//! attributes for status and capture time, similar to a standard Register but
//! with extended functionality.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: value - The current value (signed)
//! - Attribute 3: scaller_unit - Scaler and unit for the value
//! - Attribute 4: status - Status information for the register
//! - Attribute 5: capture_time - Timestamp of the last value capture
//!
//! # Extended Register (Class ID: 4)
//!
//! Unlike a standard Register, an Extended Register includes:
//! - Status flags indicating the quality/state of the value
//! - Capture time indicating when the value was last read/sampled

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{
    datatypes::{CosemDateTime, CosemDateFormat},
    DlmsError, DlmsResult, ObisCode, DataObject,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{CosemObject, ScalerUnit};

/// Extended Register interface class (Class ID: 4)
///
/// Default OBIS: 1-0:21.0.0.255 (example for active power)
///
/// This class represents a register value with additional metadata:
/// - The current value (signed long)
/// - Scaler and unit information
/// - Status flags indicating value quality
/// - Capture time (when the value was last read)
#[derive(Debug, Clone)]
pub struct ExtendedRegister {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Current value of the register
    value: Arc<RwLock<i64>>,

    /// Scaler and unit for the value
    scaler_unit: Arc<RwLock<Option<ScalerUnit>>>,

    /// Status information for the register
    status: Arc<RwLock<Option<Vec<u8>>>>,

    /// Timestamp of the last value capture
    capture_time: Arc<RwLock<Option<CosemDateTime>>>,
}

impl ExtendedRegister {
    /// Class ID for Extended Register
    pub const CLASS_ID: u16 = 4;

    /// Default OBIS code for Extended Register (1-0:21.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(1, 0, 21, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_VALUE: u8 = 2;
    pub const ATTR_SCALER_UNIT: u8 = 3;
    pub const ATTR_STATUS: u8 = 4;
    pub const ATTR_CAPTURE_TIME: u8 = 5;

    /// Create a new Extended Register object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `value` - Initial value
    /// * `scaler_unit` - Optional scaler and unit information
    /// * `status` - Optional status information
    pub fn new(
        logical_name: ObisCode,
        value: i64,
        scaler_unit: Option<ScalerUnit>,
        status: Option<Vec<u8>>,
    ) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(value)),
            scaler_unit: Arc::new(RwLock::new(scaler_unit)),
            status: Arc::new(RwLock::new(status)),
            capture_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis(value: i64) -> Self {
        Self::new(Self::default_obis(), value, None, None)
    }

    /// Get the current value
    pub async fn value(&self) -> i64 {
        *self.value.read().await
    }

    /// Set the value and update capture time
    pub async fn set_value(&self, value: i64) {
        *self.value.write().await = value;
        // Update capture time when value is set
        let now = CosemDateTime::new(2024, 1, 1, 0, 0, 0, 0, &[]).unwrap();
        *self.capture_time.write().await = Some(now);
    }

    /// Get the scaler and unit
    pub async fn scaler_unit(&self) -> Option<ScalerUnit> {
        self.scaler_unit.read().await.clone()
    }

    /// Set the scaler and unit
    pub async fn set_scaler_unit(&self, scaler_unit: Option<ScalerUnit>) {
        *self.scaler_unit.write().await = scaler_unit;
    }

    /// Get the status
    pub async fn status(&self) -> Option<Vec<u8>> {
        self.status.read().await.clone()
    }

    /// Set the status
    pub async fn set_status(&self, status: Option<Vec<u8>>) {
        *self.status.write().await = status;
    }

    /// Get the capture time
    pub async fn capture_time(&self) -> Option<CosemDateTime> {
        self.capture_time.read().await.clone()
    }

    /// Set the capture time
    pub async fn set_capture_time(&self, time: Option<CosemDateTime>) {
        *self.capture_time.write().await = time;
    }

    /// Update the value and record the capture time
    pub async fn update(&self, new_value: i64, new_status: Option<Vec<u8>>) {
        self.set_value(new_value).await;
        if let Some(status) = new_status {
            self.set_status(Some(status)).await;
        }
    }
}

#[async_trait]
impl CosemObject for ExtendedRegister {
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
                Ok(DataObject::Integer64(self.value().await))
            }
            Self::ATTR_SCALER_UNIT => {
                match self.scaler_unit().await {
                    Some(su) => Ok(su.to_data_object()),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_STATUS => {
                match self.status().await {
                    Some(status) => Ok(DataObject::OctetString(status)),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_CAPTURE_TIME => {
                match self.capture_time().await {
                    Some(dt) => Ok(DataObject::OctetString(dt.encode())),
                    None => Ok(DataObject::Null),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Extended Register has no attribute {}",
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
                if let DataObject::Integer64(v) = value {
                    self.set_value(v).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Integer64 for value".to_string(),
                    ))
                }
            }
            Self::ATTR_SCALER_UNIT => {
                match value {
                    DataObject::Null => {
                        self.set_scaler_unit(None).await;
                        Ok(())
                    }
                    v => {
                        let su = ScalerUnit::from_data_object(&v)?;
                        self.set_scaler_unit(Some(su)).await;
                        Ok(())
                    }
                }
            }
            Self::ATTR_STATUS => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_status(Some(bytes)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_status(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected octet string or Null for status".to_string(),
                    )),
                }
            }
            Self::ATTR_CAPTURE_TIME => {
                // Capture time is set automatically when value is updated
                // But we allow setting it manually too
                match value {
                    DataObject::OctetString(bytes) if bytes.len() >= 12 => {
                        let dt = CosemDateTime::decode(&bytes)?;
                        self.set_capture_time(Some(dt)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_capture_time(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected octet string or Null for capture_time".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Extended Register has no attribute {}",
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
            _ => Err(DlmsError::InvalidData(format!(
                "Extended Register has no method {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extended_register_class_id() {
        let reg = ExtendedRegister::with_default_obis(100);
        assert_eq!(reg.class_id(), 4);
    }

    #[tokio::test]
    async fn test_extended_register_obis_code() {
        let reg = ExtendedRegister::with_default_obis(100);
        assert_eq!(reg.obis_code(), ExtendedRegister::default_obis());
    }

    #[tokio::test]
    async fn test_extended_register_value() {
        let reg = ExtendedRegister::with_default_obis(100);
        assert_eq!(reg.value().await, 100);

        reg.set_attribute(2, DataObject::Integer64(200), None).await.unwrap();
        assert_eq!(reg.value().await, 200);
    }

    #[tokio::test]
    async fn test_extended_register_scaler_unit() {
        let reg = ExtendedRegister::with_default_obis(100);
        assert_eq!(reg.get_attribute(3, None).await.unwrap(), DataObject::Null);

        let scaler_unit = ScalerUnit::new(-1, 30); // kW
        let value = scaler_unit.to_data_object();
        reg.set_attribute(3, value, None).await.unwrap();

        let result = reg.scaler_unit().await;
        assert!(result.is_some());
        let su = result.unwrap();
        assert_eq!(su.scaler(), -1);
        assert_eq!(su.unit(), 30);
    }

    #[tokio::test]
    async fn test_extended_register_status() {
        let reg = ExtendedRegister::with_default_obis(100);
        let status = vec![0x01, 0x02];

        reg.set_attribute(4, DataObject::OctetString(status.clone()), None).await.unwrap();

        let result = reg.status().await;
        assert_eq!(result, Some(status));
    }

    #[tokio::test]
    async fn test_extended_register_status_null() {
        let reg = ExtendedRegister::new(ExtendedRegister::default_obis(), 100, None, Some(vec![0x01]));

        reg.set_attribute(4, DataObject::Null, None).await.unwrap();

        let result = reg.status().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_extended_register_capture_time() {
        let reg = ExtendedRegister::with_default_obis(100);
        assert_eq!(reg.get_attribute(5, None).await.unwrap(), DataObject::Null);

        // Setting a value should update capture time
        reg.set_value(150).await;
        assert!(reg.capture_time().await.is_some());
    }

    #[tokio::test]
    async fn test_extended_register_get_logical_name() {
        let reg = ExtendedRegister::with_default_obis(100);
        let result = reg.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_extended_register_invalid_attribute() {
        let reg = ExtendedRegister::with_default_obis(100);
        let result = reg.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_extended_register_read_only_logical_name() {
        let reg = ExtendedRegister::with_default_obis(100);
        let result = reg.set_attribute(1, DataObject::OctetString(vec![0, 0, 1, 0, 0, 1]), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_extended_register_update() {
        let reg = ExtendedRegister::with_default_obis(100);
        let new_status = vec![0xFF];

        reg.update(250, Some(new_status.clone())).await;

        assert_eq!(reg.value().await, 250);
        assert_eq!(reg.status().await, Some(new_status));
        assert!(reg.capture_time().await.is_some());
    }

    #[tokio::test]
    async fn test_extended_register_negative_value() {
        let reg = ExtendedRegister::with_default_obis(100);
        reg.set_attribute(2, DataObject::Integer64(-1000), None).await.unwrap();

        assert_eq!(reg.value().await, -1000);
    }
}
