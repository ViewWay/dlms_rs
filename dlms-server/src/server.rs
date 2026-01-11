//! DLMS/COSEM server implementation
//!
//! This module provides server-side functionality for DLMS/COSEM protocol,
//! including object management, request handling, and association management.

use dlms_application::pdu::{
    GetRequest, GetResponse, SetRequest, SetResponse, ActionRequest, ActionResponse,
    InitiateRequest, InitiateResponse, AccessRequest, AccessResponse,
    CosemAttributeDescriptor, CosemMethodDescriptor, GetDataResult, SetDataResult, ActionResult,
    InvokeIdAndPriority, Conformance,
};
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use dlms_security::SecuritySuite;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// COSEM object interface
///
/// All COSEM objects must implement this trait to be registered with the server.
/// This provides a unified interface for accessing object attributes and methods.
///
/// # Design Philosophy
/// Using a trait allows:
/// - **Polymorphism**: Same code works with different object types
/// - **Extensibility**: Easy to add new object types
/// - **Testability**: Easy to mock objects for testing
#[async_trait::async_trait]
pub trait CosemObject: Send + Sync {
    /// Get the class ID of this object
    fn class_id(&self) -> u16;
    
    /// Get the OBIS code (logical name) of this object
    fn obis_code(&self) -> ObisCode;
    
    /// Get an attribute value
    ///
    /// # Arguments
    /// * `attribute_id` - Attribute ID to read (1-255)
    /// * `selective_access` - Optional selective access descriptor
    ///
    /// # Returns
    /// The attribute value as a `DataObject`, or error if attribute doesn't exist
    async fn get_attribute(
        &self,
        attribute_id: u8,
        selective_access: Option<&dlms_application::pdu::SelectiveAccessDescriptor>,
    ) -> DlmsResult<DataObject>;
    
    /// Set an attribute value
    ///
    /// # Arguments
    /// * `attribute_id` - Attribute ID to write (1-255)
    /// * `value` - Value to write
    /// * `selective_access` - Optional selective access descriptor
    ///
    /// # Returns
    /// `Ok(())` if successful, error otherwise
    async fn set_attribute(
        &self,
        attribute_id: u8,
        value: DataObject,
        selective_access: Option<&dlms_application::pdu::SelectiveAccessDescriptor>,
    ) -> DlmsResult<()>;
    
    /// Invoke a method
    ///
    /// # Arguments
    /// * `method_id` - Method ID to invoke (1-255)
    /// * `parameters` - Optional method parameters
    ///
    /// # Returns
    /// Optional return value from the method (if any)
    async fn invoke_method(
        &self,
        method_id: u8,
        parameters: Option<DataObject>,
    ) -> DlmsResult<Option<DataObject>>;
}

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

/// DLMS/COSEM server
///
/// Main server implementation that manages:
/// - COSEM object registry
/// - Association management
/// - Request handling (GET, SET, ACTION)
/// - Response generation
///
/// # Architecture
/// The server follows a similar architecture to the C++ reference implementation:
/// - `LogicalDevice`: Manages objects and associations
/// - `Association`: Tracks active client connections
/// - `CosemObject`: Interface for all COSEM objects
///
/// # Usage Example
/// ```rust,no_run
/// use dlms_server::server::{DlmsServer, CosemObject};
/// use dlms_core::ObisCode;
///
/// // Create server
/// let mut server = DlmsServer::new();
///
/// // Register objects
/// server.register_object(my_object).await?;
///
/// // Start listening
/// server.start().await?;
/// ```
pub struct DlmsServer {
    /// Registered COSEM objects (indexed by OBIS code)
    objects: Arc<RwLock<HashMap<ObisCode, Arc<dyn CosemObject>>>>,
    /// Active associations (indexed by client SAP)
    associations: Arc<RwLock<HashMap<u16, AssociationContext>>>,
    /// Server configuration
    config: ServerConfig,
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
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server_sap: 1,
            default_security: SecuritySuite::default(),
            default_conformance: Conformance::default(),
            max_pdu_size: 1024,
            dlms_version: 6,
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
        Self {
            objects: Arc::new(RwLock::new(HashMap::new())),
            associations: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
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
    
    /// Register an association (client connection)
    ///
    /// # Arguments
    /// * `client_sap` - Client Service Access Point address
    /// * `context` - Association context
    pub async fn register_association(&self, client_sap: u16, context: AssociationContext) {
        let mut associations = self.associations.write().await;
        associations.insert(client_sap, context);
    }
    
    /// Release an association
    ///
    /// # Arguments
    /// * `client_sap` - Client Service Access Point address
    pub async fn release_association(&self, client_sap: u16) {
        let mut associations = self.associations.write().await;
        associations.remove(&client_sap);
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
            None, // server_max_received_pdu_size (optional)
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
                    CosemAttributeDescriptor::ShortName { .. } => {
                        // SN addressing - would need base_name to OBIS mapping
                        return Err(DlmsError::InvalidData(
                            "Short Name addressing not yet supported in server".to_string(),
                        ));
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
                
                let value = object.get_attribute(attribute_id, selective_access.as_ref()).await?;
                
                // Create response
                let invoke_id = normal.invoke_id_and_priority().invoke_id();
                let result = GetDataResult::new_data(value);
                let response = GetResponse::new_normal(
                    InvokeIdAndPriority::new(invoke_id, false)?,
                    result,
                )?;
                
                Ok(response)
            }
            GetRequest::Next(_) => {
                // Get Request Next - for block transfer
                // TODO: Implement block transfer support
                Err(DlmsError::InvalidData(
                    "Get Request Next not yet implemented".to_string(),
                ))
            }
            GetRequest::WithList(_) => {
                // Get Request With List - for multiple attributes
                // TODO: Implement WithList support
                Err(DlmsError::InvalidData(
                    "Get Request WithList not yet implemented".to_string(),
                ))
            }
        }
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
                    CosemAttributeDescriptor::ShortName { .. } => {
                        return Err(DlmsError::InvalidData(
                            "Short Name addressing not yet supported in server".to_string(),
                        ));
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
                
                object.set_attribute(attribute_id, value.clone(), selective_access.as_ref()).await?;
                
                // Create response
                let invoke_id = normal.invoke_id_and_priority().invoke_id();
                let result = SetDataResult::new_success();
                let response = SetResponse::new_normal(
                    InvokeIdAndPriority::new(invoke_id, false)?,
                    result,
                )?;
                
                Ok(response)
            }
        }
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
                    CosemMethodDescriptor::ShortName { .. } => {
                        return Err(DlmsError::InvalidData(
                            "Short Name addressing not yet supported in server".to_string(),
                        ));
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
                
                let return_value = object.invoke_method(method_id, parameters.clone()).await?;
                
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
                )?;
                
                Ok(response)
            }
        }
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
