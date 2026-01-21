//! Status Mapping interface class (Class ID: 68)
//!
//! The Status Mapping interface class manages status value mappings
//! for converting between internal and external status codes.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: mapping_enabled - Whether status mapping is enabled
//! - Attribute 3: mapping_table - The status mapping table

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::CosemObject;

/// Status mapping entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusMappingEntry {
    /// Internal status code
    pub internal_status: u16,
    /// External status code
    pub external_status: u16,
}

impl StatusMappingEntry {
    /// Create a new status mapping entry
    pub fn new(internal_status: u16, external_status: u16) -> Self {
        Self {
            internal_status,
            external_status,
        }
    }
}

/// Status Mapping interface class (Class ID: 68)
///
/// Default OBIS: 0-0:68.0.0.255
///
/// This class manages status value mappings for converting between
/// internal and external status codes.
#[derive(Debug, Clone)]
pub struct StatusMapping {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Whether status mapping is enabled
    mapping_enabled: Arc<RwLock<bool>>,

    /// Internal to external mapping (internal -> external)
    internal_to_external: Arc<RwLock<HashMap<u16, u16>>>,

    /// External to internal mapping (external -> internal)
    external_to_internal: Arc<RwLock<HashMap<u16, u16>>>,
}

impl StatusMapping {
    /// Class ID for StatusMapping
    pub const CLASS_ID: u16 = 68;

    /// Default OBIS code for StatusMapping (0-0:68.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 68, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_MAPPING_ENABLED: u8 = 2;
    pub const ATTR_MAPPING_TABLE: u8 = 3;

    /// Create a new StatusMapping object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            mapping_enabled: Arc::new(RwLock::new(true)),
            internal_to_external: Arc::new(RwLock::new(HashMap::new())),
            external_to_internal: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get whether mapping is enabled
    pub async fn mapping_enabled(&self) -> bool {
        *self.mapping_enabled.read().await
    }

    /// Set whether mapping is enabled
    pub async fn set_mapping_enabled(&self, enabled: bool) {
        *self.mapping_enabled.write().await = enabled;
    }

    /// Add a mapping entry
    pub async fn add_mapping(&self, internal: u16, external: u16) {
        self.internal_to_external.write().await.insert(internal, external);
        self.external_to_internal.write().await.insert(external, internal);
    }

    /// Remove a mapping by internal status
    pub async fn remove_mapping_by_internal(&self, internal: u16) {
        if let Some(external) = self.internal_to_external.write().await.remove(&internal) {
            self.external_to_internal.write().await.remove(&external);
        }
    }

    /// Remove a mapping by external status
    pub async fn remove_mapping_by_external(&self, external: u16) {
        if let Some(internal) = self.external_to_internal.write().await.remove(&external) {
            self.internal_to_external.write().await.remove(&internal);
        }
    }

    /// Get external status from internal status
    pub async fn to_external(&self, internal: u16) -> Option<u16> {
        self.internal_to_external.read().await.get(&internal).copied()
    }

    /// Get internal status from external status
    pub async fn to_internal(&self, external: u16) -> Option<u16> {
        self.external_to_internal.read().await.get(&external).copied()
    }

    /// Convert internal status to external (with pass-through if disabled)
    pub async fn convert_to_external(&self, internal: u16) -> u16 {
        if self.mapping_enabled().await {
            self.to_external(internal).await.unwrap_or(internal)
        } else {
            internal
        }
    }

    /// Convert external status to internal (with pass-through if disabled)
    pub async fn convert_to_internal(&self, external: u16) -> u16 {
        if self.mapping_enabled().await {
            self.to_internal(external).await.unwrap_or(external)
        } else {
            external
        }
    }

    /// Get all mapping entries
    pub async fn get_mappings(&self) -> Vec<StatusMappingEntry> {
        self.internal_to_external
            .read()
            .await
            .iter()
            .map(|(&internal, &external)| StatusMappingEntry::new(internal, external))
            .collect()
    }

    /// Get the number of mappings
    pub async fn mapping_count(&self) -> usize {
        self.internal_to_external.read().await.len()
    }

    /// Clear all mappings
    pub async fn clear_mappings(&self) {
        self.internal_to_external.write().await.clear();
        self.external_to_internal.write().await.clear();
    }

    /// Check if a mapping exists for the internal status
    pub async fn has_internal_mapping(&self, internal: u16) -> bool {
        self.internal_to_external.read().await.contains_key(&internal)
    }

    /// Check if a mapping exists for the external status
    pub async fn has_external_mapping(&self, external: u16) -> bool {
        self.external_to_internal.read().await.contains_key(&external)
    }

    /// Set multiple mappings at once
    pub async fn set_mappings(&self, mappings: Vec<StatusMappingEntry>) {
        let mut int_to_ext = self.internal_to_external.write().await;
        let mut ext_to_int = self.external_to_internal.write().await;

        int_to_ext.clear();
        ext_to_int.clear();

        for entry in mappings {
            int_to_ext.insert(entry.internal_status, entry.external_status);
            ext_to_int.insert(entry.external_status, entry.internal_status);
        }
    }

