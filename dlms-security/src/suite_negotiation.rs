//! Security Suite Negotiation
//!
//! This module provides functionality for negotiating security suites between
//! DLMS/COSEM clients and servers.
//!
//! # Overview
//!
//! Security suite negotiation allows a client and server to agree on the
//! security mechanisms to use for their communication. The negotiation process:
//!
//! 1. **Client sends supported suites**: Client announces which security suites it supports
//! 2. **Server selects best suite**: Server selects the most secure suite that both support
//! 3. **Server confirms selection**: Server confirms the selected suite to the client
//! 4. **Both sides configure**: Both sides configure their security context with the agreed suite
//!
//! # Selection Algorithm
//!
//! The suite selection algorithm follows these priority rules:
//! 1. Prefer authentication + encryption over encryption only
//! 2. Prefer encryption only over authentication only
//! 3. Prefer authentication only over no security
//! 4. Within same security level, prefer more secure mechanisms
//!
//! # Example
//!
//! ```rust,no_run
//! use dlms_security::suite_negotiation::{
//!     SecuritySuiteNegotiator, SuiteId, create_common_suites
//! };
//! use dlms_security::suite::{EncryptionMechanism, AuthenticationMechanism};
//!
//! // Client side
//! let client_suites = create_common_suites();
//! let mut client = SecuritySuiteNegotiator::new(client_suites);
//!
//! // Generate proposal to send to server
//! let proposal = client.generate_proposal()?;
//!
//! // Server side
//! let server_suites = create_common_suites();
//! let mut server = SecuritySuiteNegotiator::new(server_suites);
//!
//! // Server selects best matching suite
//! let selected = server.process_and_select(&proposal)?;
//!
//! // Client accepts the selection
//! client.receive_selection(selected)?;
//!
//! // Both sides now use the selected suite
//! assert!(client.is_complete());
//! assert!(server.is_complete());
//! ```

use crate::error::{DlmsError, DlmsResult};
use crate::suite::{SecuritySuite, SecurityPolicy, EncryptionMechanism, AuthenticationMechanism};
use std::collections::HashSet;
use std::fmt;
use std::time::Duration;

/// Security suite identifier for negotiation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SuiteId {
    /// Encryption mechanism
    pub encryption: EncryptionMechanism,
    /// Authentication mechanism
    pub authentication: AuthenticationMechanism,
}

impl SuiteId {
    /// Create a new SuiteId
    pub fn new(
        encryption: EncryptionMechanism,
        authentication: AuthenticationMechanism,
    ) -> Self {
        Self {
            encryption,
            authentication,
        }
    }

    /// Get security score for comparison (higher is more secure)
    ///
    /// Scoring:
    /// - Authenticated + Encrypted: 3
    /// - Encrypted only: 2
    /// - Authenticated only: 1
    /// - No security: 0
    pub fn security_score(&self) -> u8 {
        let has_encryption = self.encryption != EncryptionMechanism::None;
        let has_auth = self.authentication != AuthenticationMechanism::None
            && self.authentication != AuthenticationMechanism::Absent;

        match (has_encryption, has_auth) {
            (true, true) => 3,
            (true, false) => 2,
            (false, true) => 1,
            (false, false) => 0,
        }
    }

    /// Check if this suite is more secure than another
    pub fn is_more_secure_than(&self, other: &SuiteId) -> bool {
        self.security_score() > other.security_score()
    }

    /// Get the default security policy for this suite
    pub fn default_policy(&self) -> SecurityPolicy {
        let has_encryption = self.encryption != EncryptionMechanism::None;
        let has_auth = self.authentication != AuthenticationMechanism::None
            && self.authentication != AuthenticationMechanism::Absent;

        match (has_encryption, has_auth) {
            (true, true) => SecurityPolicy::AuthenticatedAndEncrypted,
            (true, false) => SecurityPolicy::Encrypted,
            (false, true) => SecurityPolicy::Authenticated,
            (false, false) => SecurityPolicy::Nothing,
        }
    }
}

impl fmt::Display for SuiteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SuiteId(encryption={:?}, auth={:?})",
            self.encryption, self.authentication
        )
    }
}

impl From<SuiteId> for SecuritySuite {
    fn from(suite_id: SuiteId) -> Self {
        // Use internal constructor to create SecuritySuite without keys
        // Keys should be set separately after suite negotiation
        SecuritySuite::from_mechanisms(
            suite_id.encryption,
            suite_id.authentication,
            suite_id.default_policy(),
        )
    }
}

