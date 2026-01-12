//! DLMS/COSEM server listener implementation
//!
//! This module provides server-side connection listening and acceptance functionality.

use crate::server::{DlmsServer, AssociationContext};
use dlms_application::pdu::{
    InitiateRequest, InitiateResponse,
    GetRequest, GetResponse,
    SetRequest, SetResponse,
    ActionRequest, ActionResponse,
    AccessRequest, AccessResponse,
    ExceptionResponse,
};
use dlms_core::{DlmsError, DlmsResult};
use dlms_session::hdlc::{HdlcConnection, HdlcAddress};
use dlms_session::wrapper::WrapperSession;
use dlms_transport::{TcpTransport, TcpSettings, TransportLayer};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

/// Server listener for accepting client connections
///
/// Manages listening for incoming connections and handling them.
/// Supports both HDLC and Wrapper protocols over TCP.
///
/// # Architecture
/// The listener spawns a task for each accepted connection, allowing
/// concurrent handling of multiple clients.
///
/// # Usage Example
/// ```rust,no_run
/// use dlms_server::listener::ServerListener;
/// use dlms_server::server::DlmsServer;
///
/// let server = DlmsServer::new();
/// let listener = ServerListener::new(server, "0.0.0.0:4059".parse()?);
/// listener.start().await?;
/// ```
pub struct ServerListener {
    /// The DLMS server instance
    server: Arc<RwLock<DlmsServer>>,
    /// TCP listener address
    address: SocketAddr,
    /// HDLC local address (for HDLC connections)
    hdlc_local_address: HdlcAddress,
    /// Whether to use HDLC (true) or Wrapper (false) protocol
    use_hdlc: bool,
}

/// Client connection handler
///
/// Handles a single client connection, processing requests and sending responses.
struct ClientHandler {
    /// The DLMS server instance
    server: Arc<RwLock<DlmsServer>>,
    /// Client Service Access Point (SAP) address
    client_sap: u16,
    /// Whether connection uses HDLC (true) or Wrapper (false)
    use_hdlc: bool,
}

impl ServerListener {
    /// Create a new server listener
    ///
    /// # Arguments
    /// * `server` - The DLMS server instance
    /// * `address` - Address to listen on (e.g., "0.0.0.0:4059")
    ///
    /// # Defaults
    /// - Uses HDLC protocol
    /// - Local HDLC address: 0x01 (server)
    pub fn new(server: DlmsServer, address: SocketAddr) -> Self {
        Self {
            server: Arc::new(RwLock::new(server)),
            address,
            hdlc_local_address: HdlcAddress::new(0x01, 0x00), // Default server address
            use_hdlc: true,
        }
    }
    
    /// Set HDLC local address
    ///
    /// # Arguments
    /// * `address` - HDLC local address
    pub fn with_hdlc_address(mut self, address: HdlcAddress) -> Self {
        self.hdlc_local_address = address;
        self
    }
    
    /// Set protocol type
    ///
    /// # Arguments
    /// * `use_hdlc` - If true, use HDLC protocol; if false, use Wrapper protocol
    pub fn with_protocol(mut self, use_hdlc: bool) -> Self {
        self.use_hdlc = use_hdlc;
        self
    }
    
    /// Start listening for connections
    ///
    /// This method will block and accept connections indefinitely.
    /// Each accepted connection is handled in a separate task.
    ///
    /// # Errors
    /// Returns error if binding to the address fails
    pub async fn start(&self) -> DlmsResult<()> {
        let listener = TcpListener::bind(self.address).await
            .map_err(|e| DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::AddrNotAvailable,
                format!("Failed to bind to {}: {}", self.address, e),
            )))?;
        
        log::info!("DLMS server listening on {}", self.address);
        
        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    log::info!("Accepted connection from {}", peer_addr);
                    
                    // Extract client SAP from peer address or use default
                    // In real implementation, this might come from connection negotiation
                    let client_sap = Self::extract_client_sap(&peer_addr);
                    
                    // Spawn task to handle this connection
                    let server = self.server.clone();
                    let use_hdlc = self.use_hdlc;
                    let hdlc_local = self.hdlc_local_address;
                    
                    tokio::spawn(async move {
                        let handler = ClientHandler::new(server, client_sap, use_hdlc);
                        if let Err(e) = handler.handle_connection(stream, hdlc_local).await {
                            log::error!("Error handling connection from {}: {}", peer_addr, e);
                        }
                    });
                }
                Err(e) => {
                    log::error!("Error accepting connection: {}", e);
                    // Continue accepting other connections
                }
            }
        }
    }
    
    /// Extract client SAP from peer address
    ///
    /// This is a simplified implementation. In a real system, the client SAP
    /// might be negotiated during connection establishment or come from
    /// configuration.
    fn extract_client_sap(peer_addr: &SocketAddr) -> u16 {
        // Use port number as SAP (simplified)
        // In real implementation, this should come from HDLC address negotiation
        (peer_addr.port() % 65536) as u16
    }
}

