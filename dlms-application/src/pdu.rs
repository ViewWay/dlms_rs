//! PDU (Protocol Data Unit) handling for DLMS/COSEM application layer
//!
//! This module provides structures and encoding/decoding for DLMS/COSEM application layer PDUs.
//! PDUs are the fundamental units of communication in the DLMS/COSEM protocol stack.
//!
//! # Architecture Overview
//!
//! DLMS/COSEM uses a layered protocol architecture:
//! - **Application Layer**: PDU structures (this module)
//! - **Session Layer**: HDLC or Wrapper protocol
//! - **Transport Layer**: TCP, UDP, or Serial
//!
//! # PDU Types
//!
//! The DLMS/COSEM protocol defines several PDU types:
//! - **Initiate**: Connection establishment and negotiation
//! - **Get/Set/Action**: Data access operations
//! - **Event Notification**: Asynchronous event reporting
//! - **Exception**: Error reporting
//!
//! # Encoding Format
//!
//! All PDUs are encoded using A-XDR (Aligned eXternal Data Representation), which provides:
//! - Compact binary format
//! - Efficient parsing
//! - Type safety through tags
//!
//! # Why This Design?
//!
//! 1. **Type Safety**: Each PDU type is a distinct Rust enum variant or struct, preventing
//!    mixing of incompatible PDU types at compile time.
//! 2. **Zero-Copy Decoding**: Where possible, we use references to avoid unnecessary allocations.
//! 3. **Error Handling**: All encoding/decoding operations return `Result` types for proper
//!    error propagation.
//! 4. **Extensibility**: The enum-based design allows easy addition of new PDU types.
//!
//! # Optimization Considerations
//!
//! - **Memory Allocation**: PDU structures use `Vec<u8>` for variable-length fields.
//!   Future optimization: Use `Bytes` or `BytesMut` for zero-copy operations.
//! - **Encoding Caching**: Currently, PDUs are encoded on-demand. For high-frequency
//!   operations, consider caching encoded representations.
//! - **Validation**: Input validation is performed during construction. Consider
//!   lazy validation for better performance in hot paths.

use dlms_core::{DlmsError, DlmsResult, ObisCode};
use dlms_core::datatypes::{BitString, DataObject};
use dlms_asn1::{AxdrDecoder, AxdrEncoder};
use crate::addressing::{LogicalNameReference, ShortNameReference, AccessSelector};

/// DLMS protocol version number
///
/// Currently, DLMS/COSEM supports version 6 (the most recent standard version).
/// This constant is used in InitiateRequest/Response PDUs to negotiate protocol capabilities.
pub const DLMS_VERSION_6: u8 = 6;

/// Maximum PDU size for DLMS/COSEM communication
///
/// This represents the maximum size of a PDU that can be transmitted in a single frame.
/// The actual negotiated size may be smaller based on device capabilities.
///
/// # Why 65535?
/// This is the maximum value for a 16-bit unsigned integer (u16::MAX), which is the
/// standard size field in DLMS/COSEM protocol. Most devices use smaller values
/// (typically 1024-4096 bytes) to optimize memory usage.
pub const MAX_PDU_SIZE: u16 = 65535;

/// Conformance bits for DLMS/COSEM protocol negotiation
///
/// Conformance is a 24-bit bitstring that indicates which DLMS/COSEM features
/// are supported by the client or server. Each bit represents a specific capability.
///
/// # Bit Layout (from LSB to MSB, bit 0 = LSB, bit 23 = MSB)
/// - Bit 0: General protection (reserved for future use)
/// - Bit 1: General block transfer (reserved for future use)
/// - Bit 2: Reserved
/// - Bit 3: Block read
/// - Bit 4: Block write
/// - Bit 5: Unconfirmed write
/// - Bit 6-7: Reserved
/// - Bit 8: Attribute 0 supported with SET
/// - Bit 9: Priority management supported
/// - Bit 10: Attribute 0 supported with GET
/// - Bit 11: Block transfer with GET or READ
/// - Bit 12: Block transfer with SET or WRITE
/// - Bit 13: Block transfer with ACTION
/// - Bit 14: Multiple references
/// - Bit 15: Information report
/// - Bit 16: Data notification
/// - Bit 17: Reserved
/// - Bit 18: Parameterized access
/// - Bit 19: GET
/// - Bit 20: SET
/// - Bit 21: Selective access
/// - Bit 22: Event notification
/// - Bit 23: ACTION
///
/// # Reference
/// Based on Green Book 8, Table 75 - Conformance bit definitions
/// and csm_definitions.h from cosemlib reference implementation
///
/// # Why BitString?
/// Using a BitString allows efficient representation of 24 boolean flags in a
/// compact format. This is more memory-efficient than using 24 separate boolean fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conformance {
    bits: BitString,
}

impl Conformance {
    /// Create a new Conformance with all bits set to false
    ///
    /// # Returns
    /// A Conformance instance with 24 bits, all set to false (no features supported)
    pub fn new() -> Self {
        // Conformance is a 24-bit bitstring (3 bytes)
        let bytes = vec![0u8; 3];
        Self {
            bits: BitString::from_bytes(bytes, 24),
        }
    }

    /// Create a Conformance from a BitString
    ///
    /// # Arguments
    /// * `bits` - BitString containing conformance bits (must be 24 bits)
    ///
    /// # Returns
    /// Returns `Ok(Conformance)` if the BitString has exactly 24 bits, `Err` otherwise
    ///
    /// # Why Validate Length?
    /// The DLMS/COSEM standard specifies exactly 24 bits for conformance. Enforcing
    /// this at construction time prevents encoding/decoding errors later.
    pub fn from_bit_string(bits: BitString) -> DlmsResult<Self> {
        if bits.num_bits() != 24 {
            return Err(DlmsError::InvalidData(format!(
                "Conformance must be exactly 24 bits, got {}",
                bits.num_bits()
            )));
        }
        Ok(Self { bits })
    }

    /// Get the underlying BitString
    pub fn bits(&self) -> &BitString {
        &self.bits
    }

    /// Encode conformance to A-XDR format
    ///
    /// Encoding format: BitString (4 bytes: 1 byte length + 3 bytes data)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();
        encoder.encode_bit_string(&self.bits)?;
        Ok(encoder.into_bytes())
    }

    /// Decode conformance from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);
        let bits = decoder.decode_bit_string()?;
        Self::from_bit_string(bits)
    }

    /// Set a specific conformance bit
    ///
    /// # Arguments
    /// * `bit` - Bit index (0-23, where 0 is LSB and 23 is MSB)
    /// * `value` - Value to set (true = supported, false = not supported)
    ///
    /// # Returns
    /// Returns `Err` if bit index is out of range (>= 24)
    pub fn set_bit(&mut self, bit: usize, value: bool) -> DlmsResult<()> {
        if bit >= 24 {
            return Err(DlmsError::InvalidData(format!(
                "Conformance bit index must be 0-23, got {}",
                bit
            )));
        }
        self.bits.set_bit(bit, value);
        Ok(())
    }

    /// Get a specific conformance bit
    ///
    /// # Arguments
    /// * `bit` - Bit index (0-23, where 0 is LSB and 23 is MSB)
    ///
    /// # Returns
    /// Returns `None` if bit index is out of range, `Some(bool)` otherwise
    pub fn get_bit(&self, bit: usize) -> Option<bool> {
        if bit >= 24 {
            return None;
        }
        Some(self.bits.get_bit(bit))
    }

    /// Set block read capability (bit 3)
    pub fn set_block_read(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(3, value)
    }

    /// Get block read capability (bit 3)
    pub fn block_read(&self) -> bool {
        self.get_bit(3).unwrap_or(false)
    }

    /// Set block write capability (bit 4)
    pub fn set_block_write(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(4, value)
    }

    /// Get block write capability (bit 4)
    pub fn block_write(&self) -> bool {
        self.get_bit(4).unwrap_or(false)
    }

    /// Set unconfirmed write capability (bit 5)
    pub fn set_unconfirmed_write(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(5, value)
    }

    /// Get unconfirmed write capability (bit 5)
    pub fn unconfirmed_write(&self) -> bool {
        self.get_bit(5).unwrap_or(false)
    }

    /// Set attribute 0 supported with SET (bit 8)
    pub fn set_attribute0_supported_with_set(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(8, value)
    }

    /// Get attribute 0 supported with SET (bit 8)
    pub fn attribute0_supported_with_set(&self) -> bool {
        self.get_bit(8).unwrap_or(false)
    }

    /// Set priority management supported (bit 9)
    pub fn set_priority_mgmt_supported(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(9, value)
    }

    /// Get priority management supported (bit 9)
    pub fn priority_mgmt_supported(&self) -> bool {
        self.get_bit(9).unwrap_or(false)
    }

    /// Set attribute 0 supported with GET (bit 10)
    pub fn set_attribute0_supported_with_get(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(10, value)
    }

    /// Get attribute 0 supported with GET (bit 10)
    pub fn attribute0_supported_with_get(&self) -> bool {
        self.get_bit(10).unwrap_or(false)
    }

    /// Set block transfer with GET or READ (bit 11)
    pub fn set_block_transfer_with_get_or_read(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(11, value)
    }

    /// Get block transfer with GET or READ (bit 11)
    pub fn block_transfer_with_get_or_read(&self) -> bool {
        self.get_bit(11).unwrap_or(false)
    }

    /// Set block transfer with SET or WRITE (bit 12)
    pub fn set_block_transfer_with_set_or_write(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(12, value)
    }

    /// Get block transfer with SET or WRITE (bit 12)
    pub fn block_transfer_with_set_or_write(&self) -> bool {
        self.get_bit(12).unwrap_or(false)
    }

    /// Set block transfer with ACTION (bit 13)
    pub fn set_block_transfer_with_action(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(13, value)
    }

    /// Get block transfer with ACTION (bit 13)
    pub fn block_transfer_with_action(&self) -> bool {
        self.get_bit(13).unwrap_or(false)
    }

    /// Set multiple references capability (bit 14)
    pub fn set_multiple_references(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(14, value)
    }

    /// Get multiple references capability (bit 14)
    pub fn multiple_references(&self) -> bool {
        self.get_bit(14).unwrap_or(false)
    }

    /// Set information report capability (bit 15)
    pub fn set_information_report(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(15, value)
    }

    /// Get information report capability (bit 15)
    pub fn information_report(&self) -> bool {
        self.get_bit(15).unwrap_or(false)
    }

    /// Set data notification capability (bit 16)
    pub fn set_data_notification(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(16, value)
    }

    /// Get data notification capability (bit 16)
    pub fn data_notification(&self) -> bool {
        self.get_bit(16).unwrap_or(false)
    }

    /// Set parameterized access capability (bit 18)
    pub fn set_parameterized_access(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(18, value)
    }

    /// Get parameterized access capability (bit 18)
    pub fn parameterized_access(&self) -> bool {
        self.get_bit(18).unwrap_or(false)
    }

    /// Set GET capability (bit 19)
    pub fn set_get(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(19, value)
    }

    /// Get GET capability (bit 19)
    pub fn get(&self) -> bool {
        self.get_bit(19).unwrap_or(false)
    }

    /// Set SET capability (bit 20)
    pub fn set_set(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(20, value)
    }

    /// Get SET capability (bit 20)
    pub fn set(&self) -> bool {
        self.get_bit(20).unwrap_or(false)
    }

    /// Set selective access capability (bit 21)
    pub fn set_selective_access(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(21, value)
    }

    /// Get selective access capability (bit 21)
    pub fn selective_access(&self) -> bool {
        self.get_bit(21).unwrap_or(false)
    }

    /// Set event notification capability (bit 22)
    pub fn set_event_notification(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(22, value)
    }

    /// Get event notification capability (bit 22)
    pub fn event_notification(&self) -> bool {
        self.get_bit(22).unwrap_or(false)
    }

    /// Set ACTION capability (bit 23)
    pub fn set_action(&mut self, value: bool) -> DlmsResult<()> {
        self.set_bit(23, value)
    }

    /// Get ACTION capability (bit 23)
    pub fn action(&self) -> bool {
        self.get_bit(23).unwrap_or(false)
    }
}

impl Default for Conformance {
    fn default() -> Self {
        Self::new()
    }
}

/// Initiate Request PDU
///
/// This PDU is sent by the client to initiate a DLMS/COSEM association.
/// It contains the client's proposed protocol parameters and capabilities.
///
/// # Structure
/// - `dedicated_key`: Optional dedicated key for secure association (used in high-security scenarios)
/// - `response_allowed`: Whether the client allows responses (default: true)
/// - `proposed_quality_of_service`: Optional quality of service parameter
/// - `proposed_dlms_version_number`: DLMS protocol version (typically 6)
/// - `proposed_conformance`: BitString indicating supported features
/// - `client_max_receive_pdu_size`: Maximum PDU size the client can receive
///
/// # Why These Fields?
/// - **dedicated_key**: Allows pre-shared key authentication for enhanced security
/// - **response_allowed**: Enables unidirectional communication modes (e.g., push notifications)
/// - **proposed_quality_of_service**: Future extension for QoS negotiation
/// - **proposed_dlms_version_number**: Ensures protocol compatibility
/// - **proposed_conformance**: Negotiates feature support (block transfer, selective access, etc.)
/// - **client_max_receive_pdu_size**: Prevents buffer overflows and enables fragmentation
///
/// # Optimization Note
/// The `dedicated_key` and `proposed_quality_of_service` are optional fields. In the
/// common case where they are not used, we avoid allocating memory for them.
#[derive(Debug, Clone, PartialEq)]
pub struct InitiateRequest {
    /// Optional dedicated key for secure association
    pub dedicated_key: Option<Vec<u8>>,
    /// Whether responses are allowed (default: true)
    pub response_allowed: bool,
    /// Optional quality of service parameter
    pub proposed_quality_of_service: Option<i8>,
    /// Proposed DLMS version number (typically 6)
    pub proposed_dlms_version_number: u8,
    /// Proposed conformance bits (24-bit bitstring)
    pub proposed_conformance: Conformance,
    /// Maximum PDU size the client can receive
    pub client_max_receive_pdu_size: u16,
}

impl InitiateRequest {
    /// Create a new InitiateRequest with default values
    ///
    /// # Default Values
    /// - `dedicated_key`: None
    /// - `response_allowed`: true
    /// - `proposed_quality_of_service`: None
    /// - `proposed_dlms_version_number`: DLMS_VERSION_6 (6)
    /// - `proposed_conformance`: Empty (no features)
    /// - `client_max_receive_pdu_size`: 65535 (maximum)
    ///
    /// # Why These Defaults?
    /// These defaults represent the most permissive configuration, allowing
    /// maximum compatibility with different server implementations.
    pub fn new() -> Self {
        Self {
            dedicated_key: None,
            response_allowed: true,
            proposed_quality_of_service: None,
            proposed_dlms_version_number: DLMS_VERSION_6,
            proposed_conformance: Conformance::new(),
            client_max_receive_pdu_size: MAX_PDU_SIZE,
        }
    }

    /// Create a new InitiateRequest with specified parameters
    ///
    /// # Arguments
    /// * `proposed_conformance` - Conformance bits indicating supported features
    /// * `client_max_receive_pdu_size` - Maximum PDU size client can receive
    ///
    /// # Returns
    /// Returns `Ok(InitiateRequest)` if parameters are valid, `Err` otherwise
    ///
    /// # Validation
    /// - `client_max_receive_pdu_size` must be > 0
    /// - `proposed_dlms_version_number` should be 6 (current standard)
    pub fn with_params(
        proposed_conformance: Conformance,
        client_max_receive_pdu_size: u16,
    ) -> DlmsResult<Self> {
        if client_max_receive_pdu_size == 0 {
            return Err(DlmsError::InvalidData(
                "client_max_receive_pdu_size must be > 0".to_string(),
            ));
        }

        Ok(Self {
            dedicated_key: None,
            response_allowed: true,
            proposed_quality_of_service: None,
            proposed_dlms_version_number: DLMS_VERSION_6,
            proposed_conformance,
            client_max_receive_pdu_size,
        })
    }

    /// Encode InitiateRequest to A-XDR format
    ///
    /// Encoding order (as per DLMS standard, encoded in reverse order):
    /// 1. client_max_receive_pdu_size (Unsigned16)
    /// 2. proposed_conformance (BitString, 24 bits)
    /// 3. proposed_dlms_version_number (Unsigned8)
    /// 4. proposed_quality_of_service (optional Integer8)
    /// 5. response_allowed (Boolean, default true)
    /// 6. dedicated_key (optional OctetString)
    ///
    /// # Why This Order?
    /// A-XDR encoding uses reverse order (last field first) for efficiency.
    /// The DLMS/COSEM standard (IEC 62056-47) specifies this encoding order.
    ///
    /// # Optional Field Encoding
    /// Optional fields in A-XDR are encoded as:
    /// 1. A Boolean flag indicating whether the field is used
    /// 2. The field value (if the flag is true)
    ///
    /// This allows the decoder to read the flag first, then conditionally read the value.
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Encode in reverse order (A-XDR convention)
        // 1. client_max_receive_pdu_size (Unsigned16)
        encoder.encode_u16(self.client_max_receive_pdu_size)?;

        // 2. proposed_conformance (BitString, 24 bits)
        encoder.encode_bit_string(self.proposed_conformance.bits())?;

        // 3. proposed_dlms_version_number (Unsigned8)
        encoder.encode_u8(self.proposed_dlms_version_number)?;

        // 4. proposed_quality_of_service (optional Integer8)
        // Optional field: encode usage flag first, then value (if present)
        // Note: In A-XDR, optional fields are encoded as: flag, then value
        encoder.encode_bool(self.proposed_quality_of_service.is_some())?;
        if let Some(qos) = self.proposed_quality_of_service {
            encoder.encode_i8(qos)?;
        }

