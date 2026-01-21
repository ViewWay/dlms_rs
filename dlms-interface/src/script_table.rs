//! Script Table interface class (Class ID: 9)
//!
//! The Script Table interface class stores scripts that can be executed
//! on the meter. Each script consists of a sequence of actions that
//! perform specific operations.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: script_count - Number of scripts in the table
//! - Attribute 3: scripts - Array of script descriptors
//!
//! # Methods
//!
//! - Method 1: script_execute(script_id) - Execute a specific script
//!
//! # Script Table (Class ID: 9)
//!
//! Scripts are used to automate operations on the meter. Each script:
//! - Has a unique script_id (1-255)
//! - Contains a list of actions to execute
//! - Can be invoked remotely or triggered internally

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Script Action - represents a single action within a script
#[derive(Debug, Clone)]
pub struct ScriptAction {
    /// Type of action
    pub action_type: u8,
    /// Action-specific parameters
    pub parameters: Vec<DataObject>,
}

impl ScriptAction {
    /// Create a new script action
    pub fn new(action_type: u8, parameters: Vec<DataObject>) -> Self {
        Self {
            action_type,
            parameters,
        }
    }

    /// Create a simple action with no parameters
    pub fn simple(action_type: u8) -> Self {
        Self {
            action_type,
            parameters: Vec::new(),
        }
    }
}

/// Script Descriptor - represents a single script in the table
#[derive(Debug, Clone)]
pub struct ScriptDescriptor {
    /// Script identifier (1-255)
    pub script_id: u8,
    /// List of actions in this script
    pub actions: Vec<ScriptAction>,
}

impl ScriptDescriptor {
    /// Create a new script descriptor
    pub fn new(script_id: u8, actions: Vec<ScriptAction>) -> Self {
        Self {
            script_id,
            actions,
        }
    }

    /// Create an empty script
    pub fn empty(script_id: u8) -> Self {
        Self {
            script_id,
            actions: Vec::new(),
        }
    }

    /// Add an action to this script
    pub fn add_action(&mut self, action: ScriptAction) {
        self.actions.push(action);
    }
}

/// Script Table interface class (Class ID: 9)
///
/// Default OBIS: 0-0:10.0.0.255
///
/// This class manages scripts that can be executed on the meter.
/// Scripts are used for automation and batch operations.
#[derive(Debug, Clone)]
pub struct ScriptTable {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Scripts stored in this table
    scripts: Arc<RwLock<Vec<ScriptDescriptor>>>,
}

impl ScriptTable {
    /// Class ID for Script Table
    pub const CLASS_ID: u16 = 9;

    /// Default OBIS code for Script Table (0-0:10.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 10, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_SCRIPT_COUNT: u8 = 2;
    pub const ATTR_SCRIPTS: u8 = 3;

    /// Method IDs
    pub const METHOD_SCRIPT_EXECUTE: u8 = 1;

