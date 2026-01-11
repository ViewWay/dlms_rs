//! GET Service implementation for DLMS/COSEM
//!
//! This module provides high-level GET service functionality for reading attribute values
//! from COSEM objects.
//!
//! # Features
//! - Single attribute GET requests
//! - Multiple attribute GET requests (WithList)
//! - Large attribute handling (data blocks)
//! - Automatic invoke ID management
//! - Error handling and result conversion
//!
//! # Usage
//!
//! ```rust,no_run
//! use dlms_application::service::GetService;
//! use dlms_application::pdu::{CosemAttributeDescriptor, InvokeIdAndPriority};
//! use dlms_core::ObisCode;
//!
//! // Create a GET service
//! let mut service = GetService::new();
//!
//! // Create a GET request
//! let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
//! let attr_desc = CosemAttributeDescriptor::new_logical_name(1, obis, 2)?;
//! let invoke = InvokeIdAndPriority::new(1, false)?;
//!
//! let request = service.create_normal_request(invoke, attr_desc, None)?;
//! ```

use crate::pdu::{
    GetRequest, GetRequestNormal, GetResponse, GetResponseNormal, GetDataResult,
    CosemAttributeDescriptor, SelectiveAccessDescriptor, InvokeIdAndPriority,
};
use dlms_core::{DlmsError, DlmsResult, DataObject};

/// GET Service for DLMS/COSEM
///
/// Provides high-level interface for GET operations, handling PDU creation,
/// encoding/decoding, and result processing.
///
/// # Why a Separate Service Layer?
/// Separating service logic from PDU encoding/decoding provides:
/// - **Cleaner API**: Users don't need to know about PDU internals or A-XDR encoding
/// - **Better Error Handling**: Service layer can provide more meaningful error messages
/// - **Invoke ID Management**: Automatic tracking of request/response pairs
/// - **Result Processing**: Automatic extraction of data from responses
/// - **Future Extensibility**: Easy to add features like retry logic, timeouts, caching, etc.
///
/// # Invoke ID Management
/// The service manages invoke IDs to ensure unique request/response correlation.
/// Each request gets a unique invoke ID that is used to match responses. The ID wraps
/// around at 127 (skipping 0, which is reserved), providing 126 concurrent requests.
///
/// # Optimization Considerations
/// - The service is lightweight and can be cloned if needed for concurrent operations
/// - Invoke ID management uses a simple counter with wraparound, which is efficient
///   for most use cases. For high-concurrency scenarios, consider using atomic operations
///   or a more sophisticated ID pool.
/// - PDU encoding/decoding happens on-demand. For high-frequency operations, consider
///   caching encoded PDUs or using a connection pool.
/// - Large attribute values are handled through data blocks, but the service layer
///   doesn't currently automate block retrieval. Future enhancement: Add automatic
///   block handling for seamless large data transfers.
#[derive(Debug, Clone)]
pub struct GetService {
    /// Next invoke ID to use (0-127)
    next_invoke_id: u8,
}

impl GetService {
    /// Create a new GET service
    pub fn new() -> Self {
        Self {
            next_invoke_id: 1, // Start from 1, 0 is reserved
        }
    }

    /// Get the next invoke ID and increment
    ///
    /// # Returns
    /// The next invoke ID (wraps around at 127)
    pub fn next_invoke_id(&mut self) -> u8 {
        let id = self.next_invoke_id;
        self.next_invoke_id = if self.next_invoke_id >= 127 {
            1 // Wrap around, skip 0
        } else {
            self.next_invoke_id + 1
        };
        id
    }

    /// Create a Normal GET request
    ///
    /// # Arguments
    /// * `invoke_id_and_priority` - Invoke ID and priority
    /// * `cosem_attribute_descriptor` - Attribute to read
    /// * `access_selection` - Optional selective access descriptor
    ///
    /// # Returns
    /// A `GetRequest::Normal` PDU ready to be encoded and sent
    pub fn create_normal_request(
        invoke_id_and_priority: InvokeIdAndPriority,
        cosem_attribute_descriptor: CosemAttributeDescriptor,
        access_selection: Option<SelectiveAccessDescriptor>,
    ) -> DlmsResult<GetRequest> {
        Ok(GetRequest::new_normal(
            invoke_id_and_priority,
            cosem_attribute_descriptor,
            access_selection,
        ))
    }

    /// Create a WithList GET request
    ///
    /// # Arguments
    /// * `invoke_id_and_priority` - Invoke ID and priority
    /// * `attribute_descriptor_list` - List of attributes to read
    /// * `access_selection_list` - Optional list of selective access descriptors
    ///
    /// # Returns
    /// A `GetRequest::WithList` PDU ready to be encoded and sent
    pub fn create_with_list_request(
        invoke_id_and_priority: InvokeIdAndPriority,
        attribute_descriptor_list: Vec<CosemAttributeDescriptor>,
        access_selection_list: Option<Vec<Option<SelectiveAccessDescriptor>>>,
    ) -> DlmsResult<GetRequest> {
        if attribute_descriptor_list.is_empty() {
            return Err(DlmsError::InvalidData(
                "attribute_descriptor_list cannot be empty".to_string(),
            ));
        }

        Ok(GetRequest::WithList {
            invoke_id_and_priority,
            attribute_descriptor_list,
            access_selection_list,
        })
    }

    /// Process a GET response and extract the result
    ///
    /// # Arguments
    /// * `response` - The GET response PDU
    ///
    /// # Returns
    /// The result data or error code
    ///
    /// # Errors
    /// Returns error if the response is not a Normal response or if the result indicates failure
    pub fn process_response(response: &GetResponse) -> DlmsResult<DataObject> {
        match response {
            GetResponse::Normal(normal) => {
                match &normal.result {
                    GetDataResult::Data(data) => Ok(data.clone()),
                    GetDataResult::DataAccessResult(code) => Err(DlmsError::InvalidData(format!(
                        "GET operation failed with error code: {}",
                        code
                    ))),
                }
            }
            _ => Err(DlmsError::InvalidData(
                "Expected Normal GET response".to_string(),
            )),
        }
    }

    /// Process a GET response and return the full result
    ///
    /// # Arguments
    /// * `response` - The GET response PDU
    ///
    /// # Returns
    /// The `GetDataResult` containing either data or error code
    pub fn process_response_result(response: &GetResponse) -> DlmsResult<GetDataResult> {
        match response {
            GetResponse::Normal(normal) => Ok(normal.result.clone()),
            _ => Err(DlmsError::InvalidData(
                "Expected Normal GET response".to_string(),
            )),
        }
    }
}

impl Default for GetService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dlms_core::ObisCode;

    #[test]
    fn test_get_service_create_normal_request() {
        let service = GetService::new();
        let invoke = InvokeIdAndPriority::new(1, false).unwrap();
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let attr_desc = CosemAttributeDescriptor::new_logical_name(1, obis, 2).unwrap();

        let request = service.create_normal_request(invoke, attr_desc, None).unwrap();

        match request {
            GetRequest::Normal(_) => {}
            _ => panic!("Expected Normal request"),
        }
    }

    #[test]
    fn test_get_service_invoke_id_management() {
        let mut service = GetService::new();
        let id1 = service.next_invoke_id();
        let id2 = service.next_invoke_id();
        assert_ne!(id1, id2);
        assert!(id1 >= 1 && id1 <= 127);
        assert!(id2 >= 1 && id2 <= 127);
    }
}