        // 5. response_allowed (Boolean, default true)
        encoder.encode_bool(self.response_allowed)?;

        // 6. dedicated_key (optional OctetString)
        // Optional field: encode usage flag first, then value (if present)
        // Note: In A-XDR, optional fields are encoded as: flag, then value
        encoder.encode_bool(self.dedicated_key.is_some())?;
        if let Some(ref key) = self.dedicated_key {
            encoder.encode_octet_string(key)?;
        }

        Ok(encoder.into_bytes())
    }

    /// Decode InitiateRequest from A-XDR format
    ///
    /// Decoding order (reverse of encoding order):
    /// 1. dedicated_key (optional OctetString) - usage flag first, then value if used
    /// 2. response_allowed (Boolean)
    /// 3. proposed_quality_of_service (optional Integer8) - usage flag first, then value if used
    /// 4. proposed_dlms_version_number (Unsigned8)
    /// 5. proposed_conformance (BitString, 24 bits)
    /// 6. client_max_receive_pdu_size (Unsigned16)
    ///
    /// # Error Handling
    /// Returns `Err` if:
    /// - The data is too short
    /// - Invalid encoding format
    /// - Conformance bitstring is not 24 bits
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode in reverse order (A-XDR convention)
        // 1. dedicated_key (optional OctetString)
        // Optional field: decode usage flag first, then value if used
        let dedicated_key_used = decoder.decode_bool()?;
        let dedicated_key = if dedicated_key_used {
            Some(decoder.decode_octet_string()?)
        } else {
            None
        };

        // 2. response_allowed (Boolean)
        let response_allowed = decoder.decode_bool()?;

        // 3. proposed_quality_of_service (optional Integer8)
        // Optional field: decode usage flag first, then value if used
        let proposed_quality_of_service = {
            let qos_used = decoder.decode_bool()?;
            if qos_used {
                Some(decoder.decode_i8()?)
            } else {
                None
            }
        };

        // 4. proposed_dlms_version_number (Unsigned8)
        let proposed_dlms_version_number = decoder.decode_u8()?;

        // 5. proposed_conformance (BitString)
        let conformance_bits = decoder.decode_bit_string()?;
        let proposed_conformance = Conformance::from_bit_string(conformance_bits)?;

        // 6. client_max_receive_pdu_size (Unsigned16)
        let client_max_receive_pdu_size = decoder.decode_u16()?;

        Ok(Self {
            dedicated_key,
            response_allowed,
            proposed_quality_of_service,
            proposed_dlms_version_number,
            proposed_conformance,
            client_max_receive_pdu_size,
        })
    }
}

impl Default for InitiateRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Initiate Response PDU
///
/// This PDU is sent by the server in response to an InitiateRequest.
/// It contains the negotiated protocol parameters and server capabilities.
///
/// # Structure
/// - `negotiated_quality_of_service`: Optional negotiated quality of service
/// - `negotiated_dlms_version_number`: Negotiated DLMS version (typically 6)
/// - `negotiated_conformance`: BitString indicating supported features
/// - `server_max_receive_pdu_size`: Maximum PDU size the server can receive
/// - `vaa_name`: VAA (Vendor Application Association) name identifier
///
/// # Why These Fields?
/// - **negotiated_quality_of_service**: Allows QoS negotiation (future extension)
/// - **negotiated_dlms_version_number**: Confirms protocol version compatibility
/// - **negotiated_conformance**: Indicates which features the server supports
/// - **server_max_receive_pdu_size**: Prevents buffer overflows on server side
/// - **vaa_name**: Identifies the vendor-specific application association
///
/// # Negotiation Process
/// The server typically selects the minimum of client and server capabilities:
/// - Version: Minimum of client and server versions
/// - Conformance: Intersection of client and server conformance bits
/// - PDU Size: Minimum of client and server max sizes
#[derive(Debug, Clone, PartialEq)]
pub struct InitiateResponse {
    /// Optional negotiated quality of service
    pub negotiated_quality_of_service: Option<i8>,
    /// Negotiated DLMS version number
    pub negotiated_dlms_version_number: u8,
    /// Negotiated conformance bits (24-bit bitstring)
    pub negotiated_conformance: Conformance,
    /// Maximum PDU size the server can receive
    pub server_max_receive_pdu_size: u16,
    /// VAA (Vendor Application Association) name identifier
    pub vaa_name: i16,
}

impl InitiateResponse {
    /// Create a new InitiateResponse
    ///
    /// # Arguments
    /// * `negotiated_dlms_version_number` - Negotiated DLMS version (typically 6)
    /// * `negotiated_conformance` - Negotiated conformance bits
    /// * `server_max_receive_pdu_size` - Maximum PDU size server can receive
    /// * `vaa_name` - VAA name identifier (typically 0x0007 for standard DLMS)
    ///
    /// # Returns
    /// Returns `Ok(InitiateResponse)` if parameters are valid, `Err` otherwise
    ///
    /// # Validation
    /// - `server_max_receive_pdu_size` must be > 0
    pub fn new(
        negotiated_dlms_version_number: u8,
        negotiated_conformance: Conformance,
        server_max_receive_pdu_size: u16,
        vaa_name: i16,
    ) -> DlmsResult<Self> {
        if server_max_receive_pdu_size == 0 {
            return Err(DlmsError::InvalidData(
                "server_max_receive_pdu_size must be > 0".to_string(),
            ));
        }

        Ok(Self {
            negotiated_quality_of_service: None,
            negotiated_dlms_version_number,
            negotiated_conformance,
            server_max_receive_pdu_size,
            vaa_name,
        })
    }

    /// Encode InitiateResponse to A-XDR format
    ///
    /// Encoding order (as per DLMS standard, encoded in reverse order):
    /// 1. vaa_name (Integer16)
    /// 2. server_max_receive_pdu_size (Unsigned16)
    /// 3. negotiated_conformance (BitString, 24 bits)
    /// 4. negotiated_dlms_version_number (Unsigned8)
    /// 5. negotiated_quality_of_service (optional Integer8)
    ///
    /// # Optional Field Encoding
    /// Optional fields are encoded as: value (if present), then usage flag.
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Encode in reverse order (A-XDR convention)
        // 1. vaa_name (Integer16)
        encoder.encode_i16(self.vaa_name)?;

        // 2. server_max_receive_pdu_size (Unsigned16)
        encoder.encode_u16(self.server_max_receive_pdu_size)?;

        // 3. negotiated_conformance (BitString, 24 bits)
        encoder.encode_bit_string(self.negotiated_conformance.bits())?;

        // 4. negotiated_dlms_version_number (Unsigned8)
        encoder.encode_u8(self.negotiated_dlms_version_number)?;

        // 5. negotiated_quality_of_service (optional Integer8)
        // Optional field: encode usage flag first, then value (if present)
        // Note: In A-XDR, optional fields are encoded as: flag, then value
        encoder.encode_bool(self.negotiated_quality_of_service.is_some())?;
        if let Some(qos) = self.negotiated_quality_of_service {
            encoder.encode_i8(qos)?;
        }

        Ok(encoder.into_bytes())
    }

    /// Decode InitiateResponse from A-XDR format
    ///
    /// Decoding order (reverse of encoding order):
    /// 1. negotiated_quality_of_service (optional Integer8) - usage flag first, then value if used
    /// 2. negotiated_dlms_version_number (Unsigned8)
    /// 3. negotiated_conformance (BitString, 24 bits)
    /// 4. server_max_receive_pdu_size (Unsigned16)
    /// 5. vaa_name (Integer16)
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode in reverse order (A-XDR convention)
        // 1. negotiated_quality_of_service (optional Integer8)
        // Optional field: decode usage flag first, then value if used
        let negotiated_quality_of_service = {
            let qos_used = decoder.decode_bool()?;
            if qos_used {
                Some(decoder.decode_i8()?)
            } else {
                None
            }
        };

        // 2. negotiated_dlms_version_number (Unsigned8)
        let negotiated_dlms_version_number = decoder.decode_u8()?;

        // 3. negotiated_conformance (BitString)
        let conformance_bits = decoder.decode_bit_string()?;
        let negotiated_conformance = Conformance::from_bit_string(conformance_bits)?;

        // 4. server_max_receive_pdu_size (Unsigned16)
        let server_max_receive_pdu_size = decoder.decode_u16()?;

        // 5. vaa_name (Integer16)
        let vaa_name = decoder.decode_i16()?;

        Ok(Self {
            negotiated_quality_of_service,
            negotiated_dlms_version_number,
            negotiated_conformance,
            server_max_receive_pdu_size,
            vaa_name,
        })
    }
}

// ============================================================================
// Get Request/Response PDU Implementation
// ============================================================================

/// Invoke ID and Priority
///
/// This is an 8-bit bitstring that combines:
/// - **Invoke ID** (bits 0-6): Unique identifier for the request/response pair
/// - **Priority** (bit 7): High priority flag (0 = normal, 1 = high)
///
/// # Why Combine ID and Priority?
/// Combining these into a single byte reduces message overhead while maintaining
/// the ability to track multiple concurrent requests and prioritize them.
///
/// # Invoke ID Range
/// Valid invoke IDs are 0-127 (7 bits). ID 0 is typically reserved for unconfirmed
/// operations. IDs are assigned by the client and echoed by the server in responses.
///
/// # Priority Usage
/// High priority requests are processed before normal priority requests, which is
/// useful for time-critical operations like event notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvokeIdAndPriority {
    /// Invoke ID (0-127)
    invoke_id: u8,
    /// High priority flag
    high_priority: bool,
}

impl InvokeIdAndPriority {
    /// Create a new InvokeIdAndPriority
    ///
    /// # Arguments
    /// * `invoke_id` - Invoke ID (0-127)
    /// * `high_priority` - Whether this is a high priority request
    ///
    /// # Returns
    /// Returns `Ok(InvokeIdAndPriority)` if valid, `Err` otherwise
    ///
    /// # Validation
    /// - `invoke_id` must be <= 127 (7 bits)
    pub fn new(invoke_id: u8, high_priority: bool) -> DlmsResult<Self> {
        if invoke_id > 127 {
            return Err(DlmsError::InvalidData(format!(
                "Invoke ID must be <= 127, got {}",
                invoke_id
            )));
        }
        Ok(Self {
            invoke_id,
            high_priority,
        })
    }

    /// Get invoke ID
    pub fn invoke_id(&self) -> u8 {
        self.invoke_id
    }

    /// Check if high priority
    pub fn is_high_priority(&self) -> bool {
        self.high_priority
    }

    /// Encode to A-XDR format (8-bit BitString)
    ///
    /// Encoding format:
    /// - Bit 7: High priority flag (1 = high, 0 = normal)
    /// - Bits 0-6: Invoke ID
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();
        let mut byte = self.invoke_id;
        if self.high_priority {
            byte |= 0x80; // Set bit 7
        }
        // Encode as 8-bit BitString
        let bits = BitString::from_bytes(vec![byte], 8);
        encoder.encode_bit_string(&bits)?;
        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);
        let bits = decoder.decode_bit_string()?;
        
        if bits.num_bits() != 8 {
            return Err(DlmsError::InvalidData(format!(
                "InvokeIdAndPriority must be 8 bits, got {}",
                bits.num_bits()
            )));
        }

        let bytes = bits.as_bytes();
        if bytes.is_empty() {
            return Err(DlmsError::InvalidData(
                "Empty BitString for InvokeIdAndPriority".to_string(),
            ));
        }

        let byte = bytes[0];
        let high_priority = (byte & 0x80) != 0;
        let invoke_id = byte & 0x7F;

        Self::new(invoke_id, high_priority)
    }
}

/// COSEM Attribute Descriptor
///
/// Describes a COSEM object attribute to be accessed. Supports both Logical Name (LN)
/// and Short Name addressing methods.
///
/// # Structure
/// - `class_id`: COSEM interface class ID (e.g., 1 for Data, 3 for Register)
/// - `instance_id`: Object instance identifier (OBIS code for LN, or base name for SN)
/// - `attribute_id`: Attribute number within the class (1-255)
///
/// # Why This Structure?
/// This structure encapsulates all information needed to reference a COSEM attribute,
/// regardless of the addressing method used. The addressing method is determined
/// by the instance_id format (6 bytes for LN, 2 bytes for SN).
///
/// # Optimization Note
/// For LN addressing, we use the existing `LogicalNameReference` structure.
/// For SN addressing, we use the existing `ShortNameReference` structure.
/// This avoids duplication and ensures consistency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CosemAttributeDescriptor {
    /// Logical Name addressing
    LogicalName(LogicalNameReference),
    /// Short Name addressing
    ShortName(ShortNameReference),
}

impl CosemAttributeDescriptor {
    /// Create a new descriptor using Logical Name addressing
    ///
    /// # Arguments
    /// * `class_id` - COSEM interface class ID
    /// * `instance_id` - OBIS code (6 bytes)
    /// * `attribute_id` - Attribute ID (1-255)
    pub fn new_logical_name(
        class_id: u16,
        instance_id: ObisCode,
        attribute_id: u8,
    ) -> DlmsResult<Self> {
        Ok(Self::LogicalName(LogicalNameReference::new(
            class_id,
            instance_id,
            attribute_id,
        )?))
    }

    /// Create a new descriptor using Short Name addressing
    ///
    /// # Arguments
    /// * `base_name` - Base name (16-bit address)
    /// * `attribute_id` - Attribute ID (1-255)
    pub fn new_short_name(base_name: u16, attribute_id: u8) -> DlmsResult<Self> {
        Ok(Self::ShortName(ShortNameReference::new(base_name, attribute_id)?))
    }

    /// Encode to A-XDR format
    ///
    /// Encoding format (A-XDR, reverse order):
    /// 1. attribute_id (Integer8)
    /// 2. instance_id (OctetString - 6 bytes for LN, 2 bytes for SN)
    /// 3. class_id (Unsigned16)
    ///
    /// # Why This Order?
    /// A-XDR uses reverse order encoding. The decoder reads fields in reverse order
    /// to match the encoding order.
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        match self {
            CosemAttributeDescriptor::LogicalName(ref ln_ref) => {
                // Encode in reverse order
                // 1. attribute_id (Integer8)
                encoder.encode_i8(ln_ref.id as i8)?;

                // 2. instance_id (OctetString, 6 bytes for OBIS code)
                let obis_bytes = ln_ref.instance_id.as_bytes();
                encoder.encode_octet_string(obis_bytes)?;

                // 3. class_id (Unsigned16)
                encoder.encode_u16(ln_ref.class_id)?;
            }
            CosemAttributeDescriptor::ShortName(ref sn_ref) => {
                // Encode in reverse order
                // 1. attribute_id (Integer8)
                encoder.encode_i8(sn_ref.id as i8)?;

                // 2. instance_id (OctetString, 2 bytes for base name)
                // Note: For SN addressing, we encode base_name as a 2-byte OctetString
                encoder.encode_octet_string(&sn_ref.base_name.to_be_bytes())?;

                // 3. class_id (Unsigned16)
                // Note: For SN addressing, class_id is typically 0 or not used
                // But we encode it for consistency with the structure
                encoder.encode_u16(0)?; // SN addressing doesn't use class_id in the same way
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    ///
    /// # Decoding Strategy
    /// The decoder determines the addressing method by checking the instance_id length:
    /// - 6 bytes: Logical Name addressing
    /// - 2 bytes: Short Name addressing
    ///
    /// # Error Handling
    /// Returns error if instance_id length is neither 2 nor 6 bytes.
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode in reverse order
        // 1. class_id (Unsigned16)
        let class_id = decoder.decode_u16()?;

        // 2. instance_id (OctetString)
        let instance_bytes = decoder.decode_octet_string()?;

        // 3. attribute_id (Integer8)
        // Note: decode_i8 returns i8, but attribute_id is u8. We cast the signed value to unsigned.
        // This is safe because attribute IDs are always positive values (0-255 range).
        let attribute_id_i8: i8 = decoder.decode_i8()?;
        let attribute_id: u8 = attribute_id_i8 as u8;

        // Determine addressing method by instance_id length
        match instance_bytes.len() {
            6 => {
                // Logical Name addressing
                let instance_id = ObisCode::new(
                    instance_bytes[0],
                    instance_bytes[1],
                    instance_bytes[2],
                    instance_bytes[3],
                    instance_bytes[4],
                    instance_bytes[5],
                );
                Ok(Self::LogicalName(LogicalNameReference::new(
                    class_id,
                    instance_id,
                    attribute_id,
                )?))
            }
            2 => {
                // Short Name addressing
                let base_name = u16::from_be_bytes([instance_bytes[0], instance_bytes[1]]);
                Ok(Self::ShortName(ShortNameReference::new(base_name, attribute_id)?))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid instance_id length: expected 2 or 6 bytes, got {}",
                instance_bytes.len()
            ))),
        }
    }
}

/// Selective Access Descriptor
///
/// Describes selective access parameters for array/table attributes. This allows
/// accessing specific elements or ranges within large attributes.
///
/// # Structure
/// - `access_selector`: Selector type (0 = entry index, 1 = date range, etc.)
/// - `access_parameters`: Selector-specific parameters (encoded as DataObject)
///
/// # Why Selective Access?
/// Some attributes (like Profile Generic buffer) can contain thousands of entries.
/// Selective access allows:
/// - Reading specific entries by index
/// - Reading entries within a date/time range
/// - Filtering entries by criteria
///
/// This significantly reduces bandwidth and processing time.
///
/// # Access Selector Values
/// - 0: Entry index (start_index, count)
/// - 1: Date range (from_date, to_date)
/// - 2-255: Reserved for future use
#[derive(Debug, Clone, PartialEq)]
pub struct SelectiveAccessDescriptor {
    /// Access selector type (0-255)
    pub access_selector: u8,
    /// Access parameters (encoded as DataObject)
    pub access_parameters: DataObject,
}

