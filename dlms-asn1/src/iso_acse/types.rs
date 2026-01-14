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
use crate::ber::{BerEncoder, BerDecoder, BerTag, BerTagClass};

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
/// # Encoding
/// AP Title can be encoded in multiple forms. For DLMS/COSEM, we typically
/// use Form 2 (OCTET STRING), which contains the system title.
///
/// # TODO
/// - [ ] 实现完整的 APTitle 支持（包括 Form 1 和 Form 2）
/// - [ ] 实现 APTitleForm2 结构
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct APTitle {
    /// System title (typically 8 bytes for DLMS/COSEM)
    system_title: Vec<u8>,
}

impl APTitle {
    /// Create new AP Title from system title
    ///
    /// # Arguments
    /// * `system_title` - System title bytes (typically 8 bytes)
    pub fn new(system_title: Vec<u8>) -> Self {
        Self { system_title }
    }

    /// Get system title
    pub fn system_title(&self) -> &[u8] {
        &self.system_title
    }

    /// Encode to BER format
    ///
    /// # Encoding Format
    /// AP Title Form 2 is encoded as:
    /// - Tag: Universal, Primitive, tag 4 (OCTET STRING)
    /// - Length: Number of bytes
    /// - Value: System title bytes
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();
        encoder.encode_octet_string(&self.system_title)?;
        Ok(encoder.into_bytes())
    }

    /// Decode from BER format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = BerDecoder::new(data);
        let system_title = decoder.decode_octet_string()?;
        Ok(Self::new(system_title))
    }
}

/// AE Qualifier (Application Entity Qualifier)
///
/// Qualifies an application entity within an application process.
///
/// # TODO
/// - [ ] 实现完整的 AEQualifier 支持（包括 Form 1 和 Form 2）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AEQualifier {
    /// Qualifier value
    value: Vec<u8>,
}

impl AEQualifier {
    /// Create new AE Qualifier
    pub fn new(value: Vec<u8>) -> Self {
        Self { value }
    }

    /// Get qualifier value
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
/// # TODO
/// - [ ] 实现完整的 ACSE Requirements 位定义（需要查看标准）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ACSERequirements {
    /// Requirements bit string (typically 1 byte)
    bits: Vec<u8>,
    num_bits: usize,
}

impl ACSERequirements {
    /// Create new ACSE Requirements
    pub fn new(bits: Vec<u8>, num_bits: usize) -> Self {
        Self { bits, num_bits }
    }

    /// Get requirements bits
    pub fn bits(&self) -> &[u8] {
        &self.bits
    }

    /// Get number of bits
    pub fn num_bits(&self) -> usize {
        self.num_bits
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

/// Mechanism Name
///
/// Identifies an authentication mechanism.
///
/// # Encoding
/// Mechanism Name is encoded as an OBJECT IDENTIFIER.
///
/// # TODO
/// - [ ] 实现常用的认证机制 OID 常量（如 Low-level, HLS5-GMAC 等）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MechanismName {
    /// Object identifier components
    oid: Vec<u32>,
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
/// Authentication value (password, challenge response, etc.).
///
/// # Encoding
/// Authentication Value is a CHOICE type that can be encoded in different forms.
/// For DLMS/COSEM, we typically use OCTET STRING.
///
/// # CHOICE Definition (from DLMS Green Book)
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
///
/// # Current Implementation
/// Currently only supports OCTET STRING (tag [9]) which is the most common
/// form used in DLMS/COSEM. Full CHOICE support is planned for future implementation.
///
/// # TODO
/// - [ ] 实现完整的 AuthenticationValue CHOICE 支持（所有数据类型变体）
/// - [ ] 保持向后兼容性（现有OCTET STRING支持必须继续工作）
/// - [ ] 添加类型安全的枚举来表示所有CHOICE变体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticationValue {
    /// Authentication value bytes
    /// 
    /// Note: Currently only supports OCTET STRING format.
    /// Future implementation will support all CHOICE variants.
    value: Vec<u8>,
}

impl AuthenticationValue {
    /// Create new Authentication Value
    pub fn new(value: Vec<u8>) -> Self {
        Self { value }
    }

    /// Get authentication value bytes
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
/// List of alternative application context names.
///
/// # TODO
/// - [ ] 实现完整的 ApplicationContextNameList 结构（SEQUENCE OF）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationContextNameList {
    /// List of OIDs (placeholder - full implementation needed)
    oids: Vec<Vec<u32>>,
}

impl ApplicationContextNameList {
    /// Create new Application Context Name List
    pub fn new(oids: Vec<Vec<u32>>) -> Self {
        Self { oids }
    }

    /// Get OID list
    pub fn oids(&self) -> &[Vec<u32>] {
        &self.oids
    }

    /// Encode to BER format
    ///
    /// # TODO
    /// - [ ] 实现完整的编码逻辑（SEQUENCE OF OBJECT IDENTIFIER）
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        // Placeholder implementation
        Err(DlmsError::InvalidData(
            "ApplicationContextNameList encoding not yet implemented".to_string(),
        ))
    }

    /// Decode from BER format
    ///
    /// # TODO
    /// - [ ] 实现完整的解码逻辑
    pub fn decode(_data: &[u8]) -> DlmsResult<Self> {
        Err(DlmsError::InvalidData(
            "ApplicationContextNameList decoding not yet implemented".to_string(),
        ))
    }
}

/// Associate Source Diagnostic
///
/// Provides diagnostic information about association rejection.
///
/// # TODO
/// - [ ] 实现完整的 AssociateSourceDiagnostic 结构（CHOICE 类型）
/// - [ ] 实现 AcseServiceUser 和 AcseServiceProvider 枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociateSourceDiagnostic {
    /// Diagnostic value (placeholder)
    value: i64,
}

impl AssociateSourceDiagnostic {
    /// Create new Associate Source Diagnostic
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    /// Get diagnostic value
    pub fn value(&self) -> i64 {
        self.value
    }

    /// Encode to BER format
    ///
    /// # TODO
    /// - [ ] 实现完整的编码逻辑（CHOICE 类型）
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        // Placeholder: encode as INTEGER
        let mut encoder = BerEncoder::new();
        encoder.encode_integer(self.value)?;
        Ok(encoder.into_bytes())
    }

    /// Decode from BER format
    ///
    /// # TODO
    /// - [ ] 实现完整的解码逻辑
    pub fn decode(_data: &[u8]) -> DlmsResult<Self> {
        // Placeholder: decode as INTEGER
        let mut decoder = BerDecoder::new(_data);
        let value = decoder.decode_integer()?;
        Ok(Self::new(value))
    }
}
