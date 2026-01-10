//! BER (Basic Encoding Rules) encoder and decoder for ASN.1
//!
//! This module provides BER encoding/decoding functionality for ASN.1 structures,
//! which is used in the ISO-ACSE layer of DLMS/COSEM protocol.
//!
//! # ASN.1 BER Encoding Overview
//!
//! BER (Basic Encoding Rules) is the most commonly used ASN.1 encoding format.
//! Each ASN.1 value is encoded as a TLV (Tag-Length-Value) triplet:
//!
//! ```
//! [Tag] [Length] [Value]
//! ```
//!
//! ## Tag Encoding
//!
//! The tag identifies the type of the data:
//! - **Class** (2 bits): Universal (00), Application (01), Context-specific (10), Private (11)
//! - **Constructed/Primitive** (1 bit): 0 = Primitive, 1 = Constructed
//! - **Tag Number** (5-31 bits): The actual tag number
//!
//! Tag encoding format:
//! ```
//! Bits: 8 7 6 5 4 3 2 1
//!       C C P T T T T T
//! ```
//! Where:
//! - CC = Class (00=Universal, 01=Application, 10=Context, 11=Private)
//! - P = Primitive (0) or Constructed (1)
//! - TTTTT = Tag number (0-30), or 11111 indicates extended tag
//!
//! ## Length Encoding
//!
//! Length can be encoded in two forms:
//! - **Short form** (1 byte): For lengths 0-127
//!   - Bit 7 = 0
//!   - Bits 6-0 = length value
//! - **Long form** (2-127 bytes): For lengths > 127
//!   - First byte: Bit 7 = 1, Bits 6-0 = number of length bytes
//!   - Following bytes: Big-endian length value
//!
//! ## Value Encoding
//!
//! The value encoding depends on the data type:
//! - **Primitive types**: Direct encoding (INTEGER, OCTET STRING, etc.)
//! - **Constructed types**: Sequence of TLV triplets (SEQUENCE, SET, etc.)
//!
//! # Why BER for ISO-ACSE?
//!
//! ISO-ACSE (Association Control Service Element) uses BER encoding as specified
//! in ISO 8650. This is different from A-XDR used in the COSEM application layer.
//!
//! # Implementation Notes
//!
//! 1. **Tag Classes**: We support all four tag classes (Universal, Application,
//!    Context-specific, Private) as required by ISO-ACSE.
//! 2. **Extended Tags**: Tags > 30 are encoded using extended tag format.
//! 3. **Indefinite Length**: Currently not supported (definite length only).
//!    This is sufficient for DLMS/COSEM use cases.
//! 4. **Error Handling**: All encoding/decoding operations return `Result` types
//!    for proper error propagation.
//!
//! # Optimization Considerations
//!
//! - **Memory Allocation**: BER encoding uses `Vec<u8>` for buffers. For
//!   high-frequency operations, consider using `BytesMut` for zero-copy operations.
//! - **Tag Caching**: Frequently used tags can be pre-encoded and cached.
//! - **Length Pre-calculation**: For constructed types, we can pre-calculate
//!   lengths to avoid multiple passes.

pub mod encoder;
pub mod decoder;
pub mod types;

pub use encoder::BerEncoder;
pub use decoder::BerDecoder;
pub use types::{BerTag, BerTagClass, BerLength};
