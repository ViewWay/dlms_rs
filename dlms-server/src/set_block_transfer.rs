//! SET Block Transfer State Management
//!
//! This module provides state management for SET block transfer operations.
//!
//! When a client sends a large attribute value using SetRequest::WithFirstDataBlock
//! and SetRequest::WithDataBlock, the server needs to accumulate the data blocks
//! until the complete value is received, then decode and set the attribute value.

use dlms_core::{DlmsResult, ObisCode};

/// Block transfer state for SET operations
///
/// Tracks the state of an in-progress SET block transfer, accumulating
/// data blocks until the complete value is received.
#[derive(Debug, Clone)]
pub struct SetBlockTransferState {
    /// Invoke ID of the original request
    invoke_id: u8,
    /// OBIS code of the object being written
    obis_code: ObisCode,
    /// Attribute ID being written
    attribute_id: u8,
    /// Accumulated data from all received blocks
    accumulated_data: Vec<u8>,
    /// Expected block size (bytes per block)
    block_size: usize,
    /// Current block number
    current_block: u32,
    /// Last block flag
    last_block: bool,
    /// Access selection (optional, from WithFirstDataBlock)
    access_selection: Option<Vec<u8>>,
}

impl SetBlockTransferState {
    /// Create a new SET block transfer state
    ///
    /// # Arguments
    /// * `invoke_id` - Invoke ID of the original request
    /// * `obis_code` - OBIS code of the object being written
    /// * `attribute_id` - Attribute ID being written
    /// * `first_block_data` - First block data
    /// * `block_size` - Expected block size in bytes
    pub fn new(
        invoke_id: u8,
        obis_code: ObisCode,
        attribute_id: u8,
        first_block_data: Vec<u8>,
        block_size: usize,
    ) -> Self {
        Self {
            invoke_id,
            obis_code,
            attribute_id,
            accumulated_data: first_block_data,
            block_size,
            current_block: 0,
            last_block: false, // Don't auto-detect - protocol controls this
            access_selection: None,
        }
    }

    /// Create a new SET block transfer state with access selection
    ///
    /// # Arguments
    /// * `invoke_id` - Invoke ID of the original request
    /// * `obis_code` - OBIS code of the object being written
    /// * `attribute_id` - Attribute ID being written
    /// * `first_block_data` - First block data
    /// * `block_size` - Expected block size in bytes
    /// * `access_selection` - Optional access selection data
    pub fn with_access_selection(
        invoke_id: u8,
        obis_code: ObisCode,
        attribute_id: u8,
        first_block_data: Vec<u8>,
        block_size: usize,
        access_selection: Vec<u8>,
    ) -> Self {
        let mut state = Self::new(invoke_id, obis_code, attribute_id, first_block_data, block_size);
        state.access_selection = Some(access_selection);
        state
    }

    /// Add a new data block to the transfer
    ///
    /// # Arguments
    /// * `block_number` - Block number (must be current_block + 1)
    /// * `block_data` - Block data to append
    /// * `last_block` - Last block flag
    ///
    /// # Returns
    /// Ok(()) if successful, Err if block number is invalid
    pub fn add_block(&mut self, block_number: u32, block_data: &[u8], last_block: bool) -> DlmsResult<()> {
        // Validate block number
        if block_number != self.current_block + 1 {
            return Err(dlms_core::DlmsError::InvalidData(format!(
                "SET block transfer: Expected block {}, got {}",
                self.current_block + 1,
                block_number
            )));
        }

        // Append data
        self.accumulated_data.extend_from_slice(block_data);
        self.current_block = block_number;
        self.last_block = last_block;

        Ok(())
    }

    /// Check if the transfer is complete
    ///
    /// # Returns
    /// true if all blocks have been received
    pub fn is_complete(&self) -> bool {
        self.last_block
    }

    /// Get the accumulated data
    ///
    /// # Returns
    /// Reference to the accumulated data bytes
    pub fn accumulated_data(&self) -> &[u8] {
        &self.accumulated_data
    }

    /// Get the accumulated data as a vector (consuming the state)
    ///
    /// # Returns
    /// The accumulated data vector
    pub fn into_data(self) -> Vec<u8> {
        self.accumulated_data
    }

    /// Get the invoke ID
    pub fn invoke_id(&self) -> u8 {
        self.invoke_id
    }

    /// Get the OBIS code
    pub fn obis_code(&self) -> ObisCode {
        self.obis_code
    }

    /// Get the attribute ID
    pub fn attribute_id(&self) -> u8 {
        self.attribute_id
    }

    /// Get the current block number
    pub fn current_block(&self) -> u32 {
        self.current_block
    }

    /// Get the block size
    pub fn block_size(&self) -> usize {
        self.block_size
    }

    /// Get the last block flag
    pub fn last_block(&self) -> bool {
        self.last_block
    }

    /// Get the access selection data (if any)
    pub fn access_selection(&self) -> Option<&[u8]> {
        self.access_selection.as_deref()
    }

    /// Get the total size of accumulated data
    pub fn total_size(&self) -> usize {
        self.accumulated_data.len()
    }

    /// Get the number of blocks received
    pub fn blocks_received(&self) -> u32 {
        self.current_block + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_block_transfer_state_new() {
        let state = SetBlockTransferState::new(
            1,
            ObisCode::new(1, 1, 1, 1, 1, 255),
            2,
            vec![0x01, 0x02, 0x03],
            512,
        );

        assert_eq!(state.invoke_id(), 1);
        assert_eq!(state.current_block(), 0);
        assert_eq!(state.total_size(), 3);
        assert_eq!(state.blocks_received(), 1);
        // last_block is set to false by default unless data size < block_size
        // In this case, we don't auto-detect - caller must set last_block explicitly
        assert!(!state.last_block());
    }

    #[test]
    fn test_set_block_transfer_add_blocks() {
        let mut state = SetBlockTransferState::new(
            1,
            ObisCode::new(1, 1, 1, 1, 1, 255),
            2,
            vec![0x01, 0x02, 0x03],
            512,
        );

        // Add second block
        state.add_block(1, &[0x04, 0x05], false).unwrap();
        assert_eq!(state.current_block(), 1);
        assert_eq!(state.total_size(), 5);
        assert_eq!(state.blocks_received(), 2);

        // Add third (last) block
        state.add_block(2, &[0x06], true).unwrap();
        assert_eq!(state.current_block(), 2);
        assert_eq!(state.total_size(), 6);
        assert!(state.last_block());
        assert!(state.is_complete());
    }

    #[test]
    fn test_set_block_transfer_invalid_block_number() {
        let mut state = SetBlockTransferState::new(
            1,
            ObisCode::new(1, 1, 1, 1, 1, 255),
            2,
            vec![0x01, 0x02, 0x03],
            512,
        );

        // Try to add block 3 instead of block 1
        let result = state.add_block(3, &[0x04, 0x05], false);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_block_transfer_with_access_selection() {
        let state = SetBlockTransferState::with_access_selection(
            1,
            ObisCode::new(1, 1, 1, 1, 1, 255),
            2,
            vec![0x01, 0x02, 0x03],
            512,
            vec![0xAA, 0xBB],
        );

        assert_eq!(state.access_selection(), Some(&[0xAA, 0xBB][..]));
    }

    #[test]
    fn test_set_block_transfer_into_data() {
        let state = SetBlockTransferState::new(
            1,
            ObisCode::new(1, 1, 1, 1, 1, 255),
            2,
            vec![0x01, 0x02, 0x03, 0x04],
            512,
        );

        let data = state.into_data();
        assert_eq!(data, vec![0x01, 0x02, 0x03, 0x04]);
    }
}
