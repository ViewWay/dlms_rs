//! Compact array type for DLMS/COSEM protocol
//!
//! Compact arrays are used to efficiently store arrays of identical data types.
//! The encoding consists of:
//! - [0] contents-description: TypeDescription
//! - [1] array-contents: IMPLICIT OCTET STRING

use crate::error::{DlmsError, DlmsResult};
use crate::datatypes::cosem_date::CosemDateFormat;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A COSEM compact array type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactArray {
    type_description: TypeDesc,
    array_contents: Vec<u8>,
}

impl CompactArray {
    /// Create a new CompactArray
    pub fn new(type_description: TypeDesc, array_contents: Vec<u8>) -> Self {
        Self {
            type_description,
            array_contents,
        }
    }

    /// Get the type description
    pub fn type_description(&self) -> &TypeDesc {
        &self.type_description
    }

    /// Get the array contents
    pub fn array_contents(&self) -> &[u8] {
        &self.array_contents
    }

    /// Encode the CompactArray to A-XDR format
    ///
    /// # Encoding Format
    /// ```text
    /// compact-array ::= SEQUENCE {
    ///     contents-description [0] IMPLICIT TypeDescription,
    ///     array-contents [1] IMPLICIT OCTET STRING
    /// }
    /// ```
    ///
    /// The encoding uses context-specific tags:
    /// - Tag 0x80 (context-specific 0, constructed) for contents-description
    /// - Tag 0x81 (context-specific 1, primitive) for array-contents
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut result = Vec::new();

        // Encode contents-description [0] IMPLICIT TypeDescription
        result.push(0x80); // Context-specific tag 0, constructed
        let type_desc_bytes = self.type_description.encode()?;
        // Encode length of type description
        if type_desc_bytes.len() < 128 {
            result.push(type_desc_bytes.len() as u8);
        } else {
            // Long form length encoding
            let len = type_desc_bytes.len();
            let num_bytes = (len as f64).log2() as usize / 8 + 1;
            result.push(0x80 | num_bytes as u8);
            result.extend_from_slice(&len.to_be_bytes()[8 - num_bytes..]);
        }
        result.extend_from_slice(&type_desc_bytes);

        // Encode array-contents [1] IMPLICIT OCTET STRING
        result.push(0x81); // Context-specific tag 1, primitive
        // Encode length of array contents
        if self.array_contents.len() < 128 {
            result.push(self.array_contents.len() as u8);
        } else {
            // Long form length encoding
            let len = self.array_contents.len();
            let num_bytes = (len as f64).log2() as usize / 8 + 1;
            result.push(0x80 | num_bytes as u8);
            result.extend_from_slice(&len.to_be_bytes()[8 - num_bytes..]);
        }
        result.extend_from_slice(&self.array_contents);

