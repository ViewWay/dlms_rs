//! ISO-ACSE types and structures
//!
//! This module provides type definitions for ISO-ACSE (Association Control Service Element)
//! structures used in DLMS/COSEM protocol.
//!
//! # ISO-ACSE Overview
//!
//! ISO-ACSE is defined in ISO/IEC 8650 and provides services for establishing and
//! releasing associations between application entities. In DLMS/COSEM, ACSE is used
//! to establish the application association before COSEM data can be exchanged.
//!
//! # Main PDU Types
//!
//! - **AARQ**: Association Request (Application tag 0)
//! - **AARE**: Association Response (Application tag 1)
//! - **RLRQ**: Release Request (Application tag 2)
//! - **RLRE**: Release Response (Application tag 3)
//!
//! # Encoding Format
//!
//! All ISO-ACSE PDUs are encoded using BER (Basic Encoding Rules) as specified in
//! ITU-T X.690. This is different from A-XDR used in the COSEM application layer.
//!
//! # Why BER for ACSE?
//!
//! ISO-ACSE is part of the OSI (Open Systems Interconnection) stack, which uses
//! BER encoding. DLMS/COSEM uses ACSE for association establishment, so it must
//! follow the ISO-ACSE encoding rules.

use crate::error::{DlmsError, DlmsResult};
use crate::ber::{BerEncoder, BerDecoder};

/// Association Result
///
/// Indicates the result of an association request.
///
/// # Values
/// - 0: accepted
/// - 1: rejected-permanent
/// - 2: rejected-transient
///
/// # Why This Enum?
/// Using an enum provides type safety and prevents invalid result values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssociateResult {
    /// Association accepted
    Accepted = 0,
    /// Association rejected (permanent)
    RejectedPermanent = 1,
    /// Association rejected (transient)
    RejectedTransient = 2,
}

impl AssociateResult {
    /// Create from integer value
    pub fn from_value(value: i64) -> DlmsResult<Self> {
        match value {
            0 => Ok(AssociateResult::Accepted),
            1 => Ok(AssociateResult::RejectedPermanent),
            2 => Ok(AssociateResult::RejectedTransient),
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid AssociateResult value: {}",
                value
            ))),
        }
    }

    /// Get integer value
    pub fn value(self) -> i64 {
        self as i64
    }

    /// Encode to BER format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();
        encoder.encode_integer(self.value())?;
        Ok(encoder.into_bytes())
    }

    /// Decode from BER format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = BerDecoder::new(data);
        let value = decoder.decode_integer()?;
        Self::from_value(value)
    }
}

/// Release Request Reason
///
/// Reason for releasing an association.
///
/// # Values
/// - 0: normal
/// - 1: urgent
/// - 2: user-defined
///
/// # TODO
/// - [ ] 实现完整的 ReleaseRequestReason 枚举（需要查看标准定义）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReleaseRequestReason {
    /// Normal release
    Normal = 0,
    /// Urgent release
    Urgent = 1,
    /// User-defined reason
    UserDefined = 2,
}

impl ReleaseRequestReason {
    /// Create from integer value
    pub fn from_value(value: i64) -> DlmsResult<Self> {
        match value {
            0 => Ok(ReleaseRequestReason::Normal),
            1 => Ok(ReleaseRequestReason::Urgent),
            2 => Ok(ReleaseRequestReason::UserDefined),
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid ReleaseRequestReason value: {}",
                value
            ))),
        }
    }

    /// Get integer value
    pub fn value(self) -> i64 {
        self as i64
    }

    /// Encode to BER format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();
        encoder.encode_integer(self.value())?;
        Ok(encoder.into_bytes())
    }

    /// Decode from BER format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = BerDecoder::new(data);
        let value = decoder.decode_integer()?;
        Self::from_value(value)
    }
}

/// Release Response Reason
///
/// Reason for releasing an association (response).
///
/// # Values
/// - 0: normal
/// - 1: not finished
/// - 2: user-defined
///
/// # TODO
/// - [ ] 实现完整的 ReleaseResponseReason 枚举（需要查看标准定义）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReleaseResponseReason {
    /// Normal release
    Normal = 0,
    /// Not finished
    NotFinished = 1,
    /// User-defined reason
    UserDefined = 2,
}

impl ReleaseResponseReason {
    /// Create from integer value
    pub fn from_value(value: i64) -> DlmsResult<Self> {
        match value {
            0 => Ok(ReleaseResponseReason::Normal),
            1 => Ok(ReleaseResponseReason::NotFinished),
            2 => Ok(ReleaseResponseReason::UserDefined),
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid ReleaseResponseReason value: {}",
                value
            ))),
        }
    }

    /// Get integer value
    pub fn value(self) -> i64 {
        self as i64
    }

    /// Encode to BER format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();
        encoder.encode_integer(self.value())?;
        Ok(encoder.into_bytes())
    }

    /// Decode from BER format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = BerDecoder::new(data);
        let value = decoder.decode_integer()?;
        Self::from_value(value)
    }
}

/// Association Information
///
/// User information carried in ACSE PDUs. In DLMS/COSEM, this typically contains
/// the InitiateRequest/Response PDU encoded in A-XDR format.
///
/// # Encoding
/// AssociationInformation is encoded as an OCTET STRING containing the user data.
///
/// # Why This Type?
/// This provides a type-safe wrapper for user information, ensuring proper
/// encoding/decoding according to ISO-ACSE specifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociationInformation {
    /// User information bytes (typically A-XDR encoded InitiateRequest/Response)
    value: Vec<u8>,
}

impl AssociationInformation {
    /// Create new AssociationInformation
    pub fn new(value: Vec<u8>) -> Self {
        Self { value }
    }

    /// Get the user information bytes
    pub fn value(&self) -> &[u8] {
        &self.value
    }

    /// Create AssociationInformation from InitiateRequest PDU bytes
    ///
    /// This is a convenience method for creating user information containing
    /// an A-XDR encoded InitiateRequest for use in AARQ APDU.
    ///
    /// # Arguments
    /// * `initiate_request_bytes` - A-XDR encoded InitiateRequest PDU
    #[must_use]
    pub fn from_initiate_request(initiate_request_bytes: Vec<u8>) -> Self {
        Self { value: initiate_request_bytes }
    }

    /// Create AssociationInformation from InitiateResponse PDU bytes
    ///
    /// This is a convenience method for creating user information containing
    /// an A-XDR encoded InitiateResponse for use in AARE APDU.
    ///
    /// # Arguments
    /// * `initiate_response_bytes` - A-XDR encoded InitiateResponse PDU
    #[must_use]
    pub fn from_initiate_response(initiate_response_bytes: Vec<u8>) -> Self {
        Self { value: initiate_response_bytes }
    }

    /// Encode to BER format (with tag)
    ///
    /// # Encoding Format
    /// AssociationInformation is encoded as:
    /// - Tag: Universal, Primitive, tag 4 (OCTET STRING)
    /// - Length: Number of bytes
    /// - Value: User information bytes
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();
        encoder.encode_octet_string(&self.value)?;
        Ok(encoder.into_bytes())
    }

    /// Decode from BER format (with tag)
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = BerDecoder::new(data);
        let value = decoder.decode_octet_string()?;
        Ok(Self::new(value))
    }

    /// Get the user information as bytes for decoding as InitiateRequest/Response
    ///
    /// Returns the raw bytes that can be passed to InitiateRequest::decode() or
    /// InitiateResponse::decode().
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.value
    }
}

impl From<Vec<u8>> for AssociationInformation {
    fn from(value: Vec<u8>) -> Self {
        Self { value }
    }
}

impl AsRef<[u8]> for AssociationInformation {
    fn as_ref(&self) -> &[u8] {
        &self.value
    }
}

/// AP Title (Application Process Title)
///
/// Identifies an application process. In DLMS/COSEM, this is typically used
/// to identify the system title for encryption.
///
/// # Encoding Forms
/// AP Title can be encoded in two forms:
/// - **Form 1**: OBJECT IDENTIFIER - standardized AP titles
/// - **Form 2**: OCTET STRING - system title for DLMS/COSEM
///
/// # Example
/// ```rust
/// // Form 2 - System title (most common in DLMS/COSEM)
/// let title = APTitle::form_2(vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
///
/// // Form 1 - Object identifier
/// let title = APTitle::form_1(vec![0, 4, 0, 127, 0, 0, 15, 0, 0, 1]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum APTitle {
    /// Form 1: Object Identifier (OID)
    /// Used for standardized application process titles
    Form1(Vec<u32>),

    /// Form 2: Octet String
    /// Used for system title in DLMS/COSEM (typically 8 bytes)
    Form2(Vec<u8>),
}

impl APTitle {
    /// Create AP Title Form 1 from OID
    ///
    /// # Arguments
    /// * `oid` - Object identifier components
    pub fn form_1(oid: Vec<u32>) -> Self {
        Self::Form1(oid)
    }