    /// Create a new Script Table object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `scripts` - Initial list of scripts
    pub fn new(logical_name: ObisCode, scripts: Vec<ScriptDescriptor>) -> Self {
        Self {
            logical_name,
            scripts: Arc::new(RwLock::new(scripts)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis(), Vec::new())
    }

    /// Get the number of scripts
    pub async fn script_count(&self) -> u8 {
        self.scripts.read().await.len() as u8
    }

    /// Get all scripts
    pub async fn scripts(&self) -> Vec<ScriptDescriptor> {
        self.scripts.read().await.clone()
    }

    /// Get a specific script by ID
    pub async fn get_script(&self, script_id: u8) -> Option<ScriptDescriptor> {
        let scripts = self.scripts.read().await;
        scripts.iter().find(|s| s.script_id == script_id).cloned()
    }

    /// Add a script to the table
    pub async fn add_script(&self, script: ScriptDescriptor) -> DlmsResult<()> {
        let mut scripts = self.scripts.write().await;
        // Check if script_id already exists
        if scripts.iter().any(|s| s.script_id == script.script_id) {
            return Err(DlmsError::InvalidData(format!(
                "Script ID {} already exists",
                script.script_id
            )));
        }
        scripts.push(script);
        Ok(())
    }

    /// Remove a script from the table
    pub async fn remove_script(&self, script_id: u8) -> DlmsResult<()> {
        let mut scripts = self.scripts.write().await;
        let original_len = scripts.len();
        scripts.retain(|s| s.script_id != script_id);
        if scripts.len() == original_len {
            return Err(DlmsError::InvalidData(format!(
                "Script ID {} not found",
                script_id
            )));
        }
        Ok(())
    }

    /// Execute a script by ID
    ///
    /// This corresponds to Method 1
    pub async fn execute_script(&self, script_id: u8) -> DlmsResult<ScriptExecutionResult> {
        let scripts = self.scripts.read().await;
        let script = scripts
            .iter()
            .find(|s| s.script_id == script_id)
            .ok_or_else(|| DlmsError::InvalidData(format!("Script ID {} not found", script_id)))?;

        // Execute each action in sequence
        let mut results = Vec::new();
        for action in &script.actions {
            // In a real implementation, this would execute the actual action
            // For now, we just record that the action was processed
            results.push(format!("Executed action type {}", action.action_type));
        }

        Ok(ScriptExecutionResult {
            script_id,
            actions_executed: results.len() as u8,
            result: DataObject::Unsigned8(0), // 0 = success
        })
    }

    /// Update a script in the table
    pub async fn update_script(&self, script: ScriptDescriptor) -> DlmsResult<()> {
        let mut scripts = self.scripts.write().await;
        let pos = scripts
            .iter()
            .position(|s| s.script_id == script.script_id)
            .ok_or_else(|| {
                DlmsError::InvalidData(format!("Script ID {} not found", script.script_id))
            })?;
        scripts[pos] = script;
        Ok(())
    }

    /// Encode scripts as a DataObject (array of arrays)
    async fn encode_scripts(&self) -> DataObject {
        let scripts = self.scripts.read().await;
        let mut script_arrays = Vec::new();

        for script in scripts.iter() {
            let mut script_data = Vec::new();
            script_data.push(DataObject::Unsigned8(script.script_id));

            // Encode actions as an array
            let mut action_arrays = Vec::new();
            for action in &script.actions {
                let mut action_data = Vec::new();
                action_data.push(DataObject::Unsigned8(action.action_type));
                // Add parameters (simplified - in reality would encode properly)
                for param in &action.parameters {
                    action_data.push(param.clone());
                }
                action_arrays.push(DataObject::Array(action_data));
            }
            script_data.push(DataObject::Array(action_arrays));

            script_arrays.push(DataObject::Array(script_data));
        }

        DataObject::Array(script_arrays)
    }
}

/// Result of script execution
#[derive(Debug, Clone)]
pub struct ScriptExecutionResult {
    /// ID of the executed script
    pub script_id: u8,
    /// Number of actions executed
    pub actions_executed: u8,
    /// Result code (0 = success)
    pub result: DataObject,
}

#[async_trait]
impl CosemObject for ScriptTable {
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
            Self::ATTR_SCRIPT_COUNT => {
                Ok(DataObject::Unsigned8(self.script_count().await))
            }
            Self::ATTR_SCRIPTS => {
                Ok(self.encode_scripts().await)
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Script Table has no attribute {}",
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
            Self::ATTR_SCRIPT_COUNT => {
                Err(DlmsError::AccessDenied(
                    "Attribute 2 (script_count) is read-only".to_string(),
                ))
            }
            Self::ATTR_SCRIPTS => {
                // Parse and replace all scripts
                if let DataObject::Array(script_arrays) = value {
                    let mut new_scripts = Vec::new();
                    for script_obj in script_arrays {
                        if let DataObject::Array(script_data) = script_obj {
                            if script_data.len() >= 2 {
                                if let DataObject::Unsigned8(script_id) = script_data[0] {
                                    if let DataObject::Array(action_arrays) = &script_data[1] {
                                        let mut actions = Vec::new();
                                        for action_obj in action_arrays {
                                            if let DataObject::Array(action_data) = action_obj {
                                                if !action_data.is_empty() {
                                                    if let DataObject::Unsigned8(action_type) = action_data[0] {
                                                        // Extract parameters if present
                                                        let params = if action_data.len() > 1 {
                                                            action_data[1..].to_vec()
                                                        } else {
                                                            Vec::new()
                                                        };
                                                        actions.push(ScriptAction::new(action_type, params));
                                                    }
                                                }
                                            }
                                        }
                                        new_scripts.push(ScriptDescriptor::new(script_id, actions));
                                    }
                                }
                            }
                        }
                    }
                    *self.scripts.write().await = new_scripts;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected array for scripts".to_string(),
                    ))
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Script Table has no attribute {}",
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
                "Script Table has no method {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_script_table_class_id() {
        let table = ScriptTable::with_default_obis();
        assert_eq!(table.class_id(), 9);
    }

    #[tokio::test]
    async fn test_script_table_obis_code() {
        let table = ScriptTable::with_default_obis();
        assert_eq!(table.obis_code(), ScriptTable::default_obis());
    }

