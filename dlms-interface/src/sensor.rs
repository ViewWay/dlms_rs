//! Sensor interface class (Class ID: 102)
//!
//! The Sensor interface class manages sensor readings and status.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: value - Current sensor value
//! - Attribute 3: unit - Unit of measurement
//! - Attribute 4: sensor_status - Status of the sensor
//! - Attribute 5: min_value - Minimum expected value
//! - Attribute 6: max_value - Maximum expected value

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Sensor Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SensorStatus {
    /// Sensor is OK
    Ok = 0,
    /// Sensor error
    Error = 1,
    /// Sensor not connected
    NotConnected = 2,
    /// Sensor reading invalid
    Invalid = 3,
    /// Sensor out of range
    OutOfRange = 4,
    /// Sensor calibrating
    Calibrating = 5,
}

impl SensorStatus {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Ok,
            1 => Self::Error,
            2 => Self::NotConnected,
            3 => Self::Invalid,
            4 => Self::OutOfRange,
            5 => Self::Calibrating,
            _ => Self::Error,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if sensor is OK
    pub fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Check if sensor has error
    pub fn has_error(self) -> bool {
        !matches!(self, Self::Ok)
    }
}

/// Sensor interface class (Class ID: 102)
///
/// Default OBIS: 0-0:102.0.0.255
///
/// This class manages sensor readings and status.
#[derive(Debug, Clone)]
pub struct Sensor {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Current sensor value
    value: Arc<RwLock<f64>>,

    /// Unit of measurement
    unit: Arc<RwLock<String>>,

    /// Status of the sensor
    sensor_status: Arc<RwLock<SensorStatus>>,

    /// Minimum expected value
    min_value: Arc<RwLock<f64>>,

    /// Maximum expected value
    max_value: Arc<RwLock<f64>>,

    /// Last reading timestamp
    last_reading_time: Arc<RwLock<Option<i64>>>,
}

impl Sensor {
    /// Class ID for Sensor
    pub const CLASS_ID: u16 = 102;

    /// Default OBIS code for Sensor (0-0:102.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 102, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_VALUE: u8 = 2;
    pub const ATTR_UNIT: u8 = 3;
    pub const ATTR_SENSOR_STATUS: u8 = 4;
    pub const ATTR_MIN_VALUE: u8 = 5;
    pub const ATTR_MAX_VALUE: u8 = 6;

    /// Create a new Sensor object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(0.0)),
            unit: Arc::new(RwLock::new(String::new())),
            sensor_status: Arc::new(RwLock::new(SensorStatus::Ok)),
            min_value: Arc::new(RwLock::new(f64::MIN)),
            max_value: Arc::new(RwLock::new(f64::MAX)),
            last_reading_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Create with unit and range
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `unit` - Unit of measurement
    /// * `min_value` - Minimum expected value
    /// * `max_value` - Maximum expected value
    pub fn with_range(logical_name: ObisCode, unit: String, min_value: f64, max_value: f64) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(0.0)),
            unit: Arc::new(RwLock::new(unit)),
            sensor_status: Arc::new(RwLock::new(SensorStatus::Ok)),
            min_value: Arc::new(RwLock::new(min_value)),
            max_value: Arc::new(RwLock::new(max_value)),
            last_reading_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the current value
    pub async fn value(&self) -> f64 {
        *self.value.read().await
    }

    /// Set the current value
    pub async fn set_value(&self, value: f64) {
        *self.value.write().await = value;

        // Update reading time
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        *self.last_reading_time.write().await = Some(now);

        // Check if value is within range
        let min = self.min_value().await;
        let max = self.max_value().await;
        if value < min || value > max {
            *self.sensor_status.write().await = SensorStatus::OutOfRange;
        }
    }

    /// Get the unit
    pub async fn unit(&self) -> String {
        self.unit.read().await.clone()
    }

    /// Set the unit
    pub async fn set_unit(&self, unit: String) {
        *self.unit.write().await = unit;
    }

    /// Get the sensor status
    pub async fn sensor_status(&self) -> SensorStatus {
        *self.sensor_status.read().await
    }

    /// Set the sensor status
    pub async fn set_sensor_status(&self, status: SensorStatus) {
        *self.sensor_status.write().await = status;
    }

    /// Get the minimum value
    pub async fn min_value(&self) -> f64 {
        *self.min_value.read().await
    }

    /// Set the minimum value
    pub async fn set_min_value(&self, min: f64) {
        *self.min_value.write().await = min;
    }

    /// Get the maximum value
    pub async fn max_value(&self) -> f64 {
        *self.max_value.read().await
    }

    /// Set the maximum value
    pub async fn set_max_value(&self, max: f64) {
        *self.max_value.write().await = max;
    }

    /// Set the range
    pub async fn set_range(&self, min: f64, max: f64) {
        self.set_min_value(min).await;
        self.set_max_value(max).await;
    }

    /// Get the last reading time
    pub async fn last_reading_time(&self) -> Option<i64> {
        *self.last_reading_time.read().await
    }

    /// Check if sensor is OK
    pub async fn is_ok(&self) -> bool {
        self.sensor_status().await.is_ok()
    }

    /// Check if sensor has error
    pub async fn has_error(&self) -> bool {
        self.sensor_status().await.has_error()
    }

    /// Mark sensor as OK
    pub async fn mark_ok(&self) {
        self.set_sensor_status(SensorStatus::Ok).await;
    }

    /// Mark sensor as error
    pub async fn mark_error(&self) {
        self.set_sensor_status(SensorStatus::Error).await;
    }

    /// Check if value is within range
    pub async fn is_in_range(&self) -> bool {
        let value = self.value().await;
        let min = self.min_value().await;
        let max = self.max_value().await;
        value >= min && value <= max
    }

    /// Read and validate a sensor value
    pub async fn read(&self, raw_value: f64) -> DlmsResult<f64> {
        let min = self.min_value().await;
        let max = self.max_value().await;

        if raw_value < min || raw_value > max {
            self.set_sensor_status(SensorStatus::OutOfRange).await;
            return Err(DlmsError::InvalidData(format!(
                "Sensor value {} out of range [{}, {}]",
                raw_value, min, max
            )));
        }

        self.set_value(raw_value).await;
        self.mark_ok().await;
        Ok(raw_value)
    }
}