    /// Create AP Title Form 2 from system title
    ///
    /// # Arguments
    /// * `system_title` - System title bytes (typically 8 bytes for DLMS/COSEM)
    pub fn form_2(system_title: Vec<u8>) -> Self {
        Self::Form2(system_title)
    }

    /// Create AP Title Form 2 from system title (backward compatible alias)
    ///
    /// # Arguments
    /// * `system_title` - System title bytes
    #[deprecated(since = "0.2.0", note = "Use form_2 instead")]
    pub fn new(system_title: Vec<u8>) -> Self {
        Self::Form2(system_title)
    }

    /// Check if this is Form 1 (OID)
    pub fn is_form_1(&self) -> bool {
        matches!(self, Self::Form1(_))
    }

    /// Check if this is Form 2 (OCTET STRING)
    pub fn is_form_2(&self) -> bool {
        matches!(self, Self::Form2(_))
    }

    /// Get the OID if this is Form 1
    pub fn as_oid(&self) -> Option<&[u32]> {
        match self {
            Self::Form1(oid) => Some(oid),
            _ => None,
        }
    }

    /// Get the system title if this is Form 2
    pub fn as_system_title(&self) -> Option<&[u8]> {
        match self {
            Self::Form2(title) => Some(title),
            _ => None,
        }
    }

    /// Get system title (backward compatible, only works for Form 2)
    ///
    /// Returns None if this is Form 1
    pub fn system_title(&self) -> Option<&[u8]> {
        self.as_system_title()
    }

    /// Encode to BER format
    ///
    /// # Encoding Format
    /// - Form 1: OBJECT IDENTIFIER (tag 0x06)
    /// - Form 2: OCTET STRING (tag 0x04)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();
        match self {
            Self::Form1(oid) => {
                encoder.encode_object_identifier(oid)?;
            }
            Self::Form2(title) => {
                encoder.encode_octet_string(title)?;
            }
        }
        Ok(encoder.into_bytes())
    }

    /// Decode from BER format
    ///
    /// Automatically detects the form based on the tag
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData(
                "Empty APTitle".to_string(),
            ));
        }

        let tag = data[0];
        let mut decoder = BerDecoder::new(data);

        match tag {
            0x06 => {
                // OBJECT IDENTIFIER - Form 1
                let oid = decoder.decode_object_identifier()?;
                Ok(Self::Form1(oid))
            }
            0x04 => {
                // OCTET STRING - Form 2
                let title = decoder.decode_octet_string()?;
                Ok(Self::Form2(title))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid APTitle tag: 0x{:02X} (expected 0x04 or 0x06)",
                tag
            ))),
        }
    }
}

impl From<Vec<u8>> for APTitle {
    fn from(value: Vec<u8>) -> Self {
        Self::Form2(value)
    }
}

impl From<Vec<u32>> for APTitle {
    fn from(value: Vec<u32>) -> Self {
        Self::Form1(value)
    }
}

/// AE Qualifier (Application Entity Qualifier)
///
/// Qualifies an application entity within an application process.
///
/// # Encoding Forms
/// AE Qualifier can be encoded in two forms:
/// - **Form 1**: INTEGER - numeric qualifier
/// - **Form 2**: OCTET STRING - binary qualifier
///
/// # Example
/// ```rust
/// // Form 2 - Octet string (most common in DLMS/COSEM)
/// let qualifier = AEQualifier::form_2(vec![0x01, 0x00]);
///
/// // Form 1 - Integer
/// let qualifier = AEQualifier::form_1(1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AEQualifier {
    /// Form 1: Integer
    /// Numeric qualifier value
    Form1(i64),

    /// Form 2: Octet String
    /// Binary qualifier value
    Form2(Vec<u8>),
}

impl AEQualifier {
    /// Create AE Qualifier Form 1 from integer
    ///
    /// # Arguments
    /// * `value` - Integer qualifier value
    pub fn form_1(value: i64) -> Self {
        Self::Form1(value)
    }

    /// Create AE Qualifier Form 2 from octet string
    ///
    /// # Arguments
    /// * `value` - Binary qualifier value
    pub fn form_2(value: Vec<u8>) -> Self {
        Self::Form2(value)
    }

    /// Create AE Qualifier Form 2 (backward compatible alias)
    ///
    /// # Arguments
    /// * `value` - Binary qualifier value
    #[deprecated(since = "0.2.0", note = "Use form_2 instead")]
    pub fn new(value: Vec<u8>) -> Self {
        Self::Form2(value)
    }

    /// Check if this is Form 1 (INTEGER)
    pub fn is_form_1(&self) -> bool {
        matches!(self, Self::Form1(_))
    }

    /// Check if this is Form 2 (OCTET STRING)
    pub fn is_form_2(&self) -> bool {
        matches!(self, Self::Form2(_))
    }

    /// Get the integer value if this is Form 1
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Form1(value) => Some(*value),
            _ => None,
        }
    }

    /// Get the octet string value if this is Form 2
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Form2(value) => Some(value),
            _ => None,
        }
    }

    /// Get the qualifier value (backward compatible)
    ///
    /// Returns None if this is Form 1
    pub fn value(&self) -> Option<&[u8]> {
        self.as_bytes()
    }

    /// Encode to BER format
    ///
    /// # Encoding Format
    /// - Form 1: INTEGER (tag 0x02)
    /// - Form 2: OCTET STRING (tag 0x04)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();
        match self {
            Self::Form1(value) => {
                encoder.encode_integer(*value)?;
            }
            Self::Form2(value) => {
                encoder.encode_octet_string(value)?;
            }
        }
        Ok(encoder.into_bytes())
    }

    /// Decode from BER format
    ///
    /// Automatically detects the form based on the tag
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData(
                "Empty AEQualifier".to_string(),
            ));
        }

        let tag = data[0];
        let mut decoder = BerDecoder::new(data);

        match tag {
            0x02 => {
                // INTEGER - Form 1
                let value = decoder.decode_integer()?;
                Ok(Self::Form1(value))
            }
            0x04 => {
                // OCTET STRING - Form 2
                let value = decoder.decode_octet_string()?;
                Ok(Self::Form2(value))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid AEQualifier tag: 0x{:02X} (expected 0x02 or 0x04)",
                tag
            ))),
        }
    }
}

impl From<Vec<u8>> for AEQualifier {
    fn from(value: Vec<u8>) -> Self {
        Self::Form2(value)
    }
}

impl From<i64> for AEQualifier {
    fn from(value: i64) -> Self {
        Self::Form1(value)
    }
}

impl From<i32> for AEQualifier {
    fn from(value: i32) -> Self {
        Self::Form1(value as i64)
    }
}

impl From<u16> for AEQualifier {
    fn from(value: u16) -> Self {
        Self::Form1(value as i64)
    }
}

impl From<u8> for AEQualifier {
    fn from(value: u8) -> Self {
        Self::Form1(value as i64)
    }
}

/// AP Invocation Identifier
///
/// Identifies a specific invocation of an application process.
///
/// # Encoding
/// AP Invocation Identifier is encoded as an INTEGER.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct APInvocationIdentifier {
    value: i64,
}

impl APInvocationIdentifier {
    /// Create new AP Invocation Identifier
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    /// Get identifier value
    pub fn value(&self) -> i64 {
        self.value
    }

    /// Encode to BER format (with tag)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();
        encoder.encode_integer(self.value)?;
        Ok(encoder.into_bytes())
    }

    /// Decode from BER format (with tag)
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = BerDecoder::new(data);
        let value = decoder.decode_integer()?;
        Ok(Self::new(value))
    }
}

/// AE Invocation Identifier
///
/// Identifies a specific invocation of an application entity.
///
/// # Encoding
/// AE Invocation Identifier is encoded as an INTEGER.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AEInvocationIdentifier {
    value: i64,
}

impl AEInvocationIdentifier {
    /// Create new AE Invocation Identifier
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    /// Get identifier value
    pub fn value(&self) -> i64 {
        self.value
    }

    /// Encode to BER format (with tag)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();
        encoder.encode_integer(self.value)?;
        Ok(encoder.into_bytes())
    }

    /// Decode from BER format (with tag)
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = BerDecoder::new(data);
        let value = decoder.decode_integer()?;
        Ok(Self::new(value))
    }
}

/// ACSE Requirements
///
/// Bit string indicating ACSE requirements supported by the sender.
///
/// # Bit Definitions (from DLMS Green Book)
/// The requirements are encoded as a bit string where each bit indicates:
/// - Bit 0 (LSB): Association not required
/// - Bit 1: Application context name list
/// - Bit 2: Authentication
/// - Bit 3-7: Reserved
///
/// # Example
/// ```rust
/// let req = ACSERequirements::new()
///     .with_authentication(true)
///     .build();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ACSERequirements {
    /// Requirements bit string (typically 1 byte)
    bits: Vec<u8>,
    num_bits: usize,
}

