//! Clock interface class (Class ID: 8)
//!
//! The Clock interface class represents a real-time clock in the meter.
//! It provides the current date/time, timezone, and DST information.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: time - Current date and time (CosemDateTime)
//! - Attribute 3: time_zone - Time zone offset in minutes from UTC
//! - Attribute 4: status - Clock status bits
//! - Attribute 5: daylight_savings_begin - DST begin date (optional)
//! - Attribute 6: daylight_savings_end - DST end date (optional)
//! - Attribute 7: daylight_savings_deviation - DST deviation in minutes (optional)
//! - Attribute 8: daylight_savings_enabled - Whether DST is enabled (optional)
//! - Attribute 9: clock_base - Clock base identifier (optional)
//!
//! # Methods
//!
//! - Method 1: adjust_time(dateTime) - Adjust the clock to a new date/time
//! - Method 2: adjust_time_to_timezone(timeZone) - Adjust the clock to a new timezone
//!
//! # Usage
//!
//! ```rust,no_run
//! use dlms_interface::Clock;
//! use dlms_core::{ObisCode, CosemDateTime};
//!
//! // Create a Clock with default OBIS (0-0:1.0.0.255)
//! let clock = Clock::with_default_obis();
//! ```

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{
    datatypes::{CosemDate, CosemDateTime, ClockStatus, CosemDateFormat},
    DlmsError, DlmsResult, ObisCode, DataObject,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Clock interface class (Class ID: 8)
///
/// Represents a real-time clock in the meter. This is one of the most
/// important interface classes for time-stamping meter readings and
/// scheduling operations.
///
/// Default OBIS: 0-0:1.0.0.255
#[derive(Debug, Clone)]
pub struct Clock {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,
    /// Current date and time
    time: Arc<RwLock<CosemDateTime>>,
    /// Time zone offset in minutes from UTC
    time_zone: Arc<RwLock<i16>>,
    /// Clock status bits (stored as u8 for the status byte)
    status: Arc<RwLock<u8>>,
    /// DST begin date (optional)
    daylight_savings_begin: Arc<RwLock<Option<CosemDate>>>,
    /// DST end date (optional)
    daylight_savings_end: Arc<RwLock<Option<CosemDate>>>,
    /// DST deviation in minutes (optional)
    daylight_savings_deviation: Arc<RwLock<i16>>,
    /// Whether DST is enabled
    daylight_savings_enabled: Arc<RwLock<bool>>,
    /// Clock base identifier
    clock_base: Arc<RwLock<u8>>,
}

impl Clock {
    /// Class ID for Clock
    pub const CLASS_ID: u16 = 8;

    /// Default OBIS code for Clock (0-0:1.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 1, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_TIME: u8 = 2;
    pub const ATTR_TIME_ZONE: u8 = 3;
    pub const ATTR_STATUS: u8 = 4;
    pub const ATTR_DAYLIGHT_SAVINGS_BEGIN: u8 = 5;
    pub const ATTR_DAYLIGHT_SAVINGS_END: u8 = 6;
    pub const ATTR_DAYLIGHT_SAVINGS_DEVIATION: u8 = 7;
    pub const ATTR_DAYLIGHT_SAVINGS_ENABLED: u8 = 8;
    pub const ATTR_CLOCK_BASE: u8 = 9;

    /// Method IDs
    pub const METHOD_ADJUST_TIME: u8 = 1;
    pub const METHOD_ADJUST_TIME_TO_TIMEZONE: u8 = 2;

    /// Create a new Clock object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `time` - Initial date/time value
    /// * `time_zone` - Time zone offset in minutes from UTC
    /// * `status` - Initial clock status byte
    pub fn new(
        logical_name: ObisCode,
        time: CosemDateTime,
        time_zone: i16,
        status: u8,
    ) -> Self {
        Self {
            logical_name,
            time: Arc::new(RwLock::new(time)),
            time_zone: Arc::new(RwLock::new(time_zone)),
            status: Arc::new(RwLock::new(status)),
            daylight_savings_begin: Arc::new(RwLock::new(None)),
            daylight_savings_end: Arc::new(RwLock::new(None)),
            daylight_savings_deviation: Arc::new(RwLock::new(60)),
            daylight_savings_enabled: Arc::new(RwLock::new(false)),
            clock_base: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with default OBIS code and a default time (2024-01-01 00:00:00)
    pub fn with_default_obis() -> Self {
        // Create a default time - 2024-01-01 00:00:00 UTC
        let default_time = CosemDateTime::new(2024, 1, 1, 0, 0, 0, 0, &[]).unwrap();
        Self::new(
            Self::default_obis(),
            default_time,
            0, // UTC
            0, // No status flags
        )
    }

    /// Get the current date/time
    pub async fn time(&self) -> CosemDateTime {
        self.time.read().await.clone()
    }

    /// Set the date/time
    pub async fn set_time(&self, time: CosemDateTime) {
        *self.time.write().await = time;
    }

    /// Get the time zone offset in minutes
    pub async fn time_zone(&self) -> i16 {
        *self.time_zone.read().await
    }

    /// Set the time zone offset
    pub async fn set_time_zone(&self, offset: i16) {
        *self.time_zone.write().await = offset;
    }

    /// Get the clock status byte
    pub async fn status(&self) -> u8 {
        *self.status.read().await
    }

    /// Set the clock status byte
    pub async fn set_status(&self, status: u8) {
        *self.status.write().await = status;
    }

    /// Get DST begin date
    pub async fn daylight_savings_begin(&self) -> Option<CosemDate> {
        self.daylight_savings_begin.read().await.clone()
    }

    /// Set DST begin date
    pub async fn set_daylight_savings_begin(&self, date: Option<CosemDate>) {
        *self.daylight_savings_begin.write().await = date;
    }

    /// Get DST end date
    pub async fn daylight_savings_end(&self) -> Option<CosemDate> {
        self.daylight_savings_end.read().await.clone()
    }

    /// Set DST end date
    pub async fn set_daylight_savings_end(&self, date: Option<CosemDate>) {
        *self.daylight_savings_end.write().await = date;
    }

    /// Get DST deviation in minutes
    pub async fn daylight_savings_deviation(&self) -> i16 {
        *self.daylight_savings_deviation.read().await
    }

    /// Set DST deviation
    pub async fn set_daylight_savings_deviation(&self, deviation: i16) {
        *self.daylight_savings_deviation.write().await = deviation;
    }

    /// Check if DST is enabled
    pub async fn daylight_savings_enabled(&self) -> bool {
        *self.daylight_savings_enabled.read().await
    }

    /// Enable or disable DST
    pub async fn set_daylight_savings_enabled(&self, enabled: bool) {
        *self.daylight_savings_enabled.write().await = enabled;
    }

    /// Get the clock base identifier
    pub async fn clock_base(&self) -> u8 {
        *self.clock_base.read().await
    }

    /// Set the clock base identifier
    pub async fn set_clock_base(&self, base: u8) {
        *self.clock_base.write().await = base;
    }

    /// Adjust the clock to a new date/time
    ///
    /// This corresponds to Method 1
    pub async fn adjust_time(&self, new_time: CosemDateTime) -> DlmsResult<()> {
        self.set_time(new_time).await;
        Ok(())
    }

    /// Adjust the clock to a new timezone
    ///
    /// This corresponds to Method 2
    pub async fn adjust_time_to_timezone(&self, time_zone: i16) -> DlmsResult<()> {
        self.set_time_zone(time_zone).await;
        Ok(())
    }

    /// Encode the time as DataObject
    async fn encode_time(&self) -> DataObject {
        let time = self.time.read().await;
        DataObject::OctetString(time.encode())
    }

    /// Encode DST begin date as DataObject
    async fn encode_dst_begin(&self) -> DataObject {
        match self.daylight_savings_begin.read().await.as_ref() {
            Some(date) => DataObject::OctetString(date.encode()),
            None => DataObject::Null,
        }
    }

    /// Encode DST end date as DataObject
    async fn encode_dst_end(&self) -> DataObject {
        match self.daylight_savings_end.read().await.as_ref() {
            Some(date) => DataObject::OctetString(date.encode()),
            None => DataObject::Null,
        }
    }
}

#[async_trait]
impl CosemObject for Clock {
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
            Self::ATTR_TIME => {
                Ok(self.encode_time().await)
            }
            Self::ATTR_TIME_ZONE => {
                Ok(DataObject::Integer16(self.time_zone().await))
            }
            Self::ATTR_STATUS => {
                Ok(DataObject::Unsigned8(self.status().await))
            }
            Self::ATTR_DAYLIGHT_SAVINGS_BEGIN => {
                Ok(self.encode_dst_begin().await)
            }
            Self::ATTR_DAYLIGHT_SAVINGS_END => {
                Ok(self.encode_dst_end().await)
            }
            Self::ATTR_DAYLIGHT_SAVINGS_DEVIATION => {
                Ok(DataObject::Integer16(self.daylight_savings_deviation().await))
            }
            Self::ATTR_DAYLIGHT_SAVINGS_ENABLED => {
                Ok(DataObject::Boolean(self.daylight_savings_enabled().await))
            }
            Self::ATTR_CLOCK_BASE => {
                Ok(DataObject::Unsigned8(self.clock_base().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Clock interface class has no attribute {}",
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
            Self::ATTR_TIME => {
                if let DataObject::OctetString(bytes) = value {
                    if bytes.len() >= 12 {
                        let dt = CosemDateTime::decode(&bytes)?;
                        self.set_time(dt).await;
                        return Ok(());
                    }
                }
                Err(DlmsError::InvalidData(
                    "Expected 12-byte octet string for time".to_string(),
                ))
            }
            Self::ATTR_TIME_ZONE => {
                if let DataObject::Integer16(tz) = value {
                    self.set_time_zone(tz).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Integer16 for time_zone".to_string(),
                    ))
                }
            }
            Self::ATTR_STATUS => {
                if let DataObject::Unsigned8(status) = value {
                    self.set_status(status).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for status".to_string(),
                    ))
                }
            }
            Self::ATTR_DAYLIGHT_SAVINGS_BEGIN => {
                match value {
                    DataObject::OctetString(bytes) if bytes.len() >= 5 => {
                        let date = CosemDate::decode(&bytes)?;
                        self.set_daylight_savings_begin(Some(date)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_daylight_savings_begin(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected octet string or Null for dst_begin".to_string(),
                    )),
                }
            }
            Self::ATTR_DAYLIGHT_SAVINGS_END => {
                match value {
                    DataObject::OctetString(bytes) if bytes.len() >= 5 => {
                        let date = CosemDate::decode(&bytes)?;
                        self.set_daylight_savings_end(Some(date)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_daylight_savings_end(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected octet string or Null for dst_end".to_string(),
                    )),
                }
            }
            Self::ATTR_DAYLIGHT_SAVINGS_DEVIATION => {
                if let DataObject::Integer16(dev) = value {
                    self.set_daylight_savings_deviation(dev).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Integer16 for dst_deviation".to_string(),
                    ))
                }
            }
            Self::ATTR_DAYLIGHT_SAVINGS_ENABLED => {
                if let DataObject::Boolean(enabled) = value {
                    self.set_daylight_savings_enabled(enabled).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Boolean for dst_enabled".to_string(),
                    ))
                }
            }
            Self::ATTR_CLOCK_BASE => {
                if let DataObject::Unsigned8(base) = value {
                    self.set_clock_base(base).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for clock_base".to_string(),
                    ))
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Clock interface class has no attribute {}",
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
            Self::METHOD_ADJUST_TIME => {
                if let Some(DataObject::OctetString(bytes)) = parameters {
                    if bytes.len() >= 12 {
                        let dt = CosemDateTime::decode(&bytes)?;
                        self.adjust_time(dt).await?;
                        return Ok(None);
                    }
                }
                Err(DlmsError::InvalidData(
                    "Method 1 requires a 12-byte date/time parameter".to_string(),
                ))
            }
            Self::METHOD_ADJUST_TIME_TO_TIMEZONE => {
                if let Some(DataObject::Integer16(tz)) = parameters {
                    self.adjust_time_to_timezone(tz).await?;
                    return Ok(None);
                }
                Err(DlmsError::InvalidData(
                    "Method 2 requires an Integer16 timezone parameter".to_string(),
                ))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Clock interface class has no method {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dlms_core::datatypes::Field;

    #[tokio::test]
    async fn test_clock_class_id() {
        let clock = Clock::with_default_obis();
        assert_eq!(clock.class_id(), 8);
    }

    #[tokio::test]
    async fn test_clock_obis_code() {
        let clock = Clock::with_default_obis();
        assert_eq!(clock.obis_code(), Clock::default_obis());
    }

    #[tokio::test]
    async fn test_clock_get_time() {
        let clock = Clock::with_default_obis();
        let result = clock.get_attribute(2, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 12);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_clock_set_time_zone() {
        let clock = Clock::with_default_obis();
        clock.set_attribute(3, DataObject::Integer16(480), None).await.unwrap();
        assert_eq!(clock.time_zone().await, 480); // UTC+8
    }

    #[tokio::test]
    async fn test_clock_status() {
        let clock = Clock::with_default_obis();
        let result = clock.get_attribute(4, None).await.unwrap();

        match result {
            DataObject::Unsigned8(status) => {
                // Check default status is 0 (no flags)
                assert_eq!(status, 0);
            }
            _ => panic!("Expected Unsigned8"),
        }
    }

    #[tokio::test]
    async fn test_clock_method_adjust_time() {
        let clock = Clock::with_default_obis();
        let new_dt = CosemDateTime::new(2024, 6, 15, 12, 30, 0, 0, &[]).unwrap();

        let param = DataObject::OctetString(new_dt.encode());
        clock.invoke_method(1, Some(param), None).await.unwrap();

        let retrieved = clock.time().await;
        assert_eq!(retrieved.get(Field::Year).unwrap(), 2024);
        assert_eq!(retrieved.get(Field::Month).unwrap(), 6);
        assert_eq!(retrieved.get(Field::DayOfMonth).unwrap(), 15);
    }

    #[tokio::test]
    async fn test_clock_method_adjust_timezone() {
        let clock = Clock::with_default_obis();
        let param = DataObject::Integer16(-300); // UTC-5
        clock.invoke_method(2, Some(param), None).await.unwrap();

        assert_eq!(clock.time_zone().await, -300);
    }

    #[tokio::test]
    async fn test_clock_dst_handling() {
        let clock = Clock::with_default_obis();

        // Set DST enabled
        clock.set_daylight_savings_enabled(true).await;
        let result = clock.get_attribute(8, None).await.unwrap();
        assert_eq!(result, DataObject::Boolean(true));

        // Set DST begin date
        let dst_begin = CosemDate::new(2024, 3, 10).unwrap();
        clock.set_daylight_savings_begin(Some(dst_begin)).await;
        let result = clock.get_attribute(5, None).await.unwrap();
        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 5);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_clock_invalid_attribute() {
        let clock = Clock::with_default_obis();
        let result = clock.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_clock_set_status() {
        let clock = Clock::with_default_obis();
        // Set status with DaylightSavingActive flag
        clock.set_attribute(4, DataObject::Unsigned8(ClockStatus::DaylightSavingActive as u8), None).await.unwrap();
        assert_eq!(clock.status().await, ClockStatus::DaylightSavingActive as u8);
    }

    #[tokio::test]
    async fn test_clock_set_dst_begin() {
        let clock = Clock::with_default_obis();
        let dst_begin = CosemDate::new(2024, 3, 10).unwrap();

        clock.set_attribute(5, DataObject::OctetString(dst_begin.encode()), None).await.unwrap();

        let retrieved = clock.daylight_savings_begin().await;
        assert!(retrieved.is_some());
        let date = retrieved.unwrap();
        assert_eq!(date.get(Field::Year).unwrap(), 2024);
        assert_eq!(date.get(Field::Month).unwrap(), 3);
        assert_eq!(date.get(Field::DayOfMonth).unwrap(), 10);
    }

    #[tokio::test]
    async fn test_clock_set_dst_null() {
        let clock = Clock::with_default_obis();
        let dst_begin = CosemDate::new(2024, 3, 10).unwrap();
        clock.set_daylight_savings_begin(Some(dst_begin)).await;

        // Clear with Null
        clock.set_attribute(5, DataObject::Null, None).await.unwrap();
        assert!(clock.daylight_savings_begin().await.is_none());
    }

    #[tokio::test]
    async fn test_clock_base() {
        let clock = Clock::with_default_obis();
        clock.set_attribute(9, DataObject::Unsigned8(1), None).await.unwrap();
        assert_eq!(clock.clock_base().await, 1);
    }
}