#[async_trait]
impl CosemObject for Sensor {
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
                Ok(DataObject::Float64(self.value().await))
            }
            Self::ATTR_UNIT => {
                Ok(DataObject::OctetString(self.unit().await.into_bytes()))
            }
            Self::ATTR_SENSOR_STATUS => {
                Ok(DataObject::Enumerate(self.sensor_status().await.to_u8()))
            }
            Self::ATTR_MIN_VALUE => {
                Ok(DataObject::Float64(self.min_value().await))
            }
            Self::ATTR_MAX_VALUE => {
                Ok(DataObject::Float64(self.max_value().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Sensor has no attribute {}",
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
                    DataObject::Float64(v) => {
                        self.set_value(v).await;
                        Ok(())
                    }
                    DataObject::Float32(v) => {
                        self.set_value(v as f64).await;
                        Ok(())
                    }
                    DataObject::Integer64(v) => {
                        self.set_value(v as f64).await;
                        Ok(())
                    }
                    DataObject::Integer32(v) => {
                        self.set_value(v as f64).await;
                        Ok(())
                    }
                    DataObject::Unsigned32(v) => {
                        self.set_value(v as f64).await;
                        Ok(())
                    }
                    DataObject::Unsigned16(v) => {
                        self.set_value(v as f64).await;
                        Ok(())
                    }
                    DataObject::Unsigned8(v) => {
                        self.set_value(v as f64).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected numeric type for value".to_string(),
                    )),
                }
            }
            Self::ATTR_UNIT => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_unit(String::from_utf8_lossy(&bytes).to_string()).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for unit".to_string(),
                    )),
                }
            }
            Self::ATTR_SENSOR_STATUS => {
                match value {
                    DataObject::Enumerate(status) => {
                        self.set_sensor_status(SensorStatus::from_u8(status)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for sensor_status".to_string(),
                    )),
                }
            }
            Self::ATTR_MIN_VALUE => {
                match value {
                    DataObject::Float64(min) => {
                        self.set_min_value(min).await;
                        Ok(())
                    }
                    DataObject::Float32(min) => {
                        self.set_min_value(min as f64).await;
                        Ok(())
                    }
                    DataObject::Integer32(min) => {
                        self.set_min_value(min as f64).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Float64/Float32/Integer for min_value".to_string(),
                    )),
                }
            }
            Self::ATTR_MAX_VALUE => {
                match value {
                    DataObject::Float64(max) => {
                        self.set_max_value(max).await;
                        Ok(())
                    }
                    DataObject::Float32(max) => {
                        self.set_max_value(max as f64).await;
                        Ok(())
                    }
                    DataObject::Integer32(max) => {
                        self.set_max_value(max as f64).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Float64/Float32/Integer for max_value".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Sensor has no attribute {}",
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
            "Sensor has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sensor_class_id() {
        let sensor = Sensor::with_default_obis();
        assert_eq!(sensor.class_id(), 102);
    }

    #[tokio::test]
    async fn test_sensor_obis_code() {
        let sensor = Sensor::with_default_obis();
        assert_eq!(sensor.obis_code(), Sensor::default_obis());
    }

    #[tokio::test]
    async fn test_sensor_initial_state() {
        let sensor = Sensor::with_default_obis();
        assert_eq!(sensor.value().await, 0.0);
        assert_eq!(sensor.unit().await, "");
        assert_eq!(sensor.sensor_status().await, SensorStatus::Ok);
        assert!(sensor.is_ok().await);
    }

    #[tokio::test]
    async fn test_sensor_with_range() {
        let sensor = Sensor::with_range(
            ObisCode::new(0, 0, 102, 0, 0, 255),
            "°C".to_string(),
            -40.0,
            100.0,
        );
        assert_eq!(sensor.unit().await, "°C");
        assert_eq!(sensor.min_value().await, -40.0);
        assert_eq!(sensor.max_value().await, 100.0);
    }

    #[tokio::test]
    async fn test_sensor_set_value() {
        let sensor = Sensor::with_default_obis();
        sensor.set_value(42.5).await;
        assert_eq!(sensor.value().await, 42.5);
        assert!(sensor.last_reading_time().await.is_some());
    }

    #[tokio::test]
    async fn test_sensor_set_unit() {
        let sensor = Sensor::with_default_obis();
        sensor.set_unit("kWh".to_string()).await;
        assert_eq!(sensor.unit().await, "kWh");
    }

    #[tokio::test]
    async fn test_sensor_set_status() {
        let sensor = Sensor::with_default_obis();
        sensor.set_sensor_status(SensorStatus::Error).await;
        assert_eq!(sensor.sensor_status().await, SensorStatus::Error);
        assert!(sensor.has_error().await);
    }

    #[tokio::test]
    async fn test_sensor_set_range() {
        let sensor = Sensor::with_default_obis();
        sensor.set_range(10.0, 100.0).await;
        assert_eq!(sensor.min_value().await, 10.0);
        assert_eq!(sensor.max_value().await, 100.0);
    }

    #[tokio::test]
    async fn test_sensor_mark_ok() {
        let sensor = Sensor::with_default_obis();
        sensor.mark_error().await;
        assert!(sensor.has_error().await);

        sensor.mark_ok().await;
        assert!(sensor.is_ok().await);
    }

    #[tokio::test]
    async fn test_sensor_mark_error() {
        let sensor = Sensor::with_default_obis();
        sensor.mark_error().await;
        assert_eq!(sensor.sensor_status().await, SensorStatus::Error);
    }

    #[tokio::test]
    async fn test_sensor_is_in_range() {
        let sensor = Sensor::with_range(
            ObisCode::new(0, 0, 102, 0, 0, 255),
            "V".to_string(),
            0.0,
            240.0,
        );
        sensor.set_value(120.0).await;
        assert!(sensor.is_in_range().await);

        sensor.set_value(300.0).await;
        assert!(!sensor.is_in_range().await);
    }

    #[tokio::test]
    async fn test_sensor_set_value_out_of_range() {
        let sensor = Sensor::with_range(
            ObisCode::new(0, 0, 102, 0, 0, 255),
            "°C".to_string(),
            -40.0,
            100.0,
        );
        sensor.set_value(150.0).await;
        assert_eq!(sensor.sensor_status().await, SensorStatus::OutOfRange);
    }

    #[tokio::test]
    async fn test_sensor_read_valid() {
        let sensor = Sensor::with_range(
            ObisCode::new(0, 0, 102, 0, 0, 255),
            "A".to_string(),
            0.0,
            10.0,
        );
        let result = sensor.read(5.0).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5.0);
        assert!(sensor.is_ok().await);
    }

    #[tokio::test]
    async fn test_sensor_read_out_of_range() {
        let sensor = Sensor::with_range(
            ObisCode::new(0, 0, 102, 0, 0, 255),
            "A".to_string(),
            0.0,
            10.0,
        );
        let result = sensor.read(15.0).await;
        assert!(result.is_err());
        assert_eq!(sensor.sensor_status().await, SensorStatus::OutOfRange);
    }

    #[tokio::test]
    async fn test_sensor_status_from_u8() {
        assert_eq!(SensorStatus::from_u8(0), SensorStatus::Ok);
        assert_eq!(SensorStatus::from_u8(1), SensorStatus::Error);
        assert_eq!(SensorStatus::from_u8(2), SensorStatus::NotConnected);
        assert_eq!(SensorStatus::from_u8(3), SensorStatus::Invalid);
        assert_eq!(SensorStatus::from_u8(4), SensorStatus::OutOfRange);
        assert_eq!(SensorStatus::from_u8(5), SensorStatus::Calibrating);
    }

    #[tokio::test]
    async fn test_sensor_status_is_ok() {
        assert!(SensorStatus::Ok.is_ok());
        assert!(!SensorStatus::Error.is_ok());
    }

    #[tokio::test]
    async fn test_sensor_status_has_error() {
        assert!(SensorStatus::Error.has_error());
        assert!(SensorStatus::NotConnected.has_error());
        assert!(!SensorStatus::Ok.has_error());
    }

    #[tokio::test]
    async fn test_sensor_get_attributes() {
        let sensor = Sensor::with_default_obis();

        // Test value
        let result = sensor.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Float64(v) => assert_eq!(v, 0.0),
            _ => panic!("Expected Float64"),
        }

        // Test sensor_status
        let result = sensor.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Enumerate(status) => assert_eq!(status, 0), // Ok
            _ => panic!("Expected Enumerate"),
        }
    }

    #[tokio::test]
    async fn test_sensor_set_attributes() {
        let sensor = Sensor::with_default_obis();

        sensor.set_attribute(2, DataObject::Float64(42.5), None)
            .await
            .unwrap();
        assert_eq!(sensor.value().await, 42.5);

        sensor.set_attribute(3, DataObject::OctetString(b"kWh".to_vec()), None)
            .await
            .unwrap();
        assert_eq!(sensor.unit().await, "kWh");
    }

    #[tokio::test]
    async fn test_sensor_set_value_from_integer() {
        let sensor = Sensor::with_default_obis();
        sensor.set_attribute(2, DataObject::Integer32(100), None)
            .await
            .unwrap();
        assert_eq!(sensor.value().await, 100.0);
    }

    #[tokio::test]
    async fn test_sensor_set_value_from_unsigned() {
        let sensor = Sensor::with_default_obis();
        sensor.set_attribute(2, DataObject::Unsigned16(50), None)
            .await
            .unwrap();
        assert_eq!(sensor.value().await, 50.0);
    }

    #[tokio::test]
    async fn test_sensor_set_min_max_value() {
        let sensor = Sensor::with_default_obis();
        sensor.set_attribute(5, DataObject::Float64(10.0), None)
            .await
            .unwrap();
        sensor.set_attribute(6, DataObject::Float64(100.0), None)
            .await
            .unwrap();
        assert_eq!(sensor.min_value().await, 10.0);
        assert_eq!(sensor.max_value().await, 100.0);
    }

    #[tokio::test]
    async fn test_sensor_set_min_max_from_float32() {
        let sensor = Sensor::with_default_obis();
        sensor.set_attribute(5, DataObject::Float32(5.0), None)
            .await
            .unwrap();
        sensor.set_attribute(6, DataObject::Float32(95.0), None)
            .await
            .unwrap();
        assert_eq!(sensor.min_value().await, 5.0);
        assert_eq!(sensor.max_value().await, 95.0);
    }

    #[tokio::test]
    async fn test_sensor_read_only_logical_name() {
        let sensor = Sensor::with_default_obis();
        let result = sensor
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 102, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sensor_invalid_attribute() {
        let sensor = Sensor::with_default_obis();
        let result = sensor.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sensor_invalid_method() {
        let sensor = Sensor::with_default_obis();
        let result = sensor.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sensor_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 102, 0, 0, 1);
        let sensor = Sensor::new(obis);
        assert_eq!(sensor.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_sensor_set_min_max_from_integer() {
        let sensor = Sensor::with_default_obis();
        sensor.set_attribute(5, DataObject::Integer32(-100), None)
            .await
            .unwrap();
        sensor.set_attribute(6, DataObject::Integer32(100), None)
            .await
            .unwrap();
        assert_eq!(sensor.min_value().await, -100.0);
        assert_eq!(sensor.max_value().await, 100.0);
    }
}