        Ok(result)
    }

    /// Decode a CompactArray from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData("Empty CompactArray data".to_string()));
        }

        let mut pos = 0;

        // Parse contents-description [0]
        if pos >= data.len() || data[pos] != 0x80 {
            return Err(DlmsError::InvalidData(
                format!("Expected tag 0x80 for contents-description, got 0x{:02X}",
                    if pos < data.len() { data[pos] } else { 0 })
            ));
        }
        pos += 1;

        // Parse length of type description
        let (type_desc_len, len_bytes) = Self::parse_length(&data[pos..])?;
        pos += len_bytes;

        if pos + type_desc_len > data.len() {
            return Err(DlmsError::InvalidData(
                format!("TypeDescription length {} exceeds available data", type_desc_len)
            ));
        }

        let type_description = TypeDesc::decode(&data[pos..pos + type_desc_len])?;
        pos += type_desc_len;

        // Parse array-contents [1]
        if pos >= data.len() || data[pos] != 0x81 {
            return Err(DlmsError::InvalidData(
                format!("Expected tag 0x81 for array-contents, got 0x{:02X}",
                    if pos < data.len() { data[pos] } else { 0 })
            ));
        }
        pos += 1;

        // Parse length of array contents
        let (array_contents_len, len_bytes2) = Self::parse_length(&data[pos..])?;
        pos += len_bytes2;

        if pos + array_contents_len > data.len() {
            return Err(DlmsError::InvalidData(
                format!("Array contents length {} exceeds available data", array_contents_len)
            ));
        }

        let array_contents = data[pos..pos + array_contents_len].to_vec();

        Ok(CompactArray {
            type_description,
            array_contents,
        })
    }

    /// Parse a BER-encoded length
    fn parse_length(data: &[u8]) -> DlmsResult<(usize, usize)> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData("Empty length data".to_string()));
        }

        let first_byte = data[0];
        if first_byte & 0x80 == 0 {
            // Short form: 0-127
            Ok((first_byte as usize, 1))
        } else {
            // Long form
            let num_bytes = (first_byte & 0x7F) as usize;
            if num_bytes == 0 || num_bytes > 4 {
                return Err(DlmsError::InvalidData(
                    format!("Invalid length byte count: {}", num_bytes)
                ));
            }
            if data.len() < 1 + num_bytes {
                return Err(DlmsError::InvalidData(
                    format!("Not enough bytes for length: need {}, have {}",
                        1 + num_bytes, data.len())
                ));
            }

            let mut len = 0usize;
            for i in 0..num_bytes {
                len = (len << 8) | (data[1 + i] as usize);
            }
            Ok((len, 1 + num_bytes))
        }
    }

    /// Create a CompactArray from a simple type list
    ///
    /// # Arguments
    /// * `elements` - Vector of DataObject elements (must all be the same type)
    ///
    /// # Errors
    /// Returns an error if elements have different types
    pub fn from_elements(elements: Vec<crate::datatypes::DataObject>) -> DlmsResult<Self> {
        if elements.is_empty() {
            return Ok(CompactArray::new(
                TypeDesc::new(Type::NullData),
                Vec::new(),
            ));
        }

        // Get the type of the first element
        let first_type = elements[0].get_type();
        let type_desc = TypeDesc::from_data_object_type(&first_type)?;

        // Encode all elements
        let mut contents = Vec::new();
        for elem in &elements {
            if elem.get_type() != first_type {
                return Err(DlmsError::InvalidData(format!(
                    "All elements must be the same type: expected {:?}, got {:?}",
                    first_type, elem.get_type()
                )));
            }
            // Encode single element (without tag)
            contents.extend_from_slice(&Self::encode_element_value(elem)?);
        }

        Ok(CompactArray::new(type_desc, contents))
    }

    /// Encode a single DataObject element (value only, no tag)
    fn encode_element_value(elem: &crate::datatypes::DataObject) -> DlmsResult<Vec<u8>> {
        use crate::datatypes::DataObject;

        match elem {
            DataObject::Null => Ok(Vec::new()),
            DataObject::Boolean(b) => Ok(vec![if *b { 0xFF } else { 0x00 }]),
            DataObject::Integer8(i) => Ok(vec![*i as u8]),
            DataObject::Integer16(i) => Ok(i.to_be_bytes().to_vec()),
            DataObject::Integer32(i) => Ok(i.to_be_bytes().to_vec()),
            DataObject::Integer64(i) => Ok(i.to_be_bytes().to_vec()),
            DataObject::Unsigned8(u) => Ok(vec![*u]),
            DataObject::Unsigned16(u) => Ok(u.to_be_bytes().to_vec()),
            DataObject::Unsigned32(u) => Ok(u.to_be_bytes().to_vec()),
            DataObject::Unsigned64(u) => Ok(u.to_be_bytes().to_vec()),
            DataObject::Float32(f) => Ok(f.to_bits().to_be_bytes().to_vec()),
            DataObject::Float64(f) => Ok(f.to_bits().to_be_bytes().to_vec()),
            DataObject::Enumerate(e) => Ok(vec![*e]),
            DataObject::Bcd(b) => Ok(vec![*b]),
            DataObject::OctetString(s) | DataObject::VisibleString(s) | DataObject::Utf8String(s) => {
                Ok(s.clone())
            }
            DataObject::BitString(bs) => {
                Ok(bs.as_bytes().to_vec())
            }
            DataObject::Date(d) => Ok(d.encode()),
            DataObject::Time(t) => Ok(t.encode()),
            DataObject::DateTime(dt) => Ok(dt.encode()),
            _ => Err(DlmsError::InvalidData(
                format!("Cannot encode {:?} as compact array element", elem.get_type())
            )),
        }
    }
}

/// The type description of a COSEM Compact Array
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeDesc {
    type_: Type,
    value: Option<Vec<TypeDesc>>, // For ARRAY or STRUCTURE types
}