/// ACSE Requirements bit position constants
///
/// These constants define the bit positions for various ACSE requirements.
pub mod acse_requirements {
    /// Bit 0: Association not required
    pub const ASSOCIATION_NOT_REQUIRED: u8 = 0x01;
    /// Bit 1: Application context name list supported
    pub const APPLICATION_CONTEXT_NAME_LIST: u8 = 0x02;
    /// Bit 2: Authentication required
    pub const AUTHENTICATION: u8 = 0x04;
    /// Bit 3-7: Reserved for future use
    pub const RESERVED_MASK: u8 = 0xF8;
}

impl ACSERequirements {
    /// Create new ACSE Requirements
    pub fn new(bits: Vec<u8>, num_bits: usize) -> Self {
        Self { bits, num_bits }
    }

    /// Create empty ACSE Requirements (all bits cleared)
    ///
    /// This is the starting point for building requirements using the builder pattern.
    ///
    /// # Example
    /// ```rust
    /// let req = ACSERequirements::empty()
    ///     .with_authentication(true)
    ///     .build();
    /// ```
    pub fn empty() -> ACSERequirementsBuilder {
        ACSERequirementsBuilder::new()
    }

    /// Create ACSE Requirements with default settings
    ///
    /// Default: Authentication required (bit 2 set)
    pub fn default_auth() -> Self {
        Self::from_bits(1, acse_requirements::AUTHENTICATION)
    }

    /// Create ACSE Requirements from a single byte value
    ///
    /// # Arguments
    /// * `num_bits` - Number of significant bits (typically 8)
    /// * `bits` - Byte containing requirement bits
    pub fn from_bits(num_bits: usize, bits: u8) -> Self {
        Self {
            bits: vec![bits],
            num_bits,
        }
    }

    /// Get requirements bits
    pub fn bits(&self) -> &[u8] {
        &self.bits
    }

    /// Get number of bits
    pub fn num_bits(&self) -> usize {
        self.num_bits
    }

    /// Check if a specific bit is set
    ///
    /// # Arguments
    /// * `bit` - Bit position (0-7 for single byte)
    pub fn is_set(&self, bit: u8) -> bool {
        if bit >= 8 || self.bits.is_empty() {
            return false;
        }
        (self.bits[0] & (1 << bit)) != 0
    }

    /// Check if authentication is required
    pub fn requires_authentication(&self) -> bool {
        self.is_set(2)
    }

    /// Check if application context name list is supported
    pub fn supports_context_name_list(&self) -> bool {
        self.is_set(1)
    }

    /// Check if association is not required
    pub fn association_not_required(&self) -> bool {
        self.is_set(0)
    }

    /// Encode to BER format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();
        // Calculate unused bits in last byte
        let unused_bits = if self.num_bits % 8 == 0 {
            0
        } else {
            8 - (self.num_bits % 8)
        };
        encoder.encode_bit_string(&self.bits, self.num_bits, unused_bits as u8)?;
        Ok(encoder.into_bytes())
    }

    /// Decode from BER format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = BerDecoder::new(data);
        let (bits, num_bits, _unused_bits) = decoder.decode_bit_string()?;
        Ok(Self::new(bits, num_bits))
    }
}

/// Builder for creating ACSE Requirements
///
/// Provides a fluent interface for constructing ACSE Requirements.
///
/// # Example
/// ```rust
/// let req = ACSERequirements::empty()
///     .with_authentication(true)
///     .with_context_name_list(true)
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct ACSERequirementsBuilder {
    bits: u8,
    num_bits: usize,
}

impl ACSERequirementsBuilder {
    /// Create a new builder with all bits cleared
    pub fn new() -> Self {
        Self {
            bits: 0,
            num_bits: 8,
        }
    }

    /// Set the association not required bit
    pub fn with_association_not_required(mut self, value: bool) -> Self {
        if value {
            self.bits |= acse_requirements::ASSOCIATION_NOT_REQUIRED;
        } else {
            self.bits &= !acse_requirements::ASSOCIATION_NOT_REQUIRED;
        }
        self
    }

    /// Set the application context name list bit
    pub fn with_context_name_list(mut self, value: bool) -> Self {
        if value {
            self.bits |= acse_requirements::APPLICATION_CONTEXT_NAME_LIST;
        } else {
            self.bits &= !acse_requirements::APPLICATION_CONTEXT_NAME_LIST;
        }
        self
    }

    /// Set the authentication bit
    pub fn with_authentication(mut self, value: bool) -> Self {
        if value {
            self.bits |= acse_requirements::AUTHENTICATION;
        } else {
            self.bits &= !acse_requirements::AUTHENTICATION;
        }
        self
    }

    /// Set a custom bit value
    ///
    /// # Arguments
    /// * `bit` - Bit position (0-7)
    /// * `value` - True to set the bit, false to clear it
    pub fn with_bit(mut self, bit: u8, value: bool) -> Self {
        if bit < 8 {
            if value {
                self.bits |= 1 << bit;
            } else {
                self.bits &= !(1 << bit);
            }
        }
        self
    }

    /// Set the raw bits value
    pub fn with_bits(mut self, bits: u8) -> Self {
        self.bits = bits;
        self
    }

    /// Set the number of bits
    pub fn with_num_bits(mut self, num_bits: usize) -> Self {
        self.num_bits = num_bits;
        self
    }

    /// Build the ACSE Requirements
    pub fn build(self) -> ACSERequirements {
        ACSERequirements {
            bits: vec![self.bits],
            num_bits: self.num_bits,
        }
    }
}

/// Mechanism Name
///
/// Identifies an authentication mechanism.
///
/// # Encoding
/// Mechanism Name is encoded as an OBJECT IDENTIFIER.
///
/// # Standard Authentication Mechanism OIDs
/// The following OID constants are defined for common authentication mechanisms:
/// - `NONE`: No authentication (0.4.0.127.0.0.15.0.0.0)
/// - `LOW`: Low-level authentication (0.4.0.127.0.0.15.0.0.1)
/// - `HIGH_MD5`: High-level authentication with MD5 (0.4.0.127.0.0.15.0.0.2)
/// - `HIGH_SHA1`: High-level authentication with SHA-1 (0.4.0.127.0.0.15.0.0.3)
/// - `HIGH_GMAC`: High-level authentication with GMAC/HLS5 (0.4.0.127.0.0.15.0.0.4)
/// - `HIGH_SHA256`: High-level authentication with SHA-256 (0.4.0.127.0.0.15.0.0.5)
///
/// # Example
/// ```rust
/// let mechanism = MechanismName::low_level();
/// assert_eq!(mechanism.oid(), &[0, 4, 0, 127, 0, 0, 15, 0, 0, 1]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MechanismName {
    /// Object identifier components
    oid: Vec<u32>,
}

/// Authentication mechanism OID constants
///
/// These constants define the standard OIDs for DLMS/COSEM authentication mechanisms
/// as specified in the DLMS Green Book.
///
/// # OID Format
/// The joint-iso-itu-t(2) country(16) countries(860) denmark(0) organization(1)...
/// format is commonly represented as: 0.4.0.127.0.0.15.x.y.z
/// where x.y.z identifies the specific mechanism.
pub mod mechanism_oid {
    /// Base OID for DLMS/COSEM authentication mechanisms
    /// 0.4.0.127.0.0.15
    pub const BASE: &[u32] = &[0, 4, 0, 127, 0, 0, 15];

    /// No authentication (0.4.0.127.0.0.15.0.0.0)
    pub const NONE: &[u32] = &[0, 4, 0, 127, 0, 0, 15, 0, 0, 0];

    /// Low-level authentication (0.4.0.127.0.0.15.0.0.1)
    /// Uses a simple password mechanism
    pub const LOW: &[u32] = &[0, 4, 0, 127, 0, 0, 15, 0, 0, 1];

    /// High-level authentication with MD5 (0.4.0.127.0.0.15.0.0.2)
    /// Uses MD5 for challenge-response
    pub const HIGH_MD5: &[u32] = &[0, 4, 0, 127, 0, 0, 15, 0, 0, 2];

    /// High-level authentication with SHA-1 (0.4.0.127.0.0.15.0.0.3)
    /// Uses SHA-1 for challenge-response
    pub const HIGH_SHA1: &[u32] = &[0, 4, 0, 127, 0, 0, 15, 0, 0, 3];

    /// High-level authentication with GMAC/HLS5 (0.4.0.127.0.0.15.0.0.4)
    /// Uses AES-GMAC for challenge-response (HLS5)
    pub const HIGH_GMAC: &[u32] = &[0, 4, 0, 127, 0, 0, 15, 0, 0, 4];

    /// High-level authentication with SHA-256 (0.4.0.127.0.0.15.0.0.5)
    /// Uses SHA-256 for challenge-response
    pub const HIGH_SHA256: &[u32] = &[0, 4, 0, 127, 0, 0, 15, 0, 0, 5];

