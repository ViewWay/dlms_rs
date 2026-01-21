//! Demand Register interface class (Class ID: 5)
//!
//! The Demand Register interface class represents a register that stores
//! maximum/minimum values over a specific period, commonly used for
//! demand measurement in energy meters.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: current_value - The current value being monitored
//! - Attribute 3: scaler_unit - Scaler and unit for the value
//! - Attribute 4: status - Status information for the register
//! - Attribute 5: capture_time - Timestamp of the last value capture
//! - Attribute 6: start_time - Start time of the current demand period
//! - Attribute 7: period - Duration of the demand period in seconds
//! - Attribute 8: number_of_periods - Number of periods to monitor (optional)
//!
//! # Demand Register (Class ID: 5)
//!
//! This class is used for:
//! - Maximum demand measurement
//! - Minimum demand measurement
//! - Average demand calculation
//! - Sliding window demand monitoring

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{
    datatypes::{CosemDateTime, CosemDateFormat},
    DlmsError, DlmsResult, ObisCode, DataObject,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{CosemObject, ScalerUnit};

/// Demand Register interface class (Class ID: 5)
///
/// Default OBIS: 1-0:14.0.0.255 (example for maximum demand)
///
/// This class represents a demand register that tracks the maximum/minimum
/// value over a specific time period. It includes:
/// - The current value being monitored
/// - Scaler and unit information
/// - Status information
/// - Capture time (when value was last read)
/// - Start time of the demand period
/// - Period duration
#[derive(Debug, Clone)]
pub struct DemandRegister {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Current value being monitored
    current_value: Arc<RwLock<i64>>,

    /// Scaler and unit for the value
    scaler_unit: Arc<RwLock<Option<ScalerUnit>>>,

    /// Status information for the register
    status: Arc<RwLock<Option<Vec<u8>>>>,

    /// Timestamp of the last value capture
    capture_time: Arc<RwLock<Option<CosemDateTime>>>,

    /// Start time of the current demand period
    start_time: Arc<RwLock<Option<CosemDateTime>>>,

    /// Duration of the demand period in seconds
    period: Arc<RwLock<u32>>,

    /// Number of periods to monitor (optional, 0 = infinite)
    number_of_periods: Arc<RwLock<Option<u32>>>,
}

impl DemandRegister {
    /// Class ID for Demand Register
    pub const CLASS_ID: u16 = 5;

    /// Default OBIS code for Demand Register (1-0:14.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(1, 0, 14, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_CURRENT_VALUE: u8 = 2;
    pub const ATTR_SCALER_UNIT: u8 = 3;
    pub const ATTR_STATUS: u8 = 4;
    pub const ATTR_CAPTURE_TIME: u8 = 5;
    pub const ATTR_START_TIME: u8 = 6;
    pub const ATTR_PERIOD: u8 = 7;
    pub const ATTR_NUMBER_OF_PERIODS: u8 = 8;

    /// Create a new Demand Register object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `current_value` - Initial value
    /// * `period` - Duration of the demand period in seconds
    /// * `scaler_unit` - Optional scaler and unit information
    pub fn new(
        logical_name: ObisCode,
        current_value: i64,
        period: u32,
        scaler_unit: Option<ScalerUnit>,
    ) -> Self {
        Self {
            logical_name,
            current_value: Arc::new(RwLock::new(current_value)),
            scaler_unit: Arc::new(RwLock::new(scaler_unit)),
            status: Arc::new(RwLock::new(None)),
            capture_time: Arc::new(RwLock::new(None)),
            start_time: Arc::new(RwLock::new(None)),
            period: Arc::new(RwLock::new(period)),
            number_of_periods: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis(period: u32) -> Self {
        Self::new(Self::default_obis(), 0, period, None)
    }

    /// Get the current value
    pub async fn current_value(&self) -> i64 {
        *self.current_value.read().await
    }

    /// Set the current value and update capture time
    pub async fn set_current_value(&self, value: i64) {
        let current = *self.current_value.read().await;
        *self.current_value.write().await = value;

        // Update capture time
        let now = CosemDateTime::new(2024, 1, 1, 0, 0, 0, 0, &[]).unwrap();
        *self.capture_time.write().await = Some(now);

        // If this is a new maximum/minimum, update status
        if value != current {
            let status = vec![0x01]; // Value changed
            *self.status.write().await = Some(status);
        }
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

    /// Get the start time
    pub async fn start_time(&self) -> Option<CosemDateTime> {
        self.start_time.read().await.clone()
    }

    /// Set the start time (starts a new demand period)
    pub async fn set_start_time(&self, time: Option<CosemDateTime>) {
        *self.start_time.write().await = time;
    }

    /// Get the period duration
    pub async fn period(&self) -> u32 {
        *self.period.read().await
    }

    /// Set the period duration
    pub async fn set_period(&self, period: u32) {
        *self.period.write().await = period;
    }

    /// Get the number of periods
    pub async fn number_of_periods(&self) -> Option<u32> {
        *self.number_of_periods.read().await
    }

    /// Set the number of periods
    pub async fn set_number_of_periods(&self, count: Option<u32>) {
        *self.number_of_periods.write().await = count;
    }

    /// Start a new demand period
    pub async fn start_period(&self) -> DlmsResult<()> {
        let now = CosemDateTime::new(2024, 1, 1, 0, 0, 0, 0, &[]).unwrap();
        self.set_start_time(Some(now)).await;
        *self.current_value.write().await = 0;
        Ok(())
    }

    /// Reset the demand register
    pub async fn reset(&self) -> DlmsResult<()> {
        *self.current_value.write().await = 0;
        *self.start_time.write().await = None;
        *self.status.write().await = None;
        *self.capture_time.write().await = None;
        Ok(())
    }
}

#[async_trait]
impl CosemObject for DemandRegister {
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
            Self::ATTR_CURRENT_VALUE => {
                Ok(DataObject::Integer64(self.current_value().await))
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
            Self::ATTR_START_TIME => {
                match self.start_time().await {
                    Some(dt) => Ok(DataObject::OctetString(dt.encode())),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_PERIOD => {
                Ok(DataObject::Unsigned32(self.period().await))
            }
            Self::ATTR_NUMBER_OF_PERIODS => {
                match self.number_of_periods().await {
                    Some(count) => Ok(DataObject::Unsigned32(count)),
                    None => Ok(DataObject::Null),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Demand Register has no attribute {}",
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
            Self::ATTR_CURRENT_VALUE => {
                if let DataObject::Integer64(v) = value {
                    self.set_current_value(v).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Integer64 for current_value".to_string(),
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
            Self::ATTR_START_TIME => {
                match value {
                    DataObject::OctetString(bytes) if bytes.len() >= 12 => {
                        let dt = CosemDateTime::decode(&bytes)?;
                        self.set_start_time(Some(dt)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_start_time(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected octet string or Null for start_time".to_string(),
                    )),
                }
            }
            Self::ATTR_PERIOD => {
                if let DataObject::Unsigned32(period) = value {
                    self.set_period(period).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned32 for period".to_string(),
                    ))
                }
            }
            Self::ATTR_NUMBER_OF_PERIODS => {
                match value {
                    DataObject::Unsigned32(count) => {
                        self.set_number_of_periods(Some(count)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_number_of_periods(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned32 or Null for number_of_periods".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Demand Register has no attribute {}",
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
                "Demand Register has no method {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demand_register_class_id() {
        let reg = DemandRegister::with_default_obis(900); // 15 minutes
        assert_eq!(reg.class_id(), 5);
    }

    #[tokio::test]
    async fn test_demand_register_obis_code() {
        let reg = DemandRegister::with_default_obis(900);
        assert_eq!(reg.obis_code(), DemandRegister::default_obis());
    }

    #[tokio::test]
    async fn test_demand_register_current_value() {
        let reg = DemandRegister::with_default_obis(900);
        assert_eq!(reg.current_value().await, 0);

        reg.set_current_value(1000).await;
        assert_eq!(reg.current_value().await, 1000);
    }

    #[tokio::test]
    async fn test_demand_register_period() {
        let reg = DemandRegister::with_default_obis(900);
        assert_eq!(reg.period().await, 900);

        reg.set_period(3600).await;
        assert_eq!(reg.period().await, 3600);
    }

    #[tokio::test]
    async fn test_demand_register_start_period() {
        let reg = DemandRegister::with_default_obis(900);

        reg.start_period().await.unwrap();

        assert!(reg.start_time().await.is_some());
        assert_eq!(reg.current_value().await, 0);
    }

    #[tokio::test]
    async fn test_demand_register_reset() {
        let reg = DemandRegister::with_default_obis(900);
        reg.set_current_value(500).await;

        reg.reset().await.unwrap();

        assert_eq!(reg.current_value().await, 0);
        assert!(reg.start_time().await.is_none());
    }

    #[tokio::test]
    async fn test_demand_register_scaler_unit() {
        let reg = DemandRegister::with_default_obis(900);
        assert_eq!(reg.get_attribute(3, None).await.unwrap(), DataObject::Null);

        let scaler_unit = ScalerUnit::new(-1, 33); // kWh
        let value = scaler_unit.to_data_object();
        reg.set_attribute(3, value, None).await.unwrap();

        let result = reg.scaler_unit().await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_demand_register_number_of_periods() {
        let reg = DemandRegister::with_default_obis(900);
        assert_eq!(reg.get_attribute(8, None).await.unwrap(), DataObject::Null);

        reg.set_attribute(8, DataObject::Unsigned32(12), None).await.unwrap();
        assert_eq!(reg.number_of_periods().await, Some(12));
    }

    #[tokio::test]
    async fn test_demand_register_get_logical_name() {
        let reg = DemandRegister::with_default_obis(900);
        let result = reg.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_demand_register_invalid_attribute() {
        let reg = DemandRegister::with_default_obis(900);
        let result = reg.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_demand_register_read_only_logical_name() {
        let reg = DemandRegister::with_default_obis(900);
        let result = reg.set_attribute(1, DataObject::OctetString(vec![0, 0, 1, 0, 14, 0]), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_demand_register_status() {
        let reg = DemandRegister::with_default_obis(900);
        let status = vec![0x01, 0x02];

        reg.set_attribute(4, DataObject::OctetString(status.clone()), None).await.unwrap();

        assert_eq!(reg.status().await, Some(status));
    }

    #[tokio::test]
    async fn test_demand_register_negative_value() {
        let reg = DemandRegister::with_default_obis(900);
        reg.set_current_value(-500).await;

        assert_eq!(reg.current_value().await, -500);
    }
}
