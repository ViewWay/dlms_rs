//! Utility Meter interface class (Class ID: 72)
//!
//! The Utility Meter interface class manages readings from external utility meters.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: meter_type - Type of utility meter (electricity, gas, water, etc.)
//! - Attribute 3: current_reading - Current meter reading
//! - Attribute 4: unit_of_measure - Unit of measurement
//! - Attribute 5: last_read_time - Time of last reading
//! - Attribute 6: reading_status - Status of reading operation
//! - Attribute 7: meter_id - Unique identifier for the meter
//! - Attribute 8: calibration_date - Last calibration date

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Utility Meter Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UtilityMeterType {
    /// Electricity meter
    Electricity = 0,
    /// Gas meter
    Gas = 1,
    /// Water meter
    Water = 2,
    /// Heat meter
    Heat = 3,
    /// Cold water meter
    ColdWater = 4,
    /// Hot water meter
    HotWater = 5,
    /// Custom meter type
    Custom = 6,
}

impl UtilityMeterType {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Electricity,
            1 => Self::Gas,
            2 => Self::Water,
            3 => Self::Heat,
            4 => Self::ColdWater,
            5 => Self::HotWater,
            _ => Self::Custom,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if this is a water meter
    pub fn is_water_meter(self) -> bool {
        matches!(self, Self::Water | Self::ColdWater | Self::HotWater)
    }
}

/// Reading Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ReadingStatus {
    /// No reading yet
    NoReading = 0,
    /// Reading successful
    Success = 1,
    /// Reading failed
    Failed = 2,
    /// Reading in progress
    InProgress = 3,
    /// Meter not accessible
    NotAccessible = 4,
    /// Reading timeout
    Timeout = 5,
}

impl ReadingStatus {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::NoReading,
            1 => Self::Success,
            2 => Self::Failed,
            3 => Self::InProgress,
            4 => Self::NotAccessible,
            5 => Self::Timeout,
            _ => Self::NoReading,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if reading was successful
    pub fn is_successful(self) -> bool {
        matches!(self, Self::Success)
    }

    /// Check if reading failed
    pub fn is_failed(self) -> bool {
        matches!(self, Self::Failed | Self::Timeout | Self::NotAccessible)
    }

    /// Check if reading is in progress
    pub fn is_in_progress(self) -> bool {
        matches!(self, Self::InProgress)
    }
}

/// Utility Meter interface class (Class ID: 72)
///
/// Default OBIS: 0-0:72.0.0.255
///
/// This class manages readings from external utility meters.
#[derive(Debug, Clone)]
pub struct UtilityMeter {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Type of utility meter
    meter_type: Arc<RwLock<UtilityMeterType>>,

    /// Current meter reading
    current_reading: Arc<RwLock<i64>>,

    /// Unit of measurement
    unit_of_measure: Arc<RwLock<String>>,

    /// Time of last reading (Unix timestamp)
    last_read_time: Arc<RwLock<Option<i64>>>,

    /// Status of reading operation
    reading_status: Arc<RwLock<ReadingStatus>>,

    /// Unique identifier for the meter
    meter_id: Arc<RwLock<String>>,

    /// Last calibration date (Unix timestamp)
    calibration_date: Arc<RwLock<Option<i64>>>,
}

impl UtilityMeter {
    /// Class ID for UtilityMeter
    pub const CLASS_ID: u16 = 72;

    /// Default OBIS code for UtilityMeter (0-0:72.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 72, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_METER_TYPE: u8 = 2;
    pub const ATTR_CURRENT_READING: u8 = 3;
    pub const ATTR_UNIT_OF_MEASURE: u8 = 4;
    pub const ATTR_LAST_READ_TIME: u8 = 5;
    pub const ATTR_READING_STATUS: u8 = 6;
    pub const ATTR_METER_ID: u8 = 7;
    pub const ATTR_CALIBRATION_DATE: u8 = 8;

    /// Create a new UtilityMeter object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `meter_type` - Type of utility meter
    pub fn new(logical_name: ObisCode, meter_type: UtilityMeterType) -> Self {
        Self {
            logical_name,
            meter_type: Arc::new(RwLock::new(meter_type)),
            current_reading: Arc::new(RwLock::new(0)),
            unit_of_measure: Arc::new(RwLock::new(String::new())),
            last_read_time: Arc::new(RwLock::new(None)),
            reading_status: Arc::new(RwLock::new(ReadingStatus::NoReading)),
            meter_id: Arc::new(RwLock::new(String::new())),
            calibration_date: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis(), UtilityMeterType::Electricity)
    }

