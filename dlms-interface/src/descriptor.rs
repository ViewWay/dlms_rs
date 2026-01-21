//! COSEM object descriptors for association and interface classes
//!
//! This module provides types for describing COSEM objects, attributes, and methods
//! used in Association LN/SN interface classes.

use dlms_core::{DlmsError, DlmsResult, ObisCode};
use dlms_asn1::ber::decoder::BerDecoder;
use dlms_asn1::ber::encoder::BerEncoder;
use std::fmt;

/// COSEM object descriptor
///
/// Describes a COSEM object with its class ID, version, and logical name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CosemObjectDescriptor {
    /// Class ID of the object
    pub class_id: u16,
    /// OBIS code (logical name)
    pub logical_name: ObisCode,
    /// Version of the interface class
    pub version: u8,
}

impl CosemObjectDescriptor {
    /// Create a new COSEM object descriptor
    pub fn new(class_id: u16, logical_name: ObisCode, version: u8) -> Self {
        Self {
            class_id,
            logical_name,
            version,
        }
    }

    /// Encode the descriptor as a structure for DLMS
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();

        // Build the structure content manually
        let mut content = Vec::new();

        // Encode class_id (integer)
        let mut class_encoder = BerEncoder::new();
        class_encoder.encode_integer(self.class_id as i64)?;
        content.extend_from_slice(class_encoder.as_bytes());

        // Encode logical_name (octet string)
        let mut name_encoder = BerEncoder::new();
        name_encoder.encode_octet_string(self.logical_name.as_bytes())?;
        content.extend_from_slice(name_encoder.as_bytes());

        // Encode version (integer)
        let mut version_encoder = BerEncoder::new();
        version_encoder.encode_integer(self.version as i64)?;
        content.extend_from_slice(version_encoder.as_bytes());

        // Wrap in SEQUENCE
        encoder.encode_sequence(&content)?;
        Ok(encoder.into_bytes())
    }

    /// Decode a COSEM object descriptor from BER-encoded bytes
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = BerDecoder::new(data);

        // Decode SEQUENCE
        let sequence_content = decoder.decode_sequence()?;

        let mut inner_decoder = BerDecoder::new(sequence_content);

        // Decode class_id
        let class_id = inner_decoder.decode_integer()? as u16;

        // Decode logical_name
        let logical_name_bytes = inner_decoder.decode_octet_string()?;
        let logical_name = ObisCode::from_bytes(&logical_name_bytes)?;

        // Decode version
        let version = inner_decoder.decode_integer()? as u8;

        Ok(Self {
            class_id,
            logical_name,
            version,
        })
    }
}

impl fmt::Display for CosemObjectDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CosemObject(class_id={}, obis={}, version={})",
            self.class_id, self.logical_name, self.version
        )
    }
}

/// Attribute access mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    /// No access
    NoAccess = 0,
    /// Read only
    ReadOnly = 1,
    /// Write only
    WriteOnly = 2,
    /// Read and write
    ReadWrite = 3,
    /// Authenticated read only
    AuthReadOnly = 4,
    /// Authenticated write only
    AuthWriteOnly = 5,
    /// Authenticated read and write
    AuthReadWrite = 6,
}

impl AccessMode {
    /// Create from mode value
    pub fn from_value(value: u8) -> DlmsResult<Self> {
        match value {
            0 => Ok(AccessMode::NoAccess),
            1 => Ok(AccessMode::ReadOnly),
            2 => Ok(AccessMode::WriteOnly),
            3 => Ok(AccessMode::ReadWrite),
            4 => Ok(AccessMode::AuthReadOnly),
            5 => Ok(AccessMode::AuthWriteOnly),
            6 => Ok(AccessMode::AuthReadWrite),
            _ => Err(DlmsError::InvalidData(format!("Invalid access mode: {}", value))),
        }
    }

    /// Get the mode value
    pub fn value(&self) -> u8 {
        *self as u8
    }

    /// Check if read is allowed
    pub fn can_read(&self) -> bool {
        matches!(
            self,
            AccessMode::ReadOnly
                | AccessMode::ReadWrite
                | AccessMode::AuthReadOnly
                | AccessMode::AuthReadWrite
        )
    }

