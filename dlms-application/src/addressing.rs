//! Addressing module for DLMS/COSEM application layer
//!
//! This module provides addressing mechanisms for DLMS/COSEM objects:
//! - Logical Name (LN) addressing: Uses OBIS codes to identify objects
//! - Short Name (SN) addressing: Uses 16-bit addresses to identify objects
//! - Object references: Class ID, instance ID, attribute/method ID
//! - Access selectors: For selective access to array/table attributes
//!
//! # Implementation Notes
//!
//! ## Why Two Addressing Methods?
//! DLMS/COSEM supports two addressing methods:
//! - **Logical Name (LN)**: More flexible, uses OBIS codes (6 bytes) to uniquely identify objects.
//!   This is the preferred method for modern implementations as it's more human-readable
//!   and doesn't require address mapping tables.
//! - **Short Name (SN)**: More compact, uses 16-bit addresses. This is legacy from older
//!   DLMS implementations and requires a mapping table (Association SN object) to convert
//!   between OBIS codes and short names.
//!
//! ## Optimization Considerations
//! - LN addressing is more verbose (6 bytes vs 2 bytes) but provides better compatibility
//! - SN addressing requires additional overhead for address mapping but can reduce
//!   message size for high-frequency operations
//! - Future optimization: Cache OBIS-to-SN mappings to reduce lookup overhead

use dlms_core::{DlmsError, DlmsResult, ObisCode};
use dlms_core::datatypes::{CosemDateTime, CosemDateFormat, DataObject};
use dlms_asn1::{AxdrDecoder, AxdrEncoder};

/// Addressing method for DLMS/COSEM objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AddressingMethod {
    /// Logical Name addressing (uses OBIS codes)
    LogicalName,
    /// Short Name addressing (uses 16-bit addresses)
    ShortName,
}

/// Object reference for Logical Name addressing
///
/// LN addressing uses:
/// - Class ID: The COSEM interface class identifier
/// - Instance ID: The OBIS code (6 bytes) identifying the object instance
/// - Attribute/Method ID: The attribute or method number within the class
///
/// # Why This Structure?
/// This structure encapsulates all information needed to reference an object
/// using LN addressing. The OBIS code provides a globally unique identifier
/// that doesn't require address mapping tables.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogicalNameReference {
    /// COSEM interface class ID
    pub class_id: u16,
    /// OBIS code (6 bytes) identifying the object instance
    pub instance_id: ObisCode,
    /// Attribute ID (for attribute access) or Method ID (for method invocation)
    pub id: u8,
}

impl LogicalNameReference {
    /// Create a new Logical Name reference
    ///
    /// # Arguments
    ///
    /// * `class_id` - COSEM interface class ID
    /// * `instance_id` - OBIS code identifying the object instance
    /// * `id` - Attribute ID (1-255) or Method ID (1-255)
    ///
    /// # Returns
    ///
    /// Returns `Ok(LogicalNameReference)` if valid, `Err(DlmsError)` otherwise
    ///
    /// # Validation
    ///
    /// - Class ID must be in range [1, 65535] (u16 range, but typically < 256)
    /// - ID must be in range [1, 255] (0 is reserved)
    pub fn new(class_id: u16, instance_id: ObisCode, id: u8) -> DlmsResult<Self> {
        if id == 0 {
            return Err(DlmsError::InvalidData(
                "Attribute/Method ID cannot be 0".to_string(),
            ));
        }
        Ok(Self {
            class_id,
            instance_id,
            id,
        })
    }

    /// Encode to A-XDR format
    ///
    /// Encoding format (A-XDR):
    /// - Class ID: Unsigned16
    /// - Instance ID: OctetString (6 bytes)
    /// - ID: Unsigned8
    ///
    /// # Why A-XDR?
    /// A-XDR (Aligned eXternal Data Representation) is the standard encoding
    /// format for DLMS/COSEM. It provides a compact, efficient binary format
    /// that's easier to parse than BER/DER encoding.
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();
        
        // Encode class ID as Unsigned16
        encoder.encode_u16(self.class_id)?;
        
        // Encode instance ID (OBIS code) as OctetString
        let obis_bytes = self.instance_id.as_bytes();
        encoder.encode_octet_string(obis_bytes)?;
        
