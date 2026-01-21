//! Action Schedule interface class (Class ID: 95)
//!
//! The Action Schedule interface class manages scheduled action executions.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: enabled - Whether the schedule is enabled
//! - Attribute 3: execution_time - Scheduled execution time
//! - Attribute 4: action_script - Script to execute
//! - Attribute 5: last_execution_time - Timestamp of last execution
//! - Attribute 6: execution_count - Number of times executed

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Action Schedule interface class (Class ID: 95)
///
/// Default OBIS: 0-0:95.0.0.255
///
/// This class manages scheduled action executions.
#[derive(Debug, Clone)]
pub struct ActionSchedule {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Whether the schedule is enabled
    enabled: Arc<RwLock<bool>>,

    /// Scheduled execution time (Unix timestamp)
    execution_time: Arc<RwLock<Option<i64>>>,

    /// Script to execute
    action_script: Arc<RwLock<u8>>,

    /// Timestamp of last execution
    last_execution_time: Arc<RwLock<Option<i64>>>,

    /// Number of times executed
    execution_count: Arc<RwLock<u32>>,
}

impl ActionSchedule {
    /// Class ID for ActionSchedule
    pub const CLASS_ID: u16 = 95;

    /// Default OBIS code for ActionSchedule (0-0:95.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 95, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_ENABLED: u8 = 2;
    pub const ATTR_EXECUTION_TIME: u8 = 3;
    pub const ATTR_ACTION_SCRIPT: u8 = 4;
    pub const ATTR_LAST_EXECUTION_TIME: u8 = 5;
    pub const ATTR_EXECUTION_COUNT: u8 = 6;

