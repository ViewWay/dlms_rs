//! Image Transfer interface class (Class ID: 18)
//!
//! The Image Transfer interface class manages firmware and data image transfers
//! to DLMS/COSEM devices. It handles the complete image transfer process including
//! initiation, block transfer, verification, and activation.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: image_size - Size of the image in bytes
//! - Attribute 3: image_transferred_blocks - Number of successfully transferred blocks
//! - Attribute 4: image_first_not_transferred_block - First block not yet transferred
//! - Attribute 5: image_transfer_enabled - Whether image transfer is enabled
//! - Attribute 6: image_transfer_status - Current status of image transfer
//! - Attribute 7: image_to_activate_info - Information about the image to activate
//!
//! # Methods
//!
//! - Method 1: image_transform_initiate - Initiate a new image transfer
//! - Method 2: image_transform_block - Transfer a block of image data
//! - Method 3: image_transform_verify - Verify the transferred image
//! - Method 4: image_transform_activate - Activate the transferred image

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Image Transfer Status
///
/// Represents the current state of the image transfer process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ImageTransferStatus {
    /// Image transfer initiated - waiting for blocks
    Initiated = 0,
    /// Image transfer in progress - blocks being received
    InProgress = 1,
    /// Image transfer verified - ready for activation
    Verified = 2,
    /// Image transfer failed - verification failed
    VerificationFailed = 3,
    /// Image transfer failed - other reason
    TransferFailed = 4,
    /// No image transfer in progress
    Idle = 5,
}

impl ImageTransferStatus {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Initiated,
            1 => Self::InProgress,
            2 => Self::Verified,
            3 => Self::VerificationFailed,
            4 => Self::TransferFailed,
            _ => Self::Idle,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if transfer is active (not idle or failed)
    pub fn is_active(self) -> bool {
        matches!(self, Self::Initiated | Self::InProgress | Self::Verified)
    }

    /// Check if transfer has failed
    pub fn is_failed(self) -> bool {
        matches!(self, Self::VerificationFailed | Self::TransferFailed)
    }

    /// Check if transfer is complete and verified
    pub fn is_verified(self) -> bool {
        matches!(self, Self::Verified)
    }
}

/// Image Information
///
/// Contains metadata about a firmware or data image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageInfo {
    /// Image identification
    pub image_identification: Vec<u8>,
    /// Image size in bytes
    pub image_size: u32,
    /// Checksum of the image
    pub checksum: Vec<u8>,
    /// Signature of the image (for signed images)
    pub signature: Vec<u8>,
}

impl ImageInfo {
    /// Create a new image info
    pub fn new(image_identification: Vec<u8>, image_size: u32) -> Self {
        Self {
            image_identification,
            image_size,
            checksum: Vec::new(),
            signature: Vec::new(),
        }
    }

    /// Create with checksum
    pub fn with_checksum(mut self, checksum: Vec<u8>) -> Self {
        self.checksum = checksum;
        self
    }

    /// Create with signature
    pub fn with_signature(mut self, signature: Vec<u8>) -> Self {
        self.signature = signature;
        self
    }

    /// Convert to data object
    pub fn to_data_object(&self) -> DataObject {
        DataObject::Array(vec![
            DataObject::OctetString(self.image_identification.clone()),
            DataObject::Unsigned32(self.image_size),
            DataObject::OctetString(self.checksum.clone()),
            DataObject::OctetString(self.signature.clone()),
        ])
    }

    /// Create from data object
    pub fn from_data_object(value: &DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Array(arr) if arr.len() >= 2 => {
                let image_identification = match &arr[0] {
                    DataObject::OctetString(bytes) => bytes.clone(),
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected OctetString for image_identification".to_string(),
                        ))
                    }
                };
                let image_size = match &arr[1] {
                    DataObject::Unsigned32(size) => *size,
                    DataObject::Unsigned16(size) => *size as u32,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Unsigned32 for image_size".to_string(),
                        ))
                    }
                };
                let checksum = if arr.len() > 2 {
                    match &arr[2] {
                        DataObject::OctetString(bytes) => bytes.clone(),
                        _ => Vec::new(),
                    }
                } else {
                    Vec::new()
                };
                let signature = if arr.len() > 3 {
                    match &arr[3] {
                        DataObject::OctetString(bytes) => bytes.clone(),
                        _ => Vec::new(),
                    }
                } else {
                    Vec::new()
                };
                Ok(Self {
                    image_identification,
                    image_size,
                    checksum,
                    signature,
                })
            }
            _ => Err(DlmsError::InvalidData(
                "Expected Array for ImageInfo".to_string(),
            )),
        }
    }
}

