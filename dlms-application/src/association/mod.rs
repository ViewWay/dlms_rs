//! Association module for DLMS/COSEM connections
//!
//! This module provides the application layer association management,
//! implementing COSEM-OPEN and COSEM-RELEASE service primitives.
//!
//! # Association Lifecycle
//!
//! ```text
//!     COSEM-OPEN.request           COSEM-RELEASE.request
//!    (AARQ sent)                   (RLRQ sent)
//!    -----------                   ------------
//!   |             |               |             |
//!   v             v               v             v
//! Inactive -> Idle -> AssociationPending -> Associated -> ReleasePending -> Inactive
//! ```
//!
//! # Example
//!
//! ```rust
//! use dlms_application::association::{Association, AssociationContext, SapAddress};
//!
//! let ctx = AssociationContext::with_defaults();
//! let association = Association::new(ctx);
//! ```

pub mod state;
pub mod context;
pub mod events;

pub use state::{
    AssociationState,
    OpenResult,
    ReleaseResult,
    RejectReason,
    AbortReason,
    ReleaseRejectReason,
};

pub use context::{
    AssociationContext,
    SapAddress,
    NegotiatedParameters,
    SystemTitle,
};

pub use events::{
    AssociationEvent,
    AssociationEventListener,
    CallbackEventListener,
};

#[cfg(feature = "tokio")]
pub use events::channel_listener::ChannelEventListener;

use std::sync::Arc;
use std::fmt;

// Re-export for convenience in this module
use crate::pdu::{InitiateRequest, InitiateResponse, Conformance};
use dlms_asn1::iso_acse::{AARQApdu, AAREApdu, RLRQApdu, RLREApdu, AssociateResult};
use dlms_core::DlmsResult;

/// Events emitter for internal use
#[derive(Clone)]
struct EventsEmitter {
    listeners: Arc<Vec<Arc<dyn AssociationEventListener>>>,
}

impl EventsEmitter {
    fn new() -> Self {
        Self {
            listeners: Arc::new(Vec::new()),
        }
    }

    fn add_listener(&mut self, listener: Arc<dyn AssociationEventListener>) {
        let mut listeners = self.listeners.as_ref().iter().cloned().collect::<Vec<_>>();
        listeners.push(listener);
        self.listeners = Arc::new(listeners);
    }

    fn emit(&self, event: AssociationEvent) {
        for listener in self.listeners.iter() {
            listener.on_event(event.clone());
        }
    }
}

impl Default for EventsEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for EventsEmitter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventsEmitter")
            .field("listener_count", &self.listeners.len())
            .finish()
    }
}

/// DLMS/COSEM Application Association
///
/// The Association struct manages an application layer connection between
/// a DLMS client and server, implementing COSEM-OPEN and COSEM-RELEASE
/// service primitives.
///
/// # Service Primitives
///
/// - **COSEM-OPEN.request**: `open()` - Establish application association
/// - **COSEM-OPEN.confirm**: Returns `OpenResult`
/// - **COSEM-RELEASE.request**: `release()` - Release application association
/// - **COSEM-RELEASE.confirm**: Returns `ReleaseResult`
/// - **COSEM-ABORT.indication**: `abort()` - Association aborted
///
/// # Example
///
/// ```rust
/// use dlms_application::association::{Association, AssociationContext, SapAddress};
///
/// let ctx = AssociationContext::with_defaults();
/// let mut association = Association::new(ctx);
///
/// // Check if association is active
/// if association.is_active() {
///     // Perform DLMS operations
/// }
///
/// // Release the association
/// let result = association.release();
/// ```
#[derive(Clone, Debug)]
pub struct Association {
    /// Association context containing state and parameters
    context: AssociationContext,

    /// Event emitters for listener notifications
    events: EventsEmitter,
}

impl Association {
    /// Create a new association
    ///
    /// # Arguments
    /// * `context` - The association context
    #[must_use]
    pub fn new(context: AssociationContext) -> Self {
        Self {
            context,
            events: EventsEmitter::new(),
        }
    }

