//! jDLMS - Rust implementation of DLMS/COSEM protocol
//!
//! This library provides a complete implementation of the DLMS/COSEM
//! communication standard for smart meter communication.
//!
//! # Architecture
//!
//! This library is organized as a workspace with multiple crates:
//!
//! - `dlms-core`: Core types, error handling, and utilities
//! - `dlms-asn1`: ASN.1 encoding/decoding
//! - `dlms-transport`: Transport layer (TCP, UDP, Serial)
//! - `dlms-session`: Session layer (HDLC, Wrapper)
//! - `dlms-security`: Security layer (encryption, authentication)
//! - `dlms-application`: Application layer (PDU, services)
//! - `dlms-interface`: COSEM interface classes
//! - `dlms-client`: Client implementation
//! - `dlms-server`: Server implementation
//!
//! # Usage
//!
//! ```no_run
//! use dlms::client::ConnectionBuilder;
//! ```
//!
//! # Examples
//!
//! See the `examples/` directory for usage examples.

// Re-export core types
pub use dlms_core::{DlmsError, DlmsResult, ObisCode};
pub use dlms_core::datatypes::*;

// Re-export client API
pub mod client {
    pub use dlms_client::*;
}

// Re-export server API
pub mod server {
    pub use dlms_server::*;
}

// Re-export interface classes
pub mod interface {
    pub use dlms_interface::*;
}