/// Image Transfer interface class (Class ID: 18)
///
/// Default OBIS: 0-0:18.0.0.255
///
/// This class manages the complete firmware/data image transfer process.
#[derive(Debug, Clone)]
pub struct ImageTransfer {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Total size of the image in bytes
    image_size: Arc<RwLock<u32>>,

    /// Number of blocks successfully transferred
    image_transferred_blocks: Arc<RwLock<u32>>,

    /// First block number not yet transferred
    image_first_not_transferred_block: Arc<RwLock<u32>>,

    /// Block size in bytes
    block_size: Arc<RwLock<u32>>,

    /// Whether image transfer is enabled
    image_transfer_enabled: Arc<RwLock<bool>>,

    /// Current transfer status
    image_transfer_status: Arc<RwLock<ImageTransferStatus>>,

    /// Information about the image to be activated
    image_to_activate_info: Arc<RwLock<Option<ImageInfo>>>,

    /// Current image being transferred
    current_image_info: Arc<RwLock<Option<ImageInfo>>>,

    /// Transferred image data
    image_data: Arc<RwLock<Vec<u8>>>,
}

impl ImageTransfer {
    /// Class ID for Image Transfer
    pub const CLASS_ID: u16 = 18;

    /// Default OBIS code for Image Transfer (0-0:18.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 18, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_IMAGE_SIZE: u8 = 2;
    pub const ATTR_IMAGE_TRANSFERRED_BLOCKS: u8 = 3;
    pub const ATTR_IMAGE_FIRST_NOT_TRANSFERRED_BLOCK: u8 = 4;
    pub const ATTR_IMAGE_TRANSFER_ENABLED: u8 = 5;
    pub const ATTR_IMAGE_TRANSFER_STATUS: u8 = 6;
    pub const ATTR_IMAGE_TO_ACTIVATE_INFO: u8 = 7;

    /// Method IDs
    pub const METHOD_IMAGE_TRANSFORM_INITIATE: u8 = 1;
    pub const METHOD_IMAGE_TRANSFORM_BLOCK: u8 = 2;
    pub const METHOD_IMAGE_TRANSFORM_VERIFY: u8 = 3;
    pub const METHOD_IMAGE_TRANSFORM_ACTIVATE: u8 = 4;

    /// Default block size (256 bytes)
    pub const DEFAULT_BLOCK_SIZE: u32 = 256;

    /// Create a new Image Transfer object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            image_size: Arc::new(RwLock::new(0)),
            image_transferred_blocks: Arc::new(RwLock::new(0)),
            image_first_not_transferred_block: Arc::new(RwLock::new(0)),
            block_size: Arc::new(RwLock::new(Self::DEFAULT_BLOCK_SIZE)),
            image_transfer_enabled: Arc::new(RwLock::new(true)),
            image_transfer_status: Arc::new(RwLock::new(ImageTransferStatus::Idle)),
            image_to_activate_info: Arc::new(RwLock::new(None)),
            current_image_info: Arc::new(RwLock::new(None)),
            image_data: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get the image size
    pub async fn image_size(&self) -> u32 {
        *self.image_size.read().await
    }

    /// Get the number of transferred blocks
    pub async fn image_transferred_blocks(&self) -> u32 {
        *self.image_transferred_blocks.read().await
    }

    /// Get the first not transferred block
    pub async fn image_first_not_transferred_block(&self) -> u32 {
        *self.image_first_not_transferred_block.read().await
    }

    /// Get the block size
    pub async fn block_size(&self) -> u32 {
        *self.block_size.read().await
    }

    /// Set the block size
    pub async fn set_block_size(&self, size: u32) {
        *self.block_size.write().await = size;
    }