    /// ECDSA authentication (0.4.0.127.0.0.15.0.0.6)
    /// Uses ECDSA for signature-based authentication
    pub const ECDSA: &[u32] = &[0, 4, 0, 127, 0, 0, 15, 0, 0, 6];

    /// LSM (Low Security Message) authentication (0.4.0.127.0.0.15.0.0.7)
    /// Low-level security message authentication
    pub const LSM: &[u32] = &[0, 4, 0, 127, 0, 0, 15, 0, 0, 7];

    /// Ephemeral key authentication (0.4.0.127.0.0.15.0.0.8)
    /// Uses ephemeral keys for authentication
    pub const EPHEMERAL_KEY: &[u32] = &[0, 4, 0, 127, 0, 0, 15, 0, 0, 8];

    /// SCRAM (Salted Challenge Response Authentication Mechanism)
    /// (0.4.0.127.0.0.15.0.0.9)
    pub const SCRAM: &[u32] = &[0, 4, 0, 127, 0, 0, 15, 0, 0, 9];
}

impl MechanismName {
    /// Create new Mechanism Name from OID
    pub fn new(oid: Vec<u32>) -> Self {
        Self { oid }
    }

    /// Get OID components
    pub fn oid(&self) -> &[u32] {
        &self.oid
    }

    /// Create mechanism name for no authentication
    pub fn none() -> Self {
        Self::new(mechanism_oid::NONE.to_vec())
    }

    /// Create mechanism name for low-level authentication
    pub fn low_level() -> Self {
        Self::new(mechanism_oid::LOW.to_vec())
    }

    /// Create mechanism name for high-level MD5 authentication
    pub fn high_md5() -> Self {
        Self::new(mechanism_oid::HIGH_MD5.to_vec())
    }

    /// Create mechanism name for high-level SHA-1 authentication
    pub fn high_sha1() -> Self {
        Self::new(mechanism_oid::HIGH_SHA1.to_vec())
    }

    /// Create mechanism name for high-level GMAC/HLS5 authentication
    pub fn high_gmac() -> Self {
        Self::new(mechanism_oid::HIGH_GMAC.to_vec())
    }

    /// Create mechanism name for high-level SHA-256 authentication
    pub fn high_sha256() -> Self {
        Self::new(mechanism_oid::HIGH_SHA256.to_vec())
    }

    /// Create mechanism name for ECDSA authentication
    pub fn ecdsa() -> Self {
        Self::new(mechanism_oid::ECDSA.to_vec())
    }

    /// Create mechanism name for LSM authentication
    pub fn lsm() -> Self {
        Self::new(mechanism_oid::LSM.to_vec())
    }

    /// Create mechanism name for ephemeral key authentication
    pub fn ephemeral_key() -> Self {
        Self::new(mechanism_oid::EPHEMERAL_KEY.to_vec())
    }

    /// Create mechanism name for SCRAM authentication
    pub fn scram() -> Self {
        Self::new(mechanism_oid::SCRAM.to_vec())
    }

    /// Check if this is no authentication
    pub fn is_none(&self) -> bool {
        self.oid() == mechanism_oid::NONE
    }

    /// Check if this is low-level authentication
    pub fn is_low_level(&self) -> bool {
        self.oid() == mechanism_oid::LOW
    }

    /// Check if this is high-level authentication (any variant)
    pub fn is_high_level(&self) -> bool {
        self.oid().len() > 9 && self.oid()[0..9] == mechanism_oid::BASE[..9]
    }

    /// Get the authentication level name for debugging
    pub fn level_name(&self) -> &'static str {
        if self.is_none() {
            "none"
        } else if self.is_low_level() {
            "low"
        } else if self.oid() == mechanism_oid::HIGH_MD5 {
            "high_md5"
        } else if self.oid() == mechanism_oid::HIGH_SHA1 {
            "high_sha1"
        } else if self.oid() == mechanism_oid::HIGH_GMAC {
            "high_gmac"
        } else if self.oid() == mechanism_oid::HIGH_SHA256 {
            "high_sha256"
        } else if self.oid() == mechanism_oid::ECDSA {
            "ecdsa"
        } else if self.oid() == mechanism_oid::LSM {
            "lsm"
        } else {
            "unknown"
        }
    }

    /// Encode to BER format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();
        encoder.encode_object_identifier(&self.oid)?;
        Ok(encoder.into_bytes())
    }

    /// Decode from BER format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = BerDecoder::new(data);
        let oid = decoder.decode_object_identifier()?;
        Ok(Self::new(oid))
    }
}

/// Authentication Value
///
/// Authentication value (password, challenge response, etc.) used in ACSE association.
///
/// # Encoding
/// Authentication Value is a CHOICE type that can be encoded in different forms.
/// For DLMS/COSEM, we typically use OCTET STRING (tag [9]) for passwords and
/// challenge responses.
///
/// # CHOICE Definition (from DLMS Green Book)
/// ```asn1
/// AuthenticationValue ::= CHOICE
/// {
///     -- simple data types
///     null-data [0],
///     boolean [3],
///     bit-string [4],
///     double-long [5],
///     double-long-unsigned [6],
///     octet-string [9],
///     visible-string [10],
///     utf8-string [12],
///     bcd [13],
///     integer [15],
///     long [16],
///     unsigned [17],
///     long-unsigned [18],
///     long64 [20],
///     long64-unsigned [21],
///     enum [22],
///     float32 [23],
///     float64 [24],
///     date-time [25],
///     date [26],
///     time [27],
///     delta-integer [28],
///     delta-long [29],
///     delta-double-long [30],
///     delta-unsigned [31],
///     delta-long-unsigned [32],
///     delta-double-long-unsigned [33],
///     -- complex data types
///     array [1],
///     structure [2],
///     compact-array [19]
/// }
/// ```
///
/// # Example
/// ```rust
/// // Create an octet-string authentication value (most common)
/// let auth = AuthenticationValue::octet_string(vec![0x01, 0x02, 0x03]);
///
/// // Create an integer authentication value
/// let auth = AuthenticationValue::integer(12345);
///
/// // Check the type
/// match auth {
///     AuthenticationValue::OctetString(data) => println!("Password: {:?}", data),
///     AuthenticationValue::Integer(n) => println!("PIN: {}", n),
///     _ => println!("Other auth type"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum AuthenticationValue {
    /// Null data (tag 0)
    NullData,

    /// Array (tag 1) - complex data type
    Array(Vec<Vec<u8>>),

    /// Structure (tag 2) - complex data type
    Structure(Vec<Vec<u8>>),

    /// Boolean (tag 3)
    Boolean(bool),

    /// Bit string (tag 4)
    BitString {
        /// The bit string data
        data: Vec<u8>,
        /// Number of unused bits in the last byte
        unused_bits: u8,
    },

    /// Double long (tag 5) - i64
    DoubleLong(i64),

    /// Double long unsigned (tag 6) - u64
    DoubleLongUnsigned(u64),

    /// Octet string (tag 9) - most commonly used for passwords
    OctetString(Vec<u8>),

    /// Visible string (tag 10) - ASCII string
    VisibleString(String),

    /// UTF-8 string (tag 12)
    Utf8String(String),

    /// BCD (tag 13) - Binary Coded Decimal
    Bcd(Vec<u8>),

    /// Integer (tag 15) - i8
    Integer(i8),

    /// Long (tag 16) - i16
    Long(i16),

    /// Unsigned (tag 17) - u8
    Unsigned(u8),

    /// Long unsigned (tag 18) - u16
    LongUnsigned(u16),

    /// Compact array (tag 19) - complex data type
    CompactArray(Vec<u8>),

    /// Long64 (tag 20) - i32
    Long64(i32),

    /// Long64 unsigned (tag 21) - u32
    Long64Unsigned(u32),

    /// Enum (tag 22) - u8
    Enum(u8),

    /// Float32 (tag 23) - f32
    Float32(f32),

    /// Float64 (tag 24) - f64
    Float64(f64),

    /// Date time (tag 25) - octet string (12 bytes)
    DateTime(Vec<u8>),

    /// Date (tag 26) - octet string (5 bytes)
    Date(Vec<u8>),

    /// Time (tag 27) - octet string (4 bytes)
    Time(Vec<u8>),

    /// Delta integer (tag 28) - i8 with delta encoding
    DeltaInteger(i8),

    /// Delta long (tag 29) - i16 with delta encoding
    DeltaLong(i16),

    /// Delta double long (tag 30) - i64 with delta encoding
    DeltaDoubleLong(i64),

    /// Delta unsigned (tag 31) - u8 with delta encoding
    DeltaUnsigned(u8),

    /// Delta long unsigned (tag 32) - u16 with delta encoding
    DeltaLongUnsigned(u16),

    /// Delta double long unsigned (tag 33) - u64 with delta encoding
    DeltaDoubleLongUnsigned(u64),
}

