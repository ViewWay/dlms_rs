//! Association state machine for DLMS/COSEM connections
//!
//! This module defines the states of an application layer association (connection)
//! between a DLMS client and server, following the COSEM-OPEN and COSEM-RELEASE
//! service primitives defined in the DLMS Green Book.

use std::fmt::{self, Display};

/// Association state for DLMS/COSEM application layer connections
///
/// The association state machine follows the COSEM-OPEN and COSEM-RELEASE
/// service primitives defined in DLMS/COSEM Green Book Edition 9.
///
/// # State Transitions
///
/// ```text
///     COSEM-OPEN.request           COSEM-RELEASE.request
///    (AARQ sent)                   (RLRQ sent)
///    -----------                   ------------
///   |             |               |             |
///   v             v               v             v
/// Inactive -> Idle -> AssociationPending -> Associated -> ReleasePending -> Inactive
///                                    |
///                                    | COSEM-ABORT.indication
///                                    v
///                                 Inactive
/// ```
///
/// # State Descriptions
///
/// - **Inactive**: No association exists, physical connection not established
/// - **Idle**: Physical connection established (HDLC/Wrapper ready), but no application association
/// - **AssociationPending**: AARQ sent, waiting for AARE (association in progress)
/// - **Associated**: Application association established, normal operation
/// - **ReleasePending**: RLRQ sent, waiting for RLRE (release in progress)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssociationState {
    /// No association exists
    ///
    /// This is the initial state before any connection is established,
    /// or after a connection is fully closed.
    Inactive,

    /// Physical connection established, but no application association
    ///
    /// In this state:
    /// - HDLC connection is established (SNRM/UA handshake complete)
    /// - Wrapper connection is established (if using UDP/TCP)
    /// - Application association (AARQ/AARE) has not been performed yet
    Idle,

    /// Association establishment in progress
    ///
    /// This state is entered when:
    /// - COSEM-OPEN.request is issued (AARQ sent)
    /// - Waiting for COSEM-OPEN.confirm (AARE response)
    ///
    /// The state transitions to:
    /// - `Associated` if AARE indicates success
    /// - `Idle` if AARE indicates rejection
    /// - `Inactive` if connection is lost
    AssociationPending,

    /// Application association established
    ///
    /// This is the normal operating state where DLMS operations
    /// (GET, SET, ACTION) can be performed.
    ///
    /// In this state:
    /// - AARQ/AARE handshake complete
    /// - Security context established (if encryption enabled)
    /// - Protocol parameters negotiated (conformance, PDU size, version)
    /// - Normal DLMS operations can proceed
    Associated,

    /// Association release in progress
    ///
    /// This state is entered when:
    /// - COSEM-RELEASE.request is issued (RLRQ sent)
    /// - Waiting for COSEM-RELEASE.confirm (RLRE response)
    ///
    /// The state transitions to:
    /// - `Inactive` after successful release
    /// - `Associated` if release is rejected by server
    ReleasePending,
}

impl AssociationState {
    /// Check if the association is in an active state (can perform operations)
    ///
    /// Returns true if the state is `Associated`, meaning normal DLMS
    /// operations can be performed.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Associated)
    }

    /// Check if the association is in a transitional state
    ///
    /// Returns true if the state is `AssociationPending` or `ReleasePending`,
    /// meaning an operation is in progress.
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        matches!(self, Self::AssociationPending | Self::ReleasePending)
    }

    /// Check if the connection is established at physical layer
    ///
    /// Returns true if the state is `Idle` or any state beyond (meaning
    /// HDLC/Wrapper connection is ready).
    #[must_use]
    pub const fn is_connected(&self) -> bool {
        matches!(self, Self::Idle | Self::AssociationPending | Self::Associated | Self::ReleasePending)
    }

    /// Get the next state after a successful AARQ/AARE exchange
    #[must_use]
    pub const fn after_association_success(&self) -> Self {
        match self {
            Self::Idle => Self::Associated,
            _ => *self,
        }
    }

    /// Get the next state after association failure
    #[must_use]
    pub const fn after_association_failure(&self) -> Self {
        match self {
            Self::AssociationPending => Self::Idle,
            _ => *self,
        }
    }

    /// Get the next state after a successful RLRQ/RLRE exchange
    #[must_use]
    pub const fn after_release_success(&self) -> Self {
        match self {
            Self::Associated => Self::Inactive,
            Self::ReleasePending => Self::Inactive,
            _ => *self,
        }
    }
}