/// Security suite proposal for negotiation
#[derive(Debug, Clone)]
pub struct SuiteProposal {
    /// Suite identifier
    pub suite_id: SuiteId,
    /// Supported security policies
    pub policies: Vec<SecurityPolicy>,
    /// Additional negotiation parameters
    pub parameters: NegotiationParameters,
}

impl SuiteProposal {
    /// Create a new suite proposal
    pub fn new(
        encryption: EncryptionMechanism,
        authentication: AuthenticationMechanism,
    ) -> Self {
        let suite_id = SuiteId::new(encryption, authentication);
        Self {
            suite_id,
            policies: vec![suite_id.default_policy()],
            parameters: NegotiationParameters::default(),
        }
    }

    /// Add a security policy to the proposal
    pub fn with_policy(mut self, policy: SecurityPolicy) -> Self {
        self.policies.push(policy);
        self
    }

    /// Set negotiation parameters
    pub fn with_parameters(mut self, parameters: NegotiationParameters) -> Self {
        self.parameters = parameters;
        self
    }

    /// Check if a given policy is supported
    pub fn supports_policy(&self, policy: SecurityPolicy) -> bool {
        self.policies.contains(&policy)
    }
}

/// Negotiation parameters for security suite
#[derive(Debug, Clone)]
pub struct NegotiationParameters {
    /// Maximum frame size supported
    pub max_frame_size: Option<u16>,
    /// Supported cipher suites
    pub cipher_suites: Vec<String>,
    /// Maximum number of pending requests
    pub max_pending_requests: Option<u8>,
    /// Custom parameters
    pub custom_params: Vec<(String, Vec<u8>)>,
}

impl Default for NegotiationParameters {
    fn default() -> Self {
        Self {
            max_frame_size: None,
            cipher_suites: vec![],
            max_pending_requests: None,
            custom_params: vec![],
        }
    }
}

/// Negotiation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NegotiationState {
    /// Not started
    NotStarted,
    /// Proposal sent, waiting for response
    ProposalSent,
    /// Proposal received, response pending
    ProposalReceived,
    /// Negotiation completed
    Completed,
    /// Negotiation failed
    Failed,
}

impl NegotiationState {
    /// Check if state is terminal (completed or failed)
    pub fn is_terminal(&self) -> bool {
        matches!(self, NegotiationState::Completed | NegotiationState::Failed)
    }
}

/// Negotiation timeout configuration
#[derive(Debug, Clone)]
pub struct NegotiationTimeout {
    /// Wait time for response
    pub response_timeout: Duration,
    /// Maximum retry attempts
    pub max_retries: u8,
    /// Delay between retries
    pub retry_delay: Duration,
}

impl Default for NegotiationTimeout {
    fn default() -> Self {
        Self {
            response_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_secs(5),
        }
    }
}

/// Negotiation error
#[derive(Debug, Clone)]
pub enum NegotiationError {
    /// Operation timed out
    Timeout,
    /// Maximum retry attempts exceeded
    MaxRetriesExceeded,
    /// No compatible suite found
    NoCompatibleSuite,
    /// Proposal rejected by peer
    RefusedByPeer,
    /// Invalid response received
    InvalidResponse,
    /// Invalid state for operation
    InvalidState,
}

impl fmt::Display for NegotiationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NegotiationError::Timeout => write!(f, "Negotiation timed out"),
            NegotiationError::MaxRetriesExceeded => write!(f, "Maximum retry attempts exceeded"),
            NegotiationError::NoCompatibleSuite => write!(f, "No compatible security suite found"),
            NegotiationError::RefusedByPeer => write!(f, "Proposal refused by peer"),
            NegotiationError::InvalidResponse => write!(f, "Invalid response received"),
            NegotiationError::InvalidState => write!(f, "Invalid state for operation"),
        }
    }
}

impl std::error::Error for NegotiationError {}

/// Security suite negotiator
///
/// Handles the negotiation process between client and server to select
/// the most appropriate security suite.
pub struct SecuritySuiteNegotiator {
    /// Supported suites (local side)
    supported_suites: HashSet<SuiteId>,
    /// Preferred suites (ordered by preference)
    preferred_suites: Vec<SuiteId>,
    /// Negotiation state
    state: NegotiationState,
    /// Timeout configuration
    timeout_config: NegotiationTimeout,
    /// Retry counter
    retry_count: u8,
    /// Selected suite (after successful negotiation)
    selected_suite: Option<SuiteId>,
}