    /// Get mapping table as a DataObject array
    async fn mapping_table_as_data(&self) -> DataObject {
        let mappings = self.get_mappings().await;
        let entries: Vec<DataObject> = mappings
            .iter()
            .map(|m| {
                DataObject::Array(vec![
                    DataObject::Unsigned16(m.internal_status),
                    DataObject::Unsigned16(m.external_status),
                ])
            })
            .collect();

        DataObject::Array(entries)
    }
}

#[async_trait]
impl CosemObject for StatusMapping {
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
            Self::ATTR_MAPPING_ENABLED => {
                Ok(DataObject::Boolean(self.mapping_enabled().await))
            }
            Self::ATTR_MAPPING_TABLE => {
                Ok(self.mapping_table_as_data().await)
            }
            _ => Err(DlmsError::InvalidData(format!(
                "StatusMapping has no attribute {}",
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
            Self::ATTR_MAPPING_ENABLED => {
                match value {
                    DataObject::Boolean(enabled) => {
                        self.set_mapping_enabled(enabled).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Boolean for mapping_enabled".to_string(),
                    )),
                }
            }
            Self::ATTR_MAPPING_TABLE => {
                match value {
                    DataObject::Array(entries) => {
                        let mut mappings = Vec::new();
                        for entry in entries {
                            if let DataObject::Array(pair) = entry {
                                if pair.len() >= 2 {
                                    let internal = match &pair[0] {
                                        DataObject::Unsigned16(v) => *v,
                                        DataObject::Unsigned8(v) => *v as u16,
                                        _ => return Err(DlmsError::InvalidData(
                                            "Expected Unsigned16 for internal status".to_string(),
                                        )),
                                    };
                                    let external = match &pair[1] {
                                        DataObject::Unsigned16(v) => *v,
                                        DataObject::Unsigned8(v) => *v as u16,
                                        _ => return Err(DlmsError::InvalidData(
                                            "Expected Unsigned16 for external status".to_string(),
                                        )),
                                    };
                                    mappings.push(StatusMappingEntry::new(internal, external));
                                }
                            }
                        }
                        self.set_mappings(mappings).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Array for mapping_table".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "StatusMapping has no attribute {}",
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
            "StatusMapping has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_status_mapping_class_id() {
        let mapping = StatusMapping::with_default_obis();
        assert_eq!(mapping.class_id(), 68);
    }

    #[tokio::test]
    async fn test_status_mapping_obis_code() {
        let mapping = StatusMapping::with_default_obis();
        assert_eq!(mapping.obis_code(), StatusMapping::default_obis());
    }

    #[tokio::test]
    async fn test_status_mapping_initial_state() {
        let mapping = StatusMapping::with_default_obis();
        assert!(mapping.mapping_enabled().await);
        assert_eq!(mapping.mapping_count().await, 0);
    }

    #[tokio::test]
    async fn test_status_mapping_add_mapping() {
        let mapping = StatusMapping::with_default_obis();
        mapping.add_mapping(1, 10).await;
        mapping.add_mapping(2, 20).await;

        assert_eq!(mapping.mapping_count().await, 2);
        assert!(mapping.has_internal_mapping(1).await);
        assert!(mapping.has_external_mapping(10).await);
    }

    #[tokio::test]
    async fn test_status_mapping_to_external() {
        let mapping = StatusMapping::with_default_obis();
        mapping.add_mapping(1, 10).await;
        mapping.add_mapping(2, 20).await;

        assert_eq!(mapping.to_external(1).await, Some(10));
        assert_eq!(mapping.to_external(2).await, Some(20));
        assert_eq!(mapping.to_external(3).await, None); // No mapping
    }

    #[tokio::test]
    async fn test_status_mapping_to_internal() {
        let mapping = StatusMapping::with_default_obis();
        mapping.add_mapping(1, 10).await;
        mapping.add_mapping(2, 20).await;

        assert_eq!(mapping.to_internal(10).await, Some(1));
        assert_eq!(mapping.to_internal(20).await, Some(2));
        assert_eq!(mapping.to_internal(30).await, None); // No mapping
    }

    #[tokio::test]
    async fn test_status_mapping_convert_to_external_enabled() {
        let mapping = StatusMapping::with_default_obis();
        mapping.add_mapping(1, 10).await;

        assert_eq!(mapping.convert_to_external(1).await, 10);
        assert_eq!(mapping.convert_to_external(5).await, 5); // Pass-through
    }

    #[tokio::test]
    async fn test_status_mapping_convert_to_external_disabled() {
        let mapping = StatusMapping::with_default_obis();
        mapping.add_mapping(1, 10).await;
        mapping.set_mapping_enabled(false).await;

        // Mapping is disabled, should pass through
        assert_eq!(mapping.convert_to_external(1).await, 1);
    }

    #[tokio::test]
    async fn test_status_mapping_convert_to_internal_enabled() {
        let mapping = StatusMapping::with_default_obis();
        mapping.add_mapping(1, 10).await;

        assert_eq!(mapping.convert_to_internal(10).await, 1);
        assert_eq!(mapping.convert_to_internal(50).await, 50); // Pass-through
    }

    #[tokio::test]
    async fn test_status_mapping_convert_to_internal_disabled() {
        let mapping = StatusMapping::with_default_obis();
        mapping.add_mapping(1, 10).await;
        mapping.set_mapping_enabled(false).await;

        // Mapping is disabled, should pass through
        assert_eq!(mapping.convert_to_internal(10).await, 10);
    }

    #[tokio::test]
    async fn test_status_mapping_remove_by_internal() {
        let mapping = StatusMapping::with_default_obis();
        mapping.add_mapping(1, 10).await;
        mapping.add_mapping(2, 20).await;

        mapping.remove_mapping_by_internal(1).await;

        assert_eq!(mapping.mapping_count().await, 1);
        assert!(!mapping.has_internal_mapping(1).await);
        assert!(!mapping.has_external_mapping(10).await);
    }

    #[tokio::test]
    async fn test_status_mapping_remove_by_external() {
        let mapping = StatusMapping::with_default_obis();
        mapping.add_mapping(1, 10).await;
        mapping.add_mapping(2, 20).await;

        mapping.remove_mapping_by_external(20).await;

        assert_eq!(mapping.mapping_count().await, 1);
        assert!(!mapping.has_internal_mapping(2).await);
        assert!(!mapping.has_external_mapping(20).await);
    }

    #[tokio::test]
    async fn test_status_mapping_get_mappings() {
        let mapping = StatusMapping::with_default_obis();
        mapping.add_mapping(1, 10).await;
        mapping.add_mapping(2, 20).await;
        mapping.add_mapping(3, 30).await;

        let mappings = mapping.get_mappings().await;
        assert_eq!(mappings.len(), 3);
        assert!(mappings.contains(&StatusMappingEntry::new(1, 10)));
        assert!(mappings.contains(&StatusMappingEntry::new(2, 20)));
        assert!(mappings.contains(&StatusMappingEntry::new(3, 30)));
    }

    #[tokio::test]
    async fn test_status_mapping_clear_mappings() {
        let mapping = StatusMapping::with_default_obis();
        mapping.add_mapping(1, 10).await;
        mapping.add_mapping(2, 20).await;

        mapping.clear_mappings().await;

        assert_eq!(mapping.mapping_count().await, 0);
    }

    #[tokio::test]
    async fn test_status_mapping_set_mappings() {
        let mapping = StatusMapping::with_default_obis();
        let mappings = vec![
            StatusMappingEntry::new(1, 10),
            StatusMappingEntry::new(2, 20),
            StatusMappingEntry::new(3, 30),
        ];

        mapping.set_mappings(mappings).await;

        assert_eq!(mapping.mapping_count().await, 3);
        assert_eq!(mapping.to_external(1).await, Some(10));
        assert_eq!(mapping.to_external(2).await, Some(20));
        assert_eq!(mapping.to_external(3).await, Some(30));
    }

    #[tokio::test]
    async fn test_status_mapping_get_attributes() {
        let mapping = StatusMapping::with_default_obis();

        // Test mapping_enabled
        let result = mapping.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Boolean(enabled) => assert!(enabled),
            _ => panic!("Expected Boolean"),
        }

        // Test mapping_table (should be empty array)
        let result = mapping.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Array(entries) => assert!(entries.is_empty()),
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_status_mapping_set_attributes() {
        let mapping = StatusMapping::with_default_obis();

        mapping.set_attribute(2, DataObject::Boolean(false), None)
            .await
            .unwrap();
        assert!(!mapping.mapping_enabled().await);
    }

    #[tokio::test]
    async fn test_status_mapping_set_mapping_table() {
        let mapping = StatusMapping::with_default_obis();

        let table = DataObject::Array(vec![
            DataObject::Array(vec![
                DataObject::Unsigned16(1),
                DataObject::Unsigned16(10),
            ]),
            DataObject::Array(vec![
                DataObject::Unsigned16(2),
                DataObject::Unsigned16(20),
            ]),
        ]);

        mapping.set_attribute(3, table, None).await.unwrap();

        assert_eq!(mapping.mapping_count().await, 2);
        assert_eq!(mapping.to_external(1).await, Some(10));
        assert_eq!(mapping.to_external(2).await, Some(20));
    }

    #[tokio::test]
    async fn test_status_mapping_read_only_logical_name() {
        let mapping = StatusMapping::with_default_obis();
        let result = mapping
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 68, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_status_mapping_invalid_attribute() {
        let mapping = StatusMapping::with_default_obis();
        let result = mapping.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_status_mapping_invalid_method() {
        let mapping = StatusMapping::with_default_obis();
        let result = mapping.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_status_mapping_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 68, 0, 0, 1);
        let mapping = StatusMapping::new(obis);
        assert_eq!(mapping.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_status_mapping_entry_new() {
        let entry = StatusMappingEntry::new(100, 200);
        assert_eq!(entry.internal_status, 100);
        assert_eq!(entry.external_status, 200);
    }
}