        // Encode attribute/method ID as Unsigned8
        encoder.encode_u8(self.id)?;
        
        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);
        
        let class_id = decoder.decode_u16()?;
        let instance_bytes = decoder.decode_octet_string()?;
        
        if instance_bytes.len() != 6 {
            return Err(DlmsError::InvalidData(format!(
                "Invalid OBIS code length: expected 6 bytes, got {}",
                instance_bytes.len()
            )));
        }
        
        let instance_id = ObisCode::new(
            instance_bytes[0],
            instance_bytes[1],
            instance_bytes[2],
            instance_bytes[3],
            instance_bytes[4],
            instance_bytes[5],
        );
        
        let id = decoder.decode_u8()?;
        
        Self::new(class_id, instance_id, id)
    }
}

/// Object reference for Short Name addressing
///
/// SN addressing uses:
/// - Base Name: 16-bit address identifying the object
/// - Attribute/Method ID: The attribute or method number
///
/// # Why This Structure?
/// SN addressing is more compact (2 bytes vs 6 bytes for OBIS) but requires
/// a mapping table to convert between OBIS codes and short names. This is
/// typically provided by the Association SN object (class ID 12).
///
/// # Optimization Note
/// For high-frequency operations, SN addressing can reduce message size
/// by ~4 bytes per object reference. However, the overhead of maintaining
/// the address mapping table may offset this benefit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShortNameReference {
    /// Base name (16-bit address)
    pub base_name: u16,
    /// Attribute ID (for attribute access) or Method ID (for method invocation)
    pub id: u8,
}

impl ShortNameReference {
    /// Create a new Short Name reference
    ///
    /// # Arguments
    ///
    /// * `base_name` - 16-bit base address
    /// * `id` - Attribute ID (1-255) or Method ID (1-255)
    ///
    /// # Returns
    ///
    /// Returns `Ok(ShortNameReference)` if valid, `Err(DlmsError)` otherwise
    pub fn new(base_name: u16, id: u8) -> DlmsResult<Self> {
        if id == 0 {
            return Err(DlmsError::InvalidData(
                "Attribute/Method ID cannot be 0".to_string(),
            ));
        }
        Ok(Self { base_name, id })
    }

    /// Encode to A-XDR format
    ///
    /// Encoding format (A-XDR):
    /// - Base Name: Unsigned16
    /// - ID: Unsigned8
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();
        encoder.encode_u16(self.base_name)?;
        encoder.encode_u8(self.id)?;
        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);
        let base_name = decoder.decode_u16()?;
        let id = decoder.decode_u8()?;
        Self::new(base_name, id)
    }
}

/// Access selector for selective access to array/table attributes
///
/// Selective access allows reading/writing specific elements or ranges
/// within array or table attributes, rather than the entire attribute.
///
/// # Why Selective Access?
/// Some attributes (like Profile Generic buffer) can be very large.
/// Selective access allows:
/// - Reading specific entries by index
/// - Reading entries within a date/time range
/// - Reading entries matching certain criteria
///
/// This significantly reduces bandwidth and processing time for large datasets.
///
/// # Implementation Note
/// The current implementation supports basic access selectors. Full support
/// for complex selectors (date ranges, criteria matching) requires additional
/// COSEM ASN.1 structures that will be implemented in the COSEM ASN.1 module.
#[derive(Debug, Clone, PartialEq)]
pub enum AccessSelector {
    /// No selective access (access entire attribute)
    None,
    /// Access by entry index (for array/table attributes)
    /// 
    /// Format: [start_index, count]
    /// - start_index: First entry to access (0-based)
    /// - count: Number of entries to access
    EntryIndex {
        start_index: u32,
        count: u32,
    },
    /// Access by date range (for Profile Generic and similar)
    ///
    /// Format: [from_date, to_date]
    /// - from_date: Start date/time (inclusive)
    /// - to_date: End date/time (inclusive)
    ///
    /// # Usage
    /// Used for accessing entries in Profile Generic buffer or similar
    /// time-series data within a specific date/time range.
    DateRange {
        from_date: CosemDateTime,
        to_date: CosemDateTime,
    },
    /// Access by value range (for numeric attributes)
    ///
    /// Format: [from_value, to_value]
    /// - from_value: Start value (inclusive)
    /// - to_value: End value (inclusive)
    ///
    /// # Usage
    /// Used for accessing entries in arrays or tables where values
    /// fall within a specific numeric range.
    ValueRange {
        from_value: DataObject,
        to_value: DataObject,
    },
}

