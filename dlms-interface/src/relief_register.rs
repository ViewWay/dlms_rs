//! Relief Register interface class (Class ID: 87)
//!
//! The Relief Register interface class manages relief/alarm threshold registers
//! that trigger when values exceed certain limits.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: current_value - Current value of the register
//! - Attribute 3: normal_value - Normal threshold value
//! - Attribute 4: relief_value - Relief threshold value
//! - Attribute 5: status - Current status (normal/relief)

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Relief Register Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ReliefStatus {
    /// Normal operation (within normal threshold)
    Normal = 0,
    /// Warning (exceeded normal threshold but below relief threshold)
    Warning = 1,
    /// Relief (exceeded relief threshold)
    Relief = 2,
}

impl ReliefStatus {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Normal,
            1 => Self::Warning,
            2 => Self::Relief,
            _ => Self::Normal,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if status indicates relief needed
    pub fn is_relief(self) -> bool {
        matches!(self, Self::Relief)
    }

    /// Check if status is normal
    pub fn is_normal(self) -> bool {
        matches!(self, Self::Normal)
    }
}

/// Relief Register interface class (Class ID: 87)
///
/// Default OBIS: 0-0:87.0.0.255
///
/// This class manages relief/alarm threshold registers that trigger
/// when values exceed certain limits.
#[derive(Debug, Clone)]
pub struct ReliefRegister {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Current value of the register
    current_value: Arc<RwLock<i64>>,

    /// Normal threshold value (warning triggers above this)
    normal_value: Arc<RwLock<i64>>,

    /// Relief threshold value (relief triggers above this)
    relief_value: Arc<RwLock<i64>>,

    /// Current status
    status: Arc<RwLock<ReliefStatus>>,
}

impl ReliefRegister {
    /// Class ID for ReliefRegister
    pub const CLASS_ID: u16 = 87;

    /// Default OBIS code for ReliefRegister (0-0:87.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 87, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_CURRENT_VALUE: u8 = 2;
    pub const ATTR_NORMAL_VALUE: u8 = 3;
    pub const ATTR_RELIEF_VALUE: u8 = 4;
    pub const ATTR_STATUS: u8 = 5;

