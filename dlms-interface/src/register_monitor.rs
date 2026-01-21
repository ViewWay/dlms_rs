//! Register Monitor interface class (Class ID: 21)
//!
//! The Register Monitor interface class monitors a register value
//! and can trigger actions when thresholds are crossed.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: monitored_value - Reference to the monitored register
//! - Attribute 3: threshold_list - List of thresholds with actions
//! - Attribute 4: actions - Actions to execute on threshold crossing
//!
//! # Methods
//!
//! - Method 1: activate() - Activate the monitor
//! - Method 2: deactivate() - Deactivate the monitor
//!
//! # Register Monitor (Class ID: 21)
//!
//! This class monitors register values and can trigger actions
//! when configured thresholds are crossed (rising or falling).

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{CosemObject, descriptor::ObisCodeExt};

/// Threshold direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ThresholdDirection {
    /// Both rising and falling
    Both = 0,
    /// Rising only (value exceeds threshold)
    Rising = 1,
    /// Falling only (value drops below threshold)
    Falling = 2,
}

impl ThresholdDirection {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Both,
            1 => Self::Rising,
            2 => Self::Falling,
            _ => Self::Both,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Monitor threshold configuration
#[derive(Debug, Clone)]
pub struct MonitorThreshold {
    /// Threshold value
    pub value: DataObject,
    /// Direction to trigger on
    pub direction: ThresholdDirection,
    /// Action to execute when threshold is crossed
    pub action: MonitorAction,
}

impl MonitorThreshold {
    /// Create a new monitor threshold
    pub fn new(value: DataObject, direction: ThresholdDirection, action: MonitorAction) -> Self {
        Self {
            value,
            direction,
            action,
        }
    }

    /// Create from data object (array)
    pub fn from_data_object(value: &DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Array(arr) if arr.len() >= 3 => {
                let threshold_value = arr[0].clone();
                let direction = match &arr[1] {
                    DataObject::Enumerate(d) => ThresholdDirection::from_u8(*d),
                    DataObject::Unsigned8(d) => ThresholdDirection::from_u8(*d),
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Enumerate for direction".to_string(),
                        ))
                    }
                };
                let action = MonitorAction::from_data_object(&arr[2])?;
                Ok(Self {
                    value: threshold_value,
                    direction,
                    action,
                })
            }
            _ => Err(DlmsError::InvalidData(
                "Expected Array for MonitorThreshold".to_string(),
            )),
        }
    }

    /// Convert to data object
    pub fn to_data_object(&self) -> DataObject {
        DataObject::Array(vec![
            self.value.clone(),
            DataObject::Enumerate(self.direction.to_u8()),
            self.action.to_data_object(),
        ])
    }
}

/// Monitor action
#[derive(Debug, Clone, PartialEq)]
pub enum MonitorAction {
    /// No action
    None,
    /// Send an event/notification
    SendEvent,
    /// Execute a script (script table index)
    ExecuteScript(u8),
    /// Set a specific object's value
    SetValue {
        class_id: u16,
        obis_code: ObisCode,
        attribute_id: u8,
        value: DataObject,
    },
}

