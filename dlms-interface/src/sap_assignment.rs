//! SAP Assignment interface class (Class ID: 17)
//!
//! The SAP Assignment interface class manages the assignment of Short Names (SAPs)
//! to COSEM objects for Short Name addressing mode.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: sap_list - List of SAP assignments
//!
//! # Methods
//!
//! - Method 1: sap_assign(short_name, class_id, obis_code) - Assign a SAP to an object
//! - Method 2: sap_release(short_name) - Release a SAP assignment

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Short Name (SAP) type
///
/// SAPs are 16-bit values used to identify COSEM objects in Short Name addressing mode.
pub type ShortName = u16;

/// SAP Assignment Entry
///
/// Represents a single SAP assignment mapping a short name to a COSEM object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SapAssignmentEntry {
    /// Short Name (SAP) value
    pub short_name: ShortName,
    /// Class ID of the COSEM object
    pub class_id: u16,
    /// OBIS code (logical name) of the COSEM object
    pub obis_code: ObisCode,
}

impl SapAssignmentEntry {
    /// Create a new SAP assignment
    pub fn new(short_name: ShortName, class_id: u16, obis_code: ObisCode) -> Self {
        Self {
            short_name,
            class_id,
            obis_code,
        }
    }

    /// Create from data object (structure)
    pub fn from_data_object(value: &DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Array(arr) if arr.len() >= 3 => {
                let short_name = match &arr[0] {
                    DataObject::Unsigned16(sn) => *sn,
                    DataObject::Unsigned8(sn) => *sn as u16,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Unsigned16 for short_name".to_string(),
                        ))
                    }
                };
                let class_id = match &arr[1] {
                    DataObject::Unsigned16(cid) => *cid,
                    DataObject::Unsigned8(cid) => *cid as u16,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Unsigned16 for class_id".to_string(),
                        ))
                    }
                };
                let obis_code = match &arr[2] {
                    DataObject::OctetString(bytes) if bytes.len() >= 6 => {
                        ObisCode::new(bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5])
                    }
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected OctetString for obis_code".to_string(),
                        ))
                    }
                };
                Ok(Self {
                    short_name,
                    class_id,
                    obis_code,
                })
            }
            _ => Err(DlmsError::InvalidData(
                "Expected Array for SapAssignment".to_string(),
            )),
        }
    }

    /// Convert to data object
    pub fn to_data_object(&self) -> DataObject {
        DataObject::Array(vec![
            DataObject::Unsigned16(self.short_name),
            DataObject::Unsigned16(self.class_id),
            DataObject::OctetString(self.obis_code.to_bytes().to_vec()),
        ])
    }
}

/// SAP Assignment interface class (Class ID: 17)
///
/// Default OBIS: 0-0:17.0.0.255
///
/// This class manages the assignment of Short Names (SAPs) to COSEM objects
/// for Short Name addressing mode.
#[derive(Debug, Clone)]
pub struct SapAssignment {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// SAP assignment list (short_name -> assignment mapping)
    sap_list: Arc<RwLock<HashMap<ShortName, SapAssignmentEntry>>>,
}

impl SapAssignment {
    /// Class ID for SAP Assignment
    pub const CLASS_ID: u16 = 17;

    /// Default OBIS code for SAP Assignment (0-0:17.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 17, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_SAP_LIST: u8 = 2;

    /// Method IDs
    pub const METHOD_SAP_ASSIGN: u8 = 1;
    pub const METHOD_SAP_RELEASE: u8 = 2;

    /// Create a new SAP Assignment object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            sap_list: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get the number of SAP assignments
    pub async fn len(&self) -> usize {
        self.sap_list.read().await.len()
    }

    /// Check if the SAP list is empty
    pub async fn is_empty(&self) -> bool {
        self.sap_list.read().await.is_empty()
    }

