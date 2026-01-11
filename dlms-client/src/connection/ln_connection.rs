//! Logical Name (LN) addressing connection for DLMS/COSEM client
//!
//! This module provides the Logical Name (LN) connection implementation, which uses
//! 6-byte OBIS codes to identify COSEM objects.
//!
//! # Architecture
//!
//! LN connections integrate multiple protocol layers:
//! - **Transport Layer**: TCP, UDP, or Serial
//! - **Session Layer**: HDLC or Wrapper
//! - **Security Layer**: Encryption and authentication (future)
//! - **Application Layer**: PDU encoding/decoding with LN addressing
//!
//! # Connection Flow
//!
//! According to dlms-docs/dlms/cosem连接过程.txt:
//! 1. **Transport Open**: Open TCP/Serial connection
//! 2. **Session Open**: Establish HDLC/Wrapper session
//!    - HDLC: SNRM -> UA (Set Normal Response Mode handshake)
//!    - Wrapper: Direct open (no handshake needed)
//! 3. **Application Initiate**: Send InitiateRequest, receive InitiateResponse
//! 4. **Ready**: Connection is ready for GET/SET/ACTION operations
//!
//! # Why LN Addressing?
//! Logical Name addressing uses OBIS codes (6 bytes) to uniquely identify objects.
//! This provides:
//! - **Human Readable**: OBIS codes follow a standard format (A.B.C.D.E.F)
//! - **Globally Unique**: OBIS codes are standardized across all DLMS devices
//! - **Flexible**: Can address any object regardless of device configuration
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use dlms_client::connection::{Connection, LnConnection, LnConnectionConfig};
//! use dlms_core::ObisCode;
//!
//! // Create connection
//! let config = LnConnectionConfig::default();
//! let mut conn = LnConnection::new(config);
//!
//! // Open connection
//! conn.open().await?;
//!
//! // Read attribute
//! let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
//! let value = conn.get_attribute(obis, 1, 2).await?;
//!
//! // Close connection
//! conn.close().await?;
//! ```

use super::connection::{Connection, ConnectionState};
use dlms_application::service::{GetService, SetService, ActionService};
use dlms_application::pdu::{
    InitiateRequest, InitiateResponse, GetRequest, GetResponse, SetRequest, SetResponse,
    ActionRequest, ActionResponse, CosemAttributeDescriptor, CosemMethodDescriptor,
    InvokeIdAndPriority, Conformance,
};
use dlms_application::addressing::LogicalNameReference;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use dlms_session::hdlc::{HdlcConnection, HdlcAddress};
use dlms_session::wrapper::WrapperSession;
use dlms_transport::{TcpTransport, SerialTransport, TransportLayer, StreamAccessor};
use dlms_security::SecuritySuite;
use std::time::Duration;

/// Session layer type
///
/// Distinguishes between HDLC and Wrapper session layers, which have different
/// connection establishment procedures and data framing.
#[derive(Debug)]
enum SessionLayer {
    /// HDLC session (for Serial transport)
    Hdlc(Box<dyn std::any::Any + Send + Sync>),
    /// Wrapper session (for TCP/UDP transport)
    Wrapper(Box<dyn std::any::Any + Send + Sync>),
}

/// Logical Name (LN) connection configuration
///
/// Configuration parameters for establishing an LN connection, including
/// transport settings, session parameters, and security configuration.
#[derive(Debug, Clone)]
pub struct LnConnectionConfig {
    /// Local HDLC address (for HDLC sessions, required if using HDLC)
    pub local_address: Option<HdlcAddress>,
    /// Remote HDLC address (for HDLC sessions, required if using HDLC)
    pub remote_address: Option<HdlcAddress>,
    /// Client ID (for Wrapper sessions, required if using Wrapper)
    pub client_id: Option<u16>,
    /// Logical device ID (for Wrapper sessions, required if using Wrapper)
    pub logical_device_id: Option<u16>,
    /// Security suite configuration (optional, for future use)
    pub security_suite: Option<SecuritySuite>,
    /// Conformance bits (client capabilities)
    pub conformance: Conformance,
    /// Maximum PDU size
    pub max_pdu_size: u16,
    /// DLMS version
    pub dlms_version: u8,
}

impl Default for LnConnectionConfig {
    fn default() -> Self {
        Self {
            local_address: None,
            remote_address: None,
            client_id: Some(0x10),
            logical_device_id: Some(0x01),
            security_suite: None,
            conformance: Conformance::default(),
            max_pdu_size: 1024,
            dlms_version: 6,
        }
    }
}