impl MonitorAction {
    /// Create from data object
    pub fn from_data_object(value: &DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Enumerate(0) => Ok(Self::None),
            DataObject::Enumerate(1) => Ok(Self::SendEvent),
            DataObject::Array(arr) if arr.len() >= 2 => {
                match &arr[0] {
                    DataObject::Enumerate(2) | DataObject::Unsigned8(2) => {
                        // Execute script
                        let script_id = match &arr[1] {
                            DataObject::Unsigned8(id) => *id,
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Expected Unsigned8 for script_id".to_string(),
                                ))
                            }
                        };
                        Ok(Self::ExecuteScript(script_id))
                    }
                    DataObject::Enumerate(3) | DataObject::Unsigned8(3) => {
                        // Set value
                        if arr.len() < 5 {
                            return Err(DlmsError::InvalidData(
                                "SetValue action requires 5 elements".to_string(),
                            ));
                        }
                        let class_id = match &arr[1] {
                            DataObject::Unsigned16(id) => *id,
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Expected Unsigned16 for class_id".to_string(),
                                ))
                            }
                        };
                        let obis_code = match &arr[2] {
                            DataObject::OctetString(bytes) if bytes.len() == 6 => {
                                ObisCode::from_bytes(bytes)?
                            }
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Expected OctetString(6) for obis_code".to_string(),
                                ))
                            }
                        };
                        let attribute_id = match &arr[3] {
                            DataObject::Unsigned8(id) => *id,
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Expected Unsigned8 for attribute_id".to_string(),
                                ))
                            }
                        };
                        let value = arr[4].clone();
                        Ok(Self::SetValue {
                            class_id,
                            obis_code,
                            attribute_id,
                            value,
                        })
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Unknown action type".to_string(),
                    )),
                }
            }
            _ => Ok(Self::None),
        }
    }

    /// Convert to data object
    pub fn to_data_object(&self) -> DataObject {
        match self {
            Self::None => DataObject::Enumerate(0),
            Self::SendEvent => DataObject::Enumerate(1),
            Self::ExecuteScript(id) => {
                DataObject::Array(vec![
                    DataObject::Enumerate(2),
                    DataObject::Unsigned8(*id),
                ])
            }
            Self::SetValue {
                class_id,
                obis_code,
                attribute_id,
                value,
            } => DataObject::Array(vec![
                DataObject::Enumerate(3),
                DataObject::Unsigned16(*class_id),
                DataObject::OctetString(obis_code.to_bytes().to_vec()),
                DataObject::Unsigned8(*attribute_id),
                value.clone(),
            ]),
        }
    }
}

/// Monitored value reference
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitoredValueRef {
    /// Class ID of the monitored object
    pub class_id: u16,
    /// Logical name (OBIS code) of the monitored object
    pub obis_code: ObisCode,
    /// Attribute index to monitor
    pub attribute_index: u8,
}

impl MonitoredValueRef {
    /// Create a new monitored value reference
    pub fn new(class_id: u16, obis_code: ObisCode, attribute_index: u8) -> Self {
        Self {
            class_id,
            obis_code,
            attribute_index,
        }
    }

    /// Create from data object
    pub fn from_data_object(value: &DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Array(arr) if arr.len() >= 3 => {
                let class_id = match &arr[0] {
                    DataObject::Unsigned16(id) => *id,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Unsigned16 for class_id".to_string(),
                        ))
                    }
                };
                let obis_code = match &arr[1] {
                    DataObject::OctetString(bytes) if bytes.len() == 6 => {
                        ObisCode::from_bytes(bytes)?
                    }
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected OctetString(6) for obis_code".to_string(),
                        ))
                    }
                };
                let attribute_index = match &arr[2] {
                    DataObject::Unsigned8(idx) => *idx,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Unsigned8 for attribute_index".to_string(),
                        ))
                    }
                };
                Ok(Self {
                    class_id,
                    obis_code,
                    attribute_index,
                })
            }
            _ => Err(DlmsError::InvalidData(
                "Expected Array for MonitoredValueRef".to_string(),
            )),
        }
    }

    /// Convert to data object
    pub fn to_data_object(&self) -> DataObject {
        DataObject::Array(vec![
            DataObject::Unsigned16(self.class_id),
            DataObject::OctetString(self.obis_code.to_bytes().to_vec()),
            DataObject::Unsigned8(self.attribute_index),
        ])
    }
}

/// Register Monitor interface class (Class ID: 21)
///
/// Default OBIS: 0-0:14.0.0.255
///
/// This class monitors register values and triggers actions
/// when configured thresholds are crossed.
#[derive(Debug, Clone)]
pub struct RegisterMonitor {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Reference to the monitored value
    monitored_value: Arc<RwLock<Option<MonitoredValueRef>>>,

