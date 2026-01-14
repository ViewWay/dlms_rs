//! Encrypted PDU types for DLMS/COSEM
//!
//! This module provides encrypted PDU types used in DLMS/COSEM when
//! security (encryption) is enabled.
//!
//! # Encryption Types
//!
//! There are two categories of encrypted PDUs:
//! - **Global encryption (glo-*)**: Uses the global cipher key
//! - **Dedicated encryption (ded-*)**: Uses a dedicated (per-association) cipher key
//!
//! # PDU Types
//!
//! For each encryption category, there are 17 PDU types corresponding
//! to the standard DLMS/COSEM application PDUs:
//!
//! | Type | Security Control Value | Description |
//! |------|----------------------|-------------|
//! | 0 | 0x00/0x80 | Reserved |
//! | 1 | 0x01/0x81 | Encrypted GetRequest |
//! | 2 | 0x02/0x82 | Encrypted GetResponse |
//! | 3 | 0x03/0x83 | Encrypted SetRequest |
//! | 4 | 0x04/0x84 | Encrypted SetResponse |
//! | 5 | 0x05/0x85 | Encrypted ActionRequest |
//! | 6 | 0x06/0x86 | Encrypted ActionResponse |
//! | 7 | 0x07/0x87 | Encrypted EventNotification |
//! | 8 | 0x08/0x88 | Encrypted AccessRequest |
//! | 9 | 0x09/0x89 | Encrypted AccessResponse |
//! | 10 | 0x0A/0x8A | Encrypted GetRequestWithList |
//! | 11 | 0x0B/0x8B | Encrypted GetResponseWithList |
//! | 12 | 0x0C/0x8C | Encrypted SetRequestWithList (if applicable) |
//! | 13 | 0x0D/0x8D | Encrypted SetResponseWithList (if applicable) |
//! | 14 | 0x0E/0x8E | Encrypted DataNotification |
//! | 15 | 0x0F/0x8F | Encrypted ExceptionResponse |
//! | 16 | 0x10/0x90 | Encrypted Reserved/Other |
//!
//! # Security Control Byte Format
//!
//! ```text
//! Bit 7: Encryption Key Selector
//!   0 = Global cipher key (glo-*)
//!   1 = Dedicated cipher key (ded-*)
//!
//! Bits 6-0: PDU Type
//!   0-16 = PDU type identifier
//! ```
//!
//! # Encoding Format
//!
//! Encrypted PDUs are encoded as:
//! ```text
//! Security Control (1 byte)
//! System Title (8 bytes)
//! Frame Counter (4 bytes)
//! Encrypted Data (variable)
//! Authentication Tag (12 bytes for GMAC)
//! ```

use dlms_core::{DlmsError, DlmsResult};
use crate::pdu::{
    GetRequest, GetResponse, SetRequest, SetResponse,
    ActionRequest, ActionResponse, EventNotification, AccessRequest, AccessResponse,
};

/// Security control byte for encrypted PDUs
///
/// The security control byte indicates:
/// - Bit 7: Encryption key selector (0=global, 1=dedicated)
/// - Bits 6-0: PDU type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecurityControl(u8);

impl SecurityControl {
    /// Create a new security control byte
    ///
    /// # Arguments
    /// * `key_type` - The key type (Global or Dedicated)
    /// * `pdu_type` - The PDU type (0-16)
    #[must_use]
    pub const fn new(key_type: KeyType, pdu_type: u8) -> Self {
        let base = match key_type {
            KeyType::Global => 0x00,
            KeyType::Dedicated => 0x80,
        };
        Self(base | (pdu_type & 0x7F))
    }

    /// Get the security control byte value
    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }

    /// Get the key type
    #[must_use]
    pub const fn key_type(self) -> KeyType {
        if self.0 & 0x80 == 0 {
            KeyType::Global
        } else {
            KeyType::Dedicated
        }
    }

    /// Get the PDU type
    #[must_use]
    pub const fn pdu_type(self) -> u8 {
        self.0 & 0x7F
    }

    /// Check if this is global encryption
    #[must_use]
    pub const fn is_global(self) -> bool {
        self.0 & 0x80 == 0
    }

    /// Check if this is dedicated encryption
    #[must_use]
    pub const fn is_dedicated(self) -> bool {
        self.0 & 0x80 != 0
    }
}

