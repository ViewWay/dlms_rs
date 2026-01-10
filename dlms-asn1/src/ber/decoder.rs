//! BER decoder for ASN.1 structures
//!
//! This module provides decoding functionality for ASN.1 values using BER encoding rules.
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use dlms_asn1::ber::BerDecoder;
//!
//! let mut decoder = BerDecoder::new(&data);
//! let integer = decoder.decode_integer()?;
//! ```

use crate::error::{DlmsError, DlmsResult};
use crate::ber::types::{BerTag, BerLength};

/// BER decoder for ASN.1 structures
///
/// This decoder follows the BER decoding rules as specified in ITU-T X.690.
/// It reads TLV (Tag-Length-Value) triplets from a byte buffer.
///
/// # Position Tracking
///
/// The decoder maintains a position pointer that advances as data is decoded.
/// This allows sequential decoding of multiple values from the same buffer.
///
/// # Error Handling
///
/// All decoding operations return `Result` types. Errors can occur due to:
/// - Buffer underflow (not enough data)
/// - Invalid encoding format
/// - Integer overflow
/// - Unexpected tag values
pub struct BerDecoder<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> BerDecoder<'a> {
    /// Create a new BER decoder
    ///
    /// # Arguments
    /// * `buffer` - Buffer containing BER-encoded data
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    /// Get current position in buffer
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get remaining bytes
    pub fn remaining(&self) -> usize {
        self.buffer.len().saturating_sub(self.position)
    }

    /// Check if there is more data to decode
    pub fn has_remaining(&self) -> bool {
        self.position < self.buffer.len()
    }

    /// Read a byte from the buffer
    ///
    /// # Returns
    /// Returns the byte at current position, advancing the position.
    ///
    /// # Error Handling
    /// Returns error if buffer is exhausted.
    fn read_byte(&mut self) -> DlmsResult<u8> {
        if self.position >= self.buffer.len() {
            return Err(DlmsError::InvalidData(
                "Buffer exhausted while reading byte".to_string(),
            ));
        }
        let byte = self.buffer[self.position];
        self.position += 1;
        Ok(byte)
    }

    /// Read multiple bytes from the buffer
    ///
    /// # Arguments
    /// * `count` - Number of bytes to read
    ///
    /// # Returns
    /// Returns a slice of the requested bytes.
    ///
    /// # Error Handling
    /// Returns error if buffer doesn't have enough bytes.
    fn read_bytes(&mut self, count: usize) -> DlmsResult<&'a [u8]> {
        if self.position + count > self.buffer.len() {
            return Err(DlmsError::InvalidData(format!(
                "Buffer exhausted: need {} bytes, have {}",
                count,
                self.buffer.len() - self.position
            )));
        }
        let start = self.position;
        self.position += count;
        Ok(&self.buffer[start..start + count])
    }

    /// Decode a TLV (Tag-Length-Value) triplet
    ///
    /// # Returns
    /// Returns `Ok((tag, value_bytes, total_bytes_consumed))` if successful.
    ///
    /// # Decoding Process
    /// 1. Decode tag
    /// 2. Decode length
    /// 3. Read value bytes
    ///
    /// # Why This Method?
    /// This is the fundamental BER decoding operation. All other decoding methods
    /// use this internally to ensure consistent decoding format.
    pub fn decode_tlv(&mut self) -> DlmsResult<(BerTag, &'a [u8], usize)> {
        let start_pos = self.position;

        // Decode tag
        let (tag, tag_bytes) = BerTag::decode(&self.buffer[self.position..])?;
        self.position += tag_bytes;

        // Decode length
        let (length, length_bytes) = BerLength::decode(&self.buffer[self.position..])?;
        self.position += length_bytes;

        // Read value
        let value_len = length.value();
        let value = self.read_bytes(value_len)?;

        let total_bytes = self.position - start_pos;
        Ok((tag, value, total_bytes))
    }

    /// Decode an INTEGER
    ///
    /// # Returns
    /// Returns the decoded integer value.
    ///
    /// # Decoding Format
    /// INTEGER is decoded from:
    /// - Tag: Universal, Primitive, tag 2
    /// - Length: Number of bytes in value
    /// - Value: Two's complement representation (big-endian)
    ///
    /// # Error Handling
    /// Returns error if:
    /// - Tag is not INTEGER
    /// - Value is too large for i64
    pub fn decode_integer(&mut self) -> DlmsResult<i64> {
        let (tag, value, _) = self.decode_tlv()?;

        // Verify tag
        if tag.class() != crate::ber::types::BerTagClass::Universal
            || tag.is_constructed()
            || tag.number() != 2
        {
            return Err(DlmsError::InvalidData(format!(
                "Expected INTEGER tag, got {:?}",
                tag
            )));
        }

        // Decode two's complement value
        self.decode_integer_value(value)
    }

    /// Decode integer value (helper method)
    ///
    /// Converts big-endian two's complement bytes to i64.
    fn decode_integer_value(&self, bytes: &[u8]) -> DlmsResult<i64> {
        if bytes.is_empty() {
            return Err(DlmsError::InvalidData("Empty integer encoding".to_string()));
        }

        if bytes.len() > 8 {
            return Err(DlmsError::InvalidData(format!(
                "Integer too large: {} bytes (max 8)",
                bytes.len()
            )));
        }

        // Check sign bit (MSB of first byte)
        let is_negative = (bytes[0] & 0x80) != 0;

        // Build value (big-endian)
        let mut value = 0i64;
        for &byte in bytes {
            value = (value << 8) | (byte as i64);
        }

        // Sign extend if negative
        if is_negative {
            // Sign extend: fill upper bits with 1s
            let shift = 64 - (bytes.len() * 8);
            value = (value << shift) >> shift;
        }

        Ok(value)
    }

    /// Decode an OCTET STRING
    ///
    /// # Returns
    /// Returns the decoded octet string bytes.
    ///
    /// # Decoding Format
    /// OCTET STRING is decoded from:
    /// - Tag: Universal, Primitive, tag 4
    /// - Length: Number of bytes
    /// - Value: Raw bytes
    ///
    /// # Error Handling
    /// Returns error if tag is not OCTET STRING.
    pub fn decode_octet_string(&mut self) -> DlmsResult<Vec<u8>> {
        let (tag, value, _) = self.decode_tlv()?;

        // Verify tag
        if tag.class() != crate::ber::types::BerTagClass::Universal
            || tag.is_constructed()
            || tag.number() != 4
        {
            return Err(DlmsError::InvalidData(format!(
                "Expected OCTET STRING tag, got {:?}",
                tag
            )));
        }

        Ok(value.to_vec())
    }

    /// Decode a BIT STRING
    ///
    /// # Returns
    /// Returns `(bytes, num_bits, unused_bits)` where:
    /// - `bytes`: Bit string bytes
    /// - `num_bits`: Total number of bits
    /// - `unused_bits`: Number of unused bits in last byte (0-7)
    ///
    /// # Decoding Format
    /// BIT STRING is decoded from:
    /// - Tag: Universal, Primitive, tag 3
    /// - Length: Number of bytes + 1 (for unused bits byte)
    /// - Value: Unused bits count (1 byte) + bit string bytes
    ///
    /// # Error Handling
    /// Returns error if tag is not BIT STRING or unused bits > 7.
    pub fn decode_bit_string(&mut self) -> DlmsResult<(Vec<u8>, usize, u8)> {
        let (tag, value, _) = self.decode_tlv()?;

        // Verify tag
        if tag.class() != crate::ber::types::BerTagClass::Universal
            || tag.is_constructed()
            || tag.number() != 3
        {
            return Err(DlmsError::InvalidData(format!(
                "Expected BIT STRING tag, got {:?}",
                tag
            )));
        }

        if value.is_empty() {
            return Err(DlmsError::InvalidData("Empty bit string encoding".to_string()));
        }

        // First byte is unused bits count
        let unused_bits = value[0];
        if unused_bits > 7 {
            return Err(DlmsError::InvalidData(format!(
                "Invalid unused bits: {} (must be 0-7)",
                unused_bits
            )));
        }

        // Remaining bytes are the bit string
        let bytes = value[1..].to_vec();
        let num_bits = if bytes.is_empty() {
            0
        } else {
            bytes.len() * 8 - (unused_bits as usize)
        };

        Ok((bytes, num_bits, unused_bits))
    }

    /// Decode an OBJECT IDENTIFIER
    ///
    /// # Returns
    /// Returns the decoded OID components as a vector.
    ///
    /// # Decoding Format
    /// OBJECT IDENTIFIER is decoded from:
    /// - Tag: Universal, Primitive, tag 6
    /// - Length: Number of bytes in encoded value
    /// - Value: Encoded OID (first two components encoded as 40*X + Y)
    ///
    /// # OID Decoding Rules
    /// - First byte: 40*X + Y, decode to get first two components
    /// - Remaining bytes: Base-128 encoded components
    ///
    /// # Error Handling
    /// Returns error if tag is not OBJECT IDENTIFIER or encoding is invalid.
    pub fn decode_object_identifier(&mut self) -> DlmsResult<Vec<u32>> {
        let (tag, value, _) = self.decode_tlv()?;

        // Verify tag
        if tag.class() != crate::ber::types::BerTagClass::Universal
            || tag.is_constructed()
            || tag.number() != 6
        {
            return Err(DlmsError::InvalidData(format!(
                "Expected OBJECT IDENTIFIER tag, got {:?}",
                tag
            )));
        }

        if value.is_empty() {
            return Err(DlmsError::InvalidData("Empty object identifier encoding".to_string()));
        }

        // Decode first two components from first byte
        let first_byte = value[0];
        let first_component = (first_byte / 40) as u32;
        let second_component = (first_byte % 40) as u32;

        let mut oid = vec![first_component, second_component];

        // Decode remaining components (base-128)
        let mut pos = 1;
        while pos < value.len() {
            let mut component = 0u32;
            let mut has_more = true;

            while has_more && pos < value.len() {
                let byte = value[pos];
                has_more = (byte & 0x80) != 0;
                component = component
                    .checked_mul(128)
                    .and_then(|x| x.checked_add((byte & 0x7F) as u32))
                    .ok_or_else(|| DlmsError::InvalidData("OID component overflow".to_string()))?;
                pos += 1;

                // Prevent infinite loop
                if pos > value.len() + 10 {
                    return Err(DlmsError::InvalidData("Invalid OID encoding".to_string()));
                }
            }

            oid.push(component);
        }

        Ok(oid)
    }

    /// Decode a SEQUENCE (constructed)
    ///
    /// # Returns
    /// Returns the decoded sequence element bytes (each element is a TLV).
    ///
    /// # Decoding Format
    /// SEQUENCE is decoded from:
    /// - Tag: Universal, Constructed, tag 16
    /// - Length: Total length of all elements
    /// - Value: Concatenated element TLVs
    ///
    /// # Error Handling
    /// Returns error if tag is not SEQUENCE or not constructed.
    pub fn decode_sequence(&mut self) -> DlmsResult<&'a [u8]> {
        let (tag, value, _) = self.decode_tlv()?;

        // Verify tag
        if tag.class() != crate::ber::types::BerTagClass::Universal
            || !tag.is_constructed()
            || tag.number() != 16
        {
            return Err(DlmsError::InvalidData(format!(
                "Expected SEQUENCE tag, got {:?}",
                tag
            )));
        }

        Ok(value)
    }

    /// Decode a context-specific tag
    ///
    /// # Arguments
    /// * `expected_tag_number` - Expected context-specific tag number
    /// * `constructed` - Whether this is expected to be constructed
    ///
    /// # Returns
    /// Returns the decoded value bytes.
    ///
    /// # Error Handling
    /// Returns error if tag doesn't match expected values.
    pub fn decode_context_specific(&mut self, expected_tag_number: u32, constructed: bool) -> DlmsResult<&'a [u8]> {
        let (tag, value, _) = self.decode_tlv()?;

        // Verify tag
        if tag.class() != crate::ber::types::BerTagClass::ContextSpecific
            || tag.is_constructed() != constructed
            || tag.number() != expected_tag_number
        {
            return Err(DlmsError::InvalidData(format!(
                "Expected context-specific tag {}, got {:?}",
                expected_tag_number, tag
            )));
        }

        Ok(value)
    }

    /// Decode an application tag
    ///
    /// # Arguments
    /// * `expected_tag_number` - Expected application tag number
    /// * `constructed` - Whether this is expected to be constructed
    ///
    /// # Returns
    /// Returns the decoded value bytes.
    ///
    /// # Error Handling
    /// Returns error if tag doesn't match expected values.
    pub fn decode_application(&mut self, expected_tag_number: u32, constructed: bool) -> DlmsResult<&'a [u8]> {
        let (tag, value, _) = self.decode_tlv()?;

        // Verify tag
        if tag.class() != crate::ber::types::BerTagClass::Application
            || tag.is_constructed() != constructed
            || tag.number() != expected_tag_number
        {
            return Err(DlmsError::InvalidData(format!(
                "Expected application tag {}, got {:?}",
                expected_tag_number, tag
            )));
        }

        Ok(value)
    }

    /// Skip a TLV (useful for skipping optional fields)
    ///
    /// # Returns
    /// Returns the number of bytes skipped.
    pub fn skip_tlv(&mut self) -> DlmsResult<usize> {
        let (_, _, bytes_consumed) = self.decode_tlv()?;
        Ok(bytes_consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_integer() {
        // Encode integer 12345
        use super::super::encoder::BerEncoder;
        let mut encoder = BerEncoder::new();
        encoder.encode_integer(12345).unwrap();
        let encoded = encoder.into_bytes();

        // Decode it
        let mut decoder = BerDecoder::new(&encoded);
        let value = decoder.decode_integer().unwrap();
        assert_eq!(value, 12345);
    }

    #[test]
    fn test_decode_octet_string() {
        // Encode octet string
        use super::super::encoder::BerEncoder;
        let mut encoder = BerEncoder::new();
        encoder.encode_octet_string(b"Hello").unwrap();
        let encoded = encoder.into_bytes();

        // Decode it
        let mut decoder = BerDecoder::new(&encoded);
        let value = decoder.decode_octet_string().unwrap();
        assert_eq!(value, b"Hello");
    }

    #[test]
    fn test_decode_object_identifier() {
        // Encode OID
        use super::super::encoder::BerEncoder;
        let mut encoder = BerEncoder::new();
        encoder.encode_object_identifier(&[1, 2, 840, 113549]).unwrap();
        let encoded = encoder.into_bytes();

        // Decode it
        let mut decoder = BerDecoder::new(&encoded);
        let oid = decoder.decode_object_identifier().unwrap();
        assert_eq!(oid, vec![1, 2, 840, 113549]);
    }
}