impl SelectiveAccessDescriptor {
    /// Create a new SelectiveAccessDescriptor
    ///
    /// # Arguments
    /// * `access_selector` - Selector type (0 = entry index, 1 = date range, etc.)
    /// * `access_parameters` - Selector-specific parameters
    pub fn new(access_selector: u8, access_parameters: DataObject) -> Self {
        Self {
            access_selector,
            access_parameters,
        }
    }

    /// Encode to A-XDR format
    ///
    /// Encoding format (A-XDR, reverse order):
    /// 1. access_parameters (DataObject)
    /// 2. access_selector (Unsigned8)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Encode in reverse order
        // 1. access_parameters (DataObject)
        encoder.encode_data_object(&self.access_parameters)?;

        // 2. access_selector (Unsigned8)
        encoder.encode_u8(self.access_selector)?;

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode in reverse order
        // 1. access_selector (Unsigned8)
        let access_selector = decoder.decode_u8()?;

        // 2. access_parameters (DataObject)
        let access_parameters = decoder.decode_data_object()?;

        Ok(Self {
            access_selector,
            access_parameters,
        })
    }
}

/// Get Data Result
///
/// Result of a GET operation. Can be either:
/// - **Data**: Successfully retrieved data (DataObject)
/// - **DataAccessResult**: Error code indicating why the access failed
///
/// # Why CHOICE Type?
/// Using a CHOICE type allows the same structure to represent both success and
/// failure cases, reducing code duplication and improving type safety.
///
/// # Data Access Result Codes
/// Based on Green Book 8 and csm_definitions.h reference implementation:
/// - 0: Success (should use Data variant instead)
/// - 1: Hardware fault
/// - 2: Temporary failure
/// - 3: Read-write denied
/// - 4: Object undefined
/// - 5-8: Reserved
/// - 9: Object class inconsistent
/// - 10: Reserved
/// - 11: Object unavailable
/// - 12: Type unmatched
/// - 13: Scope of access violated
/// - 14: Data block unavailable
/// - 15: Long GET aborted
/// - 16: No long GET in progress
/// - 17: Long SET aborted
/// - 18: No long SET in progress
/// - 19: Data block number invalid
/// - 20-249: Reserved
/// - 250: Other reason
/// - 251-254: Reserved
/// - 255: Not set
#[derive(Debug, Clone, PartialEq)]
pub enum GetDataResult {
    /// Successfully retrieved data
    Data(DataObject),
    /// Data access error code
    DataAccessResult(u8),
}

/// Data Access Result error codes
///
/// Based on Green Book 8 and csm_definitions.h reference implementation.
/// These constants provide type-safe error code values for DataAccessResult.
pub mod data_access_result {
    /// Success (should use Data variant instead)
    pub const SUCCESS: u8 = 0;
    /// Hardware fault
    pub const HARDWARE_FAULT: u8 = 1;
    /// Temporary failure
    pub const TEMPORARY_FAILURE: u8 = 2;
    /// Read-write denied
    pub const READ_WRITE_DENIED: u8 = 3;
    /// Object undefined
    pub const OBJECT_UNDEFINED: u8 = 4;
    /// Object class inconsistent
    pub const OBJECT_CLASS_INCONSISTENT: u8 = 9;
    /// Object unavailable
    pub const OBJECT_UNAVAILABLE: u8 = 11;
    /// Type unmatched
    pub const TYPE_UNMATCHED: u8 = 12;
    /// Scope of access violated
    pub const SCOPE_OF_ACCESS_VIOLATED: u8 = 13;
    /// Data block unavailable
    pub const DATA_BLOCK_UNAVAILABLE: u8 = 14;
    /// Long GET aborted
    pub const LONG_GET_ABORTED: u8 = 15;
    /// No long GET in progress
    pub const NO_LONG_GET_IN_PROGRESS: u8 = 16;
    /// Long SET aborted
    pub const LONG_SET_ABORTED: u8 = 17;
    /// No long SET in progress
    pub const NO_LONG_SET_IN_PROGRESS: u8 = 18;
    /// Data block number invalid
    pub const DATA_BLOCK_NUMBER_INVALID: u8 = 19;
    /// Other reason
    pub const OTHER_REASON: u8 = 250;
    /// Not set
    pub const NOT_SET: u8 = 255;
}

impl GetDataResult {
    /// Create a new GetDataResult with data
    pub fn new_data(data: DataObject) -> Self {
        Self::Data(data)
    }

    /// Create a new GetDataResult with error code
    pub fn new_error(code: u8) -> Self {
        Self::DataAccessResult(code)
    }

    /// Create a new GetDataResult with a standard error code
    ///
    /// # Arguments
    /// * `code` - One of the constants from `data_access_result` module
    ///
    /// # Example
    /// ```rust,no_run
    /// use dlms_application::pdu::{GetDataResult, data_access_result};
    /// let result = GetDataResult::new_standard_error(data_access_result::HARDWARE_FAULT);
    /// ```
    pub fn new_standard_error(code: u8) -> Self {
        Self::DataAccessResult(code)
    }

    /// Check if this is a success result
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Data(_))
    }

    /// Get the error code if this is an error result
    pub fn error_code(&self) -> Option<u8> {
        match self {
            Self::DataAccessResult(code) => Some(*code),
            _ => None,
        }
    }

    /// Get the data if this is a success result
    pub fn data(&self) -> Option<&DataObject> {
        match self {
            Self::Data(data) => Some(data),
            _ => None,
        }
    }

    /// Get a human-readable description of the error code
    ///
    /// # Returns
    /// A string describing the error, or "Unknown error code" if the code is not recognized
    pub fn error_description(&self) -> &'static str {
        match self {
            Self::Data(_) => "Success",
            Self::DataAccessResult(code) => match *code {
                data_access_result::SUCCESS => "Success",
                data_access_result::HARDWARE_FAULT => "Hardware fault",
                data_access_result::TEMPORARY_FAILURE => "Temporary failure",
                data_access_result::READ_WRITE_DENIED => "Read-write denied",
                data_access_result::OBJECT_UNDEFINED => "Object undefined",
                data_access_result::OBJECT_CLASS_INCONSISTENT => "Object class inconsistent",
                data_access_result::OBJECT_UNAVAILABLE => "Object unavailable",
                data_access_result::TYPE_UNMATCHED => "Type unmatched",
                data_access_result::SCOPE_OF_ACCESS_VIOLATED => "Scope of access violated",
                data_access_result::DATA_BLOCK_UNAVAILABLE => "Data block unavailable",
                data_access_result::LONG_GET_ABORTED => "Long GET aborted",
                data_access_result::NO_LONG_GET_IN_PROGRESS => "No long GET in progress",
                data_access_result::LONG_SET_ABORTED => "Long SET aborted",
                data_access_result::NO_LONG_SET_IN_PROGRESS => "No long SET in progress",
                data_access_result::DATA_BLOCK_NUMBER_INVALID => "Data block number invalid",
                data_access_result::OTHER_REASON => "Other reason",
                data_access_result::NOT_SET => "Not set",
                _ => "Unknown error code",
            },
        }
    }

    /// Encode to A-XDR format
    ///
    /// Encoding format (A-XDR CHOICE):
    /// 1. Choice tag (Enumerate: 0 = Data, 1 = DataAccessResult)
    /// 2. Value (DataObject for Data, Unsigned8 for DataAccessResult)
    ///
    /// # A-XDR CHOICE Encoding Order
    /// According to A-XDR standard, CHOICE types are encoded as: tag + value.
    /// The tag comes first to identify which variant is present, followed by the value.
    ///
    /// # Why This Order?
    /// - **Tag First**: Allows the decoder to know which variant to expect before reading the value
    /// - **Standard Compliance**: Matches A-XDR standard specification
    /// - **Consistency**: Matches the decode order (tag first, then value) and other result types
    /// - **Roundtrip Compatibility**: Ensures encode/decode roundtrip works correctly
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        match self {
            GetDataResult::Data(data) => {
                // Encode choice tag first (0 = Data)
                encoder.encode_u8(0)?;
                // Encode value after tag
                encoder.encode_data_object(data)?;
            }
            GetDataResult::DataAccessResult(code) => {
                // Encode choice tag first (1 = DataAccessResult)
                encoder.encode_u8(1)?;
                // Encode value after tag
                encoder.encode_u8(*code)?;
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    ///
    /// # A-XDR CHOICE Decoding Order
    /// Decodes in the same order as encoding: tag first, then value.
    /// This matches the A-XDR standard and ensures roundtrip compatibility.
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode choice tag first
        let choice_tag = decoder.decode_u8()?;

        match choice_tag {
            0 => {
                // Data variant: decode value after tag
                let data_obj = decoder.decode_data_object()?;
                Ok(Self::Data(data_obj))
            }
            1 => {
                // DataAccessResult variant: decode value after tag
                let code = decoder.decode_u8()?;
                Ok(Self::DataAccessResult(code))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid GetDataResult choice tag: {} (expected 0 or 1)",
                choice_tag
            ))),
        }
    }
}

/// Get Request Normal
///
/// Single attribute GET request. This is the most common GET request type.
///
/// # Structure
/// - `invoke_id_and_priority`: Invoke ID and priority
/// - `cosem_attribute_descriptor`: Attribute to read
/// - `access_selection`: Optional selective access descriptor
///
/// # Usage
/// This request is used to read a single attribute from a COSEM object.
/// If selective access is provided, only the specified elements are returned.
#[derive(Debug, Clone, PartialEq)]
pub struct GetRequestNormal {
    /// Invoke ID and priority
    pub invoke_id_and_priority: InvokeIdAndPriority,
    /// Attribute descriptor
    pub cosem_attribute_descriptor: CosemAttributeDescriptor,
    /// Optional selective access descriptor
    pub access_selection: Option<SelectiveAccessDescriptor>,
}

impl GetRequestNormal {
    /// Create a new GetRequestNormal
    ///
    /// # Arguments
    /// * `invoke_id_and_priority` - Invoke ID and priority
    /// * `cosem_attribute_descriptor` - Attribute to read
    /// * `access_selection` - Optional selective access
    pub fn new(
        invoke_id_and_priority: InvokeIdAndPriority,
        cosem_attribute_descriptor: CosemAttributeDescriptor,
        access_selection: Option<SelectiveAccessDescriptor>,
    ) -> Self {
        Self {
            invoke_id_and_priority,
            cosem_attribute_descriptor,
            access_selection,
        }
    }

    /// Encode to A-XDR format
    ///
    /// Encoding order (A-XDR, reverse order):
    /// 1. access_selection (optional SelectiveAccessDescriptor)
    /// 2. cosem_attribute_descriptor (CosemAttributeDescriptor)
    /// 3. invoke_id_and_priority (InvokeIdAndPriority)
    ///
    /// # Optional Field Encoding
    /// Optional fields are encoded as: flag, then value (if flag is true).
    ///
    /// # Nested Structure Encoding
    /// In A-XDR, SEQUENCE fields are directly concatenated without additional
    /// length prefixes. Each nested structure encodes its fields directly into
    /// the parent structure's buffer.
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Encode in reverse order
        // 1. access_selection (optional SelectiveAccessDescriptor)
        // Optional field: encode usage flag first, then value (if present)
        encoder.encode_bool(self.access_selection.is_some())?;
        if let Some(ref access) = self.access_selection {
            // Directly encode the nested structure's fields
            let access_bytes = access.encode()?;
            encoder.encode_bytes(&access_bytes)?;
        }

        // 2. cosem_attribute_descriptor (CosemAttributeDescriptor)
        // Directly encode the nested structure's fields
        let attr_bytes = self.cosem_attribute_descriptor.encode()?;
        encoder.encode_bytes(&attr_bytes)?;

        // 3. invoke_id_and_priority (InvokeIdAndPriority)
        // Directly encode the nested structure's fields
        let invoke_bytes = self.invoke_id_and_priority.encode()?;
        encoder.encode_bytes(&invoke_bytes)?;

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    ///
    /// # Decoding Strategy
    /// In A-XDR, SEQUENCE fields are directly concatenated. We decode each
    /// field in sequence from the decoder's current position.
    ///
    /// # Note on Nested Structures
    /// Nested structures are decoded by creating a temporary decoder from the current
    /// position, decoding the structure, then calculating bytes consumed by re-encoding.
    /// This approach works because A-XDR structures have deterministic encoding lengths.
    ///
    /// # Future Optimization
    /// Consider modifying decode methods to return (value, bytes_consumed) tuples
    /// to avoid the need for re-encoding to calculate consumed bytes.
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);
        let mut pos = 0;

        // Decode in reverse order (A-XDR convention)
        // 1. invoke_id_and_priority (InvokeIdAndPriority)
        // Decode from current position
        let invoke_id_and_priority = InvokeIdAndPriority::decode(&data[pos..])?;
        // Calculate bytes consumed by re-encoding
        let invoke_encoded = invoke_id_and_priority.encode()?;
        pos += invoke_encoded.len();

        // 2. cosem_attribute_descriptor (CosemAttributeDescriptor)
        let cosem_attribute_descriptor = CosemAttributeDescriptor::decode(&data[pos..])?;
        let attr_encoded = cosem_attribute_descriptor.encode()?;
        pos += attr_encoded.len();

        // 3. access_selection (optional SelectiveAccessDescriptor)
        // Optional field: decode usage flag first, then value if used
        // Create a temporary decoder to read the boolean flag
        let mut temp_decoder = AxdrDecoder::new(&data[pos..]);
        let access_used = temp_decoder.decode_bool()?;
        pos += temp_decoder.position();

        let access_selection = if access_used {
            let access = SelectiveAccessDescriptor::decode(&data[pos..])?;
            let access_encoded = access.encode()?;
            pos += access_encoded.len();
            Some(access)
        } else {
            None
        };

        Ok(Self {
            invoke_id_and_priority,
            cosem_attribute_descriptor,
            access_selection,
        })
    }
}

/// Get Response Normal
///
/// Single attribute GET response. Contains the result of a GetRequestNormal.
///
/// # Structure
/// - `invoke_id_and_priority`: Echoed invoke ID and priority from request
/// - `result`: Get data result (success or error)
///
/// # Usage
/// This response is sent by the server in response to a GetRequestNormal.
/// The invoke_id_and_priority must match the request to allow correlation.
#[derive(Debug, Clone, PartialEq)]
pub struct GetResponseNormal {
    /// Invoke ID and priority (echoed from request)
    pub invoke_id_and_priority: InvokeIdAndPriority,
    /// Get data result
    pub result: GetDataResult,
}

impl GetResponseNormal {
    /// Create a new GetResponseNormal
    ///
    /// # Arguments
    /// * `invoke_id_and_priority` - Invoke ID and priority (from request)
    /// * `result` - Get data result
    pub fn new(
        invoke_id_and_priority: InvokeIdAndPriority,
        result: GetDataResult,
    ) -> Self {
        Self {
            invoke_id_and_priority,
            result,
        }
    }

    /// Encode to A-XDR format
    ///
    /// Encoding order (A-XDR, reverse order):
    /// 1. result (GetDataResult)
    /// 2. invoke_id_and_priority (InvokeIdAndPriority)
    ///
    /// # Nested Structure Encoding
    /// Nested structures are directly concatenated in A-XDR SEQUENCE.
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Encode in reverse order
        // 1. result (GetDataResult)
        // Directly encode the nested structure's fields
        let result_bytes = self.result.encode()?;
        encoder.encode_bytes(&result_bytes)?;

        // 2. invoke_id_and_priority (InvokeIdAndPriority)
        // Directly encode the nested structure's fields
        let invoke_bytes = self.invoke_id_and_priority.encode()?;
        encoder.encode_bytes(&invoke_bytes)?;

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    ///
    /// # Decoding Strategy
    /// Decode nested structures from the current position, tracking bytes consumed.
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut pos = 0;

        // Decode in reverse order (A-XDR convention)
        // 1. invoke_id_and_priority (InvokeIdAndPriority)
        let invoke_id_and_priority = InvokeIdAndPriority::decode(&data[pos..])?;
        let invoke_encoded = invoke_id_and_priority.encode()?;
        pos += invoke_encoded.len();

        // 2. result (GetDataResult)
        let result = GetDataResult::decode(&data[pos..])?;
        // Note: We don't need to track position for result since it's the last field

        Ok(Self {
            invoke_id_and_priority,
            result,
        })
    }
}

/// Get Request PDU
///
/// CHOICE type representing different GET request variants:
/// - **Normal**: Single attribute request
/// - **Next**: Continue reading data block (for large attributes)
/// - **WithList**: Multiple attribute request
///
/// # Why CHOICE Type?
/// DLMS/COSEM supports multiple GET request types for different use cases.
/// Using a CHOICE type allows the same PDU structure to handle all variants
/// while maintaining type safety.
///
/// # Usage
/// Most common usage is `Normal` for reading a single attribute. `Next` is used
/// when a previous GET request returned a data block that needs continuation.
/// `WithList` is used for batch reading multiple attributes in a single request.
#[derive(Debug, Clone, PartialEq)]
pub enum GetRequest {
    /// Single attribute GET request
    Normal(GetRequestNormal),
    /// Continue reading data block
    ///
    /// # TODO
    /// - [ ] 实现 GetRequestNext 结构
    Next {
        /// Invoke ID and priority
        invoke_id_and_priority: InvokeIdAndPriority,
        /// Block number (for continuation)
        block_number: u32,
    },
    /// Multiple attribute GET request
    ///
    /// # TODO
    /// - [ ] 实现 GetRequestWithList 结构
    WithList {
        /// Invoke ID and priority
        invoke_id_and_priority: InvokeIdAndPriority,
        /// List of attribute descriptors
        attribute_descriptor_list: Vec<CosemAttributeDescriptor>,
        /// Optional access selection list (one per descriptor)
        access_selection_list: Option<Vec<Option<SelectiveAccessDescriptor>>>,
    },
}