impl ClientHandler {
    /// Create a new client handler
    fn new(
        server: Arc<RwLock<DlmsServer>>,
        client_sap: u16,
        use_hdlc: bool,
    ) -> Self {
        Self {
            server,
            client_sap,
            use_hdlc,
        }
    }
    
    /// Handle a client connection
    ///
    /// This method processes the connection lifecycle:
    /// 1. Establish session layer (HDLC or Wrapper)
    /// 2. Process Initiate Request/Response
    /// 3. Process GET/SET/ACTION requests
    /// 4. Clean up on disconnect
    async fn handle_connection(
        &self,
        stream: TcpStream,
        hdlc_local_address: HdlcAddress,
    ) -> DlmsResult<()> {
        // Create transport
        let tcp_settings = TcpSettings {
            read_timeout: Some(std::time::Duration::from_secs(30)),
            write_timeout: Some(std::time::Duration::from_secs(30)),
        };
        let transport = TcpTransport::new(stream, tcp_settings);
        
        if self.use_hdlc {
            self.handle_hdlc_connection(transport, hdlc_local_address).await
        } else {
            self.handle_wrapper_connection(transport).await
        }
    }
    
    /// Handle HDLC connection
    async fn handle_hdlc_connection(
        &self,
        transport: TcpTransport,
        local_address: HdlcAddress,
    ) -> DlmsResult<()> {
        // Create HDLC connection in server mode
        // Note: Remote address will be determined from SNRM/UA handshake
        // Server connections use LLC_RESPONSE header for responses
        let remote_address = HdlcAddress::new(0x10, 0x00); // Default client address (will be updated from SNRM)
        let mut hdlc_conn = HdlcConnection::new_server(transport, local_address, remote_address);
        
        // Wait for SNRM frame and respond with UA (server-side handshake)
        // This implements the server-side of the SNRM/UA handshake:
        // 1. Wait for SNRM frame from client
        // 2. Generate UA response with server parameters
        // 3. Send UA frame to client
        // 4. Update connection state to Connected
        hdlc_conn.accept().await?;
        
        // Process Initiate Request
        self.process_initiate(&mut hdlc_conn).await?;
        
        // Process requests in a loop
        loop {
            // Receive data from client
            let data = match hdlc_conn.receive_segmented(Some(std::time::Duration::from_secs(30))).await {
                Ok(data) => data,
                Err(e) => {
                    log::error!("Error receiving data: {}", e);
                    break;
                }
            };
            
            // Parse and process request
            match self.parse_and_route_request_hdlc(&data, hdlc_conn).await {
                Ok(_) => {
                    // Request processed successfully
                }
                Err(e) => {
                    log::error!("Error processing request: {}", e);
                    // Send exception response if possible
                    if let Err(send_err) = self.send_exception_response_hdlc(hdlc_conn, e).await {
                        log::error!("Failed to send exception response: {}", send_err);
                    }
                    // Continue processing other requests
                }
            }
        }
        
        // Clean up association
        {
            let mut server = self.server.write().await;
            server.release_association(self.client_sap).await;
        }
        
        Ok(())
    }
    