    /// Check if transfer is enabled
    pub async fn is_transfer_enabled(&self) -> bool {
        *self.image_transfer_enabled.read().await
    }

    /// Set transfer enabled state
    pub async fn set_transfer_enabled(&self, enabled: bool) {
        *self.image_transfer_enabled.write().await = enabled;
    }

    /// Get the transfer status
    pub async fn transfer_status(&self) -> ImageTransferStatus {
        *self.image_transfer_status.read().await
    }

    /// Get the image to activate info
    pub async fn image_to_activate_info(&self) -> Option<ImageInfo> {
        self.image_to_activate_info.read().await.clone()
    }

    /// Set the image to activate info
    pub async fn set_image_to_activate_info(&self, info: ImageInfo) {
        *self.image_to_activate_info.write().await = Some(info);
    }

    /// Initiate image transfer
    ///
    /// # Arguments
    /// * `image_size` - Total size of the image in bytes
    /// * `image_identification` - Image identification bytes
    pub async fn initiate_transfer(&self, image_size: u32, image_identification: Vec<u8>) -> DlmsResult<()> {
        if !self.is_transfer_enabled().await {
            return Err(DlmsError::InvalidData(
                "Image transfer is not enabled".to_string(),
            ));
        }

        let status = self.transfer_status().await;
        if status.is_active() {
            return Err(DlmsError::InvalidData(
                "Image transfer already in progress".to_string(),
            ));
        }

        *self.image_size.write().await = image_size;
        *self.image_transferred_blocks.write().await = 0;
        *self.image_first_not_transferred_block.write().await = 0;
        *self.image_transfer_status.write().await = ImageTransferStatus::Initiated;
        *self.image_data.write().await = Vec::new();

        let info = ImageInfo::new(image_identification.clone(), image_size);
        *self.current_image_info.write().await = Some(info);

        Ok(())
    }

    /// Transfer a block of image data
    ///
    /// # Arguments
    /// * `block_number` - Block number to transfer
    /// * `block_data` - Block data bytes
    pub async fn transfer_block(&self, block_number: u32, block_data: Vec<u8>) -> DlmsResult<()> {
        if !self.is_transfer_enabled().await {
            return Err(DlmsError::InvalidData(
                "Image transfer is not enabled".to_string(),
            ));
        }

        let status = self.transfer_status().await;
        if !status.is_active() {
            return Err(DlmsError::InvalidData(
                "No image transfer in progress".to_string(),
            ));
        }

        let expected_block = *self.image_first_not_transferred_block.read().await;
        if block_number != expected_block {
            return Err(DlmsError::InvalidData(format!(
                "Expected block {}, got {}",
                expected_block, block_number
            )));
        }

        // Store the block data
        let mut data = self.image_data.write().await;
        data.extend_from_slice(&block_data);
        drop(data);

        // Update counters
        *self.image_transferred_blocks.write().await += 1;
        *self.image_first_not_transferred_block.write().await += 1;
        *self.image_transfer_status.write().await = ImageTransferStatus::InProgress;

        // Check if transfer is complete
        let transferred_bytes = (*self.image_transferred_blocks.read().await as usize)
            * (*self.block_size.read().await as usize);
        let total_size = *self.image_size.read().await as usize;

        if transferred_bytes >= total_size {
            *self.image_transfer_status.write().await = ImageTransferStatus::Verified;
        }

        Ok(())
    }

    /// Verify the transferred image
    ///
    /// In a real implementation, this would verify the checksum and/or signature.
    pub async fn verify_image(&self) -> DlmsResult<bool> {
        let status = self.transfer_status().await;
        if !status.is_active() {
            return Err(DlmsError::InvalidData(
                "No image transfer to verify".to_string(),
            ));
        }

        // In a real implementation, we would verify the checksum/signature here
        // For now, we'll consider it verified if we received all blocks
        let transferred_bytes = (*self.image_transferred_blocks.read().await as usize)
            * (*self.block_size.read().await as usize);
        let total_size = *self.image_size.read().await as usize;

        let verified = transferred_bytes >= total_size;
        if verified {
            *self.image_transfer_status.write().await = ImageTransferStatus::Verified;
        } else {
            *self.image_transfer_status.write().await = ImageTransferStatus::VerificationFailed;
        }

        Ok(verified)
    }