impl TypeDesc {
    /// Create a new TypeDesc with a value (for ARRAY or STRUCTURE)
    pub fn new_with_value(value: Vec<TypeDesc>, type_: Type) -> DlmsResult<Self> {
        if (type_ == Type::Array || type_ == Type::Structure) && value.is_empty() {
            return Err(DlmsError::InvalidData(
                "For type structure/array the value must be set!".to_string(),
            ));
        }
        Ok(Self {
            type_: type_,
            value: Some(value),
        })
    }

    /// Create a new TypeDesc without a value
    pub fn new(type_: Type) -> Self {
        Self {
            type_: type_,
            value: None,
        }
    }

    /// Get the type
    pub fn get_type(&self) -> &Type {
        &self.type_
    }

    /// Get the value (for ARRAY or STRUCTURE types)
    pub fn get_value(&self) -> Option<&Vec<TypeDesc>> {
        self.value.as_ref()
    }

    /// Create a TypeDesc from a DataObjectType
    pub fn from_data_object_type(data_type: &crate::datatypes::DataObjectType) -> DlmsResult<Self> {
        use crate::datatypes::DataObjectType;
        let type_ = match data_type {
            DataObjectType::NullData => Type::NullData,
            DataObjectType::Array => Type::Array,
            DataObjectType::Structure => Type::Structure,
            DataObjectType::Boolean => Type::Bool,
            DataObjectType::BitString => Type::BitString,
            DataObjectType::DoubleLong => Type::DoubleLong,
            DataObjectType::DoubleLongUnsigned => Type::DoubleLongUnsigned,
            DataObjectType::OctetString => Type::OctetString,
            DataObjectType::Utf8String => Type::Utf8String,
            DataObjectType::VisibleString => Type::VisibleString,
            DataObjectType::Bcd => Type::Bcd,
            DataObjectType::Integer => Type::Integer,
            DataObjectType::LongInteger => Type::LongInteger,
            DataObjectType::Unsigned => Type::Unsigned,
            DataObjectType::LongUnsigned => Type::LongUnsigned,
            DataObjectType::Long64 => Type::Long64,
            DataObjectType::Long64Unsigned => Type::Long64Unsigned,
            DataObjectType::Enumerate => Type::Enumerate,
            DataObjectType::Float32 => Type::Float32,
            DataObjectType::Float64 => Type::Float64,
            DataObjectType::DateTime => Type::DateTime,
            DataObjectType::Date => Type::Date,
            DataObjectType::Time => Type::Time,
            DataObjectType::DontCare => Type::DontCare,
            DataObjectType::CompactArray => {
                return Err(DlmsError::InvalidData(
                    "CompactArray cannot be nested in CompactArray".to_string()
                ))
            }
        };
        Ok(TypeDesc::new(type_))
    }

    /// Encode the TypeDesc to A-XDR format
    ///
    /// # Encoding Format
    /// ```text
    /// TypeDescription ::= CHOICE {
    ///     null-data             [0] IMPLICIT NULL,
    ///     array                 [1] IMPLICIT SEQUENCE OF TypeDescription,
    ///     structure             [2] IMPLICIT SEQUENCE OF TypeDescription,
    ///     bool                  [3] IMPLICIT NULL,
    ///     bit-string            [4] IMPLICIT NULL,
    ///     double-long           [5] IMPLICIT NULL,
    ///     double-long-unsigned  [6] IMPLICIT NULL,
    ///     octet-string          [9] IMPLICIT NULL,
    ///     visible-string        [10] IMPLICIT NULL,
    ///     utf8-string           [12] IMPLICIT NULL,
    ///     bcd                   [13] IMPLICIT NULL,
    ///     integer               [15] IMPLICIT NULL,
    ///     long-integer          [16] IMPLICIT NULL,
    ///     unsigned              [17] IMPLICIT NULL,
    ///     long-unsigned         [18] IMPLICIT NULL,
    ///     long64                [20] IMPLICIT NULL,
    ///     long64-unsigned       [21] IMPLICIT NULL,
    ///     enum                  [22] IMPLICIT NULL,
    ///     float32               [23] IMPLICIT NULL,
    ///     float64               [24] IMPLICIT NULL,
    ///     date-time             [25] IMPLICIT NULL,
    ///     date                  [26] IMPLICIT NULL,
    ///     time                  [27] IMPLICIT NULL
    /// }
    /// ```
    ///
    /// For simple types: only the tag (context-specific number) is encoded
    /// For Array/Structure: tag + number of elements + element descriptions
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut result = Vec::new();