    /// List of thresholds with associated actions
    threshold_list: Arc<RwLock<Vec<MonitorThreshold>>>,

    /// Active state
    is_active: Arc<RwLock<bool>>,

    /// Last recorded value (for comparison)
    last_value: Arc<RwLock<Option<DataObject>>>,
}

impl RegisterMonitor {
    /// Class ID for Register Monitor
    pub const CLASS_ID: u16 = 21;

    /// Default OBIS code for Register Monitor (0-0:14.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 14, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_MONITORED_VALUE: u8 = 2;
    pub const ATTR_THRESHOLD_LIST: u8 = 3;

    /// Method IDs
    pub const METHOD_ACTIVATE: u8 = 1;
    pub const METHOD_DEACTIVATE: u8 = 2;

    /// Create a new Register Monitor object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            monitored_value: Arc::new(RwLock::new(None)),
            threshold_list: Arc::new(RwLock::new(Vec::new())),
            is_active: Arc::new(RwLock::new(false)),
            last_value: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get the monitored value reference
    pub async fn monitored_value(&self) -> Option<MonitoredValueRef> {
        self.monitored_value.read().await.clone()
    }

    /// Set the monitored value reference
    pub async fn set_monitored_value(&self, value: Option<MonitoredValueRef>) {
        *self.monitored_value.write().await = value;
    }

    /// Get the threshold list
    pub async fn threshold_list(&self) -> Vec<MonitorThreshold> {
        self.threshold_list.read().await.clone()
    }

    /// Add a threshold
    pub async fn add_threshold(&self, threshold: MonitorThreshold) {
        self.threshold_list.write().await.push(threshold);
    }

    /// Remove a threshold by index
    pub async fn remove_threshold(&self, index: usize) -> DlmsResult<()> {
        let mut list = self.threshold_list.write().await;
        if index >= list.len() {
            return Err(DlmsError::InvalidData("Index out of bounds".to_string()));
        }
        list.remove(index);
        Ok(())
    }

    /// Clear all thresholds
    pub async fn clear_thresholds(&self) {
        self.threshold_list.write().await.clear();
    }

    /// Check if the monitor is active
    pub async fn is_active(&self) -> bool {
        *self.is_active.read().await
    }

    /// Activate the monitor
    pub async fn activate(&self) -> DlmsResult<()> {
        if self.monitored_value().await.is_none() {
            return Err(DlmsError::InvalidData(
                "No monitored value configured".to_string(),
            ));
        }
        *self.is_active.write().await = true;
        Ok(())
    }

    /// Deactivate the monitor
    pub async fn deactivate(&self) {
        *self.is_active.write().await = false;
    }

    /// Get the last recorded value
    pub async fn last_value(&self) -> Option<DataObject> {
        self.last_value.read().await.clone()
    }

    /// Update the monitored value and check thresholds
    /// Returns a list of actions that should be executed
    pub async fn update_value(&self, new_value: DataObject) -> Vec<MonitorAction> {
        let mut actions = Vec::new();
        let old_value = self.last_value().await;
        let thresholds = self.threshold_list().await;

        for threshold in &thresholds {
            match &old_value {
                None => {
                    // First value - check if threshold is crossed from zero/none
                    // This is implementation specific
                }
                Some(old) => {
                    // Check if value crossed threshold
                    if self.check_threshold_crossed(old, &new_value, threshold) {
                        actions.push(threshold.action.clone());
                    }
                }
            }
        }

        *self.last_value.write().await = Some(new_value);
        actions
    }