impl Default for AssociationState {
    fn default() -> Self {
        Self::Inactive
    }
}

impl Display for AssociationState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inactive => write!(f, "Inactive"),
            Self::Idle => write!(f, "Idle"),
            Self::AssociationPending => write!(f, "AssociationPending"),
            Self::Associated => write!(f, "Associated"),
            Self::ReleasePending => write!(f, "ReleasePending"),
        }
    }
}

/// Result of COSEM-OPEN.request
///
/// This is returned when `Association::open()` is called, representing
/// the COSEM-OPEN.confirm primitive.
#[derive(Debug, Clone, PartialEq)]
pub enum OpenResult {
    /// Association successfully established
    Success {
        /// Negotiated protocol version
        version: u8,
        /// Negotiated conformance bits
        conformance: u32,
        /// Server maximum PDU size
        max_pdu_size: u16,
    },

    /// Association rejected by server
    Rejected {
        /// Reason for rejection
        reason: RejectReason,
        /// Diagnostic information from server
        diagnostic: Option<String>,
    },

    /// Association failed due to local error
    Failed {
        /// Error description
        error: String,
    },

    /// Association aborted
    Aborted {
        /// Abort reason
        reason: AbortReason,
    },
}

/// Reason for association rejection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    /// No reason given
    None,
    /// Sender is not authorized to establish an association
    NotAuthorized,
    /// Called AP title not recognized
    CalledApTitleNotRecognized,
    /// Called AP invocation identifier not recognized
    CalledApInvocationIdNotRecognized,
    /// Caller not authorized to invoke this operation
    NotAuthorizedToInvoke,
    /// Other reason
    Other(u8),
}

/// Reason for association abort
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbortReason {
    /// No reason given
    None,
    /// Authentication failure
    AuthenticationFailed,
    /// Authentication required
    AuthenticationRequired,
    /// Cipher suite not supported
    CipherSuiteNotSupported,
    /// Other reason
    Other(u8),
}

/// Result of COSEM-RELEASE.request
///
/// This is returned when `Association::release()` is called, representing
/// the COSEM-RELEASE.confirm primitive.
#[derive(Debug, Clone, PartialEq)]
pub enum ReleaseResult {
    /// Association successfully released
    Success,

    /// Release rejected by server
    Rejected {
        /// Reason for rejection
        reason: ReleaseRejectReason,
    },

    /// Release failed due to local error
    Failed {
        /// Error description
        error: String,
    },
}

/// Reason for release rejection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseRejectReason {
    /// No reason given
    None,
    /// Not associated (no association to release)
    NotAssociated,
    /// Other reason
    Other(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        assert!(!AssociationState::Inactive.is_connected());
        assert!(!AssociationState::Inactive.is_active());

        assert!(AssociationState::Idle.is_connected());
        assert!(!AssociationState::Idle.is_active());

        assert!(AssociationState::Associated.is_active());
        assert!(AssociationState::Associated.is_connected());
    }

    #[test]
    fn test_state_after_association() {
        let state = AssociationState::Idle;
        let next = state.after_association_success();
        assert_eq!(next, AssociationState::Associated);
    }

    #[test]
    fn test_state_after_release() {
        let state = AssociationState::Associated;
        let next = state.after_release_success();
        assert_eq!(next, AssociationState::Inactive);
    }

    #[test]
    fn test_display_state() {
        assert_eq!(AssociationState::Inactive.to_string(), "Inactive");
        assert_eq!(AssociationState::Idle.to_string(), "Idle");
        assert_eq!(AssociationState::Associated.to_string(), "Associated");
    }
}
