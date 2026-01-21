//! Schedule interface class (Class ID: 10)
//!
//! The Schedule interface class defines time-based execution of scripts.
//! It stores scripts that are executed at specific times.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: scripts - Array of schedule entries
//!
//! # Methods
//!
//! - Method 1: script_execute(script_id) - Execute a specific script
//!
//! # Schedule (Class ID: 10)
//!
//! Schedules are used to trigger scripts at specific times. Each entry:
//! - Has a script_id to execute
//! - Has an execution time (CosemDateTime)
//! - Can be enabled or disabled

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{
    datatypes::{CosemDateTime, CosemDateFormat},
    DlmsError, DlmsResult, ObisCode, DataObject,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Schedule Entry - represents a single scheduled script execution
#[derive(Debug, Clone)]
pub struct ScheduleEntry {
    /// Script identifier to execute
    pub script_id: u8,
    /// Execution time
    pub execution_time: CosemDateTime,
    /// Whether this entry is enabled
    pub enabled: bool,
}

impl ScheduleEntry {
    /// Create a new schedule entry
    pub fn new(script_id: u8, execution_time: CosemDateTime) -> Self {
        Self {
            script_id,
            execution_time,
            enabled: true,
        }
    }

    /// Create a disabled schedule entry
    pub fn disabled(script_id: u8, execution_time: CosemDateTime) -> Self {
        Self {
            script_id,
            execution_time,
            enabled: false,
        }
    }

    /// Check if this schedule entry is due (time has passed and entry is enabled)
    pub fn is_due(&self, current_time: &CosemDateTime) -> bool {
        if !self.enabled {
            return false;
        }
        // Compare timestamps - simplified check
        // In a real implementation, this would do proper datetime comparison
        self.execution_time.encode() <= current_time.encode()
    }
}

/// Schedule interface class (Class ID: 10)
///
/// Default OBIS: 0-0:11.0.0.255
///
/// This class manages time-based execution of scripts.
#[derive(Debug, Clone)]
pub struct Schedule {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Schedule entries
    entries: Arc<RwLock<Vec<ScheduleEntry>>>,
}

impl Schedule {
    /// Class ID for Schedule
    pub const CLASS_ID: u16 = 10;

    /// Default OBIS code for Schedule (0-0:11.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 11, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_SCRIPTS: u8 = 2;

    /// Method IDs
    pub const METHOD_SCRIPT_EXECUTE: u8 = 1;

    /// Create a new Schedule object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `entries` - Initial list of schedule entries
    pub fn new(logical_name: ObisCode, entries: Vec<ScheduleEntry>) -> Self {
        Self {
            logical_name,
            entries: Arc::new(RwLock::new(entries)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis(), Vec::new())
    }

    /// Get the number of entries
    pub async fn entry_count(&self) -> usize {
        self.entries.read().await.len()
    }

    /// Get all entries
    pub async fn entries(&self) -> Vec<ScheduleEntry> {
        self.entries.read().await.clone()
    }

    /// Get a specific entry by index
    pub async fn get_entry(&self, index: usize) -> Option<ScheduleEntry> {
        let entries = self.entries.read().await;
        entries.get(index).cloned()
    }

    /// Add an entry to the schedule
    pub async fn add_entry(&self, entry: ScheduleEntry) -> DlmsResult<()> {
        let mut entries = self.entries.write().await;
        entries.push(entry);
        Ok(())
    }

    /// Remove an entry from the schedule by index
    pub async fn remove_entry(&self, index: usize) -> DlmsResult<()> {
        let mut entries = self.entries.write().await;
        if index >= entries.len() {
            return Err(DlmsError::InvalidData(format!(
                "Entry index {} out of bounds",
                index
            )));
        }
        entries.remove(index);
        Ok(())
    }

    /// Enable or disable an entry
    pub async fn set_entry_enabled(&self, index: usize, enabled: bool) -> DlmsResult<()> {
        let mut entries = self.entries.write().await;
        if index >= entries.len() {
            return Err(DlmsError::InvalidData(format!(
                "Entry index {} out of bounds",
                index
            )));
        }
        entries[index].enabled = enabled;
        Ok(())
    }

    /// Update an entry's execution time
    pub async fn update_entry_time(&self, index: usize, new_time: CosemDateTime) -> DlmsResult<()> {
        let mut entries = self.entries.write().await;
        if index >= entries.len() {
            return Err(DlmsError::InvalidData(format!(
                "Entry index {} out of bounds",
                index
            )));
        }
        entries[index].execution_time = new_time;
        Ok(())
    }

    /// Find due entries (entries whose execution time has passed)
    pub async fn find_due_entries(&self, current_time: &CosemDateTime) -> Vec<ScheduleEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|e| e.is_due(current_time))
            .cloned()
            .collect()
    }

    /// Execute a script by ID
    ///
    /// This corresponds to Method 1
    pub async fn execute_script(&self, script_id: u8) -> DlmsResult<ScriptExecutionResult> {
        // In a real implementation, this would:
        // 1. Find the script in a ScriptTable object
        // 2. Execute the script
        // 3. Return the result

        Ok(ScriptExecutionResult {
            script_id,
            success: true,
            result: DataObject::Unsigned8(0), // 0 = success
        })
    }

    /// Clear all entries
    pub async fn clear(&self) -> DlmsResult<()> {
        self.entries.write().await.clear();
        Ok(())
    }

    /// Encode entries as a DataObject (array of structures)
    async fn encode_entries(&self) -> DataObject {
        let entries = self.entries.read().await;
        let mut entry_arrays = Vec::new();

        for entry in entries.iter() {
            let mut entry_data = Vec::new();
            entry_data.push(DataObject::Unsigned8(entry.script_id));
            entry_data.push(DataObject::OctetString(entry.execution_time.encode()));
            entry_data.push(DataObject::Boolean(entry.enabled));
            entry_arrays.push(DataObject::Array(entry_data));
        }

        DataObject::Array(entry_arrays)
    }
}