/// Authentication value tag constants
pub mod auth_value_tag {
    pub const NULL_DATA: u8 = 0;
    pub const ARRAY: u8 = 1;
    pub const STRUCTURE: u8 = 2;
    pub const BOOLEAN: u8 = 3;
    pub const BIT_STRING: u8 = 4;
    pub const DOUBLE_LONG: u8 = 5;
    pub const DOUBLE_LONG_UNSIGNED: u8 = 6;
    pub const OCTET_STRING: u8 = 9;
    pub const VISIBLE_STRING: u8 = 10;
    pub const UTF8_STRING: u8 = 12;
    pub const BCD: u8 = 13;
    pub const INTEGER: u8 = 15;
    pub const LONG: u8 = 16;
    pub const UNSIGNED: u8 = 17;
    pub const LONG_UNSIGNED: u8 = 18;
    pub const COMPACT_ARRAY: u8 = 19;
    pub const LONG64: u8 = 20;
    pub const LONG64_UNSIGNED: u8 = 21;
    pub const ENUM: u8 = 22;
    pub const FLOAT32: u8 = 23;
    pub const FLOAT64: u8 = 24;
    pub const DATE_TIME: u8 = 25;
    pub const DATE: u8 = 26;
    pub const TIME: u8 = 27;
    pub const DELTA_INTEGER: u8 = 28;
    pub const DELTA_LONG: u8 = 29;
    pub const DELTA_DOUBLE_LONG: u8 = 30;
    pub const DELTA_UNSIGNED: u8 = 31;
    pub const DELTA_LONG_UNSIGNED: u8 = 32;
    pub const DELTA_DOUBLE_LONG_UNSIGNED: u8 = 33;
}

impl AuthenticationValue {
    /// Create a null data authentication value
    pub fn null() -> Self {
        Self::NullData
    }

    /// Create an octet string authentication value (most common for passwords)
    ///
    /// # Arguments
    /// * `data` - The password or challenge response bytes
    pub fn octet_string(data: Vec<u8>) -> Self {
        Self::OctetString(data)
    }

    /// Create an integer authentication value (for PIN codes)
    ///
    /// # Arguments
    /// * `value` - The integer value
    pub fn integer(value: i8) -> Self {
        Self::Integer(value)
    }

    /// Create a long authentication value
    ///
    /// # Arguments
    /// * `value` - The long value
    pub fn long(value: i16) -> Self {
        Self::Long(value)
    }

    /// Create an unsigned authentication value
    ///
    /// # Arguments
    /// * `value` - The unsigned value
    pub fn unsigned(value: u8) -> Self {
        Self::Unsigned(value)
    }

    /// Create a long unsigned authentication value
    ///
    /// # Arguments
    /// * `value` - The long unsigned value
    pub fn long_unsigned(value: u16) -> Self {
        Self::LongUnsigned(value)
    }

    /// Create a double long authentication value
    ///
    /// # Arguments
    /// * `value` - The double long value
    pub fn double_long(value: i64) -> Self {
        Self::DoubleLong(value)
    }

    /// Create a double long unsigned authentication value
    ///
    /// # Arguments
    /// * `value` - The double long unsigned value
    pub fn double_long_unsigned(value: u64) -> Self {
        Self::DoubleLongUnsigned(value)
    }

    /// Create a visible string authentication value (ASCII)
    ///
    /// # Arguments
    /// * `value` - The ASCII string
    pub fn visible_string(value: String) -> Self {
        Self::VisibleString(value)
    }

    /// Create a UTF-8 string authentication value
    ///
    /// # Arguments
    /// * `value` - The UTF-8 string
    pub fn utf8_string(value: String) -> Self {
        Self::Utf8String(value)
    }

    /// Create a boolean authentication value
    ///
    /// # Arguments
    /// * `value` - The boolean value
    pub fn boolean(value: bool) -> Self {
        Self::Boolean(value)
    }

    /// Create a bit string authentication value
    ///
    /// # Arguments
    /// * `data` - The bit string data
    /// * `unused_bits` - Number of unused bits in the last byte
    pub fn bit_string(data: Vec<u8>, unused_bits: u8) -> Self {
        Self::BitString { data, unused_bits }
    }

    /// Create a BCD authentication value
    ///
    /// # Arguments
    /// * `data` - The BCD encoded data
    pub fn bcd(data: Vec<u8>) -> Self {
        Self::Bcd(data)
    }

    /// Get the tag value for this variant
    pub fn tag(&self) -> u8 {
        match self {
            Self::NullData => auth_value_tag::NULL_DATA,
            Self::Array(_) => auth_value_tag::ARRAY,
            Self::Structure(_) => auth_value_tag::STRUCTURE,
            Self::Boolean(_) => auth_value_tag::BOOLEAN,
            Self::BitString { .. } => auth_value_tag::BIT_STRING,
            Self::DoubleLong(_) => auth_value_tag::DOUBLE_LONG,
            Self::DoubleLongUnsigned(_) => auth_value_tag::DOUBLE_LONG_UNSIGNED,
            Self::OctetString(_) => auth_value_tag::OCTET_STRING,
            Self::VisibleString(_) => auth_value_tag::VISIBLE_STRING,
            Self::Utf8String(_) => auth_value_tag::UTF8_STRING,
            Self::Bcd(_) => auth_value_tag::BCD,
            Self::Integer(_) => auth_value_tag::INTEGER,
            Self::Long(_) => auth_value_tag::LONG,
            Self::Unsigned(_) => auth_value_tag::UNSIGNED,
            Self::LongUnsigned(_) => auth_value_tag::LONG_UNSIGNED,
            Self::CompactArray(_) => auth_value_tag::COMPACT_ARRAY,
            Self::Long64(_) => auth_value_tag::LONG64,
            Self::Long64Unsigned(_) => auth_value_tag::LONG64_UNSIGNED,
            Self::Enum(_) => auth_value_tag::ENUM,
            Self::Float32(_) => auth_value_tag::FLOAT32,
            Self::Float64(_) => auth_value_tag::FLOAT64,
            Self::DateTime(_) => auth_value_tag::DATE_TIME,
            Self::Date(_) => auth_value_tag::DATE,
            Self::Time(_) => auth_value_tag::TIME,
            Self::DeltaInteger(_) => auth_value_tag::DELTA_INTEGER,
            Self::DeltaLong(_) => auth_value_tag::DELTA_LONG,
            Self::DeltaDoubleLong(_) => auth_value_tag::DELTA_DOUBLE_LONG,
            Self::DeltaUnsigned(_) => auth_value_tag::DELTA_UNSIGNED,
            Self::DeltaLongUnsigned(_) => auth_value_tag::DELTA_LONG_UNSIGNED,
            Self::DeltaDoubleLongUnsigned(_) => auth_value_tag::DELTA_DOUBLE_LONG_UNSIGNED,
        }
    }

    /// Check if this is an octet string value
    pub fn is_octet_string(&self) -> bool {
        matches!(self, Self::OctetString(_))
    }

    /// Check if this is a null value
    pub fn is_null(&self) -> bool {
        matches!(self, Self::NullData)
    }

    /// Get the octet string data if this is an octet string variant
    pub fn as_octet_string(&self) -> Option<&[u8]> {
        match self {
            Self::OctetString(data) => Some(data),
            _ => None,
        }
    }

    /// Encode to BER format with context-specific tag
    ///
    /// The encoding format is:
    /// - Tag: Context-specific, primitive, tag number (0x80 | tag)
    /// - Length: Number of bytes in the value
    /// - Value: The encoded value
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut result = Vec::new();

        // Context-specific tag: 0b10000000 | tag_number
        // Bits 7-6: class (10 = context-specific)
        // Bit 5: primitive (0) or constructed (1)
        // Bits 4-0: tag number
        let is_constructed = matches!(self, Self::Array(_) | Self::Structure(_) | Self::CompactArray(_));
        let tag_byte = 0x80 | self.tag() | (if is_constructed { 0x20 } else { 0 });
        result.push(tag_byte);

