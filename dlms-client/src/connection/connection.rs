//! Connection trait and implementations for DLMS/COSEM client
//!
//! This module provides the core connection abstraction for DLMS/COSEM client operations.
//!
//! # Architecture
//!
//! The connection layer integrates multiple protocol layers:
//! - **Transport Layer**: TCP, UDP, or Serial communication
//! - **Session Layer**: HDLC or Wrapper protocol
//! - **Security Layer**: Encryption and authentication
//! - **Application Layer**: PDU encoding/decoding and services
//!
//! # Connection Types
//!
//! - **Logical Name (LN) Connection**: Uses logical name addressing (6-byte OBIS codes)
//! - **Short Name (SN) Connection**: Uses short name addressing (2-byte addresses)
//!
//! # Design Philosophy
//!
//! The connection abstraction provides:
//! - **Unified API**: Same interface for LN and SN connections
//! - **Type Safety**: Compile-time guarantees about addressing method
//! - **Error Handling**: Comprehensive error propagation from all layers
//! - **Async Operations**: Non-blocking I/O for high performance
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use dlms_client::connection::{Connection, LnConnection};
//! use dlms_application::service::GetService;
//! use dlms_core::ObisCode;
//!
//! // Create and open connection
//! let mut conn = LnConnection::new(...)?;
//! conn.open().await?;
//!
//! // Perform operations
//! let service = GetService::new();
//! let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
//! let value = conn.get_attribute(obis, 2).await?;
//!
//! // Close connection
//! conn.close().await?;
//! ```

use dlms_core::{DlmsResult, ObisCode, DataObject};
use std::time::Duration;

/// Connection trait for DLMS/COSEM client operations
///
/// This trait provides a unified interface for all DLMS/COSEM connection types,
/// abstracting away the differences between LN and SN addressing, HDLC and Wrapper
/// protocols, and different transport mechanisms.
///
/// # Why a Trait?
/// Using a trait allows:
/// - **Polymorphism**: Same code works with different connection types
/// - **Testability**: Easy to mock connections for testing
/// - **Extensibility**: Easy to add new connection types in the future
///
/// # Connection Lifecycle
/// 1. **Create**: Use a builder to create a connection
/// 2. **Open**: Establish transport, session, and application layer connections
/// 3. **Use**: Perform GET, SET, ACTION operations
/// 4. **Close**: Cleanly close all layers
///
/// # Error Handling
/// All operations return `DlmsResult` to handle:
/// - Transport layer errors (network issues, timeouts)
/// - Session layer errors (frame errors, protocol violations)
/// - Application layer errors (PDU errors, access denied)
/// - Security errors (authentication failures, encryption errors)
#[async_trait::async_trait]
pub trait Connection: Send + Sync {
    /// Open the connection
    ///
    /// This establishes connections at all protocol layers:
    /// 1. Transport layer (TCP/Serial)
    /// 2. Session layer (HDLC/Wrapper handshake)
    /// 3. Application layer (Initiate Request/Response)
    ///
    /// # Errors
    /// Returns error if any layer fails to establish
    async fn open(&mut self) -> DlmsResult<()>;

    /// Close the connection
    ///
    /// This cleanly closes all protocol layers:
    /// 1. Application layer (Release Request/Response if needed)
    /// 2. Session layer (DISC/DM/UA for HDLC)
    /// 3. Transport layer (close socket/serial port)
    ///
    /// # Errors
    /// Returns error if closing fails at any layer
    async fn close(&mut self) -> DlmsResult<()>;

    /// Check if the connection is open
    fn is_open(&self) -> bool;

    /// Get an attribute value using logical name addressing
    ///
    /// # Arguments
    /// * `obis_code` - OBIS code of the object
    /// * `class_id` - Class ID of the object
    /// * `attribute_id` - Attribute ID to read
    ///
    /// # Returns
    /// The attribute value as a `DataObject`
    ///
    /// # Errors
    /// Returns error if the connection is not open, if the request fails, or if the response indicates an error
    async fn get_attribute(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        attribute_id: u8,
    ) -> DlmsResult<DataObject>;

    /// Set an attribute value using logical name addressing
    ///
    /// # Arguments
    /// * `obis_code` - OBIS code of the object
    /// * `class_id` - Class ID of the object
    /// * `attribute_id` - Attribute ID to write
    /// * `value` - Value to write
    ///
    /// # Returns
    /// Ok(()) if successful
    ///
    /// # Errors
    /// Returns error if the connection is not open, if the request fails, or if the response indicates an error
    async fn set_attribute(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        attribute_id: u8,
        value: DataObject,
    ) -> DlmsResult<()>;

    /// Invoke a method using logical name addressing
    ///
    /// # Arguments
    /// * `obis_code` - OBIS code of the object
    /// * `class_id` - Class ID of the object
    /// * `method_id` - Method ID to invoke
    /// * `parameters` - Optional method parameters
    ///
    /// # Returns
    /// Optional return value from the method (if any)
    ///
    /// # Errors
    /// Returns error if the connection is not open, if the request fails, or if the response indicates an error
    async fn invoke_method(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        method_id: u8,
        parameters: Option<DataObject>,
    ) -> DlmsResult<Option<DataObject>>;

    /// Send a raw PDU and receive a response
    ///
    /// # Arguments
    /// * `request` - The request PDU to send
    /// * `timeout` - Optional timeout for the response
    ///
    /// # Returns
    /// The response PDU
    ///
    /// # Errors
    /// Returns error if the connection is not open, if sending fails, or if receiving times out
    async fn send_request(
        &mut self,
        request: &[u8],
        timeout: Option<Duration>,
    ) -> DlmsResult<Vec<u8>>;
}

/// Connection state
///
/// Tracks the current state of a DLMS/COSEM connection to ensure
/// operations are only performed when the connection is in the correct state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connection is closed (initial state)
    Closed,
    /// Transport layer is open, but session layer is not established
    TransportOpen,
    /// Session layer is established, but application layer is not initiated
    SessionOpen,
    /// Application layer is initiated, connection is fully ready
    Ready,
}

impl ConnectionState {
    /// Check if the connection is ready for operations
    pub fn is_ready(&self) -> bool {
        matches!(self, ConnectionState::Ready)
    }

    /// Check if the connection can be closed
    pub fn can_close(&self) -> bool {
        !matches!(self, ConnectionState::Closed)
    }
}