    /// Get all SAP assignments
    pub async fn assignments(&self) -> Vec<SapAssignmentEntry> {
        self.sap_list
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    /// Get assignment by short name
    pub async fn get(&self, short_name: ShortName) -> Option<SapAssignmentEntry> {
        self.sap_list.read().await.get(&short_name).cloned()
    }

    /// Find short name by class_id and obis_code
    pub async fn find_short_name(&self, class_id: u16, obis_code: ObisCode) -> Option<ShortName> {
        let list = self.sap_list.read().await;
        for (sn, assignment) in list.iter() {
            if assignment.class_id == class_id && assignment.obis_code == obis_code {
                return Some(*sn);
            }
        }
        None
    }

    /// Assign a SAP to an object
    ///
    /// # Arguments
    /// * `short_name` - Short Name to assign
    /// * `class_id` - Class ID of the object
    /// * `obis_code` - OBIS code of the object
    pub async fn assign(&self, short_name: ShortName, class_id: u16, obis_code: ObisCode) {
        let assignment = SapAssignmentEntry::new(short_name, class_id, obis_code);
        let mut list = self.sap_list.write().await;
        list.insert(short_name, assignment);
    }

    /// Release a SAP assignment
    ///
    /// # Arguments
    /// * `short_name` - Short Name to release
    ///
    /// # Returns
    /// `true` if the assignment was found and removed, `false` otherwise
    pub async fn release(&self, short_name: ShortName) -> bool {
        let mut list = self.sap_list.write().await;
        list.remove(&short_name).is_some()
    }

    /// Clear all SAP assignments
    pub async fn clear(&self) {
        self.sap_list.write().await.clear();
    }

    /// Check if a short name is assigned
    pub async fn is_assigned(&self, short_name: ShortName) -> bool {
        self.sap_list.read().await.contains_key(&short_name)
    }
}

#[async_trait]
impl CosemObject for SapAssignment {
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
            Self::ATTR_SAP_LIST => {
                let assignments = self.assignments().await;
                let data: Vec<DataObject> = assignments
                    .iter()
                    .map(|a| a.to_data_object())
                    .collect();
                Ok(DataObject::Array(data))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "SAP Assignment has no attribute {}",
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
            Self::ATTR_SAP_LIST => {
                match value {
                    DataObject::Array(arr) => {
                        self.clear().await;
                        for item in arr {
                            if let Ok(assignment) = SapAssignmentEntry::from_data_object(&item) {
                                let mut list = self.sap_list.write().await;
                                list.insert(assignment.short_name, assignment);
                            }
                        }
                        Ok(())
                    }
                    DataObject::Null => {
                        self.clear().await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Array for sap_list".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "SAP Assignment has no attribute {}",
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
            Self::METHOD_SAP_ASSIGN => {
                match parameters {
                    Some(DataObject::Array(arr)) if arr.len() >= 3 => {
                        let short_name = match &arr[0] {
                            DataObject::Unsigned16(sn) => *sn,
                            DataObject::Unsigned8(sn) => *sn as u16,
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Expected Unsigned16 for short_name".to_string(),
                                ))
                            }
                        };
                        let class_id = match &arr[1] {
                            DataObject::Unsigned16(cid) => *cid,
                            DataObject::Unsigned8(cid) => *cid as u16,
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Expected Unsigned16 for class_id".to_string(),
                                ))
                            }
                        };
                        let obis_code = match &arr[2] {
                            DataObject::OctetString(bytes) if bytes.len() >= 6 => {
                                ObisCode::new(bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5])
                            }
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Expected OctetString for obis_code".to_string(),
                                ))
                            }
                        };
                        self.assign(short_name, class_id, obis_code).await;
                        Ok(None)
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Method 1 expects Array parameter with short_name, class_id, obis_code".to_string(),
                    )),
                }
            }
            Self::METHOD_SAP_RELEASE => {
                match parameters {
                    Some(DataObject::Unsigned16(short_name)) => {
                        self.release(short_name).await;
                        Ok(None)
                    }
                    Some(DataObject::Unsigned8(short_name)) => {
                        self.release(short_name as u16).await;
                        Ok(None)
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Method 2 expects Unsigned16 parameter with short_name".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "SAP Assignment has no method {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sap_assignment_class_id() {
        let sap = SapAssignment::with_default_obis();
        assert_eq!(sap.class_id(), 17);
    }

    #[tokio::test]
    async fn test_sap_assignment_obis_code() {
        let sap = SapAssignment::with_default_obis();
        assert_eq!(sap.obis_code(), SapAssignment::default_obis());
    }

    #[tokio::test]
    async fn test_sap_assignment_new() {
        let obis = ObisCode::new(1, 1, 1, 1, 1, 1);
        let sap = SapAssignment::new(obis);
        assert_eq!(sap.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_sap_assignment_entry() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let entry = SapAssignmentEntry::new(100, 3, obis);
        assert_eq!(entry.short_name, 100);
        assert_eq!(entry.class_id, 3);
        assert_eq!(entry.obis_code, obis);
    }

    #[tokio::test]
    async fn test_sap_assignment_entry_to_data_object() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let entry = SapAssignmentEntry::new(100, 3, obis);
        let data = entry.to_data_object();

        match data {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 3);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_sap_assignment_entry_from_data_object() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let data = DataObject::Array(vec![
            DataObject::Unsigned16(100),
            DataObject::Unsigned16(3),
            DataObject::OctetString(obis.to_bytes().to_vec()),
        ]);

        let entry = SapAssignmentEntry::from_data_object(&data).unwrap();
        assert_eq!(entry.short_name, 100);
        assert_eq!(entry.class_id, 3);
        assert_eq!(entry.obis_code, obis);
    }

    #[tokio::test]
    async fn test_sap_assignment_initial_state() {
        let sap = SapAssignment::with_default_obis();
        assert!(sap.is_empty().await);
        assert_eq!(sap.len().await, 0);
    }

    #[tokio::test]
    async fn test_sap_assignment_assign() {
        let sap = SapAssignment::with_default_obis();
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);

        sap.assign(100, 3, obis).await;

        assert!(!sap.is_empty().await);
        assert_eq!(sap.len().await, 1);
        assert!(sap.is_assigned(100).await);
    }

    #[tokio::test]
    async fn test_sap_assignment_get() {
        let sap = SapAssignment::with_default_obis();
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);

        sap.assign(100, 3, obis).await;

        let assignment = sap.get(100).await;
        assert!(assignment.is_some());
        let a = assignment.unwrap();
        assert_eq!(a.short_name, 100);
        assert_eq!(a.class_id, 3);
    }

    #[tokio::test]
    async fn test_sap_assignment_find_short_name() {
        let sap = SapAssignment::with_default_obis();
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);

        sap.assign(100, 3, obis).await;

        let found = sap.find_short_name(3, obis).await;
        assert_eq!(found, Some(100));

        let not_found = sap.find_short_name(7, obis).await;
        assert_eq!(not_found, None);
    }

    #[tokio::test]
    async fn test_sap_assignment_release() {
        let sap = SapAssignment::with_default_obis();
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);

        sap.assign(100, 3, obis).await;
        assert_eq!(sap.len().await, 1);

        let released = sap.release(100).await;
        assert!(released);
        assert!(sap.is_empty().await);
    }

    #[tokio::test]
    async fn test_sap_assignment_release_nonexistent() {
        let sap = SapAssignment::with_default_obis();
        let released = sap.release(999).await;
        assert!(!released);
    }

    #[tokio::test]
    async fn test_sap_assignment_clear() {
        let sap = SapAssignment::with_default_obis();
        let obis1 = ObisCode::new(1, 1, 1, 8, 0, 255);
        let obis2 = ObisCode::new(1, 1, 1, 8, 1, 255);

        sap.assign(100, 3, obis1).await;
        sap.assign(101, 3, obis2).await;

        assert_eq!(sap.len().await, 2);

        sap.clear().await;
        assert!(sap.is_empty().await);
    }

    #[tokio::test]
    async fn test_sap_assignment_assignments() {
        let sap = SapAssignment::with_default_obis();
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);

        sap.assign(100, 3, obis).await;
        sap.assign(101, 7, obis).await;

        let assignments = sap.assignments().await;
        assert_eq!(assignments.len(), 2);
    }

    #[tokio::test]
    async fn test_sap_assignment_get_logical_name() {
        let sap = SapAssignment::with_default_obis();
        let result = sap.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_sap_assignment_get_sap_list() {
        let sap = SapAssignment::with_default_obis();
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);

        sap.assign(100, 3, obis).await;

        let result = sap.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 1);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_sap_assignment_set_sap_list() {
        let sap = SapAssignment::with_default_obis();
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);

        let entry_data = DataObject::Array(vec![
            DataObject::Unsigned16(100),
            DataObject::Unsigned16(3),
            DataObject::OctetString(obis.to_bytes().to_vec()),
        ]);

        sap.set_attribute(2, DataObject::Array(vec![entry_data]), None)
            .await
            .unwrap();

        assert_eq!(sap.len().await, 1);
        assert!(sap.is_assigned(100).await);
    }

    #[tokio::test]
    async fn test_sap_assignment_set_null_clears() {
        let sap = SapAssignment::with_default_obis();
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);

        sap.assign(100, 3, obis).await;
        assert_eq!(sap.len().await, 1);

        sap.set_attribute(2, DataObject::Null, None)
            .await
            .unwrap();

        assert!(sap.is_empty().await);
    }

    #[tokio::test]
    async fn test_sap_assignment_read_only_logical_name() {
        let sap = SapAssignment::with_default_obis();
        let result = sap
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 17, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sap_assignment_method_assign() {
        let sap = SapAssignment::with_default_obis();
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);

        let params = DataObject::Array(vec![
            DataObject::Unsigned16(100),
            DataObject::Unsigned16(3),
            DataObject::OctetString(obis.to_bytes().to_vec()),
        ]);

        sap.invoke_method(1, Some(params), None).await.unwrap();
        assert!(sap.is_assigned(100).await);
    }

    #[tokio::test]
    async fn test_sap_assignment_method_release() {
        let sap = SapAssignment::with_default_obis();
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);

        sap.assign(100, 3, obis).await;
        assert!(sap.is_assigned(100).await);

        sap.invoke_method(2, Some(DataObject::Unsigned16(100)), None)
            .await
            .unwrap();

        assert!(!sap.is_assigned(100).await);
    }

    #[tokio::test]
    async fn test_sap_assignment_invalid_attribute() {
        let sap = SapAssignment::with_default_obis();
        let result = sap.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sap_assignment_invalid_method() {
        let sap = SapAssignment::with_default_obis();
        let result = sap.invoke_method(99, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sap_assignment_entry_from_data_object_unsigned8() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let data = DataObject::Array(vec![
            DataObject::Unsigned8(100),
            DataObject::Unsigned8(3),
            DataObject::OctetString(obis.to_bytes().to_vec()),
        ]);

        let entry = SapAssignmentEntry::from_data_object(&data).unwrap();
        assert_eq!(entry.short_name, 100);
        assert_eq!(entry.class_id, 3);
    }

    #[tokio::test]
    async fn test_sap_assignment_is_assigned() {
        let sap = SapAssignment::with_default_obis();
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);

        assert!(!sap.is_assigned(100).await);

        sap.assign(100, 3, obis).await;
        assert!(sap.is_assigned(100).await);

        sap.release(100).await;
        assert!(!sap.is_assigned(100).await);
    }
}