    /// Check if a threshold was crossed
    fn check_threshold_crossed(
        &self,
        old_value: &DataObject,
        new_value: &DataObject,
        threshold: &MonitorThreshold,
    ) -> bool {
        // Simple comparison for numeric values
        // A real implementation would handle different types properly
        match (&threshold.value, old_value, new_value) {
            (DataObject::Integer64(threshold_val), DataObject::Integer64(old), DataObject::Integer64(new)) => {
                match threshold.direction {
                    ThresholdDirection::Both => {
                        (old <= threshold_val && new > threshold_val)
                            || (old >= threshold_val && new < threshold_val)
                    }
                    ThresholdDirection::Rising => old <= threshold_val && new > threshold_val,
                    ThresholdDirection::Falling => old >= threshold_val && new < threshold_val,
                }
            }
            (DataObject::Unsigned32(threshold_val), DataObject::Unsigned32(old), DataObject::Unsigned32(new)) => {
                match threshold.direction {
                    ThresholdDirection::Both => {
                        (old <= threshold_val && new > threshold_val)
                            || (old >= threshold_val && new < threshold_val)
                    }
                    ThresholdDirection::Rising => old <= threshold_val && new > threshold_val,
                    ThresholdDirection::Falling => old >= threshold_val && new < threshold_val,
                }
            }
            _ => false,
        }
    }
}