    #[tokio::test]
    async fn test_script_table_empty() {
        let table = ScriptTable::with_default_obis();
        assert_eq!(table.script_count().await, 0);

        let result = table.get_attribute(2, None).await.unwrap();
        assert_eq!(result, DataObject::Unsigned8(0));
    }

    #[tokio::test]
    async fn test_script_table_add_script() {
        let table = ScriptTable::with_default_obis();
        let script = ScriptDescriptor::new(1, vec![
            ScriptAction::simple(1),
            ScriptAction::simple(2),
        ]);

        table.add_script(script).await.unwrap();
        assert_eq!(table.script_count().await, 1);
    }

    #[tokio::test]
    async fn test_script_table_duplicate_id() {
        let table = ScriptTable::with_default_obis();
        let script1 = ScriptDescriptor::new(1, vec![ScriptAction::simple(1)]);
        let script2 = ScriptDescriptor::new(1, vec![ScriptAction::simple(2)]);

        table.add_script(script1).await.unwrap();
        let result = table.add_script(script2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_script_table_remove_script() {
        let table = ScriptTable::with_default_obis();
        let script = ScriptDescriptor::new(1, vec![ScriptAction::simple(1)]);

        table.add_script(script).await.unwrap();
        assert_eq!(table.script_count().await, 1);

        table.remove_script(1).await.unwrap();
        assert_eq!(table.script_count().await, 0);
    }

    #[tokio::test]
    async fn test_script_table_remove_nonexistent() {
        let table = ScriptTable::with_default_obis();
        let result = table.remove_script(99).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_script_table_execute() {
        let table = ScriptTable::with_default_obis();
        let script = ScriptDescriptor::new(1, vec![
            ScriptAction::simple(1),
            ScriptAction::simple(2),
            ScriptAction::simple(3),
        ]);

        table.add_script(script).await.unwrap();
        let result = table.execute_script(1).await.unwrap();
        assert_eq!(result.script_id, 1);
        assert_eq!(result.actions_executed, 3);
    }

    #[tokio::test]
    async fn test_script_table_execute_nonexistent() {
        let table = ScriptTable::with_default_obis();
        let result = table.execute_script(99).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_script_table_method_invoke() {
        let table = ScriptTable::with_default_obis();
        let script = ScriptDescriptor::new(5, vec![ScriptAction::simple(1)]);

        table.add_script(script).await.unwrap();
        let result = table.invoke_method(1, Some(DataObject::Unsigned8(5)), None).await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_script_table_method_invalid_params() {
        let table = ScriptTable::with_default_obis();
        let result = table.invoke_method(1, Some(DataObject::Integer64(123)), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_script_table_get_logical_name() {
        let table = ScriptTable::with_default_obis();
        let result = table.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_script_table_read_only_count() {
        let table = ScriptTable::with_default_obis();
        let result = table.set_attribute(2, DataObject::Unsigned8(5), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_script_table_read_only_logical_name() {
        let table = ScriptTable::with_default_obis();
        let result = table.set_attribute(1, DataObject::OctetString(vec![0, 0, 10, 0, 0, 1]), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_script_table_update_script() {
        let table = ScriptTable::with_default_obis();
        let script = ScriptDescriptor::new(1, vec![ScriptAction::simple(1)]);
        table.add_script(script).await.unwrap();

        let updated = ScriptDescriptor::new(1, vec![
            ScriptAction::simple(1),
            ScriptAction::simple(2),
        ]);
        table.update_script(updated).await.unwrap();

        let retrieved = table.get_script(1).await.unwrap();
        assert_eq!(retrieved.actions.len(), 2);
    }

    #[tokio::test]
    async fn test_script_action_with_parameters() {
        let action = ScriptAction::new(1, vec![
            DataObject::Unsigned8(10),
            DataObject::Integer32(12345),
        ]);

        assert_eq!(action.action_type, 1);
        assert_eq!(action.parameters.len(), 2);
    }

    #[tokio::test]
    async fn test_script_descriptor_add_action() {
        let mut script = ScriptDescriptor::empty(1);
        assert_eq!(script.actions.len(), 0);

        script.add_action(ScriptAction::simple(1));
        script.add_action(ScriptAction::simple(2));
        assert_eq!(script.actions.len(), 2);
    }

    #[tokio::test]
    async fn test_script_table_invalid_attribute() {
        let table = ScriptTable::with_default_obis();
        let result = table.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_script_table_invalid_method() {
        let table = ScriptTable::with_default_obis();
        let result = table.invoke_method(99, None, None).await;
        assert!(result.is_err());
    }
}
