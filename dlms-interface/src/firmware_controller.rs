//! Firmware Controller interface class (Class ID: 83)
//!
//! The Firmware Controller interface class manages firmware updates and versioning.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: firmware_version - Current firmware version
//! - Attribute 3: firmware_signature - Firmware signature/identifier
//! - Attribute 4: update_status - Status of the last firmware update
//! - Attribute 5: update_time - Timestamp of the last firmware update

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Firmware Update Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FirmwareUpdateStatus {
    /// No update performed
    NoUpdate = 0,
    /// Update in progress
    InProgress = 1,
    /// Update successful
    Success = 2,
    /// Update failed
    Failed = 3,
    /// Update scheduled
    Scheduled = 4,
    /// Update verification failed
    VerificationFailed = 5,
}

impl FirmwareUpdateStatus {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::NoUpdate,
            1 => Self::InProgress,
            2 => Self::Success,
            3 => Self::Failed,
            4 => Self::Scheduled,
            _ => Self::NoUpdate,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if update completed successfully
    pub fn is_success(self) -> bool {
        matches!(self, Self::Success)
    }

    /// Check if update failed
    pub fn is_failure(self) -> bool {
        matches!(self, Self::Failed | Self::VerificationFailed)
    }

    /// Check if update is in progress
    pub fn is_in_progress(self) -> bool {
        matches!(self, Self::InProgress)
    }
}

/// Firmware Controller interface class (Class ID: 83)
///
/// Default OBIS: 0-0:83.0.0.255
///
/// This class manages firmware updates and versioning.
#[derive(Debug, Clone)]
pub struct FirmwareController {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Current firmware version
    firmware_version: Arc<RwLock<String>>,

    /// Firmware signature/identifier
    firmware_signature: Arc<RwLock<String>>,

    /// Status of the last firmware update
    update_status: Arc<RwLock<FirmwareUpdateStatus>>,

    /// Timestamp of the last firmware update (Unix timestamp)
    update_time: Arc<RwLock<Option<i64>>>,
}

impl FirmwareController {
    /// Class ID for FirmwareController
    pub const CLASS_ID: u16 = 83;

    /// Default OBIS code for FirmwareController (0-0:83.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 83, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_FIRMWARE_VERSION: u8 = 2;
    pub const ATTR_FIRMWARE_SIGNATURE: u8 = 3;
    pub const ATTR_UPDATE_STATUS: u8 = 4;
    pub const ATTR_UPDATE_TIME: u8 = 5;

