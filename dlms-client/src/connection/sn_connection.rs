//! Short Name (SN) addressing connection for DLMS/COSEM client
//!
//! This module provides the Short Name (SN) connection implementation, which uses
//! 16-bit addresses to identify COSEM objects.
//!
//! # Architecture
//!
//! SN connections integrate multiple protocol layers:
//! - **Transport Layer**: TCP, UDP, or Serial
//! - **Session Layer**: HDLC or Wrapper
//! - **Security Layer**: Encryption and authentication (future)
//! - **Application Layer**: PDU encoding/decoding with SN addressing
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
//! # Why SN Addressing?
//! Short Name addressing uses 16-bit addresses (2 bytes) to identify objects.
//! This provides:
//! - **Compact**: More efficient than LN addressing (2 bytes vs 6 bytes)
//! - **Fast**: Direct addressing without OBIS code lookup
//! - **Legacy Support**: Required for older DLMS implementations
//!
//! # Limitations
//! - Requires address mapping table (Association SN object, class ID 12)
//! - Less human-readable than OBIS codes
//! - Address mapping must be established before use

use super::connection::{Connection, ConnectionState};
use dlms_application::service::{GetService, SetService, ActionService};
use dlms_application::pdu::{
    InitiateRequest, InitiateResponse, GetRequest, GetResponse, SetRequest, SetResponse,
    ActionRequest, ActionResponse, CosemAttributeDescriptor, CosemMethodDescriptor,
    InvokeIdAndPriority, Conformance,
};
use dlms_application::addressing::ShortNameReference;
use dlms_core::{DlmsError, DlmsResult, DataObject};
use dlms_session::hdlc::{HdlcConnection, HdlcAddress};
use dlms_session::wrapper::WrapperSession;
use dlms_transport::{TcpTransport, SerialTransport, TcpSettings, SerialSettings, TransportLayer};
use dlms_security::SecuritySuite;
use std::time::Duration;
use std::net::SocketAddr;

// Re-use session layer and transport config from LN connection
// These are internal types shared between LN and SN connections
use super::ln_connection::{SessionLayer, TransportConfig};

/// Short Name (SN) connection configuration
///
/// Configuration parameters for establishing an SN connection, including
/// transport settings, session parameters, and security configuration.
///
/// # Note
/// SN connections use the same transport and session layer configuration as LN connections.
/// The main difference is in the addressing method used for object references.
#[derive(Debug, Clone)]
pub struct SnConnectionConfig {
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

impl Default for SnConnectionConfig {
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

/// Short Name (SN) connection implementation
///
/// Provides a high-level interface for DLMS/COSEM operations using
/// short name addressing (16-bit addresses).
pub struct SnConnection {
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
    config: SnConnectionConfig,
    /// Negotiated conformance (from InitiateResponse)
    negotiated_conformance: Option<Conformance>,
    /// Server max PDU size (from InitiateResponse)
    server_max_pdu_size: Option<u16>,
}

impl SnConnection {
    /// Create a new SN connection with configuration
    pub fn new(config: SnConnectionConfig) -> Self {
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
impl Connection for SnConnection {
    async fn open(&mut self) -> DlmsResult<()> {
        if !matches!(self.state, ConnectionState::Closed) {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::AlreadyConnected,
                "Connection is already open",
            )));
        }

        // Step 1: Determine session type and create transport
        let transport_config = self.config.transport.as_ref().ok_or_else(|| {
            DlmsError::InvalidData("Transport configuration is required".to_string())
        })?;

        // Step 2: Create and open transport layer, then create session layer
        // Re-use the same logic as LnConnection
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
        obis_code: dlms_core::ObisCode,
        class_id: u16,
        attribute_id: u8,
    ) -> DlmsResult<DataObject> {
        // SN addressing uses base_name instead of OBIS code
        // For now, we'll use the base_name from a mapping, but typically
        // the user should provide the base_name directly
        // TODO: Add OBIS to base_name mapping support
        
        // For SN addressing, we need the base_name (16-bit address)
        // This is a limitation - we should accept base_name directly
        // For now, return an error indicating SN addressing needs base_name
        Err(DlmsError::InvalidData(
            "SN addressing requires base_name (16-bit address) instead of OBIS code. Use get_attribute_by_base_name() instead.".to_string(),
        ))
    }

    /// Get an attribute value using short name addressing
    ///
    /// # Arguments
    /// * `base_name` - 16-bit base address of the object
    /// * `class_id` - Class ID of the object
    /// * `attribute_id` - Attribute ID to read
    ///
    /// # Returns
    /// The attribute value as a `DataObject`
    ///
    /// # Errors
    /// Returns error if the connection is not open, if the request fails, or if the response indicates an error
    pub async fn get_attribute_by_base_name(
        &mut self,
        base_name: u16,
        class_id: u16,
        attribute_id: u8,
    ) -> DlmsResult<DataObject> {
        if !self.is_open() {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Connection is not open",
            )));
        }

        // Create attribute descriptor with SN addressing
        let sn_ref = ShortNameReference::new(base_name, attribute_id)?;
        let attribute_descriptor = CosemAttributeDescriptor::ShortName(sn_ref);

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
        obis_code: dlms_core::ObisCode,
        class_id: u16,
        attribute_id: u8,
        value: DataObject,
    ) -> DlmsResult<()> {
        // SN addressing requires base_name
        Err(DlmsError::InvalidData(
            "SN addressing requires base_name (16-bit address) instead of OBIS code. Use set_attribute_by_base_name() instead.".to_string(),
        ))
    }

    /// Set an attribute value using short name addressing
    ///
    /// # Arguments
    /// * `base_name` - 16-bit base address of the object
    /// * `class_id` - Class ID of the object
    /// * `attribute_id` - Attribute ID to write
    /// * `value` - Value to write
    ///
    /// # Returns
    /// Ok(()) if successful
    ///
    /// # Errors
    /// Returns error if the connection is not open, if the request fails, or if the response indicates an error
    pub async fn set_attribute_by_base_name(
        &mut self,
        base_name: u16,
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

        // Create attribute descriptor with SN addressing
        let sn_ref = ShortNameReference::new(base_name, attribute_id)?;
        let attribute_descriptor = CosemAttributeDescriptor::ShortName(sn_ref);

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
        obis_code: dlms_core::ObisCode,
        class_id: u16,
        method_id: u8,
        parameters: Option<DataObject>,
    ) -> DlmsResult<Option<DataObject>> {
        // SN addressing requires base_name
        Err(DlmsError::InvalidData(
            "SN addressing requires base_name (16-bit address) instead of OBIS code. Use invoke_method_by_base_name() instead.".to_string(),
        ))
    }

    /// Invoke a method using short name addressing
    ///
    /// # Arguments
    /// * `base_name` - 16-bit base address of the object
    /// * `class_id` - Class ID of the object
    /// * `method_id` - Method ID to invoke
    /// * `parameters` - Optional method parameters
    ///
    /// # Returns
    /// Optional return value from the method (if any)
    ///
    /// # Errors
    /// Returns error if the connection is not open, if the request fails, or if the response indicates an error
    pub async fn invoke_method_by_base_name(
        &mut self,
        base_name: u16,
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

        // Create method descriptor with SN addressing
        let sn_ref = ShortNameReference::new(base_name, method_id)?;
        let method_descriptor = CosemMethodDescriptor::ShortName(sn_ref);

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