    /// Check if write is allowed
    pub fn can_write(&self) -> bool {
        matches!(
            self,
            AccessMode::WriteOnly
                | AccessMode::ReadWrite
                | AccessMode::AuthWriteOnly
                | AccessMode::AuthReadWrite
        )
    }

    /// Check if authentication is required
    pub fn requires_auth(&self) -> bool {
        matches!(
            self,
            AccessMode::AuthReadOnly
                | AccessMode::AuthWriteOnly
                | AccessMode::AuthReadWrite
        )
    }
}

/// Attribute descriptor
///
/// Describes a single attribute of a COSEM object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeDescriptor {
    /// Class ID of the object
    pub class_id: u16,
    /// OBIS code (logical name)
    pub logical_name: ObisCode,
    /// Attribute ID
    pub attribute_id: u8,
}

impl AttributeDescriptor {
    /// Create a new attribute descriptor
    pub fn new(class_id: u16, logical_name: ObisCode, attribute_id: u8) -> Self {
        Self {
            class_id,
            logical_name,
            attribute_id,
        }
    }

    /// Encode the attribute descriptor
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        // Structure: [class_id, logical_name, attribute_id]
        let mut result = Vec::new();

        // Encode class_id (2 bytes, big-endian)
        result.push((self.class_id >> 8) as u8);
        result.push((self.class_id & 0xFF) as u8);

        // Encode logical_name (6 bytes)
        result.extend_from_slice(self.logical_name.as_bytes());

        // Encode attribute_id (1 byte)
        result.push(self.attribute_id);

        Ok(result)
    }

    /// Decode an attribute descriptor from bytes
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.len() < 9 {
            return Err(DlmsError::InvalidData(
                "Insufficient data for AttributeDescriptor".to_string(),
            ));
        }

        let class_id = u16::from_be_bytes([data[0], data[1]]);
        let logical_name = ObisCode::from_bytes(&data[2..8])?;
        let attribute_id = data[8];

        Ok(Self {
            class_id,
            logical_name,
            attribute_id,
        })
    }
}

/// Method descriptor
///
/// Describes a single method of a COSEM object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodDescriptor {
    /// Class ID of the object
    pub class_id: u16,
    /// OBIS code (logical name)
    pub logical_name: ObisCode,
    /// Method ID
    pub method_id: u8,
}

impl MethodDescriptor {
    /// Create a new method descriptor
    pub fn new(class_id: u16, logical_name: ObisCode, method_id: u8) -> Self {
        Self {
            class_id,
            logical_name,
            method_id,
        }
    }

    /// Encode the method descriptor
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        // Structure: [class_id, logical_name, method_id]
        let mut result = Vec::new();

        // Encode class_id (2 bytes, big-endian)
        result.push((self.class_id >> 8) as u8);
        result.push((self.class_id & 0xFF) as u8);

        // Encode logical_name (6 bytes)
        result.extend_from_slice(self.logical_name.as_bytes());

        // Encode method_id (1 byte)
        result.push(self.method_id);

        Ok(result)
    }

    /// Decode a method descriptor from bytes
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.len() < 9 {
            return Err(DlmsError::InvalidData(
                "Insufficient data for MethodDescriptor".to_string(),
            ));
        }

        let class_id = u16::from_be_bytes([data[0], data[1]]);
        let logical_name = ObisCode::from_bytes(&data[2..8])?;
        let method_id = data[8];

        Ok(Self {
            class_id,
            logical_name,
            method_id,
        })
    }
}

/// Access right entry
///
/// Defines access rights for a specific user or group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessRight {
    /// User ID (0 = public, 1-16 = specific users)
    pub user_id: u8,
    /// Access rights for attributes (vector of (attribute_id, access_mode))
    pub attribute_rights: Vec<(u8, AccessMode)>,
    /// Access rights for methods (vector of (method_id, access_mode))
    pub method_rights: Vec<(u8, AccessMode)>,
}