    /// Create a new FirmwareController object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            firmware_version: Arc::new(RwLock::new(String::from("1.0.0"))),
            firmware_signature: Arc::new(RwLock::new(String::new())),
            update_status: Arc::new(RwLock::new(FirmwareUpdateStatus::NoUpdate)),
            update_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Create with specific firmware version
    pub fn with_version(logical_name: ObisCode, version: String) -> Self {
        Self {
            logical_name,
            firmware_version: Arc::new(RwLock::new(version)),
            firmware_signature: Arc::new(RwLock::new(String::new())),
            update_status: Arc::new(RwLock::new(FirmwareUpdateStatus::NoUpdate)),
            update_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the firmware version
    pub async fn firmware_version(&self) -> String {
        self.firmware_version.read().await.clone()
    }

    /// Set the firmware version
    pub async fn set_firmware_version(&self, version: String) {
        *self.firmware_version.write().await = version;
    }

    /// Get the firmware signature
    pub async fn firmware_signature(&self) -> String {
        self.firmware_signature.read().await.clone()
    }

    /// Set the firmware signature
    pub async fn set_firmware_signature(&self, signature: String) {
        *self.firmware_signature.write().await = signature;
    }

    /// Get the update status
    pub async fn update_status(&self) -> FirmwareUpdateStatus {
        *self.update_status.read().await
    }

    /// Set the update status
    pub async fn set_update_status(&self, status: FirmwareUpdateStatus) {
        *self.update_status.write().await = status;
    }

    /// Get the update time
    pub async fn update_time(&self) -> Option<i64> {
        *self.update_time.read().await
    }

    /// Set the update time
    pub async fn set_update_time(&self, time: Option<i64>) {
        *self.update_time.write().await = time;
    }

    /// Check if update was successful
    pub async fn is_update_successful(&self) -> bool {
        self.update_status().await.is_success()
    }

    /// Check if update failed
    pub async fn is_update_failed(&self) -> bool {
        self.update_status().await.is_failure()
    }

    /// Check if update is in progress
    pub async fn is_update_in_progress(&self) -> bool {
        self.update_status().await.is_in_progress()
    }

    /// Record a successful firmware update
    pub async fn record_successful_update(&self, version: String, signature: String) {
        self.set_firmware_version(version).await;
        self.set_firmware_signature(signature).await;
        self.set_update_status(FirmwareUpdateStatus::Success).await;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.set_update_time(Some(now)).await;
    }

    /// Record a failed firmware update
    pub async fn record_failed_update(&self) {
        self.set_update_status(FirmwareUpdateStatus::Failed).await;
    }

    /// Start a firmware update
    pub async fn start_update(&self) {
        self.set_update_status(FirmwareUpdateStatus::InProgress).await;
    }

    /// Get firmware version as a semver tuple (major, minor, patch)
    pub async fn version_tuple(&self) -> (u32, u32, u32) {
        let version = self.firmware_version().await;
        let parts: Vec<&str> = version.split('.').collect();

        let major = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

        (major, minor, patch)
    }

    /// Compare versions
    /// Returns: 1 if current > other, 0 if equal, -1 if current < other
    pub async fn compare_version(&self, other: &str) -> i8 {
        let current = self.firmware_version().await;
        let current_parts: Vec<u32> = current
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        let other_parts: Vec<u32> = other
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        let max_len = current_parts.len().max(other_parts.len());

        for i in 0..max_len {
            let c = current_parts.get(i).unwrap_or(&0);
            let o = other_parts.get(i).unwrap_or(&0);

            if c > o {
                return 1;
            } else if c < o {
                return -1;
            }
        }

        0
    }
}

#[async_trait]
impl CosemObject for FirmwareController {
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
            Self::ATTR_FIRMWARE_VERSION => {
                Ok(DataObject::OctetString(self.firmware_version().await.into_bytes()))
            }
            Self::ATTR_FIRMWARE_SIGNATURE => {
                Ok(DataObject::OctetString(self.firmware_signature().await.into_bytes()))
            }
            Self::ATTR_UPDATE_STATUS => {
                Ok(DataObject::Enumerate(self.update_status().await.to_u8()))
            }
            Self::ATTR_UPDATE_TIME => {
                match self.update_time().await {
                    Some(timestamp) => Ok(DataObject::Integer64(timestamp)),
                    None => Ok(DataObject::Null),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "FirmwareController has no attribute {}",
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
            Self::ATTR_FIRMWARE_VERSION => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_firmware_version(String::from_utf8_lossy(&bytes).to_string()).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for firmware_version".to_string(),
                    )),
                }
            }
            Self::ATTR_FIRMWARE_SIGNATURE => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_firmware_signature(String::from_utf8_lossy(&bytes).to_string()).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for firmware_signature".to_string(),
                    )),
                }
            }
            Self::ATTR_UPDATE_STATUS => {
                match value {
                    DataObject::Enumerate(status) => {
                        self.set_update_status(FirmwareUpdateStatus::from_u8(status)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for update_status".to_string(),
                    )),
                }
            }
            Self::ATTR_UPDATE_TIME => {
                match value {
                    DataObject::Integer64(timestamp) => {
                        self.set_update_time(Some(timestamp)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_update_time(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 or Null for update_time".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "FirmwareController has no attribute {}",
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
            "FirmwareController has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_firmware_controller_class_id() {
        let fc = FirmwareController::with_default_obis();
        assert_eq!(fc.class_id(), 83);
    }

    #[tokio::test]
    async fn test_firmware_controller_obis_code() {
        let fc = FirmwareController::with_default_obis();
        assert_eq!(fc.obis_code(), FirmwareController::default_obis());
    }

    #[tokio::test]
    async fn test_firmware_update_status_from_u8() {
        assert_eq!(FirmwareUpdateStatus::from_u8(0), FirmwareUpdateStatus::NoUpdate);
        assert_eq!(FirmwareUpdateStatus::from_u8(1), FirmwareUpdateStatus::InProgress);
        assert_eq!(FirmwareUpdateStatus::from_u8(2), FirmwareUpdateStatus::Success);
        assert_eq!(FirmwareUpdateStatus::from_u8(3), FirmwareUpdateStatus::Failed);
        assert_eq!(FirmwareUpdateStatus::from_u8(4), FirmwareUpdateStatus::Scheduled);
    }

    #[tokio::test]
    async fn test_firmware_update_status_is_success() {
        assert!(FirmwareUpdateStatus::Success.is_success());
        assert!(!FirmwareUpdateStatus::Failed.is_success());
        assert!(!FirmwareUpdateStatus::InProgress.is_success());
    }

    #[tokio::test]
    async fn test_firmware_update_status_is_failure() {
        assert!(FirmwareUpdateStatus::Failed.is_failure());
        assert!(FirmwareUpdateStatus::VerificationFailed.is_failure());
        assert!(!FirmwareUpdateStatus::Success.is_failure());
        assert!(!FirmwareUpdateStatus::InProgress.is_failure());
    }

    #[tokio::test]
    async fn test_firmware_update_status_is_in_progress() {
        assert!(FirmwareUpdateStatus::InProgress.is_in_progress());
        assert!(!FirmwareUpdateStatus::Success.is_in_progress());
        assert!(!FirmwareUpdateStatus::Failed.is_in_progress());
    }

    #[tokio::test]
    async fn test_firmware_controller_initial_state() {
        let fc = FirmwareController::with_default_obis();
        assert_eq!(fc.firmware_version().await, "1.0.0");
        assert_eq!(fc.update_status().await, FirmwareUpdateStatus::NoUpdate);
        assert_eq!(fc.update_time().await, None);
    }

    #[tokio::test]
    async fn test_firmware_controller_set_firmware_version() {
        let fc = FirmwareController::with_default_obis();
        fc.set_firmware_version(String::from("2.1.0")).await;
        assert_eq!(fc.firmware_version().await, "2.1.0");
    }

    #[tokio::test]
    async fn test_firmware_controller_set_firmware_signature() {
        let fc = FirmwareController::with_default_obis();
        fc.set_firmware_signature(String::from("ABC123")).await;
        assert_eq!(fc.firmware_signature().await, "ABC123");
    }

    #[tokio::test]
    async fn test_firmware_controller_set_update_status() {
        let fc = FirmwareController::with_default_obis();
        fc.set_update_status(FirmwareUpdateStatus::Success).await;
        assert!(fc.is_update_successful().await);
    }

    #[tokio::test]
    async fn test_firmware_controller_set_update_time() {
        let fc = FirmwareController::with_default_obis();
        fc.set_update_time(Some(1609459200)).await;
        assert_eq!(fc.update_time().await, Some(1609459200));
    }

    #[tokio::test]
    async fn test_firmware_controller_is_update_successful() {
        let fc = FirmwareController::with_default_obis();
        assert!(!fc.is_update_successful().await);

        fc.set_update_status(FirmwareUpdateStatus::Success).await;
        assert!(fc.is_update_successful().await);
    }

    #[tokio::test]
    async fn test_firmware_controller_is_update_failed() {
        let fc = FirmwareController::with_default_obis();
        assert!(!fc.is_update_failed().await);

        fc.set_update_status(FirmwareUpdateStatus::Failed).await;
        assert!(fc.is_update_failed().await);
    }

    #[tokio::test]
    async fn test_firmware_controller_is_update_in_progress() {
        let fc = FirmwareController::with_default_obis();
        assert!(!fc.is_update_in_progress().await);

        fc.set_update_status(FirmwareUpdateStatus::InProgress).await;
        assert!(fc.is_update_in_progress().await);
    }

    #[tokio::test]
    async fn test_firmware_controller_record_successful_update() {
        let fc = FirmwareController::with_default_obis();
        fc.record_successful_update(String::from("2.0.0"), String::from("NEW-SIG")).await;

        assert_eq!(fc.firmware_version().await, "2.0.0");
        assert_eq!(fc.firmware_signature().await, "NEW-SIG");
        assert!(fc.is_update_successful().await);
        assert!(fc.update_time().await.is_some());
    }

    #[tokio::test]
    async fn test_firmware_controller_record_failed_update() {
        let fc = FirmwareController::with_default_obis();
        fc.record_failed_update().await;
        assert!(fc.is_update_failed().await);
    }

    #[tokio::test]
    async fn test_firmware_controller_start_update() {
        let fc = FirmwareController::with_default_obis();
        fc.start_update().await;
        assert!(fc.is_update_in_progress().await);
    }

    #[tokio::test]
    async fn test_firmware_controller_version_tuple() {
        let fc = FirmwareController::with_version(ObisCode::new(0, 0, 83, 0, 0, 255), String::from("2.5.1"));
        assert_eq!(fc.version_tuple().await, (2, 5, 1));
    }

    #[tokio::test]
    async fn test_firmware_controller_version_tuple_invalid() {
        let fc = FirmwareController::with_version(ObisCode::new(0, 0, 83, 0, 0, 255), String::from("invalid"));
        assert_eq!(fc.version_tuple().await, (0, 0, 0));
    }

    #[tokio::test]
    async fn test_firmware_controller_compare_version_greater() {
        let fc = FirmwareController::with_version(ObisCode::new(0, 0, 83, 0, 0, 255), String::from("2.0.0"));
        assert_eq!(fc.compare_version("1.5.0").await, 1);
        assert_eq!(fc.compare_version("2.0.0").await, 0);
        assert_eq!(fc.compare_version("2.1.0").await, -1);
    }

    #[tokio::test]
    async fn test_firmware_controller_get_attributes() {
        let fc = FirmwareController::with_default_obis();

        // Test firmware_version
        let result = fc.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(String::from_utf8_lossy(&bytes), "1.0.0");
            }
            _ => panic!("Expected OctetString"),
        }

        // Test update_status
        let result = fc.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Enumerate(status) => assert_eq!(status, 0), // NoUpdate
            _ => panic!("Expected Enumerate"),
        }
    }

    #[tokio::test]
    async fn test_firmware_controller_set_attributes() {
        let fc = FirmwareController::with_default_obis();

        fc.set_attribute(2, DataObject::OctetString(b"3.0.0".to_vec()), None)
            .await
            .unwrap();
        assert_eq!(fc.firmware_version().await, "3.0.0");

        fc.set_attribute(4, DataObject::Enumerate(2), None) // Success
            .await
            .unwrap();
        assert!(fc.is_update_successful().await);
    }

    #[tokio::test]
    async fn test_firmware_controller_read_only_logical_name() {
        let fc = FirmwareController::with_default_obis();
        let result = fc
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 83, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_firmware_controller_invalid_attribute() {
        let fc = FirmwareController::with_default_obis();
        let result = fc.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_firmware_controller_invalid_method() {
        let fc = FirmwareController::with_default_obis();
        let result = fc.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_firmware_controller_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 83, 0, 0, 1);
        let fc = FirmwareController::new(obis);
        assert_eq!(fc.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_firmware_controller_update_time_null() {
        let fc = FirmwareController::with_default_obis();
        fc.set_attribute(5, DataObject::Null, None)
            .await
            .unwrap();
        assert_eq!(fc.update_time().await, None);
    }

    #[tokio::test]
    async fn test_firmware_controller_compare_version_different_lengths() {
        let fc = FirmwareController::with_version(ObisCode::new(0, 0, 83, 0, 0, 255), String::from("2.0"));
        assert_eq!(fc.compare_version("2.0.0").await, 0);
        assert_eq!(fc.compare_version("2.0.1").await, -1);
        assert_eq!(fc.compare_version("1.9.9").await, 1);
    }
}
