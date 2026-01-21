//! Compact Data interface class (Class ID: 92)
//!
//! The Compact Data interface class manages compact data for efficient transmission.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: buffer - The compact data buffer
//! - Attribute 3: buffer_size - Size of the buffer
//! - Attribute 4: capture_time - Timestamp of data capture

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Compact Data interface class (Class ID: 92)
///
/// Default OBIS: 0-0:92.0.0.255
///
/// This class manages compact data for efficient transmission.
#[derive(Debug, Clone)]
pub struct CompactData {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// The compact data buffer
    buffer: Arc<RwLock<Vec<u8>>>,

    /// Size of the buffer
    buffer_size: Arc<RwLock<usize>>,

    /// Timestamp of data capture (Unix timestamp)
    capture_time: Arc<RwLock<Option<i64>>>,
}

impl CompactData {
    /// Class ID for CompactData
    pub const CLASS_ID: u16 = 92;

    /// Default OBIS code for CompactData (0-0:92.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 92, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_BUFFER: u8 = 2;
    pub const ATTR_BUFFER_SIZE: u8 = 3;
    pub const ATTR_CAPTURE_TIME: u8 = 4;

    /// Create a new CompactData object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            buffer: Arc::new(RwLock::new(Vec::new())),
            buffer_size: Arc::new(RwLock::new(1024)),
            capture_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Create with specific buffer size
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `buffer_size` - Size of the buffer
    pub fn with_buffer_size(logical_name: ObisCode, buffer_size: usize) -> Self {
        Self {
            logical_name,
            buffer: Arc::new(RwLock::new(Vec::with_capacity(buffer_size))),
            buffer_size: Arc::new(RwLock::new(buffer_size)),
            capture_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the buffer content
    pub async fn buffer(&self) -> Vec<u8> {
        self.buffer.read().await.clone()
    }

    /// Set the buffer content
    pub async fn set_buffer(&self, data: Vec<u8>) -> DlmsResult<()> {
        let buffer_size = self.buffer_size().await;
        if data.len() > buffer_size {
            return Err(DlmsError::InvalidData(format!(
                "Buffer size {} exceeds maximum {}",
                data.len(),
                buffer_size
            )));
        }
        *self.buffer.write().await = data;
        Ok(())
    }

    /// Get the buffer size
    pub async fn buffer_size(&self) -> usize {
        *self.buffer_size.read().await
    }

    /// Set the buffer size
    pub async fn set_buffer_size(&self, size: usize) {
        *self.buffer_size.write().await = size;
    }

    /// Get the current buffer length
    pub async fn len(&self) -> usize {
        self.buffer.read().await.len()
    }

    /// Check if the buffer is empty
    pub async fn is_empty(&self) -> bool {
        self.buffer.read().await.is_empty()
    }

    /// Get the capture time
    pub async fn capture_time(&self) -> Option<i64> {
        *self.capture_time.read().await
    }

    /// Set the capture time
    pub async fn set_capture_time(&self, time: Option<i64>) {
        *self.capture_time.write().await = time;
    }

    /// Clear the buffer
    pub async fn clear(&self) {
        self.buffer.write().await.clear();
    }

    /// Append data to the buffer
    pub async fn append(&self, data: &[u8]) -> DlmsResult<()> {
        let buffer_size = self.buffer_size().await;
        let current_len = self.len().await;
        if current_len + data.len() > buffer_size {
            return Err(DlmsError::InvalidData(format!(
                "Append would exceed buffer size {}",
                buffer_size
            )));
        }
        self.buffer.write().await.extend_from_slice(data);
        Ok(())
    }

    /// Get the remaining capacity
    pub async fn remaining_capacity(&self) -> usize {
        let max = self.buffer_size().await;
        let current = self.len().await;
        max.saturating_sub(current)
    }

    /// Get a byte at the specified index
    pub async fn get(&self, index: usize) -> DlmsResult<u8> {
        let buffer = self.buffer.read().await;
        if index >= buffer.len() {
            return Err(DlmsError::InvalidData(format!(
                "Index {} out of bounds (size: {})",
                index,
                buffer.len()
            )));
        }
        Ok(buffer[index])
    }

    /// Set a byte at the specified index
    pub async fn set(&self, index: usize, value: u8) -> DlmsResult<()> {
        let mut buffer = self.buffer.write().await;
        if index >= buffer.len() {
            return Err(DlmsError::InvalidData(format!(
                "Index {} out of bounds (size: {})",
                index,
                buffer.len()
            )));
        }
        buffer[index] = value;
        Ok(())
    }

    /// Get a slice of the buffer
    pub async fn get_range(&self, start: usize, end: usize) -> DlmsResult<Vec<u8>> {
        let buffer = self.buffer.read().await;
        if start > end || end > buffer.len() {
            return Err(DlmsError::InvalidData(format!(
                "Invalid range [{}, {}) for buffer of size {}",
                start,
                end,
                buffer.len()
            )));
        }
        Ok(buffer[start..end].to_vec())
    }

    /// Update capture time to current time
    pub async fn update_capture_time(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.set_capture_time(Some(now)).await;
    }

    /// Get buffer as hex string
    pub async fn to_hex_string(&self) -> String {
        let buffer = self.buffer().await;
        buffer.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Set buffer from hex string
    pub async fn from_hex_string(&self, hex: &str) -> DlmsResult<()> {
        let hex = hex.trim_start_matches("0x");
        if hex.len() % 2 != 0 {
            return Err(DlmsError::InvalidData(
                "Hex string must have even length".to_string(),
            ));
        }

        let mut bytes = Vec::new();
        for i in (0..hex.len()).step_by(2) {
            let byte_str = &hex[i..i + 2];
            let byte = u8::from_str_radix(byte_str, 16).map_err(|_| {
                DlmsError::InvalidData(format!("Invalid hex string: {}", byte_str))
            })?;
            bytes.push(byte);
        }

        self.set_buffer(bytes).await
    }

    /// Get buffer fill ratio (0.0 to 1.0)
    pub async fn fill_ratio(&self) -> f64 {
        let buffer_size = self.buffer_size().await;
        if buffer_size == 0 {
            return 0.0;
        }
        let current_len = self.len().await;
        (current_len as f64) / (buffer_size as f64)
    }

    /// Check if buffer is full
    pub async fn is_full(&self) -> bool {
        self.len().await >= self.buffer_size().await
    }

    /// Compress buffer using simple run-length encoding
    pub async fn compress_rle(&self) -> Vec<u8> {
        let buffer = self.buffer().await;
        let mut compressed = Vec::new();

        if buffer.is_empty() {
            return compressed;
        }

        let mut current = buffer[0];
        let mut count = 1u8;

        for &byte in &buffer[1..] {
            if byte == current && count < 255 {
                count += 1;
            } else {
                compressed.push(current);
                compressed.push(count);
                current = byte;
                count = 1;
            }
        }

        compressed.push(current);
        compressed.push(count);

        compressed
    }

    /// Decompress RLE data
    pub async fn decompress_rle(&self, data: &[u8]) -> DlmsResult<Vec<u8>> {
        if data.len() % 2 != 0 {
            return Err(DlmsError::InvalidData(
                "RLE data must have even length".to_string(),
            ));
        }

        let mut decompressed = Vec::new();
        for chunk in data.chunks_exact(2) {
            let byte = chunk[0];
            let count = chunk[1];
            for _ in 0..count {
                decompressed.push(byte);
            }
        }

        Ok(decompressed)
    }
}

#[async_trait]
impl CosemObject for CompactData {
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
            Self::ATTR_BUFFER => {
                Ok(DataObject::OctetString(self.buffer().await))
            }
            Self::ATTR_BUFFER_SIZE => {
                Ok(DataObject::Unsigned16(self.buffer_size().await as u16))
            }
            Self::ATTR_CAPTURE_TIME => {
                match self.capture_time().await {
                    Some(timestamp) => Ok(DataObject::Integer64(timestamp)),
                    None => Ok(DataObject::Null),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "CompactData has no attribute {}",
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
            Self::ATTR_BUFFER => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_buffer(bytes).await?;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for buffer".to_string(),
                    )),
                }
            }
            Self::ATTR_BUFFER_SIZE => {
                match value {
                    DataObject::Unsigned16(size) => {
                        self.set_buffer_size(size as usize).await;
                        Ok(())
                    }
                    DataObject::Unsigned8(size) => {
                        self.set_buffer_size(size as usize).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned16/Unsigned8 for buffer_size".to_string(),
                    )),
                }
            }
            Self::ATTR_CAPTURE_TIME => {
                match value {
                    DataObject::Integer64(timestamp) => {
                        self.set_capture_time(Some(timestamp)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_capture_time(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 or Null for capture_time".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "CompactData has no attribute {}",
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
            "CompactData has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compact_data_class_id() {
        let cd = CompactData::with_default_obis();
        assert_eq!(cd.class_id(), 92);
    }

    #[tokio::test]
    async fn test_compact_data_obis_code() {
        let cd = CompactData::with_default_obis();
        assert_eq!(cd.obis_code(), CompactData::default_obis());
    }

    #[tokio::test]
    async fn test_compact_data_initial_state() {
        let cd = CompactData::with_default_obis();
        assert!(cd.is_empty().await);
        assert_eq!(cd.len().await, 0);
        assert_eq!(cd.buffer_size().await, 1024);
        assert_eq!(cd.capture_time().await, None);
    }

    #[tokio::test]
    async fn test_compact_data_set_buffer() {
        let cd = CompactData::with_default_obis();
        let data = vec![1, 2, 3, 4, 5];
        cd.set_buffer(data.clone()).await.unwrap();
        assert_eq!(cd.buffer().await, data);
        assert_eq!(cd.len().await, 5);
    }

    #[tokio::test]
    async fn test_compact_data_set_buffer_too_large() {
        let cd = CompactData::with_buffer_size(ObisCode::new(0, 0, 92, 0, 0, 255), 10);
        let data = vec![1u8; 20];
        let result = cd.set_buffer(data).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compact_data_clear() {
        let cd = CompactData::with_default_obis();
        cd.set_buffer(vec![1, 2, 3]).await.unwrap();
        assert!(!cd.is_empty().await);

        cd.clear().await;
        assert!(cd.is_empty().await);
    }

    #[tokio::test]
    async fn test_compact_data_append() {
        let cd = CompactData::with_default_obis();
        cd.set_buffer(vec![1, 2, 3]).await.unwrap();
        cd.append(&[4, 5]).await.unwrap();
        assert_eq!(cd.buffer().await, vec![1, 2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn test_compact_data_append_too_much() {
        let cd = CompactData::with_buffer_size(ObisCode::new(0, 0, 92, 0, 0, 255), 5);
        cd.set_buffer(vec![1, 2, 3]).await.unwrap();
        let result = cd.append(&[4, 5, 6]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compact_data_remaining_capacity() {
        let cd = CompactData::with_buffer_size(ObisCode::new(0, 0, 92, 0, 0, 255), 100);
        assert_eq!(cd.remaining_capacity().await, 100);

        cd.set_buffer(vec![1u8; 30]).await.unwrap();
        assert_eq!(cd.remaining_capacity().await, 70);
    }

    #[tokio::test]
    async fn test_compact_data_get() {
        let cd = CompactData::with_default_obis();
        cd.set_buffer(vec![10, 20, 30]).await.unwrap();
        assert_eq!(cd.get(0).await.unwrap(), 10);
        assert_eq!(cd.get(1).await.unwrap(), 20);
        assert_eq!(cd.get(2).await.unwrap(), 30);
    }

    #[tokio::test]
    async fn test_compact_data_get_out_of_bounds() {
        let cd = CompactData::with_default_obis();
        cd.set_buffer(vec![10, 20]).await.unwrap();
        let result = cd.get(5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compact_data_set() {
        let cd = CompactData::with_default_obis();
        cd.set_buffer(vec![10, 20, 30]).await.unwrap();
        cd.set(1, 99).await.unwrap();
        assert_eq!(cd.get(1).await.unwrap(), 99);
    }

    #[tokio::test]
    async fn test_compact_data_set_out_of_bounds() {
        let cd = CompactData::with_default_obis();
        cd.set_buffer(vec![10, 20]).await.unwrap();
        let result = cd.set(5, 99).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compact_data_get_range() {
        let cd = CompactData::with_default_obis();
        cd.set_buffer(vec![1, 2, 3, 4, 5]).await.unwrap();
        assert_eq!(cd.get_range(1, 4).await.unwrap(), vec![2, 3, 4]);
    }

    #[tokio::test]
    async fn test_compact_data_get_range_invalid() {
        let cd = CompactData::with_default_obis();
        cd.set_buffer(vec![1, 2, 3]).await.unwrap();
        let result = cd.get_range(1, 10).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compact_data_fill_ratio() {
        let cd = CompactData::with_buffer_size(ObisCode::new(0, 0, 92, 0, 0, 255), 100);
        assert_eq!(cd.fill_ratio().await, 0.0);

        cd.set_buffer(vec![1u8; 50]).await.unwrap();
        assert!((cd.fill_ratio().await - 0.5).abs() < 0.001);

        cd.set_buffer(vec![1u8; 100]).await.unwrap();
        assert_eq!(cd.fill_ratio().await, 1.0);
    }

    #[tokio::test]
    async fn test_compact_data_is_full() {
        let cd = CompactData::with_buffer_size(ObisCode::new(0, 0, 92, 0, 0, 255), 10);
        assert!(!cd.is_full().await);

        cd.set_buffer(vec![1u8; 10]).await.unwrap();
        assert!(cd.is_full().await);
    }

    #[tokio::test]
    async fn test_compact_data_to_hex_string() {
        let cd = CompactData::with_default_obis();
        cd.set_buffer(vec![0x01, 0x02, 0xAB, 0xFF]).await.unwrap();
        assert_eq!(cd.to_hex_string().await, "0102abff");
    }

    #[tokio::test]
    async fn test_compact_data_from_hex_string() {
        let cd = CompactData::with_default_obis();
        cd.from_hex_string("0102ABFF").await.unwrap();
        assert_eq!(cd.buffer().await, vec![0x01, 0x02, 0xAB, 0xFF]);
    }

    #[tokio::test]
    async fn test_compact_data_from_hex_string_with_prefix() {
        let cd = CompactData::with_default_obis();
        cd.from_hex_string("0x0102ABFF").await.unwrap();
        assert_eq!(cd.buffer().await, vec![0x01, 0x02, 0xAB, 0xFF]);
    }

    #[tokio::test]
    async fn test_compact_data_from_hex_string_invalid_length() {
        let cd = CompactData::with_default_obis();
        let result = cd.from_hex_string("0102ABF").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compact_data_compress_rle() {
        let cd = CompactData::with_default_obis();
        cd.set_buffer(vec![1, 1, 1, 2, 2, 3, 3, 3, 3]).await.unwrap();
        let compressed = cd.compress_rle().await;
        assert_eq!(compressed, vec![1, 3, 2, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_compact_data_decompress_rle() {
        let cd = CompactData::with_default_obis();
        let compressed = vec![1, 3, 2, 2, 3, 4];
        let decompressed = cd.decompress_rle(&compressed).await.unwrap();
        assert_eq!(decompressed, vec![1, 1, 1, 2, 2, 3, 3, 3, 3]);
    }

    #[tokio::test]
    async fn test_compact_data_decompress_rle_invalid_length() {
        let cd = CompactData::with_default_obis();
        let result = cd.decompress_rle(&[1, 2, 3]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compact_data_with_buffer_size() {
        let cd = CompactData::with_buffer_size(ObisCode::new(0, 0, 92, 0, 0, 255), 512);
        assert_eq!(cd.buffer_size().await, 512);
    }

    #[tokio::test]
    async fn test_compact_data_get_attributes() {
        let cd = CompactData::with_default_obis();

        // Test buffer
        let result = cd.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::OctetString(bytes) => assert!(bytes.is_empty()),
            _ => panic!("Expected OctetString"),
        }

        // Test buffer_size
        let result = cd.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Unsigned16(size) => assert_eq!(size, 1024),
            _ => panic!("Expected Unsigned16"),
        }
    }

    #[tokio::test]
    async fn test_compact_data_set_attributes() {
        let cd = CompactData::with_default_obis();

        cd.set_attribute(2, DataObject::OctetString(vec![10, 20, 30]), None)
            .await
            .unwrap();
        assert_eq!(cd.buffer().await, vec![10, 20, 30]);

        cd.set_attribute(3, DataObject::Unsigned16(512), None)
            .await
            .unwrap();
        assert_eq!(cd.buffer_size().await, 512);
    }

    #[tokio::test]
    async fn test_compact_data_set_capture_time() {
        let cd = CompactData::with_default_obis();
        cd.set_attribute(4, DataObject::Integer64(1609459200), None)
            .await
            .unwrap();
        assert_eq!(cd.capture_time().await, Some(1609459200));
    }

    #[tokio::test]
    async fn test_compact_data_read_only_logical_name() {
        let cd = CompactData::with_default_obis();
        let result = cd
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 92, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compact_data_invalid_attribute() {
        let cd = CompactData::with_default_obis();
        let result = cd.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compact_data_invalid_method() {
        let cd = CompactData::with_default_obis();
        let result = cd.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compact_data_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 92, 0, 0, 1);
        let cd = CompactData::new(obis);
        assert_eq!(cd.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_compact_data_set_buffer_size_u8() {
        let cd = CompactData::with_default_obis();
        cd.set_attribute(3, DataObject::Unsigned8(200), None)
            .await
            .unwrap();
        assert_eq!(cd.buffer_size().await, 200);
    }
}