impl AccessRight {
    /// Create a new access right entry
    pub fn new(user_id: u8) -> Self {
        Self {
            user_id,
            attribute_rights: Vec::new(),
            method_rights: Vec::new(),
        }
    }

    /// Add an attribute access right
    pub fn add_attribute_right(&mut self, attribute_id: u8, mode: AccessMode) {
        self.attribute_rights.push((attribute_id, mode));
    }

    /// Add a method access right
    pub fn add_method_right(&mut self, method_id: u8, mode: AccessMode) {
        self.method_rights.push((method_id, mode));
    }

    /// Check if attribute access is allowed
    pub fn can_access_attribute(&self, attribute_id: u8, mode: AccessMode) -> bool {
        for (attr_id, access_mode) in &self.attribute_rights {
            if *attr_id == attribute_id {
                match mode {
                    AccessMode::ReadOnly => return access_mode.can_read(),
                    AccessMode::WriteOnly => return access_mode.can_write(),
                    AccessMode::ReadWrite => return access_mode.can_read() && access_mode.can_write(),
                    _ => return *access_mode == mode,
                }
            }
        }
        false
    }

    /// Check if method access is allowed
    pub fn can_access_method(&self, method_id: u8) -> bool {
        for (meth_id, _) in &self.method_rights {
            if *meth_id == method_id {
                return true;
            }
        }
        false
    }
}

/// User information
///
/// Contains user authentication and identification information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserInfo {
    /// User ID (1-16)
    pub user_id: u8,
    /// User name (optional)
    pub user_name: Option<String>,
    /// Authentication key (for HLS authentication)
    pub auth_key: Option<Vec<u8>>,
    /// Password (for LOW level authentication)
    pub password: Option<Vec<u8>>,
}

impl UserInfo {
    /// Create a new user info
    pub fn new(user_id: u8) -> Self {
        Self {
            user_id,
            user_name: None,
            auth_key: None,
            password: None,
        }
    }

    /// Set the user name
    pub fn with_name(mut self, name: String) -> Self {
        self.user_name = Some(name);
        self
    }

    /// Set the authentication key
    pub fn with_auth_key(mut self, key: Vec<u8>) -> Self {
        self.auth_key = Some(key);
        self
    }

    /// Set the password
    pub fn with_password(mut self, password: Vec<u8>) -> Self {
        self.password = Some(password);
        self
    }

    /// Validate user ID
    pub fn validate_user_id(&self) -> DlmsResult<()> {
        if self.user_id == 0 || self.user_id > 16 {
            return Err(DlmsError::InvalidData(
                "User ID must be between 1 and 16".to_string(),
            ));
        }
        Ok(())
    }
}

/// Capture object definition for Profile Generic
///
/// Defines which objects and attributes to capture in a profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureObjectDefinition {
    /// Class ID of the object to capture
    pub class_id: u16,
    /// OBIS code (logical name)
    pub logical_name: ObisCode,
    /// Attribute index to capture (0-based)
    pub attribute_index: u8,
    /// Data index (for array/structure attributes)
    pub data_index: Option<u8>,
}

impl CaptureObjectDefinition {
    /// Create a new capture object definition
    pub fn new(class_id: u16, logical_name: ObisCode, attribute_index: u8) -> Self {
        Self {
            class_id,
            logical_name,
            attribute_index,
            data_index: None,
        }
    }

    /// Set the data index
    pub fn with_data_index(mut self, index: u8) -> Self {
        self.data_index = Some(index);
        self
    }

    /// Encode the capture object definition
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut result = Vec::new();

        // Encode class_id (2 bytes, big-endian)
        result.push((self.class_id >> 8) as u8);
        result.push((self.class_id & 0xFF) as u8);

        // Encode logical_name (6 bytes)
        result.extend_from_slice(self.logical_name.as_bytes());

        // Encode attribute_index (1 byte)
        result.push(self.attribute_index);

        // Encode data_index (1 byte, 0xFF if not set)
        result.push(self.data_index.unwrap_or(0xFF));