/// Logical Name (LN) connection implementation
///
/// Provides a high-level interface for DLMS/COSEM operations using
/// logical name addressing (OBIS codes).
///
/// # Connection State Management
///
/// The connection maintains state to ensure operations are only performed
/// when the connection is ready. State transitions:
/// - `Closed` -> `TransportOpen` (transport.open())
/// - `TransportOpen` -> `SessionOpen` (session.open() or HDLC SNRM/UA)
/// - `SessionOpen` -> `Ready` (InitiateRequest/Response)
/// - Any state -> `Closed` (close())
///
/// # Error Handling
///
/// All operations return `DlmsResult` to handle errors from all layers:
/// - Transport errors: network issues, timeouts
/// - Session errors: frame errors, protocol violations
/// - Application errors: PDU errors, access denied
/// - Security errors: authentication failures, encryption errors
///
/// # Optimization Considerations
///
/// - Services are created once and reused for all operations
/// - Invoke IDs are managed automatically by services
/// - PDU encoding/decoding happens on-demand
/// - Future optimization: Connection pooling, request queuing, PDU caching
pub struct LnConnection {
    /// Connection state
    state: ConnectionState,
    /// Session layer (HDLC or Wrapper)
    session: Option<SessionLayer>,
    /// GET service
    get_service: GetService,
    /// SET service
    set_service: SetService,
    /// ACTION service
    action_service: ActionService,
    /// Connection configuration
    config: LnConnectionConfig,
    /// Negotiated conformance (from InitiateResponse)
    negotiated_conformance: Option<Conformance>,
    /// Server max PDU size (from InitiateResponse)
    server_max_pdu_size: Option<u16>,
}

impl LnConnection {
    /// Create a new LN connection with configuration
    ///
    /// # Arguments
    /// * `config` - Connection configuration
    ///
    /// # Returns
    /// A new LN connection in `Closed` state
    pub fn new(config: LnConnectionConfig) -> Self {
        Self {
            state: ConnectionState::Closed,
            session: None,
            get_service: GetService::new(),
            set_service: SetService::new(),
            action_service: ActionService::new(),
            config,
            negotiated_conformance: None,
            server_max_pdu_size: None,
        }
    }

    /// Send data through the session layer
    ///
    /// # Arguments
    /// * `data` - Data to send
    ///
    /// # Returns
    /// Ok(()) if successful
    ///
    /// # Errors
    /// Returns error if session is not established or sending fails
    async fn send_session_data(&mut self, data: &[u8]) -> DlmsResult<()> {
        match &mut self.session {
            Some(SessionLayer::Hdlc(_)) => {
                // TODO: Implement HDLC sending
                // For now, return error as HDLC connection needs proper type handling
                Err(DlmsError::InvalidData(
                    "HDLC session sending not yet fully implemented".to_string(),
                ))
            }
            Some(SessionLayer::Wrapper(wrapper)) => {
                // Cast to WrapperSession<TcpTransport> or WrapperSession<SerialTransport>
                // This is a limitation of the current design - we need to use generics or
                // a trait object approach
                // TODO: Refactor to use proper trait-based session layer
                Err(DlmsError::InvalidData(
                    "Wrapper session sending needs proper type handling".to_string(),
                ))
            }
            None => Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Session layer is not established",
            ))),
        }
    }

    /// Receive data from the session layer
    ///
    /// # Arguments
    /// * `timeout` - Optional timeout for receiving
    ///
    /// # Returns
    /// Received data
    ///
    /// # Errors
    /// Returns error if session is not established or receiving fails
    async fn receive_session_data(
        &mut self,
        timeout: Option<Duration>,
    ) -> DlmsResult<Vec<u8>> {
        match &mut self.session {
            Some(SessionLayer::Hdlc(_)) => {
                // TODO: Implement HDLC receiving
                Err(DlmsError::InvalidData(
                    "HDLC session receiving not yet fully implemented".to_string(),
                ))
            }
            Some(SessionLayer::Wrapper(_)) => {
                // TODO: Implement Wrapper receiving
                Err(DlmsError::InvalidData(
                    "Wrapper session receiving needs proper type handling".to_string(),
                ))
            }
            None => Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Session layer is not established",
            ))),
        }
    }
}