impl From<SecurityControl> for u8 {
    fn from(sc: SecurityControl) -> Self {
        sc.0
    }
}

impl From<u8> for SecurityControl {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

/// Encryption key type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    /// Global cipher key (shared by all associations)
    Global,
    /// Dedicated cipher key (specific to one association)
    Dedicated,
}

/// PDU type identifier for encrypted PDUs
///
/// This enum represents the different PDU types that can be encrypted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EncryptedPduType {
    /// Reserved (type 0)
    Reserved = 0,
    /// Encrypted GetRequest
    GetRequest = 1,
    /// Encrypted GetResponse
    GetResponse = 2,
    /// Encrypted SetRequest
    SetRequest = 3,
    /// Encrypted SetResponse
    SetResponse = 4,
    /// Encrypted ActionRequest
    ActionRequest = 5,
    /// Encrypted ActionResponse
    ActionResponse = 6,
    /// Encrypted EventNotification
    EventNotification = 7,
    /// Encrypted AccessRequest
    AccessRequest = 8,
    /// Encrypted AccessResponse
    AccessResponse = 9,
    /// Encrypted GetRequestWithList
    GetRequestWithList = 10,
    /// Encrypted GetResponseWithList
    GetResponseWithList = 11,
    /// Encrypted SetRequestWithList
    SetRequestWithList = 12,
    /// Encrypted SetResponseWithList
    SetResponseWithList = 13,
    /// Encrypted DataNotification
    DataNotification = 14,
    /// Encrypted ExceptionResponse
    ExceptionResponse = 15,
    /// Encrypted Other
    Other = 16,
}

impl EncryptedPduType {
    /// Create from u8 value
    ///
    /// # Arguments
    /// * `value` - The PDU type value (0-16)
    ///
    /// # Returns
    /// Returns `Some(EncryptedPduType)` if valid, `None` otherwise.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Reserved),
            1 => Some(Self::GetRequest),
            2 => Some(Self::GetResponse),
            3 => Some(Self::SetRequest),
            4 => Some(Self::SetResponse),
            5 => Some(Self::ActionRequest),
            6 => Some(Self::ActionResponse),
            7 => Some(Self::EventNotification),
            8 => Some(Self::AccessRequest),
            9 => Some(Self::AccessResponse),
            10 => Some(Self::GetRequestWithList),
            11 => Some(Self::GetResponseWithList),
            12 => Some(Self::SetRequestWithList),
            13 => Some(Self::SetResponseWithList),
            14 => Some(Self::DataNotification),
            15 => Some(Self::ExceptionResponse),
            16 => Some(Self::Other),
            _ => None,
        }
    }

    /// Get the u8 value
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Get the name of this PDU type
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Reserved => "Reserved",
            Self::GetRequest => "GetRequest",
            Self::GetResponse => "GetResponse",
            Self::SetRequest => "SetRequest",
            Self::SetResponse => "SetResponse",
            Self::ActionRequest => "ActionRequest",
            Self::ActionResponse => "ActionResponse",
            Self::EventNotification => "EventNotification",
            Self::AccessRequest => "AccessRequest",
            Self::AccessResponse => "AccessResponse",
            Self::GetRequestWithList => "GetRequestWithList",
            Self::GetResponseWithList => "GetResponseWithList",
            Self::SetRequestWithList => "SetRequestWithList",
            Self::SetResponseWithList => "SetResponseWithList",
            Self::DataNotification => "DataNotification",
            Self::ExceptionResponse => "ExceptionResponse",
            Self::Other => "Other",
        }
    }
}

/// Global encrypted PDU (glo-*)
///
/// Uses the global cipher key for encryption.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEncryptedPdu {
    /// Security control byte (0x00-0x10)
    security_control: SecurityControl,
    /// System title (8 bytes)
    system_title: [u8; 8],
    /// Frame counter (4 bytes)
    frame_counter: u32,
    /// Encrypted data
    encrypted_data: Vec<u8>,
    /// Authentication tag (12 bytes for GMAC)
    authentication_tag: [u8; 12],
}