    /// Create a new ActionSchedule object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            enabled: Arc::new(RwLock::new(false)),
            execution_time: Arc::new(RwLock::new(None)),
            action_script: Arc::new(RwLock::new(0)),
            last_execution_time: Arc::new(RwLock::new(None)),
            execution_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Create with specific script
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `script_id` - Script ID to execute
    pub fn with_script(logical_name: ObisCode, script_id: u8) -> Self {
        Self {
            logical_name,
            enabled: Arc::new(RwLock::new(false)),
            execution_time: Arc::new(RwLock::new(None)),
            action_script: Arc::new(RwLock::new(script_id)),
            last_execution_time: Arc::new(RwLock::new(None)),
            execution_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Get the enabled status
    pub async fn enabled(&self) -> bool {
        *self.enabled.read().await
    }

    /// Set the enabled status
    pub async fn set_enabled(&self, enabled: bool) {
        *self.enabled.write().await = enabled;
    }

    /// Enable the schedule
    pub async fn enable(&self) {
        self.set_enabled(true).await;
    }

    /// Disable the schedule
    pub async fn disable(&self) {
        self.set_enabled(false).await;
    }

    /// Get the execution time
    pub async fn execution_time(&self) -> Option<i64> {
        *self.execution_time.read().await
    }

    /// Set the execution time
    pub async fn set_execution_time(&self, time: Option<i64>) {
        *self.execution_time.write().await = time;
    }

    /// Schedule execution at a specific timestamp
    pub async fn schedule_at(&self, timestamp: i64) {
        self.set_execution_time(Some(timestamp)).await;
        self.enable().await;
    }

    /// Get the action script ID
    pub async fn action_script(&self) -> u8 {
        *self.action_script.read().await
    }

    /// Set the action script ID
    pub async fn set_action_script(&self, script_id: u8) {
        *self.action_script.write().await = script_id;
    }

    /// Get the last execution time
    pub async fn last_execution_time(&self) -> Option<i64> {
        *self.last_execution_time.read().await
    }

    /// Get the execution count
    pub async fn execution_count(&self) -> u32 {
        *self.execution_count.read().await
    }

    /// Reset the execution count
    pub async fn reset_execution_count(&self) {
        *self.execution_count.write().await = 0;
    }

    /// Check if the schedule is enabled
    pub async fn is_enabled(&self) -> bool {
        self.enabled().await
    }

    /// Check if the schedule is due (execution time has passed)
    pub async fn is_due(&self) -> bool {
        if !self.enabled().await {
            return false;
        }
        if let Some(exec_time) = self.execution_time().await {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            exec_time <= now
        } else {
            false
        }
    }

    /// Check if the schedule is pending (has a future execution time)
    pub async fn is_pending(&self) -> bool {
        if !self.enabled().await {
            return false;
        }
        if let Some(exec_time) = self.execution_time().await {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            exec_time > now
        } else {
            false
        }
    }

    /// Simulate executing the scheduled action
    pub async fn execute(&self) -> DlmsResult<()> {
        if !self.enabled().await {
            return Err(DlmsError::InvalidData(
                "Schedule is not enabled".to_string(),
            ));
        }

        if !self.is_due().await {
            return Err(DlmsError::InvalidData(
                "Schedule is not due yet".to_string(),
            ));
        }

        // Record execution
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        *self.last_execution_time.write().await = Some(now);
        *self.execution_count.write().await += 1;

        // Clear execution time for one-time schedule
        *self.execution_time.write().await = None;
        *self.enabled.write().await = false;

        Ok(())
    }

    /// Schedule execution after a delay (in seconds)
    pub async fn schedule_after(&self, delay_seconds: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.schedule_at(now + delay_seconds as i64).await;
    }

    /// Get time until execution (in seconds)
    pub async fn time_until_execution(&self) -> Option<i64> {
        if let Some(exec_time) = self.execution_time().await {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            Some(exec_time - now)
        } else {
            None
        }
    }

    /// Check if the action has been executed at least once
    pub async fn has_executed(&self) -> bool {
        self.execution_count().await > 0
    }
}

#[async_trait]
impl CosemObject for ActionSchedule {
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
            Self::ATTR_ENABLED => {
                Ok(DataObject::Boolean(self.enabled().await))
            }
            Self::ATTR_EXECUTION_TIME => {
                match self.execution_time().await {
                    Some(timestamp) => Ok(DataObject::Integer32(timestamp as i32)),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_ACTION_SCRIPT => {
                Ok(DataObject::Unsigned8(self.action_script().await))
            }
            Self::ATTR_LAST_EXECUTION_TIME => {
                match self.last_execution_time().await {
                    Some(timestamp) => Ok(DataObject::Integer32(timestamp as i32)),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_EXECUTION_COUNT => {
                Ok(DataObject::Unsigned32(self.execution_count().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "ActionSchedule has no attribute {}",
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
            Self::ATTR_EXECUTION_TIME => {
                match value {
                    DataObject::Integer32(timestamp) => {
                        self.set_execution_time(Some(timestamp as i64)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_execution_time(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer32 or Null for execution_time".to_string(),
                    )),
                }
            }
            Self::ATTR_ACTION_SCRIPT => {
                match value {
                    DataObject::Unsigned8(script_id) => {
                        self.set_action_script(script_id).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for action_script".to_string(),
                    )),
                }
            }
            Self::ATTR_LAST_EXECUTION_TIME => {
                Err(DlmsError::AccessDenied(
                    "Attribute 5 (last_execution_time) is read-only".to_string(),
                ))
            }
            Self::ATTR_EXECUTION_COUNT => {
                Err(DlmsError::AccessDenied(
                    "Attribute 6 (execution_count) is read-only".to_string(),
                ))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "ActionSchedule has no attribute {}",
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
            "ActionSchedule has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_action_schedule_class_id() {
        let schedule = ActionSchedule::with_default_obis();
        assert_eq!(schedule.class_id(), 95);
    }

    #[tokio::test]
    async fn test_action_schedule_obis_code() {
        let schedule = ActionSchedule::with_default_obis();
        assert_eq!(schedule.obis_code(), ActionSchedule::default_obis());
    }

    #[tokio::test]
    async fn test_action_schedule_initial_state() {
        let schedule = ActionSchedule::with_default_obis();
        assert!(!schedule.enabled().await);
        assert_eq!(schedule.execution_time().await, None);
        assert_eq!(schedule.action_script().await, 0);
        assert_eq!(schedule.last_execution_time().await, None);
        assert_eq!(schedule.execution_count().await, 0);
    }

    #[tokio::test]
    async fn test_action_schedule_with_script() {
        let schedule = ActionSchedule::with_script(ObisCode::new(0, 0, 95, 0, 0, 255), 42);
        assert_eq!(schedule.action_script().await, 42);
    }

    #[tokio::test]
    async fn test_action_schedule_enable_disable() {
        let schedule = ActionSchedule::with_default_obis();
        assert!(!schedule.is_enabled().await);

        schedule.enable().await;
        assert!(schedule.is_enabled().await);

        schedule.disable().await;
        assert!(!schedule.is_enabled().await);
    }

    #[tokio::test]
    async fn test_action_schedule_set_execution_time() {
        let schedule = ActionSchedule::with_default_obis();
        schedule.set_execution_time(Some(1609459200)).await;
        assert_eq!(schedule.execution_time().await, Some(1609459200));
    }

    #[tokio::test]
    async fn test_action_schedule_set_action_script() {
        let schedule = ActionSchedule::with_default_obis();
        schedule.set_action_script(10).await;
        assert_eq!(schedule.action_script().await, 10);
    }

    #[tokio::test]
    async fn test_action_schedule_schedule_at() {
        let schedule = ActionSchedule::with_default_obis();
        schedule.schedule_at(1609459200).await;
        assert!(schedule.enabled().await);
        assert_eq!(schedule.execution_time().await, Some(1609459200));
    }

    #[tokio::test]
    async fn test_action_schedule_is_due() {
        let schedule = ActionSchedule::with_default_obis();
        let past_time = 1000000000; // Year 2001

        schedule.set_execution_time(Some(past_time)).await;
        schedule.enable().await;
        assert!(schedule.is_due().await);
    }

    #[tokio::test]
    async fn test_action_schedule_is_not_due_when_disabled() {
        let schedule = ActionSchedule::with_default_obis();
        let past_time = 1000000000;

        schedule.set_execution_time(Some(past_time)).await;
        // Don't enable
        assert!(!schedule.is_due().await);
    }

    #[tokio::test]
    async fn test_action_schedule_is_pending() {
        let schedule = ActionSchedule::with_default_obis();
        let future_time = 4000000000; // Year 2097

        schedule.set_execution_time(Some(future_time)).await;
        schedule.enable().await;
        assert!(schedule.is_pending().await);
    }

    #[tokio::test]
    async fn test_action_schedule_is_not_pending_when_past() {
        let schedule = ActionSchedule::with_default_obis();
        let past_time = 1000000000;

        schedule.set_execution_time(Some(past_time)).await;
        schedule.enable().await;
        assert!(!schedule.is_pending().await);
    }

    #[tokio::test]
    async fn test_action_schedule_execute() {
        let schedule = ActionSchedule::with_default_obis();
        let past_time = 1000000000;

        schedule.set_execution_time(Some(past_time)).await;
        schedule.enable().await;

        schedule.execute().await.unwrap();

        assert!(!schedule.enabled().await);
        assert_eq!(schedule.execution_time().await, None);
        assert!(schedule.last_execution_time().await.is_some());
        assert_eq!(schedule.execution_count().await, 1);
    }

    #[tokio::test]
    async fn test_action_schedule_execute_when_not_enabled() {
        let schedule = ActionSchedule::with_default_obis();
        let past_time = 1000000000;

        schedule.set_execution_time(Some(past_time)).await;
        // Don't enable

        let result = schedule.execute().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_action_schedule_execute_when_not_due() {
        let schedule = ActionSchedule::with_default_obis();
        let future_time = 4000000000;

        schedule.set_execution_time(Some(future_time)).await;
        schedule.enable().await;

        let result = schedule.execute().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_action_schedule_schedule_after() {
        let schedule = ActionSchedule::with_default_obis();
        schedule.schedule_after(3600).await; // 1 hour from now

        assert!(schedule.enabled().await);
        assert!(schedule.is_pending().await);

        let time_until = schedule.time_until_execution().await;
        assert!(time_until.is_some());
        assert!(time_until.unwrap() > 0 && time_until.unwrap() <= 3601);
    }

    #[tokio::test]
    async fn test_action_schedule_time_until_execution() {
        let schedule = ActionSchedule::with_default_obis();
        schedule.set_execution_time(Some(1609459200)).await;

        let time_until = schedule.time_until_execution().await;
        assert!(time_until.is_some());
    }

    #[tokio::test]
    async fn test_action_schedule_time_until_execution_none() {
        let schedule = ActionSchedule::with_default_obis();
        assert_eq!(schedule.time_until_execution().await, None);
    }

    #[tokio::test]
    async fn test_action_schedule_reset_execution_count() {
        let schedule = ActionSchedule::with_default_obis();
        let past_time = 1000000000;

        schedule.set_execution_time(Some(past_time)).await;
        schedule.enable().await;
        schedule.execute().await.unwrap();

        assert_eq!(schedule.execution_count().await, 1);

        schedule.reset_execution_count().await;
        assert_eq!(schedule.execution_count().await, 0);
    }

    #[tokio::test]
    async fn test_action_schedule_has_executed() {
        let schedule = ActionSchedule::with_default_obis();
        assert!(!schedule.has_executed().await);

        let past_time = 1000000000;
        schedule.set_execution_time(Some(past_time)).await;
        schedule.enable().await;
        schedule.execute().await.unwrap();

        assert!(schedule.has_executed().await);
    }

    #[tokio::test]
    async fn test_action_schedule_get_attributes() {
        let schedule = ActionSchedule::with_default_obis();

        // Test enabled
        let result = schedule.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Boolean(enabled) => assert!(!enabled),
            _ => panic!("Expected Boolean"),
        }

        // Test action_script
        let result = schedule.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Unsigned8(script) => assert_eq!(script, 0),
            _ => panic!("Expected Unsigned8"),
        }
    }

    #[tokio::test]
    async fn test_action_schedule_set_attributes() {
        let schedule = ActionSchedule::with_default_obis();

        schedule.set_attribute(2, DataObject::Boolean(true), None)
            .await
            .unwrap();
        assert!(schedule.enabled().await);

        schedule.set_attribute(4, DataObject::Unsigned8(123), None)
            .await
            .unwrap();
        assert_eq!(schedule.action_script().await, 123);
    }

    #[tokio::test]
    async fn test_action_schedule_set_execution_time_attribute() {
        let schedule = ActionSchedule::with_default_obis();
        schedule.set_attribute(3, DataObject::Integer32(1234567890), None)
            .await
            .unwrap();
        assert_eq!(schedule.execution_time().await, Some(1234567890));
    }

    #[tokio::test]
    async fn test_action_schedule_set_execution_time_null() {
        let schedule = ActionSchedule::with_default_obis();
        schedule.set_execution_time(Some(1234567890)).await;
        schedule.set_attribute(3, DataObject::Null, None)
            .await
            .unwrap();
        assert_eq!(schedule.execution_time().await, None);
    }

    #[tokio::test]
    async fn test_action_schedule_read_only_logical_name() {
        let schedule = ActionSchedule::with_default_obis();
        let result = schedule
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 95, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_action_schedule_read_only_last_execution_time() {
        let schedule = ActionSchedule::with_default_obis();
        let result = schedule
            .set_attribute(5, DataObject::Integer32(1234567890), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_action_schedule_read_only_execution_count() {
        let schedule = ActionSchedule::with_default_obis();
        let result = schedule
            .set_attribute(6, DataObject::Unsigned32(100), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_action_schedule_invalid_attribute() {
        let schedule = ActionSchedule::with_default_obis();
        let result = schedule.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_action_schedule_invalid_method() {
        let schedule = ActionSchedule::with_default_obis();
        let result = schedule.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_action_schedule_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 95, 0, 0, 1);
        let schedule = ActionSchedule::new(obis);
        assert_eq!(schedule.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_action_schedule_multiple_executions() {
        let schedule = ActionSchedule::with_default_obis();
        let past_time = 1000000000;

        // First execution
        schedule.set_execution_time(Some(past_time)).await;
        schedule.enable().await;
        schedule.execute().await.unwrap();

        // Schedule again
        schedule.schedule_at(past_time + 100).await;
        schedule.execute().await.unwrap();

        assert_eq!(schedule.execution_count().await, 2);
    }
}