#[async_trait::async_trait]
impl Connection for LnConnection {
    async fn open(&mut self) -> DlmsResult<()> {
        if !matches!(self.state, ConnectionState::Closed) {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::AlreadyConnected,
                "Connection is already open",
            )));
        }

        // TODO: Implement full connection opening
        // 1. Determine session type (HDLC vs Wrapper) based on config
        // 2. Create and open transport layer
        // 3. Create and open session layer
        // 4. Send InitiateRequest
        // 5. Receive InitiateResponse
        // 6. Update state to Ready

        // For now, return error indicating implementation needed
        Err(DlmsError::InvalidData(
            "LnConnection::open() full implementation in progress".to_string(),
        ))
    }

    async fn close(&mut self) -> DlmsResult<()> {
        // TODO: Implement full connection closing
        // 1. Send Release Request (if needed)
        // 2. Close session layer (DISC/DM/UA for HDLC)
        // 3. Close transport layer
        // 4. Update state to Closed

        self.state = ConnectionState::Closed;
        self.session = None;
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.state.is_ready()
    }

    async fn get_attribute(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        attribute_id: u8,
    ) -> DlmsResult<DataObject> {
        if !self.is_open() {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Connection is not open",
            )));
        }

        // Create attribute descriptor with LN addressing
        let attribute_descriptor = CosemAttributeDescriptor {
            class_id,
            instance_id: LogicalNameReference::new(obis_code),
            attribute_id,
        };

        // Create GET request using GetService
        let invoke_id = self.get_service.next_invoke_id();
        let invoke_id_and_priority = InvokeIdAndPriority::new(invoke_id, false)
            .map_err(|e| DlmsError::InvalidData(format!("Invalid invoke ID: {}", e)))?;

        let request = GetRequest::new_normal(
            invoke_id_and_priority,
            attribute_descriptor,
            None, // No selective access
        );

        // Encode request
        let request_bytes = request.encode()?;

        // Send request and receive response
        let response_bytes = self.send_request(&request_bytes, Some(Duration::from_secs(30))).await?;

        // Decode response
        let response = GetResponse::decode(&response_bytes)?;

        // Process response using GetService
        self.get_service.process_response(&response)
    }

    async fn set_attribute(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        attribute_id: u8,
        value: DataObject,
    ) -> DlmsResult<()> {
        if !self.is_open() {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Connection is not open",
            )));
        }

        // Create attribute descriptor with LN addressing
        let ln_ref = LogicalNameReference::new(class_id, obis_code, attribute_id)?;
        let attribute_descriptor = CosemAttributeDescriptor::LogicalName(ln_ref);

        // Create SET request using SetService
        let invoke_id = self.set_service.next_invoke_id();
        let invoke_id_and_priority = InvokeIdAndPriority::new(invoke_id, false)
            .map_err(|e| DlmsError::InvalidData(format!("Invalid invoke ID: {}", e)))?;

        let request = SetRequest::new_normal(
            invoke_id_and_priority,
            attribute_descriptor,
            None, // No selective access
            value,
        );

        // Encode request
        let request_bytes = request.encode()?;

        // Send request and receive response
        let response_bytes = self.send_request(&request_bytes, Some(Duration::from_secs(30))).await?;

        // Decode response
        let response = SetResponse::decode(&response_bytes)?;

        // Process response using SetService
        self.set_service.process_response(&response)?;
        Ok(())
    }

    async fn invoke_method(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        method_id: u8,
        parameters: Option<DataObject>,
    ) -> DlmsResult<Option<DataObject>> {
        if !self.is_open() {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Connection is not open",
            )));
        }

        // Create method descriptor with LN addressing
        let ln_ref = LogicalNameReference::new(class_id, obis_code, method_id)?;
        let method_descriptor = CosemMethodDescriptor::LogicalName(ln_ref);

        // Create ACTION request using ActionService
        let invoke_id = self.action_service.next_invoke_id();
        let invoke_id_and_priority = InvokeIdAndPriority::new(invoke_id, false)
            .map_err(|e| DlmsError::InvalidData(format!("Invalid invoke ID: {}", e)))?;

        let request = ActionRequest::new_normal(
            invoke_id_and_priority,
            method_descriptor,
            parameters,
        );

        // Encode request
        let request_bytes = request.encode()?;

        // Send request and receive response
        let response_bytes = self.send_request(&request_bytes, Some(Duration::from_secs(30))).await?;

        // Decode response
        let response = ActionResponse::decode(&response_bytes)?;

        // Process response using ActionService
        self.action_service.process_response(&response)
    }

    async fn send_request(
        &mut self,
        request: &[u8],
        timeout: Option<Duration>,
    ) -> DlmsResult<Vec<u8>> {
        if !self.is_open() {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Connection is not open",
            )));
        }

        // Send request through session layer
        self.send_session_data(request).await?;

        // Receive response through session layer
        self.receive_session_data(timeout).await
    }
}
