//! BER encoder for ASN.1 structures
//!
//! This module provides encoding functionality for ASN.1 values using BER encoding rules.
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use dlms_asn1::ber::{BerEncoder, BerTag, BerTagClass};
//!
//! let mut encoder = BerEncoder::new();
//! encoder.encode_integer(12345)?;
//! let bytes = encoder.into_bytes();
//! ```

use crate::error::{DlmsError, DlmsResult};
use crate::ber::types::{BerTag, BerTagClass, BerLength};
use std::io::Write;

/// BER encoder for ASN.1 structures
///
/// This encoder follows the BER encoding rules as specified in ITU-T X.690.
/// Each encoded value consists of a TLV (Tag-Length-Value) triplet.
///
/// # Memory Management
///
/// The encoder uses a `Vec<u8>` buffer to accumulate encoded data. For
/// high-frequency operations, consider using `with_capacity()` to pre-allocate
/// buffer space.
///
/// # Error Handling
///
/// All encoding operations return `Result` types. Errors can occur due to:
/// - Buffer allocation failures
/// - Invalid input data
/// - Integer overflow
pub struct BerEncoder {
    buffer: Vec<u8>,
}

impl BerEncoder {
    /// Create a new BER encoder
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    /// Create a new BER encoder with initial capacity
    ///
    /// # Arguments
    /// * `capacity` - Initial buffer capacity in bytes
    ///
    /// # Why Pre-allocate?
    /// Pre-allocating the buffer can reduce memory reallocations during encoding,
    /// improving performance for large structures.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
        }
    }

    /// Encode a TLV (Tag-Length-Value) triplet
    ///
    /// # Arguments
    /// * `tag` - BER tag
    /// * `value` - Value bytes (already encoded)
    ///
    /// # Encoding Process
    /// 1. Encode tag
    /// 2. Encode length
    /// 3. Append value bytes
    ///
    /// # Why This Method?
    /// This is the fundamental BER encoding operation. All other encoding methods
    /// use this internally to ensure consistent encoding format.
    pub fn encode_tlv(&mut self, tag: &BerTag, value: &[u8]) -> DlmsResult<()> {
        // Encode tag
        self.buffer.extend_from_slice(&tag.encode());

        // Encode length
        let length = BerLength::new(value.len());
        self.buffer.extend_from_slice(&length.encode());

        // Append value
        self.buffer.extend_from_slice(value);

        Ok(())
    }

    /// Encode an INTEGER
    ///
    /// # Arguments
    /// * `value` - Integer value (signed, two's complement)
    ///
    /// # Encoding Format
    /// INTEGER is encoded as:
    /// - Tag: Universal, Primitive, tag 2
    /// - Length: Number of bytes in value
    /// - Value: Two's complement representation (big-endian, minimal encoding)
    ///
    /// # Minimal Encoding
    /// BER requires minimal encoding: the value must use the minimum number
    /// of bytes. For example, 127 should be encoded as 1 byte (0x7F), not 2 bytes (0x00 0x7F).
    ///
    /// # Why Two's Complement?
    /// Two's complement is the standard representation for signed integers,
    /// allowing efficient encoding of both positive and negative values.
    pub fn encode_integer(&mut self, value: i64) -> DlmsResult<()> {
        let tag = BerTag::universal(false, 2); // INTEGER tag

        // Encode value in two's complement (big-endian, minimal)
        let bytes = self.encode_integer_value(value);
        self.encode_tlv(&tag, &bytes)
    }

    /// Encode integer value (helper method)
    ///
    /// Returns minimal two's complement representation.
    fn encode_integer_value(&self, value: i64) -> Vec<u8> {
        if value == 0 {
            return vec![0];
        }

        // Calculate minimum number of bytes needed
        let mut bytes = Vec::new();
        let mut remaining = value;

        // Handle negative numbers
        if value < 0 {
            // For negative numbers, we need to ensure sign extension
            // Find the minimum number of bytes that preserves the sign
            let mut temp = value;
            while temp != -1 && temp != 0 {
                bytes.push((temp & 0xFF) as u8);
                temp >>= 8;
            }
            // If the most significant byte would be 0x00 (positive), we need an extra 0xFF
            if bytes.is_empty() || (bytes[bytes.len() - 1] & 0x80) == 0 {
                bytes.push(0xFF);
            }
        } else {
            // For positive numbers, find minimum bytes
            let mut temp = value;
            while temp > 0 {
                bytes.push((temp & 0xFF) as u8);
                temp >>= 8;
            }
            // If the most significant byte has bit 7 set, we need an extra 0x00
            if !bytes.is_empty() && (bytes[bytes.len() - 1] & 0x80) != 0 {
                bytes.push(0x00);
            }
        }

        bytes.reverse(); // Big-endian
        bytes
    }

    /// Encode an OCTET STRING
    ///
    /// # Arguments
    /// * `value` - Octet string bytes
    ///
    /// # Encoding Format
    /// OCTET STRING is encoded as:
    /// - Tag: Universal, Primitive, tag 4
    /// - Length: Number of bytes
    /// - Value: Raw bytes
    pub fn encode_octet_string(&mut self, value: &[u8]) -> DlmsResult<()> {
        let tag = BerTag::universal(false, 4); // OCTET STRING tag
        self.encode_tlv(&tag, value)
    }

    /// Encode a BIT STRING
    ///
    /// # Arguments
    /// * `value` - Bit string bytes
    /// * `num_bits` - Number of bits (may be less than 8 * bytes.len())
    /// * `unused_bits` - Number of unused bits in the last byte (0-7)
    ///
    /// # Encoding Format
    /// BIT STRING is encoded as:
    /// - Tag: Universal, Primitive, tag 3
    /// - Length: Number of bytes + 1 (for unused bits byte)
    /// - Value: Unused bits count (1 byte) + bit string bytes
    ///
    /// # Why Unused Bits?
    /// BIT STRING can have a length that is not a multiple of 8. The unused
    /// bits byte indicates how many bits in the last byte are not used.
    pub fn encode_bit_string(&mut self, value: &[u8], num_bits: usize, unused_bits: u8) -> DlmsResult<()> {
        if unused_bits > 7 {
            return Err(DlmsError::InvalidData(
                "Unused bits must be 0-7".to_string(),
            ));
        }

        let tag = BerTag::universal(false, 3); // BIT STRING tag

        // Build value: unused bits byte + bit string bytes
        let mut bytes = vec![unused_bits];
        bytes.extend_from_slice(value);

        self.encode_tlv(&tag, &bytes)
    }

    /// Encode an OBJECT IDENTIFIER
    ///
    /// # Arguments
    /// * `oid` - Object identifier components (e.g., [1, 2, 840, 113549] for RSA)
    ///
    /// # Encoding Format
    /// OBJECT IDENTIFIER is encoded as:
    /// - Tag: Universal, Primitive, tag 6
    /// - Length: Number of bytes in encoded value
    /// - Value: Encoded OID (first two components encoded specially)
    ///
    /// # OID Encoding Rules
    /// - First two components (X.Y) are encoded as: 40*X + Y
    /// - Remaining components are encoded in base-128 (variable length)
    /// - Each byte has bit 7 set except the last byte of each component
    ///
    /// # Why Special Encoding?
    /// OID encoding is optimized for the common case where the first two
    /// components are small (typically 0-2), allowing efficient encoding.
    pub fn encode_object_identifier(&mut self, oid: &[u32]) -> DlmsResult<()> {
        if oid.len() < 2 {
            return Err(DlmsError::InvalidData(
                "Object identifier must have at least 2 components".to_string(),
            ));
        }

        let tag = BerTag::universal(false, 6); // OBJECT IDENTIFIER tag

        // Encode first two components: 40*X + Y
        let first_byte = 40u32
            .checked_mul(oid[0])
            .and_then(|x| x.checked_add(oid[1]))
            .ok_or_else(|| DlmsError::InvalidData("OID component too large".to_string()))?;

        let mut bytes = vec![first_byte as u8];

        // Encode remaining components in base-128
        for &component in &oid[2..] {
            let mut temp = component;
            let mut component_bytes = Vec::new();

            // Encode component (LSB first, then reverse)
            loop {
                component_bytes.push((temp & 0x7F) as u8);
                temp >>= 7;
                if temp == 0 {
                    break;
                }
            }

            // Reverse and set continuation bits
            for (i, &byte) in component_bytes.iter().rev().enumerate() {
                if i < component_bytes.len() - 1 {
                    bytes.push(byte | 0x80); // Set continuation bit
                } else {
                    bytes.push(byte); // Last byte, no continuation bit
                }
            }
        }

        self.encode_tlv(&tag, &bytes)
    }

    /// Encode a SEQUENCE (constructed)
    ///
    /// # Arguments
    /// * `elements` - Encoded element bytes (each element is already a TLV)
    ///
    /// # Encoding Format
    /// SEQUENCE is encoded as:
    /// - Tag: Universal, Constructed, tag 16
    /// - Length: Total length of all elements
    /// - Value: Concatenated element TLVs
    ///
    /// # Why Constructed?
    /// SEQUENCE is a constructed type because it contains other ASN.1 values.
    /// The constructed flag (bit 5) indicates that the value contains nested TLVs.
    pub fn encode_sequence(&mut self, elements: &[u8]) -> DlmsResult<()> {
        let tag = BerTag::universal(true, 16); // SEQUENCE tag (constructed)
        self.encode_tlv(&tag, elements)
    }

    /// Encode a SEQUENCE OF (constructed)
    ///
    /// Same as SEQUENCE, but for homogeneous sequences.
    pub fn encode_sequence_of(&mut self, elements: &[u8]) -> DlmsResult<()> {
        // SEQUENCE OF uses the same encoding as SEQUENCE
        self.encode_sequence(elements)
    }

    /// Encode a context-specific tag
    ///
    /// # Arguments
    /// * `tag_number` - Context-specific tag number
    /// * `value` - Encoded value bytes
    /// * `constructed` - Whether this is a constructed type
    ///
    /// # Usage
    /// Context-specific tags are used in SEQUENCE/SET to identify optional
    /// or alternative fields without requiring unique tag numbers globally.
    pub fn encode_context_specific(&mut self, tag_number: u32, value: &[u8], constructed: bool) -> DlmsResult<()> {
        let tag = BerTag::context_specific(constructed, tag_number);
        self.encode_tlv(&tag, value)
    }

    /// Encode an application tag
    ///
    /// # Arguments
    /// * `tag_number` - Application tag number
    /// * `value` - Encoded value bytes
    /// * `constructed` - Whether this is a constructed type
    ///
    /// # Usage
    /// Application tags are used for application-specific types, such as
    /// AARQ/AARE in ISO-ACSE (tags 0 and 1).
    pub fn encode_application(&mut self, tag_number: u32, value: &[u8], constructed: bool) -> DlmsResult<()> {
        let tag = BerTag::application(constructed, tag_number);
        self.encode_tlv(&tag, value)
    }

    /// Get the encoded bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }

    /// Get a reference to the encoded bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Clear the encoder buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl Default for BerEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_integer() {
        let mut encoder = BerEncoder::new();
        encoder.encode_integer(12345).unwrap();
        let bytes = encoder.into_bytes();
        // INTEGER tag (0x02) + length + value
        assert!(bytes.len() >= 3);
        assert_eq!(bytes[0], 0x02); // INTEGER tag
    }

    #[test]
    fn test_encode_octet_string() {
        let mut encoder = BerEncoder::new();
        encoder.encode_octet_string(b"Hello").unwrap();
        let bytes = encoder.into_bytes();
        assert_eq!(bytes[0], 0x04); // OCTET STRING tag
        assert_eq!(bytes[1], 5); // Length
    }

    #[test]
    fn test_encode_object_identifier() {
        let mut encoder = BerEncoder::new();
        encoder.encode_object_identifier(&[1, 2, 840, 113549]).unwrap();
        let bytes = encoder.into_bytes();
        assert_eq!(bytes[0], 0x06); // OBJECT IDENTIFIER tag
    }

    #[test]
    fn test_encode_sequence() {
        let mut encoder = BerEncoder::new();
        let mut element_encoder = BerEncoder::new();
        element_encoder.encode_integer(123).unwrap();
        encoder.encode_sequence(element_encoder.as_bytes()).unwrap();
        let bytes = encoder.into_bytes();
        assert_eq!(bytes[0] & 0x20, 0x20); // Constructed flag
        assert_eq!(bytes[0] & 0x1F, 16); // SEQUENCE tag
    }
}
