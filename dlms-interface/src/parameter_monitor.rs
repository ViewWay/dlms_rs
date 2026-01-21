//! Parameter Monitor interface class (Class ID: 96)
//!
//! The Parameter Monitor interface class monitors parameter values and triggers actions based on thresholds.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: monitored_value - Reference to the value being monitored
//! - Attribute 3: threshold_upper - Upper threshold value
//! - Attribute 4: threshold_lower - Lower threshold value
//! - Attribute 5: action_threshold_upper - Action when upper threshold is exceeded
//! - Attribute 6: action_threshold_lower - Action when lower threshold is exceeded
//! - Attribute 7: enabled - Whether monitoring is enabled
//! - Attribute 8: last_trigger_time - Timestamp of last trigger

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Parameter Monitor interface class (Class ID: 96)
///
/// Default OBIS: 0-0:96.0.0.255
///
/// This class monitors parameter values and triggers actions based on thresholds.
#[derive(Debug, Clone)]
pub struct ParameterMonitor {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Reference to the value being monitored (as bytes)
    monitored_value: Arc<RwLock<Vec<u8>>>,

    /// Upper threshold value (as signed 64-bit)
    threshold_upper: Arc<RwLock<i64>>,

    /// Lower threshold value (as signed 64-bit)
    threshold_lower: Arc<RwLock<i64>>,

    /// Action when upper threshold is exceeded (script ID)
    action_threshold_upper: Arc<RwLock<u8>>,

    /// Action when lower threshold is exceeded (script ID)
    action_threshold_lower: Arc<RwLock<u8>>,

    /// Whether monitoring is enabled
    enabled: Arc<RwLock<bool>>,

    /// Timestamp of last trigger
    last_trigger_time: Arc<RwLock<Option<i64>>>,

    /// Current value
    current_value: Arc<RwLock<i64>>,

    /// Trigger count
    trigger_count: Arc<RwLock<u32>>,
}

impl ParameterMonitor {
    /// Class ID for ParameterMonitor
    pub const CLASS_ID: u16 = 96;

    /// Default OBIS code for ParameterMonitor (0-0:96.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 96, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_MONITORED_VALUE: u8 = 2;
    pub const ATTR_THRESHOLD_UPPER: u8 = 3;
    pub const ATTR_THRESHOLD_LOWER: u8 = 4;
    pub const ATTR_ACTION_THRESHOLD_UPPER: u8 = 5;
    pub const ATTR_ACTION_THRESHOLD_LOWER: u8 = 6;
    pub const ATTR_ENABLED: u8 = 7;
    pub const ATTR_LAST_TRIGGER_TIME: u8 = 8;

