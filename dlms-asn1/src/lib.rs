//! ASN.1 Encoding for DLMS/COSEM Protocol
//!
//! This crate provides ASN.1 encoding and decoding functionality for the DLMS/COSEM
//! protocol, including A-XDR (Extended Data Encoding Rules), BER (Basic Encoding Rules),
//! COSEM ASN.1 definitions, and ISO-ACSE (Application Control Service Element).
//!
//! # Overview
//!
//! DLMS/COSEM uses multiple encoding rules depending on the layer:
//!
//! - **A-XDR**: Used for application layer data (PDU encoding)
//! - **BER**: Used for presentation and session layers (ACSE, AARQ, AARE)
//! - **COSEM ASN.1**: COSEM-specific data structures
//! - **ISO-ACSE**: Association control service element
//!
//! # A-XDR Encoding
//!
//! A-XDR (Extended Data Encoding Rules) is a DLMS-specific encoding scheme for
//! encoding COSEM data objects:
//!
//! ```rust
//! use dlms_asn1::{AxdrEncoder, AxdrDecoder};
//! use dlms_core::DataObject;
//!
//! // Encode an integer value
//! let mut encoder = AxdrEncoder::new();
//! encoder.encode_u32(0x12345678)?;
//!
//! // Decode back
//! let mut decoder = AxdrDecoder::new(encoder.as_bytes());
//! let value = decoder.decode_u32()?;
//!
//! assert_eq!(value, 0x12345678);
//! # Ok::<(), dlms_core::DlmsError>(())
//! ```
//!
//! ## Supported A-XDR Types
//!
//! - Numeric: Integer8, Integer16, Integer32, Integer64, Unsigned8-64, Float32, Float64
//! - Strings: OctetString, VisibleString, Utf8String, BitString
//! - Complex: Array, Structure, CompactArray
//! - Time: Date, Time, DateTime
//! - Special: Null, Boolean, BCD, Enumerate
//!
//! # BER Encoding
//!
//! BER (Basic Encoding Rules) is used for encoding ISO-ACSE PDUs:
//!
//! ```rust
//! use dlms_asn1::ber::BerEncoder;
//!
//! let mut encoder = BerEncoder::new();
//! encoder.encode_integer(0x12345678)?;
//! let _bytes = encoder.into_bytes();
//! # Ok::<(), dlms_core::DlmsError>(())
//! ```
//!
//! # ISO-ACSE
//!
//! ISO-ACSE (Application Control Service Element) handles connection establishment
//! and release:
//!
//! ```rust,ignore
//! use dlms_asn1::{AARQApdu, AAREApdu, AAREApduBuilder};
//!
//! // Create an association request
//! let aarq = AARQApdu::new(
//!     DLMS_APPLICATION_CONTEXT_NAME,
//!     None,  // No authentication
//!     None,  // No sender ACSE requirements
//! )?;
//!
//! // Encode the request
//! let encoded = aarq.encode()?;
//! ```
//!
//! # Module Structure
//!
//! - [`axdr`] - A-XDR encoder/decoder for DLMS data types
//! - [`ber`] - BER encoder/decoder for ASN.1 structures
//! - [`cosem`] - COSEM-specific ASN.1 definitions
//! - [`iso_acse`] - ISO-ACSE protocol implementation
//! - [`error`] - ASN.1 specific error types
//!
//! # Implementation Status
//!
//! ## A-XDR Encoding/Decoding
//! - [x] Basic data type encoding/decoding
//! - [x] DataObject encoding/decoding
//! - [x] CompactArray encoding/decoding
//! - [x] Float32/Float64 IEEE 754 encoding
//! - [x] Length encoding (short and long form)
//!
//! ## COSEM ASN.1
//! - [x] CosemObjectIdentifier
//! - [x] CosemAttributeDescriptor
//! - [x] CosemMethodDescriptor
//!
//! ## ISO-ACSE
//! - [x] Basic types (AssociateResult, ReleaseRequestReason)
//! - [x] APTitle (Form 1 and Form 2)
//! - [x] AEQualifier (Form 1 and Form 2)
//! - [x] AssociationInformation
//! - [x] ACSERequirements with bit definitions
//! - [x] MechanismName with OID constants
//! - [x] ApplicationContextNameList
//! - [x] AuthenticationValue (all 33 CHOICE variants)
//! - [x] AssociateSourceDiagnostic
//! - [x] AARQ (Association Request)
//! - [x] AARE (Association Response)
//! - [x] RLRQ (Release Request)
//! - [x] RLRE (Release Response)

pub mod error;
pub mod axdr;
pub mod ber;
pub mod cosem;
pub mod iso_acse;

pub use error::{DlmsError, DlmsResult};
pub use axdr::{AxdrEncoder, AxdrDecoder};
pub use axdr::types::{AxdrTag, LengthEncoding};
pub use ber::{BerEncoder, BerDecoder, BerTag, BerTagClass, BerLength};
pub use iso_acse::{
    AARQApdu, AAREApdu, RLRQApdu, RLREApdu,
    DLMS_APPLICATION_CONTEXT_NAME, DLMS_APPLICATION_CONTEXT_NAME_CIPHERED
};
