//! Alarm interface class (Class ID: 73)
//!
//! The Alarm interface class manages alarm/event notifications for meters.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: alarm_type - Type of alarm
//! - Attribute 3: alarm_status - Current alarm status
//! - Attribute 4: alarm_value - Value associated with the alarm
//! - Attribute 5: alarm_threshold - Threshold for triggering alarm
//! - Attribute 6: alarm_time - Time when alarm was triggered
//! - Attribute 7: alarm_count - Number of times alarm was triggered
//! - Attribute 8: alarm_enabled - Whether the alarm is enabled

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Alarm Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlarmType {
    /// No alarm
    None = 0,
    /// Power failure alarm
    PowerFailure = 1,
    /// Low battery alarm
    LowBattery = 2,
    /// Tamper alarm
    Tamper = 3,
    /// Overload alarm
    Overload = 4,
    /// Underload alarm
    Underload = 5,
    /// Phase loss alarm
    PhaseLoss = 6,
    /// Overvoltage alarm
    Overvoltage = 7,
    /// Undervoltage alarm
    Undervoltage = 8,
    /// Custom alarm type
    Custom = 255,
}

impl AlarmType {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::PowerFailure,
            2 => Self::LowBattery,
            3 => Self::Tamper,
            4 => Self::Overload,
            5 => Self::Underload,
            6 => Self::PhaseLoss,
            7 => Self::Overvoltage,
            8 => Self::Undervoltage,
            _ => Self::Custom,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if this is a critical alarm
    pub fn is_critical(self) -> bool {
        matches!(self, Self::PowerFailure | Self::Tamper | Self::PhaseLoss)
    }
}

/// Alarm Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlarmStatus {
    /// Alarm inactive (no alarm)
    Inactive = 0,
    /// Alarm active
    Active = 1,
    /// Alarm acknowledged
    Acknowledged = 2,
    /// Alarm disabled
    Disabled = 3,
}

impl AlarmStatus {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Inactive,
            1 => Self::Active,
            2 => Self::Acknowledged,
            3 => Self::Disabled,
            _ => Self::Inactive,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if alarm is active
    pub fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }

    /// Check if alarm is inactive
    pub fn is_inactive(self) -> bool {
        matches!(self, Self::Inactive)
    }

    /// Check if alarm can trigger
    pub fn can_trigger(self) -> bool {
        !matches!(self, Self::Disabled)
    }
}

/// Alarm interface class (Class ID: 73)
///
/// Default OBIS: 0-0:73.0.0.255
///
/// This class manages alarm/event notifications for meters.
#[derive(Debug, Clone)]
pub struct Alarm {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Type of alarm
    alarm_type: Arc<RwLock<AlarmType>>,

    /// Current alarm status
    alarm_status: Arc<RwLock<AlarmStatus>>,

    /// Value associated with the alarm
    alarm_value: Arc<RwLock<i64>>,

    /// Threshold for triggering alarm
    alarm_threshold: Arc<RwLock<i64>>,

    /// Time when alarm was triggered (Unix timestamp)
    alarm_time: Arc<RwLock<Option<i64>>>,

    /// Number of times alarm was triggered
    alarm_count: Arc<RwLock<u32>>,

    /// Whether the alarm is enabled
    alarm_enabled: Arc<RwLock<bool>>,
}

impl Alarm {
    /// Class ID for Alarm
    pub const CLASS_ID: u16 = 73;

    /// Default OBIS code for Alarm (0-0:73.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 73, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_ALARM_TYPE: u8 = 2;
    pub const ATTR_ALARM_STATUS: u8 = 3;
    pub const ATTR_ALARM_VALUE: u8 = 4;
    pub const ATTR_ALARM_THRESHOLD: u8 = 5;
    pub const ATTR_ALARM_TIME: u8 = 6;
    pub const ATTR_ALARM_COUNT: u8 = 7;
    pub const ATTR_ALARM_ENABLED: u8 = 8;

