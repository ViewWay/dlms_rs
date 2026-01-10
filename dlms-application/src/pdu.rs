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

use dlms_core::{DlmsError, DlmsResult};
use dlms_core::datatypes::{BitString, DataObject};
use dlms_asn1::{AxdrDecoder, AxdrEncoder};

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
/// # Bit Layout (from LSB to MSB)
/// - Bits 0-7: General features (attribute 0 access, multiple references, etc.)
/// - Bits 8-15: Data access features (block transfer, selective access, etc.)
/// - Bits 16-23: Service features (action, event notification, etc.)
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
    /// 1. The field value (if present)
    /// 2. A Boolean flag indicating whether the field is used
    ///
    /// This allows the decoder to know whether to read the field value.
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
        // Optional field: encode value first (if present), then usage flag
        if let Some(qos) = self.proposed_quality_of_service {
            encoder.encode_i8(qos)?;
        }
        encoder.encode_bool(self.proposed_quality_of_service.is_some())?;

        // 5. response_allowed (Boolean, default true)
        encoder.encode_bool(self.response_allowed)?;

        // 6. dedicated_key (optional OctetString)
        // Optional field: encode value first (if present), then usage flag
        if let Some(ref key) = self.dedicated_key {
            encoder.encode_octet_string(key)?;
        }
        encoder.encode_bool(self.dedicated_key.is_some())?;

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
    /// * `negotiated_dlms_version_number` - Negotiated DLMS version
    /// * `negotiated_conformance` - Negotiated conformance bits
    /// * `server_max_receive_pdu_size` - Maximum PDU size server can receive
    /// * `vaa_name` - VAA name identifier
    ///
    /// # Returns
    /// Returns `Ok(InitiateResponse)` if parameters are valid, `Err` otherwise
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
        // Optional field: encode value first (if present), then usage flag
        if let Some(qos) = self.negotiated_quality_of_service {
            encoder.encode_i8(qos)?;
        }
        encoder.encode_bool(self.negotiated_quality_of_service.is_some())?;

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
}
