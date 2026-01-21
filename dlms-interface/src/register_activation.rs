//! Register Activation interface class (Class ID: 6)
//!
//! The Register Activation interface class represents a register that
//! can be activated/deactivated and tracks its activation status.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: value - The current value (signed)
//! - Attribute 3: scaler_unit - Scaler and unit for the value
//! - Attribute 4: status - Status of the register activation
//! - Attribute 5: activation_time - Timestamp of activation
//! - Attribute 6: activation_time_old - Previous activation timestamp
//!
//! # Methods
//!
//! - Method 1: activate(data) - Activate the register with data
//!
//! # Register Activation (Class ID: 6)
//!
//! This class is used for registers that track activation status,
//! such as tariff activation or special billing period activation.

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{
    datatypes::{CosemDateTime, CosemDateFormat},
    DlmsError, DlmsResult, ObisCode, DataObject,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{CosemObject, ScalerUnit};

/// Register Activation interface class (Class ID: 6)
///
/// Default OBIS: 1-0:0.0.0.255 (example for activation register)
///
/// This class represents a register value with activation timestamping.
/// It's commonly used for tariff activation or billing period tracking.
#[derive(Debug, Clone)]
pub struct RegisterActivation {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Current value of the register
    value: Arc<RwLock<i64>>,

    /// Scaler and unit for the value
    scaler_unit: Arc<RwLock<Option<ScalerUnit>>>,

    /// Status of the register activation
    status: Arc<RwLock<bool>>,

    /// Timestamp of current activation
    activation_time: Arc<RwLock<Option<CosemDateTime>>>,

    /// Timestamp of previous activation
    activation_time_old: Arc<RwLock<Option<CosemDateTime>>>,
}

impl RegisterActivation {
    /// Class ID for Register Activation
    pub const CLASS_ID: u16 = 6;

    /// Default OBIS code for Register Activation (1-0:0.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(1, 0, 0, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_VALUE: u8 = 2;
    pub const ATTR_SCALER_UNIT: u8 = 3;
    pub const ATTR_STATUS: u8 = 4;
    pub const ATTR_ACTIVATION_TIME: u8 = 5;
    pub const ATTR_ACTIVATION_TIME_OLD: u8 = 6;

    /// Method IDs
    pub const METHOD_ACTIVATE: u8 = 1;

    /// Create a new Register Activation object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `value` - Initial value
    /// * `scaler_unit` - Optional scaler and unit information
    pub fn new(logical_name: ObisCode, value: i64, scaler_unit: Option<ScalerUnit>) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(value)),
            scaler_unit: Arc::new(RwLock::new(scaler_unit)),
            status: Arc::new(RwLock::new(false)),
            activation_time: Arc::new(RwLock::new(None)),
            activation_time_old: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis(value: i64) -> Self {
        Self::new(Self::default_obis(), value, None)
    }

    /// Get the current value
    pub async fn value(&self) -> i64 {
        *self.value.read().await
    }

    /// Set the value
    pub async fn set_value(&self, value: i64) {
        *self.value.write().await = value;
    }

    /// Get the scaler and unit
    pub async fn scaler_unit(&self) -> Option<ScalerUnit> {
        self.scaler_unit.read().await.clone()
    }

    /// Set the scaler and unit
    pub async fn set_scaler_unit(&self, scaler_unit: Option<ScalerUnit>) {
        *self.scaler_unit.write().await = scaler_unit;
    }

    /// Get the activation status
    pub async fn status(&self) -> bool {
        *self.status.read().await
    }

    /// Set the activation status
    pub async fn set_status(&self, status: bool) {
        *self.status.write().await = status;
    }

    /// Get the activation time
    pub async fn activation_time(&self) -> Option<CosemDateTime> {
        self.activation_time.read().await.clone()
    }

    /// Set the activation time
    pub async fn set_activation_time(&self, time: Option<CosemDateTime>) {
        *self.activation_time.write().await = time;
    }

    /// Get the old activation time
    pub async fn activation_time_old(&self) -> Option<CosemDateTime> {
        self.activation_time_old.read().await.clone()
    }

    /// Set the old activation time
    pub async fn set_activation_time_old(&self, time: Option<CosemDateTime>) {
        *self.activation_time_old.write().await = time;
    }

    /// Check if the register is currently activated
    pub async fn is_active(&self) -> bool {
        self.status().await && self.activation_time().await.is_some()
    }

    /// Activate the register with a new value and timestamp
    ///
    /// This corresponds to Method 1
    pub async fn activate(&self, value: i64, timestamp: CosemDateTime) -> DlmsResult<()> {
        // Store current activation time as old
        let current_time = self.activation_time().await;
        *self.activation_time_old.write().await = current_time;

        // Set new value and activation time
        self.set_value(value).await;
        self.set_activation_time(Some(timestamp)).await;
        self.set_status(true).await;
        Ok(())
    }

    /// Deactivate the register
    pub async fn deactivate(&self) {
        self.set_status(false).await;
    }

    /// Reset the activation state
    pub async fn reset(&self) {
        self.set_status(false).await;
        self.set_activation_time(None).await;
        self.set_activation_time_old(None).await;
        self.set_value(0).await;
    }

    /// Get the time since activation in seconds
    pub async fn time_since_activation(&self) -> Option<u64> {
        // In a real implementation, this would calculate the actual time difference
        // For now, we return a placeholder value
        if self.is_active().await {
            Some(0)
        } else {
            None
        }
    }
}

