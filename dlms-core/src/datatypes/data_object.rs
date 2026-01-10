//! Data object types for DLMS/COSEM protocol

use crate::error::{DlmsError, DlmsResult};
use crate::datatypes::bit_string::BitString;
use crate::datatypes::cosem_date::{CosemDate, CosemDateFormat};
use crate::datatypes::cosem_time::CosemTime;
use crate::datatypes::cosem_date_time::CosemDateTime;
use crate::datatypes::compact_array::CompactArray;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Container class holding data to send to the smart meter or received by the smart meter
///
/// Stores various data types including numbers, lists, byte arrays, BitString, or date/time formats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataObject {
    /// Null data
    Null,
    /// Boolean value
    Boolean(bool),
    /// Integer 8-bit
    Integer8(i8),
    /// Integer 16-bit
    Integer16(i16),
    /// Integer 32-bit
    Integer32(i32),
    /// Integer 64-bit
    Integer64(i64),
    /// Unsigned integer 8-bit
    Unsigned8(u8),
    /// Unsigned integer 16-bit
    Unsigned16(u16),
    /// Unsigned integer 32-bit
    Unsigned32(u32),
    /// Unsigned integer 64-bit
    Unsigned64(u64),
    /// Float 32-bit
    Float32(f32),
    /// Float 64-bit
    Float64(f64),
    /// Enumeration (8-bit)
    Enumerate(u8),
    /// BCD (Binary Coded Decimal)
    Bcd(u8),
    /// Octet string
    OctetString(Vec<u8>),
    /// Visible string
    VisibleString(Vec<u8>),
    /// UTF-8 string
    Utf8String(Vec<u8>),
    /// Bit string
    BitString(BitString),
    /// Array of DataObjects
    Array(Vec<DataObject>),
    /// Structure (ordered list of DataObjects)
    Structure(Vec<DataObject>),
    /// Compact array
    CompactArray(CompactArray),
    /// Date
    Date(CosemDate),
    /// Time
    Time(CosemTime),
    /// Date and time
    DateTime(CosemDateTime),
}

/// Type enumeration for DataObject
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataObjectType {
    /// Null
    NullData,
    /// Array
    Array,
    /// Structure
    Structure,
    /// Boolean
    Boolean,
    /// Bit string
    BitString,
    /// Integer 32-bit
    DoubleLong,
    /// Unsigned integer 32-bit
    DoubleLongUnsigned,
    /// Octet string
    OctetString,
    /// UTF-8 string
    Utf8String,
    /// Visible string
    VisibleString,
    /// BCD
    Bcd,
    /// Integer 8-bit
    Integer,
    /// Integer 16-bit
    LongInteger,
    /// Unsigned integer 8-bit
    Unsigned,
    /// Unsigned integer 16-bit
    LongUnsigned,
    /// Compact array
    CompactArray,
    /// Integer 64-bit
    Long64,
    /// Unsigned integer 64-bit
    Long64Unsigned,
    /// Enumeration
    Enumerate,
    /// Float 32-bit
    Float32,
    /// Float 64-bit
    Float64,
    /// Date time
    DateTime,
    /// Date
    Date,
    /// Time
    Time,
    /// Don't care
    DontCare,
}

impl DataObjectType {
    /// Check if this type is a number type
    pub fn is_number(&self) -> bool {
        matches!(
            self,
            DataObjectType::DoubleLong
                | DataObjectType::DoubleLongUnsigned
                | DataObjectType::Integer
                | DataObjectType::LongInteger
                | DataObjectType::Unsigned
                | DataObjectType::LongUnsigned
                | DataObjectType::Long64
                | DataObjectType::Long64Unsigned
                | DataObjectType::Enumerate
                | DataObjectType::Bcd
                | DataObjectType::Float32
                | DataObjectType::Float64
        )
    }
}

