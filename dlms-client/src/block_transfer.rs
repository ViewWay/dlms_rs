//! Block transfer service for DLMS/COSEM client
//!
//! This module provides functionality for writing large attribute values
//! using SetRequest block transfer (WithFirstDataBlock/WithDataBlock).

use dlms_core::{DlmsResult, ObisCode, DataObject};
use dlms_application::pdu::{
    SetRequest, SetResponse,
    CosemAttributeDescriptor, InvokeIdAndPriority, SetDataResult,
};
use std::time::Duration;

/// Block transfer configuration
#[derive(Debug, Clone)]
pub struct BlockTransferConfig {
    /// Maximum block size in bytes
    pub max_block_size: usize,
    /// Request timeout
    pub timeout: Duration,
    /// Maximum number of retries per block
    pub max_retries: u32,
}

impl Default for BlockTransferConfig {
    fn default() -> Self {
        Self {
            max_block_size: 512,
            timeout: Duration::from_secs(10),
            max_retries: 3,
        }
    }
}

impl BlockTransferConfig {
    /// Create a new block transfer config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum block size
    pub fn with_max_block_size(mut self, size: usize) -> Self {
        self.max_block_size = size;
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the maximum retries
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }
}

/// Block transfer writer
///
/// Provides methods for writing large attribute values using block transfer.
pub struct BlockTransferWriter<'a> {
    /// Reference to the connection
    connection: &'a mut (dyn crate::Connection + Send + Sync),
    /// Configuration
    config: BlockTransferConfig,
}

impl<'a> BlockTransferWriter<'a> {
    /// Create a new block transfer writer
    ///
    /// # Arguments
    /// * `connection` - Reference to the connection
    pub fn new(connection: &'a mut (dyn crate::Connection + Send + Sync)) -> Self {
        Self {
            connection,
            config: BlockTransferConfig::default(),
        }
    }

    /// Create a new block transfer writer with custom config
    ///
    /// # Arguments
    /// * `connection` - Reference to the connection
    /// * `config` - Block transfer configuration
    pub fn with_config(
        connection: &'a mut (dyn crate::Connection + Send + Sync),
        config: BlockTransferConfig,
    ) -> Self {
        Self { connection, config }
    }

    /// Set the configuration
    pub fn with_config_mut(mut self, config: BlockTransferConfig) -> Self {
        self.config = config;
        self
    }

    /// Write a large attribute value using block transfer
    ///
    /// This method automatically handles block transfer when the value
    /// exceeds the maximum block size.
    ///
    /// # Arguments
    /// * `obis_code` - OBIS code of the object
    /// * `class_id` - Class ID of the object
    /// * `attribute_id` - Attribute ID to write
    /// * `value` - Value to write (as DataObject)
    ///
    /// # Returns
    /// Ok(()) if successful, Err otherwise
    pub async fn write_attribute(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        attribute_id: u8,
        value: DataObject,
    ) -> DlmsResult<()> {
        // Encode the value to bytes
        let value_bytes = self.encode_data_object(&value)?;

        // If value fits in a single block, use normal SET
        if value_bytes.len() <= self.config.max_block_size {
            return self
                .connection
                .set_attribute(obis_code, class_id, attribute_id, value)
                .await;
        }

        // Use block transfer for large values
        self.write_attribute_blocks(obis_code, class_id, attribute_id, &value_bytes).await
    }

    /// Write a large byte array using block transfer
    ///
    /// This is useful for writing raw data that's already encoded.
    ///
    /// # Arguments
    /// * `obis_code` - OBIS code of the object
    /// * `class_id` - Class ID of the object
    /// * `attribute_id` - Attribute ID to write
    /// * `data` - Raw data bytes to write
    ///
    /// # Returns
    /// Ok(()) if successful, Err otherwise
    pub async fn write_attribute_raw(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        attribute_id: u8,
        data: &[u8],
    ) -> DlmsResult<()> {
        // If data fits in a single block, encode as DataObject and use normal SET
        if data.len() <= self.config.max_block_size {
            let value = DataObject::OctetString(data.to_vec());
            return self
                .connection
                .set_attribute(obis_code, class_id, attribute_id, value)
                .await;
        }

        // Use block transfer for large values
        self.write_attribute_blocks(obis_code, class_id, attribute_id, data).await
    }

    /// Internal method to write data using block transfer
    async fn write_attribute_blocks(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        attribute_id: u8,
        data: &[u8],
    ) -> DlmsResult<()> {
        // Create attribute descriptor
        let descriptor = CosemAttributeDescriptor::new_logical_name(class_id, obis_code, attribute_id)?;

        // Calculate number of blocks
        let total_blocks = (data.len() + self.config.max_block_size - 1) / self.config.max_block_size;
        let mut block_number: u32 = 0;

        // Split data into blocks
        for (i, chunk) in data.chunks(self.config.max_block_size).enumerate() {
            let is_last_block = i == total_blocks - 1;

            if i == 0 {
                // First block - use WithFirstDataBlock
                self.send_first_block(
                    descriptor.clone(),
                    block_number,
                    is_last_block,
                    chunk,
                ).await?;
            } else {
                // Subsequent blocks - use WithDataBlock
                self.send_next_block(
                    block_number,
                    is_last_block,
                    chunk,
                ).await?;
            }

            block_number += 1;
        }

        Ok(())
    }

