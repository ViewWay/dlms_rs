//! Transport Layer for DLMS/COSEM Protocol
//!
//! This crate provides transport layer implementations for TCP, UDP, and Serial communication.
//!
//! # Overview
//!
//! The transport layer handles the physical transmission of DLMS/COSEM data over various
//! communication media. DLMS/COSEM supports multiple transport mechanisms:
//!
//! - **TCP**: Reliable stream-oriented transport for IP networks
//! - **UDP**: Datagram transport for IP networks
//! - **Serial**: Point-to-point serial communication (RS-485, RS-232)
//!
//! # TCP Transport
//!
//! TCP transport provides reliable, connection-oriented communication over IP networks:
//!
//! ```rust,ignore
//! use dlms_transport::{TcpTransport, TcpSettings};
//!
//! // Configure TCP connection
//! let settings = TcpSettings::new("127.0.0.1:4059")
//!     .with_connect_timeout_secs(10)
//!     .with_read_timeout_secs(30);
//!
//! // Create transport
//! let transport = TcpTransport::connect(&settings).await?;
//!
//! // Send data
//! transport.send_all(&data).await?;
//!
//! // Receive data
//! let buffer = transport.recv_exact(1024).await?;
//! ```
//!
//! # UDP Transport
//!
//! UDP transport provides lightweight datagram communication:
//!
//! ```rust,ignore
//! use dlms_transport::{UdpTransport, UdpSettings};
//!
//! // Configure UDP
//! let settings = UdpSettings::new("127.0.0.1:4059")
//!     .with_bind_address("0.0.0.0:0")
//!     .with_max_payload_size(2048);
//!
//! // Create transport
//! let transport = UdpTransport::bind(&settings).await?;
//!
//! // Send datagram
//! transport.send_to(&data, "127.0.0.1:4059").await?;
//!
//! // Receive datagram
//! let (data, addr) = transport.recv_from().await?;
//! ```
//!
//! # Serial Transport
//!
//! Serial transport handles communication over serial ports (RS-485, RS-232):
//!
//! ```rust,ignore
//! use dlms_transport::{SerialTransport, SerialSettings};
//!
//! // Configure serial port
//! let settings = SerialSettings::new("/dev/ttyUSB0")
//!     .with_baud_rate(9600)
//!     .with_data_bits(8)
//!     .with_parity(None)
//!     .with_stop_bits(1)
//!     .with_timeout_secs(5);
//!
//! // Create transport
//! let transport = SerialTransport::open(&settings).await?;
//!
//! // Send data
//! transport.send_all(&data).await?;
//!
//! // Receive data
//! let buffer = transport.recv_exact(1024).await?;
//! ```
//!
//! # Transport Layer Trait
//!
//! The [`TransportLayer`] trait provides a unified interface for all transport types:
//!
//! ```rust,ignore
//! use dlms_transport::TransportLayer;
//! use async_trait::async_trait;
//!
//! async fn communicate<T: TransportLayer>(transport: &mut T) -> Result<(), Error> {
//!     // Send data
//!     transport.send_all(&[0x01, 0x02, 0x03]).await?;
//!
//!     // Receive data
//!     let data = transport.recv_exact(100).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Connection Settings
//!
//! Each transport type has specific settings:
//!
//! ## TCP Settings
//!
//! - `connect_timeout_secs`: Connection timeout (default: 30)
//! - `read_timeout_secs`: Read timeout (default: 60)
//! - `write_timeout_secs`: Write timeout (default: 30)
//! - `tcp_nodelay`: Disable Nagle's algorithm (default: true)
//! - `keepalive_secs`: TCP keepalive interval (default: 60)
//!
//! ## UDP Settings
//!
//! - `bind_address`: Local bind address
//! - `max_payload_size`: Maximum datagram size (default: 2048)
//! - `recv_timeout_secs`: Receive timeout (default: 30)
//! - `send_timeout_secs`: Send timeout (default: 30)
//!
//! ## Serial Settings
//!
//! - `baud_rate`: Communication speed (default: 9600)
//! - `data_bits`: Data bits (default: 8)
//! - `parity`: Parity checking (default: None)
//! - `stop_bits`: Stop bits (default: 1)
//! - `timeout_secs`: Operation timeout (default: 5)
//!
//! # Implementation Status
//!
//! ## TCP Transport
//! - [x] TCP connection management
//! - [x] Async read/write operations
//! - [ ] Connection pooling
//! - [ ] Auto-reconnect mechanism
//! - [ ] Timeout handling optimization
//!
//! ## UDP Transport
//! - [x] UDP socket management
//! - [x] Async read/write operations
//! - [ ] Packet fragmentation/reassembly
//! - [ ] Packet loss detection
//!
//! ## Serial Transport
//! - [x] Serial connection management
//! - [x] Async read/write operations
//! - [ ] Serial parameter auto-detection
//! - [ ] Flow control support
//! - [ ] Multi-port device management
//!
//! ## Common Features
//! - [ ] Transport layer statistics
//! - [ ] Connection state monitoring
//! - [ ] Error recovery mechanism
//!
//! # Module Structure
//!
//! - [`tcp`] - TCP transport implementation
//! - [`udp`] - UDP transport implementation
//! - [`serial`] - Serial transport implementation
//! - [`stream`] - Transport layer trait definitions
//! - [`error`] - Transport layer error types
//!
//! # References
//!
//! - IEC 62056-46: DLMS/COSEM HDLC Protocol (Transport over TCP)
//! - IEC 62056-53: DLMS/COSEM Wrapper Protocol (Transport over UDP)

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