    /// Create an association with default settings
    ///
    /// This creates an association with default SAP addresses (1/1).
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(AssociationContext::with_defaults())
    }

    /// Get the association context
    #[must_use]
    pub const fn context(&self) -> &AssociationContext {
        &self.context
    }

    /// Get a mutable reference to the association context
    pub fn context_mut(&mut self) -> &mut AssociationContext {
        &mut self.context
    }

    /// Get the current association state
    #[must_use]
    pub fn state(&self) -> AssociationState {
        self.context.state
    }

    /// Check if the association is active (can perform operations)
    ///
    /// Returns true if the state is `Associated`, meaning normal DLMS
    /// operations can be performed.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.context.is_active()
    }

    /// Check if the association is in a transitional state
    ///
    /// Returns true if the state is `AssociationPending` or `ReleasePending`.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.context.state.is_pending()
    }

    /// Check if the connection is established at physical layer
    ///
    /// Returns true if the state is `Idle` or any state beyond.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.context.state.is_connected()
    }

    /// Add an event listener
    ///
    /// # Arguments
    /// * `listener` - The listener to add
    pub fn add_listener(&mut self, listener: Arc<dyn AssociationEventListener>) {
        self.events.add_listener(listener);
    }

    /// Notify listeners of an event
    fn notify(&self, event: AssociationEvent) {
        self.events.emit(event);
    }

    /// Transition to a new state
    ///
    /// This method updates the association state and emits appropriate events.
    ///
    /// # Arguments
    /// * `new_state` - The new state to transition to
    pub fn transition_to(&mut self, new_state: AssociationState) {
        let old_state = self.context.state;

        // Emit event based on transition
        match (old_state, new_state) {
            (AssociationState::Idle, AssociationState::Associated) => {
                self.notify(AssociationEvent::Established {
                    version: self.context.negotiated_params()
                        .map(|p| p.dlms_version)
                        .unwrap_or(6),
                });
            }
            (AssociationState::AssociationPending, AssociationState::Idle) => {
                self.notify(AssociationEvent::EstablishmentFailed {
                    reason: RejectReason::None,
                    diagnostic: None,
                });
            }
            (AssociationState::Associated, AssociationState::Inactive) |
            (AssociationState::ReleasePending, AssociationState::Inactive) => {
                self.notify(AssociationEvent::Released);
            }
            (_, AssociationState::Inactive) => {
                self.notify(AssociationEvent::ConnectionLost);
            }
            _ => {}
        }

        self.context.transition_to(new_state);
    }

    // ============================================================
    // COSEM-OPEN Service Primitive Helper Methods
    // ============================================================

    /// Build AARQ APDU from InitiateRequest (COSEM-OPEN.request preparation)
    ///
    /// This method creates an AARQ (Association Request) APDU containing
    /// the InitiateRequest in the user_information field.
    ///
    /// # Arguments
    /// * `initiate_request` - The InitiateRequest PDU to include
    /// * `application_context_name` - Application context OID (optional)
    ///
    /// # Returns
    /// Returns the encoded AARQ APDU bytes ready for transmission.
    ///
    /// # Example
    /// ```rust
    /// use dlms_application::association::Association;
    /// use dlms_application::pdu::InitiateRequest;
    ///
    /// let mut association = Association::with_defaults();
    /// let initiate_req = InitiateRequest::new();
    ///
    /// let aarq_bytes = association.build_aarq(&initiate_req, None).unwrap();
    /// // Send aarq_bytes to server...
    /// ```
    pub fn build_aarq(
        &self,
        initiate_request: &InitiateRequest,
        application_context_name: Option<Vec<u32>>,
    ) -> DlmsResult<Vec<u8>> {
        // Use DLMS/COSEM application context if not specified
        let app_ctx = application_context_name.unwrap_or_else(|| {
            // DLMS/COSEM logical name application context: 1.0.17.0.0.8.0.101
            vec![1, 0, 17, 0, 0, 8, 0, 101]
        });

        // Create AARQ
        let mut aarq = AARQApdu::new(app_ctx);

        // Encode InitiateRequest and add to user_information
        let initiate_bytes = initiate_request.encode()?;
        aarq.set_initiate_request(initiate_bytes);

        // Encode AARQ to BER
        aarq.encode()
    }

    /// Process AARE APDU and extract InitiateResponse (COSEM-OPEN.confirm handling)
    ///
    /// This method decodes an AARE (Association Response) APDU and extracts
    /// the InitiateResponse from the user_information field.
    ///
    /// # Arguments
    /// * `aare_bytes` - The received AARE APDU bytes
    ///
    /// # Returns
    /// Returns `Ok(OpenResult)` indicating the result of association establishment.
    ///
    /// # Example
    /// ```rust
    /// use dlms_application::association::Association;
    ///
    /// let mut association = Association::with_defaults();
    /// association.on_connected(); // Set state to Idle
    ///
    /// // After receiving AARE from server...
    /// // let aare_bytes = receive_from_server();
    /// // let result = association.process_aare(&aare_bytes)?;
    /// ```
    pub fn process_aare(&mut self, aare_bytes: &[u8]) -> DlmsResult<OpenResult> {
        // Decode AARE
        let aare = AAREApdu::decode(aare_bytes)?;

        // Check if association was accepted
        match aare.result {
            AssociateResult::Accepted => {
                // Extract InitiateResponse from user_information
                if let Some(initiate_bytes) = aare.get_initiate_response() {
                    match InitiateResponse::decode(initiate_bytes) {
                        Ok(initiate_res) => {
                            // Create negotiated parameters
                            let negotiated_params = NegotiatedParameters::from_initiate(
                                &InitiateRequest::new(), // Use defaults for client params
                                &initiate_res,
                            );

                            // Update context
                            self.context.update_negotiated_params(negotiated_params);

                            // Transition to Associated state
                            self.transition_to(AssociationState::Associated);

                            Ok(OpenResult::Success {
                                version: initiate_res.negotiated_dlms_version_number,
                                conformance: {
                                    let bytes = initiate_res.negotiated_conformance.bits().to_bytes();
                                    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], 0])
                                },
                                max_pdu_size: initiate_res.server_max_receive_pdu_size,
                            })
                        }
                        Err(e) => {
                            // Failed to decode InitiateResponse
                            self.transition_to(AssociationState::Idle);
                            Ok(OpenResult::Failed {
                                error: format!("Failed to decode InitiateResponse: {}", e),
                            })
                        }
                    }
                } else {
                    // No InitiateResponse in AARE
                    self.transition_to(AssociationState::Idle);
                    Ok(OpenResult::Failed {
                        error: "AARE missing InitiateResponse".to_string(),
                    })
                }
            }
            AssociateResult::RejectedPermanent | AssociateResult::RejectedTransient => {
                // Association rejected
                self.transition_to(AssociationState::Idle);

                let reason = match aare.result_source_diagnostic.value() {
                    0 => RejectReason::None,
                    1 => RejectReason::NotAuthorized,
                    2 => RejectReason::CalledApTitleNotRecognized,
                    3 => RejectReason::CalledApInvocationIdNotRecognized,
                    4 => RejectReason::NotAuthorizedToInvoke,
                    n => RejectReason::Other(n as u8),
                };

                Ok(OpenResult::Rejected {
                    reason,
                    diagnostic: Some(format!("Result: {:?}", aare.result)),
                })
            }
        }
    }

    // ============================================================
    // COSEM-RELEASE Service Primitive Helper Methods
    // ============================================================

    /// Build RLRQ APDU (COSEM-RELEASE.request preparation)
    ///
    /// This method creates an RLRQ (Release Request) APDU.
    ///
    /// # Returns
    /// Returns the encoded RLRQ APDU bytes ready for transmission.
    ///
    /// # Example
    /// ```rust
    /// use dlms_application::association::Association;
    ///
    /// let mut association = Association::with_defaults();
    /// // ... association is active ...
    ///
    /// let rlrq_bytes = association.build_rlrq().unwrap();
    /// // Send rlrq_bytes to server...
    /// ```
    pub fn build_rlrq(&self) -> DlmsResult<Vec<u8>> {
        let rlrq = RLRQApdu::new();
        rlrq.encode()
    }

    /// Process RLRE APDU (COSEM-RELEASE.confirm handling)
    ///
    /// This method decodes an RLRE (Release Response) APDU.
    ///
    /// # Arguments
    /// * `rlre_bytes` - The received RLRE APDU bytes
    ///
    /// # Returns
    /// Returns `Ok(ReleaseResult)` indicating the result of association release.
    ///
    /// # Example
    /// ```rust
    /// use dlms_application::association::Association;
    ///
    /// let mut association = Association::with_defaults();
    /// // ... association is active ...
    ///
    /// // After receiving RLRE from server...
    /// // let rlre_bytes = receive_from_server();
    /// // let result = association.process_rlre(&rlre_bytes)?;
    /// ```
    pub fn process_rlre(&mut self, rlre_bytes: &[u8]) -> DlmsResult<ReleaseResult> {
        // Decode RLRE
        let _rlre = RLREApdu::decode(rlre_bytes)?;

        // Transition to Inactive state
        self.transition_to(AssociationState::Inactive);

        Ok(ReleaseResult::Success)
    }

    /// Open the association (COSEM-OPEN.request)
    ///
    /// This method initiates the association establishment process.
    /// In a client implementation, this would send an AARQ APDU.
    ///
    /// # Returns
    ///
    /// Returns `OpenResult` indicating success or failure.
    ///
    /// # Note
    ///
    /// This is a placeholder for the full implementation which requires
    /// AARQ/AARE encoding support. The actual implementation will be
    /// completed in Phase 2.
    pub fn open(&mut self) -> OpenResult {
        if !self.context.state.is_connected() {
            return OpenResult::Failed {
                error: "Physical connection not established".to_string(),
            };
        }

        if self.context.state.is_active() {
            return OpenResult::Failed {
                error: "Association already active".to_string(),
            };
        }

        // Transition to pending state
        self.context.transition_to(AssociationState::AssociationPending);

        // Placeholder: In Phase 2, this will:
        // 1. Encode AARQ with InitiateRequest in user_Information
        // 2. Send AARQ to server
        // 3. Wait for and decode AARE
        // 4. Extract InitiateResponse from user_Information
        // 5. Update negotiated parameters
        // 6. Transition to Associated or Idle based on result

        // For now, return a placeholder success
        OpenResult::Success {
            version: 6,
            conformance: 0,
            max_pdu_size: 2048,
        }
    }

    /// Release the association (COSEM-RELEASE.request)
    ///
    /// This method initiates the association release process.
    /// In a client implementation, this would send an RLRQ APDU.
    ///
    /// # Returns
    ///
    /// Returns `ReleaseResult` indicating success or failure.
    ///
    /// # Note
    ///
    /// This is a placeholder for the full implementation.
    /// The actual implementation will be completed in Phase 2.
    pub fn release(&mut self) -> ReleaseResult {
        if !self.context.state.is_active() {
            return ReleaseResult::Failed {
                error: "No active association to release".to_string(),
            };
        }

        // Transition to pending state
        self.context.transition_to(AssociationState::ReleasePending);

        // Placeholder: In Phase 2, this will:
        // 1. Encode RLRQ
        // 2. Send RLRQ to server
        // 3. Wait for and decode RLRE
        // 4. Transition to Inactive

        self.context.transition_to(AssociationState::Inactive);
        ReleaseResult::Success
    }

    /// Abort the association (COSEM-ABORT.indication)
    ///
    /// This method handles association abort, typically called when
    /// the connection is unexpectedly lost or aborted.
    ///
    /// # Arguments
    /// * `reason` - The reason for the abort
    pub fn abort(&mut self, reason: AbortReason) {
        self.notify(AssociationEvent::Aborted { reason });
        self.context.transition_to(AssociationState::Inactive);
    }

    /// Handle connection established event
    ///
    /// This should be called when the physical connection (HDLC/Wrapper)
    /// is established.
    pub fn on_connected(&mut self) {
        if self.context.state == AssociationState::Inactive {
            self.context.transition_to(AssociationState::Idle);
        }
    }

    /// Handle connection lost event
    ///
    /// This should be called when the physical connection is lost.
    pub fn on_connection_lost(&mut self) {
        self.notify(AssociationEvent::ConnectionLost);
        self.context.transition_to(AssociationState::Inactive);
    }

    /// Handle authentication failure
    ///
    /// This should be called when authentication fails during
    /// association establishment.
    ///
    /// # Arguments
    /// * `details` - Details about the authentication failure
    pub fn on_auth_failure(&mut self, details: String) {
        self.notify(AssociationEvent::AuthenticationFailed { details });
        self.context.transition_to(AssociationState::Idle);
    }

    /// Handle protocol error
    ///
    /// This should be called when a protocol error occurs.
    ///
    /// # Arguments
    /// * `error` - Error description
    pub fn on_protocol_error(&mut self, error: String) {
        self.notify(AssociationEvent::ProtocolError { error });
    }
}

