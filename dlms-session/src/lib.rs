//! DLMS/COSEM Session Layer
//!
//! This crate provides session layer implementations for DLMS/COSEM protocol,
//! supporting HDLC (High-level Data Link Control) protocol.
//!
//! # Overview
//!
//! The session layer manages the communication session between client and server.
//! DLMS/COSEM supports HDLC as the primary session protocol:
//!
//! - **HDLC**: Traditional connection-oriented protocol with framing, addressing,
//!   and flow control
//!
//! # HDLC Protocol
//!
//! HDLC is the traditional DLMS session protocol operating over serial connections:
//!
//! ```rust,ignore
//! use dlms_session::{HdlcConnection, HdlcAddress, HdlcParameters};
//! use tokio::serial::SerialStream;
//!
//! // Create HDLC connection
//! let serial = SerialStream::open("/dev/ttyUSB0", &serial_settings).await?;
//! let params = HdlcParameters::new(
//!     HdlcAddress::client(0x01),  // Client address
//!     HdlcAddress::server(0x01),  // Server address
//!     2048,  // Max info field length
//! );
//!
//! let mut connection = HdlcConnection::new(serial, params)?;
//!
//! // Send connection request
//! connection.send_snrm().await?;
//!
//! // Wait for UA (Unnumbered Acknowledgment)
//! let frame = connection.recv_frame().await?;
//!
//! // Send data
//! let data = vec![0x01, 0x02, 0x03];
//! connection.send_data(&data).await?;
//! ```
//!
//! # HDLC Window Management
//!
//! The HDLC layer implements a sliding window protocol for flow control:
//!
//! ```rust,ignore
//! use dlms_session::{HdlcConnection, HdlcParameters, SendWindow, ReceiveWindow};
//!
//! let params = HdlcParameters::new(
//!     HdlcAddress::client(0x01),
//!     HdlcAddress::server(0x01),
//!     2048,
//! )
//! .with_window_size(8);  // Window size 8
//!
//! let mut connection = HdlcConnection::new(serial, params)?;
//! ```
//!
//! # Frame Structure
//!
//! HDLC frames have the following structure:
//!
//! ```text
//! +-----+-----+-----+-----+------+-----+-----+
//! | Flag | Addr | Ctrl | HCS | Info | FCS | Flag |
//! +-----+-----+-----+-----+------+-----+-----+
//! | 0x7E | ... | ... | ... | ... | ... | 0x7E |
//! +-----+-----+-----+-----+------+-----+-----+
//! ```
//!
//! Where:
//! - **Flag**: 0x7E frame delimiter
//! - **Addr**: HDLC address (client/server)
//! - **Ctrl**: Control field (I/S/U frames)
//! - **HCS**: Header Check Sequence (CRC-16)
//! - **Info**: Information field (application data)
//! - **FCS**: Frame Check Sequence (CRC-16)
//!
//! # Connection Management
//!
//! ## HDLC Connection States
//!
//! ```text
//!      ┌─────────┐
//!      │   Idle   │
//!      └────┬────┘
//!           │ SNRM
//!           ▼
//!      ┌─────────┐
//!      │ Pending │
//!      └────┬────┘
//!           │ UA received
//!           ▼
//!      ┌─────────┐
//!      │ Connected│
//!      └─────────┘
//!           │ DISC
//!           ▼
//!      ┌─────────┐
//!      │  Closed  │
//!      └─────────┘
//! ```
//!
//! ## Connection Statistics
//!
//! ```rust,ignore
//! use dlms_session::HdlcConnection;
//!
//! let stats = connection.statistics().await;
//! println!("Frames sent: {}", stats.frames_sent);
//! println!("Frames received: {}", stats.frames_received);
//! println!("Errors: {}", stats.frame_errors);
//! ```
//!
//! # Module Structure
//!
//! - [`hdlc`] - HDLC protocol implementation
//!   - [`hdlc::connection`] - HDLC connection management
//!   - [`hdlc::frame`] - HDLC frame encoding/decoding
//!   - [`hdlc::window`] - Sliding window protocol
//!   - [`hdlc::address`] - HDLC addressing
//! - [`error`] - Session layer error types
//!
//! # Implementation Status
//!
//! ## HDLC
//! - [x] HDLC address encoding/decoding
//! - [x] HDLC frame encoding/decoding
//! - [x] FCS calculation and verification (CRC-16)
//! - [x] HCS calculation and verification
//! - [x] Connection management (SNRM/UA, DISC/DM/UA)
//! - [x] Sliding window protocol (I-frames, RR-frames)
//! - [x] Frame retransmission
//! - [x] Parameter negotiation
//! - [x] LLC header support
//! - [x] Statistics collection
//! - [x] State machine management
//!
//! # References
//!
//! - IEC 62056-46: DLMS/COSEM HDLC Protocol

pub mod error;
pub mod hdlc;
pub mod wrapper;

pub use error::{DlmsError, DlmsResult};

// Wrapper exports
pub use wrapper::{
    WrapperSession, WrapperHeader, WrapperPdu, WRAPPER_HEADER_LENGTH,
};

// HDLC exports
pub use hdlc::{
    HdlcConnection, HdlcParameters, HdlcAddress, HdlcFrame, FrameType,
    HdlcConnectionState, HdlcStatistics, SendWindow, ReceiveWindow,
    HdlcAddressPair,
};