impl AccessSelector {
    /// Create an Entry Index access selector
    ///
    /// # Arguments
    /// * `start_index` - First entry to access (0-based)
    /// * `count` - Number of entries to access
    ///
    /// # Returns
    /// AccessSelector::EntryIndex variant
    pub fn entry_index(start_index: u32, count: u32) -> Self {
        Self::EntryIndex { start_index, count }
    }

    /// Create a Date Range access selector
    ///
    /// # Arguments
    /// * `from_date` - Start date/time (inclusive)
    /// * `to_date` - End date/time (inclusive)
    ///
    /// # Returns
    /// AccessSelector::DateRange variant
    pub fn date_range(from_date: CosemDateTime, to_date: CosemDateTime) -> Self {
        Self::DateRange { from_date, to_date }
    }

    /// Create a Value Range access selector
    ///
    /// # Arguments
    /// * `from_value` - Start value (inclusive)
    /// * `to_value` - End value (inclusive)
    ///
    /// # Returns
    /// AccessSelector::ValueRange variant
    ///
    /// # Note
    /// Both values must be numeric types (Integer, Unsigned, etc.)
    pub fn value_range(from_value: DataObject, to_value: DataObject) -> Self {
        Self::ValueRange { from_value, to_value }
    }

    /// Get the access selector type code
    ///
    /// # Returns
    /// Selector type code:
    /// - None: None (no selector)
    /// - EntryIndex: 0
    /// - DateRange: 1
    /// - ValueRange: 2
    pub fn selector_type(&self) -> Option<u8> {
        match self {
            AccessSelector::None => None,
            AccessSelector::EntryIndex { .. } => Some(0),
            AccessSelector::DateRange { .. } => Some(1),
            AccessSelector::ValueRange { .. } => Some(2),
        }
    }

    /// Convert to SelectiveAccessDescriptor
    ///
    /// # Returns
    /// SelectiveAccessDescriptor if selector is not None, None otherwise
    pub fn to_selective_access_descriptor(&self) -> DlmsResult<Option<crate::pdu::SelectiveAccessDescriptor>> {
        use crate::pdu::SelectiveAccessDescriptor;
        
        match self {
            AccessSelector::None => Ok(None),
            selector => {
                let selector_type = selector.selector_type().ok_or_else(|| {
                    DlmsError::InvalidData("Cannot convert None selector to descriptor".to_string())
                })?;
                
                // Encode access parameters as DataObject
                let access_parameters = match selector {
                    AccessSelector::EntryIndex { start_index, count } => {
                        DataObject::new_structure(vec![
                            DataObject::new_unsigned32(*start_index),
                            DataObject::new_unsigned32(*count),
                        ])
                    }
                    AccessSelector::DateRange { from_date, to_date } => {
                        DataObject::new_structure(vec![
                            DataObject::new_octet_string(from_date.encode()),
                            DataObject::new_octet_string(to_date.encode()),
                        ])
                    }
                    AccessSelector::ValueRange { from_value, to_value } => {
                        DataObject::new_structure(vec![
                            from_value.clone(),
                            to_value.clone(),
                        ])
                    }
                    AccessSelector::None => unreachable!(),
                };
                
                Ok(Some(SelectiveAccessDescriptor::new(selector_type, access_parameters)))
            }
        }
    }