impl GlobalEncryptedPdu {
    /// Create a new global encrypted PDU
    ///
    /// # Arguments
    /// * `pdu_type` - The PDU type (0-16)
    /// * `system_title` - System title (8 bytes)
    /// * `frame_counter` - Frame counter
    /// * `encrypted_data` - Encrypted data
    /// * `authentication_tag` - Authentication tag (12 bytes)
    pub fn new(
        pdu_type: EncryptedPduType,
        system_title: [u8; 8],
        frame_counter: u32,
        encrypted_data: Vec<u8>,
        authentication_tag: [u8; 12],
    ) -> Self {
        Self {
            security_control: SecurityControl::new(KeyType::Global, pdu_type.as_u8()),
            system_title,
            frame_counter,
            encrypted_data,
            authentication_tag,
        }
    }

    /// Get the PDU type
    #[must_use]
    pub fn pdu_type(&self) -> Option<EncryptedPduType> {
        EncryptedPduType::from_u8(self.security_control.pdu_type())
    }

    /// Get the security control byte
    #[must_use]
    pub const fn security_control(&self) -> SecurityControl {
        self.security_control
    }

    /// Get the system title
    #[must_use]
    pub const fn system_title(&self) -> &[u8; 8] {
        &self.system_title
    }

    /// Get the frame counter
    #[must_use]
    pub const fn frame_counter(&self) -> u32 {
        self.frame_counter
    }

    /// Get the encrypted data
    #[must_use]
    pub fn encrypted_data(&self) -> &[u8] {
        &self.encrypted_data
    }

    /// Get the authentication tag
    #[must_use]
    pub const fn authentication_tag(&self) -> &[u8; 12] {
        &self.authentication_tag
    }

    /// Encode to bytes
    ///
    /// Encoding format:
    /// - Security Control (1 byte)
    /// - System Title (8 bytes)
    /// - Frame Counter (4 bytes, big-endian)
    /// - Encrypted Data (variable)
    /// - Authentication Tag (12 bytes)
    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(25 + self.encrypted_data.len());

        result.push(self.security_control.value());
        result.extend_from_slice(&self.system_title);
        result.extend_from_slice(&self.frame_counter.to_be_bytes());
        result.extend_from_slice(&self.encrypted_data);
        result.extend_from_slice(&self.authentication_tag);

        result
    }

    /// Decode from bytes
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.len() < 25 {
            return Err(DlmsError::InvalidData(
                "Encrypted PDU too short".to_string(),
            ));
        }

        let security_control = SecurityControl::from(data[0]);

        if !security_control.is_global() {
            return Err(DlmsError::InvalidData(
                "Not a global encrypted PDU".to_string(),
            ));
        }

        let mut system_title = [0u8; 8];
        system_title.copy_from_slice(&data[1..9]);

        let frame_counter = u32::from_be_bytes([data[9], data[10], data[11], data[12]]);

        let encrypted_data_len = data.len() - 25;
        let encrypted_data = data[13..13 + encrypted_data_len].to_vec();

        let mut authentication_tag = [0u8; 12];
        authentication_tag.copy_from_slice(&data[data.len() - 12..]);

        Ok(Self {
            security_control,
            system_title,
            frame_counter,
            encrypted_data,
            authentication_tag,
        })
    }
}

/// Dedicated encrypted PDU (ded-*)
///
/// Uses a dedicated (per-association) cipher key for encryption.
#[derive(Debug, Clone, PartialEq)]
pub struct DedicatedEncryptedPdu {
    /// Security control byte (0x80-0x90)
    security_control: SecurityControl,
    /// System title (8 bytes)
    system_title: [u8; 8],
    /// Frame counter (4 bytes)
    frame_counter: u32,
    /// Encrypted data
    encrypted_data: Vec<u8>,
    /// Authentication tag (12 bytes for GMAC)
    authentication_tag: [u8; 12],
}

impl DedicatedEncryptedPdu {
    /// Create a new dedicated encrypted PDU
    ///
    /// # Arguments
    /// * `pdu_type` - The PDU type (0-16)
    /// * `system_title` - System title (8 bytes)
    /// * `frame_counter` - Frame counter
    /// * `encrypted_data` - Encrypted data
    /// * `authentication_tag` - Authentication tag (12 bytes)
    pub fn new(
        pdu_type: EncryptedPduType,
        system_title: [u8; 8],
        frame_counter: u32,
        encrypted_data: Vec<u8>,
        authentication_tag: [u8; 12],
    ) -> Self {
        Self {
            security_control: SecurityControl::new(KeyType::Dedicated, pdu_type.as_u8()),
            system_title,
            frame_counter,
            encrypted_data,
            authentication_tag,
        }
    }