        match self.type_ {
            Type::Array | Type::Structure => {
                // Encode tag
                result.push(0x80 | (self.type_.code() as u8));

                if let Some(ref value) = self.value {
                    // Number of elements
                    result.push(value.len() as u8);

                    // Encode each element description
                    for elem_desc in value {
                        result.extend_from_slice(&elem_desc.encode()?);
                    }
                } else {
                    return Err(DlmsError::InvalidData(
                        format!("{:?} requires value", self.type_)
                    ));
                }
            }
            _ => {
                // Simple types: only the tag
                result.push(0x80 | (self.type_.code() as u8));
            }
        }

        Ok(result)
    }

    /// Decode a TypeDesc from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData("Empty TypeDesc data".to_string()));
        }

        let first_byte = data[0];
        // Extract tag number (remove context-specific flag 0x80)
        let tag_num = (first_byte & 0x7F) as i32;
        let type_ = Type::from_code(tag_num);

        if type_ == Type::ErrNoneSelected {
            return Err(DlmsError::InvalidData(
                format!("Invalid type code: {}", tag_num)
            ));
        }

        match type_ {
            Type::Array | Type::Structure => {
                // Parse number of elements
                if data.len() < 2 {
                    return Err(DlmsError::InvalidData(
                        format!("Not enough data for {:?}: need at least 2 bytes", type_)
                    ));
                }

                let num_elements = data[1] as usize;
                let mut value = Vec::with_capacity(num_elements);
                let mut pos = 2;

                for _ in 0..num_elements {
                    if pos >= data.len() {
                        return Err(DlmsError::InvalidData(
                            format!("Not enough data for element descriptions")
                        ));
                    }

                    // Find the end of this element description
                    // For simple types, it's just 1 byte
                    let next_byte = data[pos];
                    let next_tag = Type::from_code((next_byte & 0x7F) as i32);

                    let elem_desc = if next_tag == Type::Array || next_tag == Type::Structure {
                        // Complex type - need to parse it fully
                        let elem_len = Self::calculate_complex_type_len(&data[pos..])?;
                        let elem_desc = TypeDesc::decode(&data[pos..pos + elem_len])?;
                        pos += elem_len;
                        elem_desc
                    } else {
                        // Simple type - just 1 byte
                        let elem_desc = TypeDesc::decode(&data[pos..pos + 1])?;
                        pos += 1;
                        elem_desc
                    };

                    value.push(elem_desc);
                }

                Ok(TypeDesc {
                    type_,
                    value: Some(value),
                })
            }
            _ => {
                // Simple types: just the tag
                Ok(TypeDesc {
                    type_,
                    value: None,
                })
            }
        }
    }

    /// Calculate the length of a complex type encoding
    fn calculate_complex_type_len(data: &[u8]) -> DlmsResult<usize> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData("Empty data for complex type".to_string()));
        }

        let first_byte = data[0];
        let tag_num = (first_byte & 0x7F) as i32;
        let type_ = Type::from_code(tag_num);

        if type_ != Type::Array && type_ != Type::Structure {
            return Ok(1); // Simple type
        }

        if data.len() < 2 {
            return Err(DlmsError::InvalidData("Incomplete complex type encoding".to_string()));
        }

        let num_elements = data[1] as usize;
        let mut pos = 2;
        let mut total = 2;

        for _ in 0..num_elements {
            if pos >= data.len() {
                return Err(DlmsError::InvalidData("Incomplete complex type encoding".to_string()));
            }

            let next_byte = data[pos];
            let next_tag = Type::from_code((next_byte & 0x7F) as i32);

            if next_tag == Type::Array || next_tag == Type::Structure {
                let nested_len = Self::calculate_complex_type_len(&data[pos..])?;
                pos += nested_len;
                total += nested_len;
            } else {
                pos += 1;
                total += 1;
            }
        }

        Ok(total)
    }
}