    /// Create a new Alarm object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `alarm_type` - Type of alarm
    pub fn new(logical_name: ObisCode, alarm_type: AlarmType) -> Self {
        Self {
            logical_name,
            alarm_type: Arc::new(RwLock::new(alarm_type)),
            alarm_status: Arc::new(RwLock::new(AlarmStatus::Inactive)),
            alarm_value: Arc::new(RwLock::new(0)),
            alarm_threshold: Arc::new(RwLock::new(0)),
            alarm_time: Arc::new(RwLock::new(None)),
            alarm_count: Arc::new(RwLock::new(0)),
            alarm_enabled: Arc::new(RwLock::new(true)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis(), AlarmType::None)
    }

    /// Get the alarm type
    pub async fn alarm_type(&self) -> AlarmType {
        *self.alarm_type.read().await
    }

    /// Set the alarm type
    pub async fn set_alarm_type(&self, alarm_type: AlarmType) {
        *self.alarm_type.write().await = alarm_type;
    }

    /// Get the alarm status
    pub async fn alarm_status(&self) -> AlarmStatus {
        *self.alarm_status.read().await
    }

    /// Set the alarm status
    pub async fn set_alarm_status(&self, status: AlarmStatus) {
        *self.alarm_status.write().await = status;
    }

    /// Get the alarm value
    pub async fn alarm_value(&self) -> i64 {
        *self.alarm_value.read().await
    }

    /// Set the alarm value
    pub async fn set_alarm_value(&self, value: i64) {
        *self.alarm_value.write().await = value;
    }

    /// Get the alarm threshold
    pub async fn alarm_threshold(&self) -> i64 {
        *self.alarm_threshold.read().await
    }

    /// Set the alarm threshold
    pub async fn set_alarm_threshold(&self, threshold: i64) {
        *self.alarm_threshold.write().await = threshold;
    }

    /// Get the alarm time
    pub async fn alarm_time(&self) -> Option<i64> {
        *self.alarm_time.read().await
    }

    /// Set the alarm time
    pub async fn set_alarm_time(&self, time: Option<i64>) {
        *self.alarm_time.write().await = time;
    }

    /// Get the alarm count
    pub async fn alarm_count(&self) -> u32 {
        *self.alarm_count.read().await
    }

    /// Get whether the alarm is enabled
    pub async fn alarm_enabled(&self) -> bool {
        *self.alarm_enabled.read().await
    }

    /// Set whether the alarm is enabled
    pub async fn set_alarm_enabled(&self, enabled: bool) {
        *self.alarm_enabled.write().await = enabled;
    }

    /// Trigger the alarm
    pub async fn trigger(&self) {
        if !self.alarm_enabled().await {
            return;
        }
        *self.alarm_status.write().await = AlarmStatus::Active;
        *self.alarm_time.write().await = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
        );
        *self.alarm_count.write().await += 1;
    }

    /// Acknowledge the alarm
    pub async fn acknowledge(&self) {
        *self.alarm_status.write().await = AlarmStatus::Acknowledged;
    }

    /// Clear/reset the alarm
    pub async fn clear(&self) {
        *self.alarm_status.write().await = AlarmStatus::Inactive;
        *self.alarm_time.write().await = None;
    }

    /// Disable the alarm
    pub async fn disable(&self) {
        *self.alarm_enabled.write().await = false;
    }

    /// Enable the alarm
    pub async fn enable(&self) {
        *self.alarm_enabled.write().await = true;
    }

    /// Check if alarm is active
    pub async fn is_active(&self) -> bool {
        self.alarm_status().await.is_active()
    }

    /// Check if alarm is inactive
    pub async fn is_inactive(&self) -> bool {
        self.alarm_status().await.is_inactive()
    }

    /// Check if alarm can trigger
    pub async fn can_trigger(&self) -> bool {
        self.alarm_enabled().await && self.alarm_status().await.can_trigger()
    }

    /// Check if alarm is critical
    pub async fn is_critical(&self) -> bool {
        self.alarm_type().await.is_critical()
    }

    /// Check value against threshold and trigger if needed
    pub async fn check_and_trigger(&self) {
        if !self.can_trigger().await {
            return;
        }

        let value = self.alarm_value().await;
        let threshold = self.alarm_threshold().await;

        // Trigger if value exceeds threshold (simple implementation)
        if value > threshold {
            self.trigger().await;
        }
    }
}

