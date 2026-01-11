//! SET Service implementation for DLMS/COSEM
//!
//! This module provides high-level SET service functionality for writing attribute values
//! to COSEM objects.
//!
//! # Design Philosophy
//!
//! The service layer abstracts away PDU encoding/decoding details, providing a clean
//! high-level API for SET operations. This separation of concerns allows:
//! - Easier testing and maintenance
//! - Better error handling and result processing
//! - Automatic invoke ID management
//!
//! # Optimization Considerations
//!
//! - Invoke ID management uses a simple counter with wraparound. For high-concurrency
//!   scenarios, consider using atomic operations or a more sophisticated ID pool.
//! - PDU encoding/decoding happens on-demand. For high-frequency operations, consider
//!   caching encoded PDUs or using a connection pool.

use crate::pdu::{
    SetRequest, SetRequestNormal, SetResponse, SetResponseNormal, SetDataResult,
    CosemAttributeDescriptor, SelectiveAccessDescriptor, InvokeIdAndPriority,
};
use dlms_core::{DlmsError, DlmsResult, DataObject};

/// SET Service for DLMS/COSEM
///
/// Provides high-level interface for SET operations, handling PDU creation,
/// encoding/decoding, and result processing.
///
/// # Why a Separate Service Layer?
/// Separating service logic from PDU encoding/decoding provides:
/// - **Cleaner API**: Users don't need to know about PDU internals
/// - **Better Error Handling**: Service layer can provide more meaningful error messages
/// - **Invoke ID Management**: Automatic tracking of request/response pairs
/// - **Future Extensibility**: Easy to add features like retry logic, timeouts, etc.
///
/// # Optimization Considerations
/// - The service is lightweight and can be cloned if needed for concurrent operations
/// - Invoke ID management is simple but effective for most use cases
/// - Future optimization: Add connection pooling and request queuing for high-throughput scenarios
#[derive(Debug, Clone)]
pub struct SetService {
    /// Next invoke ID to use (0-127)
    next_invoke_id: u8,
}

impl SetService {
    /// Create a new SET service
    pub fn new() -> Self {
        Self {
            next_invoke_id: 1,
        }
    }

    /// Get the next invoke ID and increment
    pub fn next_invoke_id(&mut self) -> u8 {
        let id = self.next_invoke_id;
        self.next_invoke_id = if self.next_invoke_id >= 127 {
            1
        } else {
            self.next_invoke_id + 1
        };
        id
    }

    /// Create a Normal SET request
    ///
    /// # Arguments
    /// * `invoke_id_and_priority` - Invoke ID and priority
    /// * `cosem_attribute_descriptor` - Attribute to write
    /// * `access_selection` - Optional selective access descriptor
    /// * `value` - Data value to write
    ///
    /// # Returns
    /// A `SetRequest::Normal` PDU ready to be encoded and sent
    pub fn create_normal_request(
        invoke_id_and_priority: InvokeIdAndPriority,
        cosem_attribute_descriptor: CosemAttributeDescriptor,
        access_selection: Option<SelectiveAccessDescriptor>,
        value: DataObject,
    ) -> DlmsResult<SetRequest> {
        Ok(SetRequest::new_normal(
            invoke_id_and_priority,
            cosem_attribute_descriptor,
            access_selection,
            value,
        ))
    }

    /// Process a SET response and check for success
    ///
    /// # Arguments
    /// * `response` - The SET response PDU
    ///
    /// # Returns
    /// Ok(()) if successful, Err if failed
    pub fn process_response(response: &SetResponse) -> DlmsResult<()> {
        match response {
            SetResponse::Normal(normal) => {
                match &normal.result {
                    SetDataResult::Success => Ok(()),
                    SetDataResult::DataAccessResult(code) => Err(DlmsError::InvalidData(format!(
                        "SET operation failed with error code: {}",
                        code
                    ))),
                }
            }
            _ => Err(DlmsError::InvalidData(
                "Expected Normal SET response".to_string(),
            )),
        }
    }
}

impl Default for SetService {
    fn default() -> Self {
        Self::new()
    }
}