    /// Send the first data block
    async fn send_first_block(
        &mut self,
        descriptor: CosemAttributeDescriptor,
        block_number: u32,
        last_block: bool,
        block_data: &[u8],
    ) -> DlmsResult<()> {
        let invoke_id = InvokeIdAndPriority::new(1, false)?;

        let request = SetRequest::new_with_first_data_block(
            invoke_id,
            descriptor,
            None, // access_selection
            block_number,
            last_block,
            block_data.to_vec(),
        );

        self.send_and_wait(request, last_block).await
    }

    /// Send a subsequent data block
    async fn send_next_block(
        &mut self,
        block_number: u32,
        last_block: bool,
        block_data: &[u8],
    ) -> DlmsResult<()> {
        let invoke_id = InvokeIdAndPriority::new(1, false)?;

        let request = SetRequest::new_with_data_block(
            invoke_id,
            block_number,
            last_block,
            block_data.to_vec(),
        );

        self.send_and_wait(request, last_block).await
    }

    /// Send request and process response
    async fn send_and_wait(
        &mut self,
        request: SetRequest,
        _expected_last_block: bool,
    ) -> DlmsResult<()> {
        let request_data = request.encode()?;

        // Send request
        let response_data = self.connection.send_request(
            &request_data,
            Some(self.config.timeout),
        ).await?;

        // Parse response
        match SetResponse::decode(&response_data)? {
            SetResponse::Normal(normal) => {
                // Final response - operation complete
                match normal.result {
                    SetDataResult::Success => Ok(()),
                    SetDataResult::DataAccessResult(code) => {
                        Err(dlms_core::DlmsError::InvalidData(format!(
                            "Block transfer failed with error code {}",
                            code
                        )))
                    }
                }
            }
            SetResponse::WithDataBlock { .. } => {
                // Block acknowledged, more blocks expected
                Ok(())
            }
            SetResponse::WithList(_) => {
                Err(dlms_core::DlmsError::InvalidData(
                    "Unexpected WithList response during block transfer".to_string()
                ))
            }
        }
    }

    /// Encode a DataObject to bytes
    fn encode_data_object(&self, value: &DataObject) -> DlmsResult<Vec<u8>> {
        use dlms_asn1::AxdrEncoder;

        let mut encoder = AxdrEncoder::new();
        encoder.encode_data_object(value)?;
        Ok(encoder.into_bytes())
    }

    /// Write multiple large attributes using block transfer
    ///
    /// # Arguments
    /// * `attributes` - List of (obis_code, class_id, attribute_id, value) tuples
    ///
    /// # Returns
    /// Ok(()) if all writes succeeded, Err with details of any failures
    pub async fn write_attributes(
        &mut self,
        attributes: Vec<(ObisCode, u16, u8, DataObject)>,
    ) -> DlmsResult<()> {
        for (obis_code, class_id, attribute_id, value) in attributes {
            self.write_attribute(obis_code, class_id, attribute_id, value).await?;
        }
        Ok(())
    }
}

/// Helper trait for types that can be written using block transfer
#[allow(async_fn_in_trait)]
pub trait BlockTransferWritable<'a> {
    /// Write using block transfer
    async fn write_with_block_transfer(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        attribute_id: u8,
        value: DataObject,
    ) -> DlmsResult<()>;
}

impl<'a> BlockTransferWritable<'a> for BlockTransferWriter<'a> {
    async fn write_with_block_transfer(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        attribute_id: u8,
        value: DataObject,
    ) -> DlmsResult<()> {
        self.write_attribute(obis_code, class_id, attribute_id, value).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_transfer_config_default() {
        let config = BlockTransferConfig::default();
        assert_eq!(config.max_block_size, 512);
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_block_transfer_config_builder() {
        let config = BlockTransferConfig::new()
            .with_max_block_size(1024)
            .with_timeout(Duration::from_secs(30))
            .with_max_retries(5);

        assert_eq!(config.max_block_size, 1024);
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_data_chunking() {
        let data = vec![0u8; 1500]; // 1500 bytes
        let chunk_size = 512;

        let chunks: Vec<_> = data.chunks(chunk_size).collect();
        assert_eq!(chunks.len(), 3); // 512 + 512 + 476
        assert_eq!(chunks[0].len(), 512);
        assert_eq!(chunks[1].len(), 512);
        assert_eq!(chunks[2].len(), 476);
    }
}
