//! BER encoding types (Tag, Length, etc.)

use crate::error::{DlmsError, DlmsResult};

/// BER Tag Class
///
/// ASN.1 defines four tag classes:
/// - **Universal**: Standard ASN.1 types (INTEGER, OCTET STRING, etc.)
/// - **Application**: Application-specific types
/// - **Context-specific**: Context-dependent types (used in SEQUENCE/SET)
/// - **Private**: Private/implementation-specific types
///
/// # Why Four Classes?
/// Tag classes allow different applications to use the same tag numbers
/// without conflicts. For example, tag 0 in Universal class is BOOLEAN,
/// but tag 0 in Application class can be a different type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BerTagClass {
    /// Universal class (00)
    Universal = 0,
    /// Application class (01)
    Application = 1,
    /// Context-specific class (10)
    ContextSpecific = 2,
    /// Private class (11)
    Private = 3,
}

impl BerTagClass {
    /// Get tag class from bits (bits 7-6 of tag byte)
    pub fn from_bits(bits: u8) -> DlmsResult<Self> {
        match (bits >> 6) & 0x03 {
            0 => Ok(BerTagClass::Universal),
            1 => Ok(BerTagClass::Application),
            2 => Ok(BerTagClass::ContextSpecific),
            3 => Ok(BerTagClass::Private),
            _ => unreachable!(), // Only 2 bits, so 0-3 are the only possibilities
        }
    }

    /// Convert tag class to bits (for encoding)
    pub fn to_bits(self) -> u8 {
        (self as u8) << 6
    }
}

/// BER Tag
///
/// A BER tag identifies the type of an ASN.1 value. It consists of:
/// - **Class**: Universal, Application, Context-specific, or Private
/// - **Constructed/Primitive**: Whether the value is constructed (contains other values)
/// - **Tag Number**: The actual tag number (0-30 for short form, or extended)
///
/// # Encoding Format
///
/// Short form (tag number 0-30):
/// ```
/// Bits: 8 7 6 5 4 3 2 1
///       C C P T T T T T
/// ```
///
/// Extended form (tag number > 30):
/// ```
/// First byte:  C C P 1 1 1 1 1  (all tag bits set to 1)
/// Following bytes: 1 T T T T T T T  (continuation bytes, last byte has bit 7 = 0)
/// ```
///
/// # Why This Design?
/// The tag encoding allows efficient representation of common tags (0-30)
/// in a single byte, while supporting larger tag numbers when needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BerTag {
    /// Tag class
    class: BerTagClass,
    /// Whether this is a constructed type
    constructed: bool,
    /// Tag number
    number: u32,
}

impl BerTag {
    /// Create a new BER tag
    ///
    /// # Arguments
    /// * `class` - Tag class
    /// * `constructed` - Whether this is a constructed type
    /// * `number` - Tag number
    ///
    /// # Returns
    /// Returns `Ok(BerTag)` if valid, `Err` otherwise
    pub fn new(class: BerTagClass, constructed: bool, number: u32) -> Self {
        Self {
            class,
            constructed,
            number,
        }
    }

    /// Create a Universal class tag
    pub fn universal(constructed: bool, number: u32) -> Self {
        Self::new(BerTagClass::Universal, constructed, number)
    }

    /// Create an Application class tag
    pub fn application(constructed: bool, number: u32) -> Self {
        Self::new(BerTagClass::Application, constructed, number)
    }

    /// Create a Context-specific class tag
    pub fn context_specific(constructed: bool, number: u32) -> Self {
        Self::new(BerTagClass::ContextSpecific, constructed, number)
    }

    /// Create a Private class tag
    pub fn private(constructed: bool, number: u32) -> Self {
        Self::new(BerTagClass::Private, constructed, number)
    }

    /// Get tag class
    pub fn class(&self) -> BerTagClass {
        self.class
    }

    /// Check if tag is constructed
    pub fn is_constructed(&self) -> bool {
        self.constructed
    }

    /// Get tag number
    pub fn number(&self) -> u32 {
        self.number
    }

    /// Encode tag to bytes
    ///
    /// # Encoding Strategy
    /// - If tag number <= 30: Use short form (1 byte)
    /// - If tag number > 30: Use extended form (multiple bytes)
    ///
    /// # Why Extended Form?
    /// Extended form allows encoding tag numbers > 30, which is necessary
    /// for some ASN.1 specifications that use large tag numbers.
    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::new();