impl GetRequest {
    /// Create a new Normal GET request
    ///
    /// # Arguments
    /// * `invoke_id_and_priority` - Invoke ID and priority
    /// * `cosem_attribute_descriptor` - Attribute to read
    /// * `access_selection` - Optional selective access
    pub fn new_normal(
        invoke_id_and_priority: InvokeIdAndPriority,
        cosem_attribute_descriptor: CosemAttributeDescriptor,
        access_selection: Option<SelectiveAccessDescriptor>,
    ) -> Self {
        Self::Normal(GetRequestNormal::new(
            invoke_id_and_priority,
            cosem_attribute_descriptor,
            access_selection,
        ))
    }

    /// Encode to A-XDR format
    ///
    /// Encoding format (A-XDR CHOICE):
    /// 1. Choice tag (Enumerate: 1 = Normal, 2 = Next, 3 = WithList)
    /// 2. Choice value (encoded according to variant)
    ///
    /// # Why This Encoding?
    /// A-XDR CHOICE types are encoded as: value + tag (reverse order).
    /// The tag identifies which variant is present.
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        match self {
            GetRequest::Normal(normal) => {
                // Encode choice tag first (1 = Normal)
                encoder.encode_u8(1)?;
                // Encode value after tag (as octet string with length prefix)
                let normal_bytes = normal.encode()?;
                encoder.encode_octet_string(&normal_bytes)?;
            }
            GetRequest::Next {
                invoke_id_and_priority,
                block_number,
            } => {
                // Encode choice tag first (2 = Next)
                encoder.encode_u8(2)?;
                // Encode value after tag (in reverse order for SEQUENCE)
                encoder.encode_u32(*block_number)?;
                let invoke_bytes = invoke_id_and_priority.encode()?;
                encoder.encode_octet_string(&invoke_bytes)?;
            }
            GetRequest::WithList {
                invoke_id_and_priority,
                attribute_descriptor_list,
                access_selection_list,
            } => {
                // Validate: attribute_descriptor_list must not be empty
                if attribute_descriptor_list.is_empty() {
                    return Err(DlmsError::InvalidData(
                        "GetRequest::WithList: attribute_descriptor_list cannot be empty".to_string(),
                    ));
                }

                // Validate: if access_selection_list exists, it must have the same length
                if let Some(ref access_list) = access_selection_list {
                    if access_list.len() != attribute_descriptor_list.len() {
                        return Err(DlmsError::InvalidData(format!(
                            "GetRequest::WithList: access_selection_list length ({}) must match attribute_descriptor_list length ({})",
                            access_list.len(),
                            attribute_descriptor_list.len()
                        )));
                    }
                }

                // Encode in reverse order (A-XDR SEQUENCE convention)
                // 1. access_selection_list (optional array of optional SelectiveAccessDescriptor)
                if let Some(ref access_list) = access_selection_list {
                    // Encode usage flag: true (array exists)
                    encoder.encode_bool(true)?;
                    
                    // Encode array length
                    let len_enc = if access_list.len() < 128 {
                        LengthEncoding::Short(access_list.len() as u8)
                    } else {
                        LengthEncoding::Long(access_list.len())
                    };
                    encoder.encode_bytes(&len_enc.encode())?;
                    
                // Encode each element (in forward order, as per A-XDR array encoding)
                // Each element is optional, so encode flag then value
                for access_opt in access_list.iter() {
                    encoder.encode_bool(access_opt.is_some())?;
                    if let Some(ref access_desc) = access_opt {
                        let access_bytes = access_desc.encode()?;
                        encoder.encode_octet_string(&access_bytes)?;
                    }
                }
                } else {
                    // Encode usage flag: false (array does not exist)
                    encoder.encode_bool(false)?;
                }
                
                // 2. attribute_descriptor_list (required array of CosemAttributeDescriptor)
                // Encode array length
                let len_enc = if attribute_descriptor_list.len() < 128 {
                    LengthEncoding::Short(attribute_descriptor_list.len() as u8)
                } else {
                    LengthEncoding::Long(attribute_descriptor_list.len())
                };
                encoder.encode_bytes(&len_enc.encode())?;
                
                // Encode each element (in forward order, as per A-XDR array encoding)
                for attr_desc in attribute_descriptor_list.iter() {
                    let attr_bytes = attr_desc.encode()?;
                    encoder.encode_octet_string(&attr_bytes)?;
                }
                
                // 3. invoke_id_and_priority (InvokeIdAndPriority)
                let invoke_bytes = invoke_id_and_priority.encode()?;
                encoder.encode_octet_string(&invoke_bytes)?;
                
                // 4. Choice tag (3 = WithList)
                encoder.encode_u8(3)?;
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode choice tag first (A-XDR reverse order)
        let choice_tag = decoder.decode_u8()?;

        match choice_tag {
            1 => {
                // Normal variant
                let normal_bytes = decoder.decode_octet_string()?;
                let normal = GetRequestNormal::decode(&normal_bytes)?;
                Ok(Self::Normal(normal))
            }
            2 => {
                // Next variant
                // Decode in reverse order (A-XDR SEQUENCE convention)
                // Encoding order: tag, block_number, invoke_bytes (SEQUENCE fields in reverse order)
                // Decoding order: tag, then decode fields in reverse of encoding order
                // Since encoding is: block_number, invoke_bytes, decoding should be: invoke_bytes, block_number
                let invoke_bytes = decoder.decode_octet_string()?;
                let invoke_id_and_priority = InvokeIdAndPriority::decode(&invoke_bytes)?;
                let block_number = decoder.decode_u32()?;
                Ok(Self::Next {
                    invoke_id_and_priority,
                    block_number,
                })
            }
            3 => {
                // WithList variant
                // Decode in reverse order (A-XDR SEQUENCE convention)
                // 1. invoke_id_and_priority (InvokeIdAndPriority)
                let invoke_bytes = decoder.decode_octet_string()?;
                let invoke_id_and_priority = InvokeIdAndPriority::decode(&invoke_bytes)?;
                
                // 2. attribute_descriptor_list (required array of CosemAttributeDescriptor)
                // Decode array length: first byte indicates format
                let first_byte: u8 = decoder.decode_u8()?;
                let attr_list_len: usize = if (first_byte & 0x80) == 0 {
                    // Short form: length < 128
                    first_byte as usize
                } else {
                    // Long form: length-of-length byte + length bytes
                    let length_of_length = (first_byte & 0x7F) as usize;
                    if length_of_length == 0 || length_of_length > 4 {
                        return Err(DlmsError::InvalidData(format!(
                            "GetRequest::WithList: Invalid length-of-length: {}",
                            length_of_length
                        )));
                    }
                    let len_bytes = decoder.decode_fixed_bytes(length_of_length)?;
                    let mut len = 0usize;
                    for &byte in len_bytes.iter() {
                        len = (len << 8) | (byte as usize);
                    }
                    len
                };
                
                if attr_list_len == 0 {
                    return Err(DlmsError::InvalidData(
                        "GetRequest::WithList: attribute_descriptor_list cannot be empty".to_string(),
                    ));
                }
                
                // Decode each element (in forward order)
                let mut attribute_descriptor_list = Vec::with_capacity(attr_list_len);
                for _ in 0..attr_list_len {
                    let attr_bytes = decoder.decode_octet_string()?;
                    attribute_descriptor_list.push(CosemAttributeDescriptor::decode(&attr_bytes)?);
                }
                
                // 3. access_selection_list (optional array of optional SelectiveAccessDescriptor)
                // Decode usage flag first
                let has_access_list = decoder.decode_bool()?;
                let access_selection_list = if has_access_list {
                    // Decode array length
                    let first_byte: u8 = decoder.decode_u8()?;
                    let access_list_len: usize = if (first_byte & 0x80) == 0 {
                        // Short form
                        first_byte as usize
                    } else {
                        // Long form
                        let length_of_length = (first_byte & 0x7F) as usize;
                        if length_of_length == 0 || length_of_length > 4 {
                            return Err(DlmsError::InvalidData(format!(
                                "GetRequest::WithList: Invalid length-of-length for access_selection_list: {}",
                                length_of_length
                            )));
                        }
                        let len_bytes = decoder.decode_fixed_bytes(length_of_length)?;
                        let mut len = 0usize;
                        for &byte in len_bytes.iter() {
                            len = (len << 8) | (byte as usize);
                        }
                        len
                    };
                    
                    // Validate length matches attribute_descriptor_list
                    if access_list_len != attribute_descriptor_list.len() {
                        return Err(DlmsError::InvalidData(format!(
                            "GetRequest::WithList: access_selection_list length ({}) does not match attribute_descriptor_list length ({})",
                            access_list_len,
                            attribute_descriptor_list.len()
                        )));
                    }
                    
                    // Decode each element (in forward order)
                    // Each element is optional, so decode flag then value
                    let mut access_list = Vec::with_capacity(access_list_len);
                    for _ in 0..access_list_len {
                        let has_access = decoder.decode_bool()?;
                        let access = if has_access {
                            let access_bytes = decoder.decode_octet_string()?;
                            Some(SelectiveAccessDescriptor::decode(&access_bytes)?)
                        } else {
                            None
                        };
                        access_list.push(access);
                    }
                    
                    Some(access_list)
                } else {
                    None
                };
                
                Ok(Self::WithList {
                    invoke_id_and_priority,
                    attribute_descriptor_list,
                    access_selection_list,
                })
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid GetRequest choice tag: {} (expected 1, 2, or 3)",
                choice_tag
            ))),
        }
    }
}

/// Get Response PDU
///
/// CHOICE type representing different GET response variants:
/// - **Normal**: Single attribute response
/// - **WithDataBlock**: Data block response (for large attributes)
/// - **WithList**: Multiple attribute response
///
/// # Why CHOICE Type?
/// The response type matches the request type. Normal requests get Normal responses,
/// but large attributes may be split into data blocks, requiring WithDataBlock responses.
/// WithList requests get WithList responses.
///
/// # Data Block Handling
/// When an attribute is too large to fit in a single response, the server splits it
/// into blocks. The client must send GetRequest::Next to retrieve subsequent blocks.
#[derive(Debug, Clone, PartialEq)]
pub enum GetResponse {
    /// Single attribute GET response
    Normal(GetResponseNormal),
    /// Data block response (for large attributes)
    ///
    /// # TODO
    /// - [ ] 实现 GetResponseWithDataBlock 结构
    WithDataBlock {
        /// Invoke ID and priority
        invoke_id_and_priority: InvokeIdAndPriority,
        /// Block number
        block_number: u32,
        /// Last block flag
        last_block: bool,
        /// Block data
        block_data: Vec<u8>,
    },
    /// Multiple attribute GET response
    ///
    /// # TODO
    /// - [ ] 实现 GetResponseWithList 结构
    WithList {
        /// Invoke ID and priority
        invoke_id_and_priority: InvokeIdAndPriority,
        /// List of results (one per requested attribute)
        result_list: Vec<GetDataResult>,
    },
}

impl GetResponse {
    /// Create a new Normal GET response
    ///
    /// # Arguments
    /// * `invoke_id_and_priority` - Invoke ID and priority (from request)
    /// * `result` - Get data result
    pub fn new_normal(
        invoke_id_and_priority: InvokeIdAndPriority,
        result: GetDataResult,
    ) -> Self {
        Self::Normal(GetResponseNormal::new(invoke_id_and_priority, result))
    }