        // Encode the value
        let value_bytes = match self {
            Self::NullData => Vec::new(),
            Self::OctetString(data) | Self::Bcd(data) | Self::DateTime(data) | Self::Date(data) | Self::Time(data) => {
                let mut enc = BerEncoder::new();
                enc.encode_octet_string(data)?;
                enc.into_bytes()
            }
            Self::Boolean(value) => {
                let mut enc = BerEncoder::new();
                enc.encode_boolean(*value)?;
                enc.into_bytes()
            }
            Self::Integer(value) => {
                let mut enc = BerEncoder::new();
                enc.encode_integer(*value as i64)?;
                enc.into_bytes()
            }
            Self::Long(value) => {
                let mut enc = BerEncoder::new();
                enc.encode_integer(*value as i64)?;
                enc.into_bytes()
            }
            Self::Unsigned(value) => {
                let mut enc = BerEncoder::new();
                enc.encode_integer(*value as i64)?;
                enc.into_bytes()
            }
            Self::LongUnsigned(value) => {
                let mut enc = BerEncoder::new();
                enc.encode_integer(*value as i64)?;
                enc.into_bytes()
            }
            Self::Long64(value) => {
                let mut enc = BerEncoder::new();
                enc.encode_integer(*value as i64)?;
                enc.into_bytes()
            }
            Self::Long64Unsigned(value) => {
                let mut enc = BerEncoder::new();
                // Encode as u64
                enc.encode_unsigned_integer(*value as u64)?;
                enc.into_bytes()
            }
            Self::DoubleLong(value) => {
                let mut enc = BerEncoder::new();
                enc.encode_integer(*value)?;
                enc.into_bytes()
            }
            Self::DoubleLongUnsigned(value) => {
                let mut enc = BerEncoder::new();
                enc.encode_unsigned_integer(*value)?;
                enc.into_bytes()
            }
            Self::VisibleString(value) | Self::Utf8String(value) => {
                let mut enc = BerEncoder::new();
                enc.encode_octet_string(value.as_bytes())?;
                enc.into_bytes()
            }
            Self::BitString { data, unused_bits } => {
                let mut enc = BerEncoder::new();
                enc.encode_bit_string(data, data.len() * 8, *unused_bits)?;
                enc.into_bytes()
            }
            Self::Enum(value) => {
                let mut enc = BerEncoder::new();
                enc.encode_integer(*value as i64)?;
                enc.into_bytes()
            }
            Self::Float32(value) => {
                value.to_be_bytes().to_vec()
            }
            Self::Float64(value) => {
                value.to_be_bytes().to_vec()
            }
            Self::DeltaInteger(value) => {
                let mut enc = BerEncoder::new();
                enc.encode_integer(*value as i64)?;
                enc.into_bytes()
            }
            Self::DeltaLong(value) => {
                let mut enc = BerEncoder::new();
                enc.encode_integer(*value as i64)?;
                enc.into_bytes()
            }
            Self::DeltaDoubleLong(value) => {
                let mut enc = BerEncoder::new();
                enc.encode_integer(*value)?;
                enc.into_bytes()
            }
            Self::DeltaUnsigned(value) => {
                let mut enc = BerEncoder::new();
                enc.encode_integer(*value as i64)?;
                enc.into_bytes()
            }
            Self::DeltaLongUnsigned(value) => {
                let mut enc = BerEncoder::new();
                enc.encode_integer(*value as i64)?;
                enc.into_bytes()
            }
            Self::DeltaDoubleLongUnsigned(value) => {
                let mut enc = BerEncoder::new();
                enc.encode_unsigned_integer(*value)?;
                enc.into_bytes()
            }
            Self::Array(_) | Self::Structure(_) | Self::CompactArray(_) => {
                // For complex types, encode as raw octet string for now
                // Full implementation would require proper encoding of each type
                return Err(DlmsError::InvalidData(
                    "Complex type encoding not yet implemented".to_string(),
                ));
            }
        };

        // Length
        if value_bytes.len() < 128 {
            result.push(value_bytes.len() as u8);
        } else {
            // Long form
            let length = value_bytes.len();
            let length_bytes = length.to_be_bytes();
            let num_length_bytes = length_bytes.iter().rev()
                .skip_while(|&&b| b == 0)
                .count();

            result.push(0x80 | num_length_bytes as u8);
            result.extend_from_slice(&length_bytes[(8 - num_length_bytes)..]);
        }

        // Value
        result.extend_from_slice(&value_bytes);

        Ok(result)
    }

    /// Decode from BER format with context-specific tag
    ///
    /// Expects a context-specific tag followed by the encoded value
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData(
                "Empty AuthenticationValue".to_string(),
            ));
        }

        // First byte is the context-specific tag
        let tag_byte = data[0];
        let tag_class = (tag_byte >> 6) & 0x03;
        let tag_number = tag_byte & 0x1F;

        if tag_class != 2 {
            return Err(DlmsError::InvalidData(format!(
                "Expected context-specific tag, got class {}",
                tag_class
            )));
        }

        // Get length
        if data.len() < 2 {
            return Err(DlmsError::InvalidData(
                "Incomplete AuthenticationValue (missing length)".to_string(),
            ));
        }

        let length_byte = data[1];
        let (length_bytes, pos) = if length_byte < 128 {
            (2usize, 2)
        } else {
            let num_length_bytes = (length_byte & 0x7F) as usize;
            if data.len() < 2 + num_length_bytes {
                return Err(DlmsError::InvalidData(
                    "Incomplete long-form length".to_string(),
                ));
            }

            let mut length = 0usize;
            for i in 0..num_length_bytes {
                length = (length << 8) | (data[2 + i] as usize);
            }
            (2 + num_length_bytes, 2 + num_length_bytes)
        };

        // Extract value data
        let _value_length = if length_bytes == 2 {
            length_byte as usize
        } else {
            // Need to parse long form
            let num_length_bytes = (length_byte & 0x7F) as usize;
            let mut length = 0usize;
            for i in 0..num_length_bytes {
                length = (length << 8) | (data[2 + i] as usize);
            }
            length
        };

        if data.len() < pos + length_bytes {
            return Err(DlmsError::InvalidData(
                "Incomplete AuthenticationValue (truncated)".to_string(),
            ));
        }

        let value_data = &data[pos..pos + length_bytes];

        // Decode based on tag number
        let mut decoder = BerDecoder::new(value_data);

        Ok(match tag_number {
            auth_value_tag::NULL_DATA => Self::NullData,
            auth_value_tag::OCTET_STRING => {
                let data = decoder.decode_octet_string()?;
                Self::OctetString(data)
            }
            auth_value_tag::BOOLEAN => {
                let value = decoder.decode_boolean()?;
                Self::Boolean(value)
            }
            auth_value_tag::INTEGER => {
                let value = decoder.decode_integer()?;
                Self::Integer(value as i8)
            }
            auth_value_tag::LONG => {
                let value = decoder.decode_integer()?;
                Self::Long(value as i16)
            }
            auth_value_tag::UNSIGNED => {
                let value = decoder.decode_integer()?;
                Self::Unsigned(value as u8)
            }
            auth_value_tag::LONG_UNSIGNED => {
                let value = decoder.decode_integer()?;
                Self::LongUnsigned(value as u16)
            }
            auth_value_tag::LONG64 => {
                let value = decoder.decode_integer()?;
                Self::Long64(value as i32)
            }
            auth_value_tag::LONG64_UNSIGNED => {
                let value = decoder.decode_unsigned_integer()?;
                Self::Long64Unsigned(value as u32)
            }
            auth_value_tag::DOUBLE_LONG => {
                let value = decoder.decode_integer()?;
                Self::DoubleLong(value)
            }
            auth_value_tag::DOUBLE_LONG_UNSIGNED => {
                let value = decoder.decode_unsigned_integer()?;
                Self::DoubleLongUnsigned(value)
            }
            auth_value_tag::VISIBLE_STRING => {
                let data = decoder.decode_octet_string()?;
                Self::VisibleString(String::from_utf8_lossy(&data).to_string())
            }
            auth_value_tag::UTF8_STRING => {
                let data = decoder.decode_octet_string()?;
                Self::Utf8String(String::from_utf8_lossy(&data).to_string())
            }
            auth_value_tag::ENUM => {
                let value = decoder.decode_integer()?;
                Self::Enum(value as u8)
            }
            auth_value_tag::BCD | auth_value_tag::DATE_TIME | auth_value_tag::DATE | auth_value_tag::TIME => {
                let data = decoder.decode_octet_string()?;
                match tag_number {
                    auth_value_tag::BCD => Self::Bcd(data),
                    auth_value_tag::DATE_TIME => Self::DateTime(data),
                    auth_value_tag::DATE => Self::Date(data),
                    auth_value_tag::TIME => Self::Time(data),
                    _ => unreachable!(),
                }
            }
            auth_value_tag::BIT_STRING => {
                let (data, _num_bits, unused_bits) = decoder.decode_bit_string()?;
                Self::BitString { data, unused_bits }
            }
            auth_value_tag::DELTA_INTEGER => {
                let value = decoder.decode_integer()?;
                Self::DeltaInteger(value as i8)
            }
            auth_value_tag::DELTA_LONG => {
                let value = decoder.decode_integer()?;
                Self::DeltaLong(value as i16)
            }
            auth_value_tag::DELTA_DOUBLE_LONG => {
                let value = decoder.decode_integer()?;
                Self::DeltaDoubleLong(value)
            }
            auth_value_tag::DELTA_UNSIGNED => {
                let value = decoder.decode_integer()?;
                Self::DeltaUnsigned(value as u8)
            }
            auth_value_tag::DELTA_LONG_UNSIGNED => {
                let value = decoder.decode_integer()?;
                Self::DeltaLongUnsigned(value as u16)
            }
            auth_value_tag::DELTA_DOUBLE_LONG_UNSIGNED => {
                let value = decoder.decode_unsigned_integer()?;
                Self::DeltaDoubleLongUnsigned(value)
            }
            _ => {
                return Err(DlmsError::InvalidData(format!(
                    "Unsupported AuthenticationValue tag: {}",
                    tag_number
                )));
            }
        })
    }
}