    /// Create a new ParameterMonitor object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            monitored_value: Arc::new(RwLock::new(Vec::new())),
            threshold_upper: Arc::new(RwLock::new(100)),
            threshold_lower: Arc::new(RwLock::new(0)),
            action_threshold_upper: Arc::new(RwLock::new(0)),
            action_threshold_lower: Arc::new(RwLock::new(0)),
            enabled: Arc::new(RwLock::new(false)),
            last_trigger_time: Arc::new(RwLock::new(None)),
            current_value: Arc::new(RwLock::new(0)),
            trigger_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Create with thresholds
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `lower` - Lower threshold
    /// * `upper` - Upper threshold
    pub fn with_thresholds(logical_name: ObisCode, lower: i64, upper: i64) -> Self {
        Self {
            logical_name,
            monitored_value: Arc::new(RwLock::new(Vec::new())),
            threshold_upper: Arc::new(RwLock::new(upper)),
            threshold_lower: Arc::new(RwLock::new(lower)),
            action_threshold_upper: Arc::new(RwLock::new(0)),
            action_threshold_lower: Arc::new(RwLock::new(0)),
            enabled: Arc::new(RwLock::new(false)),
            last_trigger_time: Arc::new(RwLock::new(None)),
            current_value: Arc::new(RwLock::new(0)),
            trigger_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Get the monitored value reference
    pub async fn monitored_value(&self) -> Vec<u8> {
        self.monitored_value.read().await.clone()
    }

    /// Set the monitored value reference
    pub async fn set_monitored_value(&self, value: Vec<u8>) {
        *self.monitored_value.write().await = value;
    }

    /// Get the upper threshold
    pub async fn threshold_upper(&self) -> i64 {
        *self.threshold_upper.read().await
    }

    /// Set the upper threshold
    pub async fn set_threshold_upper(&self, threshold: i64) {
        *self.threshold_upper.write().await = threshold;
    }

    /// Get the lower threshold
    pub async fn threshold_lower(&self) -> i64 {
        *self.threshold_lower.read().await
    }

    /// Set the lower threshold
    pub async fn set_threshold_lower(&self, threshold: i64) {
        *self.threshold_lower.write().await = threshold;
    }

    /// Set both thresholds
    pub async fn set_thresholds(&self, lower: i64, upper: i64) {
        self.set_threshold_lower(lower).await;
        self.set_threshold_upper(upper).await;
    }

    /// Get the upper threshold action (script ID)
    pub async fn action_threshold_upper(&self) -> u8 {
        *self.action_threshold_upper.read().await
    }

    /// Set the upper threshold action
    pub async fn set_action_threshold_upper(&self, action: u8) {
        *self.action_threshold_upper.write().await = action;
    }

    /// Get the lower threshold action (script ID)
    pub async fn action_threshold_lower(&self) -> u8 {
        *self.action_threshold_lower.read().await
    }

    /// Set the lower threshold action
    pub async fn set_action_threshold_lower(&self, action: u8) {
        *self.action_threshold_lower.write().await = action;
    }

    /// Get the enabled status
    pub async fn enabled(&self) -> bool {
        *self.enabled.read().await
    }

    /// Set the enabled status
    pub async fn set_enabled(&self, enabled: bool) {
        *self.enabled.write().await = enabled;
    }

    /// Enable monitoring
    pub async fn enable(&self) {
        self.set_enabled(true).await;
    }

    /// Disable monitoring
    pub async fn disable(&self) {
        self.set_enabled(false).await;
    }

    /// Get the last trigger time
    pub async fn last_trigger_time(&self) -> Option<i64> {
        *self.last_trigger_time.read().await
    }

    /// Get the current value
    pub async fn current_value(&self) -> i64 {
        *self.current_value.read().await
    }

    /// Set the current value (simulated update)
    pub async fn set_current_value(&self, value: i64) {
        *self.current_value.write().await = value;

        if self.enabled().await {
            self.check_thresholds(value).await;
        }
    }

    /// Get the trigger count
    pub async fn trigger_count(&self) -> u32 {
        *self.trigger_count.read().await
    }

    /// Reset the trigger count
    pub async fn reset_trigger_count(&self) {
        *self.trigger_count.write().await = 0;
    }

    /// Check if value exceeds thresholds
    pub async fn check_thresholds(&self, value: i64) {
        let upper = self.threshold_upper().await;
        let lower = self.threshold_lower().await;

        if value > upper {
            self.trigger_upper().await;
        } else if value < lower {
            self.trigger_lower().await;
        }
    }

    /// Trigger upper threshold action
    pub async fn trigger_upper(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        *self.last_trigger_time.write().await = Some(now);
        *self.trigger_count.write().await += 1;
    }

    /// Trigger lower threshold action
    pub async fn trigger_lower(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        *self.last_trigger_time.write().await = Some(now);
        *self.trigger_count.write().await += 1;
    }

    /// Check if value is within thresholds
    pub async fn is_within_thresholds(&self, value: i64) -> bool {
        let upper = self.threshold_upper().await;
        let lower = self.threshold_lower().await;
        value >= lower && value <= upper
    }

    /// Check if current value is within thresholds
    pub async fn is_current_within_thresholds(&self) -> bool {
        let value = self.current_value().await;
        self.is_within_thresholds(value).await
    }

    /// Get threshold range
    pub async fn threshold_range(&self) -> i64 {
        let upper = self.threshold_upper().await;
        let lower = self.threshold_lower().await;
        upper - lower
    }

    /// Check if enabled
    pub async fn is_enabled(&self) -> bool {
        self.enabled().await
    }

    /// Get time since last trigger (in seconds)
    pub async fn time_since_last_trigger(&self) -> Option<i64> {
        if let Some(last_time) = self.last_trigger_time().await {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            Some(now - last_time)
        } else {
            None
        }
    }
}

#[async_trait]
impl CosemObject for ParameterMonitor {
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
            Self::ATTR_MONITORED_VALUE => {
                Ok(DataObject::OctetString(self.monitored_value().await))
            }
            Self::ATTR_THRESHOLD_UPPER => {
                Ok(DataObject::Integer64(self.threshold_upper().await))
            }
            Self::ATTR_THRESHOLD_LOWER => {
                Ok(DataObject::Integer64(self.threshold_lower().await))
            }
            Self::ATTR_ACTION_THRESHOLD_UPPER => {
                Ok(DataObject::Unsigned8(self.action_threshold_upper().await))
            }
            Self::ATTR_ACTION_THRESHOLD_LOWER => {
                Ok(DataObject::Unsigned8(self.action_threshold_lower().await))
            }
            Self::ATTR_ENABLED => {
                Ok(DataObject::Boolean(self.enabled().await))
            }
            Self::ATTR_LAST_TRIGGER_TIME => {
                match self.last_trigger_time().await {
                    Some(timestamp) => Ok(DataObject::Integer32(timestamp as i32)),
                    None => Ok(DataObject::Null),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "ParameterMonitor has no attribute {}",
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
            Self::ATTR_MONITORED_VALUE => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_monitored_value(bytes).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for monitored_value".to_string(),
                    )),
                }
            }
            Self::ATTR_THRESHOLD_UPPER => {
                match value {
                    DataObject::Integer64(threshold) => {
                        self.set_threshold_upper(threshold).await;
                        Ok(())
                    }
                    DataObject::Integer32(threshold) => {
                        self.set_threshold_upper(threshold as i64).await;
                        Ok(())
                    }
                    DataObject::Unsigned32(threshold) => {
                        self.set_threshold_upper(threshold as i64).await;
                        Ok(())
                    }
                    DataObject::Unsigned16(threshold) => {
                        self.set_threshold_upper(threshold as i64).await;
                        Ok(())
                    }
                    DataObject::Unsigned8(threshold) => {
                        self.set_threshold_upper(threshold as i64).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64/Integer32/Unsigned for threshold_upper".to_string(),
                    )),
                }
            }
            Self::ATTR_THRESHOLD_LOWER => {
                match value {
                    DataObject::Integer64(threshold) => {
                        self.set_threshold_lower(threshold).await;
                        Ok(())
                    }
                    DataObject::Integer32(threshold) => {
                        self.set_threshold_lower(threshold as i64).await;
                        Ok(())
                    }
                    DataObject::Unsigned32(threshold) => {
                        self.set_threshold_lower(threshold as i64).await;
                        Ok(())
                    }
                    DataObject::Unsigned16(threshold) => {
                        self.set_threshold_lower(threshold as i64).await;
                        Ok(())
                    }
                    DataObject::Unsigned8(threshold) => {
                        self.set_threshold_lower(threshold as i64).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64/Integer32/Unsigned for threshold_lower".to_string(),
                    )),
                }
            }
            Self::ATTR_ACTION_THRESHOLD_UPPER => {
                match value {
                    DataObject::Unsigned8(action) => {
                        self.set_action_threshold_upper(action).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for action_threshold_upper".to_string(),
                    )),
                }
            }
            Self::ATTR_ACTION_THRESHOLD_LOWER => {
                match value {
                    DataObject::Unsigned8(action) => {
                        self.set_action_threshold_lower(action).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for action_threshold_lower".to_string(),
                    )),
                }
            }
            Self::ATTR_ENABLED => {
                match value {
                    DataObject::Boolean(enabled) => {
                        self.set_enabled(enabled).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Boolean for enabled".to_string(),
                    )),
                }
            }
            Self::ATTR_LAST_TRIGGER_TIME => {
                Err(DlmsError::AccessDenied(
                    "Attribute 8 (last_trigger_time) is read-only".to_string(),
                ))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "ParameterMonitor has no attribute {}",
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
            "ParameterMonitor has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parameter_monitor_class_id() {
        let pm = ParameterMonitor::with_default_obis();
        assert_eq!(pm.class_id(), 96);
    }

    #[tokio::test]
    async fn test_parameter_monitor_obis_code() {
        let pm = ParameterMonitor::with_default_obis();
        assert_eq!(pm.obis_code(), ParameterMonitor::default_obis());
    }

    #[tokio::test]
    async fn test_parameter_monitor_initial_state() {
        let pm = ParameterMonitor::with_default_obis();
        assert!(!pm.enabled().await);
        assert_eq!(pm.threshold_upper().await, 100);
        assert_eq!(pm.threshold_lower().await, 0);
        assert_eq!(pm.action_threshold_upper().await, 0);
        assert_eq!(pm.action_threshold_lower().await, 0);
        assert_eq!(pm.last_trigger_time().await, None);
        assert_eq!(pm.trigger_count().await, 0);
    }

    #[tokio::test]
    async fn test_parameter_monitor_with_thresholds() {
        let pm = ParameterMonitor::with_thresholds(ObisCode::new(0, 0, 96, 0, 0, 255), -50, 150);
        assert_eq!(pm.threshold_lower().await, -50);
        assert_eq!(pm.threshold_upper().await, 150);
    }

    #[tokio::test]
    async fn test_parameter_monitor_set_thresholds() {
        let pm = ParameterMonitor::with_default_obis();
        pm.set_thresholds(10, 90).await;
        assert_eq!(pm.threshold_lower().await, 10);
        assert_eq!(pm.threshold_upper().await, 90);
    }

    #[tokio::test]
    async fn test_parameter_monitor_enable_disable() {
        let pm = ParameterMonitor::with_default_obis();
        assert!(!pm.is_enabled().await);

        pm.enable().await;
        assert!(pm.is_enabled().await);

        pm.disable().await;
        assert!(!pm.is_enabled().await);
    }

    #[tokio::test]
    async fn test_parameter_monitor_set_actions() {
        let pm = ParameterMonitor::with_default_obis();
        pm.set_action_threshold_upper(10).await;
        pm.set_action_threshold_lower(20).await;
        assert_eq!(pm.action_threshold_upper().await, 10);
        assert_eq!(pm.action_threshold_lower().await, 20);
    }

    #[tokio::test]
    async fn test_parameter_monitor_set_current_value() {
        let pm = ParameterMonitor::with_default_obis();
        pm.set_current_value(50).await;
        assert_eq!(pm.current_value().await, 50);
    }

    #[tokio::test]
    async fn test_parameter_monitor_trigger_upper() {
        let pm = ParameterMonitor::with_default_obis();
        pm.enable().await;
        pm.set_current_value(150).await; // Exceeds upper threshold of 100
        assert!(pm.last_trigger_time().await.is_some());
        assert_eq!(pm.trigger_count().await, 1);
    }

    #[tokio::test]
    async fn test_parameter_monitor_trigger_lower() {
        let pm = ParameterMonitor::with_default_obis();
        pm.enable().await;
        pm.set_current_value(-10).await; // Below lower threshold of 0
        assert!(pm.last_trigger_time().await.is_some());
        assert_eq!(pm.trigger_count().await, 1);
    }

    #[tokio::test]
    async fn test_parameter_monitor_no_trigger_when_disabled() {
        let pm = ParameterMonitor::with_default_obis();
        // Don't enable
        pm.set_current_value(150).await;
        assert_eq!(pm.last_trigger_time().await, None);
        assert_eq!(pm.trigger_count().await, 0);
    }

    #[tokio::test]
    async fn test_parameter_monitor_is_within_thresholds() {
        let pm = ParameterMonitor::with_default_obis();
        assert!(pm.is_within_thresholds(50).await);
        assert!(!pm.is_within_thresholds(150).await);
        assert!(!pm.is_within_thresholds(-10).await);
    }

    #[tokio::test]
    async fn test_parameter_monitor_is_current_within_thresholds() {
        let pm = ParameterMonitor::with_default_obis();
        pm.set_current_value(50).await;
        assert!(pm.is_current_within_thresholds().await);

        pm.set_current_value(150).await;
        assert!(!pm.is_current_within_thresholds().await);
    }

    #[tokio::test]
    async fn test_parameter_monitor_threshold_range() {
        let pm = ParameterMonitor::with_thresholds(ObisCode::new(0, 0, 96, 0, 0, 255), 0, 100);
        assert_eq!(pm.threshold_range().await, 100);
    }

    #[tokio::test]
    async fn test_parameter_monitor_time_since_last_trigger() {
        let pm = ParameterMonitor::with_default_obis();
        assert_eq!(pm.time_since_last_trigger().await, None);

        pm.trigger_upper().await;
        let time = pm.time_since_last_trigger().await;
        assert!(time.is_some());
        assert!(time.unwrap() >= 0 && time.unwrap() < 10); // Should be very recent
    }

    #[tokio::test]
    async fn test_parameter_monitor_reset_trigger_count() {
        let pm = ParameterMonitor::with_default_obis();
        pm.enable().await;
        pm.set_current_value(150).await;
        assert_eq!(pm.trigger_count().await, 1);

        pm.reset_trigger_count().await;
        assert_eq!(pm.trigger_count().await, 0);
    }

    #[tokio::test]
    async fn test_parameter_monitor_multiple_triggers() {
        let pm = ParameterMonitor::with_default_obis();
        pm.enable().await;
        pm.set_current_value(150).await;
        pm.set_current_value(-10).await;
        assert_eq!(pm.trigger_count().await, 2);
    }

    #[tokio::test]
    async fn test_parameter_monitor_get_attributes() {
        let pm = ParameterMonitor::with_default_obis();

        // Test threshold_upper
        let result = pm.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Integer64(threshold) => assert_eq!(threshold, 100),
            _ => panic!("Expected Integer64"),
        }

        // Test threshold_lower
        let result = pm.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Integer64(threshold) => assert_eq!(threshold, 0),
            _ => panic!("Expected Integer64"),
        }

        // Test enabled
        let result = pm.get_attribute(7, None).await.unwrap();
        match result {
            DataObject::Boolean(enabled) => assert!(!enabled),
            _ => panic!("Expected Boolean"),
        }
    }

    #[tokio::test]
    async fn test_parameter_monitor_set_attributes() {
        let pm = ParameterMonitor::with_default_obis();

        pm.set_attribute(3, DataObject::Integer64(200), None)
            .await
            .unwrap();
        assert_eq!(pm.threshold_upper().await, 200);

        pm.set_attribute(4, DataObject::Integer64(-50), None)
            .await
            .unwrap();
        assert_eq!(pm.threshold_lower().await, -50);

        pm.set_attribute(7, DataObject::Boolean(true), None)
            .await
            .unwrap();
        assert!(pm.enabled().await);
    }

    #[tokio::test]
    async fn test_parameter_monitor_set_threshold_u32() {
        let pm = ParameterMonitor::with_default_obis();
        pm.set_attribute(3, DataObject::Unsigned32(300), None)
            .await
            .unwrap();
        assert_eq!(pm.threshold_upper().await, 300);
    }

    #[tokio::test]
    async fn test_parameter_monitor_set_threshold_u16() {
        let pm = ParameterMonitor::with_default_obis();
        pm.set_attribute(3, DataObject::Unsigned16(400), None)
            .await
            .unwrap();
        assert_eq!(pm.threshold_upper().await, 400);
    }

    #[tokio::test]
    async fn test_parameter_monitor_set_threshold_u8() {
        let pm = ParameterMonitor::with_default_obis();
        pm.set_attribute(3, DataObject::Unsigned8(50), None)
            .await
            .unwrap();
        assert_eq!(pm.threshold_upper().await, 50);
    }

    #[tokio::test]
    async fn test_parameter_monitor_set_threshold_i32() {
        let pm = ParameterMonitor::with_default_obis();
        pm.set_attribute(3, DataObject::Integer32(250), None)
            .await
            .unwrap();
        assert_eq!(pm.threshold_upper().await, 250);
    }

    #[tokio::test]
    async fn test_parameter_monitor_read_only_logical_name() {
        let pm = ParameterMonitor::with_default_obis();
        let result = pm
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 96, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parameter_monitor_read_only_last_trigger_time() {
        let pm = ParameterMonitor::with_default_obis();
        let result = pm
            .set_attribute(8, DataObject::Integer32(1234567890), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parameter_monitor_invalid_attribute() {
        let pm = ParameterMonitor::with_default_obis();
        let result = pm.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parameter_monitor_invalid_method() {
        let pm = ParameterMonitor::with_default_obis();
        let result = pm.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parameter_monitor_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 96, 0, 0, 1);
        let pm = ParameterMonitor::new(obis);
        assert_eq!(pm.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_parameter_monitor_set_monitored_value() {
        let pm = ParameterMonitor::with_default_obis();
        pm.set_monitored_value(vec![1, 2, 3, 4]).await;
        assert_eq!(pm.monitored_value().await, vec![1, 2, 3, 4]);
    }
}
