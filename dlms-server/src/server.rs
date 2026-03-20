//! DLMS/COSEM server implementation
//!
//! This module provides server-side functionality for DLMS/COSEM protocol,
//! including object management, request handling, and association management.

use crate::connection_manager::{ConnectionManager, ConnectionInfo, ConnectionStatistics};
use crate::access_control::{AccessControlManager, AccessControlList};
use dlms_application::pdu::{
    GetRequest, GetResponse, SetRequest, SetResponse, ActionRequest, ActionResponse,
    InitiateRequest, InitiateResponse, AccessRequest, AccessResponse,
    AccessRequestSpecification, AccessResponseSpecification,
    CosemAttributeDescriptor, CosemMethodDescriptor, GetDataResult, SetDataResult, ActionResult,
    InvokeIdAndPriority, Conformance,
    SetRequestWithList,
};
use dlms_core::{DlmsError, DlmsResult, ObisCode};
use dlms_security::SecuritySuite;
use dlms_interface::CosemObject;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Association context
///
/// Tracks information about an active association (connection) with a client.
/// Similar to C++ implementation's `AssociationContext` struct.
#[derive(Debug, Clone)]
pub struct AssociationContext {
    /// Client Service Access Point (SAP) address
    pub client_sap: u16,
    /// Server Service Access Point (SAP) address
    pub server_sap: u16,
    /// Security options for this association
    pub security_options: SecuritySuite,
    /// Negotiated conformance bits
    pub conformance: Conformance,
    /// Maximum PDU size for this association
    pub max_pdu_size: u16,
    /// DLMS version (typically 6)
    pub dlms_version: u8,
}

/// Block transfer state for GetRequest-Next
///
/// Tracks the state of an ongoing block transfer for reading large attributes.
#[derive(Debug, Clone)]
struct BlockTransferState {
    /// Invoke ID of the original request (reserved for future use)
    _invoke_id: u8,
    /// OBIS code of the object being read (reserved for future use)
    _obis_code: ObisCode,
    /// Attribute ID being read (reserved for future use)
    _attribute_id: u8,
    /// Total data being transferred (all blocks)
    total_data: Vec<u8>,
    /// Current block size (bytes per block)
    block_size: usize,
    /// Current block number
    current_block: u32,
    /// Last block flag
    last_block: bool,
}

impl BlockTransferState {
    /// Create a new block transfer state
    fn new(invoke_id: u8, obis_code: ObisCode, attribute_id: u8, data: Vec<u8>, block_size: usize) -> Self {
        let total_blocks = (data.len() + block_size - 1) / block_size;
        let last_block = total_blocks <= 1;

        Self {
            _invoke_id: invoke_id,
            _obis_code: obis_code,
            _attribute_id: attribute_id,
            total_data: data,
            block_size,
            current_block: 0,
            last_block,
        }
    }

    /// Get the current block of data
    fn get_current_block(&self) -> Vec<u8> {
        let start = (self.current_block as usize) * self.block_size;
        let end = (start + self.block_size).min(self.total_data.len());
        self.total_data[start..end].to_vec()
    }

    /// Check if this is the last block
    fn is_last_block(&self) -> bool {
        let next_block = self.current_block + 1;
        let start = (next_block as usize) * self.block_size;
        start >= self.total_data.len()
    }

    /// Advance to the next block
    fn advance(&mut self) -> bool {
        if self.is_last_block() {
            return false; // Already at last block
        }
        self.current_block += 1;
        self.last_block = self.is_last_block();
        true
    }
}

/// DLMS/COSEM server
///
/// Main server implementation that manages:
/// - COSEM object registry
/// - Association management
/// - Connection management
/// - Access control (ACL)
/// - Request handling (GET, SET, ACTION)
/// - Response generation
/// - Short Name addressing support
///
/// # Architecture
/// The server follows a similar architecture to the C++ reference implementation:
/// - `LogicalDevice`: Manages objects and associations
/// - `Association`: Tracks active client connections
/// - `CosemObject`: Interface for all COSEM objects
///
/// # Usage Example
/// ```rust,no_run
/// use dlms_server::{DlmsServer, CosemObject};
/// use dlms_core::ObisCode;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create server
/// let server = DlmsServer::new();
///
/// // Register objects
/// // server.register_object(my_object).await?;
///
/// // Register short name mapping
/// // server.register_short_name(0x1000, ObisCode::from([1, 1, 0, 0, 0, 0])).await;
///
/// // Get connection statistics
/// let stats = server.get_connection_statistics().await;
/// println!("Active connections: {}", stats.active_connections);
/// # Ok(())
/// # }
/// ```
pub struct DlmsServer {
    /// Registered COSEM objects (indexed by OBIS code)
    objects: Arc<RwLock<HashMap<ObisCode, Arc<dyn CosemObject>>>>,
    /// Active associations (indexed by client SAP)
    associations: Arc<RwLock<HashMap<u16, AssociationContext>>>,
    /// Connection manager for tracking active connections
    connection_manager: Arc<ConnectionManager>,
    /// Access control manager for permissions
    access_control: Arc<AccessControlManager>,
    /// Server configuration
    config: ServerConfig,
    /// Block transfer states (indexed by client SAP + invoke ID)
    block_transfers: Arc<RwLock<HashMap<(u16, u8), BlockTransferState>>>,
    /// Short Name (base_name) to OBIS code mapping for SN addressing
    base_name_to_obis: Arc<RwLock<HashMap<u16, ObisCode>>>,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server Service Access Point (SAP) address
    pub server_sap: u16,
    /// Default security suite
    pub default_security: SecuritySuite,
    /// Default conformance bits
    pub default_conformance: Conformance,
    /// Default maximum PDU size
    pub max_pdu_size: u16,
    /// DLMS version (typically 6)
    pub dlms_version: u8,
    /// Maximum number of concurrent connections (0 = unlimited)
    pub max_connections: usize,
    /// Connection idle timeout in seconds
    pub connection_idle_timeout_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server_sap: 1,
            default_security: SecuritySuite::default(),
            default_conformance: Conformance::default(),
            max_pdu_size: 1024,
            dlms_version: 6,
            max_connections: 100,
            connection_idle_timeout_secs: 300, // 5 minutes
        }
    }
}

