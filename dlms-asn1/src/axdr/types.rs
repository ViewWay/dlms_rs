//! A-XDR types for DLMS/COSEM

use crate::error::{DlmsError, DlmsResult};

/// A-XDR tag values for different data types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxdrTag {
    Null = 0x00,
    Boolean = 0x03,
    Integer8 = 0x0F,
    Integer16 = 0x10,
    Integer32 = 0x05,
    Integer64 = 0x14,
    Unsigned8 = 0x11,
    Unsigned16 = 0x12,
    Unsigned32 = 0x06,
    Unsigned64 = 0x15,
    Float32 = 0x17,
    Float64 = 0x18,
    OctetString = 0x09,
    VisibleString = 0x0A,
    Utf8String = 0x0C,
    Bcd = 0x0D,
    BitString = 0x04,
    Date = 0x1A,
    Time = 0x1B,
    DateTime = 0x19,
    Array = 0x01,
    Structure = 0x02,
    CompactArray = 0x13,
    Enumerate = 0x16,
    DontCare = 0xFF,
}

impl AxdrTag {
    /// Get tag from u8 value
    pub fn from_u8(value: u8) -> DlmsResult<Self> {
        match value {
            0x00 => Ok(AxdrTag::Null),
            0x03 => Ok(AxdrTag::Boolean),
            0x0F => Ok(AxdrTag::Integer8),
            0x10 => Ok(AxdrTag::Integer16),
            0x05 => Ok(AxdrTag::Integer32),
            0x14 => Ok(AxdrTag::Integer64),
            0x11 => Ok(AxdrTag::Unsigned8),
            0x12 => Ok(AxdrTag::Unsigned16),
            0x06 => Ok(AxdrTag::Unsigned32),
            0x15 => Ok(AxdrTag::Unsigned64),
            0x17 => Ok(AxdrTag::Float32),
            0x18 => Ok(AxdrTag::Float64),
            0x09 => Ok(AxdrTag::OctetString),
            0x0A => Ok(AxdrTag::VisibleString),
            0x0C => Ok(AxdrTag::Utf8String),
            0x0D => Ok(AxdrTag::Bcd),
            0x04 => Ok(AxdrTag::BitString),
            0x1A => Ok(AxdrTag::Date),
            0x1B => Ok(AxdrTag::Time),
            0x19 => Ok(AxdrTag::DateTime),
            0x01 => Ok(AxdrTag::Array),
            0x02 => Ok(AxdrTag::Structure),
            0x13 => Ok(AxdrTag::CompactArray),
            0x16 => Ok(AxdrTag::Enumerate),
            0xFF => Ok(AxdrTag::DontCare),
            _ => Err(DlmsError::InvalidData(format!(
                "Unknown A-XDR tag: 0x{:02X}",
                value
            ))),
        }
    }

    /// Convert tag to u8 value
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Length encoding for variable-length types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthEncoding {
    /// Short form: length < 128, encoded in 1 byte
    Short(u8),
    /// Long form: length >= 128, encoded with length-of-length byte + length bytes
    Long(usize),
}

impl LengthEncoding {
    /// Encode length to bytes
    pub fn encode(&self) -> Vec<u8> {
        match self {
            LengthEncoding::Short(len) => vec![*len],
            LengthEncoding::Long(len) => {
                let mut result = Vec::new();
                let mut len = *len;
                let mut bytes = Vec::new();
                while len > 0 {
                    bytes.push((len & 0xFF) as u8);
                    len >>= 8;
                }
                bytes.reverse();
                result.push(0x80 | bytes.len() as u8);
                result.extend_from_slice(&bytes);
                result
            }
        }
    }

    /// Decode length from bytes
    pub fn decode(bytes: &[u8]) -> DlmsResult<(Self, usize)> {
        if bytes.is_empty() {
            return Err(DlmsError::InvalidData("Not enough bytes for length".to_string()));
        }

        let first_byte = bytes[0];
        if (first_byte & 0x80) == 0 {
            // Short form
            Ok((LengthEncoding::Short(first_byte), 1))
        } else {
            // Long form
            let length_of_length = (first_byte & 0x7F) as usize;
            if length_of_length == 0 || length_of_length > 4 {
                return Err(DlmsError::InvalidData(format!(
                    "Invalid length-of-length: {}",
                    length_of_length
                )));
            }
            if bytes.len() < 1 + length_of_length {
                return Err(DlmsError::InvalidData("Not enough bytes for long length".to_string()));
            }

            let mut len = 0usize;
            for &byte in &bytes[1..1+length_of_length] {
                len = (len << 8) | (byte as usize);
            }
            Ok((LengthEncoding::Long(len), 1 + length_of_length))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_encoding_short() {
        let enc = LengthEncoding::Short(10);
        let bytes = enc.encode();
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 10);
    }

    #[test]
    fn test_length_encoding_long() {
        let enc = LengthEncoding::Long(256);
        let bytes = enc.encode();
        assert!(bytes.len() > 1);
        let (decoded, consumed) = LengthEncoding::decode(&bytes).unwrap();
        match decoded {
            LengthEncoding::Long(len) => assert_eq!(len, 256),
            _ => panic!("Expected Long form"),
        }
        assert_eq!(consumed, bytes.len());
    }
}