/// Legacy Authentication Value struct for backward compatibility
///
/// This provides a simple wrapper around the OCTET STRING variant for
/// backward compatibility with existing code.
///
/// # Deprecated
/// Use `AuthenticationValue` enum directly instead.
#[derive(Debug, Clone, PartialEq, Eq)]
#[deprecated(since = "0.2.0", note = "Use AuthenticationValue enum instead")]
pub struct AuthenticationValueLegacy {
    /// Authentication value bytes
    value: Vec<u8>,
}

impl AuthenticationValueLegacy {
    /// Create new Authentication Value
    pub fn new(value: Vec<u8>) -> Self {
        Self { value }
    }

    /// Convert to the new AuthenticationValue enum
    pub fn to_auth_value(&self) -> AuthenticationValue {
        AuthenticationValue::OctetString(self.value.clone())
    }

    /// Get authentication value bytes
    pub fn value(&self) -> &[u8] {
        &self.value
    }
}

/// Implementation Data
///
/// Implementation-specific information.
///
/// # Encoding
/// Implementation Data is encoded as an OCTET STRING.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementationData {
    /// Implementation data bytes
    value: Vec<u8>,
}

impl ImplementationData {
    /// Create new Implementation Data
    pub fn new(value: Vec<u8>) -> Self {
        Self { value }
    }

    /// Get implementation data bytes
    pub fn value(&self) -> &[u8] {
        &self.value
    }

    /// Encode to BER format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();
        encoder.encode_octet_string(&self.value)?;
        Ok(encoder.into_bytes())
    }

    /// Decode from BER format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = BerDecoder::new(data);
        let value = decoder.decode_octet_string()?;
        Ok(Self::new(value))
    }
}

/// Application Context Name List
///
/// List of alternative application context names that the sending entity supports.
///
/// # ASN.1 Definition
/// ```asn1
/// ApplicationContextNameList ::= SEQUENCE SIZE (1..MAX) OF OBJECT IDENTIFIER
/// ```
///
/// # Usage
/// This is used in AARQ to indicate alternative application contexts that the
/// client can use if the server doesn't support the primary context.
///
/// # Example
/// ```rust
/// let list = ApplicationContextNameList::builder()
///     .with_oid(&[0, 4, 0, 127, 0, 0, 15, 0, 0, 1])
///     .with_oid(&[0, 4, 0, 127, 0, 0, 15, 0, 0, 2])
///     .build();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationContextNameList {
    /// List of OIDs representing alternative application contexts
    oids: Vec<Vec<u32>>,
}

/// Builder for creating ApplicationContextNameList
///
/// Provides a fluent interface for constructing application context name lists.
#[derive(Debug, Clone, Default)]
pub struct ApplicationContextNameListBuilder {
    oids: Vec<Vec<u32>>,
}

impl ApplicationContextNameListBuilder {
    /// Create a new empty builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an OID to the list
    ///
    /// # Arguments
    /// * `oid` - Object identifier to add
    pub fn with_oid(mut self, oid: &[u32]) -> Self {
        self.oids.push(oid.to_vec());
        self
    }

    /// Add multiple OIDs to the list
    ///
    /// # Arguments
    /// * `oids` - Slice of object identifiers to add
    pub fn with_oids(mut self, oids: &[Vec<u32>]) -> Self {
        self.oids.extend(oids.iter().cloned());
        self
    }

    /// Build the ApplicationContextNameList
    pub fn build(self) -> ApplicationContextNameList {
        ApplicationContextNameList { oids: self.oids }
    }
}

impl ApplicationContextNameList {
    /// Create new Application Context Name List
    ///
    /// # Arguments
    /// * `oids` - List of object identifiers
    pub fn new(oids: Vec<Vec<u32>>) -> Self {
        Self { oids }
    }

    /// Create an empty builder
    ///
    /// This is the starting point for building a list using the builder pattern.
    pub fn builder() -> ApplicationContextNameListBuilder {
        ApplicationContextNameListBuilder::new()
    }

    /// Create an empty list
    pub fn empty() -> Self {
        Self { oids: Vec::new() }
    }

    /// Get OID list
    pub fn oids(&self) -> &[Vec<u32>] {
        &self.oids
    }

    /// Get the number of OIDs in the list
    pub fn len(&self) -> usize {
        self.oids.len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.oids.is_empty()
    }

    /// Add an OID to the list
    ///
    /// # Arguments
    /// * `oid` - Object identifier to add
    pub fn add(&mut self, oid: Vec<u32>) {
        self.oids.push(oid);
    }

    /// Remove an OID from the list by index
    ///
    /// # Arguments
    /// * `index` - Index of the OID to remove
    pub fn remove(&mut self, index: usize) -> DlmsResult<()> {
        if index >= self.oids.len() {
            return Err(DlmsError::InvalidData(format!(
                "Index {} out of bounds (len: {})",
                index,
                self.oids.len()
            )));
        }
        self.oids.remove(index);
        Ok(())
    }

    /// Clear all OIDs from the list
    pub fn clear(&mut self) {
        self.oids.clear();
    }

    /// Encode to BER format
    ///
    /// # Encoding Format
    /// ApplicationContextNameList is encoded as a SEQUENCE OF OBJECT IDENTIFIER:
    /// - Tag: Universal, Constructed, tag 16 (SEQUENCE)
    /// - Length: Number of bytes in the sequence
    /// - Value: Concatenated encodings of each OBJECT IDENTIFIER
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        // Encode each OID
        let mut oid_encodings = Vec::new();
        for oid in &self.oids {
            let mut encoder = BerEncoder::new();
            encoder.encode_object_identifier(oid)?;
            oid_encodings.push(encoder.into_bytes());
        }

        // Calculate total content length
        let content_length: usize = oid_encodings.iter().map(|e| e.len()).sum();

        // Build SEQUENCE encoding
        let mut result = Vec::new();

        // SEQUENCE tag (0x30 = Universal, Constructed, SEQUENCE)
        result.push(0x30);

        // Length (use long form if needed)
        if content_length < 128 {
            result.push(content_length as u8);
        } else {
            // Long form length encoding
            let length_bytes = content_length.to_be_bytes();
            let num_length_bytes = length_bytes.iter().rev()
                .skip_while(|&&b| b == 0)
                .count();

            result.push(0x80 | num_length_bytes as u8);
            result.extend_from_slice(&length_bytes[(8 - num_length_bytes)..]);
        }

        // Content (concatenated OID encodings)
        for encoding in oid_encodings {
            result.extend_from_slice(&encoding);
        }

        Ok(result)
    }

    /// Decode from BER format
    ///
    /// # Expected Format
    /// SEQUENCE OF OBJECT IDENTIFIER as defined in ISO/IEC 8650
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData(
                "Empty ApplicationContextNameList".to_string(),
            ));
        }

        // First byte should be SEQUENCE tag (0x30)
        let tag_byte = data[0];
        if tag_byte != 0x30 {
            return Err(DlmsError::InvalidData(format!(
                "Expected SEQUENCE tag (0x30), got 0x{:02X}",
                tag_byte
            )));
        }

        // Decode SEQUENCE
        let mut oids = Vec::new();
        let mut pos = 0;

        // Skip SEQUENCE tag
        pos += 1;

        // Get length
        if pos >= data.len() {
            return Err(DlmsError::InvalidData(
                "Incomplete length in ApplicationContextNameList".to_string(),
            ));
        }

        let length_byte = data[pos];
        pos += 1;

        let content_length = if length_byte < 128 {
            length_byte as usize
        } else {
            // Long form
            let num_length_bytes = (length_byte & 0x7F) as usize;
            if pos + num_length_bytes > data.len() {
                return Err(DlmsError::InvalidData(
                    "Incomplete long-form length".to_string(),
                ));
            }

            let mut length = 0usize;
            for _ in 0..num_length_bytes {
                length = (length << 8) | (data[pos] as usize);
                pos += 1;
            }
            length
        };

        // Decode each OBJECT IDENTIFIER
        let end_pos = pos + content_length;
        while pos < end_pos {
            // Find the end of this OID encoding
            if pos >= data.len() {
                return Err(DlmsError::InvalidData(
                    "Incomplete OID encoding in ApplicationContextNameList".to_string(),
                ));
            }

            let oid_tag = data[pos];
            pos += 1;

            if oid_tag != 0x06 {
                // 0x06 = OBJECT IDENTIFIER tag
                return Err(DlmsError::InvalidData(format!(
                    "Expected OBJECT IDENTIFIER tag (0x06), got 0x{:02X}",
                    oid_tag
                )));
            }

            // Get OID length
            if pos >= data.len() {
                return Err(DlmsError::InvalidData(
                    "Incomplete OID length".to_string(),
                ));
            }

            let oid_length = data[pos] as usize;
            pos += 1;

            // Extract OID data
            if pos + oid_length > data.len() {
                return Err(DlmsError::InvalidData(
                    "Incomplete OID data".to_string(),
                ));
            }

            let oid_data = &data[pos..pos + oid_length];
            pos += oid_length;

            // Decode OID
            let mut oid_decoder = BerDecoder::new(oid_data);
            let oid = oid_decoder.decode_object_identifier()?;
            oids.push(oid);
        }

        Ok(Self { oids })
    }

    /// Check if the list contains a specific OID
    ///
    /// # Arguments
    /// * `oid` - Object identifier to check
    pub fn contains(&self, oid: &[u32]) -> bool {
        self.oids.iter().any(|o| o == oid)
    }

    /// Get the OID at a specific index
    ///
    /// # Arguments
    /// * `index` - Index of the OID to get
    pub fn get(&self, index: usize) -> Option<&[u32]> {
        self.oids.get(index).map(|v| v.as_slice())
    }
}