#[async_trait]
impl CosemObject for RegisterMonitor {
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
                match self.monitored_value().await {
                    Some(ref val) => Ok(val.to_data_object()),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_THRESHOLD_LIST => {
                let thresholds = self.threshold_list().await;
                let data: Vec<DataObject> =
                    thresholds.iter().map(|t| t.to_data_object()).collect();
                Ok(DataObject::Array(data))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Register Monitor has no attribute {}",
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
                    DataObject::Null => {
                        self.set_monitored_value(None).await;
                        Ok(())
                    }
                    _ => {
                        let val = MonitoredValueRef::from_data_object(&value)?;
                        self.set_monitored_value(Some(val)).await;
                        Ok(())
                    }
                }
            }
            Self::ATTR_THRESHOLD_LIST => {
                match value {
                    DataObject::Array(arr) => {
                        self.clear_thresholds().await;
                        for item in arr {
                            let threshold = MonitorThreshold::from_data_object(&item)?;
                            self.add_threshold(threshold).await;
                        }
                        Ok(())
                    }
                    DataObject::Null => {
                        self.clear_thresholds().await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Array for threshold_list".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Register Monitor has no attribute {}",
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
            Self::METHOD_ACTIVATE => {
                self.activate().await?;
                Ok(None)
            }
            Self::METHOD_DEACTIVATE => {
                self.deactivate().await;
                Ok(None)
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Register Monitor has no method {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_monitor_class_id() {
        let monitor = RegisterMonitor::with_default_obis();
        assert_eq!(monitor.class_id(), 21);
    }

    #[tokio::test]
    async fn test_register_monitor_obis_code() {
        let monitor = RegisterMonitor::with_default_obis();
        assert_eq!(monitor.obis_code(), RegisterMonitor::default_obis());
    }

    #[tokio::test]
    async fn test_register_monitor_initial_state() {
        let monitor = RegisterMonitor::with_default_obis();
        assert!(monitor.monitored_value().await.is_none());
        assert!(monitor.threshold_list().await.is_empty());
        assert!(!monitor.is_active().await);
        assert!(monitor.last_value().await.is_none());
    }

    #[tokio::test]
    async fn test_register_monitor_set_monitored_value() {
        let monitor = RegisterMonitor::with_default_obis();
        let val = MonitoredValueRef::new(3, ObisCode::new(1, 1, 1, 1, 1, 255), 2);
        monitor.set_monitored_value(Some(val.clone())).await;

        let retrieved = monitor.monitored_value().await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), val);
    }

    #[tokio::test]
    async fn test_register_monitor_add_threshold() {
        let monitor = RegisterMonitor::with_default_obis();
        let threshold = MonitorThreshold::new(
            DataObject::Integer64(100),
            ThresholdDirection::Rising,
            MonitorAction::SendEvent,
        );
        monitor.add_threshold(threshold.clone()).await;

        let list = monitor.threshold_list().await;
        assert_eq!(list.len(), 1);
        // Check individual fields instead of full struct comparison
        assert_eq!(list[0].direction, threshold.direction);
    }

    #[tokio::test]
    async fn test_register_monitor_remove_threshold() {
        let monitor = RegisterMonitor::with_default_obis();
        monitor
            .add_threshold(MonitorThreshold::new(
                DataObject::Integer64(100),
                ThresholdDirection::Rising,
                MonitorAction::SendEvent,
            ))
            .await;
        monitor
            .add_threshold(MonitorThreshold::new(
                DataObject::Integer64(200),
                ThresholdDirection::Falling,
                MonitorAction::SendEvent,
            ))
            .await;

        monitor.remove_threshold(0).await.unwrap();
        assert_eq!(monitor.threshold_list().await.len(), 1);
    }

    #[tokio::test]
    async fn test_register_monitor_clear_thresholds() {
        let monitor = RegisterMonitor::with_default_obis();
        monitor
            .add_threshold(MonitorThreshold::new(
                DataObject::Integer64(100),
                ThresholdDirection::Rising,
                MonitorAction::SendEvent,
            ))
            .await;
        monitor.clear_thresholds().await;
        assert!(monitor.threshold_list().await.is_empty());
    }

    #[tokio::test]
    async fn test_register_monitor_activate_deactivate() {
        let monitor = RegisterMonitor::with_default_obis();
        let val = MonitoredValueRef::new(3, ObisCode::new(1, 1, 1, 1, 1, 255), 2);
        monitor.set_monitored_value(Some(val)).await;

        assert!(!monitor.is_active().await);
        monitor.activate().await.unwrap();
        assert!(monitor.is_active().await);
        monitor.deactivate().await;
        assert!(!monitor.is_active().await);
    }

    #[tokio::test]
    async fn test_register_monitor_activate_no_value() {
        let monitor = RegisterMonitor::with_default_obis();
        let result = monitor.activate().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_monitor_update_value_rising() {
        let monitor = RegisterMonitor::with_default_obis();
        let threshold = MonitorThreshold::new(
            DataObject::Integer64(100),
            ThresholdDirection::Rising,
            MonitorAction::SendEvent,
        );
        monitor.add_threshold(threshold).await;

        // Set up a monitored value reference before activating
        let val = MonitoredValueRef::new(3, ObisCode::new(1, 1, 1, 1, 1, 255), 2);
        monitor.set_monitored_value(Some(val)).await;
        monitor.activate().await.unwrap();

        // First value sets baseline
        monitor.update_value(DataObject::Integer64(50)).await;
        // Second value crosses threshold
        let actions = monitor.update_value(DataObject::Integer64(150)).await;
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], MonitorAction::SendEvent);
    }

    #[tokio::test]
    async fn test_register_monitor_update_value_no_cross() {
        let monitor = RegisterMonitor::with_default_obis();
        let threshold = MonitorThreshold::new(
            DataObject::Integer64(100),
            ThresholdDirection::Rising,
            MonitorAction::SendEvent,
        );
        monitor.add_threshold(threshold).await;

        // First value - no previous value to compare
        let actions = monitor.update_value(DataObject::Integer64(50)).await;
        assert_eq!(actions.len(), 0);
    }

    #[tokio::test]
    async fn test_threshold_direction_from_u8() {
        assert_eq!(ThresholdDirection::from_u8(0), ThresholdDirection::Both);
        assert_eq!(ThresholdDirection::from_u8(1), ThresholdDirection::Rising);
        assert_eq!(ThresholdDirection::from_u8(2), ThresholdDirection::Falling);
        assert_eq!(ThresholdDirection::from_u8(99), ThresholdDirection::Both);
    }

    #[tokio::test]
    async fn test_monitor_action_to_data_object() {
        let action = MonitorAction::SendEvent;
        let data = action.to_data_object();
        assert_eq!(data, DataObject::Enumerate(1));

        let action = MonitorAction::ExecuteScript(5);
        let data = action.to_data_object();
        match data {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 2);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_monitored_value_ref_to_data_object() {
        let val = MonitoredValueRef::new(3, ObisCode::new(1, 1, 1, 1, 1, 255), 2);
        let data = val.to_data_object();
        match data {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 3);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_monitor_threshold_to_data_object() {
        let threshold = MonitorThreshold::new(
            DataObject::Integer64(100),
            ThresholdDirection::Rising,
            MonitorAction::SendEvent,
        );
        let data = threshold.to_data_object();
        match data {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 3);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_register_monitor_get_logical_name() {
        let monitor = RegisterMonitor::with_default_obis();
        let result = monitor.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_register_monitor_get_monitored_value() {
        let monitor = RegisterMonitor::with_default_obis();
        let result = monitor.get_attribute(2, None).await.unwrap();
        assert_eq!(result, DataObject::Null);

        let val = MonitoredValueRef::new(3, ObisCode::new(1, 1, 1, 1, 1, 255), 2);
        monitor.set_monitored_value(Some(val)).await;

        let result = monitor.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Array(_) => {}
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_register_monitor_get_threshold_list() {
        let monitor = RegisterMonitor::with_default_obis();
        let result = monitor.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 0);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_register_monitor_set_monitored_value_via_attribute() {
        let monitor = RegisterMonitor::with_default_obis();
        let val_data = DataObject::Array(vec![
            DataObject::Unsigned16(3),
            DataObject::OctetString(vec![1, 1, 1, 1, 1, 255]),
            DataObject::Unsigned8(2),
        ]);
        monitor.set_attribute(2, val_data, None).await.unwrap();

        assert!(monitor.monitored_value().await.is_some());
    }

    #[tokio::test]
    async fn test_register_monitor_set_threshold_list_via_attribute() {
        let monitor = RegisterMonitor::with_default_obis();
        let threshold_data = DataObject::Array(vec![
            DataObject::Integer64(100),
            DataObject::Enumerate(1),
            DataObject::Enumerate(1),
        ]);
        monitor
            .set_attribute(3, DataObject::Array(vec![threshold_data]), None)
            .await
            .unwrap();

        assert_eq!(monitor.threshold_list().await.len(), 1);
    }

    #[tokio::test]
    async fn test_register_monitor_read_only_logical_name() {
        let monitor = RegisterMonitor::with_default_obis();
        let result = monitor
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 14, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_monitor_method_activate() {
        let monitor = RegisterMonitor::with_default_obis();
        let val = MonitoredValueRef::new(3, ObisCode::new(1, 1, 1, 1, 1, 255), 2);
        monitor.set_monitored_value(Some(val)).await;

        let result = monitor.invoke_method(1, None, None).await;
        assert!(result.is_ok());
        assert!(monitor.is_active().await);
    }

    #[tokio::test]
    async fn test_register_monitor_method_deactivate() {
        let monitor = RegisterMonitor::with_default_obis();
        let val = MonitoredValueRef::new(3, ObisCode::new(1, 1, 1, 1, 1, 255), 2);
        monitor.set_monitored_value(Some(val)).await;
        monitor.activate().await.unwrap();

        monitor.invoke_method(2, None, None).await.unwrap();
        assert!(!monitor.is_active().await);
    }

    #[tokio::test]
    async fn test_register_monitor_invalid_attribute() {
        let monitor = RegisterMonitor::with_default_obis();
        let result = monitor.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_monitor_invalid_method() {
        let monitor = RegisterMonitor::with_default_obis();
        let result = monitor.invoke_method(99, None, None).await;
        assert!(result.is_err());
    }
}