    /// Get the meter type
    pub async fn meter_type(&self) -> UtilityMeterType {
        *self.meter_type.read().await
    }

    /// Set the meter type
    pub async fn set_meter_type(&self, meter_type: UtilityMeterType) {
        *self.meter_type.write().await = meter_type;
    }

    /// Get the current reading
    pub async fn current_reading(&self) -> i64 {
        *self.current_reading.read().await
    }

    /// Set the current reading
    pub async fn set_current_reading(&self, reading: i64) {
        *self.current_reading.write().await = reading;
    }

    /// Get the unit of measure
    pub async fn unit_of_measure(&self) -> String {
        self.unit_of_measure.read().await.clone()
    }

    /// Set the unit of measure
    pub async fn set_unit_of_measure(&self, unit: String) {
        *self.unit_of_measure.write().await = unit;
    }

    /// Get the last read time
    pub async fn last_read_time(&self) -> Option<i64> {
        *self.last_read_time.read().await
    }

    /// Set the last read time
    pub async fn set_last_read_time(&self, time: Option<i64>) {
        *self.last_read_time.write().await = time;
    }

    /// Get the reading status
    pub async fn reading_status(&self) -> ReadingStatus {
        *self.reading_status.read().await
    }

    /// Set the reading status
    pub async fn set_reading_status(&self, status: ReadingStatus) {
        *self.reading_status.write().await = status;
    }

    /// Get the meter ID
    pub async fn meter_id(&self) -> String {
        self.meter_id.read().await.clone()
    }

    /// Set the meter ID
    pub async fn set_meter_id(&self, id: String) {
        *self.meter_id.write().await = id;
    }

    /// Get the calibration date
    pub async fn calibration_date(&self) -> Option<i64> {
        *self.calibration_date.read().await
    }

    /// Set the calibration date
    pub async fn set_calibration_date(&self, date: Option<i64>) {
        *self.calibration_date.write().await = date;
    }

    /// Record a new reading
    pub async fn record_reading(&self, reading: i64) {
        *self.current_reading.write().await = reading;
        *self.last_read_time.write().await = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
        );
        *self.reading_status.write().await = ReadingStatus::Success;
    }

    /// Start reading
    pub async fn start_reading(&self) {
        *self.reading_status.write().await = ReadingStatus::InProgress;
    }

    /// Mark reading as failed
    pub async fn mark_reading_failed(&self) {
        *self.reading_status.write().await = ReadingStatus::Failed;
    }

    /// Mark reading as timeout
    pub async fn mark_reading_timeout(&self) {
        *self.reading_status.write().await = ReadingStatus::Timeout;
    }

    /// Mark meter as not accessible
    pub async fn mark_not_accessible(&self) {
        *self.reading_status.write().await = ReadingStatus::NotAccessible;
    }

    /// Check if reading was successful
    pub async fn is_reading_successful(&self) -> bool {
        self.reading_status().await.is_successful()
    }

    /// Check if reading failed
    pub async fn is_reading_failed(&self) -> bool {
        self.reading_status().await.is_failed()
    }

    /// Check if reading is in progress
    pub async fn is_reading_in_progress(&self) -> bool {
        self.reading_status().await.is_in_progress()
    }

    /// Check if this is a water meter
    pub async fn is_water_meter(&self) -> bool {
        self.meter_type().await.is_water_meter()
    }
}