    /// Get the PDU type
    #[must_use]
    pub fn pdu_type(&self) -> Option<EncryptedPduType> {
        EncryptedPduType::from_u8(self.security_control.pdu_type())
    }

    /// Get the security control byte
    #[must_use]
    pub const fn security_control(&self) -> SecurityControl {
        self.security_control
    }

    /// Get the system title
    #[must_use]
    pub const fn system_title(&self) -> &[u8; 8] {
        &self.system_title
    }

    /// Get the frame counter
    #[must_use]
    pub const fn frame_counter(&self) -> u32 {
        self.frame_counter
    }

    /// Get the encrypted data
    #[must_use]
    pub fn encrypted_data(&self) -> &[u8] {
        &self.encrypted_data
    }

    /// Get the authentication tag
    #[must_use]
    pub const fn authentication_tag(&self) -> &[u8; 12] {
        &self.authentication_tag
    }

    /// Encode to bytes
    ///
    /// Encoding format:
    /// - Security Control (1 byte)
    /// - System Title (8 bytes)
    /// - Frame Counter (4 bytes, big-endian)
    /// - Encrypted Data (variable)
    /// - Authentication Tag (12 bytes)
    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(25 + self.encrypted_data.len());

        result.push(self.security_control.value());
        result.extend_from_slice(&self.system_title);
        result.extend_from_slice(&self.frame_counter.to_be_bytes());
        result.extend_from_slice(&self.encrypted_data);
        result.extend_from_slice(&self.authentication_tag);

        result
    }

    /// Decode from bytes
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.len() < 25 {
            return Err(DlmsError::InvalidData(
                "Encrypted PDU too short".to_string(),
            ));
        }

        let security_control = SecurityControl::from(data[0]);

        if !security_control.is_dedicated() {
            return Err(DlmsError::InvalidData(
                "Not a dedicated encrypted PDU".to_string(),
            ));
        }

        let mut system_title = [0u8; 8];
        system_title.copy_from_slice(&data[1..9]);

        let frame_counter = u32::from_be_bytes([data[9], data[10], data[11], data[12]]);

        let encrypted_data_len = data.len() - 25;
        let encrypted_data = data[13..13 + encrypted_data_len].to_vec();

        let mut authentication_tag = [0u8; 12];
        authentication_tag.copy_from_slice(&data[data.len() - 12..]);

        Ok(Self {
            security_control,
            system_title,
            frame_counter,
            encrypted_data,
            authentication_tag,
        })
    }
}

/// Encrypted PDU enumeration
///
/// This enum represents either a global or dedicated encrypted PDU.
#[derive(Debug, Clone, PartialEq)]
pub enum EncryptedPdu {
    /// Global encrypted PDU (glo-*)
    Global(GlobalEncryptedPdu),
    /// Dedicated encrypted PDU (ded-*)
    Dedicated(DedicatedEncryptedPdu),
}