impl Default for Association {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl From<AssociationContext> for Association {
    fn from(context: AssociationContext) -> Self {
        Self::new(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_association_new() {
        let association = Association::with_defaults();
        assert_eq!(association.state(), AssociationState::Inactive);
        assert!(!association.is_active());
        assert!(!association.is_connected());
    }

    #[test]
    fn test_association_from_context() {
        let ctx = AssociationContext::with_defaults();
        let association = Association::new(ctx);
        assert_eq!(association.state(), AssociationState::Inactive);
    }

    #[test]
    fn test_on_connected() {
        let mut association = Association::with_defaults();
        association.on_connected();
        assert_eq!(association.state(), AssociationState::Idle);
        assert!(association.is_connected());
        assert!(!association.is_active());
    }

    #[test]
    fn test_abort() {
        let mut association = Association::with_defaults();
        association.on_connected();
        association.abort(AbortReason::Other(1));
        assert_eq!(association.state(), AssociationState::Inactive);
    }

    #[test]
    fn test_on_connection_lost() {
        let mut association = Association::with_defaults();
        association.on_connected();
        association.on_connection_lost();
        assert_eq!(association.state(), AssociationState::Inactive);
    }

    #[test]
    fn test_release_without_association() {
        let mut association = Association::with_defaults();
        let result = association.release();
        assert!(matches!(result, ReleaseResult::Failed { .. }));
    }

    #[test]
    fn test_open_without_connection() {
        let mut association = Association::with_defaults();
        let result = association.open();
        assert!(matches!(result, OpenResult::Failed { .. }));
    }

    #[test]
    fn test_event_listener() {
        use std::sync::{Arc, Mutex};

        let mut association = Association::with_defaults();
        let last_event = Arc::new(Mutex::new(None));

        let listener = {
            let last_event = last_event.clone();
            Arc::new(CallbackEventListener::new(move |event| {
                *last_event.lock().unwrap() = Some(event);
            }))
        };

        association.add_listener(listener);
        association.on_connected();

        let received = last_event.lock().unwrap().clone();
        // Connection doesn't trigger an event, only state transitions from connected states
        assert!(received.is_none());
    }

    #[test]
    fn test_build_aarq() {
        let association = Association::with_defaults();
        let initiate_req = InitiateRequest::new();

        let aarq_bytes = association.build_aarq(&initiate_req, None);
        assert!(aarq_bytes.is_ok());

        // Verify AARQ can be decoded
        let decoded = AARQApdu::decode(&aarq_bytes.unwrap());
        assert!(decoded.is_ok());
    }

    #[test]
    fn test_build_rlrq() {
        let association = Association::with_defaults();

        let rlrq_bytes = association.build_rlrq();
        assert!(rlrq_bytes.is_ok());

        // Verify RLRQ can be decoded
        let decoded = RLRQApdu::decode(&rlrq_bytes.unwrap());
        assert!(decoded.is_ok());
    }

    #[test]
    fn test_process_aare_accepted() {
        let mut association = Association::with_defaults();
        association.on_connected(); // Set state to Idle

        // Create a successful AARE with InitiateResponse
        let initiate_res = InitiateResponse::new();

        // Create AARE with InitiateResponse in user_information
        let mut aare = AAREApdu::new(
            vec![1, 0, 17, 0, 0, 8, 0, 101], // DLMS application context
            AssociateResult::Accepted,
            AssociateSourceDiagnostic::new(0),
        );
        let initiate_bytes = initiate_res.encode().unwrap();
        aare.set_initiate_response(initiate_bytes);

        let aare_bytes = aare.encode().unwrap();
        let result = association.process_aare(&aare_bytes);

        assert!(result.is_ok());
        let open_result = result.unwrap();
        assert!(matches!(open_result, OpenResult::Success { .. }));
        assert_eq!(association.state(), AssociationState::Associated);
        assert!(association.is_active());
    }

    #[test]
    fn test_process_aare_rejected() {
        let mut association = Association::with_defaults();
        association.on_connected(); // Set state to Idle

        // Create a rejecting AARE
        let aare = AAREApdu::new(
            vec![1, 0, 17, 0, 0, 8, 0, 101], // DLMS application context
            AssociateResult::RejectedPermanent,
            AssociateSourceDiagnostic::new(1), // Not authorized
        );

        let aare_bytes = aare.encode().unwrap();
        let result = association.process_aare(&aare_bytes);

        assert!(result.is_ok());
        let open_result = result.unwrap();
        assert!(matches!(open_result, OpenResult::Rejected { .. }));
        assert_eq!(association.state(), AssociationState::Idle);
    }

    #[test]
    fn test_process_rlre() {
        let mut association = Association::with_defaults();
        // Manually set to Associated state
        association.context.transition_to(AssociationState::Associated);

        let rlre = RLREApdu::new();
        let rlre_bytes = rlre.encode().unwrap();
        let result = association.process_rlre(&rlre_bytes);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ReleaseResult::Success));
        assert_eq!(association.state(), AssociationState::Inactive);
    }
}