    /// Encode to A-XDR format
    ///
    /// Encoding format (A-XDR CHOICE):
    /// 1. Choice tag (Enumerate: 1 = Normal, 2 = WithDataBlock, 3 = WithList)
    /// 2. Choice value (encoded according to variant)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        match self {
            GetResponse::Normal(normal) => {
                // Encode value first (A-XDR reverse order)
                let normal_bytes = normal.encode()?;
                encoder.encode_bytes(&normal_bytes)?;
                // Encode choice tag (1 = Normal)
                encoder.encode_u8(1)?;
            }
            GetResponse::WithDataBlock {
                invoke_id_and_priority,
                block_number,
                last_block,
                block_data,
            } => {
                // Encode value first (A-XDR reverse order)
                encoder.encode_octet_string(block_data)?;
                encoder.encode_bool(*last_block)?;
                encoder.encode_u32(*block_number)?;
                let invoke_bytes = invoke_id_and_priority.encode()?;
                encoder.encode_bytes(&invoke_bytes)?;
                // Encode choice tag (2 = WithDataBlock)
                encoder.encode_u8(2)?;
            }
            GetResponse::WithList {
                invoke_id_and_priority,
                result_list,
            } => {
                // Validate: result_list must not be empty
                if result_list.is_empty() {
                    return Err(DlmsError::InvalidData(
                        "GetResponse::WithList: result_list cannot be empty".to_string(),
                    ));
                }
                
                // Encode in reverse order (A-XDR SEQUENCE convention)
                // 1. result_list (required array of GetDataResult)
                // Encode array length
                let len_enc = if result_list.len() < 128 {
                    LengthEncoding::Short(result_list.len() as u8)
                } else {
                    LengthEncoding::Long(result_list.len())
                };
                encoder.encode_bytes(&len_enc.encode())?;
                
                // Encode each element (in forward order, as per A-XDR array encoding)
                for result in result_list.iter() {
                    let result_bytes = result.encode()?;
                    encoder.encode_bytes(&result_bytes)?;
                }
                
                // 2. invoke_id_and_priority (InvokeIdAndPriority)
                let invoke_bytes = invoke_id_and_priority.encode()?;
                encoder.encode_bytes(&invoke_bytes)?;
                
                // 3. Choice tag (3 = WithList)
                encoder.encode_u8(3)?;
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode choice tag first (A-XDR reverse order)
        let choice_tag = decoder.decode_u8()?;

        match choice_tag {
            1 => {
                // Normal variant
                let normal_bytes = decoder.decode_octet_string()?;
                let normal = GetResponseNormal::decode(&normal_bytes)?;
                Ok(Self::Normal(normal))
            }
            2 => {
                // WithDataBlock variant
                let invoke_bytes = decoder.decode_octet_string()?;
                let invoke_id_and_priority = InvokeIdAndPriority::decode(&invoke_bytes)?;
                let block_number = decoder.decode_u32()?;
                let last_block = decoder.decode_bool()?;
                let block_data = decoder.decode_octet_string()?;
                Ok(Self::WithDataBlock {
                    invoke_id_and_priority,
                    block_number,
                    last_block,
                    block_data,
                })
            }
            3 => {
                // WithList variant
                // Decode in reverse order (A-XDR SEQUENCE convention)
                // 1. invoke_id_and_priority (InvokeIdAndPriority)
                let invoke_bytes = decoder.decode_octet_string()?;
                let invoke_id_and_priority = InvokeIdAndPriority::decode(&invoke_bytes)?;
                
                // 2. result_list (required array of GetDataResult)
                // Decode array length: first byte indicates format
                let first_byte: u8 = decoder.decode_u8()?;
                let result_list_len: usize = if (first_byte & 0x80) == 0 {
                    // Short form: length < 128
                    first_byte as usize
                } else {
                    // Long form: length-of-length byte + length bytes
                    let length_of_length = (first_byte & 0x7F) as usize;
                    if length_of_length == 0 || length_of_length > 4 {
                        return Err(DlmsError::InvalidData(format!(
                            "GetResponse::WithList: Invalid length-of-length: {}",
                            length_of_length
                        )));
                    }
                    let len_bytes = decoder.decode_fixed_bytes(length_of_length)?;
                    let mut len = 0usize;
                    for &byte in len_bytes.iter() {
                        len = (len << 8) | (byte as usize);
                    }
                    len
                };
                
                if result_list_len == 0 {
                    return Err(DlmsError::InvalidData(
                        "GetResponse::WithList: result_list cannot be empty".to_string(),
                    ));
                }
                
                // Decode each element (in forward order)
                let mut result_list = Vec::with_capacity(result_list_len);
                for _ in 0..result_list_len {
                    let result_bytes = decoder.decode_octet_string()?;
                    result_list.push(GetDataResult::decode(&result_bytes)?);
                }
                
                Ok(Self::WithList {
                    invoke_id_and_priority,
                    result_list,
                })
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid GetResponse choice tag: {} (expected 1, 2, or 3)",
                choice_tag
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conformance_new() {
        let conformance = Conformance::new();
        assert_eq!(conformance.bits().num_bits(), 24);
    }

    #[test]
    fn test_conformance_encode_decode() {
        let conformance = Conformance::new();
        let encoded = conformance.encode().unwrap();
        let decoded = Conformance::decode(&encoded).unwrap();
        assert_eq!(conformance, decoded);
    }

    #[test]
    fn test_initiate_request_new() {
        let request = InitiateRequest::new();
        assert_eq!(request.proposed_dlms_version_number, DLMS_VERSION_6);
        assert_eq!(request.response_allowed, true);
        assert_eq!(request.client_max_receive_pdu_size, MAX_PDU_SIZE);
    }

    #[test]
    fn test_initiate_request_encode_decode() {
        let conformance = Conformance::new();
        let request = InitiateRequest::with_params(conformance, 1024).unwrap();
        
        let encoded = request.encode().unwrap();
        let decoded = InitiateRequest::decode(&encoded).unwrap();
        
        assert_eq!(request.proposed_dlms_version_number, decoded.proposed_dlms_version_number);
        assert_eq!(request.client_max_receive_pdu_size, decoded.client_max_receive_pdu_size);
    }

    #[test]
    fn test_initiate_response_encode_decode() {
        let conformance = Conformance::new();
        let response = InitiateResponse::new(
            DLMS_VERSION_6,
            conformance,
            1024,
            0x0007, // Standard VAA name
        ).unwrap();
        
        let encoded = response.encode().unwrap();
        let decoded = InitiateResponse::decode(&encoded).unwrap();
        
        assert_eq!(response.negotiated_dlms_version_number, decoded.negotiated_dlms_version_number);
        assert_eq!(response.server_max_receive_pdu_size, decoded.server_max_receive_pdu_size);
        assert_eq!(response.vaa_name, decoded.vaa_name);
    }

    #[test]
    fn test_invoke_id_and_priority() {
        let invoke = InvokeIdAndPriority::new(1, false).unwrap();
        assert_eq!(invoke.invoke_id(), 1);
        assert_eq!(invoke.is_high_priority(), false);
    }

    #[test]
    fn test_invoke_id_and_priority_encode_decode() {
        let invoke = InvokeIdAndPriority::new(42, true).unwrap();
        let encoded = invoke.encode().unwrap();
        let decoded = InvokeIdAndPriority::decode(&encoded).unwrap();
        assert_eq!(invoke, decoded);
    }

    #[test]
    fn test_cosem_attribute_descriptor_logical_name() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let desc = CosemAttributeDescriptor::new_logical_name(1, obis, 2).unwrap();
        
        match desc {
            CosemAttributeDescriptor::LogicalName(ref ln_ref) => {
                assert_eq!(ln_ref.class_id, 1);
                assert_eq!(ln_ref.id, 2);
            }
            _ => panic!("Expected LogicalName variant"),
        }
    }

    #[test]
    fn test_cosem_attribute_descriptor_encode_decode() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let desc = CosemAttributeDescriptor::new_logical_name(1, obis, 2).unwrap();
        
        let encoded = desc.encode().unwrap();
        let decoded = CosemAttributeDescriptor::decode(&encoded).unwrap();
        
        assert_eq!(desc, decoded);
    }

    #[test]
    fn test_get_request_normal_encode_decode() {
        let invoke = InvokeIdAndPriority::new(1, false).unwrap();
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let attr_desc = CosemAttributeDescriptor::new_logical_name(1, obis, 2).unwrap();
        
        let request = GetRequest::new_normal(invoke, attr_desc, None);
        let encoded = request.encode().unwrap();
        let decoded = GetRequest::decode(&encoded).unwrap();
        
        match (&request, &decoded) {
            (GetRequest::Normal(req), GetRequest::Normal(dec)) => {
                assert_eq!(req.invoke_id_and_priority, dec.invoke_id_and_priority);
                assert_eq!(req.cosem_attribute_descriptor, dec.cosem_attribute_descriptor);
            }
            _ => panic!("Expected Normal variants"),
        }
    }

    #[test]
    fn test_get_response_normal_encode_decode() {
        let invoke = InvokeIdAndPriority::new(1, false).unwrap();
        let data = DataObject::new_unsigned32(12345);
        let result = GetDataResult::new_data(data);
        
        let response = GetResponse::new_normal(invoke, result);
        let encoded = response.encode().unwrap();
        let decoded = GetResponse::decode(&encoded).unwrap();
        
        match (&response, &decoded) {
            (GetResponse::Normal(resp), GetResponse::Normal(dec)) => {
                assert_eq!(resp.invoke_id_and_priority, dec.invoke_id_and_priority);
                assert_eq!(resp.result.is_success(), dec.result.is_success());
            }
            _ => panic!("Expected Normal variants"),
        }
    }

    #[test]
    fn test_get_request_with_list_encode_decode() {
        let invoke = InvokeIdAndPriority::new(1, false).unwrap();
        let obis1 = ObisCode::new(1, 1, 1, 8, 0, 255);
        let obis2 = ObisCode::new(1, 1, 2, 8, 0, 255);
        let attr_desc1 = CosemAttributeDescriptor::new_logical_name(1, obis1, 2).unwrap();
        let attr_desc2 = CosemAttributeDescriptor::new_logical_name(1, obis2, 2).unwrap();
        
        let attribute_descriptor_list = vec![attr_desc1.clone(), attr_desc2.clone()];
        
        // Test without access_selection_list
        let request = GetRequest::WithList {
            invoke_id_and_priority: invoke.clone(),
            attribute_descriptor_list: attribute_descriptor_list.clone(),
            access_selection_list: None,
        };
        
        let encoded = request.encode().unwrap();
        let decoded = GetRequest::decode(&encoded).unwrap();
        
        match (&request, &decoded) {
            (GetRequest::WithList { invoke_id_and_priority: inv1, attribute_descriptor_list: attrs1, access_selection_list: access1 },
             GetRequest::WithList { invoke_id_and_priority: inv2, attribute_descriptor_list: attrs2, access_selection_list: access2 }) => {
                assert_eq!(inv1, inv2);
                assert_eq!(attrs1.len(), attrs2.len());
                assert_eq!(attrs1[0], attrs2[0]);
                assert_eq!(attrs1[1], attrs2[1]);
                assert_eq!(access1, access2);
            }
            _ => panic!("Expected WithList variants"),
        }
    }

    #[test]
    fn test_get_request_with_list_with_access_selection() {
        let invoke = InvokeIdAndPriority::new(1, false).unwrap();
        let obis1 = ObisCode::new(1, 1, 1, 8, 0, 255);
        let attr_desc1 = CosemAttributeDescriptor::new_logical_name(1, obis1, 2).unwrap();
        
        let access_selector = SelectiveAccessDescriptor::new(
            0, // Entry index
            DataObject::new_structure(vec![
                DataObject::new_unsigned32(0), // start_index
                DataObject::new_unsigned32(10), // count
            ]),
        );
        
        let attribute_descriptor_list = vec![attr_desc1.clone()];
        let access_selection_list = Some(vec![Some(access_selector.clone())]);
        
        let request = GetRequest::WithList {
            invoke_id_and_priority: invoke.clone(),
            attribute_descriptor_list,
            access_selection_list: access_selection_list.clone(),
        };
        
        let encoded = request.encode().unwrap();
        let decoded = GetRequest::decode(&encoded).unwrap();
        
        match (&request, &decoded) {
            (GetRequest::WithList { invoke_id_and_priority: inv1, attribute_descriptor_list: attrs1, access_selection_list: access1 }, 
            GetRequest::WithList { invoke_id_and_priority: inv2, attribute_descriptor_list: attrs2, access_selection_list: access2 }) => {
                assert_eq!(inv1, inv2);
                assert_eq!(attrs1.len(), attrs2.len());
                assert_eq!(attrs1[0], attrs2[0]);
                assert_eq!(access1.is_some(), access2.is_some());
                if let (Some(a1), Some(a2)) = (access1, access2) {
                    assert_eq!(a1.len(), a2.len());
                    assert_eq!(a1[0].is_some(), a2[0].is_some());
                }
            }
            _ => panic!("Expected WithList variants"),
        }
    }

    #[test]
    fn test_get_response_with_list_encode_decode() {
        let invoke = InvokeIdAndPriority::new(1, false).unwrap();
        let data1 = DataObject::new_unsigned32(12345);
        let data2 = DataObject::new_unsigned32(67890);
        let result1 = GetDataResult::new_data(data1);
        let result2 = GetDataResult::new_data(data2);
        
        let result_list = vec![result1.clone(), result2.clone()];
        
        let response = GetResponse::WithList {
            invoke_id_and_priority: invoke.clone(),
            result_list: result_list.clone(),
        };
        
        let encoded = response.encode().unwrap();
        let decoded = GetResponse::decode(&encoded).unwrap();
        
        match (&response, &decoded) {
            (GetResponse::WithList { invoke_id_and_priority: inv1, result_list: results1 },
            GetResponse::WithList { invoke_id_and_priority: inv2, result_list: results2 }) => {
                assert_eq!(inv1, inv2);
                assert_eq!(results1.len(), results2.len());
                assert_eq!(results1[0].is_success(), results2[0].is_success());
                assert_eq!(results1[1].is_success(), results2[1].is_success());
            }
            _ => panic!("Expected WithList variants"),
        }
    }

    #[test]
    fn test_get_response_with_list_mixed_results() {
        let invoke = InvokeIdAndPriority::new(1, false).unwrap();
        let data1 = DataObject::new_unsigned32(12345);
        let result1 = GetDataResult::new_data(data1);
        let result2 = GetDataResult::new_error(4); // Object undefined
        
        let result_list = vec![result1.clone(), result2.clone()];
        
        let response = GetResponse::WithList {
            invoke_id_and_priority: invoke.clone(),
            result_list: result_list.clone(),
        };
        
        let encoded = response.encode().unwrap();
        let decoded = GetResponse::decode(&encoded).unwrap();
        
        match (&response, &decoded) {
            (GetResponse::WithList { invoke_id_and_priority: inv1, result_list: results1 },
            GetResponse::WithList { invoke_id_and_priority: inv2, result_list: results2 }) => {
                assert_eq!(inv1, inv2);
                assert_eq!(results1.len(), results2.len());
                assert_eq!(results1[0].is_success(), results2[0].is_success());
                assert_eq!(results1[1].is_success(), results2[1].is_success());
                assert_eq!(results1[1].error_code(), results2[1].error_code());
            }
            _ => panic!("Expected WithList variants"),
        }
    }

    #[test]
    fn test_get_request_with_list_empty_list_error() {
        let invoke = InvokeIdAndPriority::new(1, false).unwrap();
        let request = GetRequest::WithList {
            invoke_id_and_priority: invoke,
            attribute_descriptor_list: vec![],
            access_selection_list: None,
        };
        
        let result = request.encode();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_get_response_with_list_empty_list_error() {
        let invoke = InvokeIdAndPriority::new(1, false).unwrap();
        let response = GetResponse::WithList {
            invoke_id_and_priority: invoke,
            result_list: vec![],
        };
        
        let result = response.encode();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_get_request_next_encode_decode() {
        let invoke = InvokeIdAndPriority::new(1, false).unwrap();
        let block_number = 5u32;
        
        let request = GetRequest::Next {
            invoke_id_and_priority: invoke.clone(),
            block_number,
        };
        
        let encoded = request.encode().unwrap();
        let decoded = GetRequest::decode(&encoded).unwrap();
        
        match (&request, &decoded) {
            (GetRequest::Next { invoke_id_and_priority: inv1, block_number: bn1 },
            GetRequest::Next { invoke_id_and_priority: inv2, block_number: bn2 }) => {
                assert_eq!(inv1, inv2);
                assert_eq!(bn1, bn2);
            }
            _ => panic!("Expected Next variants"),
        }
    }

    #[test]
    fn test_get_response_with_data_block_encode_decode() {
        let invoke = InvokeIdAndPriority::new(1, false).unwrap();
        let block_number = 5u32;
        let last_block = false;
        let block_data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        
        let response = GetResponse::WithDataBlock {
            invoke_id_and_priority: invoke.clone(),
            block_number,
            last_block,
            block_data: block_data.clone(),
        };
        
        let encoded = response.encode().unwrap();
        let decoded = GetResponse::decode(&encoded).unwrap();
        
        match (&response, &decoded) {
            (GetResponse::WithDataBlock { invoke_id_and_priority: inv1, block_number: bn1, last_block: lb1, block_data: bd1 },
            GetResponse::WithDataBlock { invoke_id_and_priority: inv2, block_number: bn2, last_block: lb2, block_data: bd2 }) => {
                assert_eq!(inv1, inv2);
                assert_eq!(bn1, bn2);
                assert_eq!(lb1, lb2);
                assert_eq!(bd1, bd2);
            }
            _ => panic!("Expected WithDataBlock variants"),
        }
    }

    #[test]
    fn test_get_response_with_data_block_last_block() {
        let invoke = InvokeIdAndPriority::new(1, false).unwrap();
        let block_number = 10u32;
        let last_block = true;
        let block_data = vec![0xFF, 0xFE, 0xFD];
        
        let response = GetResponse::WithDataBlock {
            invoke_id_and_priority: invoke.clone(),
            block_number,
            last_block,
            block_data: block_data.clone(),
        };
        
        let encoded = response.encode().unwrap();
        let decoded = GetResponse::decode(&encoded).unwrap();
        
        match decoded {
            GetResponse::WithDataBlock { invoke_id_and_priority: _, block_number: bn, last_block: lb, block_data: bd } => {
                assert_eq!(bn, block_number);
                assert_eq!(lb, last_block);
                assert_eq!(bd, block_data);
            }
            _ => panic!("Expected WithDataBlock variant"),
        }
    }
}

// ============================================================================
// Set Request/Response PDU Implementation
// ============================================================================

/// Set Data Result
///
/// Result of a SET operation. Can be either:
/// - **Success**: Operation completed successfully (no data returned)
/// - **DataAccessResult**: Error code indicating why the access failed
///
/// # Why This Design?
/// SET operations typically don't return data on success, only error codes on failure.
/// This CHOICE type allows representing both success and failure cases in a type-safe manner.
///
/// # Optimization Considerations
/// - Using an enum instead of separate success/error fields reduces memory overhead
/// - The error code is a simple u8, avoiding unnecessary allocations
/// - Future optimization: Consider using a custom error type with more context
#[derive(Debug, Clone, PartialEq)]
pub enum SetDataResult {
    /// Operation succeeded
    Success,
    /// Data access error code
    DataAccessResult(u8),
}

impl SetDataResult {
    /// Create a new SetDataResult with success
    pub fn new_success() -> Self {
        Self::Success
    }

    /// Create a new SetDataResult with error code
    pub fn new_error(code: u8) -> Self {
        Self::DataAccessResult(code)
    }

    /// Create a new SetDataResult with a standard error code
    ///
    /// # Arguments
    /// * `code` - One of the constants from `data_access_result` module
    ///
    /// # Example
    /// ```rust,no_run
    /// use dlms_application::pdu::{SetDataResult, data_access_result};
    /// let result = SetDataResult::new_standard_error(data_access_result::HARDWARE_FAULT);
    /// ```
    pub fn new_standard_error(code: u8) -> Self {
        Self::DataAccessResult(code)
    }

    /// Check if this is a success result
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// Get the error code if this is an error result
    pub fn error_code(&self) -> Option<u8> {
        match self {
            Self::DataAccessResult(code) => Some(*code),
            _ => None,
        }
    }

    /// Get a human-readable description of the error code
    ///
    /// # Returns
    /// A string describing the error, or "Unknown error code" if the code is not recognized
    pub fn error_description(&self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::DataAccessResult(code) => match *code {
                data_access_result::SUCCESS => "Success",
                data_access_result::HARDWARE_FAULT => "Hardware fault",
                data_access_result::TEMPORARY_FAILURE => "Temporary failure",
                data_access_result::READ_WRITE_DENIED => "Read-write denied",
                data_access_result::OBJECT_UNDEFINED => "Object undefined",
                data_access_result::OBJECT_CLASS_INCONSISTENT => "Object class inconsistent",
                data_access_result::OBJECT_UNAVAILABLE => "Object unavailable",
                data_access_result::TYPE_UNMATCHED => "Type unmatched",
                data_access_result::SCOPE_OF_ACCESS_VIOLATED => "Scope of access violated",
                data_access_result::DATA_BLOCK_UNAVAILABLE => "Data block unavailable",
                data_access_result::LONG_GET_ABORTED => "Long GET aborted",
                data_access_result::NO_LONG_GET_IN_PROGRESS => "No long GET in progress",
                data_access_result::LONG_SET_ABORTED => "Long SET aborted",
                data_access_result::NO_LONG_SET_IN_PROGRESS => "No long SET in progress",
                data_access_result::DATA_BLOCK_NUMBER_INVALID => "Data block number invalid",
                data_access_result::OTHER_REASON => "Other reason",
                data_access_result::NOT_SET => "Not set",
                _ => "Unknown error code",
            },
        }
    }

    /// Encode to A-XDR format
    ///
    /// Encoding format (A-XDR CHOICE):
    /// - Success: tag 0 (no value)
    /// - DataAccessResult: tag 1 + error code (Unsigned8)
    ///
    /// # A-XDR CHOICE Encoding Order
    /// According to A-XDR standard, CHOICE types are encoded as: tag + value.
    /// The tag comes first to identify which variant is present, followed by the value.
    ///
    /// # Why This Order?
    /// - **Tag First**: Allows the decoder to know which variant to expect before reading the value
    /// - **Standard Compliance**: Matches A-XDR standard specification
    /// - **Consistency**: Matches the decode order (tag first, then value)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        match self {
            SetDataResult::Success => {
                // Encode choice tag (0 = Success)
                // Success variant has no value, only the tag
                encoder.encode_u8(0)?;
            }
            SetDataResult::DataAccessResult(code) => {
                // Encode choice tag first (1 = DataAccessResult)
                encoder.encode_u8(1)?;
                // Encode value after tag
                encoder.encode_u8(*code)?;
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    ///
    /// # A-XDR CHOICE Decoding Order
    /// Decodes in the same order as encoding: tag first, then value.
    /// This matches the A-XDR standard and ensures roundtrip compatibility.
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode choice tag first
        let choice_tag = decoder.decode_u8()?;

        match choice_tag {
            0 => Ok(Self::Success),
            1 => {
                // DataAccessResult variant: decode value after tag
                let code = decoder.decode_u8()?;
                Ok(Self::DataAccessResult(code))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid SetDataResult choice tag: {} (expected 0 or 1)",
                choice_tag
            ))),
        }
    }
}