/// The types of a type description
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    ErrNoneSelected = -1,
    NullData = 0,
    Array = 1,
    Structure = 2,
    Bool = 3,
    BitString = 4,
    DoubleLong = 5,
    DoubleLongUnsigned = 6,
    OctetString = 9,
    VisibleString = 10,
    Utf8String = 12,
    Bcd = 13,
    Integer = 15,
    LongInteger = 16,
    Unsigned = 17,
    LongUnsigned = 18,
    Long64 = 20,
    Long64Unsigned = 21,
    Enumerate = 22,
    Float32 = 23,
    Float64 = 24,
    DateTime = 25,
    Date = 26,
    Time = 27,
    DontCare = 255,
}

impl Type {
    /// Get the type code value
    pub fn code(&self) -> i32 {
        *self as i32
    }

    /// Get type from code value
    pub fn from_code(code: i32) -> Self {
        match code {
            0 => Type::NullData,
            1 => Type::Array,
            2 => Type::Structure,
            3 => Type::Bool,
            4 => Type::BitString,
            5 => Type::DoubleLong,
            6 => Type::DoubleLongUnsigned,
            9 => Type::OctetString,
            10 => Type::VisibleString,
            12 => Type::Utf8String,
            13 => Type::Bcd,
            15 => Type::Integer,
            16 => Type::LongInteger,
            17 => Type::Unsigned,
            18 => Type::LongUnsigned,
            20 => Type::Long64,
            21 => Type::Long64Unsigned,
            22 => Type::Enumerate,
            23 => Type::Float32,
            24 => Type::Float64,
            25 => Type::DateTime,
            26 => Type::Date,
            27 => Type::Time,
            255 => Type::DontCare,
            _ => Type::ErrNoneSelected,
        }
    }
}

/// The description array of a COSEM Compact Array
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescriptionArray {
    num_of_elements: usize,
    type_description: TypeDesc,
}

impl DescriptionArray {
    /// Create a new DescriptionArray
    pub fn new(num_of_elements: usize, type_description: TypeDesc) -> Self {
        Self {
            num_of_elements,
            type_description,
        }
    }

    /// Get the number of elements
    pub fn num_of_elements(&self) -> usize {
        self.num_of_elements
    }

    /// Get the type description
    pub fn type_description(&self) -> &TypeDesc {
        &self.type_description
    }
}

