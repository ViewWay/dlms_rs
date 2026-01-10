//! A-XDR encoder for DLMS/COSEM

use crate::error::{DlmsError, DlmsResult};
use crate::axdr::types::{AxdrTag, LengthEncoding};
use dlms_core::datatypes::*;
use std::io::Write;

/// A-XDR encoder for encoding DLMS/COSEM data types
pub struct AxdrEncoder {
    buffer: Vec<u8>,
}

impl AxdrEncoder {
    /// Create a new encoder
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    /// Create a new encoder with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
        }
    }

    /// Encode a DataObject
    pub fn encode_data_object(&mut self, obj: &DataObject) -> DlmsResult<()> {
        use DataObject::*;
        use AxdrTag::*;

        match obj {
            Null => self.encode_tag(Null)?,
            Boolean(b) => {
                self.encode_tag(Boolean)?;
                self.encode_bool(*b)?;
            }
            Integer8(i) => {
                self.encode_tag(Integer8)?;
                self.encode_i8(*i)?;
            }
            Integer16(i) => {
                self.encode_tag(Integer16)?;
                self.encode_i16(*i)?;
            }
            Integer32(i) => {
                self.encode_tag(Integer32)?;
                self.encode_i32(*i)?;
            }
            Integer64(i) => {
                self.encode_tag(Integer64)?;
                self.encode_i64(*i)?;
            }
            Unsigned8(u) => {
                self.encode_tag(Unsigned8)?;
                self.encode_u8(*u)?;
            }
            Unsigned16(u) => {
                self.encode_tag(Unsigned16)?;
                self.encode_u16(*u)?;
            }
            Unsigned32(u) => {
                self.encode_tag(Unsigned32)?;
                self.encode_u32(*u)?;
            }
            Unsigned64(u) => {
                self.encode_tag(Unsigned64)?;
                self.encode_u64(*u)?;
            }
            Float32(f) => {
                self.encode_tag(Float32)?;
                self.encode_f32(*f)?;
            }
            Float64(f) => {
                self.encode_tag(Float64)?;
                self.encode_f64(*f)?;
            }
            Enumerate(e) => {
                self.encode_tag(Enumerate)?;
                self.encode_u8(*e)?;
            }
            Bcd(b) => {
                self.encode_tag(Bcd)?;
                self.encode_u8(*b)?;
            }
            OctetString(s) => {
                self.encode_tag(OctetString)?;
                self.encode_octet_string(s)?;
            }
            VisibleString(s) => {
                self.encode_tag(VisibleString)?;
                self.encode_octet_string(s)?;
            }
            Utf8String(s) => {
                self.encode_tag(Utf8String)?;
                self.encode_octet_string(s)?;
            }
            BitString(bs) => {
                self.encode_tag(BitString)?;
                self.encode_bit_string(bs)?;
            }
            Array(arr) => {
                self.encode_tag(Array)?;
                self.encode_array(arr)?;
            }
            Structure(s) => {
                self.encode_tag(Structure)?;
                self.encode_structure(s)?;
            }
            CompactArray(ca) => {
                self.encode_tag(CompactArray)?;
                // TODO: Implement compact array encoding
                return Err(DlmsError::InvalidData(
                    "CompactArray encoding not yet implemented".to_string(),
                ));
            }
            Date(d) => {
                self.encode_tag(Date)?;
                self.encode_bytes(&d.encode())?;
            }
            Time(t) => {
                self.encode_tag(Time)?;
                self.encode_bytes(&t.encode())?;
            }
            DateTime(dt) => {
                self.encode_tag(DateTime)?;
                self.encode_bytes(&dt.encode())?;
            }
        }
        Ok(())
    }

    /// Encode a tag
    pub fn encode_tag(&mut self, tag: AxdrTag) -> DlmsResult<()> {
        self.buffer.write_all(&[tag.to_u8()]).map_err(|e| {
            DlmsError::Asn1Encoding(format!("Failed to write tag: {}", e))
        })?;
        Ok(())
    }

    /// Encode a boolean
    pub fn encode_bool(&mut self, value: bool) -> DlmsResult<()> {
        self.buffer.write_all(&[if value { 0xFF } else { 0x00 }]).map_err(|e| {
            DlmsError::Asn1Encoding(format!("Failed to write boolean: {}", e))
        })?;
        Ok(())
    }

    /// Encode an i8
    pub fn encode_i8(&mut self, value: i8) -> DlmsResult<()> {
        self.buffer.write_all(&[value as u8]).map_err(|e| {
            DlmsError::Asn1Encoding(format!("Failed to write i8: {}", e))
        })?;
        Ok(())
    }

    /// Encode an i16 (big-endian)
    pub fn encode_i16(&mut self, value: i16) -> DlmsResult<()> {
        self.buffer.write_all(&value.to_be_bytes()).map_err(|e| {
            DlmsError::Asn1Encoding(format!("Failed to write i16: {}", e))
        })?;
        Ok(())
    }

    /// Encode an i32 (big-endian)
    pub fn encode_i32(&mut self, value: i32) -> DlmsResult<()> {
        self.buffer.write_all(&value.to_be_bytes()).map_err(|e| {
            DlmsError::Asn1Encoding(format!("Failed to write i32: {}", e))
        })?;
        Ok(())
    }

    /// Encode an i64 (big-endian)
    pub fn encode_i64(&mut self, value: i64) -> DlmsResult<()> {
        self.buffer.write_all(&value.to_be_bytes()).map_err(|e| {
            DlmsError::Asn1Encoding(format!("Failed to write i64: {}", e))
        })?;
        Ok(())
    }

    /// Encode a u8
    pub fn encode_u8(&mut self, value: u8) -> DlmsResult<()> {
        self.buffer.write_all(&[value]).map_err(|e| {
            DlmsError::Asn1Encoding(format!("Failed to write u8: {}", e))
        })?;
        Ok(())
    }

    /// Encode a u16 (big-endian)
    pub fn encode_u16(&mut self, value: u16) -> DlmsResult<()> {
        self.buffer.write_all(&value.to_be_bytes()).map_err(|e| {
            DlmsError::Asn1Encoding(format!("Failed to write u16: {}", e))
        })?;
        Ok(())
    }

    /// Encode a u32 (big-endian)
    pub fn encode_u32(&mut self, value: u32) -> DlmsResult<()> {
        self.buffer.write_all(&value.to_be_bytes()).map_err(|e| {
            DlmsError::Asn1Encoding(format!("Failed to write u32: {}", e))
        })?;
        Ok(())
    }

    /// Encode a u64 (big-endian)
    pub fn encode_u64(&mut self, value: u64) -> DlmsResult<()> {
        self.buffer.write_all(&value.to_be_bytes()).map_err(|e| {
            DlmsError::Asn1Encoding(format!("Failed to write u64: {}", e))
        })?;
        Ok(())
    }

    /// Encode an f32 (IEEE 754 big-endian)
    pub fn encode_f32(&mut self, value: f32) -> DlmsResult<()> {
        self.buffer.write_all(&value.to_bits().to_be_bytes()).map_err(|e| {
            DlmsError::Asn1Encoding(format!("Failed to write f32: {}", e))
        })?;
        Ok(())
    }

    /// Encode an f64 (IEEE 754 big-endian)
    pub fn encode_f64(&mut self, value: f64) -> DlmsResult<()> {
        self.buffer.write_all(&value.to_bits().to_be_bytes()).map_err(|e| {
            DlmsError::Asn1Encoding(format!("Failed to write f64: {}", e))
        })?;
        Ok(())
    }

    /// Encode an octet string
    pub fn encode_octet_string(&mut self, value: &[u8]) -> DlmsResult<()> {
        let len_enc = if value.len() < 128 {
            LengthEncoding::Short(value.len() as u8)
        } else {
            LengthEncoding::Long(value.len())
        };
        self.buffer.extend_from_slice(&len_enc.encode());
        self.buffer.extend_from_slice(value);
        Ok(())
    }

    /// Encode a bit string
    pub fn encode_bit_string(&mut self, bit_string: &BitString) -> DlmsResult<()> {
        let bytes = bit_string.as_bytes();
        let num_bits = bit_string.num_bits();
        
        // Encode length (number of bits)
        let len_enc = if num_bits < 128 {
            LengthEncoding::Short(num_bits as u8)
        } else {
            LengthEncoding::Long(num_bits)
        };
        self.buffer.extend_from_slice(&len_enc.encode());
        
        // Encode bytes
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }

    /// Encode an array
    pub fn encode_array(&mut self, array: &[DataObject]) -> DlmsResult<()> {
        // Encode length
        let len_enc = if array.len() < 128 {
            LengthEncoding::Short(array.len() as u8)
        } else {
            LengthEncoding::Long(array.len())
        };
        self.buffer.extend_from_slice(&len_enc.encode());
        
        // Encode each element
        for obj in array {
            self.encode_data_object(obj)?;
        }
        Ok(())
    }

    /// Encode a structure
    pub fn encode_structure(&mut self, structure: &[DataObject]) -> DlmsResult<()> {
        // Encode length
        let len_enc = if structure.len() < 128 {
            LengthEncoding::Short(structure.len() as u8)
        } else {
            LengthEncoding::Long(structure.len())
        };
        self.buffer.extend_from_slice(&len_enc.encode());
        
        // Encode each element
        for obj in structure {
            self.encode_data_object(obj)?;
        }
        Ok(())
    }

    /// Encode raw bytes
    pub fn encode_bytes(&mut self, bytes: &[u8]) -> DlmsResult<()> {
        self.buffer.extend_from_slice(bytes);
        Ok(())
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

impl Default for AxdrEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_null() {
        let mut encoder = AxdrEncoder::new();
        encoder.encode_tag(AxdrTag::Null).unwrap();
        assert_eq!(encoder.as_bytes(), &[0x00]);
    }

    #[test]
    fn test_encode_boolean() {
        let mut encoder = AxdrEncoder::new();
        encoder.encode_tag(AxdrTag::Boolean).unwrap();
        encoder.encode_bool(true).unwrap();
        assert_eq!(encoder.as_bytes(), &[0x03, 0xFF]);
    }

    #[test]
    fn test_encode_integer32() {
        let mut encoder = AxdrEncoder::new();
        encoder.encode_tag(AxdrTag::Integer32).unwrap();
        encoder.encode_i32(0x12345678).unwrap();
        assert_eq!(encoder.as_bytes(), &[0x05, 0x12, 0x34, 0x56, 0x78]);
    }
}