        // Build first byte: class (2 bits) + constructed (1 bit) + tag (5 bits)
        let class_bits = self.class.to_bits();
        let constructed_bit = if self.constructed { 0x20 } else { 0x00 };

        if self.number <= 30 {
            // Short form: single byte
            let tag_byte = class_bits | constructed_bit | (self.number as u8 & 0x1F);
            result.push(tag_byte);
        } else {
            // Extended form: first byte has all tag bits set to 1
            let first_byte = class_bits | constructed_bit | 0x1F;
            result.push(first_byte);

            // Encode tag number in continuation bytes
            let mut remaining = self.number;
            let mut bytes = Vec::new();
            while remaining > 0 {
                bytes.push((remaining & 0x7F) as u8);
                remaining >>= 7;
            }

            // Reverse bytes and set continuation bit (bit 7) on all but last
            for (i, &byte) in bytes.iter().rev().enumerate() {
                if i < bytes.len() - 1 {
                    result.push(byte | 0x80); // Set continuation bit
                } else {
                    result.push(byte); // Last byte, no continuation bit
                }
            }
        }

        result
    }

    /// Decode tag from bytes
    ///
    /// # Returns
    /// Returns `Ok((BerTag, bytes_consumed))` if successful, `Err` otherwise
    ///
    /// # Error Handling
    /// Returns error if:
    /// - Buffer is too short
    /// - Invalid tag encoding
    pub fn decode(data: &[u8]) -> DlmsResult<(Self, usize)> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData("Empty buffer for tag decoding".to_string()));
        }

        let first_byte = data[0];
        let class = BerTagClass::from_bits(first_byte)?;
        let constructed = (first_byte & 0x20) != 0;
        let tag_bits = first_byte & 0x1F;

        if tag_bits < 31 {
            // Short form: tag number is in the 5 bits
            Ok((
                Self::new(class, constructed, tag_bits as u32),
                1, // Consumed 1 byte
            ))
        } else {
            // Extended form: read continuation bytes
            let mut tag_number = 0u32;
            let mut pos = 1;
            let mut has_more = true;

            while has_more && pos < data.len() {
                let byte = data[pos];
                has_more = (byte & 0x80) != 0;
                tag_number = (tag_number << 7) | ((byte & 0x7F) as u32);
                pos += 1;

                // Prevent infinite loop (max 5 bytes for u32)
                if pos > 5 {
                    return Err(DlmsError::InvalidData(
                        "Tag number too large or invalid encoding".to_string(),
                    ));
                }
            }

            if has_more {
                return Err(DlmsError::InvalidData(
                    "Incomplete extended tag encoding".to_string(),
                ));
            }

            Ok((Self::new(class, constructed, tag_number), pos))
        }
    }
}

/// BER Length encoding
///
/// BER length can be encoded in two forms:
/// - **Short form**: For lengths 0-127 (1 byte)
/// - **Long form**: For lengths > 127 (2-127 bytes)
///
/// # Encoding Format
///
/// Short form:
/// ```
/// Byte: 0 L L L L L L L
/// ```
/// Where L = length value (0-127)
///
/// Long form:
/// ```
/// First byte:  1 N N N N N N N  (N = number of length bytes)
/// Following bytes: L L L L L L L L  (big-endian length value)
/// ```
///
/// # Why Two Forms?
/// Short form is more efficient for common small lengths (most ASN.1 values
/// are < 128 bytes), while long form supports arbitrarily large lengths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BerLength {
    /// Short form: length 0-127
    Short(u8),
    /// Long form: length > 127, encoded with length-of-length
    Long(usize),
}

impl BerLength {
    /// Create a new BER length
    ///
    /// Automatically chooses short or long form based on the length value.
    pub fn new(length: usize) -> Self {
        if length < 128 {
            BerLength::Short(length as u8)
        } else {
            BerLength::Long(length)
        }
    }

    /// Get the length value
    pub fn value(&self) -> usize {
        match self {
            BerLength::Short(l) => *l as usize,
            BerLength::Long(l) => *l,
        }
    }