    /// Activate the transferred image
    ///
    /// In a real implementation, this would initiate the firmware activation process.
    pub async fn activate_image(&self) -> DlmsResult<()> {
        let status = self.transfer_status().await;
        if !status.is_verified() {
            return Err(DlmsError::InvalidData(
                "Cannot activate image: not verified".to_string(),
            ));
        }

        // Move current image to activate info
        let current = self.current_image_info.read().await.clone();
        if let Some(info) = current {
            *self.image_to_activate_info.write().await = Some(info);
        }

        // Reset transfer status
        *self.image_transfer_status.write().await = ImageTransferStatus::Idle;
        *self.image_transferred_blocks.write().await = 0;
        *self.image_first_not_transferred_block.write().await = 0;
        *self.image_data.write().await = Vec::new();

        Ok(())
    }

    /// Reset the transfer state
    pub async fn reset(&self) {
        *self.image_transfer_status.write().await = ImageTransferStatus::Idle;
        *self.image_transferred_blocks.write().await = 0;
        *self.image_first_not_transferred_block.write().await = 0;
        *self.image_data.write().await = Vec::new();
        *self.current_image_info.write().await = None;
    }

    /// Calculate total number of blocks for the image
    pub async fn total_blocks(&self) -> u32 {
        let block_size = *self.block_size.read().await;
        let image_size = *self.image_size.read().await;
        (image_size + block_size - 1) / block_size
    }

    /// Get transfer progress as a percentage (0-100)
    pub async fn progress(&self) -> u8 {
        let total = self.total_blocks().await;
        if total == 0 {
            return 0;
        }
        let transferred = *self.image_transferred_blocks.read().await;
        ((transferred * 100) / total) as u8
    }
}