impl SecuritySuiteNegotiator {
    /// Create a new security suite negotiator
    ///
    /// # Arguments
    /// * `supported_suites` - Suites supported by this side
    pub fn new(supported_suites: Vec<SuiteId>) -> Self {
        let suite_set: HashSet<SuiteId> = supported_suites.iter().cloned().collect();
        Self {
            supported_suites: suite_set,
            preferred_suites: supported_suites,
            state: NegotiationState::NotStarted,
            timeout_config: NegotiationTimeout::default(),
            retry_count: 0,
            selected_suite: None,
        }
    }

    /// Create with custom preference order
    ///
    /// # Arguments
    /// * `supported_suites` - All suites this side supports
    /// * `preferred_order` - Preferred suites in order of preference
    pub fn with_preferences(
        supported_suites: Vec<SuiteId>,
        preferred_order: Vec<SuiteId>,
    ) -> Self {
        let suite_set: HashSet<SuiteId> = supported_suites.iter().cloned().collect();
        Self {
            supported_suites: suite_set,
            preferred_suites: preferred_order,
            state: NegotiationState::NotStarted,
            timeout_config: NegotiationTimeout::default(),
            retry_count: 0,
            selected_suite: None,
        }
    }

    /// Create with timeout configuration
    pub fn with_timeout(
        supported_suites: Vec<SuiteId>,
        timeout_config: NegotiationTimeout,
    ) -> Self {
        let suite_set: HashSet<SuiteId> = supported_suites.iter().cloned().collect();
        Self {
            supported_suites: suite_set,
            preferred_suites: supported_suites,
            state: NegotiationState::NotStarted,
            timeout_config,
            retry_count: 0,
            selected_suite: None,
        }
    }

    /// Get the current negotiation state
    pub fn state(&self) -> NegotiationState {
        self.state
    }

    /// Get the selected suite (if negotiation completed successfully)
    pub fn selected_suite(&self) -> Option<SuiteId> {
        self.selected_suite
    }

    /// Check if negotiation is complete
    pub fn is_complete(&self) -> bool {
        self.state == NegotiationState::Completed
    }

    /// Check if negotiation failed
    pub fn is_failed(&self) -> bool {
        self.state == NegotiationState::Failed
    }

    /// Get retry count
    pub fn retry_count(&self) -> u8 {
        self.retry_count
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count = self.retry_count.saturating_add(1);
    }

    /// Check if max retries exceeded
    pub fn is_max_retries_exceeded(&self) -> bool {
        self.retry_count >= self.timeout_config.max_retries
    }

    /// Generate a proposal for the other side
    ///
    /// Returns a list of supported suites ordered by preference.
    pub fn generate_proposal(&mut self) -> DlmsResult<Vec<SuiteProposal>> {
        if self.state != NegotiationState::NotStarted {
            return Err(DlmsError::Security(
                "Negotiation already started".to_string(),
            ));
        }

        let proposals: Vec<SuiteProposal> = self
            .preferred_suites
            .iter()
            .map(|suite_id| {
                SuiteProposal::new(suite_id.encryption, suite_id.authentication)
                    .with_policy(suite_id.default_policy())
            })
            .collect();

        self.state = NegotiationState::ProposalSent;
        Ok(proposals)
    }

    /// Process a proposal from the other side and select best suite
    ///
    /// This method is called by the server (or responder) to select the best
    /// suite from the client's proposal.
    ///
    /// # Arguments
    /// * `remote_proposals` - Suites proposed by the other side
    ///
    /// # Returns
    /// The selected suite, or error if no compatible suite found
    pub fn process_and_select(
        &mut self,
        remote_proposals: &[SuiteProposal],
    ) -> DlmsResult<SuiteId> {
        if self.state != NegotiationState::NotStarted
            && self.state != NegotiationState::ProposalReceived
        {
            return Err(DlmsError::Security(
                "Invalid negotiation state".to_string(),
            ));
        }

        // Find the best matching suite
        let selected = self.find_best_matching_suite(remote_proposals)?;

        if let Some(suite) = selected {
            self.state = NegotiationState::Completed;
            self.selected_suite = Some(suite);
            Ok(suite)
        } else {
            self.state = NegotiationState::Failed;
            Err(DlmsError::Security(
                "No compatible security suite found".to_string(),
            ))
        }
    }