impl DataObject {
    /// Get the type of this DataObject
    pub fn get_type(&self) -> DataObjectType {
        match self {
            DataObject::Null => DataObjectType::NullData,
            DataObject::Boolean(_) => DataObjectType::Boolean,
            DataObject::Integer8(_) => DataObjectType::Integer,
            DataObject::Integer16(_) => DataObjectType::LongInteger,
            DataObject::Integer32(_) => DataObjectType::DoubleLong,
            DataObject::Integer64(_) => DataObjectType::Long64,
            DataObject::Unsigned8(_) => DataObjectType::Unsigned,
            DataObject::Unsigned16(_) => DataObjectType::LongUnsigned,
            DataObject::Unsigned32(_) => DataObjectType::DoubleLongUnsigned,
            DataObject::Unsigned64(_) => DataObjectType::Long64Unsigned,
            DataObject::Float32(_) => DataObjectType::Float32,
            DataObject::Float64(_) => DataObjectType::Float64,
            DataObject::Enumerate(_) => DataObjectType::Enumerate,
            DataObject::Bcd(_) => DataObjectType::Bcd,
            DataObject::OctetString(_) => DataObjectType::OctetString,
            DataObject::VisibleString(_) => DataObjectType::VisibleString,
            DataObject::Utf8String(_) => DataObjectType::Utf8String,
            DataObject::BitString(_) => DataObjectType::BitString,
            DataObject::Array(_) => DataObjectType::Array,
            DataObject::Structure(_) => DataObjectType::Structure,
            DataObject::CompactArray(_) => DataObjectType::CompactArray,
            DataObject::Date(_) => DataObjectType::Date,
            DataObject::Time(_) => DataObjectType::Time,
            DataObject::DateTime(_) => DataObjectType::DateTime,
        }
    }

    /// Constructs an empty datum (NULL_DATA)
    pub fn new_null() -> Self {
        DataObject::Null
    }

    /// Constructs a boolean data
    pub fn new_bool(bool_val: bool) -> Self {
        DataObject::Boolean(bool_val)
    }

    /// Constructs an integer 8-bit data
    pub fn new_integer8(int8: i8) -> Self {
        DataObject::Integer8(int8)
    }

    /// Constructs an integer 16-bit data
    pub fn new_integer16(int16: i16) -> Self {
        DataObject::Integer16(int16)
    }

    /// Constructs an integer 32-bit data
    pub fn new_integer32(int32: i32) -> Self {
        DataObject::Integer32(int32)
    }

    /// Constructs an integer 64-bit data
    pub fn new_integer64(int64: i64) -> Self {
        DataObject::Integer64(int64)
    }

    /// Constructs an unsigned integer 8-bit data
    pub fn new_unsigned8(u_int8: u8) -> Self {
        DataObject::Unsigned8(u_int8)
    }

    /// Constructs an unsigned integer 16-bit data
    pub fn new_unsigned16(u_int16: u16) -> Self {
        DataObject::Unsigned16(u_int16)
    }

    /// Constructs an unsigned integer 32-bit data
    pub fn new_unsigned32(u_int32: u32) -> Self {
        DataObject::Unsigned32(u_int32)
    }

    /// Constructs an unsigned integer 64-bit data
    pub fn new_unsigned64(u_int64: u64) -> Self {
        DataObject::Unsigned64(u_int64)
    }

    /// Constructs a float 32-bit data
    pub fn new_float32(float32: f32) -> Self {
        DataObject::Float32(float32)
    }

    /// Constructs a float 64-bit data
    pub fn new_float64(float64: f64) -> Self {
        DataObject::Float64(float64)
    }

    /// Constructs an enumeration data
    pub fn new_enumerate(enum_val: u8) -> Self {
        // enum_val is u8, so it's always in valid range [0, 255]
        // No validation needed as the type system enforces the constraint
        DataObject::Enumerate(enum_val)
    }

    /// Constructs a BCD data
    pub fn new_bcd(bcd: u8) -> Self {
        DataObject::Bcd(bcd)
    }

    /// Constructs an octet string data
    pub fn new_octet_string(string: Vec<u8>) -> Self {
        DataObject::OctetString(string)
    }

    /// Constructs a visible string data
    pub fn new_visible_string(string: Vec<u8>) -> Self {
        DataObject::VisibleString(string)
    }

    /// Constructs a UTF-8 string data
    pub fn new_utf8_string(string: Vec<u8>) -> Self {
        DataObject::Utf8String(string)
    }