#[async_trait]
impl CosemObject for RegisterActivation {
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
                Ok(DataObject::Boolean(self.status().await))
            }
            Self::ATTR_ACTIVATION_TIME => {
                match self.activation_time().await {
                    Some(dt) => Ok(DataObject::OctetString(dt.encode())),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_ACTIVATION_TIME_OLD => {
                match self.activation_time_old().await {
                    Some(dt) => Ok(DataObject::OctetString(dt.encode())),
                    None => Ok(DataObject::Null),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Register Activation has no attribute {}",
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
                if let DataObject::Boolean(status) = value {
                    self.set_status(status).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Boolean for status".to_string(),
                    ))
                }
            }
            Self::ATTR_ACTIVATION_TIME => {
                match value {
                    DataObject::OctetString(bytes) if bytes.len() >= 12 => {
                        let dt = CosemDateTime::decode(&bytes)?;
                        self.set_activation_time(Some(dt)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_activation_time(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected octet string or Null for activation_time".to_string(),
                    )),
                }
            }
            Self::ATTR_ACTIVATION_TIME_OLD => {
                match value {
                    DataObject::OctetString(bytes) if bytes.len() >= 12 => {
                        let dt = CosemDateTime::decode(&bytes)?;
                        self.set_activation_time_old(Some(dt)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_activation_time_old(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected octet string or Null for activation_time_old".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Register Activation has no attribute {}",
                attribute_id
            ))),
        }
    }

    async fn invoke_method(
        &self,
        method_id: u8,
        parameters: Option<DataObject>,
        _selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<Option<DataObject>> {
        match method_id {
            Self::METHOD_ACTIVATE => {
                // Activate method: expects a structure with value and timestamp
                if let Some(DataObject::Array(params)) = parameters {
                    if params.len() >= 2 {
                        let value = match &params[0] {
                            DataObject::Integer64(v) => *v,
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Method 1 expects Integer64 as first parameter".to_string(),
                                ))
                            }
                        };
                        let timestamp = match &params[1] {
                            DataObject::OctetString(bytes) if bytes.len() >= 12 => {
                                CosemDateTime::decode(bytes)?
                            }
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Method 1 expects octet string as second parameter".to_string(),
                                ))
                            }
                        };
                        self.activate(value, timestamp).await?;
                        return Ok(None);
                    }
                }
                Err(DlmsError::InvalidData(
                    "Method 1 requires a structure parameter with value and timestamp".to_string(),
                ))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Register Activation has no method {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_activation_class_id() {
        let reg = RegisterActivation::with_default_obis(0);
        assert_eq!(reg.class_id(), 6);
    }

    #[tokio::test]
    async fn test_register_activation_obis_code() {
        let reg = RegisterActivation::with_default_obis(0);
        assert_eq!(reg.obis_code(), RegisterActivation::default_obis());
    }

    #[tokio::test]
    async fn test_register_activation_initial_state() {
        let reg = RegisterActivation::with_default_obis(100);
        assert_eq!(reg.value().await, 100);
        assert!(!reg.status().await);
        assert!(reg.activation_time().await.is_none());
        assert!(reg.activation_time_old().await.is_none());
    }

    #[tokio::test]
    async fn test_register_activation_set_value() {
        let reg = RegisterActivation::with_default_obis(0);
        reg.set_value(500).await;
        assert_eq!(reg.value().await, 500);
    }

    #[tokio::test]
    async fn test_register_activation_activate() {
        let reg = RegisterActivation::with_default_obis(0);
        let timestamp = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();

        reg.activate(100, timestamp.clone()).await.unwrap();

        assert_eq!(reg.value().await, 100);
        assert!(reg.status().await);
        assert_eq!(reg.activation_time().await, Some(timestamp));
        assert!(reg.activation_time_old().await.is_none());
    }

    #[tokio::test]
    async fn test_register_activation_twice() {
        let reg = RegisterActivation::with_default_obis(0);
        let ts1 = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let ts2 = CosemDateTime::new(2024, 6, 16, 12, 0, 0, 0, &[]).unwrap();

        reg.activate(100, ts1.clone()).await.unwrap();
        reg.activate(200, ts2.clone()).await.unwrap();

        assert_eq!(reg.value().await, 200);
        assert_eq!(reg.activation_time().await, Some(ts2));
        assert_eq!(reg.activation_time_old().await, Some(ts1));
    }

    #[tokio::test]
    async fn test_register_activation_deactivate() {
        let reg = RegisterActivation::with_default_obis(0);
        let timestamp = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();

        reg.activate(100, timestamp).await.unwrap();
        assert!(reg.is_active().await);

        reg.deactivate().await;
        assert!(!reg.is_active().await);
        // Value and timestamp should remain
        assert_eq!(reg.value().await, 100);
        assert!(reg.activation_time().await.is_some());
    }

    #[tokio::test]
    async fn test_register_activation_reset() {
        let reg = RegisterActivation::with_default_obis(100);
        let timestamp = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();

        reg.activate(200, timestamp).await.unwrap();
        reg.reset().await;

        assert!(!reg.status().await);
        assert_eq!(reg.value().await, 0);
        assert!(reg.activation_time().await.is_none());
        assert!(reg.activation_time_old().await.is_none());
    }

    #[tokio::test]
    async fn test_register_activation_get_logical_name() {
        let reg = RegisterActivation::with_default_obis(0);
        let result = reg.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_register_activation_get_value() {
        let reg = RegisterActivation::with_default_obis(123);
        let result = reg.get_attribute(2, None).await.unwrap();
        assert_eq!(result, DataObject::Integer64(123));
    }

    #[tokio::test]
    async fn test_register_activation_get_status() {
        let reg = RegisterActivation::with_default_obis(0);
        let result = reg.get_attribute(4, None).await.unwrap();
        assert_eq!(result, DataObject::Boolean(false));

        reg.set_status(true).await;
        let result = reg.get_attribute(4, None).await.unwrap();
        assert_eq!(result, DataObject::Boolean(true));
    }

    #[tokio::test]
    async fn test_register_activation_set_value_via_attribute() {
        let reg = RegisterActivation::with_default_obis(0);
        reg.set_attribute(2, DataObject::Integer64(456), None).await.unwrap();
        assert_eq!(reg.value().await, 456);
    }

    #[tokio::test]
    async fn test_register_activation_read_only_logical_name() {
        let reg = RegisterActivation::with_default_obis(0);
        let result = reg.set_attribute(1, DataObject::OctetString(vec![1, 0, 0, 0, 0, 1]), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_activation_set_activation_time() {
        let reg = RegisterActivation::with_default_obis(0);
        let timestamp = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();

        reg.set_attribute(5, DataObject::OctetString(timestamp.encode()), None).await.unwrap();
        assert!(reg.activation_time().await.is_some());
    }

    #[tokio::test]
    async fn test_register_activation_method_activate() {
        let reg = RegisterActivation::with_default_obis(0);
        let timestamp = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();

        let params = DataObject::Array(vec![
            DataObject::Integer64(999),
            DataObject::OctetString(timestamp.encode()),
        ]);

        reg.invoke_method(1, Some(params), None).await.unwrap();

        assert_eq!(reg.value().await, 999);
        assert!(reg.is_active().await);
    }

    #[tokio::test]
    async fn test_register_activation_invalid_method() {
        let reg = RegisterActivation::with_default_obis(0);
        let result = reg.invoke_method(99, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_activation_invalid_attribute() {
        let reg = RegisterActivation::with_default_obis(0);
        let result = reg.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_activation_scaler_unit() {
        let reg = RegisterActivation::with_default_obis(0);
        assert_eq!(reg.get_attribute(3, None).await.unwrap(), DataObject::Null);

        let scaler_unit = ScalerUnit::new(-1, 30); // kW
        reg.set_attribute(3, scaler_unit.to_data_object(), None).await.unwrap();

        let result = reg.scaler_unit().await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_register_activation_null_activation_time() {
        let reg = RegisterActivation::with_default_obis(0);
        let result = reg.set_attribute(5, DataObject::Null, None).await;
        assert!(result.is_ok());
        assert!(reg.activation_time().await.is_none());
    }
}