/// COSEM Method Descriptor
///
/// Describes a method to be invoked on a COSEM object. Similar to `CosemAttributeDescriptor`
/// but for method calls instead of attribute access.
///
/// # Structure
/// - `class_id`: COSEM interface class identifier (Unsigned16)
/// - `instance_id`: Object instance identifier (OBIS code for LN, base name for SN)
/// - `method_id`: Method identifier within the class (Unsigned8)
///
/// # Addressing Methods
/// Supports both Logical Name (LN) and Short Name (SN) addressing, similar to
/// `CosemAttributeDescriptor`. The addressing method is determined by the instance_id length
/// (6 bytes for LN, 2 bytes for SN).
///
/// # Why Enum for Addressing?
/// Using an enum (`LogicalName` vs `ShortName`) provides compile-time type safety and
/// prevents mixing addressing methods. This is more robust than using a single struct
/// with a flag.
///
/// # Optimization Considerations
/// - Method descriptors are typically created once and reused, so cloning overhead is minimal
/// - Future optimization: Consider caching encoded descriptors for frequently used methods
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CosemMethodDescriptor {
    /// Logical Name addressing
    LogicalName(LogicalNameReference),
    /// Short Name addressing
    ShortName(ShortNameReference),
}

impl CosemMethodDescriptor {
    /// Create a new method descriptor using Logical Name addressing
    ///
    /// # Arguments
    /// * `class_id` - COSEM interface class ID
    /// * `instance_id` - OBIS code (6 bytes)
    /// * `method_id` - Method ID within the class
    pub fn new_logical_name(
        class_id: u16,
        instance_id: ObisCode,
        method_id: u8,
    ) -> DlmsResult<Self> {
        Ok(Self::LogicalName(LogicalNameReference::new(
            class_id,
            instance_id,
            method_id,
        )?))
    }

    /// Create a new method descriptor using Short Name addressing
    ///
    /// # Arguments
    /// * `base_name` - Base name (16-bit address)
    /// * `method_id` - Method ID within the class
    pub fn new_short_name(base_name: u16, method_id: u8) -> DlmsResult<Self> {
        Ok(Self::ShortName(ShortNameReference::new(base_name, method_id)?))
    }

    /// Encode to A-XDR format
    ///
    /// Encoding format (A-XDR, reverse order):
    /// 1. method_id (Integer8)
    /// 2. instance_id (OctetString, 6 bytes for LN, 2 bytes for SN)
    /// 3. class_id (Unsigned16)
    ///
    /// # Why This Order?
    /// A-XDR encodes SEQUENCE fields in reverse order (last field first) for efficiency.
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        match self {
            CosemMethodDescriptor::LogicalName(ref ln_ref) => {
                // Encode in reverse order
                // 1. method_id (Integer8)
                encoder.encode_i8(ln_ref.id as i8)?;

                // 2. instance_id (OctetString, 6 bytes for OBIS code)
                let obis_bytes = ln_ref.instance_id.as_bytes();
                encoder.encode_octet_string(obis_bytes)?;

                // 3. class_id (Unsigned16)
                encoder.encode_u16(ln_ref.class_id)?;
            }
            CosemMethodDescriptor::ShortName(ref sn_ref) => {
                // Encode in reverse order
                // 1. method_id (Integer8)
                encoder.encode_i8(sn_ref.id as i8)?;

                // 2. instance_id (OctetString, 2 bytes for base name)
                encoder.encode_octet_string(&sn_ref.base_name.to_be_bytes())?;

                // 3. class_id (Unsigned16)
                // Note: For SN addressing, class_id is typically 0 or not used
                encoder.encode_u16(0)?;
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    ///
    /// # Decoding Strategy
    /// The decoder determines the addressing method by checking the instance_id length:
    /// - 6 bytes: Logical Name addressing
    /// - 2 bytes: Short Name addressing
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode in reverse order
        // 1. class_id (Unsigned16)
        let class_id = decoder.decode_u16()?;

        // 2. instance_id (OctetString)
        let instance_bytes = decoder.decode_octet_string()?;

        // 3. method_id (Integer8)
        // Note: decode_i8 returns i8, but method_id is u8. We cast the signed value to unsigned.
        // This is safe because method IDs are always positive values (0-255 range).
        let method_id_i8: i8 = decoder.decode_i8()?;
        let method_id: u8 = method_id_i8 as u8;

        // Determine addressing method by instance_id length
        match instance_bytes.len() {
            6 => {
                // Logical Name addressing
                let instance_id = ObisCode::new(
                    instance_bytes[0],
                    instance_bytes[1],
                    instance_bytes[2],
                    instance_bytes[3],
                    instance_bytes[4],
                    instance_bytes[5],
                );
                Ok(Self::LogicalName(LogicalNameReference::new(
                    class_id,
                    instance_id,
                    method_id,
                )?))
            }
            2 => {
                // Short Name addressing
                let base_name = u16::from_be_bytes([instance_bytes[0], instance_bytes[1]]);
                Ok(Self::ShortName(ShortNameReference::new(base_name, method_id)?))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid instance_id length: expected 2 or 6 bytes, got {}",
                instance_bytes.len()
            ))),
        }
    }
}

/// Set Request Normal
///
/// Single attribute SET request. This is the most common SET request type.
///
/// # Structure
/// - `invoke_id_and_priority`: Invoke ID and priority
/// - `cosem_attribute_descriptor`: Attribute to write
/// - `access_selection`: Optional selective access descriptor
/// - `value`: Data value to write (DataObject)
///
/// # Why Separate from GetRequest?
/// SET operations require a value to write, which GET operations don't need. Separating
/// these into distinct types provides better type safety and clearer API semantics.
///
/// # Optimization Considerations
/// - The `value` field is a `DataObject`, which may contain large data. Consider using
///   `Bytes` or `BytesMut` for zero-copy operations in high-frequency scenarios.
/// - Selective access is optional, so we use `Option` to avoid unnecessary allocations
///   when not needed.
#[derive(Debug, Clone, PartialEq)]
pub struct SetRequestNormal {
    /// Invoke ID and priority
    pub invoke_id_and_priority: InvokeIdAndPriority,
    /// Attribute descriptor
    pub cosem_attribute_descriptor: CosemAttributeDescriptor,
    /// Optional selective access descriptor
    pub access_selection: Option<SelectiveAccessDescriptor>,
    /// Value to write
    pub value: DataObject,
}

impl SetRequestNormal {
    /// Create a new SetRequestNormal
    pub fn new(
        invoke_id_and_priority: InvokeIdAndPriority,
        cosem_attribute_descriptor: CosemAttributeDescriptor,
        access_selection: Option<SelectiveAccessDescriptor>,
        value: DataObject,
    ) -> Self {
        Self {
            invoke_id_and_priority,
            cosem_attribute_descriptor,
            access_selection,
            value,
        }
    }

    /// Encode to A-XDR format
    ///
    /// Encoding order (A-XDR, reverse order):
    /// 1. value (DataObject)
    /// 2. access_selection (optional SelectiveAccessDescriptor)
    /// 3. cosem_attribute_descriptor (CosemAttributeDescriptor)
    /// 4. invoke_id_and_priority (InvokeIdAndPriority)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Encode in reverse order
        // 1. value (DataObject)
        encoder.encode_data_object(&self.value)?;

        // 2. access_selection (optional SelectiveAccessDescriptor)
        encoder.encode_bool(self.access_selection.is_some())?;
        if let Some(ref access) = self.access_selection {
            let access_bytes = access.encode()?;
            encoder.encode_bytes(&access_bytes)?;
        }

        // 3. cosem_attribute_descriptor (CosemAttributeDescriptor)
        let attr_bytes = self.cosem_attribute_descriptor.encode()?;
        encoder.encode_bytes(&attr_bytes)?;

        // 4. invoke_id_and_priority (InvokeIdAndPriority)
        let invoke_bytes = self.invoke_id_and_priority.encode()?;
        encoder.encode_bytes(&invoke_bytes)?;

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode in reverse order
        // 1. invoke_id_and_priority (InvokeIdAndPriority)
        let invoke_bytes = decoder.decode_octet_string()?;
        let invoke_id_and_priority = InvokeIdAndPriority::decode(&invoke_bytes)?;

        // 2. cosem_attribute_descriptor (CosemAttributeDescriptor)
        let attr_bytes = decoder.decode_octet_string()?;
        let cosem_attribute_descriptor = CosemAttributeDescriptor::decode(&attr_bytes)?;

        // 3. access_selection (optional SelectiveAccessDescriptor)
        let has_access = decoder.decode_bool()?;
        let access_selection = if has_access {
            let access_bytes = decoder.decode_octet_string()?;
            Some(SelectiveAccessDescriptor::decode(&access_bytes)?)
        } else {
            None
        };

        // 4. value (DataObject)
        let value = decoder.decode_data_object()?;

        Ok(Self {
            invoke_id_and_priority,
            cosem_attribute_descriptor,
            access_selection,
            value,
        })
    }
}

/// Set Response Normal
///
/// Single attribute SET response. Contains the result of a SetRequestNormal.
///
/// # Why Simpler than GetResponse?
/// SET operations typically don't return data on success, only error codes. This makes
/// the response structure simpler than GET responses, which need to return actual data.
///
/// # Optimization Considerations
/// - The result is a simple enum, minimizing memory overhead
/// - Error codes are encoded as single bytes, keeping the response compact
#[derive(Debug, Clone, PartialEq)]
pub struct SetResponseNormal {
    /// Invoke ID and priority
    pub invoke_id_and_priority: InvokeIdAndPriority,
    /// Result of the SET operation
    pub result: SetDataResult,
}

impl SetResponseNormal {
    /// Create a new SetResponseNormal
    pub fn new(invoke_id_and_priority: InvokeIdAndPriority, result: SetDataResult) -> Self {
        Self {
            invoke_id_and_priority,
            result,
        }
    }

    /// Encode to A-XDR format
    ///
    /// Encoding order (A-XDR, reverse order):
    /// 1. result (SetDataResult)
    /// 2. invoke_id_and_priority (InvokeIdAndPriority)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Encode in reverse order
        // 1. result (SetDataResult)
        let result_bytes = self.result.encode()?;
        encoder.encode_bytes(&result_bytes)?;

        // 2. invoke_id_and_priority (InvokeIdAndPriority)
        let invoke_bytes = self.invoke_id_and_priority.encode()?;
        encoder.encode_bytes(&invoke_bytes)?;

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode in reverse order
        // 1. invoke_id_and_priority (InvokeIdAndPriority)
        let invoke_bytes = decoder.decode_octet_string()?;
        let invoke_id_and_priority = InvokeIdAndPriority::decode(&invoke_bytes)?;

        // 2. result (SetDataResult)
        let result_bytes = decoder.decode_octet_string()?;
        let result = SetDataResult::decode(&result_bytes)?;

        Ok(Self {
            invoke_id_and_priority,
            result,
        })
    }
}

/// Set Request PDU
///
/// CHOICE type representing different SET request variants:
/// - **Normal**: Single attribute SET request
/// - **WithFirstDataBlock**: First data block SET request (for large values)
/// - **WithDataBlock**: Continue data block SET request
/// - **WithList**: Multiple attribute SET request
///
/// # Why CHOICE Type?
/// Different SET scenarios require different request structures. Using a CHOICE type
/// allows the protocol to handle both simple single-attribute writes and complex
/// multi-attribute or large-value writes efficiently.
///
/// # Current Implementation Status
/// Currently only the `Normal` variant is implemented. Other variants (WithDataBlock,
/// WithList) are planned for future implementation to support large data transfers
/// and batch operations.
#[derive(Debug, Clone, PartialEq)]
pub enum SetRequest {
    /// Single attribute SET request
    Normal(SetRequestNormal),
    // TODO: Implement other variants
    // WithFirstDataBlock { ... },
    // WithDataBlock { ... },
    // WithList { ... },
}

impl SetRequest {
    /// Create a new Normal SET request
    pub fn new_normal(
        invoke_id_and_priority: InvokeIdAndPriority,
        cosem_attribute_descriptor: CosemAttributeDescriptor,
        access_selection: Option<SelectiveAccessDescriptor>,
        value: DataObject,
    ) -> Self {
        Self::Normal(SetRequestNormal::new(
            invoke_id_and_priority,
            cosem_attribute_descriptor,
            access_selection,
            value,
        ))
    }

    /// Encode to A-XDR format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        match self {
            SetRequest::Normal(normal) => {
                // Encode value first (A-XDR reverse order)
                let normal_bytes = normal.encode()?;
                encoder.encode_bytes(&normal_bytes)?;
                // Encode choice tag (1 = Normal)
                encoder.encode_u8(1)?;
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode choice tag first (A-XDR reverse order)
        let choice_tag = decoder.decode_u8()?;

        match choice_tag {
            1 => {
                // Normal variant
                let normal_bytes = decoder.decode_octet_string()?;
                let normal = SetRequestNormal::decode(&normal_bytes)?;
                Ok(Self::Normal(normal))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid SetRequest choice tag: {} (expected 1)",
                choice_tag
            ))),
        }
    }
}

/// Set Response PDU
///
/// CHOICE type representing different SET response variants:
/// - **Normal**: Single attribute SET response
/// - **WithDataBlock**: Data block SET response
/// - **WithList**: Multiple attribute SET response
#[derive(Debug, Clone, PartialEq)]
pub enum SetResponse {
    /// Single attribute SET response
    Normal(SetResponseNormal),
    // TODO: Implement other variants
    // WithDataBlock { ... },
    // WithList { ... },
}

impl SetResponse {
    /// Create a new Normal SET response
    pub fn new_normal(invoke_id_and_priority: InvokeIdAndPriority, result: SetDataResult) -> Self {
        Self::Normal(SetResponseNormal::new(invoke_id_and_priority, result))
    }

    /// Encode to A-XDR format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        match self {
            SetResponse::Normal(normal) => {
                // Encode value first (A-XDR reverse order)
                let normal_bytes = normal.encode()?;
                encoder.encode_bytes(&normal_bytes)?;
                // Encode choice tag (1 = Normal)
                encoder.encode_u8(1)?;
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode choice tag first (A-XDR reverse order)
        let choice_tag = decoder.decode_u8()?;

        match choice_tag {
            1 => {
                // Normal variant
                let normal_bytes = decoder.decode_octet_string()?;
                let normal = SetResponseNormal::decode(&normal_bytes)?;
                Ok(Self::Normal(normal))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid SetResponse choice tag: {} (expected 1)",
                choice_tag
            ))),
        }
    }
}

// ============================================================================
// Action Request/Response PDU Implementation
// ============================================================================

/// Action Result
///
/// Result of an ACTION operation. Can be either:
/// - **Success with data**: Operation completed successfully and returned data
/// - **Success without data**: Operation completed successfully (no data returned)
/// - **DataAccessResult**: Error code indicating why the access failed
///
/// # Why This Design?
/// ACTION operations can return data (unlike SET operations), so we need to support
/// both success with data and success without data cases. This three-way CHOICE
/// provides clear semantics for all possible outcomes.
///
/// # Optimization Considerations
/// - The `SuccessWithData` variant contains a `DataObject`, which may be large.
///   Consider using `Arc<DataObject>` or `Bytes` for zero-copy sharing if the
///   result is used in multiple places.
/// - Error codes are simple u8 values, keeping the error case lightweight
#[derive(Debug, Clone, PartialEq)]
pub enum ActionResult {
    /// Operation succeeded with returned data
    SuccessWithData(DataObject),
    /// Operation succeeded without data
    Success,
    /// Data access error code
    DataAccessResult(u8),
}

impl ActionResult {
    /// Create a new ActionResult with success and data
    pub fn new_success_with_data(data: DataObject) -> Self {
        Self::SuccessWithData(data)
    }

    /// Create a new ActionResult with success (no data)
    pub fn new_success() -> Self {
        Self::Success
    }

    /// Create a new ActionResult with error code
    pub fn new_error(code: u8) -> Self {
        Self::DataAccessResult(code)
    }

    /// Create a new ActionResult with a standard error code
    ///
    /// # Arguments
    /// * `code` - One of the constants from `action_result` module
    ///
    /// # Example
    /// ```rust,no_run
    /// use dlms_application::pdu::{ActionResult, action_result};
    /// let result = ActionResult::new_standard_error(action_result::HARDWARE_FAULT);
    /// ```
    pub fn new_standard_error(code: u8) -> Self {
        Self::DataAccessResult(code)
    }