/// Result of script execution
#[derive(Debug, Clone)]
pub struct ScriptExecutionResult {
    /// ID of the executed script
    pub script_id: u8,
    /// Whether execution was successful
    pub success: bool,
    /// Result code (0 = success)
    pub result: DataObject,
}

#[async_trait]
impl CosemObject for Schedule {
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
            Self::ATTR_SCRIPTS => {
                Ok(self.encode_entries().await)
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Schedule has no attribute {}",
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
            Self::ATTR_SCRIPTS => {
                // Parse and replace all entries
                if let DataObject::Array(entry_arrays) = value {
                    let mut new_entries = Vec::new();
                    for entry_obj in entry_arrays {
                        if let DataObject::Array(entry_data) = entry_obj {
                            if entry_data.len() >= 3 {
                                let script_id = match &entry_data[0] {
                                    DataObject::Unsigned8(id) => *id,
                                    _ => continue,
                                };
                                let execution_time = match &entry_data[1] {
                                    DataObject::OctetString(bytes) if bytes.len() >= 12 => {
                                        CosemDateTime::decode(bytes).unwrap_or_else(|_| {
                                            CosemDateTime::new(2024, 1, 1, 0, 0, 0, 0, &[]).unwrap()
                                        })
                                    }
                                    _ => continue,
                                };
                                let enabled = match &entry_data[2] {
                                    DataObject::Boolean(b) => *b,
                                    _ => true,
                                };
                                new_entries.push(ScheduleEntry {
                                    script_id,
                                    execution_time,
                                    enabled,
                                });
                            }
                        }
                    }
                    *self.entries.write().await = new_entries;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected array for scripts".to_string(),
                    ))
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Schedule has no attribute {}",
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
            Self::METHOD_SCRIPT_EXECUTE => {
                if let Some(DataObject::Unsigned8(script_id)) = parameters {
                    let result = self.execute_script(script_id).await?;
                    Ok(Some(result.result))
                } else {
                    Err(DlmsError::InvalidData(
                        "Method 1 requires an Unsigned8 script_id parameter".to_string(),
                    ))
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Schedule has no method {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_schedule_class_id() {
        let schedule = Schedule::with_default_obis();
        assert_eq!(schedule.class_id(), 10);
    }

    #[tokio::test]
    async fn test_schedule_obis_code() {
        let schedule = Schedule::with_default_obis();
        assert_eq!(schedule.obis_code(), Schedule::default_obis());
    }

    #[tokio::test]
    async fn test_schedule_empty() {
        let schedule = Schedule::with_default_obis();
        assert_eq!(schedule.entry_count().await, 0);
    }

    #[tokio::test]
    async fn test_schedule_add_entry() {
        let schedule = Schedule::with_default_obis();
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let entry = ScheduleEntry::new(1, time);

        schedule.add_entry(entry).await.unwrap();
        assert_eq!(schedule.entry_count().await, 1);
    }

    #[tokio::test]
    async fn test_schedule_entry_enabled() {
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let entry = ScheduleEntry::new(1, time.clone());
        assert!(entry.enabled);

        let disabled = ScheduleEntry::disabled(1, time);
        assert!(!disabled.enabled);
    }

    #[tokio::test]
    async fn test_schedule_remove_entry() {
        let schedule = Schedule::with_default_obis();
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let entry = ScheduleEntry::new(1, time);

        schedule.add_entry(entry).await.unwrap();
        assert_eq!(schedule.entry_count().await, 1);

        schedule.remove_entry(0).await.unwrap();
        assert_eq!(schedule.entry_count().await, 0);
    }

    #[tokio::test]
    async fn test_schedule_remove_invalid_index() {
        let schedule = Schedule::with_default_obis();
        let result = schedule.remove_entry(0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_schedule_set_entry_enabled() {
        let schedule = Schedule::with_default_obis();
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let entry = ScheduleEntry::new(1, time);

        schedule.add_entry(entry).await.unwrap();
        schedule.set_entry_enabled(0, false).await.unwrap();

        let retrieved = schedule.get_entry(0).await.unwrap();
        assert!(!retrieved.enabled);
    }

    #[tokio::test]
    async fn test_schedule_update_entry_time() {
        let schedule = Schedule::with_default_obis();
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let entry = ScheduleEntry::new(1, time);

        schedule.add_entry(entry).await.unwrap();

        let new_time = CosemDateTime::new(2024, 6, 16, 12, 0, 0, 0, &[]).unwrap();
        schedule.update_entry_time(0, new_time.clone()).await.unwrap();

        let retrieved = schedule.get_entry(0).await.unwrap();
        assert_eq!(retrieved.execution_time.encode(), new_time.clone().encode());
    }

    #[tokio::test]
    async fn test_schedule_clear() {
        let schedule = Schedule::with_default_obis();
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let entry = ScheduleEntry::new(1, time);

        schedule.add_entry(entry).await.unwrap();
        assert_eq!(schedule.entry_count().await, 1);

        schedule.clear().await.unwrap();
        assert_eq!(schedule.entry_count().await, 0);
    }

    #[tokio::test]
    async fn test_schedule_execute_script() {
        let schedule = Schedule::with_default_obis();
        let result = schedule.execute_script(1).await.unwrap();
        assert_eq!(result.script_id, 1);
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_schedule_method_invoke() {
        let schedule = Schedule::with_default_obis();
        let result = schedule.invoke_method(1, Some(DataObject::Unsigned8(5)), None).await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_schedule_method_invalid_params() {
        let schedule = Schedule::with_default_obis();
        let result = schedule.invoke_method(1, Some(DataObject::Integer64(123)), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_schedule_get_logical_name() {
        let schedule = Schedule::with_default_obis();
        let result = schedule.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_schedule_read_only_logical_name() {
        let schedule = Schedule::with_default_obis();
        let result = schedule.set_attribute(1, DataObject::OctetString(vec![0, 0, 11, 0, 0, 1]), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_schedule_get_entries() {
        let schedule = Schedule::with_default_obis();
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        let entry = ScheduleEntry::new(1, time);

        schedule.add_entry(entry).await.unwrap();
        let result = schedule.get_attribute(2, None).await.unwrap();

        match result {
            DataObject::Array(entries) => {
                assert_eq!(entries.len(), 1);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_schedule_set_entries() {
        let schedule = Schedule::with_default_obis();
        let time = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();

        let entry_data = DataObject::Array(vec![
            DataObject::Array(vec![
                DataObject::Unsigned8(5),
                DataObject::OctetString(time.encode()),
                DataObject::Boolean(true),
            ]),
        ]);

        schedule.set_attribute(2, entry_data, None).await.unwrap();
        assert_eq!(schedule.entry_count().await, 1);

        let entry = schedule.get_entry(0).await.unwrap();
        assert_eq!(entry.script_id, 5);
        assert!(entry.enabled);
    }

    #[tokio::test]
    async fn test_schedule_invalid_attribute() {
        let schedule = Schedule::with_default_obis();
        let result = schedule.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_schedule_invalid_method() {
        let schedule = Schedule::with_default_obis();
        let result = schedule.invoke_method(99, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_schedule_entry_is_due() {
        let past_time = CosemDateTime::new(2024, 1, 1, 0, 0, 0, 0, &[]).unwrap();
        let future_time = CosemDateTime::new(2025, 1, 1, 0, 0, 0, 0, &[]).unwrap();
        let current = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();

        let past_entry = ScheduleEntry::new(1, past_time.clone());
        let future_entry = ScheduleEntry::new(1, future_time);
        let disabled_entry = ScheduleEntry::disabled(1, past_time);

        assert!(past_entry.is_due(&current));
        assert!(!future_entry.is_due(&current));
        assert!(!disabled_entry.is_due(&current));
    }
}
