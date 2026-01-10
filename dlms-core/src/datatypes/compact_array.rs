//! Compact array type for DLMS/COSEM protocol

use crate::error::{DlmsError, DlmsResult};
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
}
