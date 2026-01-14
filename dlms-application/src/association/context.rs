//! Association context for DLMS/COSEM connections
//!
//! This module defines the AssociationContext structure which holds all the
//! information about an active DLMS/COSEM association.

use crate::pdu::{Conformance, InitiateRequest, InitiateResponse};
use crate::association::state::AssociationState;
use dlms_core::DlmsResult;

/// Service Access Point address
///
/// SAP addresses identify the client and server in a DLMS/COSEM association.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SapAddress {
    /// SAP address value (0-65535)
    pub value: u16,
}

impl SapAddress {
    /// Create a new SAP address
    ///
    /// # Arguments
    /// * `value` - SAP address value (0-65535)
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self { value }
    }

    /// Get the SAP address value
    #[must_use]
    pub const fn get(&self) -> u16 {
        self.value
    }

    /// Default client SAP address
    pub const DEFAULT_CLIENT: u16 = 1;

    /// Default server SAP address
    pub const DEFAULT_SERVER: u16 = 1;
}

/// Negotiated protocol parameters from InitiateRequest/Response exchange
///
/// These parameters are negotiated during association establishment and
/// define the capabilities and constraints for the connection.
#[derive(Debug, Clone, PartialEq)]
pub struct NegotiatedParameters {
    /// DLMS version number (typically 6 for DLMS/COSEM Edition 9)
    pub dlms_version: u8,

    /// Conformance bits indicating supported features
    ///
    /// Each bit represents a specific capability (GET, SET, ACTION, etc.)
    pub conformance: Conformance,

    /// Maximum PDU size the client can receive
    pub client_max_receive_pdu_size: u16,

    /// Maximum PDU size the server can receive
    pub server_max_receive_pdu_size: u16,

    /// Negotiated quality of service (optional)
    pub quality_of_service: Option<i8>,
}

impl Default for NegotiatedParameters {
    fn default() -> Self {
        Self {
            dlms_version: 6,
            conformance: Conformance::new(),
            client_max_receive_pdu_size: 2048,
            server_max_receive_pdu_size: 2048,
            quality_of_service: None,
        }
    }
}

impl NegotiatedParameters {
    /// Create negotiated parameters from InitiateRequest and InitiateResponse
    ///
    /// # Arguments
    /// * `init_req` - The InitiateRequest sent by client
    /// * `init_res` - The InitiateResponse received from server
    pub fn from_initiate(
        init_req: &InitiateRequest,
        init_res: &InitiateResponse,
    ) -> Self {
        // The server selects the minimum of client and server capabilities
        Self {
            dlms_version: init_res.negotiated_dlms_version_number,
            conformance: init_res.negotiated_conformance.clone(),
            client_max_receive_pdu_size: init_req.client_max_receive_pdu_size,
            server_max_receive_pdu_size: init_res.server_max_receive_pdu_size,
            quality_of_service: init_res.negotiated_quality_of_service,
        }
    }

    /// Get the minimum (negotiated) PDU size
    #[must_use]
    pub fn negotiated_pdu_size(&self) -> u16 {
        self.client_max_receive_pdu_size.min(self.server_max_receive_pdu_size)
    }
}

/// System title for encryption/authentication
///
/// The system title is a unique identifier for the device (8 bytes).
/// It consists of:
/// - 3 bytes: Manufacturer ID (assigned by DLMS UA)
/// - 5 bytes: Unique identifier for the device
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemTitle {
    bytes: [u8; 8],
}

impl SystemTitle {
    /// Create a new system title from bytes
    ///
    /// # Arguments
    /// * `bytes` - 8-byte system title
    pub fn new(bytes: [u8; 8]) -> Self {
        Self { bytes }
    }

    /// Create a system title from manufacturer ID and unique ID
    ///
    /// # Arguments
    /// * `manufacturer_id` - 3-byte manufacturer ID
    /// * `unique_id` - 5-byte unique identifier
    pub fn from_parts(manufacturer_id: [u8; 3], unique_id: [u8; 5]) -> DlmsResult<Self> {
        let mut bytes = [0u8; 8];
        bytes[0..3].copy_from_slice(&manufacturer_id);
        bytes[3..8].copy_from_slice(&unique_id);
        Ok(Self { bytes })
    }

    /// Get the system title bytes
    #[must_use]
    pub const fn bytes(&self) -> &[u8; 8] {
        &self.bytes
    }

    /// Get the manufacturer ID (first 3 bytes)
    #[must_use]
    pub fn manufacturer_id(&self) -> [u8; 3] {
        [self.bytes[0], self.bytes[1], self.bytes[2]]
    }

    /// Get the unique ID (last 5 bytes)
    #[must_use]
    pub fn unique_id(&self) -> [u8; 5] {
        [
            self.bytes[3],
            self.bytes[4],
            self.bytes[5],
            self.bytes[6],
            self.bytes[7],
        ]
    }
}

/// Association context
///
/// Contains all information about an active DLMS/COSEM association,
/// including state, addresses, negotiated parameters, and security context.
#[derive(Debug, Clone)]
pub struct AssociationContext {
    /// Current association state
    pub state: AssociationState,

