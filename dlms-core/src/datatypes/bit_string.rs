//! Bit string type for DLMS/COSEM protocol

use crate::error::{DlmsError, DlmsResult};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Arbitrary string of bits (zeros and ones). A bit string value can have any length including zero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BitString {
    bytes: Vec<u8>,
    num_bits: usize,
}

impl BitString {
    /// Construct a new bit string object.
    ///
    /// # Arguments
    ///
    /// * `bit_string` - The bit string as a byte array
    /// * `num_bits` - The number of bits
    ///
    /// # Returns
    ///
    /// Returns `Ok(BitString)` if the parameters are valid, `Err(DlmsError)` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `bit_string` is empty and `num_bits` > 0
    /// - `num_bits > bit_string.len() * 8`
    pub fn new(bit_string: Vec<u8>, num_bits: usize) -> DlmsResult<Self> {
        if num_bits > bit_string.len() * 8 {
            return Err(DlmsError::InvalidData(format!(
                "bit_string is too short to hold all bits. Need {} bytes for {} bits",
                (num_bits + 7) / 8,
                num_bits
            )));
        }

        Ok(Self {
            bytes: bit_string,
            num_bits,
        })
    }

    /// Get the bit string as byte array.
    ///
    /// # Returns
    ///
    /// A reference to the byte array containing the bit string.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The number of bits in the byte array.
    ///
    /// # Returns
    ///
    /// The number of bits.
    pub fn num_bits(&self) -> usize {
        self.num_bits
    }

    /// Get a copy of the bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    /// Create a BitString from bytes (alias for new)
    ///
    /// This is a convenience method that creates a BitString from a byte array
    /// and the number of bits.
    pub fn from_bytes(bytes: Vec<u8>, num_bits: usize) -> DlmsResult<Self> {
        Self::new(bytes, num_bits)
    }

    /// Get the bit at a specific position
    ///
    /// # Arguments
    /// * `index` - The bit index (0-based)
    ///
    /// # Returns
    /// * `true` if the bit is set, `false` otherwise
    /// * `Err` if the index is out of bounds
    pub fn get_bit(&self, index: usize) -> DlmsResult<bool> {
        if index >= self.num_bits {
            return Err(DlmsError::InvalidData(format!(
                "Bit index {} out of bounds (num_bits: {})",
                index, self.num_bits
            )));
        }
        let byte_index = index / 8;
        let bit_index = 7 - (index % 8); // MSB first
        Ok((self.bytes[byte_index] >> bit_index) & 1 == 1)
    }

    /// Set the bit at a specific position
    ///
    /// # Arguments
    /// * `index` - The bit index (0-based)
    /// * `value` - The value to set (true = 1, false = 0)
    ///
    /// # Returns
    /// * `Err` if the index is out of bounds
    pub fn set_bit(&mut self, index: usize, value: bool) -> DlmsResult<()> {
        if index >= self.num_bits {
            return Err(DlmsError::InvalidData(format!(
                "Bit index {} out of bounds (num_bits: {})",
                index, self.num_bits
            )));
        }
        let byte_index = index / 8;
        let bit_index = 7 - (index % 8); // MSB first
        if value {
            self.bytes[byte_index] |= 1 << bit_index;
        } else {
            self.bytes[byte_index] &= !(1 << bit_index);
        }
        Ok(())
    }
}

impl fmt::Display for BitString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.bytes {
            write!(f, "{:02X} ", byte)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_string_new() {
        let bytes = vec![0xFF, 0x00, 0xAA];
        let bit_string = BitString::new(bytes.clone(), 24).unwrap();
        assert_eq!(bit_string.as_bytes(), &bytes);
        assert_eq!(bit_string.num_bits(), 24);
    }

    #[test]
    fn test_bit_string_invalid() {
        let bytes = vec![0xFF];
        let result = BitString::new(bytes, 16);
        assert!(result.is_err());
    }

    #[test]
    fn test_bit_string_partial_byte() {
        let bytes = vec![0xFF];
        let bit_string = BitString::new(bytes, 4).unwrap();
        assert_eq!(bit_string.num_bits(), 4);
    }
}
