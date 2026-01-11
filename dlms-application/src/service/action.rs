//! ACTION Service implementation for DLMS/COSEM
//!
//! This module provides high-level ACTION service functionality for invoking methods
//! on COSEM objects.
//!
//! # Design Philosophy
//!
//! Similar to GET and SET services, the ACTION service provides a high-level abstraction
//! over PDU handling. It manages invoke IDs and provides convenient methods for creating
//! requests and processing responses.
//!
//! # Optimization Considerations
//!
//! - ACTION operations are typically less frequent than GET/SET, so performance is less critical
//! - Method parameters may be large, so consider streaming or chunking for very large parameters
//! - Response data may be large, so consider using zero-copy types if the data is processed
//!   in multiple stages

use crate::pdu::{
    ActionRequest, ActionRequestNormal, ActionResponse, ActionResponseNormal, ActionResult,
    CosemMethodDescriptor, InvokeIdAndPriority,
};
use dlms_core::{DlmsError, DlmsResult, DataObject};

/// ACTION Service for DLMS/COSEM
///
/// Provides high-level interface for ACTION operations, handling PDU creation,
/// encoding/decoding, and result processing.
///
/// # Why a Separate Service Layer?
/// Similar to GET and SET services, the ACTION service provides:
/// - **Simplified API**: Hide PDU complexity from users
/// - **Result Processing**: Automatically extract data or error codes from responses
/// - **Invoke ID Management**: Track request/response pairs automatically
/// - **Type Safety**: Ensure correct PDU types are used for each operation
///
/// # Optimization Considerations
/// - ACTION operations typically return data, so the service handles both success
///   with data and success without data cases
/// - Method parameters are optional, allowing efficient handling of parameterless methods
/// - Future optimization: Add support for parameter blocks for very large parameters
#[derive(Debug, Clone)]
pub struct ActionService {
    /// Next invoke ID to use (0-127)
    next_invoke_id: u8,
}

impl ActionService {
    /// Create a new ACTION service
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

    /// Create a Normal ACTION request
    ///
    /// # Arguments
    /// * `invoke_id_and_priority` - Invoke ID and priority
    /// * `cosem_method_descriptor` - Method to invoke
    /// * `method_invocation_parameters` - Optional method parameters
    ///
    /// # Returns
    /// A `ActionRequest::Normal` PDU ready to be encoded and sent
    pub fn create_normal_request(
        invoke_id_and_priority: InvokeIdAndPriority,
        cosem_method_descriptor: CosemMethodDescriptor,
        method_invocation_parameters: Option<DataObject>,
    ) -> DlmsResult<ActionRequest> {
        Ok(ActionRequest::new_normal(
            invoke_id_and_priority,
            cosem_method_descriptor,
            method_invocation_parameters,
        ))
    }

    /// Process an ACTION response and extract the result
    ///
    /// # Arguments
    /// * `response` - The ACTION response PDU
    ///
    /// # Returns
    /// The result data (if any) or error code
    pub fn process_response(response: &ActionResponse) -> DlmsResult<Option<DataObject>> {
        match response {
            ActionResponse::Normal(normal) => {
                match &normal.result {
                    ActionResult::SuccessWithData(data) => Ok(Some(data.clone())),
                    ActionResult::Success => Ok(None),
                    ActionResult::DataAccessResult(code) => Err(DlmsError::InvalidData(format!(
                        "ACTION operation failed with error code: {}",
                        code
                    ))),
                }
            }
            _ => Err(DlmsError::InvalidData(
                "Expected Normal ACTION response".to_string(),
            )),
        }
    }
}

impl Default for ActionService {
    fn default() -> Self {
        Self::new()
    }
}