impl EncryptedPdu {
    /// Decode an encrypted PDU from bytes
    ///
    /// This method automatically detects whether the PDU is global or dedicated
    /// based on the security control byte.
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData(
                "Empty encrypted PDU".to_string(),
            ));
        }

        let security_control = SecurityControl::from(data[0]);

        if security_control.is_global() {
            Ok(Self::Global(GlobalEncryptedPdu::decode(data)?))
        } else {
            Ok(Self::Dedicated(DedicatedEncryptedPdu::decode(data)?))
        }
    }

    /// Encode the encrypted PDU to bytes
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::Global(pdu) => pdu.encode(),
            Self::Dedicated(pdu) => pdu.encode(),
        }
    }

    /// Get the security control byte
    #[must_use]
    pub fn security_control(&self) -> SecurityControl {
        match self {
            Self::Global(pdu) => pdu.security_control(),
            Self::Dedicated(pdu) => pdu.security_control(),
        }
    }

    /// Get the system title
    #[must_use]
    pub fn system_title(&self) -> &[u8; 8] {
        match self {
            Self::Global(pdu) => pdu.system_title(),
            Self::Dedicated(pdu) => pdu.system_title(),
        }
    }

    /// Get the frame counter
    #[must_use]
    pub fn frame_counter(&self) -> u32 {
        match self {
            Self::Global(pdu) => pdu.frame_counter(),
            Self::Dedicated(pdu) => pdu.frame_counter(),
        }
    }

    /// Check if this is a global encrypted PDU
    #[must_use]
    pub fn is_global(&self) -> bool {
        matches!(self, Self::Global(_))
    }

    /// Check if this is a dedicated encrypted PDU
    #[must_use]
    pub fn is_dedicated(&self) -> bool {
        matches!(self, Self::Dedicated(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_control_global() {
        let sc = SecurityControl::new(KeyType::Global, 1);
        assert_eq!(sc.value(), 0x01);
        assert!(sc.is_global());
        assert!(!sc.is_dedicated());
        assert_eq!(sc.pdu_type(), 1);
    }

    #[test]
    fn test_security_control_dedicated() {
        let sc = SecurityControl::new(KeyType::Dedicated, 5);
        assert_eq!(sc.value(), 0x85);
        assert!(!sc.is_global());
        assert!(sc.is_dedicated());
        assert_eq!(sc.pdu_type(), 5);
    }

    #[test]
    fn test_encrypted_pdu_type_from_u8() {
        assert_eq!(EncryptedPduType::from_u8(0), Some(EncryptedPduType::Reserved));
        assert_eq!(EncryptedPduType::from_u8(1), Some(EncryptedPduType::GetRequest));
        assert_eq!(EncryptedPduType::from_u8(16), Some(EncryptedPduType::Other));
        assert_eq!(EncryptedPduType::from_u8(17), None);
    }

    #[test]
    fn test_global_encrypted_pdu_encode_decode() {
        let pdu = GlobalEncryptedPdu::new(
            EncryptedPduType::GetRequest,
            [1, 2, 3, 4, 5, 6, 7, 8],
            0x12345678,
            vec![0xAB, 0xCD],
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        );

        let encoded = pdu.encode();
        assert_eq!(encoded.len(), 27); // 1 + 8 + 4 + 2 + 12
        assert_eq!(encoded[0], 0x01); // Security control

        let decoded = GlobalEncryptedPdu::decode(&encoded).unwrap();
        assert_eq!(decoded.system_title(), pdu.system_title());
        assert_eq!(decoded.frame_counter(), pdu.frame_counter());
        assert_eq!(decoded.encrypted_data(), pdu.encrypted_data());
    }

    #[test]
    fn test_dedicated_encrypted_pdu_encode_decode() {
        let pdu = DedicatedEncryptedPdu::new(
            EncryptedPduType::SetRequest,
            [1, 2, 3, 4, 5, 6, 7, 8],
            0x12345678,
            vec![0xAB, 0xCD],
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        );

        let encoded = pdu.encode();
        assert_eq!(encoded.len(), 27); // 1 + 8 + 4 + 2 + 12
        assert_eq!(encoded[0], 0x83); // Security control (dedicated + SetRequest)

        let decoded = DedicatedEncryptedPdu::decode(&encoded).unwrap();
        assert_eq!(decoded.system_title(), pdu.system_title());
        assert_eq!(decoded.frame_counter(), pdu.frame_counter());
        assert_eq!(decoded.encrypted_data(), pdu.encrypted_data());
    }

    #[test]
    fn test_encrypted_pdu_decode_auto_detect() {
        let global_pdu = GlobalEncryptedPdu::new(
            EncryptedPduType::GetRequest,
            [1, 2, 3, 4, 5, 6, 7, 8],
            0x12345678,
            vec![0xAB],
            [0; 12],
        );

        let dedicated_pdu = DedicatedEncryptedPdu::new(
            EncryptedPduType::GetRequest,
            [1, 2, 3, 4, 5, 6, 7, 8],
            0x12345678,
            vec![0xAB],
            [0; 12],
        );

        let global_encoded = global_pdu.encode();
        let dedicated_encoded = dedicated_pdu.encode();

        let global_decoded = EncryptedPdu::decode(&global_encoded).unwrap();
        assert!(global_decoded.is_global());

        let dedicated_decoded = EncryptedPdu::decode(&dedicated_encoded).unwrap();
        assert!(dedicated_decoded.is_dedicated());
    }
}