#[async_trait]
impl CosemObject for UtilityMeter {
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
            Self::ATTR_METER_TYPE => {
                Ok(DataObject::Enumerate(self.meter_type().await.to_u8()))
            }
            Self::ATTR_CURRENT_READING => {
                Ok(DataObject::Integer64(self.current_reading().await))
            }
            Self::ATTR_UNIT_OF_MEASURE => {
                Ok(DataObject::OctetString(self.unit_of_measure().await.into_bytes()))
            }
            Self::ATTR_LAST_READ_TIME => {
                match self.last_read_time().await {
                    Some(timestamp) => Ok(DataObject::Integer64(timestamp)),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_READING_STATUS => {
                Ok(DataObject::Enumerate(self.reading_status().await.to_u8()))
            }
            Self::ATTR_METER_ID => {
                Ok(DataObject::OctetString(self.meter_id().await.into_bytes()))
            }
            Self::ATTR_CALIBRATION_DATE => {
                match self.calibration_date().await {
                    Some(timestamp) => Ok(DataObject::Integer64(timestamp)),
                    None => Ok(DataObject::Null),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "UtilityMeter has no attribute {}",
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
            Self::ATTR_METER_TYPE => {
                match value {
                    DataObject::Enumerate(meter_type) => {
                        self.set_meter_type(UtilityMeterType::from_u8(meter_type)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for meter_type".to_string(),
                    )),
                }
            }
            Self::ATTR_CURRENT_READING => {
                match value {
                    DataObject::Integer64(reading) => {
                        self.set_current_reading(reading).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 for current_reading".to_string(),
                    )),
                }
            }
            Self::ATTR_UNIT_OF_MEASURE => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let unit = String::from_utf8_lossy(&bytes).to_string();
                        self.set_unit_of_measure(unit).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for unit_of_measure".to_string(),
                    )),
                }
            }
            Self::ATTR_LAST_READ_TIME => {
                match value {
                    DataObject::Integer64(timestamp) => {
                        self.set_last_read_time(Some(timestamp)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_last_read_time(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 or Null for last_read_time".to_string(),
                    )),
                }
            }
            Self::ATTR_READING_STATUS => {
                match value {
                    DataObject::Enumerate(status) => {
                        self.set_reading_status(ReadingStatus::from_u8(status)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for reading_status".to_string(),
                    )),
                }
            }
            Self::ATTR_METER_ID => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let id = String::from_utf8_lossy(&bytes).to_string();
                        self.set_meter_id(id).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for meter_id".to_string(),
                    )),
                }
            }
            Self::ATTR_CALIBRATION_DATE => {
                match value {
                    DataObject::Integer64(timestamp) => {
                        self.set_calibration_date(Some(timestamp)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_calibration_date(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 or Null for calibration_date".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "UtilityMeter has no attribute {}",
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
            "UtilityMeter has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_utility_meter_class_id() {
        let um = UtilityMeter::with_default_obis();
        assert_eq!(um.class_id(), 72);
    }

    #[tokio::test]
    async fn test_utility_meter_obis_code() {
        let um = UtilityMeter::with_default_obis();
        assert_eq!(um.obis_code(), UtilityMeter::default_obis());
    }

    #[tokio::test]
    async fn test_utility_meter_type_from_u8() {
        assert_eq!(UtilityMeterType::from_u8(0), UtilityMeterType::Electricity);
        assert_eq!(UtilityMeterType::from_u8(1), UtilityMeterType::Gas);
        assert_eq!(UtilityMeterType::from_u8(2), UtilityMeterType::Water);
        assert_eq!(UtilityMeterType::from_u8(3), UtilityMeterType::Heat);
        assert_eq!(UtilityMeterType::from_u8(4), UtilityMeterType::ColdWater);
        assert_eq!(UtilityMeterType::from_u8(5), UtilityMeterType::HotWater);
    }

    #[tokio::test]
    async fn test_utility_meter_type_is_water_meter() {
        assert!(!UtilityMeterType::Electricity.is_water_meter());
        assert!(!UtilityMeterType::Gas.is_water_meter());
        assert!(UtilityMeterType::Water.is_water_meter());
        assert!(UtilityMeterType::ColdWater.is_water_meter());
        assert!(UtilityMeterType::HotWater.is_water_meter());
        assert!(!UtilityMeterType::Heat.is_water_meter());
    }

    #[tokio::test]
    async fn test_reading_status_from_u8() {
        assert_eq!(ReadingStatus::from_u8(0), ReadingStatus::NoReading);
        assert_eq!(ReadingStatus::from_u8(1), ReadingStatus::Success);
        assert_eq!(ReadingStatus::from_u8(2), ReadingStatus::Failed);
        assert_eq!(ReadingStatus::from_u8(3), ReadingStatus::InProgress);
        assert_eq!(ReadingStatus::from_u8(4), ReadingStatus::NotAccessible);
        assert_eq!(ReadingStatus::from_u8(5), ReadingStatus::Timeout);
    }

    #[tokio::test]
    async fn test_reading_status_is_successful() {
        assert!(ReadingStatus::Success.is_successful());
        assert!(!ReadingStatus::NoReading.is_successful());
        assert!(!ReadingStatus::Failed.is_successful());
    }

    #[tokio::test]
    async fn test_reading_status_is_failed() {
        assert!(ReadingStatus::Failed.is_failed());
        assert!(ReadingStatus::Timeout.is_failed());
        assert!(ReadingStatus::NotAccessible.is_failed());
        assert!(!ReadingStatus::Success.is_failed());
        assert!(!ReadingStatus::InProgress.is_failed());
    }

    #[tokio::test]
    async fn test_reading_status_is_in_progress() {
        assert!(ReadingStatus::InProgress.is_in_progress());
        assert!(!ReadingStatus::Success.is_in_progress());
        assert!(!ReadingStatus::Failed.is_in_progress());
    }

    #[tokio::test]
    async fn test_utility_meter_initial_state() {
        let um = UtilityMeter::with_default_obis();
        assert_eq!(um.meter_type().await, UtilityMeterType::Electricity);
        assert_eq!(um.current_reading().await, 0);
        assert_eq!(um.unit_of_measure().await, "");
        assert_eq!(um.reading_status().await, ReadingStatus::NoReading);
        assert_eq!(um.meter_id().await, "");
    }

    #[tokio::test]
    async fn test_utility_meter_set_meter_type() {
        let um = UtilityMeter::with_default_obis();
        um.set_meter_type(UtilityMeterType::Gas).await;
        assert_eq!(um.meter_type().await, UtilityMeterType::Gas);
    }

    #[tokio::test]
    async fn test_utility_meter_set_current_reading() {
        let um = UtilityMeter::with_default_obis();
        um.set_current_reading(12345).await;
        assert_eq!(um.current_reading().await, 12345);
    }

    #[tokio::test]
    async fn test_utility_meter_set_unit_of_measure() {
        let um = UtilityMeter::with_default_obis();
        um.set_unit_of_measure("kWh".to_string()).await;
        assert_eq!(um.unit_of_measure().await, "kWh");
    }

    #[tokio::test]
    async fn test_utility_meter_record_reading() {
        let um = UtilityMeter::with_default_obis();
        um.record_reading(99999).await;

        assert_eq!(um.current_reading().await, 99999);
        assert_eq!(um.reading_status().await, ReadingStatus::Success);
        assert!(um.last_read_time().await.is_some());
    }

    #[tokio::test]
    async fn test_utility_meter_start_reading() {
        let um = UtilityMeter::with_default_obis();
        um.start_reading().await;
        assert_eq!(um.reading_status().await, ReadingStatus::InProgress);
        assert!(um.is_reading_in_progress().await);
    }

    #[tokio::test]
    async fn test_utility_meter_mark_reading_failed() {
        let um = UtilityMeter::with_default_obis();
        um.mark_reading_failed().await;
        assert_eq!(um.reading_status().await, ReadingStatus::Failed);
        assert!(um.is_reading_failed().await);
    }

    #[tokio::test]
    async fn test_utility_meter_mark_reading_timeout() {
        let um = UtilityMeter::with_default_obis();
        um.mark_reading_timeout().await;
        assert_eq!(um.reading_status().await, ReadingStatus::Timeout);
        assert!(um.is_reading_failed().await);
    }

    #[tokio::test]
    async fn test_utility_meter_mark_not_accessible() {
        let um = UtilityMeter::with_default_obis();
        um.mark_not_accessible().await;
        assert_eq!(um.reading_status().await, ReadingStatus::NotAccessible);
    }

    #[tokio::test]
    async fn test_utility_meter_is_water_meter() {
        let um = UtilityMeter::with_default_obis();
        assert!(!um.is_water_meter().await);

        um.set_meter_type(UtilityMeterType::Water).await;
        assert!(um.is_water_meter().await);
    }

    #[tokio::test]
    async fn test_utility_meter_get_attributes() {
        let um = UtilityMeter::with_default_obis();

        // Test meter_type
        let result = um.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Enumerate(mt) => assert_eq!(mt, 0), // Electricity
            _ => panic!("Expected Enumerate"),
        }

        // Test current_reading
        let result = um.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Integer64(reading) => assert_eq!(reading, 0),
            _ => panic!("Expected Integer64"),
        }
    }

    #[tokio::test]
    async fn test_utility_meter_set_attributes() {
        let um = UtilityMeter::with_default_obis();

        um.set_attribute(2, DataObject::Enumerate(2), None) // Water
            .await
            .unwrap();
        assert_eq!(um.meter_type().await, UtilityMeterType::Water);

        um.set_attribute(3, DataObject::Integer64(500), None)
            .await
            .unwrap();
        assert_eq!(um.current_reading().await, 500);
    }

    #[tokio::test]
    async fn test_utility_meter_read_only_logical_name() {
        let um = UtilityMeter::with_default_obis();
        let result = um
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 72, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_utility_meter_invalid_attribute() {
        let um = UtilityMeter::with_default_obis();
        let result = um.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_utility_meter_invalid_method() {
        let um = UtilityMeter::with_default_obis();
        let result = um.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }
}