    /// Handle Wrapper connection
    async fn handle_wrapper_connection(
        &self,
        transport: TcpTransport,
    ) -> DlmsResult<()> {
        // Create Wrapper session
        let mut wrapper = WrapperSession::new(transport, 0x01, 0x10); // Server ID, Client ID
        
        // Process Initiate Request
        self.process_initiate_wrapper(&mut wrapper).await?;
        
        // Process requests in a loop
        loop {
            // Receive data from client
            let data = match wrapper.receive().await {
                Ok(data) => data,
                Err(e) => {
                    log::error!("Error receiving data: {}", e);
                    break;
                }
            };
            
            // Parse and process request
            match self.parse_and_route_request_wrapper(&data, wrapper).await {
                Ok(_) => {
                    // Request processed successfully
                }
                Err(e) => {
                    log::error!("Error processing request: {}", e);
                    // Send exception response if possible
                    if let Err(send_err) = self.send_exception_response_wrapper(wrapper, e).await {
                        log::error!("Failed to send exception response: {}", send_err);
                    }
                    // Continue processing other requests
                }
            }
        }
        
        // Clean up association
        {
            let mut server = self.server.write().await;
            server.release_association(self.client_sap).await;
        }
        
        Ok(())
    }
    
    /// Process Initiate Request for HDLC connection
    async fn process_initiate(
        &self,
        hdlc_conn: &mut HdlcConnection<TcpTransport>,
    ) -> DlmsResult<()> {
        // Receive Initiate Request
        let data = hdlc_conn.receive_segmented(Some(std::time::Duration::from_secs(10))).await?;
        
        // Parse Initiate Request
        let request = InitiateRequest::decode(&data)?;
        
        // Handle request
        let server = self.server.read().await;
        let response = server.handle_initiate_request(&request, self.client_sap).await?;
        
        // Send response
        let response_data = response.encode()?;
        hdlc_conn.send_information(response_data, false).await?;
        
        Ok(())
    }
    
    /// Process Initiate Request for Wrapper connection
    async fn process_initiate_wrapper(
        &self,
        wrapper: &mut WrapperSession<TcpTransport>,
    ) -> DlmsResult<()> {
        // Receive Initiate Request
        let data = wrapper.receive().await?;
        
        // Parse Initiate Request
        let request = InitiateRequest::decode(&data)?;
        
        // Handle request
        let server = self.server.read().await;
        let response = server.handle_initiate_request(&request, self.client_sap).await?;
        
        // Send response
        let response_data = response.encode()?;
        wrapper.send(&response_data).await?;
        
        Ok(())
    }
    