    /// Client SAP address
    pub client_sap: SapAddress,

    /// Server SAP address
    pub server_sap: SapAddress,

    /// Local system title (for encryption/authentication)
    pub local_title: Option<SystemTitle>,

    /// Remote system title (for encryption/authentication)
    pub remote_title: Option<SystemTitle>,

    /// Negotiated protocol parameters
    pub negotiated_params: Option<NegotiatedParameters>,
}

impl AssociationContext {
    /// Create a new association context
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    /// * `server_sap` - Server SAP address
    #[must_use]
    pub fn new(client_sap: SapAddress, server_sap: SapAddress) -> Self {
        Self {
            state: AssociationState::Inactive,
            client_sap,
            server_sap,
            local_title: None,
            remote_title: None,
            negotiated_params: None,
        }
    }

    /// Create with default SAP addresses
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(
            SapAddress::new(SapAddress::DEFAULT_CLIENT),
            SapAddress::new(SapAddress::DEFAULT_SERVER),
        )
    }

    /// Set the local system title
    pub fn with_local_title(mut self, title: SystemTitle) -> Self {
        self.local_title = Some(title);
        self
    }

    /// Set the remote system title
    pub fn with_remote_title(mut self, title: SystemTitle) -> Self {
        self.remote_title = Some(title);
        self
    }

    /// Get the association state
    #[must_use]
    pub const fn state(&self) -> &AssociationState {
        &self.state
    }

    /// Check if the association is active (can perform operations)
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    /// Update the negotiated parameters
    pub fn update_negotiated_params(&mut self, params: NegotiatedParameters) {
        self.negotiated_params = Some(params);
    }

    /// Get the negotiated parameters
    ///
    /// Returns None if parameters have not been negotiated yet.
    #[must_use]
    pub fn negotiated_params(&self) -> Option<&NegotiatedParameters> {
        self.negotiated_params.as_ref()
    }

    /// Get the negotiated PDU size
    ///
    /// Returns the negotiated size, or a default if not yet negotiated.
    #[must_use]
    pub fn pdu_size(&self) -> u16 {
        self.negotiated_params
            .as_ref()
            .map(|p| p.negotiated_pdu_size())
            .unwrap_or(2048)
    }

    /// Check if a feature is supported based on conformance bits
    ///
    /// # Arguments
    /// * `bit_index` - Conformance bit index (0-23)
    ///
    /// Returns false if not yet negotiated or the bit is not set.
    pub fn supports_feature(&self, bit_index: usize) -> bool {
        self.negotiated_params
            .as_ref()
            .and_then(|p| p.conformance.get_bit(bit_index))
            .unwrap_or(false)
    }

    /// Check if GET operation is supported
    #[must_use]
    pub fn supports_get(&self) -> bool {
        self.supports_feature(19) // bit 19 = GET
    }

    /// Check if SET operation is supported
    #[must_use]
    pub fn supports_set(&self) -> bool {
        self.supports_feature(20) // bit 20 = SET
    }

    /// Check if ACTION operation is supported
    #[must_use]
    pub fn supports_action(&self) -> bool {
        self.supports_feature(23) // bit 23 = ACTION
    }

    /// Check if selective access is supported
    #[must_use]
    pub fn supports_selective_access(&self) -> bool {
        self.supports_feature(21) // bit 21 = Selective access
    }

    /// Transition to a new state
    pub fn transition_to(&mut self, new_state: AssociationState) {
        self.state = new_state;
    }
}

impl Default for AssociationContext {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sap_address() {
        let sap = SapAddress::new(100);
        assert_eq!(sap.get(), 100);
    }

    #[test]
    fn test_system_title() {
        let title = SystemTitle::new([1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(title.manufacturer_id(), [1, 2, 3]);
        assert_eq!(title.unique_id(), [4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_system_title_from_parts() {
        let title = SystemTitle::from_parts([0xAA, 0xBB, 0xCC], [1, 2, 3, 4, 5]).unwrap();
        assert_eq!(title.bytes(), &[0xAA, 0xBB, 0xCC, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_association_context_new() {
        let ctx = AssociationContext::with_defaults();
        assert_eq!(ctx.state, AssociationState::Inactive);
        assert!(!ctx.is_active());
    }

    #[test]
    fn test_association_context_with_titles() {
        let local = SystemTitle::new([1, 2, 3, 4, 5, 6, 7, 8]);
        let remote = SystemTitle::new([9, 10, 11, 12, 13, 14, 15, 16]);

        let ctx = AssociationContext::with_defaults()
            .with_local_title(local)
            .with_remote_title(remote);

        assert!(ctx.local_title.is_some());
        assert!(ctx.remote_title.is_some());
    }

    #[test]
    fn test_negotiated_parameters() {
        let params = NegotiatedParameters {
            dlms_version: 6,
            conformance: Conformance::new(),
            client_max_receive_pdu_size: 2048,
            server_max_receive_pdu_size: 1024,
            quality_of_service: None,
        };

        assert_eq!(params.negotiated_pdu_size(), 1024);
    }
}
