//! Single Action Schedule interface class (Class ID: 22)
//!
//! The Single Action Schedule interface class defines a one-time scheduled action
//! with an execution time. Unlike Schedule (Class ID: 10) which is for recurring
//! scripts, this is for single events that execute once.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: execution_time - When to execute the action
//! - Attribute 3: action_script - Script ID to execute
//! - Attribute 4: enabled - Whether the schedule is enabled
//! - Attribute 5: type - Type indicator (read-only)
//!
//! # Methods
//!
//! - Method 1: execute() - Execute the scheduled action immediately

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{
    datatypes::{CosemDateTime, CosemDateFormat},
    DlmsError, DlmsResult, ObisCode, DataObject,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Action Schedule Type
///
/// Defines the type/state of the single action schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ActionScheduleType {
    /// Single execution - action is pending execution
    SingleExecution = 0,
    /// Executed - action has been executed
    Executed = 1,
}

impl ActionScheduleType {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::SingleExecution,
            1 => Self::Executed,
            _ => Self::SingleExecution,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if this schedule is pending execution
    pub fn is_pending(self) -> bool {
        matches!(self, Self::SingleExecution)
    }

    /// Check if this schedule has been executed
    pub fn is_executed(self) -> bool {
        matches!(self, Self::Executed)
    }
}

/// Single Action Schedule interface class (Class ID: 22)
///
/// Default OBIS: 0-0:15.0.0.255
///
/// This class manages one-time scheduled actions. Once executed,
/// the type changes from SingleExecution to Executed.
#[derive(Debug, Clone)]
pub struct SingleActionSchedule {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Execution time for the scheduled action
    execution_time: Arc<RwLock<CosemDateTime>>,

    /// Script ID to execute
    action_script: Arc<RwLock<u8>>,

    /// Whether the schedule is enabled
    enabled: Arc<RwLock<bool>>,

    /// Type/state of the schedule
    type_: Arc<RwLock<ActionScheduleType>>,
}

impl SingleActionSchedule {
    /// Class ID for Single Action Schedule
    pub const CLASS_ID: u16 = 22;

    /// Default OBIS code for Single Action Schedule (0-0:15.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 15, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_EXECUTION_TIME: u8 = 2;
    pub const ATTR_ACTION_SCRIPT: u8 = 3;
    pub const ATTR_ENABLED: u8 = 4;
    pub const ATTR_TYPE: u8 = 5;

    /// Method IDs
    pub const METHOD_EXECUTE: u8 = 1;