    /// Constructs a bit string data
    pub fn new_bit_string(bit_string: BitString) -> Self {
        DataObject::BitString(bit_string)
    }

    /// Constructs an array data
    ///
    /// # Errors
    ///
    /// Returns an error if array elements have different types
    pub fn new_array(array: Vec<DataObject>) -> DlmsResult<Self> {
        if !array.is_empty() {
            let array_type = array[0].get_type();
            for (index, sub) in array.iter().enumerate() {
                if sub.get_type() != array_type {
                    return Err(DlmsError::InvalidData(format!(
                        "Array is of type {:?}, but element at {} is of type {:?}",
                        array_type,
                        index,
                        sub.get_type()
                    )));
                }
            }
        }
        Ok(DataObject::Array(array))
    }

    /// Constructs a structure data
    pub fn new_structure(structure: Vec<DataObject>) -> Self {
        DataObject::Structure(structure)
    }

    /// Constructs a compact array data
    pub fn new_compact_array(compact_array: CompactArray) -> Self {
        DataObject::CompactArray(compact_array)
    }

    /// Constructs a date data
    pub fn new_date(date: CosemDate) -> Self {
        DataObject::Date(date)
    }

    /// Constructs a time data
    pub fn new_time(time: CosemTime) -> Self {
        DataObject::Time(time)
    }

    /// Constructs a date-time data
    pub fn new_date_time(date_time: CosemDateTime) -> Self {
        DataObject::DateTime(date_time)
    }

    /// Check if this DataObject contains a BitString
    pub fn is_bit_string(&self) -> bool {
        matches!(self, DataObject::BitString(_))
    }

    /// Check if this DataObject is a number
    pub fn is_number(&self) -> bool {
        self.get_type().is_number()
    }

    /// Check if this DataObject is a complex type (Array, Structure, or CompactArray)
    pub fn is_complex(&self) -> bool {
        matches!(
            self,
            DataObject::Array(_) | DataObject::Structure(_) | DataObject::CompactArray(_)
        )
    }

    /// Check if this DataObject is a byte array
    pub fn is_byte_array(&self) -> bool {
        matches!(
            self,
            DataObject::OctetString(_) | DataObject::VisibleString(_) | DataObject::Utf8String(_)
        )
    }

    /// Check if this DataObject is a boolean
    pub fn is_boolean(&self) -> bool {
        matches!(self, DataObject::Boolean(_))
    }

    /// Check if this DataObject is a date/time format
    pub fn is_cosem_date_format(&self) -> bool {
        matches!(
            self,
            DataObject::Date(_) | DataObject::Time(_) | DataObject::DateTime(_)
        )
    }

    /// Check if this DataObject is null
    pub fn is_null(&self) -> bool {
        matches!(self, DataObject::Null)
    }

    /// Get the value as a boolean
    pub fn as_bool(&self) -> DlmsResult<bool> {
        match self {
            DataObject::Boolean(b) => Ok(*b),
            _ => Err(DlmsError::InvalidData(format!(
                "Expected Boolean, got {:?}",
                self.get_type()
            ))),
        }
    }

    /// Get the value as an integer 32
    pub fn as_integer32(&self) -> DlmsResult<i32> {
        match self {
            DataObject::Integer32(i) => Ok(*i),
            _ => Err(DlmsError::InvalidData(format!(
                "Expected Integer32, got {:?}",
                self.get_type()
            ))),
        }
    }

    /// Get the value as an unsigned integer 32
    pub fn as_unsigned32(&self) -> DlmsResult<u32> {
        match self {
            DataObject::Unsigned32(u) => Ok(*u),
            _ => Err(DlmsError::InvalidData(format!(
                "Expected Unsigned32, got {:?}",
                self.get_type()
            ))),
        }
    }

    /// Get the value as an octet string
    pub fn as_octet_string(&self) -> DlmsResult<&Vec<u8>> {
        match self {
            DataObject::OctetString(s) => Ok(s),
            _ => Err(DlmsError::InvalidData(format!(
                "Expected OctetString, got {:?}",
                self.get_type()
            ))),
        }
    }

