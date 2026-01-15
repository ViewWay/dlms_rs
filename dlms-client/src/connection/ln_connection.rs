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
use dlms_transport::{TcpTransport, SerialTransport, TcpSettings, SerialSettings, TransportLayer};
use dlms_security::SecuritySuite;
use std::time::Duration;
use std::net::SocketAddr;

/// Session layer type
///
/// Distinguishes between HDLC and Wrapper session layers, which have different
/// connection establishment procedures and data framing.
#[derive(Debug)]
pub(crate) enum SessionLayer {
    /// HDLC session with TCP transport
    HdlcTcp(HdlcConnection<TcpTransport>),
    /// HDLC session with Serial transport
    HdlcSerial(HdlcConnection<SerialTransport>),
    /// Wrapper session with TCP transport
    WrapperTcp(WrapperSession<TcpTransport>),
    /// Wrapper session with Serial transport (rare, but possible)
    WrapperSerial(WrapperSession<SerialTransport>),
}

/// Transport configuration
#[derive(Debug, Clone)]
pub(crate) enum TransportConfig {
    Tcp { address: String },
    Serial { port_name: String, baud_rate: u32 },
}

/// Logical Name (LN) connection configuration
///
/// Configuration parameters for establishing an LN connection, including
/// transport settings, session parameters, and security configuration.
#[derive(Debug, Clone)]
pub struct LnConnectionConfig {
    /// Transport configuration
    pub transport: Option<TransportConfig>,
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
            transport: None,
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
    async fn send_session_data(&mut self, data: &[u8]) -> DlmsResult<()> {
        match &mut self.session {
            Some(SessionLayer::HdlcTcp(hdlc)) => {
                hdlc.send_information(data.to_vec(), false).await
            }
            Some(SessionLayer::HdlcSerial(hdlc)) => {
                hdlc.send_information(data.to_vec(), false).await
            }
            Some(SessionLayer::WrapperTcp(wrapper)) => {
                wrapper.send(data).await
            }
            Some(SessionLayer::WrapperSerial(wrapper)) => {
                wrapper.send(data).await
            }
            None => Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Session layer is not established",
            ))),
        }
    }

    /// Receive data from the session layer
    async fn receive_session_data(
        &mut self,
        timeout: Option<Duration>,
    ) -> DlmsResult<Vec<u8>> {
        match &mut self.session {
            Some(SessionLayer::HdlcTcp(hdlc)) => {
                hdlc.receive_segmented(timeout).await
            }
            Some(SessionLayer::HdlcSerial(hdlc)) => {
                hdlc.receive_segmented(timeout).await
            }
            Some(SessionLayer::WrapperTcp(wrapper)) => {
                wrapper.receive(timeout).await
            }
            Some(SessionLayer::WrapperSerial(wrapper)) => {
                wrapper.receive(timeout).await
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
                std::io::ErrorKind::InvalidInput,
                "Connection is already open",
            )));
        }

        // Step 1: Determine session type and create transport
        // The transport configuration must be set by ConnectionBuilder::build_ln().
        // If it's None, it means the builder didn't properly transfer the transport type.
        let transport_config = self.config.transport.as_ref().ok_or_else(|| {
            DlmsError::InvalidData(
                "Transport configuration is required. This should be set by ConnectionBuilder::build_ln().".to_string()
            )
        })?;

        // Step 2: Create and open transport layer, then create session layer
        let session = match transport_config {
            TransportConfig::Tcp { address } => {
                // Parse TCP address
                let addr: SocketAddr = address.parse().map_err(|e| {
                    DlmsError::InvalidData(format!("Invalid TCP address '{}': {}", address, e))
                })?;
                let tcp_settings = TcpSettings::new(addr);
                let tcp_transport = TcpTransport::new(tcp_settings);
                
                // Determine session type based on config
                if self.config.local_address.is_some() && self.config.remote_address.is_some() {
                    // Use HDLC over TCP
                    let local_addr = self.config.local_address.unwrap();
                    let remote_addr = self.config.remote_address.unwrap();
                    let mut hdlc = HdlcConnection::new(tcp_transport, local_addr, remote_addr);
                    hdlc.open().await?;
                    SessionLayer::HdlcTcp(hdlc)
                } else {
                    // Use Wrapper over TCP
                    let client_id = self.config.client_id.unwrap_or(0x10);
                    let logical_device_id = self.config.logical_device_id.unwrap_or(0x01);
                    let mut wrapper = WrapperSession::new(tcp_transport, client_id, logical_device_id);
                    wrapper.open().await?;
                    SessionLayer::WrapperTcp(wrapper)
                }
            }
            TransportConfig::Serial { port_name, baud_rate } => {
                // Create Serial transport
                let serial_settings = SerialSettings::new(port_name.clone(), *baud_rate);
                let serial_transport = SerialTransport::new(serial_settings);
                
                // Serial typically uses HDLC
                let local_addr = self.config.local_address.ok_or_else(|| {
                    DlmsError::InvalidData("HDLC local address is required for Serial transport".to_string())
                })?;
                let remote_addr = self.config.remote_address.ok_or_else(|| {
                    DlmsError::InvalidData("HDLC remote address is required for Serial transport".to_string())
                })?;
                
                let mut hdlc = HdlcConnection::new(serial_transport, local_addr, remote_addr);
                hdlc.open().await?;
                SessionLayer::HdlcSerial(hdlc)
            }
        };

        self.session = Some(session);
        self.state = ConnectionState::SessionOpen;

        // Step 3: Send InitiateRequest
        let initiate_request = InitiateRequest {
            proposed_dlms_version_number: self.config.dlms_version,
            proposed_conformance: self.config.conformance.clone(),
            client_max_receive_pdu_size: self.config.max_pdu_size,
            proposed_quality_of_service: None,
            response_allowed: true,
            dedicated_key: None,
        };

        let request_bytes = initiate_request.encode()?;
        self.send_session_data(&request_bytes).await?;

        // Step 4: Receive InitiateResponse
        let response_bytes = self.receive_session_data(Some(Duration::from_secs(30))).await?;
        let initiate_response = InitiateResponse::decode(&response_bytes)?;

        // Step 5: Update negotiated parameters
        self.negotiated_conformance = Some(initiate_response.negotiated_conformance.clone());
        self.server_max_pdu_size = Some(initiate_response.server_max_receive_pdu_size);

        // Step 6: Update state to Ready
        self.state = ConnectionState::Ready;

        Ok(())
    }

    async fn close(&mut self) -> DlmsResult<()> {
        // Close session layer
        match &mut self.session {
            Some(SessionLayer::HdlcTcp(hdlc)) => {
                hdlc.close().await?;
            }
            Some(SessionLayer::HdlcSerial(hdlc)) => {
                hdlc.close().await?;
            }
            Some(SessionLayer::WrapperTcp(wrapper)) => {
                wrapper.close().await?;
            }
            Some(SessionLayer::WrapperSerial(wrapper)) => {
                wrapper.close().await?;
            }
            None => {
                // Already closed
            }
        }

        self.session = None;
        self.state = ConnectionState::Closed;
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
        let ln_ref = LogicalNameReference::new(class_id, obis_code, attribute_id)?;
        let attribute_descriptor = CosemAttributeDescriptor::LogicalName(ln_ref);

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
        GetService::process_response(&response)
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
        SetService::process_response(&response)?;
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
        ActionService::process_response(&response)
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