    /// Check if this is a success result
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success | Self::SuccessWithData(_))
    }

    /// Get the error code if this is an error result
    pub fn error_code(&self) -> Option<u8> {
        match self {
            Self::DataAccessResult(code) => Some(*code),
            _ => None,
        }
    }

    /// Get the data if this is a success result with data
    pub fn data(&self) -> Option<&DataObject> {
        match self {
            Self::SuccessWithData(data) => Some(data),
            _ => None,
        }
    }

    /// Get a human-readable description of the error code
    ///
    /// # Returns
    /// A string describing the error, or "Unknown error code" if the code is not recognized
    pub fn error_description(&self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::SuccessWithData(_) => "Success with data",
            Self::DataAccessResult(code) => match *code {
                action_result::SUCCESS => "Success",
                action_result::HARDWARE_FAULT => "Hardware fault",
                action_result::TEMPORARY_FAILURE => "Temporary failure",
                action_result::READ_WRITE_DENIED => "Read-write denied",
                action_result::OBJECT_UNDEFINED => "Object undefined",
                action_result::OBJECT_CLASS_INCONSISTENT => "Object class inconsistent",
                action_result::OBJECT_UNAVAILABLE => "Object unavailable",
                action_result::TYPE_UNMATCHED => "Type unmatched",
                action_result::SCOPE_OF_ACCESS_VIOLATED => "Scope of access violated",
                action_result::DATA_BLOCK_UNAVAILABLE => "Data block unavailable",
                action_result::LONG_ACTION_ABORTED => "Long ACTION aborted",
                action_result::NO_LONG_ACTION_IN_PROGRESS => "No long ACTION in progress",
                action_result::OTHER_REASON => "Other reason",
                action_result::NOT_SET => "Not set",
                _ => "Unknown error code",
            },
        }
    }

    /// Encode to A-XDR format
    ///
    /// Encoding format (A-XDR CHOICE):
    /// - Success: tag 0 (no value)
    /// - SuccessWithData: tag 1 + DataObject
    /// - DataAccessResult: tag 2 + error code (Unsigned8)
    ///
    /// # A-XDR CHOICE Encoding Order
    /// According to A-XDR standard, CHOICE types are encoded as: tag + value.
    /// The tag comes first to identify which variant is present, followed by the value.
    ///
    /// # Why This Order?
    /// - **Tag First**: Allows the decoder to know which variant to expect before reading the value
    /// - **Standard Compliance**: Matches A-XDR standard specification
    /// - **Consistency**: Matches the decode order (tag first, then value)
    /// - **Roundtrip Compatibility**: Ensures encode/decode roundtrip works correctly
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        match self {
            ActionResult::Success => {
                // Encode choice tag (0 = Success)
                // Success variant has no value, only the tag
                encoder.encode_u8(0)?;
            }
            ActionResult::SuccessWithData(data) => {
                // Encode choice tag first (1 = SuccessWithData)
                encoder.encode_u8(1)?;
                // Encode value after tag
                encoder.encode_data_object(data)?;
            }
            ActionResult::DataAccessResult(code) => {
                // Encode choice tag first (2 = DataAccessResult)
                encoder.encode_u8(2)?;
                // Encode value after tag
                encoder.encode_u8(*code)?;
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    ///
    /// # A-XDR CHOICE Decoding Order
    /// Decodes in the same order as encoding: tag first, then value.
    /// This matches the A-XDR standard and ensures roundtrip compatibility.
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode choice tag first
        let choice_tag = decoder.decode_u8()?;

        match choice_tag {
            0 => Ok(Self::Success),
            1 => {
                // SuccessWithData variant: decode value after tag
                let data_obj = decoder.decode_data_object()?;
                Ok(Self::SuccessWithData(data_obj))
            }
            2 => {
                // DataAccessResult variant: decode value after tag
                let code = decoder.decode_u8()?;
                Ok(Self::DataAccessResult(code))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid ActionResult choice tag: {} (expected 0, 1, or 2)",
                choice_tag
            ))),
        }
    }
}

/// Action Request Normal
///
/// Single method ACTION request. This is the most common ACTION request type.
///
/// # Structure
/// - `invoke_id_and_priority`: Invoke ID and priority
/// - `cosem_method_descriptor`: Method to invoke
/// - `method_invocation_parameters`: Optional method parameters (DataObject)
///
/// # Why Optional Parameters?
/// Not all methods require parameters. Making parameters optional allows the protocol
/// to efficiently handle both parameterized and non-parameterized method calls.
///
/// # Optimization Considerations
/// - Method parameters are encoded as `DataObject`, which provides flexibility but
///   may have encoding overhead. For high-frequency operations, consider caching
///   encoded parameter representations.
/// - The descriptor is cloned during encoding, but this is typically acceptable
///   as ACTION requests are less frequent than GET/SET operations.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionRequestNormal {
    /// Invoke ID and priority
    pub invoke_id_and_priority: InvokeIdAndPriority,
    /// Method descriptor
    pub cosem_method_descriptor: CosemMethodDescriptor,
    /// Optional method invocation parameters
    pub method_invocation_parameters: Option<DataObject>,
}

impl ActionRequestNormal {
    /// Create a new ActionRequestNormal
    pub fn new(
        invoke_id_and_priority: InvokeIdAndPriority,
        cosem_method_descriptor: CosemMethodDescriptor,
        method_invocation_parameters: Option<DataObject>,
    ) -> Self {
        Self {
            invoke_id_and_priority,
            cosem_method_descriptor,
            method_invocation_parameters,
        }
    }

    /// Encode to A-XDR format
    ///
    /// Encoding order (A-XDR, reverse order):
    /// 1. method_invocation_parameters (optional DataObject)
    /// 2. cosem_method_descriptor (CosemMethodDescriptor)
    /// 3. invoke_id_and_priority (InvokeIdAndPriority)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Encode in reverse order
        // 1. method_invocation_parameters (optional DataObject)
        encoder.encode_bool(self.method_invocation_parameters.is_some())?;
        if let Some(ref params) = self.method_invocation_parameters {
            encoder.encode_data_object(params)?;
        }

        // 2. cosem_method_descriptor (CosemMethodDescriptor)
        let method_bytes = self.cosem_method_descriptor.encode()?;
        encoder.encode_bytes(&method_bytes)?;

        // 3. invoke_id_and_priority (InvokeIdAndPriority)
        let invoke_bytes = self.invoke_id_and_priority.encode()?;
        encoder.encode_bytes(&invoke_bytes)?;

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode in reverse order
        // 1. invoke_id_and_priority (InvokeIdAndPriority)
        let invoke_bytes = decoder.decode_octet_string()?;
        let invoke_id_and_priority = InvokeIdAndPriority::decode(&invoke_bytes)?;

        // 2. cosem_method_descriptor (CosemMethodDescriptor)
        let method_bytes = decoder.decode_octet_string()?;
        let cosem_method_descriptor = CosemMethodDescriptor::decode(&method_bytes)?;

        // 3. method_invocation_parameters (optional DataObject)
        let has_params = decoder.decode_bool()?;
        let method_invocation_parameters = if has_params {
            Some(decoder.decode_data_object()?)
        } else {
            None
        };

        Ok(Self {
            invoke_id_and_priority,
            cosem_method_descriptor,
            method_invocation_parameters,
        })
    }
}

/// Action Response Normal
///
/// Single method ACTION response. Contains the result of an ActionRequestNormal.
///
/// # Why Different from SetResponse?
/// ACTION operations can return data, unlike SET operations. The `ActionResult` enum
/// supports both success with data and success without data cases, making it more
/// flexible than `SetDataResult`.
///
/// # Optimization Considerations
/// - The result may contain large data objects. Consider using reference counting
///   or zero-copy types if the result is processed in multiple stages.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionResponseNormal {
    /// Invoke ID and priority
    pub invoke_id_and_priority: InvokeIdAndPriority,
    /// Result of the ACTION operation
    pub result: ActionResult,
}

impl ActionResponseNormal {
    /// Create a new ActionResponseNormal
    pub fn new(invoke_id_and_priority: InvokeIdAndPriority, result: ActionResult) -> Self {
        Self {
            invoke_id_and_priority,
            result,
        }
    }

    /// Encode to A-XDR format
    ///
    /// Encoding order (A-XDR, reverse order):
    /// 1. result (ActionResult)
    /// 2. invoke_id_and_priority (InvokeIdAndPriority)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Encode in reverse order
        // 1. result (ActionResult)
        let result_bytes = self.result.encode()?;
        encoder.encode_bytes(&result_bytes)?;

        // 2. invoke_id_and_priority (InvokeIdAndPriority)
        let invoke_bytes = self.invoke_id_and_priority.encode()?;
        encoder.encode_bytes(&invoke_bytes)?;

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode in reverse order
        // 1. invoke_id_and_priority (InvokeIdAndPriority)
        let invoke_bytes = decoder.decode_octet_string()?;
        let invoke_id_and_priority = InvokeIdAndPriority::decode(&invoke_bytes)?;

        // 2. result (ActionResult)
        let result_bytes = decoder.decode_octet_string()?;
        let result = ActionResult::decode(&result_bytes)?;

        Ok(Self {
            invoke_id_and_priority,
            result,
        })
    }
}

/// Action Request PDU
///
/// CHOICE type representing different ACTION request variants:
/// - **Normal**: Single method ACTION request
/// - **WithFirstPBlock**: First parameter block ACTION request (for large parameters)
/// - **WithPBlock**: Continue parameter block ACTION request
/// - **NextPBlock**: Next parameter block request
/// - **WithList**: Multiple method ACTION request
///
/// # Why Parameter Blocks?
/// Some methods may require large parameters that exceed the maximum PDU size. Parameter
/// blocks allow splitting large parameters across multiple requests, similar to data
/// blocks in GET/SET operations.
///
/// # Current Implementation Status
/// Currently only the `Normal` variant is implemented. Other variants are planned
/// for future implementation to support large parameter transfers and batch operations.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionRequest {
    /// Single method ACTION request
    Normal(ActionRequestNormal),
    // TODO: Implement other variants
    // WithFirstPBlock { ... },
    // WithPBlock { ... },
    // NextPBlock { ... },
    // WithList { ... },
}

impl ActionRequest {
    /// Create a new Normal ACTION request
    pub fn new_normal(
        invoke_id_and_priority: InvokeIdAndPriority,
        cosem_method_descriptor: CosemMethodDescriptor,
        method_invocation_parameters: Option<DataObject>,
    ) -> Self {
        Self::Normal(ActionRequestNormal::new(
            invoke_id_and_priority,
            cosem_method_descriptor,
            method_invocation_parameters,
        ))
    }

    /// Encode to A-XDR format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        match self {
            ActionRequest::Normal(normal) => {
                // Encode value first (A-XDR reverse order)
                let normal_bytes = normal.encode()?;
                encoder.encode_bytes(&normal_bytes)?;
                // Encode choice tag (1 = Normal)
                encoder.encode_u8(1)?;
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode choice tag first (A-XDR reverse order)
        let choice_tag = decoder.decode_u8()?;

        match choice_tag {
            1 => {
                // Normal variant
                let normal_bytes = decoder.decode_octet_string()?;
                let normal = ActionRequestNormal::decode(&normal_bytes)?;
                Ok(Self::Normal(normal))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid ActionRequest choice tag: {} (expected 1)",
                choice_tag
            ))),
        }
    }
}

/// Action Response PDU
///
/// CHOICE type representing different ACTION response variants:
/// - **Normal**: Single method ACTION response
/// - **WithPBlock**: Parameter block ACTION response
/// - **NextPBlock**: Next parameter block response
/// - **WithList**: Multiple method ACTION response
#[derive(Debug, Clone, PartialEq)]
pub enum ActionResponse {
    /// Single method ACTION response
    Normal(ActionResponseNormal),
    // TODO: Implement other variants
    // WithPBlock { ... },
    // NextPBlock { ... },
    // WithList { ... },
}

impl ActionResponse {
    /// Create a new Normal ACTION response
    pub fn new_normal(invoke_id_and_priority: InvokeIdAndPriority, result: ActionResult) -> Self {
        Self::Normal(ActionResponseNormal::new(invoke_id_and_priority, result))
    }

    /// Encode to A-XDR format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        match self {
            ActionResponse::Normal(normal) => {
                // Encode value first (A-XDR reverse order)
                let normal_bytes = normal.encode()?;
                encoder.encode_bytes(&normal_bytes)?;
                // Encode choice tag (1 = Normal)
                encoder.encode_u8(1)?;
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode choice tag first (A-XDR reverse order)
        let choice_tag = decoder.decode_u8()?;

        match choice_tag {
            1 => {
                // Normal variant
                let normal_bytes = decoder.decode_octet_string()?;
                let normal = ActionResponseNormal::decode(&normal_bytes)?;
                Ok(Self::Normal(normal))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid ActionResponse choice tag: {} (expected 1)",
                choice_tag
            ))),
        }
    }
}

// ============================================================================
// Event Notification PDU Implementation
// ============================================================================

/// Event Notification PDU
///
/// Asynchronous event notification sent by the server to the client when an event occurs.
/// This is an unconfirmed service, meaning the client does not send a response.
///
/// # Structure
/// - `time`: Time when the event occurred (optional CosemDateTime)
/// - `cosem_attribute_descriptor`: Attribute that triggered the event
/// - `attribute_value`: Value of the attribute at the time of the event
///
/// # Why Unconfirmed Service?
/// Event notifications are fire-and-forget messages. The server doesn't wait for
/// acknowledgment, allowing for efficient asynchronous event reporting. This design
/// reduces latency and overhead for time-sensitive events like alarms or state changes.
///
/// # Why Optional Time?
/// Not all events require precise timestamps. Making time optional allows the protocol
/// to efficiently handle both timestamped and non-timestamped events. When time is
/// provided, it uses COSEM DateTime format (12 bytes) for consistency with other
/// time-related attributes.
///
/// # Optimization Considerations
/// - Event notifications are typically infrequent, so performance is less critical
/// - The attribute value may be large, but this is acceptable for event reporting
/// - Future optimization: Consider using a ring buffer or queue for high-frequency
///   event scenarios to avoid blocking the main communication channel
#[derive(Debug, Clone, PartialEq)]
pub struct EventNotification {
    /// Optional time when the event occurred
    pub time: Option<dlms_core::datatypes::CosemDateTime>,
    /// Attribute descriptor that triggered the event
    pub cosem_attribute_descriptor: CosemAttributeDescriptor,
    /// Attribute value at the time of the event
    pub attribute_value: DataObject,
}

impl EventNotification {
    /// Create a new EventNotification
    pub fn new(
        time: Option<dlms_core::datatypes::CosemDateTime>,
        cosem_attribute_descriptor: CosemAttributeDescriptor,
        attribute_value: DataObject,
    ) -> Self {
        Self {
            time,
            cosem_attribute_descriptor,
            attribute_value,
        }
    }

    /// Encode to A-XDR format
    ///
    /// Encoding order (A-XDR, reverse order):
    /// 1. attribute_value (DataObject)
    /// 2. cosem_attribute_descriptor (CosemAttributeDescriptor)
    /// 3. time (optional CosemDateTime)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Encode in reverse order
        // 1. attribute_value (DataObject)
        encoder.encode_data_object(&self.attribute_value)?;

        // 2. cosem_attribute_descriptor (CosemAttributeDescriptor)
        let attr_bytes = self.cosem_attribute_descriptor.encode()?;
        encoder.encode_bytes(&attr_bytes)?;

        // 3. time (optional CosemDateTime)
        encoder.encode_bool(self.time.is_some())?;
        if let Some(ref dt) = self.time {
            let time_bytes = dt.encode()?;
            encoder.encode_bytes(&time_bytes)?;
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode in reverse order
        // 1. time (optional CosemDateTime)
        let has_time = decoder.decode_bool()?;
        let time = if has_time {
            // CosemDateTime is encoded as OctetString (12 bytes)
            let time_bytes = decoder.decode_octet_string()?;
            Some(dlms_core::datatypes::CosemDateTime::decode(&time_bytes)?)
        } else {
            None
        };

        // 2. cosem_attribute_descriptor (CosemAttributeDescriptor)
        let attr_bytes = decoder.decode_octet_string()?;
        let cosem_attribute_descriptor = CosemAttributeDescriptor::decode(&attr_bytes)?;

        // 3. attribute_value (DataObject)
        let attribute_value = decoder.decode_data_object()?;

        Ok(Self {
            time,
            cosem_attribute_descriptor,
            attribute_value,
        })
    }
}

// ============================================================================
// Access Request/Response PDU Implementation
// ============================================================================

/// Access Request Specification
///
/// Specifies a single access operation (GET, SET, or ACTION) within an AccessRequest.
///
/// # Structure
/// This is a CHOICE type with three variants:
/// - **Get** (tag 1): GET operation with attribute descriptor and optional selective access
/// - **Set** (tag 2): SET operation with attribute descriptor, optional selective access, and value
/// - **Action** (tag 3): ACTION operation with method descriptor and optional parameters
///
/// # Why CHOICE Type?
/// Each access operation has different parameters:
/// - GET: needs attribute descriptor and optional selective access
/// - SET: needs attribute descriptor, optional selective access, and value to write
/// - ACTION: needs method descriptor and optional method parameters
///
/// Using a CHOICE type allows type-safe representation of these different operation types.
#[derive(Debug, Clone, PartialEq)]
pub enum AccessRequestSpecification {
    /// GET operation (tag 1)
    Get {
        /// Attribute to read
        cosem_attribute_descriptor: CosemAttributeDescriptor,
        /// Optional selective access descriptor
        access_selection: Option<SelectiveAccessDescriptor>,
    },
    /// SET operation (tag 2)
    Set {
        /// Attribute to write
        cosem_attribute_descriptor: CosemAttributeDescriptor,
        /// Optional selective access descriptor
        access_selection: Option<SelectiveAccessDescriptor>,
        /// Value to write
        value: DataObject,
    },
    /// ACTION operation (tag 3)
    Action {
        /// Method to invoke
        cosem_method_descriptor: CosemMethodDescriptor,
        /// Optional method parameters
        method_invocation_parameters: Option<DataObject>,
    },
}