    /// Create a new Single Action Schedule object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `execution_time` - When to execute the action
    /// * `action_script` - Script ID to execute
    /// * `enabled` - Whether the schedule is enabled
    pub fn new(
        logical_name: ObisCode,
        execution_time: CosemDateTime,
        action_script: u8,
        enabled: bool,
    ) -> Self {
        Self {
            logical_name,
            execution_time: Arc::new(RwLock::new(execution_time)),
            action_script: Arc::new(RwLock::new(action_script)),
            enabled: Arc::new(RwLock::new(enabled)),
            type_: Arc::new(RwLock::new(ActionScheduleType::SingleExecution)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        let time = CosemDateTime::new(2024, 1, 1, 0, 0, 0, 0, &[]).unwrap();
        Self::new(Self::default_obis(), time, 0, false)
    }

    /// Get the execution time
    pub async fn execution_time(&self) -> CosemDateTime {
        self.execution_time.read().await.clone()
    }

    /// Set the execution time
    pub async fn set_execution_time(&self, time: CosemDateTime) {
        *self.execution_time.write().await = time;
    }

    /// Get the action script ID
    pub async fn action_script(&self) -> u8 {
        *self.action_script.read().await
    }

    /// Set the action script ID
    pub async fn set_action_script(&self, script_id: u8) {
        *self.action_script.write().await = script_id;
    }

    /// Check if the schedule is enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }

    /// Set the enabled state
    pub async fn set_enabled(&self, enabled: bool) {
        *self.enabled.write().await = enabled;
    }

    /// Get the schedule type
    pub async fn type_(&self) -> ActionScheduleType {
        *self.type_.read().await
    }

    /// Set the schedule type
    pub async fn set_type(&self, type_: ActionScheduleType) {
        *self.type_.write().await = type_;
    }

    /// Check if the action is pending (not yet executed)
    pub async fn is_pending(&self) -> bool {
        self.type_().await.is_pending()
    }

    /// Check if the action has been executed
    pub async fn is_executed(&self) -> bool {
        self.type_().await.is_executed()
    }

    /// Execute the scheduled action
    ///
    /// This corresponds to Method 1. In a real implementation,
    /// this would execute the script specified by action_script.
    pub async fn execute(&self) -> DlmsResult<ExecutionResult> {
        if !self.is_enabled().await {
            return Err(DlmsError::InvalidData(
                "Cannot execute disabled schedule".to_string(),
            ));
        }

        let script_id = self.action_script().await;
        let was_executed = self.is_executed().await;

        if was_executed {
            return Err(DlmsError::InvalidData(
                "Action has already been executed".to_string(),
            ));
        }

        // Mark as executed
        self.set_type(ActionScheduleType::Executed).await;

        // In a real implementation, we would execute the script here
        Ok(ExecutionResult {
            script_id,
            success: true,
        })
    }

    /// Reset the schedule (allow re-execution)
    pub async fn reset(&self) {
        self.set_type(ActionScheduleType::SingleExecution).await;
    }
}

/// Result of action execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// ID of the executed script
    pub script_id: u8,
    /// Whether execution was successful
    pub success: bool,
}

#[async_trait]
impl CosemObject for SingleActionSchedule {
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
            Self::ATTR_EXECUTION_TIME => {
                let time = self.execution_time().await;
                Ok(DataObject::OctetString(time.encode()))
            }
            Self::ATTR_ACTION_SCRIPT => {
                let script = self.action_script().await;
                Ok(DataObject::Unsigned8(script))
            }
            Self::ATTR_ENABLED => {
                let enabled = self.is_enabled().await;
                Ok(DataObject::Boolean(enabled))
            }
            Self::ATTR_TYPE => {
                let type_ = self.type_().await;
                Ok(DataObject::Enumerate(type_.to_u8()))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Single Action Schedule has no attribute {}",
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
            Self::ATTR_EXECUTION_TIME => {
                match value {
                    DataObject::OctetString(bytes) if bytes.len() >= 12 => {
                        let time = CosemDateTime::decode(&bytes)?;
                        self.set_execution_time(time).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for execution_time".to_string(),
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
            Self::ATTR_TYPE => {
                Err(DlmsError::AccessDenied(
                    "Attribute 5 (type) is read-only".to_string(),
                ))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Single Action Schedule has no attribute {}",
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
            Self::METHOD_EXECUTE => {
                let result = self.execute().await?;
                Ok(Some(DataObject::Unsigned8(result.script_id)))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Single Action Schedule has no method {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_single_action_schedule_class_id() {
        let schedule = SingleActionSchedule::with_default_obis();
        assert_eq!(schedule.class_id(), 22);
    }

    #[tokio::test]
    async fn test_single_action_schedule_obis_code() {
        let schedule = SingleActionSchedule::with_default_obis();
        assert_eq!(schedule.obis_code(), SingleActionSchedule::default_obis());
    }

    #[tokio::test]
    async fn test_action_schedule_type_from_u8() {
        assert_eq!(
            ActionScheduleType::from_u8(0),
            ActionScheduleType::SingleExecution
        );
        assert_eq!(ActionScheduleType::from_u8(1), ActionScheduleType::Executed);
        assert_eq!(
            ActionScheduleType::from_u8(99),
            ActionScheduleType::SingleExecution
        );
    }

    #[tokio::test]
    async fn test_action_schedule_type_to_u8() {
        assert_eq!(ActionScheduleType::SingleExecution.to_u8(), 0);
        assert_eq!(ActionScheduleType::Executed.to_u8(), 1);
    }

    #[tokio::test]
    async fn test_action_schedule_type_is_pending() {
        assert!(ActionScheduleType::SingleExecution.is_pending());
        assert!(!ActionScheduleType::Executed.is_pending());
    }

    #[tokio::test]
    async fn test_action_schedule_type_is_executed() {
        assert!(!ActionScheduleType::SingleExecution.is_executed());
        assert!(ActionScheduleType::Executed.is_executed());
    }

    #[tokio::test]
    async fn test_single_action_schedule_default() {
        let schedule = SingleActionSchedule::with_default_obis();
        assert!(!schedule.is_enabled().await);
        assert_eq!(schedule.action_script().await, 0);
        assert!(schedule.is_pending().await);
    }

    #[tokio::test]
    async fn test_single_action_schedule_new() {
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let obis = ObisCode::new(1, 1, 1, 1, 1, 1);
        let schedule = SingleActionSchedule::new(obis.clone(), time.clone(), 5, true);

        assert_eq!(schedule.obis_code(), obis);
        assert_eq!(schedule.execution_time().await, time);
        assert_eq!(schedule.action_script().await, 5);
        assert!(schedule.is_enabled().await);
    }

    #[tokio::test]
    async fn test_single_action_schedule_set_execution_time() {
        let schedule = SingleActionSchedule::with_default_obis();
        let new_time = CosemDateTime::new(2024, 12, 25, 0, 0, 0, 0, &[]).unwrap();

        schedule.set_execution_time(new_time.clone()).await;
        assert_eq!(schedule.execution_time().await, new_time);
    }

    #[tokio::test]
    async fn test_single_action_schedule_set_action_script() {
        let schedule = SingleActionSchedule::with_default_obis();

        schedule.set_action_script(10).await;
        assert_eq!(schedule.action_script().await, 10);
    }

    #[tokio::test]
    async fn test_single_action_schedule_set_enabled() {
        let schedule = SingleActionSchedule::with_default_obis();

        schedule.set_enabled(true).await;
        assert!(schedule.is_enabled().await);

        schedule.set_enabled(false).await;
        assert!(!schedule.is_enabled().await);
    }

    #[tokio::test]
    async fn test_single_action_schedule_set_type() {
        let schedule = SingleActionSchedule::with_default_obis();

        schedule.set_type(ActionScheduleType::Executed).await;
        assert!(schedule.is_executed().await);
    }

    #[tokio::test]
    async fn test_single_action_schedule_execute() {
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let schedule = SingleActionSchedule::new(
            SingleActionSchedule::default_obis(),
            time,
            5,
            true,
        );

        let result = schedule.execute().await.unwrap();
        assert_eq!(result.script_id, 5);
        assert!(result.success);
        assert!(schedule.is_executed().await);
    }

    #[tokio::test]
    async fn test_single_action_schedule_execute_disabled() {
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let schedule = SingleActionSchedule::new(
            SingleActionSchedule::default_obis(),
            time,
            5,
            false, // disabled
        );

        let result = schedule.execute().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_single_action_schedule_execute_already_executed() {
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let schedule = SingleActionSchedule::new(
            SingleActionSchedule::default_obis(),
            time,
            5,
            true,
        );

        // First execution succeeds
        schedule.execute().await.unwrap();

        // Second execution should fail
        let result = schedule.execute().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_single_action_schedule_reset() {
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let schedule = SingleActionSchedule::new(
            SingleActionSchedule::default_obis(),
            time,
            5,
            true,
        );

        schedule.execute().await.unwrap();
        assert!(schedule.is_executed().await);

        schedule.reset().await;
        assert!(schedule.is_pending().await);

        // Can execute again after reset
        schedule.execute().await.unwrap();
    }

    #[tokio::test]
    async fn test_single_action_schedule_get_logical_name() {
        let schedule = SingleActionSchedule::with_default_obis();
        let result = schedule.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_single_action_schedule_get_execution_time() {
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let schedule = SingleActionSchedule::new(
            SingleActionSchedule::default_obis(),
            time,
            5,
            true,
        );

        let result = schedule.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 12);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_single_action_schedule_get_action_script() {
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let schedule = SingleActionSchedule::new(
            SingleActionSchedule::default_obis(),
            time,
            7,
            true,
        );

        let result = schedule.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Unsigned8(script) => {
                assert_eq!(script, 7);
            }
            _ => panic!("Expected Unsigned8"),
        }
    }

    #[tokio::test]
    async fn test_single_action_schedule_get_enabled() {
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let schedule = SingleActionSchedule::new(
            SingleActionSchedule::default_obis(),
            time,
            5,
            true,
        );

        let result = schedule.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Boolean(enabled) => {
                assert!(enabled);
            }
            _ => panic!("Expected Boolean"),
        }
    }

    #[tokio::test]
    async fn test_single_action_schedule_get_type() {
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let schedule = SingleActionSchedule::new(
            SingleActionSchedule::default_obis(),
            time,
            5,
            true,
        );

        let result = schedule.get_attribute(5, None).await.unwrap();
        match result {
            DataObject::Enumerate(type_) => {
                assert_eq!(type_, 0); // SingleExecution
            }
            _ => panic!("Expected Enumerate"),
        }

        // After execution, type should be Executed (1)
        schedule.execute().await.unwrap();
        let result = schedule.get_attribute(5, None).await.unwrap();
        match result {
            DataObject::Enumerate(type_) => {
                assert_eq!(type_, 1); // Executed
            }
            _ => panic!("Expected Enumerate"),
        }
    }

    #[tokio::test]
    async fn test_single_action_schedule_set_execution_time_via_attribute() {
        let schedule = SingleActionSchedule::with_default_obis();
        let new_time = CosemDateTime::new(2024, 12, 25, 0, 0, 0, 0, &[]).unwrap();

        schedule
            .set_attribute(2, DataObject::OctetString(new_time.encode()), None)
            .await
            .unwrap();

        let retrieved = schedule.execution_time().await;
        assert_eq!(retrieved.encode(), new_time.encode());
    }

    #[tokio::test]
    async fn test_single_action_schedule_set_action_script_via_attribute() {
        let schedule = SingleActionSchedule::with_default_obis();

        schedule
            .set_attribute(3, DataObject::Unsigned8(15), None)
            .await
            .unwrap();

        assert_eq!(schedule.action_script().await, 15);
    }

    #[tokio::test]
    async fn test_single_action_schedule_set_enabled_via_attribute() {
        let schedule = SingleActionSchedule::with_default_obis();

        schedule
            .set_attribute(4, DataObject::Boolean(true), None)
            .await
            .unwrap();

        assert!(schedule.is_enabled().await);

        schedule
            .set_attribute(4, DataObject::Boolean(false), None)
            .await
            .unwrap();

        assert!(!schedule.is_enabled().await);
    }

    #[tokio::test]
    async fn test_single_action_schedule_read_only_logical_name() {
        let schedule = SingleActionSchedule::with_default_obis();
        let result = schedule
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 15, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_single_action_schedule_read_only_type() {
        let schedule = SingleActionSchedule::with_default_obis();
        let result = schedule
            .set_attribute(5, DataObject::Enumerate(1), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_single_action_schedule_method_execute() {
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let schedule = SingleActionSchedule::new(
            SingleActionSchedule::default_obis(),
            time,
            8,
            true,
        );

        let result = schedule.invoke_method(1, None, None).await.unwrap();
        match result {
            Some(DataObject::Unsigned8(script_id)) => {
                assert_eq!(script_id, 8);
            }
            _ => panic!("Expected Unsigned8 result"),
        }
    }

    #[tokio::test]
    async fn test_single_action_schedule_invalid_attribute() {
        let schedule = SingleActionSchedule::with_default_obis();
        let result = schedule.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_single_action_schedule_invalid_method() {
        let schedule = SingleActionSchedule::with_default_obis();
        let result = schedule.invoke_method(99, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_single_action_schedule_invalid_execution_time() {
        let schedule = SingleActionSchedule::with_default_obis();
        let result = schedule
            .set_attribute(2, DataObject::OctetString(vec![1, 2, 3]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_single_action_schedule_invalid_action_script() {
        let schedule = SingleActionSchedule::with_default_obis();
        let result = schedule
            .set_attribute(3, DataObject::Boolean(true), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_single_action_schedule_invalid_enabled() {
        let schedule = SingleActionSchedule::with_default_obis();
        let result = schedule
            .set_attribute(4, DataObject::Unsigned8(1), None)
            .await;
        assert!(result.is_err());
    }
}