    /// Parse and route request for HDLC connection
    ///
    /// # Process
    /// 1. Identify PDU type from first byte (tag)
    /// 2. Decode the appropriate PDU
    /// 3. Route to corresponding handler method
    /// 4. Encode and send response
    ///
    /// # PDU Type Tags (DLMS standard)
    /// - InitiateRequest: 1
    /// - InitiateResponse: 2
    /// - GetRequest: 192 (0xC0)
    /// - GetResponse: 196 (0xC4)
    /// - SetRequest: 193 (0xC1)
    /// - SetResponse: 197 (0xC5)
    /// - ActionRequest: 195 (0xC3)
    /// - ActionResponse: 199 (0xC7)
    /// - AccessRequest: 3
    /// - AccessResponse: 4
    async fn parse_and_route_request_hdlc(
        &self,
        data: &[u8],
        hdlc_conn: &mut HdlcConnection<TcpTransport>,
    ) -> DlmsResult<()> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData("Empty request data".to_string()));
        }
        
        // Identify PDU type from first byte
        let pdu_tag = data[0];
        
        match pdu_tag {
            // GetRequest: 192 (0xC0)
            192 => {
                let request = GetRequest::decode(data)?;
                let server = self.server.read().await;
                let response = server.handle_get_request(&request, self.client_sap).await?;
                let response_data = response.encode()?;
                hdlc_conn.send_information(response_data, false).await?;
                Ok(())
            }
            // SetRequest: 193 (0xC1)
            193 => {
                let request = SetRequest::decode(data)?;
                let server = self.server.read().await;
                let response = server.handle_set_request(&request, self.client_sap).await?;
                let response_data = response.encode()?;
                hdlc_conn.send_information(response_data, false).await?;
                Ok(())
            }
            // ActionRequest: 195 (0xC3)
            195 => {
                let request = ActionRequest::decode(data)?;
                let server = self.server.read().await;
                let response = server.handle_action_request(&request, self.client_sap).await?;
                let response_data = response.encode()?;
                hdlc_conn.send_information(response_data, false).await?;
                Ok(())
            }
            // AccessRequest: 3
            3 => {
                let request = AccessRequest::decode(data)?;
                let server = self.server.read().await;
                let response = server.handle_access_request(&request, self.client_sap).await?;
                let response_data = response.encode()?;
                hdlc_conn.send_information(response_data, false).await?;
                Ok(())
            }
            // InitiateRequest: 1 (should be handled separately, but handle here for robustness)
            1 => {
                // This should have been handled in process_initiate, but handle here as fallback
                let request = InitiateRequest::decode(data)?;
                let server = self.server.read().await;
                let response = server.handle_initiate_request(&request, self.client_sap).await?;
                let response_data = response.encode()?;
                hdlc_conn.send_information(response_data, false).await?;
                Ok(())
            }
            _ => {
                Err(DlmsError::InvalidData(format!(
                    "Unknown PDU type tag: 0x{:02X}",
                    pdu_tag
                )))
            }
        }
    }
    
    /// Parse and route request for Wrapper connection
    ///
    /// Similar to parse_and_route_request_hdlc but uses WrapperSession for sending responses.
    async fn parse_and_route_request_wrapper(
        &self,
        data: &[u8],
        wrapper: &mut WrapperSession<TcpTransport>,
    ) -> DlmsResult<()> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData("Empty request data".to_string()));
        }
        
        // Identify PDU type from first byte
        let pdu_tag = data[0];
        
        match pdu_tag {
            // GetRequest: 192 (0xC0)
            192 => {
                let request = GetRequest::decode(data)?;
                let server = self.server.read().await;
                let response = server.handle_get_request(&request, self.client_sap).await?;
                let response_data = response.encode()?;
                wrapper.send(&response_data).await?;
                Ok(())
            }
            // SetRequest: 193 (0xC1)
            193 => {
                let request = SetRequest::decode(data)?;
                let server = self.server.read().await;
                let response = server.handle_set_request(&request, self.client_sap).await?;
                let response_data = response.encode()?;
                wrapper.send(&response_data).await?;
                Ok(())
            }
            // ActionRequest: 195 (0xC3)
            195 => {
                let request = ActionRequest::decode(data)?;
                let server = self.server.read().await;
                let response = server.handle_action_request(&request, self.client_sap).await?;
                let response_data = response.encode()?;
                wrapper.send(&response_data).await?;
                Ok(())
            }
            // AccessRequest: 3
            3 => {
                let request = AccessRequest::decode(data)?;
                let server = self.server.read().await;
                let response = server.handle_access_request(&request, self.client_sap).await?;
                let response_data = response.encode()?;
                wrapper.send(&response_data).await?;
                Ok(())
            }
            // InitiateRequest: 1 (should be handled separately, but handle here for robustness)
            1 => {
                // This should have been handled in process_initiate_wrapper, but handle here as fallback
                let request = InitiateRequest::decode(data)?;
                let server = self.server.read().await;
                let response = server.handle_initiate_request(&request, self.client_sap).await?;
                let response_data = response.encode()?;
                wrapper.send(&response_data).await?;
                Ok(())
            }
            _ => {
                Err(DlmsError::InvalidData(format!(
                    "Unknown PDU type tag: 0x{:02X}",
                    pdu_tag
                )))
            }
        }
    }
    
    /// Send exception response for HDLC connection
    ///
    /// Creates an ExceptionResponse PDU from an error and sends it to the client.
    async fn send_exception_response_hdlc(
        &self,
        hdlc_conn: &mut HdlcConnection<TcpTransport>,
        error: DlmsError,
    ) -> DlmsResult<()> {
        // Convert error to exception response
        // For now, use a generic state error
        let exception_response = ExceptionResponse::new(
            dlms_application::pdu::ExceptionState::StateError,
            dlms_application::pdu::ExceptionServiceError::Other,
        )?;
        
        let response_data = exception_response.encode()?;
        hdlc_conn.send_information(response_data, false).await?;
        
        Ok(())
    }
    
    /// Send exception response for Wrapper connection
    ///
    /// Similar to send_exception_response_hdlc but uses WrapperSession.
    async fn send_exception_response_wrapper(
        &self,
        wrapper: &mut WrapperSession<TcpTransport>,
        error: DlmsError,
    ) -> DlmsResult<()> {
        // Convert error to exception response
        // For now, use a generic state error
        let exception_response = ExceptionResponse::new(
            dlms_application::pdu::ExceptionState::StateError,
            dlms_application::pdu::ExceptionServiceError::Other,
        )?;
        
        let response_data = exception_response.encode()?;
        wrapper.send(&response_data).await?;
        
        Ok(())
    }
}