    /// Receive and validate a selected suite from the other side
    ///
    /// This method is called by the client (or initiator) to receive and
    /// validate the server's selection.
    ///
    /// # Arguments
    /// * `selected_suite` - Suite selected by the other side
    ///
    /// # Returns
    /// Ok if the suite is acceptable, error otherwise
    pub fn receive_selection(&mut self, selected_suite: SuiteId) -> DlmsResult<()> {
        if self.state != NegotiationState::ProposalSent {
            return Err(DlmsError::Security(
                "Not expecting selection at this state".to_string(),
            ));
        }

        // Verify we support this suite
        if !self.supported_suites.contains(&selected_suite) {
            self.state = NegotiationState::Failed;
            return Err(DlmsError::Security(format!(
                "Selected suite {:?} is not supported",
                selected_suite
            )));
        }

        self.state = NegotiationState::Completed;
        self.selected_suite = Some(selected_suite);
        self.retry_count = 0; // Reset retry count on success
        Ok(())
    }

    /// Handle negotiation failure
    pub fn handle_failure(&mut self, error: NegotiationError) {
        self.state = NegotiationState::Failed;

        match error {
            NegotiationError::Timeout | NegotiationError::MaxRetriesExceeded => {
                // May trigger fallback to lower security suite
            }
            _ => {}
        }
    }

    /// Find the best matching suite from proposals
    ///
    /// The selection algorithm:
    /// 1. Find suites supported by both sides
    /// 2. Score each suite by security level
    /// 3. Select the highest-scoring suite
    /// 4. Break ties by local preference order
    fn find_best_matching_suite(
        &self,
        remote_proposals: &[SuiteProposal],
    ) -> DlmsResult<Option<SuiteId>> {
        let remote_suites: Vec<SuiteId> = remote_proposals
            .iter()
            .map(|p| p.suite_id)
            .collect();

        // Find intersection of supported suites
        let mut matching_suites: Vec<SuiteId> = self
            .supported_suites
            .iter()
            .filter(|suite| remote_suites.contains(suite))
            .cloned()
            .collect();

        if matching_suites.is_empty() {
            return Ok(None);
        }

        // Sort by security score (descending), then by preference order
        matching_suites.sort_by(|a, b| {
            // First, compare by security score
            let score_cmp = b.security_score().cmp(&a.security_score());
            if score_cmp != std::cmp::Ordering::Equal {
                return score_cmp;
            }

            // Tie-breaker: use local preference order
            let a_idx = self
                .preferred_suites
                .iter()
                .position(|s| s == a)
                .unwrap_or(usize::MAX);
            let b_idx = self
                .preferred_suites
                .iter()
                .position(|s| s == b)
                .unwrap_or(usize::MAX);

            a_idx.cmp(&b_idx) // Lower index = higher preference
        });

        Ok(matching_suites.into_iter().next())
    }

    /// Reset the negotiator to initial state
    pub fn reset(&mut self) {
        self.state = NegotiationState::NotStarted;
        self.retry_count = 0;
        self.selected_suite = None;
    }

    /// Get supported suites
    pub fn supported_suites(&self) -> &HashSet<SuiteId> {
        &self.supported_suites
    }

    /// Check if a specific suite is supported
    pub fn supports_suite(&self, suite_id: &SuiteId) -> bool {
        self.supported_suites.contains(suite_id)
    }

    /// Get timeout configuration
    pub fn timeout_config(&self) -> &NegotiationTimeout {
        &self.timeout_config
    }

    /// Set timeout configuration
    pub fn set_timeout_config(&mut self, config: NegotiationTimeout) {
        self.timeout_config = config;
    }
}

impl Default for SecuritySuiteNegotiator {
    fn default() -> Self {
        // Default: support only suite 0 (no security)
        Self::new(vec![SuiteId::new(
            EncryptionMechanism::None,
            AuthenticationMechanism::None,
        )])
    }
}

/// Helper function to create a suite ID from mechanism IDs
pub fn suite_id_from_ids(
    encryption_id: i32,
    authentication_id: i32,
) -> DlmsResult<SuiteId> {
    let encryption = EncryptionMechanism::from_id(encryption_id)?;
    let authentication = AuthenticationMechanism::from_id(authentication_id)?;
    Ok(SuiteId::new(encryption, authentication))
}

/// Convert suite ID to mechanism IDs for transmission
pub fn suite_id_to_ids(suite_id: SuiteId) -> (i32, i32) {
    (suite_id.encryption.id(), suite_id.authentication.id())
}