impl Default for ApplicationContextNameList {
    fn default() -> Self {
        Self::empty()
    }
}

/// Associate Source Diagnostic
///
/// Provides diagnostic information about association rejection.
///
/// This is a CHOICE type that indicates whether the diagnostic comes from
/// the ACSE service user or the ACSE service provider.
///
/// # ASN.1 Definition
/// ```asn1
/// AssociateSourceDiagnostic ::= CHOICE {
///     acse-service-user [0] INTEGER,
///     acse-service-provider [1] INTEGER
/// }
/// ```
///
/// # Why CHOICE Type?
/// The source of the diagnostic matters because:
/// - Service user diagnostics indicate application-level issues
/// - Service provider diagnostics indicate protocol/communication issues
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssociateSourceDiagnostic {
    /// Diagnostic from ACSE service user (tag 0)
    /// Indicates application-level rejection reason
    AcseServiceUser(i64),

    /// Diagnostic from ACSE service provider (tag 1)
    /// Indicates protocol-level rejection reason
    AcseServiceProvider(i64),
}

impl AssociateSourceDiagnostic {
    /// Create new AcseServiceUser diagnostic
    pub fn service_user(value: i64) -> Self {
        Self::AcseServiceUser(value)
    }

    /// Create new AcseServiceProvider diagnostic
    pub fn service_provider(value: i64) -> Self {
        Self::AcseServiceProvider(value)
    }

    /// Get diagnostic value
    pub fn value(&self) -> i64 {
        match self {
            Self::AcseServiceUser(v) => *v,
            Self::AcseServiceProvider(v) => *v,
        }
    }

    /// Get the tag value for this variant
    #[must_use]
    pub const fn tag(&self) -> u8 {
        match self {
            Self::AcseServiceUser(_) => 0,
            Self::AcseServiceProvider(_) => 1,
        }
    }

    /// Check if this is a service user diagnostic
    #[must_use]
    pub fn is_service_user(&self) -> bool {
        matches!(self, Self::AcseServiceUser(_))
    }

    /// Check if this is a service provider diagnostic
    #[must_use]
    pub fn is_service_provider(&self) -> bool {
        matches!(self, Self::AcseServiceProvider(_))
    }

    /// Encode to BER format
    ///
    /// Encoding as CHOICE with explicit context-specific tag:
    /// - Tag (0xA0 or 0xA1 for context-specific constructed)
    /// - Length
    /// - INTEGER value
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut result = Vec::new();

        // Context-specific constructed tag: 0b10100000 | tag_number
        // Bits 7-6: class (10 = context-specific)
        // Bit 5: constructed (1)
        // Bits 4-0: tag number
        let tag_byte = 0xA0 | (self.tag() & 0x1F);
        result.push(tag_byte);

        // Encode the INTEGER value
        let value_bytes = {
            let mut enc = BerEncoder::new();
            enc.encode_integer(self.value())?;
            enc.into_bytes()
        };

        // Length
        result.push(value_bytes.len() as u8);

        // Value
        result.extend_from_slice(&value_bytes);

        Ok(result)
    }

    /// Decode from BER format
    ///
    /// Expects a context-specific constructed tag followed by an INTEGER
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData(
                "Empty AssociateSourceDiagnostic".to_string(),
            ));
        }

        // First byte should be context-specific constructed tag (0xA0 or 0xA1)
        let tag_byte = data[0];
        let tag_class = (tag_byte >> 6) & 0x03;
        let is_constructed = (tag_byte & 0x20) != 0;
        let tag_number = tag_byte & 0x1F;

        if tag_class != 2 {
            // Not context-specific
            return Err(DlmsError::InvalidData(
                format!("Expected context-specific tag, got class {}", tag_class),
            ));
        }

        if !is_constructed {
            return Err(DlmsError::InvalidData(
                "Expected constructed tag for AssociateSourceDiagnostic".to_string(),
            ));
        }

        // Second byte is length
        if data.len() < 2 {
            return Err(DlmsError::InvalidData(
                "Incomplete AssociateSourceDiagnostic (missing length)".to_string(),
            ));
        }
        let length = data[1] as usize;

        // Rest should be INTEGER value
        if data.len() < 2 + length {
            return Err(DlmsError::InvalidData(
                "Incomplete AssociateSourceDiagnostic (truncated value)".to_string(),
            ));
        }

        let value_bytes = &data[2..2 + length];
        let value = {
            let mut dec = BerDecoder::new(value_bytes);
            dec.decode_integer()?
        };

        match tag_number {
            0 => Ok(Self::AcseServiceUser(value)),
            1 => Ok(Self::AcseServiceProvider(value)),
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid AssociateSourceDiagnostic tag: {}",
                tag_number
            ))),
        }
    }

    /// Create a null diagnostic (service user, value 0)
    /// Used for successful association without specific diagnostic
    #[must_use]
    pub fn null() -> Self {
        Self::AcseServiceUser(0)
    }
}

/// ACSE Service User Diagnostic Codes
///
/// Standard diagnostic codes from ACSE service user.
///
/// # Values
/// - 0: null (no reason)
/// - 1: no-reason (no reason given)
/// - 2: application-context-name-not-supported
/// - 3: calling-AP-title-not-recognized
/// - 4: calling-AE-qualifier-not-recognized
/// - 5: calling-AP-invocation-identifier-not-recognized
/// - 6: called-AP-title-not-recognized
/// - 7: called-AE-qualifier-not-recognized
/// - 8: called-AP-invocation-identifier-not-recognized
/// - 9: calling-AE-title-not-recognized
/// - 10: called-AE-title-not-recognized
/// - 11: reason-not-specified
/// - 12: authentication-required
/// - 13: authentication-mechanism-name-not-recognized
/// - 14: authentication-failure
/// - 15: authentication-name-not-recognized
///
/// # Why Struct Instead of Enum?
/// The diagnostic codes can be extended, and using integer values allows
/// for vendor-specific codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcseServiceUserDiagnostic(pub i64);

impl AcseServiceUserDiagnostic {
    /// Null diagnostic (no reason)
    pub const NULL: Self = Self(0);
    /// No reason given
    pub const NO_REASON: Self = Self(1);
    /// Application context name not supported
    pub const CONTEXT_NOT_SUPPORTED: Self = Self(2);
    /// Authentication required
    pub const AUTHENTICATION_REQUIRED: Self = Self(12);
    /// Authentication mechanism name not recognized
    pub const AUTH_MECHANISM_NOT_RECOGNIZED: Self = Self(13);
    /// Authentication failure
    pub const AUTHENTICATION_FAILURE: Self = Self(14);
    /// Authentication name not recognized
    pub const AUTH_NAME_NOT_RECOGNIZED: Self = Self(15);

    /// Get the diagnostic value
    #[must_use]
    pub fn value(self) -> i64 {
        self.0
    }
}

/// ACSE Service Provider Diagnostic Codes
///
/// Standard diagnostic codes from ACSE service provider.
///
/// # Values
/// - 0: null (no reason)
/// - 1: no-reason (no reason given)
/// - 2: temporary-congestion
///
/// # Why Struct Instead of Enum?
/// The diagnostic codes can be extended, and using integer values allows
/// for vendor-specific codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcseServiceProviderDiagnostic(pub i64);

impl AcseServiceProviderDiagnostic {
    /// Null diagnostic (no reason)
    pub const NULL: Self = Self(0);
    /// No reason given
    pub const NO_REASON: Self = Self(1);
    /// Temporary congestion
    pub const TEMPORARY_CONGESTION: Self = Self(2);

    /// Get the diagnostic value
    #[must_use]
    pub fn value(self) -> i64 {
        self.0
    }
}

