//! HDLC connection state machine

use crate::error::{DlmsError, DlmsResult};

/// HDLC connection state
///
/// Tracks the current state of an HDLC connection to ensure operations
/// are only performed when the connection is in the correct state.
///
/// # State Transitions
/// ```
/// Closed -> Connecting (on open())
/// Connecting -> Connected (on UA received)
/// Connected -> Closing (on close())
/// Closing -> Closed (on DM/UA received or timeout)
/// ```
///
/// # Why State Machine?
/// Using explicit states provides:
/// - **Clear State Transitions**: Easy to understand connection lifecycle
/// - **Error Prevention**: Operations can validate state before execution
/// - **Debugging**: State can be logged and inspected
/// - **Testing**: States can be verified in tests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdlcConnectionState {
    /// Connection is closed (initial state)
    ///
    /// In this state:
    /// - Transport layer is closed
    /// - No frames can be sent (except SNRM for connection establishment)
    /// - No frames are expected
    Closed,
    /// Connection is being established
    ///
    /// In this state:
    /// - Transport layer is open
    /// - SNRM frame has been sent
    /// - Waiting for UA frame
    /// - Control frames (SNRM) can be sent
    /// - Information frames cannot be sent yet
    Connecting,
    /// Connection is established and ready
    ///
    /// In this state:
    /// - Transport and session layers are open
    /// - UA frame has been received
    /// - All frame types can be sent
    /// - Frames can be received
    Connected,
    /// Connection is being closed
    ///
    /// In this state:
    /// - DISC frame has been sent
    /// - Waiting for DM or UA frame
    /// - No new frames should be sent
    /// - Transport will be closed after response or timeout
    Closing,
}

impl HdlcConnectionState {
    /// Check if the connection is ready for data transmission
    ///
    /// # Returns
    /// `true` if connection is in `Connected` state, `false` otherwise
    pub fn is_ready(&self) -> bool {
        matches!(self, HdlcConnectionState::Connected)
    }

    /// Check if the connection can send information frames
    ///
    /// # Returns
    /// `true` if information frames can be sent, `false` otherwise
    pub fn can_send_information(&self) -> bool {
        matches!(self, HdlcConnectionState::Connected)
    }

    /// Check if the connection can send control frames
    ///
    /// Control frames (SNRM, DISC) can be sent in more states than information frames.
    ///
    /// # Returns
    /// `true` if control frames can be sent, `false` otherwise
    pub fn can_send_control(&self) -> bool {
        matches!(
            self,
            HdlcConnectionState::Closed
                | HdlcConnectionState::Connecting
                | HdlcConnectionState::Connected
        )
    }

    /// Check if the connection can be closed
    ///
    /// # Returns
    /// `true` if connection can be closed, `false` otherwise
    pub fn can_close(&self) -> bool {
        !matches!(self, HdlcConnectionState::Closed)
    }

    /// Validate state transition
    ///
    /// # Arguments
    /// * `new_state` - The target state
    ///
    /// # Returns
    /// `Ok(())` if transition is valid, `Err` otherwise
    ///
    /// # Valid Transitions
    /// - `Closed` -> `Connecting` (on open)
    /// - `Connecting` -> `Connected` (on UA received)
    /// - `Connecting` -> `Closed` (on error/timeout)
    /// - `Connected` -> `Closing` (on close)
    /// - `Connected` -> `Closed` (on error)
    /// - `Closing` -> `Closed` (on DM/UA received or timeout)
    pub fn validate_transition(&self, new_state: HdlcConnectionState) -> DlmsResult<()> {
        let valid = match (*self, new_state) {
            // Normal transitions
            (HdlcConnectionState::Closed, HdlcConnectionState::Connecting) => true,
            (HdlcConnectionState::Connecting, HdlcConnectionState::Connected) => true,
            (HdlcConnectionState::Connecting, HdlcConnectionState::Closed) => true, // Error/timeout
            (HdlcConnectionState::Connected, HdlcConnectionState::Closing) => true,
            (HdlcConnectionState::Connected, HdlcConnectionState::Closed) => true, // Error
            (HdlcConnectionState::Closing, HdlcConnectionState::Closed) => true,
            // Self-transitions (idempotent operations)
            (HdlcConnectionState::Closed, HdlcConnectionState::Closed) => true,
            (HdlcConnectionState::Connected, HdlcConnectionState::Connected) => true,
            // Invalid transitions
            _ => false,
        };

        if valid {
            Ok(())
        } else {
            Err(DlmsError::InvalidData(format!(
                "Invalid state transition: {:?} -> {:?}",
                self, new_state
            )))
        }
    }

    /// Get human-readable state name
    pub fn as_str(&self) -> &'static str {
        match self {
            HdlcConnectionState::Closed => "Closed",
            HdlcConnectionState::Connecting => "Connecting",
            HdlcConnectionState::Connected => "Connected",
            HdlcConnectionState::Closing => "Closing",
        }
    }
}

impl Default for HdlcConnectionState {
    fn default() -> Self {
        HdlcConnectionState::Closed
    }
}