    /// Create from SelectiveAccessDescriptor
    ///
    /// # Arguments
    /// * `descriptor` - SelectiveAccessDescriptor to convert
    ///
    /// # Returns
    /// AccessSelector variant
    pub fn from_selective_access_descriptor(descriptor: &crate::pdu::SelectiveAccessDescriptor) -> DlmsResult<Self> {
        let selector_type = descriptor.access_selector;
        let params = &descriptor.access_parameters;
        
        // Extract structure from access_parameters
        let structure = params.as_structure()?;
        
        if structure.len() != 2 {
            return Err(DlmsError::InvalidData(format!(
                "Access parameters structure must have 2 elements, got {}",
                structure.len()
            )));
        }
        
        match selector_type {
            0 => {
                // Entry Index: [Unsigned32, Unsigned32]
                let start_index = structure[0].as_unsigned32()?;
                let count = structure[1].as_unsigned32()?;
                Ok(AccessSelector::EntryIndex { start_index, count })
            }
            1 => {
                // Date Range: [OctetString, OctetString] (each is CosemDateTime)
                let from_bytes = structure[0].as_octet_string()?;
                let to_bytes = structure[1].as_octet_string()?;
                
                let from_date = CosemDateTime::decode(from_bytes)?;
                let to_date = CosemDateTime::decode(to_bytes)?;
                
                Ok(AccessSelector::DateRange { from_date, to_date })
            }
            2 => {
                // Value Range: [DataObject, DataObject]
                Ok(AccessSelector::ValueRange {
                    from_value: structure[0].clone(),
                    to_value: structure[1].clone(),
                })
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Unknown access selector type: {}",
                selector_type
            ))),
        }
    }

    /// Encode access selector to A-XDR format
    ///
    /// Encoding format:
    /// - None: Not encoded (omitted from PDU)
    /// - EntryIndex: Structure containing [Unsigned32, Unsigned32]
    /// - DateRange: Structure containing [OctetString, OctetString] (CosemDateTime encoded)
    /// - ValueRange: Structure containing [DataObject, DataObject]
    ///
    /// # Why This Encoding?
    /// A-XDR structures are encoded as arrays of elements. This allows
    /// the decoder to determine the selector type by the structure content.
    pub fn encode(&self) -> DlmsResult<Option<Vec<u8>>> {
        match self {
            AccessSelector::None => Ok(None),
            AccessSelector::EntryIndex { start_index, count } => {
                // Encode as Structure with 2 Unsigned32 elements
                let structure = vec![
                    DataObject::new_unsigned32(*start_index),
                    DataObject::new_unsigned32(*count),
                ];
                let mut encoder = AxdrEncoder::new();
                encoder.encode_structure(&structure)?;
                Ok(Some(encoder.into_bytes()))
            }
            AccessSelector::DateRange { from_date, to_date } => {
                // Encode as Structure with 2 OctetString elements (CosemDateTime encoded)
                let structure = vec![
                    DataObject::new_octet_string(from_date.encode()),
                    DataObject::new_octet_string(to_date.encode()),
                ];
                let mut encoder = AxdrEncoder::new();
                encoder.encode_structure(&structure)?;
                Ok(Some(encoder.into_bytes()))
            }
            AccessSelector::ValueRange { from_value, to_value } => {
                // Encode as Structure with 2 DataObject elements
                let structure = vec![
                    from_value.clone(),
                    to_value.clone(),
                ];
                let mut encoder = AxdrEncoder::new();
                encoder.encode_structure(&structure)?;
                Ok(Some(encoder.into_bytes()))
            }
        }
    }

    /// Decode access selector from A-XDR format
    ///
    /// # Note
    /// This method attempts to auto-detect the selector type by examining
    /// the structure content. For explicit type detection, use
    /// `from_selective_access_descriptor()` which includes the selector type.
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);
        
        // Decode structure
        let structure = decoder.decode_structure()?;
        if structure.len() != 2 {
            return Err(DlmsError::InvalidData(format!(
                "Expected structure length 2, got {}",
                structure.len()
            )));
        }
        
        // Try to decode as EntryIndex first (two Unsigned32)
        if let (Ok(start_index), Ok(count)) = (
            structure[0].as_unsigned32(),
            structure[1].as_unsigned32(),
        ) {
            return Ok(AccessSelector::EntryIndex { start_index, count });
        }
        
        // Try DateRange (two OctetString, each is CosemDateTime)
        if let (Ok(from_bytes), Ok(to_bytes)) = (
            structure[0].as_octet_string(),
            structure[1].as_octet_string(),
        ) {
            // Check if they are valid CosemDateTime (12 bytes each)
            if from_bytes.len() == CosemDateTime::LENGTH && to_bytes.len() == CosemDateTime::LENGTH {
                if let (Ok(from_date), Ok(to_date)) = (
                    CosemDateTime::decode(from_bytes),
                    CosemDateTime::decode(to_bytes),
                ) {
                    return Ok(AccessSelector::DateRange { from_date, to_date });
                }
            }
        }
        
        // Try ValueRange (any two DataObjects)
        // This is a fallback - if it's not EntryIndex or DateRange, assume ValueRange
        Ok(AccessSelector::ValueRange {
            from_value: structure[0].clone(),
            to_value: structure[1].clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logical_name_reference() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let reference = LogicalNameReference::new(1, obis, 2).unwrap();
        
        assert_eq!(reference.class_id, 1);
        assert_eq!(reference.instance_id, obis);
        assert_eq!(reference.id, 2);
    }

    #[test]
    fn test_logical_name_reference_encode_decode() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let reference = LogicalNameReference::new(1, obis, 2).unwrap();
        
        let encoded = reference.encode().unwrap();
        let decoded = LogicalNameReference::decode(&encoded).unwrap();
        
        assert_eq!(reference, decoded);
    }

    #[test]
    fn test_short_name_reference() {
        let reference = ShortNameReference::new(0x1234, 2).unwrap();
        
        assert_eq!(reference.base_name, 0x1234);
        assert_eq!(reference.id, 2);
    }

    #[test]
    fn test_short_name_reference_encode_decode() {
        let reference = ShortNameReference::new(0x1234, 2).unwrap();
        
        let encoded = reference.encode().unwrap();
        let decoded = ShortNameReference::decode(&encoded).unwrap();
        
        assert_eq!(reference, decoded);
    }

    #[test]
    fn test_access_selector_entry_index() {
        let selector = AccessSelector::entry_index(10, 5);
        
        let encoded = selector.encode().unwrap();
        assert!(encoded.is_some());
        
        let decoded = AccessSelector::decode(encoded.as_ref().unwrap()).unwrap();
        match decoded {
            AccessSelector::EntryIndex { start_index, count } => {
                assert_eq!(start_index, 10);
                assert_eq!(count, 5);
            }
            _ => panic!("Expected EntryIndex"),
        }
    }

    #[test]
    fn test_access_selector_date_range() {
        use dlms_core::datatypes::ClockStatus;
        
        let from_date = CosemDateTime::new(2024, 1, 1, 0, 0, 0, 0, &[]).unwrap();
        let to_date = CosemDateTime::new(2024, 12, 31, 23, 59, 59, 0, &[]).unwrap();
        let selector = AccessSelector::date_range(from_date.clone(), to_date.clone());
        
        let encoded = selector.encode().unwrap();
        assert!(encoded.is_some());
        
        let decoded = AccessSelector::decode(encoded.as_ref().unwrap()).unwrap();
        match decoded {
            AccessSelector::DateRange { from_date: fd, to_date: td } => {
                assert_eq!(fd.encode(), from_date.encode());
                assert_eq!(td.encode(), to_date.encode());
            }
            _ => panic!("Expected DateRange"),
        }
    }

    #[test]
    fn test_access_selector_value_range() {
        let from_value = DataObject::new_integer32(-100);
        let to_value = DataObject::new_integer32(100);
        let selector = AccessSelector::value_range(from_value.clone(), to_value.clone());
        
        let encoded = selector.encode().unwrap();
        assert!(encoded.is_some());
        
        let decoded = AccessSelector::decode(encoded.as_ref().unwrap()).unwrap();
        match decoded {
            AccessSelector::ValueRange { from_value: fv, to_value: tv } => {
                assert_eq!(fv.as_integer32().unwrap(), -100);
                assert_eq!(tv.as_integer32().unwrap(), 100);
            }
            _ => panic!("Expected ValueRange"),
        }
    }

    #[test]
    fn test_access_selector_to_selective_access_descriptor() {
        use crate::pdu::SelectiveAccessDescriptor;
        
        let selector = AccessSelector::entry_index(5, 10);
        let descriptor = selector.to_selective_access_descriptor().unwrap().unwrap();
        
        assert_eq!(descriptor.access_selector, 0);
        
        // Convert back
        let converted = AccessSelector::from_selective_access_descriptor(&descriptor).unwrap();
        match converted {
            AccessSelector::EntryIndex { start_index, count } => {
                assert_eq!(start_index, 5);
                assert_eq!(count, 10);
            }
            _ => panic!("Expected EntryIndex"),
        }
    }
}