impl AccessRequestSpecification {
    /// Encode to A-XDR format
    ///
    /// Encoding order (A-XDR CHOICE):
    /// 1. Choice tag (1 = Get, 2 = Set, 3 = Action)
    /// 2. Value (operation-specific parameters)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        match self {
            AccessRequestSpecification::Get {
                cosem_attribute_descriptor,
                access_selection,
            } => {
                // Encode choice tag first (1 = Get)
                encoder.encode_u8(1)?;
                // Encode value after tag (in reverse order for SEQUENCE)
                // 1. cosem_attribute_descriptor (CosemAttributeDescriptor) - last field first
                let attr_bytes = cosem_attribute_descriptor.encode()?;
                encoder.encode_octet_string(&attr_bytes)?;
                // 2. access_selection (optional SelectiveAccessDescriptor)
                encoder.encode_bool(access_selection.is_some())?;
                if let Some(ref access_desc) = access_selection {
                    let access_bytes = access_desc.encode()?;
                    encoder.encode_octet_string(&access_bytes)?;
                }
            }
            AccessRequestSpecification::Set {
                cosem_attribute_descriptor,
                access_selection,
                value,
            } => {
                // Encode choice tag first (2 = Set)
                encoder.encode_u8(2)?;
                // Encode value after tag (in reverse order for SEQUENCE)
                // 1. cosem_attribute_descriptor (CosemAttributeDescriptor) - last field first
                let attr_bytes = cosem_attribute_descriptor.encode()?;
                encoder.encode_octet_string(&attr_bytes)?;
                // 2. access_selection (optional SelectiveAccessDescriptor)
                encoder.encode_bool(access_selection.is_some())?;
                if let Some(ref access_desc) = access_selection {
                    let access_bytes = access_desc.encode()?;
                    encoder.encode_octet_string(&access_bytes)?;
                }
                // 3. value (DataObject)
                encoder.encode_data_object(value)?;
            }
            AccessRequestSpecification::Action {
                cosem_method_descriptor,
                method_invocation_parameters,
            } => {
                // Encode choice tag first (3 = Action)
                encoder.encode_u8(3)?;
                // Encode value after tag (in reverse order for SEQUENCE)
                // 1. cosem_method_descriptor (CosemMethodDescriptor) - last field first
                let method_bytes = cosem_method_descriptor.encode()?;
                encoder.encode_octet_string(&method_bytes)?;
                // 2. method_invocation_parameters (optional DataObject)
                encoder.encode_bool(method_invocation_parameters.is_some())?;
                if let Some(ref params) = method_invocation_parameters {
                    encoder.encode_data_object(params)?;
                }
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode choice tag first
        let choice_tag = decoder.decode_u8()?;

        match choice_tag {
            1 => {
                // Get variant: decode value after tag (in reverse order)
                // 1. cosem_attribute_descriptor (CosemAttributeDescriptor)
                let attr_bytes = decoder.decode_octet_string()?;
                let cosem_attribute_descriptor = CosemAttributeDescriptor::decode(&attr_bytes)?;
                // 2. access_selection (optional SelectiveAccessDescriptor)
                let access_used = decoder.decode_bool()?;
                let access_selection = if access_used {
                    let access_bytes = decoder.decode_octet_string()?;
                    Some(SelectiveAccessDescriptor::decode(&access_bytes)?)
                } else {
                    None
                };
                Ok(Self::Get {
                    cosem_attribute_descriptor,
                    access_selection,
                })
            }
            2 => {
                // Set variant: decode value after tag (in reverse order)
                // 1. cosem_attribute_descriptor (CosemAttributeDescriptor)
                let attr_bytes = decoder.decode_octet_string()?;
                let cosem_attribute_descriptor = CosemAttributeDescriptor::decode(&attr_bytes)?;
                // 2. access_selection (optional SelectiveAccessDescriptor)
                let access_used = decoder.decode_bool()?;
                let access_selection = if access_used {
                    let access_bytes = decoder.decode_octet_string()?;
                    Some(SelectiveAccessDescriptor::decode(&access_bytes)?)
                } else {
                    None
                };
                // 3. value (DataObject)
                let value = decoder.decode_data_object()?;
                Ok(Self::Set {
                    cosem_attribute_descriptor,
                    access_selection,
                    value,
                })
            }
            3 => {
                // Action variant: decode value after tag (in reverse order)
                // 1. cosem_method_descriptor (CosemMethodDescriptor)
                let method_bytes = decoder.decode_octet_string()?;
                let cosem_method_descriptor = CosemMethodDescriptor::decode(&method_bytes)?;
                // 2. method_invocation_parameters (optional DataObject)
                let params_used = decoder.decode_bool()?;
                let method_invocation_parameters = if params_used {
                    Some(decoder.decode_data_object()?)
                } else {
                    None
                };
                Ok(Self::Action {
                    cosem_method_descriptor,
                    method_invocation_parameters,
                })
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid AccessRequestSpecification choice tag: {} (expected 1, 2, or 3)",
                choice_tag
            ))),
        }
    }
}

/// Access Request PDU
///
/// Used for accessing multiple attributes/methods in a single request.
/// This is a more general-purpose PDU that can combine GET, SET, and ACTION operations.
///
/// # Structure
/// - `invoke_id_and_priority`: Invoke ID and priority
/// - `access_request_list`: Array of access request specifications
///
/// # Why Access Request?
/// Access Request allows combining multiple operations (GET, SET, ACTION) in a single PDU,
/// reducing protocol overhead and improving efficiency when multiple operations need to be
/// performed atomically or in sequence.
///
/// # Usage Example
/// ```rust,no_run
/// // Create an Access Request with multiple operations
/// let access_request = AccessRequest::new(
///     invoke_id_and_priority,
///     vec![
///         AccessRequestSpecification::Get { ... },
///         AccessRequestSpecification::Set { ... },
///         AccessRequestSpecification::Action { ... },
///     ],
/// )?;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct AccessRequest {
    /// Invoke ID and priority
    pub invoke_id_and_priority: InvokeIdAndPriority,
    /// List of access request specifications
    pub access_request_list: Vec<AccessRequestSpecification>,
}

impl AccessRequest {
    /// Create a new AccessRequest
    ///
    /// # Arguments
    /// * `invoke_id_and_priority` - Invoke ID and priority
    /// * `access_request_list` - List of access request specifications (must not be empty)
    ///
    /// # Errors
    /// Returns error if `access_request_list` is empty
    pub fn new(
        invoke_id_and_priority: InvokeIdAndPriority,
        access_request_list: Vec<AccessRequestSpecification>,
    ) -> DlmsResult<Self> {
        if access_request_list.is_empty() {
            return Err(DlmsError::InvalidData(
                "AccessRequest: access_request_list cannot be empty".to_string(),
            ));
        }
        Ok(Self {
            invoke_id_and_priority,
            access_request_list,
        })
    }

    /// Encode to A-XDR format
    ///
    /// Encoding order (A-XDR SEQUENCE, reverse order):
    /// 1. access_request_list (array of AccessRequestSpecification)
    /// 2. invoke_id_and_priority (InvokeIdAndPriority)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Encode in reverse order (A-XDR SEQUENCE convention)
        // 1. invoke_id_and_priority (InvokeIdAndPriority) - last field first
        let invoke_bytes = self.invoke_id_and_priority.encode()?;
        encoder.encode_octet_string(&invoke_bytes)?;

        // 2. access_request_list (array of AccessRequestSpecification)
        // Encode array length: first byte indicates format
        let list_len = self.access_request_list.len();
        if list_len >= 128 {
            return Err(DlmsError::InvalidData(format!(
                "AccessRequest: access_request_list length ({}) exceeds maximum (127)",
                list_len
            )));
        }
        encoder.encode_u8(list_len as u8)?;
        // Encode each element (in forward order, as per A-XDR array encoding)
        for access_spec in self.access_request_list.iter() {
            let spec_bytes = access_spec.encode()?;
            encoder.encode_octet_string(&spec_bytes)?;
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode in reverse order (A-XDR SEQUENCE convention)
        // 1. invoke_id_and_priority (InvokeIdAndPriority)
        let invoke_bytes = decoder.decode_octet_string()?;
        let invoke_id_and_priority = InvokeIdAndPriority::decode(&invoke_bytes)?;

        // 2. access_request_list (array of AccessRequestSpecification)
        // Decode array length: first byte indicates format
        let first_byte: u8 = decoder.decode_u8()?;
        let list_len: usize = if (first_byte & 0x80) == 0 {
            // Short form: length < 128
            first_byte as usize
        } else {
            return Err(DlmsError::InvalidData(
                "AccessRequest: Long form array length not supported".to_string(),
            ));
        };

        let mut access_request_list = Vec::with_capacity(list_len);
        for _ in 0..list_len {
            let spec_bytes = decoder.decode_octet_string()?;
            access_request_list.push(AccessRequestSpecification::decode(&spec_bytes)?);
        }

        Ok(Self {
            invoke_id_and_priority,
            access_request_list,
        })
    }
}

/// Access Response Specification
///
/// Specifies the result of a single access operation (GET, SET, or ACTION) within an AccessResponse.
///
/// # Structure
/// This is a CHOICE type with three variants:
/// - **Get** (tag 1): GET operation result (GetDataResult)
/// - **Set** (tag 2): SET operation result (SetDataResult)
/// - **Action** (tag 3): ACTION operation result (ActionResult)
///
/// # Why CHOICE Type?
/// Each access operation has different result types:
/// - GET: returns GetDataResult (Data or DataAccessResult)
/// - SET: returns SetDataResult (Success or DataAccessResult)
/// - ACTION: returns ActionResult (Success, SuccessWithData, or DataAccessResult)
///
/// Using a CHOICE type allows type-safe representation of these different result types.
#[derive(Debug, Clone, PartialEq)]
pub enum AccessResponseSpecification {
    /// GET operation result (tag 1)
    Get(GetDataResult),
    /// SET operation result (tag 2)
    Set(SetDataResult),
    /// ACTION operation result (tag 3)
    Action(ActionResult),
}

impl AccessResponseSpecification {
    /// Encode to A-XDR format
    ///
    /// Encoding order (A-XDR CHOICE):
    /// 1. Choice tag (1 = Get, 2 = Set, 3 = Action)
    /// 2. Value (operation-specific result)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        match self {
            AccessResponseSpecification::Get(result) => {
                // Encode choice tag first (1 = Get)
                encoder.encode_u8(1)?;
                // Encode value after tag
                let result_bytes = result.encode()?;
                encoder.encode_octet_string(&result_bytes)?;
            }
            AccessResponseSpecification::Set(result) => {
                // Encode choice tag first (2 = Set)
                encoder.encode_u8(2)?;
                // Encode value after tag
                let result_bytes = result.encode()?;
                encoder.encode_octet_string(&result_bytes)?;
            }
            AccessResponseSpecification::Action(result) => {
                // Encode choice tag first (3 = Action)
                encoder.encode_u8(3)?;
                // Encode value after tag
                let result_bytes = result.encode()?;
                encoder.encode_octet_string(&result_bytes)?;
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode choice tag first
        let choice_tag = decoder.decode_u8()?;

        match choice_tag {
            1 => {
                // Get variant: decode value after tag
                let result_bytes = decoder.decode_octet_string()?;
                let result = GetDataResult::decode(&result_bytes)?;
                Ok(Self::Get(result))
            }
            2 => {
                // Set variant: decode value after tag
                let result_bytes = decoder.decode_octet_string()?;
                let result = SetDataResult::decode(&result_bytes)?;
                Ok(Self::Set(result))
            }
            3 => {
                // Action variant: decode value after tag
                let result_bytes = decoder.decode_octet_string()?;
                let result = ActionResult::decode(&result_bytes)?;
                Ok(Self::Action(result))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Invalid AccessResponseSpecification choice tag: {} (expected 1, 2, or 3)",
                choice_tag
            ))),
        }
    }
}

/// Access Response PDU
///
/// Response to an AccessRequest, containing results for multiple operations.
///
/// # Structure
/// - `invoke_id_and_priority`: Invoke ID and priority (echoed from request)
/// - `access_response_list`: Array of access response specifications
///
/// # Result Ordering
/// The `access_response_list` must have the same length and order as the corresponding
/// `access_request_list` in the AccessRequest, allowing the client to correlate each
/// result with its corresponding request.
///
/// # Usage Example
/// ```rust,no_run
/// // Process Access Response
/// for (i, response_spec) in access_response.access_response_list.iter().enumerate() {
///     match response_spec {
///         AccessResponseSpecification::Get(result) => {
///             // Handle GET result
///         }
///         AccessResponseSpecification::Set(result) => {
///             // Handle SET result
///         }
///         AccessResponseSpecification::Action(result) => {
///             // Handle ACTION result
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct AccessResponse {
    /// Invoke ID and priority (echoed from request)
    pub invoke_id_and_priority: InvokeIdAndPriority,
    /// List of access response specifications
    pub access_response_list: Vec<AccessResponseSpecification>,
}

impl AccessResponse {
    /// Create a new AccessResponse
    ///
    /// # Arguments
    /// * `invoke_id_and_priority` - Invoke ID and priority (echoed from request)
    /// * `access_response_list` - List of access response specifications (must not be empty)
    ///
    /// # Errors
    /// Returns error if `access_response_list` is empty
    pub fn new(
        invoke_id_and_priority: InvokeIdAndPriority,
        access_response_list: Vec<AccessResponseSpecification>,
    ) -> DlmsResult<Self> {
        if access_response_list.is_empty() {
            return Err(DlmsError::InvalidData(
                "AccessResponse: access_response_list cannot be empty".to_string(),
            ));
        }
        Ok(Self {
            invoke_id_and_priority,
            access_response_list,
        })
    }

    /// Encode to A-XDR format
    ///
    /// Encoding order (A-XDR SEQUENCE, reverse order):
    /// 1. access_response_list (array of AccessResponseSpecification)
    /// 2. invoke_id_and_priority (InvokeIdAndPriority)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Encode in reverse order (A-XDR SEQUENCE convention)
        // 1. invoke_id_and_priority (InvokeIdAndPriority) - last field first
        let invoke_bytes = self.invoke_id_and_priority.encode()?;
        encoder.encode_octet_string(&invoke_bytes)?;

        // 2. access_response_list (array of AccessResponseSpecification)
        // Encode array length: first byte indicates format
        let list_len = self.access_response_list.len();
        if list_len >= 128 {
            return Err(DlmsError::InvalidData(format!(
                "AccessResponse: access_response_list length ({}) exceeds maximum (127)",
                list_len
            )));
        }
        encoder.encode_u8(list_len as u8)?;
        // Encode each element (in forward order, as per A-XDR array encoding)
        for response_spec in self.access_response_list.iter() {
            let spec_bytes = response_spec.encode()?;
            encoder.encode_octet_string(&spec_bytes)?;
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode in reverse order (A-XDR SEQUENCE convention)
        // 1. invoke_id_and_priority (InvokeIdAndPriority)
        let invoke_bytes = decoder.decode_octet_string()?;
        let invoke_id_and_priority = InvokeIdAndPriority::decode(&invoke_bytes)?;

        // 2. access_response_list (array of AccessResponseSpecification)
        // Decode array length: first byte indicates format
        let first_byte: u8 = decoder.decode_u8()?;
        let list_len: usize = if (first_byte & 0x80) == 0 {
            // Short form: length < 128
            first_byte as usize
        } else {
            return Err(DlmsError::InvalidData(
                "AccessResponse: Long form array length not supported".to_string(),
            ));
        };

        let mut access_response_list = Vec::with_capacity(list_len);
        for _ in 0..list_len {
            let spec_bytes = decoder.decode_octet_string()?;
            access_response_list.push(AccessResponseSpecification::decode(&spec_bytes)?);
        }

        Ok(Self {
            invoke_id_and_priority,
            access_response_list,
        })
    }
}

// ============================================================================
// Exception Response PDU Implementation
// ============================================================================

/// Exception Response PDU
///
/// Error response sent when a PDU cannot be processed due to a protocol error.
/// This is different from DataAccessResult, which indicates application-level errors.
///
/// # Structure
/// - `invoke_id_and_priority`: Invoke ID and priority from the original request
/// - `state_error`: State error code (optional)
/// - `service_error`: Service error code
///
/// # Why Separate from DataAccessResult?
/// Exception responses indicate protocol-level errors (malformed PDU, invalid state, etc.),
/// while DataAccessResult indicates application-level errors (object not found, access denied, etc.).
/// This separation allows the application to distinguish between protocol issues and
/// application-level access problems, enabling appropriate error handling strategies.
///
/// # Optimization Considerations
/// - Exception responses are rare, so performance is not critical
/// - The optional state_error field uses `Option` to avoid unnecessary allocations
/// - Error codes are simple u8 values, keeping the response compact
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionResponse {
    /// Invoke ID and priority from the original request
    pub invoke_id_and_priority: InvokeIdAndPriority,
    /// Optional state error code
    pub state_error: Option<u8>,
    /// Service error code
    pub service_error: u8,
}

impl ExceptionResponse {
    /// Create a new ExceptionResponse
    pub fn new(
        invoke_id_and_priority: InvokeIdAndPriority,
        state_error: Option<u8>,
        service_error: u8,
    ) -> Self {
        Self {
            invoke_id_and_priority,
            state_error,
            service_error,
        }
    }

    /// Encode to A-XDR format
    ///
    /// Encoding order (A-XDR, reverse order):
    /// 1. service_error (Unsigned8)
    /// 2. state_error (optional Unsigned8)
    /// 3. invoke_id_and_priority (InvokeIdAndPriority)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Encode in reverse order
        // 1. service_error (Unsigned8)
        encoder.encode_u8(self.service_error)?;

        // 2. state_error (optional Unsigned8)
        encoder.encode_bool(self.state_error.is_some())?;
        if let Some(state_err) = self.state_error {
            encoder.encode_u8(state_err)?;
        }

        // 3. invoke_id_and_priority (InvokeIdAndPriority)
        let invoke_bytes = self.invoke_id_and_priority.encode()?;
        encoder.encode_bytes(&invoke_bytes)?;

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);

        // Decode in reverse order
        // 1. invoke_id_and_priority (InvokeIdAndPriority)
        let invoke_bytes = decoder.decode_octet_string()?;
        let invoke_id_and_priority = InvokeIdAndPriority::decode(&invoke_bytes)?;

        // 2. state_error (optional Unsigned8)
        let has_state_error = decoder.decode_bool()?;
        let state_error = if has_state_error {
            Some(decoder.decode_u8()?)
        } else {
            None
        };

        // 3. service_error (Unsigned8)
        let service_error = decoder.decode_u8()?;

        Ok(Self {
            invoke_id_and_priority,
            state_error,
            service_error,
        })
    }
}