#[async_trait]
impl CosemObject for ImageTransfer {
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
            Self::ATTR_IMAGE_SIZE => {
                let size = self.image_size().await;
                Ok(DataObject::Unsigned32(size))
            }
            Self::ATTR_IMAGE_TRANSFERRED_BLOCKS => {
                let blocks = self.image_transferred_blocks().await;
                Ok(DataObject::Unsigned32(blocks))
            }
            Self::ATTR_IMAGE_FIRST_NOT_TRANSFERRED_BLOCK => {
                let block = self.image_first_not_transferred_block().await;
                Ok(DataObject::Unsigned32(block))
            }
            Self::ATTR_IMAGE_TRANSFER_ENABLED => {
                let enabled = self.is_transfer_enabled().await;
                Ok(DataObject::Boolean(enabled))
            }
            Self::ATTR_IMAGE_TRANSFER_STATUS => {
                let status = self.transfer_status().await;
                Ok(DataObject::Enumerate(status.to_u8()))
            }
            Self::ATTR_IMAGE_TO_ACTIVATE_INFO => {
                match self.image_to_activate_info().await {
                    Some(info) => Ok(info.to_data_object()),
                    None => Ok(DataObject::Null),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Image Transfer has no attribute {}",
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
            Self::ATTR_IMAGE_SIZE => {
                match value {
                    DataObject::Unsigned32(size) => {
                        *self.image_size.write().await = size;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned32 for image_size".to_string(),
                    )),
                }
            }
            Self::ATTR_IMAGE_TRANSFERRED_BLOCKS => {
                match value {
                    DataObject::Unsigned32(blocks) => {
                        *self.image_transferred_blocks.write().await = blocks;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned32 for image_transferred_blocks".to_string(),
                    )),
                }
            }
            Self::ATTR_IMAGE_FIRST_NOT_TRANSFERRED_BLOCK => {
                match value {
                    DataObject::Unsigned32(block) => {
                        *self.image_first_not_transferred_block.write().await = block;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned32 for image_first_not_transferred_block".to_string(),
                    )),
                }
            }
            Self::ATTR_IMAGE_TRANSFER_ENABLED => {
                match value {
                    DataObject::Boolean(enabled) => {
                        self.set_transfer_enabled(enabled).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Boolean for image_transfer_enabled".to_string(),
                    )),
                }
            }
            Self::ATTR_IMAGE_TRANSFER_STATUS => {
                match value {
                    DataObject::Enumerate(status) => {
                        *self.image_transfer_status.write().await = ImageTransferStatus::from_u8(status);
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for image_transfer_status".to_string(),
                    )),
                }
            }
            Self::ATTR_IMAGE_TO_ACTIVATE_INFO => {
                match value {
                    DataObject::Null => {
                        *self.image_to_activate_info.write().await = None;
                        Ok(())
                    }
                    value => {
                        let info = ImageInfo::from_data_object(&value)?;
                        self.set_image_to_activate_info(info).await;
                        Ok(())
                    }
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Image Transfer has no attribute {}",
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
            Self::METHOD_IMAGE_TRANSFORM_INITIATE => {
                match parameters {
                    Some(DataObject::Array(arr)) if arr.len() >= 2 => {
                        let image_size = match &arr[0] {
                            DataObject::Unsigned32(size) => *size,
                            DataObject::Unsigned16(size) => *size as u32,
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Expected Unsigned32 for image_size".to_string(),
                                ))
                            }
                        };
                        let image_identification = match &arr[1] {
                            DataObject::OctetString(bytes) => bytes.clone(),
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Expected OctetString for image_identification".to_string(),
                                ))
                            }
                        };
                        self.initiate_transfer(image_size, image_identification).await?;
                        Ok(None)
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Method 1 expects Array parameter with image_size and image_identification".to_string(),
                    )),
                }
            }
            Self::METHOD_IMAGE_TRANSFORM_BLOCK => {
                match parameters {
                    Some(DataObject::Array(arr)) if arr.len() >= 2 => {
                        let block_number = match &arr[0] {
                            DataObject::Unsigned32(bn) => *bn,
                            DataObject::Unsigned16(bn) => *bn as u32,
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Expected Unsigned32 for block_number".to_string(),
                                ))
                            }
                        };
                        let block_data = match &arr[1] {
                            DataObject::OctetString(bytes) => bytes.clone(),
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Expected OctetString for block_data".to_string(),
                                ))
                            }
                        };
                        self.transfer_block(block_number, block_data).await?;
                        Ok(None)
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Method 2 expects Array parameter with block_number and block_data".to_string(),
                    )),
                }
            }
            Self::METHOD_IMAGE_TRANSFORM_VERIFY => {
                let verified = self.verify_image().await?;
                Ok(Some(DataObject::Boolean(verified)))
            }
            Self::METHOD_IMAGE_TRANSFORM_ACTIVATE => {
                self.activate_image().await?;
                Ok(None)
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Image Transfer has no method {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_image_transfer_class_id() {
        let it = ImageTransfer::with_default_obis();
        assert_eq!(it.class_id(), 18);
    }

    #[tokio::test]
    async fn test_image_transfer_obis_code() {
        let it = ImageTransfer::with_default_obis();
        assert_eq!(it.obis_code(), ImageTransfer::default_obis());
    }

    #[tokio::test]
    async fn test_image_transfer_status_from_u8() {
        assert_eq!(ImageTransferStatus::from_u8(0), ImageTransferStatus::Initiated);
        assert_eq!(ImageTransferStatus::from_u8(1), ImageTransferStatus::InProgress);
        assert_eq!(ImageTransferStatus::from_u8(2), ImageTransferStatus::Verified);
        assert_eq!(ImageTransferStatus::from_u8(3), ImageTransferStatus::VerificationFailed);
        assert_eq!(ImageTransferStatus::from_u8(4), ImageTransferStatus::TransferFailed);
        assert_eq!(ImageTransferStatus::from_u8(99), ImageTransferStatus::Idle);
    }

    #[tokio::test]
    async fn test_image_transfer_status_to_u8() {
        assert_eq!(ImageTransferStatus::Initiated.to_u8(), 0);
        assert_eq!(ImageTransferStatus::InProgress.to_u8(), 1);
        assert_eq!(ImageTransferStatus::Verified.to_u8(), 2);
        assert_eq!(ImageTransferStatus::VerificationFailed.to_u8(), 3);
        assert_eq!(ImageTransferStatus::TransferFailed.to_u8(), 4);
        assert_eq!(ImageTransferStatus::Idle.to_u8(), 5);
    }

    #[tokio::test]
    async fn test_image_transfer_status_is_active() {
        assert!(ImageTransferStatus::Initiated.is_active());
        assert!(ImageTransferStatus::InProgress.is_active());
        assert!(ImageTransferStatus::Verified.is_active());
        assert!(!ImageTransferStatus::VerificationFailed.is_active());
        assert!(!ImageTransferStatus::TransferFailed.is_active());
        assert!(!ImageTransferStatus::Idle.is_active());
    }

    #[tokio::test]
    async fn test_image_transfer_status_is_failed() {
        assert!(ImageTransferStatus::VerificationFailed.is_failed());
        assert!(ImageTransferStatus::TransferFailed.is_failed());
        assert!(!ImageTransferStatus::Initiated.is_failed());
    }

    #[tokio::test]
    async fn test_image_transfer_status_is_verified() {
        assert!(ImageTransferStatus::Verified.is_verified());
        assert!(!ImageTransferStatus::InProgress.is_verified());
    }

    #[tokio::test]
    async fn test_image_transfer_initial_state() {
        let it = ImageTransfer::with_default_obis();
        assert_eq!(it.image_size().await, 0);
        assert_eq!(it.image_transferred_blocks().await, 0);
        assert_eq!(it.image_first_not_transferred_block().await, 0);
        assert!(it.is_transfer_enabled().await);
        assert_eq!(it.transfer_status().await, ImageTransferStatus::Idle);
    }

    #[tokio::test]
    async fn test_image_transfer_initiate() {
        let it = ImageTransfer::with_default_obis();
        let identification = vec![0x01, 0x02, 0x03, 0x04];

        it.initiate_transfer(1024, identification.clone()).await.unwrap();

        assert_eq!(it.image_size().await, 1024);
        assert_eq!(it.transfer_status().await, ImageTransferStatus::Initiated);
        assert_eq!(it.total_blocks().await, 4); // 1024 / 256 = 4
    }

    #[tokio::test]
    async fn test_image_transfer_initiate_disabled() {
        let it = ImageTransfer::with_default_obis();
        it.set_transfer_enabled(false).await;

        let result = it.initiate_transfer(1024, vec![1, 2, 3]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_image_transfer_initiate_already_in_progress() {
        let it = ImageTransfer::with_default_obis();

        it.initiate_transfer(1024, vec![1, 2, 3]).await.unwrap();

        let result = it.initiate_transfer(2048, vec![5, 6, 7]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_image_transfer_block() {
        let it = ImageTransfer::with_default_obis();

        it.initiate_transfer(512, vec![1, 2, 3]).await.unwrap();

        let block1 = vec![0u8; 256];
        it.transfer_block(0, block1).await.unwrap();

        assert_eq!(it.image_transferred_blocks().await, 1);
        assert_eq!(it.image_first_not_transferred_block().await, 1);
        assert_eq!(it.transfer_status().await, ImageTransferStatus::InProgress);
    }

    #[tokio::test]
    async fn test_image_transfer_block_wrong_number() {
        let it = ImageTransfer::with_default_obis();

        it.initiate_transfer(512, vec![1, 2, 3]).await.unwrap();

        let block = vec![0u8; 256];
        let result = it.transfer_block(5, block).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_image_transfer_complete() {
        let it = ImageTransfer::with_default_obis();
        it.set_block_size(128).await;

        it.initiate_transfer(256, vec![1, 2, 3]).await.unwrap();

        let block1 = vec![1u8; 128];
        let block2 = vec![2u8; 128];

        it.transfer_block(0, block1).await.unwrap();
        assert_eq!(it.transfer_status().await, ImageTransferStatus::InProgress);

        it.transfer_block(1, block2).await.unwrap();

        // After receiving all blocks, should be verified
        assert_eq!(it.transfer_status().await, ImageTransferStatus::Verified);
    }

    #[tokio::test]
    async fn test_image_transfer_verify() {
        let it = ImageTransfer::with_default_obis();
        it.set_block_size(128).await;

        it.initiate_transfer(256, vec![1, 2, 3]).await.unwrap();

        let block1 = vec![1u8; 128];
        let block2 = vec![2u8; 128];

        it.transfer_block(0, block1).await.unwrap();
        it.transfer_block(1, block2).await.unwrap();

        let verified = it.verify_image().await.unwrap();
        assert!(verified);
        assert_eq!(it.transfer_status().await, ImageTransferStatus::Verified);
    }

    #[tokio::test]
    async fn test_image_transfer_activate() {
        let it = ImageTransfer::with_default_obis();
        it.set_block_size(128).await;

        it.initiate_transfer(256, vec![1, 2, 3]).await.unwrap();

        let block1 = vec![1u8; 128];
        let block2 = vec![2u8; 128];

        it.transfer_block(0, block1).await.unwrap();
        it.transfer_block(1, block2).await.unwrap();

        it.activate_image().await.unwrap();

        assert_eq!(it.transfer_status().await, ImageTransferStatus::Idle);
        assert!(it.image_to_activate_info().await.is_some());
    }

    #[tokio::test]
    async fn test_image_transfer_activate_not_verified() {
        let it = ImageTransfer::with_default_obis();

        it.initiate_transfer(256, vec![1, 2, 3]).await.unwrap();

        let result = it.activate_image().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_image_transfer_progress() {
        let it = ImageTransfer::with_default_obis();

        it.initiate_transfer(1024, vec![1, 2, 3]).await.unwrap();
        assert_eq!(it.progress().await, 0);

        let block1 = vec![0u8; 256];
        it.transfer_block(0, block1).await.unwrap();
        assert_eq!(it.progress().await, 25); // 1/4 blocks

        let block2 = vec![0u8; 256];
        it.transfer_block(1, block2).await.unwrap();
        assert_eq!(it.progress().await, 50); // 2/4 blocks
    }

    #[tokio::test]
    async fn test_image_transfer_reset() {
        let it = ImageTransfer::with_default_obis();

        it.initiate_transfer(1024, vec![1, 2, 3]).await.unwrap();
        it.reset().await;

        assert_eq!(it.transfer_status().await, ImageTransferStatus::Idle);
        assert_eq!(it.image_transferred_blocks().await, 0);
        assert_eq!(it.image_first_not_transferred_block().await, 0);
    }

    #[tokio::test]
    async fn test_image_info_new() {
        let info = ImageInfo::new(vec![1, 2, 3], 1024);
        assert_eq!(info.image_size, 1024);
        assert_eq!(info.image_identification, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_image_info_with_checksum() {
        let info = ImageInfo::new(vec![1, 2, 3], 1024)
            .with_checksum(vec![0xAA, 0xBB]);
        assert_eq!(info.checksum, vec![0xAA, 0xBB]);
    }

    #[tokio::test]
    async fn test_image_info_to_data_object() {
        let info = ImageInfo::new(vec![1, 2, 3], 1024);
        let data = info.to_data_object();

        match data {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 4);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_image_info_from_data_object() {
        let data = DataObject::Array(vec![
            DataObject::OctetString(vec![1, 2, 3]),
            DataObject::Unsigned32(1024),
            DataObject::OctetString(vec![0xAA, 0xBB]),
            DataObject::OctetString(vec![0xCC, 0xDD]),
        ]);

        let info = ImageInfo::from_data_object(&data).unwrap();
        assert_eq!(info.image_size, 1024);
        assert_eq!(info.checksum, vec![0xAA, 0xBB]);
        assert_eq!(info.signature, vec![0xCC, 0xDD]);
    }

    #[tokio::test]
    async fn test_image_transfer_get_attributes() {
        let it = ImageTransfer::with_default_obis();

        // Test logical_name
        let result = it.get_attribute(1, None).await.unwrap();
        match result {
            DataObject::OctetString(bytes) => assert_eq!(bytes.len(), 6),
            _ => panic!("Expected OctetString"),
        }

        // Test image_size
        let result = it.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Unsigned32(size) => assert_eq!(size, 0),
            _ => panic!("Expected Unsigned32"),
        }

        // Test image_transfer_enabled
        let result = it.get_attribute(5, None).await.unwrap();
        match result {
            DataObject::Boolean(enabled) => assert!(enabled),
            _ => panic!("Expected Boolean"),
        }

        // Test image_transfer_status
        let result = it.get_attribute(6, None).await.unwrap();
        match result {
            DataObject::Enumerate(status) => assert_eq!(status, 5), // Idle
            _ => panic!("Expected Enumerate"),
        }
    }

    #[tokio::test]
    async fn test_image_transfer_set_attributes() {
        let it = ImageTransfer::with_default_obis();

        it.set_attribute(2, DataObject::Unsigned32(2048), None)
            .await
            .unwrap();
        assert_eq!(it.image_size().await, 2048);

        it.set_attribute(5, DataObject::Boolean(false), None)
            .await
            .unwrap();
        assert!(!it.is_transfer_enabled().await);

        it.set_attribute(6, DataObject::Enumerate(1), None)
            .await
            .unwrap();
        assert_eq!(it.transfer_status().await, ImageTransferStatus::InProgress);
    }

    #[tokio::test]
    async fn test_image_transfer_read_only_logical_name() {
        let it = ImageTransfer::with_default_obis();
        let result = it
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 18, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_image_transfer_set_image_to_activate() {
        let it = ImageTransfer::with_default_obis();
        let info_data = DataObject::Array(vec![
            DataObject::OctetString(vec![1, 2, 3]),
            DataObject::Unsigned32(1024),
            DataObject::OctetString(vec![0xAA, 0xBB]),
        ]);

        it.set_attribute(7, info_data, None).await.unwrap();

        let info = it.image_to_activate_info().await;
        assert!(info.is_some());
        assert_eq!(info.unwrap().image_size, 1024);
    }

    #[tokio::test]
    async fn test_image_transfer_method_initiate() {
        let it = ImageTransfer::with_default_obis();
        let params = DataObject::Array(vec![
            DataObject::Unsigned32(1024),
            DataObject::OctetString(vec![1, 2, 3, 4]),
        ]);

        it.invoke_method(1, Some(params), None).await.unwrap();
        assert_eq!(it.image_size().await, 1024);
        assert_eq!(it.transfer_status().await, ImageTransferStatus::Initiated);
    }

    #[tokio::test]
    async fn test_image_transfer_method_block() {
        let it = ImageTransfer::with_default_obis();

        it.initiate_transfer(512, vec![1, 2, 3]).await.unwrap();

        let params = DataObject::Array(vec![
            DataObject::Unsigned32(0),
            DataObject::OctetString(vec![0u8; 256]),
        ]);

        it.invoke_method(2, Some(params), None).await.unwrap();
        assert_eq!(it.image_transferred_blocks().await, 1);
    }

    #[tokio::test]
    async fn test_image_transfer_method_verify() {
        let it = ImageTransfer::with_default_obis();
        it.set_block_size(128).await;

        it.initiate_transfer(128, vec![1, 2, 3]).await.unwrap();

        let block = vec![1u8; 128];
        it.transfer_block(0, block).await.unwrap();

        let result = it.invoke_method(3, None, None).await.unwrap();
        match result {
            Some(DataObject::Boolean(verified)) => assert!(verified),
            _ => panic!("Expected Boolean result"),
        }
    }

    #[tokio::test]
    async fn test_image_transfer_method_activate() {
        let it = ImageTransfer::with_default_obis();
        it.set_block_size(128).await;

        it.initiate_transfer(128, vec![1, 2, 3]).await.unwrap();
        let block = vec![1u8; 128];
        it.transfer_block(0, block).await.unwrap();

        it.invoke_method(4, None, None).await.unwrap();
        assert_eq!(it.transfer_status().await, ImageTransferStatus::Idle);
    }

    #[tokio::test]
    async fn test_image_transfer_invalid_attribute() {
        let it = ImageTransfer::with_default_obis();
        let result = it.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_image_transfer_invalid_method() {
        let it = ImageTransfer::with_default_obis();
        let result = it.invoke_method(99, None, None).await;
        assert!(result.is_err());
    }
}