#[async_trait]
impl CosemObject for Alarm {
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
            Self::ATTR_ALARM_TYPE => {
                Ok(DataObject::Enumerate(self.alarm_type().await.to_u8()))
            }
            Self::ATTR_ALARM_STATUS => {
                Ok(DataObject::Enumerate(self.alarm_status().await.to_u8()))
            }
            Self::ATTR_ALARM_VALUE => {
                Ok(DataObject::Integer64(self.alarm_value().await))
            }
            Self::ATTR_ALARM_THRESHOLD => {
                Ok(DataObject::Integer64(self.alarm_threshold().await))
            }
            Self::ATTR_ALARM_TIME => {
                match self.alarm_time().await {
                    Some(timestamp) => Ok(DataObject::Integer64(timestamp)),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_ALARM_COUNT => {
                Ok(DataObject::Unsigned32(self.alarm_count().await))
            }
            Self::ATTR_ALARM_ENABLED => {
                Ok(DataObject::Boolean(self.alarm_enabled().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Alarm has no attribute {}",
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
            Self::ATTR_ALARM_TYPE => {
                match value {
                    DataObject::Enumerate(alarm_type) => {
                        self.set_alarm_type(AlarmType::from_u8(alarm_type)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for alarm_type".to_string(),
                    )),
                }
            }
            Self::ATTR_ALARM_STATUS => {
                match value {
                    DataObject::Enumerate(status) => {
                        self.set_alarm_status(AlarmStatus::from_u8(status)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for alarm_status".to_string(),
                    )),
                }
            }
            Self::ATTR_ALARM_VALUE => {
                match value {
                    DataObject::Integer64(value) => {
                        self.set_alarm_value(value).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 for alarm_value".to_string(),
                    )),
                }
            }
            Self::ATTR_ALARM_THRESHOLD => {
                match value {
                    DataObject::Integer64(threshold) => {
                        self.set_alarm_threshold(threshold).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 for alarm_threshold".to_string(),
                    )),
                }
            }
            Self::ATTR_ALARM_TIME => {
                match value {
                    DataObject::Integer64(timestamp) => {
                        self.set_alarm_time(Some(timestamp)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_alarm_time(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 or Null for alarm_time".to_string(),
                    )),
                }
            }
            Self::ATTR_ALARM_COUNT => {
                match value {
                    DataObject::Unsigned32(count) => {
                        *self.alarm_count.write().await = count;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned32 for alarm_count".to_string(),
                    )),
                }
            }
            Self::ATTR_ALARM_ENABLED => {
                match value {
                    DataObject::Boolean(enabled) => {
                        self.set_alarm_enabled(enabled).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Boolean for alarm_enabled".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Alarm has no attribute {}",
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
            "Alarm has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alarm_class_id() {
        let alarm = Alarm::with_default_obis();
        assert_eq!(alarm.class_id(), 73);
    }

    #[tokio::test]
    async fn test_alarm_obis_code() {
        let alarm = Alarm::with_default_obis();
        assert_eq!(alarm.obis_code(), Alarm::default_obis());
    }

    #[tokio::test]
    async fn test_alarm_type_from_u8() {
        assert_eq!(AlarmType::from_u8(0), AlarmType::None);
        assert_eq!(AlarmType::from_u8(1), AlarmType::PowerFailure);
        assert_eq!(AlarmType::from_u8(2), AlarmType::LowBattery);
        assert_eq!(AlarmType::from_u8(3), AlarmType::Tamper);
        assert_eq!(AlarmType::from_u8(4), AlarmType::Overload);
        assert_eq!(AlarmType::from_u8(5), AlarmType::Underload);
        assert_eq!(AlarmType::from_u8(6), AlarmType::PhaseLoss);
        assert_eq!(AlarmType::from_u8(7), AlarmType::Overvoltage);
        assert_eq!(AlarmType::from_u8(8), AlarmType::Undervoltage);
    }

    #[tokio::test]
    async fn test_alarm_type_is_critical() {
        assert!(!AlarmType::None.is_critical());
        assert!(AlarmType::PowerFailure.is_critical());
        assert!(!AlarmType::LowBattery.is_critical());
        assert!(AlarmType::Tamper.is_critical());
        assert!(AlarmType::PhaseLoss.is_critical());
    }

    #[tokio::test]
    async fn test_alarm_status_from_u8() {
        assert_eq!(AlarmStatus::from_u8(0), AlarmStatus::Inactive);
        assert_eq!(AlarmStatus::from_u8(1), AlarmStatus::Active);
        assert_eq!(AlarmStatus::from_u8(2), AlarmStatus::Acknowledged);
        assert_eq!(AlarmStatus::from_u8(3), AlarmStatus::Disabled);
    }

    #[tokio::test]
    async fn test_alarm_status_is_active() {
        assert!(AlarmStatus::Active.is_active());
        assert!(!AlarmStatus::Inactive.is_active());
        assert!(!AlarmStatus::Acknowledged.is_active());
    }

    #[tokio::test]
    async fn test_alarm_status_can_trigger() {
        assert!(AlarmStatus::Inactive.can_trigger());
        assert!(AlarmStatus::Active.can_trigger());
        assert!(AlarmStatus::Acknowledged.can_trigger());
        assert!(!AlarmStatus::Disabled.can_trigger());
    }

    #[tokio::test]
    async fn test_alarm_initial_state() {
        let alarm = Alarm::with_default_obis();
        assert_eq!(alarm.alarm_type().await, AlarmType::None);
        assert_eq!(alarm.alarm_status().await, AlarmStatus::Inactive);
        assert_eq!(alarm.alarm_value().await, 0);
        assert_eq!(alarm.alarm_count().await, 0);
        assert!(alarm.alarm_enabled().await);
    }

    #[tokio::test]
    async fn test_alarm_set_alarm_type() {
        let alarm = Alarm::with_default_obis();
        alarm.set_alarm_type(AlarmType::LowBattery).await;
        assert_eq!(alarm.alarm_type().await, AlarmType::LowBattery);
    }

    #[tokio::test]
    async fn test_alarm_set_alarm_value() {
        let alarm = Alarm::with_default_obis();
        alarm.set_alarm_value(100).await;
        assert_eq!(alarm.alarm_value().await, 100);
    }

    #[tokio::test]
    async fn test_alarm_set_alarm_threshold() {
        let alarm = Alarm::with_default_obis();
        alarm.set_alarm_threshold(50).await;
        assert_eq!(alarm.alarm_threshold().await, 50);
    }

    #[tokio::test]
    async fn test_alarm_trigger() {
        let alarm = Alarm::with_default_obis();
        alarm.set_alarm_threshold(50).await;
        alarm.set_alarm_value(100).await;

        alarm.trigger().await;

        assert_eq!(alarm.alarm_status().await, AlarmStatus::Active);
        assert_eq!(alarm.alarm_count().await, 1);
        assert!(alarm.alarm_time().await.is_some());
    }

    #[tokio::test]
    async fn test_alarm_trigger_when_disabled() {
        let alarm = Alarm::with_default_obis();
        alarm.set_alarm_enabled(false).await;

        alarm.trigger().await;

        assert_eq!(alarm.alarm_status().await, AlarmStatus::Inactive);
        assert_eq!(alarm.alarm_count().await, 0);
    }

    #[tokio::test]
    async fn test_alarm_acknowledge() {
        let alarm = Alarm::with_default_obis();
        alarm.trigger().await;

        alarm.acknowledge().await;

        assert_eq!(alarm.alarm_status().await, AlarmStatus::Acknowledged);
    }

    #[tokio::test]
    async fn test_alarm_clear() {
        let alarm = Alarm::with_default_obis();
        alarm.trigger().await;

        alarm.clear().await;

        assert_eq!(alarm.alarm_status().await, AlarmStatus::Inactive);
    }

    #[tokio::test]
    async fn test_alarm_enable_disable() {
        let alarm = Alarm::with_default_obis();

        alarm.disable().await;
        assert!(!alarm.alarm_enabled().await);

        alarm.enable().await;
        assert!(alarm.alarm_enabled().await);
    }

    #[tokio::test]
    async fn test_alarm_is_active() {
        let alarm = Alarm::with_default_obis();
        assert!(!alarm.is_active().await);

        alarm.trigger().await;
        assert!(alarm.is_active().await);
    }

    #[tokio::test]
    async fn test_alarm_is_critical() {
        let alarm = Alarm::with_default_obis();
        assert!(!alarm.is_critical().await);

        alarm.set_alarm_type(AlarmType::Tamper).await;
        assert!(alarm.is_critical().await);
    }

    #[tokio::test]
    async fn test_alarm_check_and_trigger() {
        let alarm = Alarm::with_default_obis();
        alarm.set_alarm_threshold(50).await;
        alarm.set_alarm_value(100).await;

        alarm.check_and_trigger().await;

        assert_eq!(alarm.alarm_status().await, AlarmStatus::Active);
    }

    #[tokio::test]
    async fn test_alarm_get_attributes() {
        let alarm = Alarm::with_default_obis();

        // Test alarm_type
        let result = alarm.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Enumerate(at) => assert_eq!(at, 0), // None
            _ => panic!("Expected Enumerate"),
        }

        // Test alarm_status
        let result = alarm.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Enumerate(st) => assert_eq!(st, 0), // Inactive
            _ => panic!("Expected Enumerate"),
        }
    }

    #[tokio::test]
    async fn test_alarm_set_attributes() {
        let alarm = Alarm::with_default_obis();

        alarm.set_attribute(2, DataObject::Enumerate(2), None) // LowBattery
            .await
            .unwrap();
        assert_eq!(alarm.alarm_type().await, AlarmType::LowBattery);

        alarm.set_attribute(4, DataObject::Integer64(75), None)
            .await
            .unwrap();
        assert_eq!(alarm.alarm_value().await, 75);
    }

    #[tokio::test]
    async fn test_alarm_read_only_logical_name() {
        let alarm = Alarm::with_default_obis();
        let result = alarm
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 73, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_alarm_invalid_attribute() {
        let alarm = Alarm::with_default_obis();
        let result = alarm.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_alarm_invalid_method() {
        let alarm = Alarm::with_default_obis();
        let result = alarm.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }
}