/// Create common security suites used in DLMS/COSEM
///
/// Returns suites in order of decreasing security:
/// 1. Suite 12: AES-GCM + HLS5-GMAC (highest security)
/// 2. Suite 2: HLS5-GMAC only
/// 3. Suite 1: Low-level authentication
/// 4. Suite 0: No security (lowest)
pub fn create_common_suites() -> Vec<SuiteId> {
    vec![
        SuiteId::new(
            EncryptionMechanism::AesGcm128,
            AuthenticationMechanism::Hls5Gmac,
        ),
        SuiteId::new(
            EncryptionMechanism::None,
            AuthenticationMechanism::Hls5Gmac,
        ),
        SuiteId::new(
            EncryptionMechanism::None,
            AuthenticationMechanism::Low,
        ),
        SuiteId::new(EncryptionMechanism::None, AuthenticationMechanism::None),
    ]
}

/// Create suites with encryption only (no authentication)
pub fn create_encryption_only_suites() -> Vec<SuiteId> {
    vec![
        SuiteId::new(
            EncryptionMechanism::AesGcm128,
            AuthenticationMechanism::None,
        ),
        SuiteId::new(EncryptionMechanism::None, AuthenticationMechanism::None),
    ]
}

/// Create suites with authentication only (no encryption)
pub fn create_authentication_only_suites() -> Vec<SuiteId> {
    vec![
        SuiteId::new(
            EncryptionMechanism::None,
            AuthenticationMechanism::Hls5Gmac,
        ),
        SuiteId::new(
            EncryptionMechanism::None,
            AuthenticationMechanism::Low,
        ),
        SuiteId::new(EncryptionMechanism::None, AuthenticationMechanism::None),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_all_test_suites() -> Vec<SuiteId> {
        vec![
            SuiteId::new(EncryptionMechanism::None, AuthenticationMechanism::None),
            SuiteId::new(
                EncryptionMechanism::None,
                AuthenticationMechanism::Low,
            ),
            SuiteId::new(
                EncryptionMechanism::None,
                AuthenticationMechanism::Hls5Gmac,
            ),
            SuiteId::new(
                EncryptionMechanism::AesGcm128,
                AuthenticationMechanism::None,
            ),
            SuiteId::new(
                EncryptionMechanism::AesGcm128,
                AuthenticationMechanism::Hls5Gmac,
            ),
        ]
    }

    #[test]
    fn test_suite_id_security_score() {
        let no_security = SuiteId::new(EncryptionMechanism::None, AuthenticationMechanism::None);
        assert_eq!(no_security.security_score(), 0);

        let auth_only = SuiteId::new(EncryptionMechanism::None, AuthenticationMechanism::Low);
        assert_eq!(auth_only.security_score(), 1);

        let enc_only = SuiteId::new(
            EncryptionMechanism::AesGcm128,
            AuthenticationMechanism::None,
        );
        assert_eq!(enc_only.security_score(), 2);

        let auth_and_enc = SuiteId::new(
            EncryptionMechanism::AesGcm128,
            AuthenticationMechanism::Hls5Gmac,
        );
        assert_eq!(auth_and_enc.security_score(), 3);
    }

    #[test]
    fn test_suite_id_comparison() {
        let low = SuiteId::new(EncryptionMechanism::None, AuthenticationMechanism::Low);
        let high = SuiteId::new(
            EncryptionMechanism::AesGcm128,
            AuthenticationMechanism::Hls5Gmac,
        );

        assert!(high.is_more_secure_than(&low));
        assert!(!low.is_more_secure_than(&high));
    }

    #[test]
    fn test_suite_id_default_policy() {
        let auth_enc = SuiteId::new(
            EncryptionMechanism::AesGcm128,
            AuthenticationMechanism::Hls5Gmac,
        );
        assert_eq!(
            auth_enc.default_policy(),
            SecurityPolicy::AuthenticatedAndEncrypted
        );

        let enc_only = SuiteId::new(
            EncryptionMechanism::AesGcm128,
            AuthenticationMechanism::None,
        );
        assert_eq!(enc_only.default_policy(), SecurityPolicy::Encrypted);

        let auth_only = SuiteId::new(
            EncryptionMechanism::None,
            AuthenticationMechanism::Hls5Gmac,
        );
        assert_eq!(auth_only.default_policy(), SecurityPolicy::Authenticated);

        let none = SuiteId::new(EncryptionMechanism::None, AuthenticationMechanism::None);
        assert_eq!(none.default_policy(), SecurityPolicy::Nothing);
    }

    #[test]
    fn test_negotiator_creation() {
        let suites = create_all_test_suites();
        let negotiator = SecuritySuiteNegotiator::new(suites.clone());

        assert_eq!(negotiator.state(), NegotiationState::NotStarted);
        assert_eq!(negotiator.supported_suites().len(), suites.len());
        assert!(!negotiator.is_complete());
        assert!(!negotiator.is_failed());
    }

    #[test]
    fn test_generate_proposal() {
        let suites = create_all_test_suites();
        let mut negotiator = SecuritySuiteNegotiator::new(suites);

        let proposal = negotiator.generate_proposal().unwrap();
        assert_eq!(proposal.len(), 5);
        assert_eq!(negotiator.state(), NegotiationState::ProposalSent);
    }

    #[test]
    fn test_generate_proposal_twice_fails() {
        let suites = create_all_test_suites();
        let mut negotiator = SecuritySuiteNegotiator::new(suites);

        negotiator.generate_proposal().unwrap();
        assert!(negotiator.generate_proposal().is_err());
    }

    #[test]
    fn test_successful_negotiation_full_flow() {
        // Both sides support same suites
        let suites = create_all_test_suites();

        let mut client = SecuritySuiteNegotiator::with_preferences(
            suites.clone(),
            suites.clone(),
        );
        let mut server = SecuritySuiteNegotiator::new(suites);

        // Client generates proposal
        let client_proposal = client.generate_proposal().unwrap();
        assert_eq!(client.state(), NegotiationState::ProposalSent);

        // Server selects best suite (should be auth+enc)
        let selected = server.process_and_select(&client_proposal).unwrap();

        // Should select the most secure suite
        assert_eq!(
            selected.encryption,
            EncryptionMechanism::AesGcm128
        );
        assert_eq!(
            selected.authentication,
            AuthenticationMechanism::Hls5Gmac
        );

        // Client accepts the selection
        client.receive_selection(selected).unwrap();
        assert!(client.is_complete());
        assert!(server.is_complete());
    }

    #[test]
    fn test_partial_suite_overlap() {
        // Client supports only no security and auth
        let client_suites = vec![
            SuiteId::new(EncryptionMechanism::None, AuthenticationMechanism::None),
            SuiteId::new(
                EncryptionMechanism::None,
                AuthenticationMechanism::Low,
            ),
        ];

        // Server supports auth and encryption
        let server_suites = vec![
            SuiteId::new(
                EncryptionMechanism::None,
                AuthenticationMechanism::Low,
            ),
            SuiteId::new(
                EncryptionMechanism::AesGcm128,
                AuthenticationMechanism::None,
            ),
        ];

        let mut client = SecuritySuiteNegotiator::new(client_suites);
        let mut server = SecuritySuiteNegotiator::new(server_suites);

        let client_proposal = client.generate_proposal().unwrap();
        let selected = server.process_and_select(&client_proposal).unwrap();

        // Should select low-level auth (only common suite)
        assert_eq!(selected.encryption, EncryptionMechanism::None);
        assert_eq!(selected.authentication, AuthenticationMechanism::Low);
    }

    #[test]
    fn test_no_compatible_suite() {
        // Client supports only encryption
        let client_suites = vec![SuiteId::new(
            EncryptionMechanism::AesGcm128,
            AuthenticationMechanism::None,
        )];

        // Server supports only auth
        let server_suites = vec![SuiteId::new(
            EncryptionMechanism::None,
            AuthenticationMechanism::Low,
        )];

        let mut client = SecuritySuiteNegotiator::new(client_suites);
        let mut server = SecuritySuiteNegotiator::new(server_suites);

        let client_proposal = client.generate_proposal().unwrap();
        let result = server.process_and_select(&client_proposal);

        assert!(result.is_err());
        assert!(server.is_failed());
    }

    #[test]
    fn test_suite_preference_order() {
        // Client prefers auth-only over encryption (unusual preference)
        let client_suites = vec![
            SuiteId::new(
                EncryptionMechanism::None,
                AuthenticationMechanism::Hls5Gmac,
            ),
            SuiteId::new(
                EncryptionMechanism::AesGcm128,
                AuthenticationMechanism::None,
            ),
            SuiteId::new(EncryptionMechanism::None, AuthenticationMechanism::None),
        ];

        let client_preferred = vec![
            SuiteId::new(
                EncryptionMechanism::None,
                AuthenticationMechanism::Hls5Gmac,
            ), // Prefer auth
            SuiteId::new(
                EncryptionMechanism::AesGcm128,
                AuthenticationMechanism::None,
            ), // Then encryption
        ];

        let mut client =
            SecuritySuiteNegotiator::with_preferences(client_suites, client_preferred);
        let server_suites = create_all_test_suites();
        let mut server = SecuritySuiteNegotiator::new(server_suites);

        let client_proposal = client.generate_proposal().unwrap();
        let selected = server.process_and_select(&client_proposal).unwrap();

        // Server should select encryption-only (score 2) since:
        // - Client doesn't support auth+enc together (score 3)
        // - Common suites are: auth-only (score 1), encryption-only (score 2), none (score 0)
        // - Encryption-only (score 2) is more secure than auth-only (score 1)
        assert_eq!(
            selected.encryption,
            EncryptionMechanism::AesGcm128
        );
        assert_eq!(
            selected.authentication,
            AuthenticationMechanism::None
        );
    }

    #[test]
    fn test_receive_selection_invalid_state() {
        let suites = create_all_test_suites();
        let mut negotiator = SecuritySuiteNegotiator::new(suites);

        let suite = SuiteId::new(EncryptionMechanism::None, AuthenticationMechanism::None);
        // Should fail - haven't sent proposal yet
        assert!(negotiator.receive_selection(suite).is_err());
    }

    #[test]
    fn test_receive_selection_unsupported_suite() {
        let client_suites = vec![SuiteId::new(
            EncryptionMechanism::None,
            AuthenticationMechanism::None,
        )];

        let mut client = SecuritySuiteNegotiator::new(client_suites);
        client.generate_proposal().unwrap();

        // Server tries to select a suite we don't support
        let unsupported = SuiteId::new(
            EncryptionMechanism::AesGcm128,
            AuthenticationMechanism::Hls5Gmac,
        );

        assert!(client.receive_selection(unsupported).is_err());
        assert!(client.is_failed());
    }

    #[test]
    fn test_retry_counting() {
        let suites = create_all_test_suites();
        let mut negotiator = SecuritySuiteNegotiator::new(suites);

        assert_eq!(negotiator.retry_count(), 0);

        negotiator.increment_retry();
        assert_eq!(negotiator.retry_count(), 1);

        negotiator.increment_retry();
        assert_eq!(negotiator.retry_count(), 2);
    }

    #[test]
    fn test_max_retries_check() {
        let timeout = NegotiationTimeout {
            response_timeout: Duration::from_secs(30),
            max_retries: 2,
            retry_delay: Duration::from_secs(5),
        };

        let suites = create_all_test_suites();
        let mut negotiator = SecuritySuiteNegotiator::with_timeout(suites, timeout);

        assert!(!negotiator.is_max_retries_exceeded());

        negotiator.increment_retry();
        assert!(!negotiator.is_max_retries_exceeded());

        negotiator.increment_retry();
        assert!(negotiator.is_max_retries_exceeded());
    }

    #[test]
    fn test_reset() {
        let suites = create_all_test_suites();
        let mut negotiator = SecuritySuiteNegotiator::new(suites);

        // Do some operations
        negotiator.generate_proposal().unwrap();
        negotiator.increment_retry();

        // Reset
        negotiator.reset();

        assert_eq!(negotiator.state(), NegotiationState::NotStarted);
        assert_eq!(negotiator.retry_count(), 0);
        assert!(negotiator.selected_suite().is_none());
    }

    #[test]
    fn test_suite_id_to_ids() {
        let suite_id = SuiteId::new(
            EncryptionMechanism::AesGcm128,
            AuthenticationMechanism::Hls5Gmac,
        );

        let (enc_id, auth_id) = suite_id_to_ids(suite_id);
        assert_eq!(enc_id, 0); // AesGcm128
        assert_eq!(auth_id, 5); // Hls5Gmac
    }

    #[test]
    fn test_suite_id_from_ids() {
        let suite_id = suite_id_from_ids(0, 5).unwrap();
        assert_eq!(suite_id.encryption, EncryptionMechanism::AesGcm128);
        assert_eq!(suite_id.authentication, AuthenticationMechanism::Hls5Gmac);
    }

    #[test]
    fn test_invalid_suite_id_from_ids() {
        assert!(suite_id_from_ids(99, 5).is_err());
        assert!(suite_id_from_ids(0, 99).is_err());
    }

    #[test]
    fn test_create_common_suites() {
        let suites = create_common_suites();
        assert_eq!(suites.len(), 4);

        // First should be most secure
        assert_eq!(suites[0].security_score(), 3);

        // Last should be least secure
        assert_eq!(suites[3].security_score(), 0);
    }

    #[test]
    fn test_create_encryption_only_suites() {
        let suites = create_encryption_only_suites();
        assert_eq!(suites.len(), 2);

        for suite in &suites {
            assert_eq!(suite.authentication, AuthenticationMechanism::None);
        }
    }

    #[test]
    fn test_create_authentication_only_suites() {
        let suites = create_authentication_only_suites();
        assert_eq!(suites.len(), 3);

        for suite in &suites {
            assert_eq!(suite.encryption, EncryptionMechanism::None);
        }
    }

    #[test]
    fn test_suite_proposal_creation() {
        let proposal = SuiteProposal::new(
            EncryptionMechanism::AesGcm128,
            AuthenticationMechanism::Hls5Gmac,
        );

        assert_eq!(
            proposal.suite_id.encryption,
            EncryptionMechanism::AesGcm128
        );
        assert_eq!(
            proposal.suite_id.authentication,
            AuthenticationMechanism::Hls5Gmac
        );
        assert!(proposal.supports_policy(SecurityPolicy::AuthenticatedAndEncrypted));
    }

    #[test]
    fn test_suite_proposal_with_policy() {
        let proposal = SuiteProposal::new(
            EncryptionMechanism::None,
            AuthenticationMechanism::Low,
        )
        .with_policy(SecurityPolicy::Authenticated);

        assert!(proposal.policies.contains(&SecurityPolicy::Authenticated));
        assert!(proposal.supports_policy(SecurityPolicy::Authenticated));
    }

    #[test]
    fn test_suite_id_to_security_suite() {
        let suite_id = SuiteId::new(
            EncryptionMechanism::AesGcm128,
            AuthenticationMechanism::Hls5Gmac,
        );

        let security_suite: SecuritySuite = suite_id.into();

        assert_eq!(
            security_suite.encryption_mechanism(),
            EncryptionMechanism::AesGcm128
        );
        assert_eq!(
            security_suite.authentication_mechanism(),
            AuthenticationMechanism::Hls5Gmac
        );
        assert_eq!(
            security_suite.security_policy(),
            SecurityPolicy::AuthenticatedAndEncrypted
        );
    }

    #[test]
    fn test_state_is_terminal() {
        assert!(!NegotiationState::NotStarted.is_terminal());
        assert!(!NegotiationState::ProposalSent.is_terminal());
        assert!(!NegotiationState::ProposalReceived.is_terminal());
        assert!(NegotiationState::Completed.is_terminal());
        assert!(NegotiationState::Failed.is_terminal());
    }

    #[test]
    fn test_timeout_config_default() {
        let config = NegotiationTimeout::default();
        assert_eq!(config.response_timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay, Duration::from_secs(5));
    }

    #[test]
    fn test_get_set_timeout_config() {
        let suites = create_common_suites();
        let mut negotiator = SecuritySuiteNegotiator::new(suites);

        let new_config = NegotiationTimeout {
            response_timeout: Duration::from_secs(60),
            max_retries: 5,
            retry_delay: Duration::from_secs(10),
        };

        negotiator.set_timeout_config(new_config.clone());
        assert_eq!(negotiator.timeout_config().response_timeout, Duration::from_secs(60));
        assert_eq!(negotiator.timeout_config().max_retries, 5);
    }

    #[test]
    fn test_supports_suite_check() {
        let suites = vec![
            SuiteId::new(
                EncryptionMechanism::AesGcm128,
                AuthenticationMechanism::Hls5Gmac,
            ),
            SuiteId::new(EncryptionMechanism::None, AuthenticationMechanism::None),
        ];

        let negotiator = SecuritySuiteNegotiator::new(suites);

        assert!(negotiator.supports_suite(&SuiteId::new(
            EncryptionMechanism::AesGcm128,
            AuthenticationMechanism::Hls5Gmac,
        )));

        assert!(!negotiator.supports_suite(&SuiteId::new(
            EncryptionMechanism::None,
            AuthenticationMechanism::Low,
        )));
    }

    #[test]
    fn test_negotiation_state_transitions() {
        let suites = create_all_test_suites();

        // Client flow
        let mut client = SecuritySuiteNegotiator::new(suites.clone());
        assert_eq!(client.state(), NegotiationState::NotStarted);

        client.generate_proposal().unwrap();
        assert_eq!(client.state(), NegotiationState::ProposalSent);

        let selected = SuiteId::new(
            EncryptionMechanism::AesGcm128,
            AuthenticationMechanism::Hls5Gmac,
        );
        client.receive_selection(selected).unwrap();
        assert_eq!(client.state(), NegotiationState::Completed);

        // Server flow
        let mut server = SecuritySuiteNegotiator::new(suites);
        let client_proposal = vec![SuiteProposal::new(
            EncryptionMechanism::None,
            AuthenticationMechanism::None,
        )];

        server.process_and_select(&client_proposal).unwrap();
        assert_eq!(server.state(), NegotiationState::Completed);
    }
}