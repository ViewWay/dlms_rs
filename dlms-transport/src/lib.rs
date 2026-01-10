//! Transport layer module for DLMS/COSEM protocol
//!
//! This crate provides transport layer implementations for TCP, UDP, and Serial communication.

pub mod error;
pub mod stream;
pub mod tcp;
pub mod udp;
pub mod serial;

pub use error::{DlmsError, DlmsResult};
pub use stream::{StreamAccessor, TransportLayer};
pub use tcp::{TcpTransport, TcpSettings};
pub use udp::{UdpTransport, UdpSettings, MAX_UDP_PAYLOAD_SIZE};
pub use serial::{SerialTransport, SerialSettings};