    /// Encode length to bytes
    ///
    /// # Returns
    /// Encoded length bytes (1 byte for short form, 2-127 bytes for long form)
    ///
    /// # Why This Encoding?
    /// The encoding format is specified by ITU-T X.690 (ASN.1 encoding rules).
    /// This ensures compatibility with all ASN.1 implementations.
    pub fn encode(&self) -> Vec<u8> {
        match self {
            BerLength::Short(length) => {
                // Short form: single byte with bit 7 = 0
                vec![*length]
            }
            BerLength::Long(length) => {
                // Long form: first byte indicates number of length bytes
                // Calculate number of bytes needed
                let mut num_bytes = 0;
                let mut temp = *length;
                while temp > 0 {
                    num_bytes += 1;
                    temp >>= 8;
                }
                // Minimum 1 byte for length
                if num_bytes == 0 {
                    num_bytes = 1;
                }

                // First byte: bit 7 = 1, bits 6-0 = number of length bytes
                let mut result = vec![0x80 | (num_bytes as u8)];

                // Encode length in big-endian format
                for i in (0..num_bytes).rev() {
                    result.push(((*length >> (i * 8)) & 0xFF) as u8);
                }

                result
            }
        }
    }

    /// Decode length from bytes
    ///
    /// # Returns
    /// Returns `Ok((BerLength, bytes_consumed))` if successful, `Err` otherwise
    ///
    /// # Error Handling
    /// Returns error if:
    /// - Buffer is too short
    /// - Invalid length encoding
    /// - Length value is too large (would cause overflow)
    pub fn decode(data: &[u8]) -> DlmsResult<(Self, usize)> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData(
                "Empty buffer for length decoding".to_string(),
            ));
        }

        let first_byte = data[0];

        if (first_byte & 0x80) == 0 {
            // Short form: length is in bits 6-0
            Ok((BerLength::Short(first_byte & 0x7F), 1))
        } else {
            // Long form: bits 6-0 indicate number of length bytes
            let num_bytes = (first_byte & 0x7F) as usize;

            if num_bytes == 0 {
                // Indefinite length (not supported)
                return Err(DlmsError::InvalidData(
                    "Indefinite length encoding not supported".to_string(),
                ));
            }

            if num_bytes > 4 {
                // Limit to 4 bytes (32-bit length, max ~4GB)
                return Err(DlmsError::InvalidData(format!(
                    "Length encoding too large: {} bytes (max 4)",
                    num_bytes
                )));
            }

            if data.len() < 1 + num_bytes {
                return Err(DlmsError::InvalidData(format!(
                    "Buffer too short for long form length: need {} bytes, got {}",
                    1 + num_bytes,
                    data.len()
                )));
            }

            // Decode length value (big-endian)
            let mut length = 0usize;
            for i in 0..num_bytes {
                length = (length << 8) | (data[1 + i] as usize);
            }

            Ok((BerLength::Long(length), 1 + num_bytes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ber_tag_short_form() {
        let tag = BerTag::universal(false, 2); // INTEGER tag
        let encoded = tag.encode();
        assert_eq!(encoded.len(), 1);
        assert_eq!(encoded[0], 0x02); // Universal, Primitive, tag 2
    }

    #[test]
    fn test_ber_tag_constructed() {
        let tag = BerTag::application(true, 0); // AARQ tag
        let encoded = tag.encode();
        assert_eq!(encoded.len(), 1);
        assert_eq!(encoded[0], 0x60); // Application, Constructed, tag 0
    }

    #[test]
    fn test_ber_tag_decode() {
        let data = [0x02]; // Universal, Primitive, tag 2
        let (tag, consumed) = BerTag::decode(&data).unwrap();
        assert_eq!(consumed, 1);
        assert_eq!(tag.class(), BerTagClass::Universal);
        assert_eq!(tag.is_constructed(), false);
        assert_eq!(tag.number(), 2);
    }

    #[test]
    fn test_ber_length_short() {
        let length = BerLength::new(100);
        let encoded = length.encode();
        assert_eq!(encoded.len(), 1);
        assert_eq!(encoded[0], 100);
    }

    #[test]
    fn test_ber_length_long() {
        let length = BerLength::new(1000);
        let encoded = length.encode();
        assert!(encoded.len() > 1);
        assert_eq!((encoded[0] & 0x80) != 0, true); // Long form flag
    }

    #[test]
    fn test_ber_length_decode() {
        let data = [100]; // Short form, length 100
        let (length, consumed) = BerLength::decode(&data).unwrap();
        assert_eq!(consumed, 1);
        assert_eq!(length.value(), 100);
    }
}