    /// Create a new ReliefRegister object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            current_value: Arc::new(RwLock::new(0)),
            normal_value: Arc::new(RwLock::new(100)),
            relief_value: Arc::new(RwLock::new(150)),
            status: Arc::new(RwLock::new(ReliefStatus::Normal)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Create with specific threshold values
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `normal_value` - Normal threshold value
    /// * `relief_value` - Relief threshold value
    pub fn with_thresholds(logical_name: ObisCode, normal_value: i64, relief_value: i64) -> Self {
        Self {
            logical_name,
            current_value: Arc::new(RwLock::new(0)),
            normal_value: Arc::new(RwLock::new(normal_value)),
            relief_value: Arc::new(RwLock::new(relief_value)),
            status: Arc::new(RwLock::new(ReliefStatus::Normal)),
        }
    }

    /// Get the current value
    pub async fn current_value(&self) -> i64 {
        *self.current_value.read().await
    }

    /// Set the current value (updates status automatically)
    pub async fn set_current_value(&self, value: i64) {
        *self.current_value.write().await = value;
        self.update_status().await;
    }

    /// Get the normal threshold value
    pub async fn normal_value(&self) -> i64 {
        *self.normal_value.read().await
    }

    /// Set the normal threshold value
    pub async fn set_normal_value(&self, value: i64) {
        *self.normal_value.write().await = value;
        self.update_status().await;
    }

    /// Get the relief threshold value
    pub async fn relief_value(&self) -> i64 {
        *self.relief_value.read().await
    }

    /// Set the relief threshold value
    pub async fn set_relief_value(&self, value: i64) {
        *self.relief_value.write().await = value;
        self.update_status().await;
    }

    /// Get the current status
    pub async fn status(&self) -> ReliefStatus {
        *self.status.read().await
    }

    /// Update status based on current value and thresholds
    async fn update_status(&self) {
        let current = self.current_value().await;
        let normal = self.normal_value().await;
        let relief = self.relief_value().await;

        let new_status = if current >= relief {
            ReliefStatus::Relief
        } else if current >= normal {
            ReliefStatus::Warning
        } else {
            ReliefStatus::Normal
        };

        *self.status.write().await = new_status;
    }

    /// Check if relief is active
    pub async fn is_relief_active(&self) -> bool {
        self.status().await.is_relief()
    }

    /// Check if status is normal
    pub async fn is_normal(&self) -> bool {
        self.status().await.is_normal()
    }

    /// Check if warning is active
    pub async fn is_warning(&self) -> bool {
        matches!(self.status().await, ReliefStatus::Warning)
    }

    /// Get the distance to normal threshold (positive if below threshold)
    pub async fn distance_to_normal(&self) -> i64 {
        self.normal_value().await - self.current_value().await
    }

    /// Get the distance to relief threshold (positive if below threshold)
    pub async fn distance_to_relief(&self) -> i64 {
        self.relief_value().await - self.current_value().await
    }

    /// Get the percentage of the relief threshold reached
    pub async fn relief_percentage(&self) -> f64 {
        let current = self.current_value().await as f64;
        let relief = self.relief_value().await as f64;
        if relief != 0.0 {
            (current / relief * 100.0).min(100.0)
        } else {
            0.0
        }
    }
}

#[async_trait]
impl CosemObject for ReliefRegister {
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
            Self::ATTR_CURRENT_VALUE => {
                Ok(DataObject::Integer64(self.current_value().await))
            }
            Self::ATTR_NORMAL_VALUE => {
                Ok(DataObject::Integer64(self.normal_value().await))
            }
            Self::ATTR_RELIEF_VALUE => {
                Ok(DataObject::Integer64(self.relief_value().await))
            }
            Self::ATTR_STATUS => {
                Ok(DataObject::Enumerate(self.status().await.to_u8()))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "ReliefRegister has no attribute {}",
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
            Self::ATTR_CURRENT_VALUE => {
                match value {
                    DataObject::Integer64(v) => {
                        self.set_current_value(v).await;
                        Ok(())
                    }
                    DataObject::Unsigned32(v) => {
                        self.set_current_value(v as i64).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 for current_value".to_string(),
                    )),
                }
            }
            Self::ATTR_NORMAL_VALUE => {
                match value {
                    DataObject::Integer64(v) => {
                        self.set_normal_value(v).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 for normal_value".to_string(),
                    )),
                }
            }
            Self::ATTR_RELIEF_VALUE => {
                match value {
                    DataObject::Integer64(v) => {
                        self.set_relief_value(v).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 for relief_value".to_string(),
                    )),
                }
            }
            Self::ATTR_STATUS => {
                Err(DlmsError::AccessDenied(
                    "Attribute 5 (status) is read-only".to_string(),
                ))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "ReliefRegister has no attribute {}",
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
            "ReliefRegister has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_relief_register_class_id() {
        let reg = ReliefRegister::with_default_obis();
        assert_eq!(reg.class_id(), 87);
    }

    #[tokio::test]
    async fn test_relief_register_obis_code() {
        let reg = ReliefRegister::with_default_obis();
        assert_eq!(reg.obis_code(), ReliefRegister::default_obis());
    }

    #[tokio::test]
    async fn test_relief_status_from_u8() {
        assert_eq!(ReliefStatus::from_u8(0), ReliefStatus::Normal);
        assert_eq!(ReliefStatus::from_u8(1), ReliefStatus::Warning);
        assert_eq!(ReliefStatus::from_u8(2), ReliefStatus::Relief);
    }

    #[tokio::test]
    async fn test_relief_status_is_relief() {
        assert!(!ReliefStatus::Normal.is_relief());
        assert!(!ReliefStatus::Warning.is_relief());
        assert!(ReliefStatus::Relief.is_relief());
    }

    #[tokio::test]
    async fn test_relief_status_is_normal() {
        assert!(ReliefStatus::Normal.is_normal());
        assert!(!ReliefStatus::Warning.is_normal());
        assert!(!ReliefStatus::Relief.is_normal());
    }

    #[tokio::test]
    async fn test_relief_register_initial_state() {
        let reg = ReliefRegister::with_default_obis();
        assert_eq!(reg.current_value().await, 0);
        assert_eq!(reg.normal_value().await, 100);
        assert_eq!(reg.relief_value().await, 150);
        assert_eq!(reg.status().await, ReliefStatus::Normal);
    }

    #[tokio::test]
    async fn test_relief_register_set_current_value_normal() {
        let reg = ReliefRegister::with_default_obis();
        reg.set_current_value(50).await;
        assert_eq!(reg.status().await, ReliefStatus::Normal);
        assert!(reg.is_normal().await);
    }

    #[tokio::test]
    async fn test_relief_register_set_current_value_warning() {
        let reg = ReliefRegister::with_default_obis();
        reg.set_current_value(120).await;
        assert_eq!(reg.status().await, ReliefStatus::Warning);
        assert!(reg.is_warning().await);
    }

    #[tokio::test]
    async fn test_relief_register_set_current_value_relief() {
        let reg = ReliefRegister::with_default_obis();
        reg.set_current_value(160).await;
        assert_eq!(reg.status().await, ReliefStatus::Relief);
        assert!(reg.is_relief_active().await);
    }

    #[tokio::test]
    async fn test_relief_register_distance_to_normal() {
        let reg = ReliefRegister::with_default_obis();
        reg.set_current_value(80).await;
        assert_eq!(reg.distance_to_normal().await, 20);
    }

    #[tokio::test]
    async fn test_relief_register_distance_to_relief() {
        let reg = ReliefRegister::with_default_obis();
        reg.set_current_value(120).await;
        assert_eq!(reg.distance_to_relief().await, 30);
    }

    #[tokio::test]
    async fn test_relief_register_relief_percentage() {
        let reg = ReliefRegister::with_default_obis();
        reg.set_current_value(75).await;
        assert!((reg.relief_percentage().await - 50.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_relief_register_with_thresholds() {
        let reg = ReliefRegister::with_thresholds(ObisCode::new(0, 0, 87, 0, 0, 255), 200, 300);
        assert_eq!(reg.normal_value().await, 200);
        assert_eq!(reg.relief_value().await, 300);

        reg.set_current_value(250).await;
        assert_eq!(reg.status().await, ReliefStatus::Warning);

        reg.set_current_value(350).await;
        assert_eq!(reg.status().await, ReliefStatus::Relief);
    }

    #[tokio::test]
    async fn test_relief_register_status_auto_update() {
        let reg = ReliefRegister::with_default_obis();

        assert_eq!(reg.status().await, ReliefStatus::Normal);

        reg.set_current_value(120).await;
        assert_eq!(reg.status().await, ReliefStatus::Warning);

        reg.set_current_value(160).await;
        assert_eq!(reg.status().await, ReliefStatus::Relief);

        reg.set_current_value(50).await;
        assert_eq!(reg.status().await, ReliefStatus::Normal);
    }

    #[tokio::test]
    async fn test_relief_register_threshold_change_updates_status() {
        let reg = ReliefRegister::with_default_obis();
        reg.set_current_value(120).await;
        assert_eq!(reg.status().await, ReliefStatus::Warning);

        // Lower normal threshold below current value -> should trigger relief
        reg.set_normal_value(100).await;
        assert_eq!(reg.status().await, ReliefStatus::Warning);

        // Lower relief threshold below current value -> should trigger relief
        reg.set_relief_value(110).await;
        assert_eq!(reg.status().await, ReliefStatus::Relief);
    }

    #[tokio::test]
    async fn test_relief_register_get_attributes() {
        let reg = ReliefRegister::with_default_obis();

        // Test current_value
        let result = reg.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Integer64(v) => assert_eq!(v, 0),
            _ => panic!("Expected Integer64"),
        }

        // Test normal_value
        let result = reg.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Integer64(v) => assert_eq!(v, 100),
            _ => panic!("Expected Integer64"),
        }

        // Test status
        let result = reg.get_attribute(5, None).await.unwrap();
        match result {
            DataObject::Enumerate(s) => assert_eq!(s, 0), // Normal
            _ => panic!("Expected Enumerate"),
        }
    }

    #[tokio::test]
    async fn test_relief_register_set_attributes() {
        let reg = ReliefRegister::with_default_obis();

        reg.set_attribute(2, DataObject::Integer64(50), None)
            .await
            .unwrap();
        assert_eq!(reg.current_value().await, 50);

        reg.set_attribute(3, DataObject::Integer64(80), None)
            .await
            .unwrap();
        assert_eq!(reg.normal_value().await, 80);
    }

    #[tokio::test]
    async fn test_relief_register_read_only_logical_name() {
        let reg = ReliefRegister::with_default_obis();
        let result = reg
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 87, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_relief_register_read_only_status() {
        let reg = ReliefRegister::with_default_obis();
        let result = reg.set_attribute(5, DataObject::Enumerate(1), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_relief_register_invalid_attribute() {
        let reg = ReliefRegister::with_default_obis();
        let result = reg.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_relief_register_invalid_method() {
        let reg = ReliefRegister::with_default_obis();
        let result = reg.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_relief_register_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 87, 0, 0, 1);
        let reg = ReliefRegister::new(obis);
        assert_eq!(reg.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_relief_register_zero_division_percentage() {
        let reg = ReliefRegister::with_thresholds(ObisCode::new(0, 0, 87, 0, 0, 255), 100, 0);
        reg.set_current_value(50).await;
        assert_eq!(reg.relief_percentage().await, 0.0);
    }

    #[tokio::test]
    async fn test_relief_register_capped_percentage() {
        let reg = ReliefRegister::with_default_obis();
        reg.set_current_value(200).await;
        assert_eq!(reg.relief_percentage().await, 100.0);
    }
}
