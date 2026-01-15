//! A-XDR decoder for DLMS/COSEM

use crate::error::{DlmsError, DlmsResult};
use crate::axdr::types::{AxdrTag, LengthEncoding};
use dlms_core::datatypes::*;
use std::io::Read;

/// A-XDR decoder for decoding DLMS/COSEM data types
pub struct AxdrDecoder<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> AxdrDecoder<'a> {
    /// Create a new decoder
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    /// Decode a DataObject
    pub fn decode_data_object(&mut self) -> DlmsResult<DataObject> {
        let tag = self.decode_tag()?;
        
        match tag {
            AxdrTag::Null => Ok(DataObject::new_null()),
            AxdrTag::Boolean => {
                let value = self.decode_bool()?;
                Ok(DataObject::new_bool(value))
            }
            AxdrTag::Integer8 => {
                let value = self.decode_i8()?;
                Ok(DataObject::new_integer8(value))
            }
            AxdrTag::Integer16 => {
                let value = self.decode_i16()?;
                Ok(DataObject::new_integer16(value))
            }
            AxdrTag::Integer32 => {
                let value = self.decode_i32()?;
                Ok(DataObject::new_integer32(value))
            }
            AxdrTag::Integer64 => {
                let value = self.decode_i64()?;
                Ok(DataObject::new_integer64(value))
            }
            AxdrTag::Unsigned8 => {
                let value = self.decode_u8()?;
                Ok(DataObject::new_unsigned8(value))
            }
            AxdrTag::Unsigned16 => {
                let value = self.decode_u16()?;
                Ok(DataObject::new_unsigned16(value))
            }
            AxdrTag::Unsigned32 => {
                let value = self.decode_u32()?;
                Ok(DataObject::new_unsigned32(value))
            }
            AxdrTag::Unsigned64 => {
                let value = self.decode_u64()?;
                Ok(DataObject::new_unsigned64(value))
            }
            AxdrTag::Float32 => {
                let value = self.decode_f32()?;
                Ok(DataObject::new_float32(value))
            }
            AxdrTag::Float64 => {
                let value = self.decode_f64()?;
                Ok(DataObject::new_float64(value))
            }
            AxdrTag::Enumerate => {
                let value = self.decode_u8()?;
                Ok(DataObject::new_enumerate(value))
            }
            AxdrTag::Bcd => {
                let value = self.decode_u8()?;
                Ok(DataObject::new_bcd(value))
            }
            AxdrTag::OctetString => {
                let value = self.decode_octet_string()?;
                Ok(DataObject::new_octet_string(value))
            }
            AxdrTag::VisibleString => {
                let value = self.decode_octet_string()?;
                Ok(DataObject::new_visible_string(value))
            }
            AxdrTag::Utf8String => {
                let value = self.decode_octet_string()?;
                Ok(DataObject::new_utf8_string(value))
            }
            AxdrTag::BitString => {
                let value = self.decode_bit_string()?;
                Ok(DataObject::new_bit_string(value))
            }
            AxdrTag::Array => {
                let value = self.decode_array()?;
                Ok(DataObject::new_array(value)?)
            }
            AxdrTag::Structure => {
                let value = self.decode_structure()?;
                Ok(DataObject::new_structure(value))
            }
            AxdrTag::CompactArray => {
                return Err(DlmsError::InvalidData(
                    "CompactArray decoding not yet implemented".to_string(),
                ));
            }
            AxdrTag::Date => {
                let bytes = self.decode_fixed_bytes(CosemDate::LENGTH)?;
                let date = CosemDate::decode(&bytes)?;
                Ok(DataObject::new_date(date))
            }
            AxdrTag::Time => {
                let bytes = self.decode_fixed_bytes(CosemTime::LENGTH)?;
                let time = CosemTime::decode(&bytes)?;
                Ok(DataObject::new_time(time))
            }
            AxdrTag::DateTime => {
                let bytes = self.decode_fixed_bytes(CosemDateTime::LENGTH)?;
                let date_time = CosemDateTime::decode(&bytes)?;
                Ok(DataObject::new_date_time(date_time))
            }
            AxdrTag::DontCare => {
                Ok(DataObject::new_null())
            }
        }
    }

    /// Decode a tag
    pub fn decode_tag(&mut self) -> DlmsResult<AxdrTag> {
        let byte = self.read_byte()?;
        AxdrTag::from_u8(byte)
    }

    /// Decode a boolean
    pub fn decode_bool(&mut self) -> DlmsResult<bool> {
        let byte = self.read_byte()?;
        Ok(byte != 0x00)
    }

    /// Decode an i8
    pub fn decode_i8(&mut self) -> DlmsResult<i8> {
        let byte = self.read_byte()?;
        Ok(byte as i8)
    }

    /// Decode an i16 (big-endian)
    pub fn decode_i16(&mut self) -> DlmsResult<i16> {
        let bytes = self.read_bytes(2)?;
        Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
    }

    /// Decode an i32 (big-endian)
    pub fn decode_i32(&mut self) -> DlmsResult<i32> {
        let bytes = self.read_bytes(4)?;
        Ok(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Decode an i64 (big-endian)
    pub fn decode_i64(&mut self) -> DlmsResult<i64> {
        let bytes = self.read_bytes(8)?;
        Ok(i64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Decode a u8
    pub fn decode_u8(&mut self) -> DlmsResult<u8> {
        self.read_byte()
    }

    /// Decode a u16 (big-endian)
    pub fn decode_u16(&mut self) -> DlmsResult<u16> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    /// Decode a u32 (big-endian)
    pub fn decode_u32(&mut self) -> DlmsResult<u32> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Decode a u64 (big-endian)
    pub fn decode_u64(&mut self) -> DlmsResult<u64> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Decode an f32 (IEEE 754 big-endian)
    pub fn decode_f32(&mut self) -> DlmsResult<f32> {
        let bytes = self.read_bytes(4)?;
        let bits = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Ok(f32::from_bits(bits))
    }

    /// Decode an f64 (IEEE 754 big-endian)
    pub fn decode_f64(&mut self) -> DlmsResult<f64> {
        let bytes = self.read_bytes(8)?;
        let bits = u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        Ok(f64::from_bits(bits))
    }

    /// Decode an octet string
    pub fn decode_octet_string(&mut self) -> DlmsResult<Vec<u8>> {
        let (len_enc, consumed) = LengthEncoding::decode(&self.buffer[self.position..])?;
        self.position += consumed;
        
        let len = match len_enc {
            LengthEncoding::Short(l) => l as usize,
            LengthEncoding::Long(l) => l,
        };
        
        self.decode_fixed_bytes(len)
    }

    /// Decode a bit string
    pub fn decode_bit_string(&mut self) -> DlmsResult<BitString> {
        let (len_enc, consumed) = LengthEncoding::decode(&self.buffer[self.position..])?;
        self.position += consumed;
        
        let num_bits = match len_enc {
            LengthEncoding::Short(l) => l as usize,
            LengthEncoding::Long(l) => l,
        };
        
        let num_bytes = (num_bits + 7) / 8;
        let bytes = self.decode_fixed_bytes(num_bytes)?;
        
        BitString::new(bytes, num_bits)
    }

    /// Decode an array
    pub fn decode_array(&mut self) -> DlmsResult<Vec<DataObject>> {
        let (len_enc, consumed) = LengthEncoding::decode(&self.buffer[self.position..])?;
        self.position += consumed;
        
        let len = match len_enc {
            LengthEncoding::Short(l) => l as usize,
            LengthEncoding::Long(l) => l,
        };
        
        let mut array = Vec::with_capacity(len);
        for _ in 0..len {
            array.push(self.decode_data_object()?);
        }
        Ok(array)
    }

    /// Decode a structure
    pub fn decode_structure(&mut self) -> DlmsResult<Vec<DataObject>> {
        let (len_enc, consumed) = LengthEncoding::decode(&self.buffer[self.position..])?;
        self.position += consumed;
        
        let len = match len_enc {
            LengthEncoding::Short(l) => l as usize,
            LengthEncoding::Long(l) => l,
        };
        
        let mut structure = Vec::with_capacity(len);
        for _ in 0..len {
            structure.push(self.decode_data_object()?);
        }
        Ok(structure)
    }

    /// Decode fixed-length bytes
    pub fn decode_fixed_bytes(&mut self, len: usize) -> DlmsResult<Vec<u8>> {
        if self.position + len > self.buffer.len() {
            return Err(DlmsError::InvalidData(format!(
                "Not enough bytes: need {}, have {}",
                len,
                self.buffer.len() - self.position
            )));
        }
        
        let result = self.buffer[self.position..self.position + len].to_vec();
        self.position += len;
        Ok(result)
    }

    /// Read a single byte
    fn read_byte(&mut self) -> DlmsResult<u8> {
        if self.position >= self.buffer.len() {
            return Err(DlmsError::InvalidData("Not enough bytes".to_string()));
        }
        let byte = self.buffer[self.position];
        self.position += 1;
        Ok(byte)
    }

    /// Read multiple bytes
    fn read_bytes(&mut self, len: usize) -> DlmsResult<Vec<u8>> {
        self.decode_fixed_bytes(len)
    }

    /// Get current position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get remaining bytes
    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.position
    }

    /// Decode a DataObject (alias for decode_data_object)
    ///
    /// This is a convenience method that delegates to decode_data_object.
    pub fn decode_data(&mut self) -> DlmsResult<DataObject> {
        self.decode_data_object()
    }

    /// Decode an Integer8 (signed 8-bit)
    ///
    /// This is a convenience method for decoding i8 values.
    pub fn decode_integer8(&mut self) -> DlmsResult<i8> {
        self.decode_i8()
    }

    /// Decode an Unsigned8 (unsigned 8-bit)
    ///
    /// This is a convenience method for decoding u8 values.
    pub fn decode_unsigned8(&mut self) -> DlmsResult<u8> {
        self.decode_u8()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_null() {
        let bytes = [0x00];
        let mut decoder = AxdrDecoder::new(&bytes);
        let tag = decoder.decode_tag().unwrap();
        assert_eq!(tag, AxdrTag::Null);
    }

    #[test]
    fn test_decode_boolean() {
        let bytes = [0x03, 0xFF];
        let mut decoder = AxdrDecoder::new(&bytes);
        let obj = decoder.decode_data_object().unwrap();
        assert_eq!(obj.as_bool().unwrap(), true);
    }

    #[test]
    fn test_decode_integer32() {
        let bytes = [0x05, 0x12, 0x34, 0x56, 0x78];
        let mut decoder = AxdrDecoder::new(&bytes);
        let obj = decoder.decode_data_object().unwrap();
        assert_eq!(obj.as_integer32().unwrap(), 0x12345678);
    }
}