        Ok(result)
    }

    /// Decode a capture object definition from bytes
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.len() < 10 {
            return Err(DlmsError::InvalidData(
                "Insufficient data for CaptureObjectDefinition".to_string(),
            ));
        }

        let class_id = u16::from_be_bytes([data[0], data[1]]);
        let logical_name = ObisCode::from_bytes(&data[2..8])?;
        let attribute_index = data[8];
        let data_index = if data[9] == 0xFF {
            None
        } else {
            Some(data[9])
        };

        Ok(Self {
            class_id,
            logical_name,
            attribute_index,
            data_index,
        })
    }
}

/// Profile entry for Profile Generic buffer
///
/// Represents a single entry in the profile buffer.
pub type ProfileEntry = Vec<dlms_core::DataObject>;

/// Sort method for Profile Generic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMethod {
    /// Not sorted
    NotSorted = 0,
    /// First in first out
    Fifo = 1,
    /// Sorted by capture object
    ByCaptureObject = 2,
}

impl SortMethod {
    /// Create from value
    pub fn from_value(value: u8) -> DlmsResult<Self> {
        match value {
            0 => Ok(SortMethod::NotSorted),
            1 => Ok(SortMethod::Fifo),
            2 => Ok(SortMethod::ByCaptureObject),
            _ => Err(DlmsError::InvalidData(format!("Invalid sort method: {}", value))),
        }
    }

    /// Get the value
    pub fn value(&self) -> u8 {
        *self as u8
    }
}

/// Helper trait for ObisCode conversion
pub trait ObisCodeExt: Sized {
    fn from_bytes(bytes: &[u8]) -> DlmsResult<Self>;
}

impl ObisCodeExt for ObisCode {
    fn from_bytes(bytes: &[u8]) -> DlmsResult<Self> {
        if bytes.len() < 6 {
            return Err(DlmsError::InvalidData(
                "Insufficient bytes for ObisCode".to_string(),
            ));
        }
        Ok(ObisCode::new(bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosem_object_descriptor() {
        let obis = ObisCode::new(0, 0, 1, 0, 0, 255);
        let desc = CosemObjectDescriptor::new(1, obis, 0);

        assert_eq!(desc.class_id, 1);
        assert_eq!(desc.version, 0);
    }

    #[test]
    fn test_access_mode() {
        let mode = AccessMode::ReadWrite;
        assert!(mode.can_read());
        assert!(mode.can_write());
        assert!(!mode.requires_auth());
    }

    #[test]
    fn test_access_mode_auth() {
        let mode = AccessMode::AuthReadWrite;
        assert!(mode.can_read());
        assert!(mode.can_write());
        assert!(mode.requires_auth());
    }

    #[test]
    fn test_attribute_descriptor_encode() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let desc = AttributeDescriptor::new(3, obis, 2);
        let encoded = desc.encode().unwrap();

        assert_eq!(encoded.len(), 9);
        assert_eq!(encoded[0], 0); // class_id high byte
        assert_eq!(encoded[1], 3); // class_id low byte
        assert_eq!(encoded[8], 2); // attribute_id
    }

    #[test]
    fn test_user_info() {
        let user = UserInfo::new(1)
            .with_name("Admin".to_string())
            .with_password(vec![0x31, 0x32, 0x33, 0x34]);

        assert_eq!(user.user_id, 1);
        assert_eq!(user.user_name, Some("Admin".to_string()));
        assert!(user.validate_user_id().is_ok());
    }

    #[test]
    fn test_user_info_invalid_id() {
        let user = UserInfo::new(0);
        assert!(user.validate_user_id().is_err());
    }

    #[test]
    fn test_capture_object_definition() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let capture = CaptureObjectDefinition::new(3, obis, 2);

        assert_eq!(capture.class_id, 3);
        assert_eq!(capture.attribute_index, 2);
        assert!(capture.data_index.is_none());
    }

    #[test]
    fn test_sort_method() {
        let sort = SortMethod::from_value(1).unwrap();
        assert_eq!(sort, SortMethod::Fifo);
        assert_eq!(sort.value(), 1);
    }
}