impl fmt::Display for CompactArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "COMPACT_ARRAY(type={:?}, size={})",
            self.type_description.get_type(),
            self.array_contents.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_array_new() {
        let type_desc = TypeDesc::new(Type::Integer);
        let contents = vec![1, 2, 3, 4];
        let compact_array = CompactArray::new(type_desc, contents);
        assert_eq!(compact_array.array_contents().len(), 4);
    }

    #[test]
    fn test_type_from_code() {
        assert_eq!(Type::from_code(0), Type::NullData);
        assert_eq!(Type::from_code(15), Type::Integer);
        assert_eq!(Type::from_code(255), Type::DontCare);
    }

    #[test]
    fn test_type_desc_encode_simple() {
        // Test simple type encoding (Integer)
        let type_desc = TypeDesc::new(Type::Integer);
        let encoded = type_desc.encode().unwrap();
        // Expected: 0x80 | 15 = 0x8F (context-specific tag 15)
        assert_eq!(encoded, vec![0x8F]);
    }

    #[test]
    fn test_type_desc_encode_float32() {
        let type_desc = TypeDesc::new(Type::Float32);
        let encoded = type_desc.encode().unwrap();
        // Expected: 0x80 | 23 = 0x97
        assert_eq!(encoded, vec![0x97]);
    }

    #[test]
    fn test_type_desc_encode_float64() {
        let type_desc = TypeDesc::new(Type::Float64);
        let encoded = type_desc.encode().unwrap();
        // Expected: 0x80 | 24 = 0x98
        assert_eq!(encoded, vec![0x98]);
    }

    #[test]
    fn test_type_desc_decode_simple() {
        let encoded = vec![0x8F]; // Integer
        let type_desc = TypeDesc::decode(&encoded).unwrap();
        assert_eq!(type_desc.get_type(), &Type::Integer);
        assert!(type_desc.get_value().is_none());
    }

    #[test]
    fn test_type_desc_decode_float32() {
        let encoded = vec![0x97]; // Float32
        let type_desc = TypeDesc::decode(&encoded).unwrap();
        assert_eq!(type_desc.get_type(), &Type::Float32);
    }

    #[test]
    fn test_type_desc_decode_float64() {
        let encoded = vec![0x98]; // Float64
        let type_desc = TypeDesc::decode(&encoded).unwrap();
        assert_eq!(type_desc.get_type(), &Type::Float64);
    }

    #[test]
    fn test_type_desc_encode_array() {
        // Test array type encoding (e.g., Array of Integer)
        let elem_type = TypeDesc::new(Type::Integer);
        let type_desc = TypeDesc::new_with_value(vec![elem_type], Type::Array).unwrap();
        let encoded = type_desc.encode().unwrap();
        // Expected: tag (0x81) + count (1) + element type (0x8F)
        assert_eq!(encoded, vec![0x81, 0x01, 0x8F]);
    }

    #[test]
    fn test_type_desc_decode_array() {
        let encoded = vec![0x81, 0x01, 0x8F]; // Array with 1 Integer element
        let type_desc = TypeDesc::decode(&encoded).unwrap();
        assert_eq!(type_desc.get_type(), &Type::Array);
        assert!(type_desc.get_value().is_some());
        let value = type_desc.get_value().unwrap();
        assert_eq!(value.len(), 1);
        assert_eq!(value[0].get_type(), &Type::Integer);
    }

    #[test]
    fn test_compact_array_encode() {
        // Create a simple compact array with Integer type
        let type_desc = TypeDesc::new(Type::Integer);
        let contents = vec![0x01, 0x02, 0x03, 0x04];
        let compact_array = CompactArray::new(type_desc, contents);

        let encoded = compact_array.encode().unwrap();
        // Structure: [0x80] [len] [type_desc] [0x81] [len] [contents]
        // type_desc for Integer = 0x8F
        assert_eq!(encoded[0], 0x80); // contents-description tag
        assert_eq!(encoded[1], 0x01); // length of type_desc
        assert_eq!(encoded[2], 0x8F); // Integer type
        assert_eq!(encoded[3], 0x81); // array-contents tag
        assert_eq!(encoded[4], 0x04); // length of contents
        assert_eq!(encoded[5..9], vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_compact_array_decode() {
        // Encoded compact array with Integer type and 4 bytes of data
        let encoded = vec![
            0x80,       // contents-description tag
            0x01,       // length of type_desc
            0x8F,       // Integer type
            0x81,       // array-contents tag
            0x04,       // length of contents
            0x01, 0x02, 0x03, 0x04, // data
        ];

        let compact_array = CompactArray::decode(&encoded).unwrap();
        assert_eq!(compact_array.type_description().get_type(), &Type::Integer);
        assert_eq!(compact_array.array_contents(), &[0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_compact_array_roundtrip() {
        // Create and encode a compact array
        let type_desc = TypeDesc::new(Type::Float32);
        let contents = vec![0x3F, 0x80, 0x00, 0x00]; // 1.0 in IEEE 754
        let original = CompactArray::new(type_desc, contents);

        let encoded = original.encode().unwrap();
        let decoded = CompactArray::decode(&encoded).unwrap();

        assert_eq!(decoded.type_description().get_type(), original.type_description().get_type());
        assert_eq!(decoded.array_contents(), original.array_contents());
    }

    #[test]
    fn test_parse_length_short() {
        // Short form length: 5
        let data = vec![0x05];
        let (len, consumed) = CompactArray::parse_length(&data).unwrap();
        assert_eq!(len, 5);
        assert_eq!(consumed, 1);
    }

    #[test]
    fn test_parse_length_long() {
        // Long form length: 2 bytes, value = 0x0123
        let data = vec![0x82, 0x01, 0x23];
        let (len, consumed) = CompactArray::parse_length(&data).unwrap();
        assert_eq!(len, 0x0123);
        assert_eq!(consumed, 3);
    }

    #[test]
    fn test_type_from_data_object_type() {
        use crate::datatypes::DataObjectType;

        let float32_type = TypeDesc::from_data_object_type(&DataObjectType::Float32).unwrap();
        assert_eq!(float32_type.get_type(), &Type::Float32);

        let float64_type = TypeDesc::from_data_object_type(&DataObjectType::Float64).unwrap();
        assert_eq!(float64_type.get_type(), &Type::Float64);

        let int_type = TypeDesc::from_data_object_type(&DataObjectType::Integer).unwrap();
        assert_eq!(int_type.get_type(), &Type::Integer);
    }
}