    /// Get the value as an array
    pub fn as_array(&self) -> DlmsResult<&Vec<DataObject>> {
        match self {
            DataObject::Array(a) => Ok(a),
            _ => Err(DlmsError::InvalidData(format!(
                "Expected Array, got {:?}",
                self.get_type()
            ))),
        }
    }

    /// Get the value as a structure
    pub fn as_structure(&self) -> DlmsResult<&Vec<DataObject>> {
        match self {
            DataObject::Structure(s) => Ok(s),
            _ => Err(DlmsError::InvalidData(format!(
                "Expected Structure, got {:?}",
                self.get_type()
            ))),
        }
    }
}

impl fmt::Display for DataObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataObject::Null => write!(f, "NULL_DATA"),
            DataObject::Boolean(b) => write!(f, "BOOLEAN: {}", b),
            DataObject::Integer8(i) => write!(f, "INTEGER: {}", i),
            DataObject::Integer16(i) => write!(f, "LONG_INTEGER: {}", i),
            DataObject::Integer32(i) => write!(f, "DOUBLE_LONG: {}", i),
            DataObject::Integer64(i) => write!(f, "LONG64: {}", i),
            DataObject::Unsigned8(u) => write!(f, "UNSIGNED: {}", u),
            DataObject::Unsigned16(u) => write!(f, "LONG_UNSIGNED: {}", u),
            DataObject::Unsigned32(u) => write!(f, "DOUBLE_LONG_UNSIGNED: {}", u),
            DataObject::Unsigned64(u) => write!(f, "LONG64_UNSIGNED: {}", u),
            DataObject::Float32(fl) => write!(f, "FLOAT32: {}", fl),
            DataObject::Float64(fl) => write!(f, "FLOAT64: {}", fl),
            DataObject::Enumerate(e) => write!(f, "ENUMERATE: {}", e),
            DataObject::Bcd(b) => write!(f, "BCD: {}", b),
            DataObject::OctetString(s) => {
                write!(f, "OCTET_STRING: ")?;
                for byte in s {
                    write!(f, "{:02X} ", byte)?;
                }
                Ok(())
            }
            DataObject::VisibleString(s) => {
                write!(f, "VISIBLE_STRING: {}", String::from_utf8_lossy(s))
            }
            DataObject::Utf8String(s) => {
                write!(f, "UTF8_STRING: {}", String::from_utf8_lossy(s))
            }
            DataObject::BitString(bs) => write!(f, "BIT_STRING: {}", bs),
            DataObject::Array(arr) => {
                write!(f, "ARRAY: {} element(s)", arr.len())?;
                for (i, elem) in arr.iter().enumerate() {
                    write!(f, "\n  [{}]: {}", i, elem)?;
                }
                Ok(())
            }
            DataObject::Structure(s) => {
                write!(f, "STRUCTURE: {} element(s)", s.len())?;
                for (i, elem) in s.iter().enumerate() {
                    write!(f, "\n  [{}]: {}", i, elem)?;
                }
                Ok(())
            }
            DataObject::CompactArray(_) => write!(f, "COMPACT_ARRAY"),
            DataObject::Date(d) => write!(f, "DATE: {}", d),
            DataObject::Time(t) => write!(f, "TIME: {}", t),
            DataObject::DateTime(dt) => write!(f, "DATE_TIME: {}", dt),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_object_null() {
        let obj = DataObject::new_null();
        assert!(obj.is_null());
        assert_eq!(obj.get_type(), DataObjectType::NullData);
    }

    #[test]
    fn test_data_object_boolean() {
        let obj = DataObject::new_bool(true);
        assert!(obj.is_boolean());
        assert_eq!(obj.as_bool().unwrap(), true);
    }

    #[test]
    fn test_data_object_array() {
        let arr = vec![
            DataObject::new_integer32(1),
            DataObject::new_integer32(2),
            DataObject::new_integer32(3),
        ];
        let obj = DataObject::new_array(arr).unwrap();
        assert!(obj.is_complex());
        assert_eq!(obj.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_data_object_array_mixed_types() {
        let arr = vec![
            DataObject::new_integer32(1),
            DataObject::new_bool(true),
        ];
        assert!(DataObject::new_array(arr).is_err());
    }
}