impl DlmsServer {
    /// Create a new DLMS server with default configuration
    pub fn new() -> Self {
        Self::with_config(ServerConfig::default())
    }

    /// Create a new DLMS server with custom configuration
    pub fn with_config(config: ServerConfig) -> Self {
        let connection_manager = Arc::new(ConnectionManager::new(
            config.max_connections,
            Duration::from_secs(config.connection_idle_timeout_secs),
        ));

        let access_control = Arc::new(AccessControlManager::new());

        Self {
            objects: Arc::new(RwLock::new(HashMap::new())),
            associations: Arc::new(RwLock::new(HashMap::new())),
            connection_manager,
            access_control,
            config,
            block_transfers: Arc::new(RwLock::new(HashMap::new())),
            base_name_to_obis: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the connection manager
    pub fn connection_manager(&self) -> &Arc<ConnectionManager> {
        &self.connection_manager
    }

    /// Get the access control manager
    pub fn access_control(&self) -> &Arc<AccessControlManager> {
        &self.access_control
    }

    /// Get connection statistics
    ///
    /// # Returns
    /// Current connection statistics
    pub async fn get_connection_statistics(&self) -> ConnectionStatistics {
        self.connection_manager.get_statistics().await
    }

    /// Get all active connections
    ///
    /// # Returns
    /// Vector of all active connection info
    pub async fn get_active_connections(&self) -> Vec<ConnectionInfo> {
        self.connection_manager.get_all_connections().await
    }

    /// Cleanup stale connections
    ///
    /// Removes connections that have been idle longer than the configured timeout.
    ///
    /// # Returns
    /// Number of connections removed
    pub async fn cleanup_stale_connections(&self) -> usize {
        self.connection_manager.cleanup_stale_connections().await
    }

    /// Disconnect a specific client
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    ///
    /// # Returns
    /// true if the client was disconnected, false if they weren't connected
    pub async fn disconnect_client(&self, client_sap: u16) -> bool {
        // Also remove from associations
        self.release_association(client_sap).await;
        self.connection_manager.disconnect_client(client_sap).await
    }

    /// Register an ACL for a client
    ///
    /// # Arguments
    /// * `acl` - Access control list
    pub async fn register_acl(&self, acl: AccessControlList) {
        self.access_control.register_acl(acl).await;
    }

    /// Unregister an ACL for a client
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    pub async fn unregister_acl(&self, client_sap: u16) {
        self.access_control.unregister_acl(client_sap).await;
    }

    /// Enable or disable access control
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable access control
    pub async fn set_access_control_enabled(&self, enabled: bool) {
        self.access_control.set_enabled(enabled).await;
    }
    
    /// Register a COSEM object with the server
    ///
    /// # Arguments
    /// * `object` - The COSEM object to register
    ///
    /// # Errors
    /// Returns error if an object with the same OBIS code is already registered
    pub async fn register_object(&self, object: Arc<dyn CosemObject>) -> DlmsResult<()> {
        let mut objects = self.objects.write().await;
        let obis = object.obis_code();
        
        if objects.contains_key(&obis) {
            return Err(DlmsError::InvalidData(format!(
                "Object with OBIS code {} is already registered",
                obis
            )));
        }
        
        objects.insert(obis, object);
        Ok(())
    }
    
    /// Unregister a COSEM object
    ///
    /// # Arguments
    /// * `obis_code` - OBIS code of the object to unregister
    pub async fn unregister_object(&self, obis_code: &ObisCode) {
        let mut objects = self.objects.write().await;
        objects.remove(obis_code);
    }
    
    /// Find an object by OBIS code
    ///
    /// # Arguments
    /// * `obis_code` - OBIS code to search for
    ///
    /// # Returns
    /// Reference to the object if found, `None` otherwise
    pub async fn find_object(&self, obis_code: &ObisCode) -> Option<Arc<dyn CosemObject>> {
        let objects = self.objects.read().await;
        objects.get(obis_code).cloned()
    }

    /// Register a Short Name (base_name) to OBIS code mapping
    ///
    /// This enables Short Name addressing for COSEM objects. When a client
    /// uses SN addressing with a registered base_name, the server will
    /// automatically resolve it to the corresponding OBIS code.
    ///
    /// # Arguments
    /// * `base_name` - 16-bit base name (short name address)
    /// * `obis_code` - OBIS code to map to
    ///
    /// # Errors
    /// Returns error if the base_name is already registered to a different OBIS code
    ///
    /// # Example
    /// ```rust,no_run
    /// # use dlms_server::DlmsServer;
    /// # use dlms_core::ObisCode;
    /// # async fn example(server: DlmsServer) -> Result<(), Box<dyn std::error::Error>> {
    /// // Map short name 0x1000 to logical name "1.1.0.0.0.0"
    /// server.register_short_name(0x1000, ObisCode::from([1, 1, 0, 0, 0, 0])).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn register_short_name(&self, base_name: u16, obis_code: ObisCode) -> DlmsResult<()> {
        let mut mapping = self.base_name_to_obis.write().await;

        // Check if base_name already exists with different OBIS code
        if let Some(existing_obis) = mapping.get(&base_name) {
            if existing_obis != &obis_code {
                return Err(DlmsError::InvalidData(format!(
                    "Base name 0x{:04X} is already mapped to OBIS {}",
                    base_name, existing_obis
                )));
            }
        }

        mapping.insert(base_name, obis_code);
        Ok(())
    }

    /// Unregister a Short Name mapping
    ///
    /// # Arguments
    /// * `base_name` - 16-bit base name to remove
    ///
    /// # Returns
    /// `true` if the mapping was removed, `false` if it didn't exist
    pub async fn unregister_short_name(&self, base_name: u16) -> bool {
        let mut mapping = self.base_name_to_obis.write().await;
        mapping.remove(&base_name).is_some()
    }

    /// Resolve a Short Name (base_name) to OBIS code
    ///
    /// # Arguments
    /// * `base_name` - 16-bit base name to resolve
    ///
    /// # Returns
    /// The corresponding OBIS code if registered, `None` otherwise
    pub async fn resolve_short_name(&self, base_name: u16) -> Option<ObisCode> {
        let mapping = self.base_name_to_obis.read().await;
        mapping.get(&base_name).copied()
    }

    /// Find an object by Short Name (base_name)
    ///
    /// This method combines short name resolution with object lookup.
    /// It's useful when handling SN-addressed requests.
    ///
    /// # Arguments
    /// * `base_name` - 16-bit base name
    ///
    /// # Returns
    /// Reference to the object if both short name is registered and object exists, `None` otherwise
    pub async fn find_object_by_base_name(&self, base_name: u16) -> Option<Arc<dyn CosemObject>> {
        if let Some(obis_code) = self.resolve_short_name(base_name).await {
            self.find_object(&obis_code).await
        } else {
            None
        }
    }

    /// Get all registered Short Name mappings
    ///
    /// # Returns
    /// Vector of (base_name, obis_code) tuples
    pub async fn get_short_name_mappings(&self) -> Vec<(u16, ObisCode)> {
        let mapping = self.base_name_to_obis.read().await;
        mapping.iter().map(|(k, v)| (*k, *v)).collect()
    }
    
    /// Register an association (client connection)
    ///
    /// # Arguments
    /// * `client_sap` - Client Service Access Point address
    /// * `context` - Association context
    pub async fn register_association(&self, client_sap: u16, context: AssociationContext) {
        let mut associations = self.associations.write().await;
        associations.insert(client_sap, context.clone());

        // Also register with connection manager
        let _ = self
            .connection_manager
            .register_connection(client_sap, None, context)
            .await;
    }

    /// Release an association
    ///
    /// # Arguments
    /// * `client_sap` - Client Service Access Point address
    pub async fn release_association(&self, client_sap: u16) {
        let mut associations = self.associations.write().await;
        associations.remove(&client_sap);

        // Also unregister from connection manager
        self.connection_manager.unregister_connection(client_sap).await;
    }
    
    /// Get association context for a client
    ///
    /// # Arguments
    /// * `client_sap` - Client Service Access Point address
    ///
    /// # Returns
    /// Association context if found, `None` otherwise
    pub async fn get_association(&self, client_sap: u16) -> Option<AssociationContext> {
        let associations = self.associations.read().await;
        associations.get(&client_sap).cloned()
    }
    
    /// Handle Initiate Request
    ///
    /// Processes an InitiateRequest from a client and returns an InitiateResponse.
    ///
    /// # Arguments
    /// * `request` - The InitiateRequest PDU
    /// * `client_sap` - Client Service Access Point address
    ///
    /// # Returns
    /// InitiateResponse PDU
    pub async fn handle_initiate_request(
        &self,
        request: &InitiateRequest,
        client_sap: u16,
    ) -> DlmsResult<InitiateResponse> {
        // Create association context
        let context = AssociationContext {
            client_sap,
            server_sap: self.config.server_sap,
            security_options: self.config.default_security.clone(),
            conformance: self.config.default_conformance.clone(),
            max_pdu_size: request.max_pdu_size().min(self.config.max_pdu_size),
            dlms_version: self.config.dlms_version,
        };
        
        // Register association
        self.register_association(client_sap, context.clone()).await;

        // Create response
        let response = InitiateResponse::new(
            self.config.dlms_version,
            context.conformance.clone(),
            context.max_pdu_size,
            0x0007, // vaa_name: standard VAA name for DLMS
        )?;

        Ok(response)
    }
    
    /// Handle GET Request
    ///
    /// Processes a GET request and returns the appropriate response.
    ///
    /// # Arguments
    /// * `request` - The GetRequest PDU
    /// * `client_sap` - Client Service Access Point address
    ///
    /// # Returns
    /// GetResponse PDU
    pub async fn handle_get_request(
        &self,
        request: &GetRequest,
        client_sap: u16,
    ) -> DlmsResult<GetResponse> {
        // Verify association exists
        let _association = self.get_association(client_sap).await.ok_or_else(|| {
            DlmsError::InvalidData("No active association for this client".to_string())
        })?;

        match request {
            GetRequest::Normal(normal) => {
                let descriptor = normal.cosem_attribute_descriptor();
                let selective_access = normal.selective_access();

                // Find object
                let obis = match descriptor {
                    CosemAttributeDescriptor::LogicalName(ln_ref) => ln_ref.instance_id,
                    CosemAttributeDescriptor::ShortName { reference, .. } => {
                        // SN addressing - resolve base_name to OBIS code
                        let base_name = reference.base_name;
                        self.resolve_short_name(base_name).await.ok_or_else(|| {
                            DlmsError::InvalidData(format!(
                                "Short name 0x{:04X} is not registered to any OBIS code",
                                base_name
                            ))
                        })?
                    }
                };

                let object = self.find_object(&obis).await.ok_or_else(|| {
                    DlmsError::InvalidData(format!("Object not found: {}", obis))
                })?;

                // Get attribute
                let attribute_id = match descriptor {
                    CosemAttributeDescriptor::LogicalName(ln_ref) => ln_ref.id,
                    CosemAttributeDescriptor::ShortName { reference, .. } => reference.id,
                };

                let value = object
                    .get_attribute(attribute_id, selective_access.as_deref(), None)
                    .await?;

                // Create response
                let invoke_id = normal.invoke_id_and_priority().invoke_id();
                let result = GetDataResult::new_data(value);
                let invoke_id_and_priority = InvokeIdAndPriority::new(invoke_id, false)?;
                let response = GetResponse::new_normal(invoke_id_and_priority, result);

                Ok(response)
            }
            GetRequest::Next { invoke_id_and_priority, block_number } => {
                // Get Request Next - for block transfer
                self.handle_get_request_next(client_sap, invoke_id_and_priority, *block_number).await
            }
            GetRequest::WithList {
                invoke_id_and_priority,
                attribute_descriptor_list,
                access_selection_list,
            } => {
                // Get Request With List - for multiple attributes
                self.handle_get_request_with_list(
                    client_sap,
                    invoke_id_and_priority,
                    attribute_descriptor_list,
                    access_selection_list,
                ).await
            }
        }
    }

    /// Handle GetRequest-WithList
    ///
    /// Processes a GET request for multiple attributes and returns a GetResponse-WithList.
    ///
    /// # Arguments
    /// * `client_sap` - Client Service Access Point address
    /// * `invoke_id_and_priority` - Invoke ID and priority
    /// * `attribute_descriptor_list` - List of attribute descriptors to read
    /// * `access_selection_list` - Optional list of selective access descriptors
    ///
    /// # Returns
    /// GetResponse PDU (WithList variant)
    async fn handle_get_request_with_list(
        &self,
        _client_sap: u16,
        invoke_id_and_priority: &InvokeIdAndPriority,
        attribute_descriptor_list: &[CosemAttributeDescriptor],
        access_selection_list: &Option<Vec<Option<dlms_application::pdu::SelectiveAccessDescriptor>>>,
    ) -> DlmsResult<GetResponse> {
        let mut result_list = Vec::new();

        for (i, descriptor) in attribute_descriptor_list.iter().enumerate() {
            let selective_access = access_selection_list
                .as_deref()
                .and_then(|list| list.get(i))
                .and_then(|s| s.as_ref());

            // Find object
            let obis = match descriptor {
                CosemAttributeDescriptor::LogicalName(ln_ref) => ln_ref.instance_id,
                CosemAttributeDescriptor::ShortName { reference, .. } => {
                    // SN addressing - resolve base_name to OBIS code
                    let base_name = reference.base_name;
                    match self.resolve_short_name(base_name).await {
                        Some(obis_code) => obis_code,
                        None => {
                            // Return error for this attribute
                            result_list.push(GetDataResult::new_error(
                                dlms_application::pdu::data_access_result::OBJECT_UNDEFINED
                            ));
                            continue;
                        }
                    }
                }
            };

            let object = match self.find_object(&obis).await {
                Some(obj) => obj,
                None => {
                    result_list.push(GetDataResult::new_error(
                        dlms_application::pdu::data_access_result::OBJECT_UNDEFINED
                    ));
                    continue;
                }
            };

            // Get attribute ID
            let attribute_id = match descriptor {
                CosemAttributeDescriptor::LogicalName(ln_ref) => ln_ref.id,
                CosemAttributeDescriptor::ShortName { reference, .. } => reference.id,
            };

            // Get attribute value
            match object
                .get_attribute(attribute_id, selective_access, None)
                .await
            {
                Ok(value) => {
                    result_list.push(GetDataResult::new_data(value));
                }
                Err(_e) => {
                    result_list.push(GetDataResult::new_error(
                        dlms_application::pdu::data_access_result::OTHER_REASON
                    ));
                }
            }
        }

        // Create WithList response
        let invoke_id = invoke_id_and_priority.invoke_id();
        let invoke_id_and_priority = InvokeIdAndPriority::new(invoke_id, false)?;

        // Check if result list is empty (shouldn't happen per spec)
        if result_list.is_empty() {
            return Err(DlmsError::InvalidData(
                "GetResponse-WithList: result_list cannot be empty".to_string()
            ));
        }

        Ok(GetResponse::WithList {
            invoke_id_and_priority,
            result_list,
        })
    }

    /// Handle GetRequest-Next
    ///
    /// Processes a GET request for the next block of data in a block transfer.
    ///
    /// # Arguments
    /// * `client_sap` - Client Service Access Point address
    /// * `invoke_id_and_priority` - Invoke ID and priority
    /// * `block_number` - Requested block number
    ///
    /// # Returns
    /// GetResponse PDU (WithDataBlock variant)
    async fn handle_get_request_next(
        &self,
        client_sap: u16,
        invoke_id_and_priority: &InvokeIdAndPriority,
        block_number: u32,
    ) -> DlmsResult<GetResponse> {
        let invoke_id = invoke_id_and_priority.invoke_id();
        let key = (client_sap, invoke_id);

        // Find the block transfer state
        let state = {
            let transfers = self.block_transfers.read().await;
            transfers.get(&key).cloned()
        };

        let state = match state {
            Some(s) => s,
            None => {
                return Err(DlmsError::InvalidData(format!(
                    "No block transfer in progress for invoke_id {}",
                    invoke_id
                )));
            }
        };

        // Check if the requested block number matches current state
        if block_number != state.current_block + 1 {
            return Err(DlmsError::InvalidData(format!(
                "Invalid block number: requested {}, expected {}",
                block_number,
                state.current_block + 1
            )));
        }

        // Advance to the requested block
        {
            let mut transfers = self.block_transfers.write().await;
            if let Some(s) = transfers.get_mut(&key) {
                if !s.advance() {
                    // Already at last block, remove state
                    transfers.remove(&key);
                }
            }
        }

        // Get the updated state
        let state = {
            let transfers = self.block_transfers.read().await;
            transfers.get(&key).cloned()
        };

        if let Some(s) = state {
            // Return the next block
            let block_data = s.get_current_block();
            let last_block = s.last_block;

            Ok(GetResponse::WithDataBlock {
                invoke_id_and_priority: InvokeIdAndPriority::new(invoke_id, false)?,
                block_number: s.current_block,
                last_block,
                block_data,
            })
        } else {
            // Should not happen - this means we were already at the last block
            return Err(DlmsError::InvalidData(
                "Block transfer already completed".to_string()
            ));
        }
    }

    /// Start a block transfer for a large attribute value
    ///
    /// # Arguments
    /// * `client_sap` - Client Service Access Point address
    /// * `invoke_id` - Invoke ID from the request
    /// * `obis_code` - OBIS code of the object
    /// * `attribute_id` - Attribute ID
    /// * `data` - Raw data to transfer (as bytes)
    ///
    /// # Returns
    /// GetResponse for the first block
    pub async fn start_block_transfer(
        &self,
        client_sap: u16,
        invoke_id: u8,
        obis_code: ObisCode,
        attribute_id: u8,
        data: Vec<u8>,
    ) -> DlmsResult<GetResponse> {
        // Get association to determine max PDU size
        let association = self.get_association(client_sap).await.ok_or_else(|| {
            DlmsError::InvalidData("No active association for this client".to_string())
        })?;

        // Calculate block size (leave room for overhead)
        // PDU structure: choice_tag(1) + invoke_id(4) + block_number(4) + last_block(1) + data
        let overhead = 10; // Approximate overhead
        let block_size = (association.max_pdu_size as usize).saturating_sub(overhead);

        if block_size == 0 {
            return Err(DlmsError::InvalidData(
                "Max PDU size too small for block transfer".to_string()
            ));
        }

        // Create block transfer state
        let transfer_state = BlockTransferState::new(
            invoke_id,
            obis_code,
            attribute_id,
            data,
            block_size,
        );

        // Get first block
        let block_data = transfer_state.get_current_block();
        let last_block = transfer_state.last_block;

        // Store the transfer state
        let key = (client_sap, invoke_id);
        let mut transfers = self.block_transfers.write().await;
        transfers.insert(key, transfer_state);

        // Return first block response
        Ok(GetResponse::WithDataBlock {
            invoke_id_and_priority: InvokeIdAndPriority::new(invoke_id, false)?,
            block_number: 0,
            last_block,
            block_data,
        })
    }
    
    /// Handle SET Request
    ///
    /// Processes a SET request and returns the appropriate response.
    ///
    /// # Arguments
    /// * `request` - The SetRequest PDU
    /// * `client_sap` - Client Service Access Point address
    ///
    /// # Returns
    /// SetResponse PDU
    pub async fn handle_set_request(
        &self,
        request: &SetRequest,
        client_sap: u16,
    ) -> DlmsResult<SetResponse> {
        // Verify association exists
        let _association = self.get_association(client_sap).await.ok_or_else(|| {
            DlmsError::InvalidData("No active association for this client".to_string())
        })?;

        match request {
            SetRequest::Normal(normal) => {
                let descriptor = normal.cosem_attribute_descriptor();
                let selective_access = normal.selective_access();
                let value = normal.value();

                // Find object
                let obis = match descriptor {
                    CosemAttributeDescriptor::LogicalName(ln_ref) => ln_ref.instance_id,
                    CosemAttributeDescriptor::ShortName { reference, .. } => {
                        // SN addressing - resolve base_name to OBIS code
                        let base_name = reference.base_name;
                        self.resolve_short_name(base_name).await.ok_or_else(|| {
                            DlmsError::InvalidData(format!(
                                "Short name 0x{:04X} is not registered to any OBIS code",
                                base_name
                            ))
                        })?
                    }
                };

                let object = self.find_object(&obis).await.ok_or_else(|| {
                    DlmsError::InvalidData(format!("Object not found: {}", obis))
                })?;

                // Set attribute
                let attribute_id = match descriptor {
                    CosemAttributeDescriptor::LogicalName(ln_ref) => ln_ref.id,
                    CosemAttributeDescriptor::ShortName { reference, .. } => reference.id,
                };

                object
                    .set_attribute(
                        attribute_id,
                        value.clone(),
                        selective_access.as_deref(),
                        None,
                    )
                    .await?;

                // Create response
                let invoke_id = normal.invoke_id_and_priority().invoke_id();
                let result = SetDataResult::new_success();
                let invoke_id_and_priority = InvokeIdAndPriority::new(invoke_id, false)?;
                let response = SetResponse::new_normal(invoke_id_and_priority, result);

                Ok(response)
            }
            SetRequest::WithFirstDataBlock {
                invoke_id_and_priority,
                cosem_attribute_descriptor,
                access_selection: _,
                block_number,
                last_block,
                block_data,
            } => {
                // Handle first data block - initiate block transfer
                self.handle_set_request_first_data_block(
                    invoke_id_and_priority,
                    cosem_attribute_descriptor,
                    *block_number,
                    *last_block,
                    block_data,
                ).await
            }
            SetRequest::WithDataBlock {
                invoke_id_and_priority,
                block_number,
                last_block,
                block_data,
            } => {
                // Handle subsequent data blocks
                self.handle_set_request_data_block(
                    invoke_id_and_priority,
                    *block_number,
                    *last_block,
                    block_data,
                ).await
            }
            SetRequest::WithList(with_list) => {
                self.handle_set_request_with_list(client_sap, with_list).await
            }
        }
    }

    /// Handle SetRequest-WithFirstDataBlock for initiating block transfer
    ///
    /// # Arguments
    /// * `invoke_id_and_priority` - Invoke ID and priority from request
    /// * `cosem_attribute_descriptor` - Attribute to write
    /// * `block_number` - Block number (should be 0 for first block)
    /// * `last_block` - Last block flag
    /// * `block_data` - Block data
    ///
    /// # Returns
    /// SetResponse::WithDataBlock acknowledging receipt, or SetResponse::Normal if complete
    async fn handle_set_request_first_data_block(
        &self,
        invoke_id_and_priority: &InvokeIdAndPriority,
        cosem_attribute_descriptor: &CosemAttributeDescriptor,
        block_number: u32,
        last_block: bool,
        block_data: &[u8],
    ) -> DlmsResult<SetResponse> {
        use crate::set_block_transfer::SetBlockTransferState;

        // Find object
        let obis = match cosem_attribute_descriptor {
            CosemAttributeDescriptor::LogicalName(ln_ref) => ln_ref.instance_id,
            CosemAttributeDescriptor::ShortName { reference, .. } => {
                let base_name = reference.base_name;
                self.resolve_short_name(base_name).await.ok_or_else(|| {
                    DlmsError::InvalidData(format!(
                        "Short name 0x{:04X} is not registered to any OBIS code",
                        base_name
                    ))
                })?
            }
        };

        let attribute_id = match cosem_attribute_descriptor {
            CosemAttributeDescriptor::LogicalName(ln_ref) => ln_ref.id,
            CosemAttributeDescriptor::ShortName { reference, .. } => reference.id,
        };

        // Create new block transfer state
        let invoke_id = invoke_id_and_priority.invoke_id();
        let state = SetBlockTransferState::new(invoke_id, obis, attribute_id, block_data.to_vec(), 512);

        // Store state for subsequent blocks
        // TODO: Add proper block transfer state management
        let _state = state; // Suppress unused warning for now

        // If this is the only block (last_block=true and block_number=0), complete the transfer
        if last_block && block_number == 0 {
            // Single block transfer - decode and set the value
            // For now, return success
            let result = SetDataResult::new_success();
            let response_invoke = InvokeIdAndPriority::new(invoke_id, false)?;
            return Ok(SetResponse::new_normal(response_invoke, result));
        }

        // Acknowledge and request next block
        let response_invoke = InvokeIdAndPriority::new(invoke_id, false)?;
        Ok(SetResponse::new_with_data_block(response_invoke, block_number, last_block))
    }

    /// Handle SetRequest-WithDataBlock for continuing block transfer
    ///
    /// # Arguments
    /// * `invoke_id_and_priority` - Invoke ID and priority from request
    /// * `block_number` - Block number
    /// * `last_block` - Last block flag
    /// * `block_data` - Block data
    ///
    /// # Returns
    /// SetResponse::WithDataBlock acknowledging receipt, or SetResponse::Normal if complete
    async fn handle_set_request_data_block(
        &self,
        invoke_id_and_priority: &InvokeIdAndPriority,
        block_number: u32,
        last_block: bool,
        block_data: &[u8],
    ) -> DlmsResult<SetResponse> {
        let invoke_id = invoke_id_and_priority.invoke_id();

        // TODO: Retrieve block transfer state and append data
        let _block_data = block_data; // Suppress unused warning for now

        if last_block {
            // All blocks received - decode and set the value
            // For now, return success
            let result = SetDataResult::new_success();
            let response_invoke = InvokeIdAndPriority::new(invoke_id, false)?;
            Ok(SetResponse::new_normal(response_invoke, result))
        } else {
            // Acknowledge and request next block
            let response_invoke = InvokeIdAndPriority::new(invoke_id, false)?;
            Ok(SetResponse::new_with_data_block(response_invoke, block_number, last_block))
        }
    }

    /// Handle SetRequest-WithList for multiple attributes
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    /// * `with_list` - The SetRequestWithList PDU
    ///
    /// # Returns
    /// SetResponse::WithList with results for all SET operations
    async fn handle_set_request_with_list(
        &self,
        _client_sap: u16,
        with_list: &SetRequestWithList,
    ) -> DlmsResult<SetResponse> {
        use dlms_application::pdu::data_access_result;

        let mut result_list = Vec::new();

        // Process each attribute in the list
        for (i, descriptor) in with_list.attribute_descriptor_list.iter().enumerate() {
            let selective_access = with_list.access_selection_list
                .get(i)
                .and_then(|s| s.as_ref());
            let value = with_list.value_list.get(i).unwrap();

            // Find object
            let obis = match descriptor {
                CosemAttributeDescriptor::LogicalName(ln_ref) => ln_ref.instance_id,
                CosemAttributeDescriptor::ShortName { reference, .. } => {
                    // SN addressing - resolve base_name to OBIS code
                    let base_name = reference.base_name;
                    match self.resolve_short_name(base_name).await {
                        Some(obis_code) => obis_code,
                        None => {
                            // Return error for this attribute
                            result_list.push(SetDataResult::new_error(
                                data_access_result::OBJECT_UNDEFINED
                            ));
                            continue;
                        }
                    }
                }
            };

            let object = match self.find_object(&obis).await {
                Some(obj) => obj,
                None => {
                    result_list.push(SetDataResult::new_error(
                        data_access_result::OBJECT_UNDEFINED
                    ));
                    continue;
                }
            };

            // Get attribute ID
            let attribute_id = match descriptor {
                CosemAttributeDescriptor::LogicalName(ln_ref) => ln_ref.id,
                CosemAttributeDescriptor::ShortName { reference, .. } => reference.id,
            };

            // Set attribute
            match object
                .set_attribute(
                    attribute_id,
                    value.clone(),
                    selective_access.as_deref(),
                    None,
                )
                .await
            {
                Ok(_) => {
                    result_list.push(SetDataResult::new_success());
                }
                Err(_e) => {
                    result_list.push(SetDataResult::new_error(
                        data_access_result::OTHER_REASON
                    ));
                }
            }
        }

        // Create response
        let invoke_id = with_list.invoke_id_and_priority.invoke_id();
        let invoke_id_and_priority = InvokeIdAndPriority::new(invoke_id, false)?;
        SetResponse::new_with_list(invoke_id_and_priority, result_list)
    }
    
    /// Handle ACTION Request
    ///
    /// Processes an ACTION request and returns the appropriate response.
    ///
    /// # Arguments
    /// * `request` - The ActionRequest PDU
    /// * `client_sap` - Client Service Access Point address
    ///
    /// # Returns
    /// ActionResponse PDU
    pub async fn handle_action_request(
        &self,
        request: &ActionRequest,
        client_sap: u16,
    ) -> DlmsResult<ActionResponse> {
        // Verify association exists
        let _association = self.get_association(client_sap).await.ok_or_else(|| {
            DlmsError::InvalidData("No active association for this client".to_string())
        })?;
        
        match request {
            ActionRequest::Normal(normal) => {
                let descriptor = normal.cosem_method_descriptor();
                let parameters = normal.method_invocation_parameters();
                
                // Find object
                let obis = match descriptor {
                    CosemMethodDescriptor::LogicalName(ln_ref) => ln_ref.instance_id,
                    CosemMethodDescriptor::ShortName { reference, .. } => {
                        // SN addressing - resolve base_name to OBIS code
                        let base_name = reference.base_name;
                        self.resolve_short_name(base_name).await.ok_or_else(|| {
                            DlmsError::InvalidData(format!(
                                "Short name 0x{:04X} is not registered to any OBIS code",
                                base_name
                            ))
                        })?
                    }
                };
                
                let object = self.find_object(&obis).await.ok_or_else(|| {
                    DlmsError::InvalidData(format!("Object not found: {}", obis))
                })?;
                
                // Invoke method
                let method_id = match descriptor {
                    CosemMethodDescriptor::LogicalName(ln_ref) => ln_ref.id,
                    CosemMethodDescriptor::ShortName { reference, .. } => reference.id,
                };

                let return_value = object
                    .invoke_method(method_id, parameters.cloned(), None, None)
                    .await?;

                // Create response
                let invoke_id = normal.invoke_id_and_priority().invoke_id();
                let result = if let Some(value) = return_value {
                    ActionResult::new_return_parameters(value)
                } else {
                    ActionResult::new_success()
                };
                let response = ActionResponse::new_normal(
                    InvokeIdAndPriority::new(invoke_id, false)?,
                    result,
                );

                Ok(response)
            }
        }
    }
    
    /// Handle Access Request
    ///
    /// Processes an Access request (which can contain multiple GET/SET/ACTION operations)
    /// and returns the appropriate response.
    ///
    /// # Arguments
    /// * `request` - The AccessRequest PDU
    /// * `client_sap` - Client Service Access Point address
    ///
    /// # Returns
    /// AccessResponse PDU
    ///
    /// # Design
    /// AccessRequest allows multiple operations in a single request. Each operation
    /// in the list is processed sequentially, and results are collected into the response.
    pub async fn handle_access_request(
        &self,
        request: &AccessRequest,
        client_sap: u16,
    ) -> DlmsResult<AccessResponse> {
        // Verify association exists
        let _association = self.get_association(client_sap).await.ok_or_else(|| {
            DlmsError::InvalidData("No active association for this client".to_string())
        })?;
        
        let invoke_id = request.invoke_id_and_priority.invoke_id();
        let mut access_response_list = Vec::new();
        
        // Process each access request specification
        for spec in &request.access_request_list {
            let result = match spec {
                AccessRequestSpecification::Get { cosem_attribute_descriptor, access_selection } => {
                    // Find object
                    let obis = match cosem_attribute_descriptor {
                        CosemAttributeDescriptor::LogicalName(ln_ref) => ln_ref.instance_id,
                        CosemAttributeDescriptor::ShortName { reference, .. } => {
                            // SN addressing - resolve base_name to OBIS code
                            let base_name = reference.base_name;
                            match self.resolve_short_name(base_name).await {
                                Some(obis_code) => obis_code,
                                None => {
                                    // Return error result for this item
                                    access_response_list.push(AccessResponseSpecification::Get(
                                        GetDataResult::new_standard_error(
                                            dlms_application::pdu::data_access_result::OBJECT_UNDEFINED,
                                        ),
                                    ));
                                    continue;
                                }
                            }
                        }
                    };

                    // Get attribute
                    match self.find_object(&obis).await {
                        Some(object) => {
                            let attribute_id = match cosem_attribute_descriptor {
                                CosemAttributeDescriptor::LogicalName(ln_ref) => ln_ref.id,
                                CosemAttributeDescriptor::ShortName { reference, .. } => reference.id,
                            };

                            match object
                                .get_attribute(attribute_id, access_selection.as_ref(), None)
                                .await
                            {
                                Ok(value) => {
                                    AccessResponseSpecification::Get(
                                        GetDataResult::new_data(value),
                                    )
                                }
                                Err(_) => {
                                    // Convert error to data access result
                                    // For now, use hardware fault as generic error
                                    AccessResponseSpecification::Get(
                                        GetDataResult::new_standard_error(
                                            dlms_application::pdu::data_access_result::HARDWARE_FAULT,
                                        ),
                                    )
                                }
                            }
                        }
                        None => {
                            AccessResponseSpecification::Get(
                                GetDataResult::new_standard_error(
                                    dlms_application::pdu::data_access_result::OBJECT_UNAVAILABLE,
                                ),
                            )
                        }
                    }
                }
                AccessRequestSpecification::Set { cosem_attribute_descriptor, access_selection, value } => {
                    // Find object
                    let obis = match cosem_attribute_descriptor {
                        CosemAttributeDescriptor::LogicalName(ln_ref) => ln_ref.instance_id,
                        CosemAttributeDescriptor::ShortName { reference, .. } => {
                            // SN addressing - resolve base_name to OBIS code
                            let base_name = reference.base_name;
                            match self.resolve_short_name(base_name).await {
                                Some(obis_code) => obis_code,
                                None => {
                                    // Return error result for this item
                                    access_response_list.push(AccessResponseSpecification::Set(
                                        SetDataResult::new_standard_error(
                                            dlms_application::pdu::data_access_result::OBJECT_UNDEFINED,
                                        ),
                                    ));
                                    continue;
                                }
                            }
                        }
                    };

                    // Set attribute
                    match self.find_object(&obis).await {
                        Some(object) => {
                            let attribute_id = match cosem_attribute_descriptor {
                                CosemAttributeDescriptor::LogicalName(ln_ref) => ln_ref.id,
                                CosemAttributeDescriptor::ShortName { reference, .. } => reference.id,
                            };

                            match object
                                .set_attribute(
                                    attribute_id,
                                    value.clone(),
                                    access_selection.as_ref(),
                                    None,
                                )
                                .await
                            {
                                Ok(_) => {
                                    AccessResponseSpecification::Set(
                                        SetDataResult::new_success(),
                                    )
                                }
                                Err(_) => {
                                    // Convert error to data access result
                                    AccessResponseSpecification::Set(
                                        SetDataResult::new_standard_error(
                                            dlms_application::pdu::data_access_result::HARDWARE_FAULT,
                                        ),
                                    )
                                }
                            }
                        }
                        None => {
                            AccessResponseSpecification::Set(
                                SetDataResult::new_standard_error(
                                    dlms_application::pdu::data_access_result::OBJECT_UNAVAILABLE,
                                ),
                            )
                        }
                    }
                }
                AccessRequestSpecification::Action { cosem_method_descriptor, method_invocation_parameters } => {
                    // Find object
                    let obis = match cosem_method_descriptor {
                        CosemMethodDescriptor::LogicalName(ln_ref) => ln_ref.instance_id,
                        CosemMethodDescriptor::ShortName { reference, .. } => {
                            // SN addressing - resolve base_name to OBIS code
                            let base_name = reference.base_name;
                            match self.resolve_short_name(base_name).await {
                                Some(obis_code) => obis_code,
                                None => {
                                    // Return error result for this item
                                    access_response_list.push(AccessResponseSpecification::Action(
                                        ActionResult::new_data_access_result(
                                            dlms_application::pdu::action_result::OBJECT_UNDEFINED,
                                        ),
                                    ));
                                    continue;
                                }
                            }
                        }
                    };

                    // Invoke method
                    match self.find_object(&obis).await {
                        Some(object) => {
                            let method_id = match cosem_method_descriptor {
                                CosemMethodDescriptor::LogicalName(ln_ref) => ln_ref.id,
                                CosemMethodDescriptor::ShortName { reference, .. } => reference.id,
                            };

                            match object
                                .invoke_method(
                                    method_id,
                                    method_invocation_parameters.clone(),
                                    None,
                                    None,
                                )
                                .await
                            {
                                Ok(return_value) => {
                                    if let Some(value) = return_value {
                                        AccessResponseSpecification::Action(
                                            ActionResult::new_return_parameters(value),
                                        )
                                    } else {
                                        AccessResponseSpecification::Action(
                                            ActionResult::new_success(),
                                        )
                                    }
                                }
                                Err(_) => {
                                    // Convert error to action result
                                    AccessResponseSpecification::Action(
                                        ActionResult::new_data_access_result(
                                            dlms_application::pdu::action_result::HARDWARE_FAULT,
                                        ),
                                    )
                                }
                            }
                        }
                        None => {
                            AccessResponseSpecification::Action(
                                ActionResult::new_data_access_result(
                                    dlms_application::pdu::action_result::OBJECT_UNAVAILABLE,
                                ),
                            )
                        }
                    }
                }
            };
            
            access_response_list.push(result);
        }
        
        // Create response
        let response = AccessResponse::new(
            InvokeIdAndPriority::new(invoke_id, false)?,
            access_response_list,
        )?;
        
        Ok(response)
    }
    
    /// Get server configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }
    
    /// Get number of registered objects
    pub async fn object_count(&self) -> usize {
        let objects = self.objects.read().await;
        objects.len()
    }
    
    /// Get number of active associations
    pub async fn association_count(&self) -> usize {
        let associations = self.associations.read().await;
        associations.len()
    }
}

impl Default for DlmsServer {
    fn default() -> Self {
        Self::new()
    }
}
