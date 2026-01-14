//! Association events for DLMS/COSEM connections
//!
//! This module defines events that can occur during the lifetime of an
//! association, following the COSEM-ABORT.indication and other events.

use crate::association::state::{AbortReason, RejectReason};

/// An event that can occur on an association
///
/// Events are notifications about state changes or external conditions
/// that affect the association.
#[derive(Debug, Clone, PartialEq)]
pub enum AssociationEvent {
    /// Association successfully established
    ///
    /// This event is generated when a COSEM-OPEN.confirm indicates success.
    Established {
        /// The negotiated DLMS version
        version: u8,
    },

    /// Association establishment failed
    ///
    /// This event is generated when a COSEM-OPEN.confirm indicates failure.
    EstablishmentFailed {
        /// Reason for rejection
        reason: RejectReason,
        /// Diagnostic information
        diagnostic: Option<String>,
    },

    /// Association released normally
    ///
    /// This event is generated when a COSEM-RELEASE.confirm indicates success.
    Released,

    /// Association release failed
    ///
    /// This event is generated when a COSEM-RELEASE.confirm indicates failure,
    /// but the association remains active.
    ReleaseFailed {
        /// Error description
        error: String,
    },

    /// Association aborted
    ///
    /// This corresponds to COSEM-ABORT.indication, which can occur
    /// when the physical connection is lost or aborted.
    Aborted {
        /// Abort reason
        reason: AbortReason,
    },

    /// Connection lost
    ///
    /// This event is generated when the physical connection (HDLC/Wrapper)
    /// is unexpectedly lost.
    ConnectionLost,

    /// Authentication failure
    ///
    /// This event is generated when authentication fails during
    /// association establishment.
    AuthenticationFailed {
        /// Failure details
        details: String,
    },

    /// Protocol error
    ///
    /// This event is generated when a protocol error occurs that
    /// affects the association.
    ProtocolError {
        /// Error description
        error: String,
    },
}

impl AssociationEvent {
    /// Check if the event is a successful establishment event
    #[must_use]
    pub fn is_established(&self) -> bool {
        matches!(self, Self::Established { .. })
    }

    /// Check if the event is a release event (successful or failed)
    #[must_use]
    pub fn is_release_event(&self) -> bool {
        matches!(self, Self::Released | Self::ReleaseFailed { .. })
    }

    /// Check if the event is an abort or termination event
    #[must_use]
    pub fn is_termination(&self) -> bool {
        matches!(
            self,
            Self::Aborted { .. } | Self::ConnectionLost | Self::Released
        )
    }

    /// Check if the event indicates a failure
    #[must_use]
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            Self::EstablishmentFailed { .. }
                | Self::ReleaseFailed { .. }
                | Self::Aborted { .. }
                | Self::ConnectionLost
                | Self::AuthenticationFailed { .. }
                | Self::ProtocolError { .. }
        )
    }

    /// Get a human-readable description of the event
    pub fn description(&self) -> String {
        match self {
            Self::Established { version } => {
                format!("Association established (DLMS version {})", version)
            }
            Self::EstablishmentFailed { reason, diagnostic } => {
                format!(
                    "Association establishment failed: {:?}{:?}",
                    reason,
                    diagnostic.as_ref().map(|d| format!(" - {}", d)).unwrap_or_default()
                )
            }
            Self::Released => "Association released".to_string(),
            Self::ReleaseFailed { error } => {
                format!("Association release failed: {}", error)
            }
            Self::Aborted { reason } => {
                format!("Association aborted: {:?}", reason)
            }
            Self::ConnectionLost => "Connection lost".to_string(),
            Self::AuthenticationFailed { details } => {
                format!("Authentication failed: {}", details)
            }
            Self::ProtocolError { error } => {
                format!("Protocol error: {}", error)
            }
        }
    }
}

/// Event listener for association events
///
/// Implement this trait to receive notifications about association events.
pub trait AssociationEventListener: Send + Sync {
    /// Called when an association event occurs
    ///
    /// # Arguments
    /// * `event` - The event that occurred
    fn on_event(&self, event: AssociationEvent);
}

/// Callback-based event listener
///
/// A simple implementation of AssociationEventListener that uses a callback function.
pub struct CallbackEventListener<F>
where
    F: Fn(AssociationEvent) + Send + Sync,
{
    callback: F,
}

impl<F> CallbackEventListener<F>
where
    F: Fn(AssociationEvent) + Send + Sync,
{
    /// Create a new callback-based event listener
    ///
    /// # Arguments
    /// * `callback` - Function to call when an event occurs
    #[must_use]
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}

impl<F> AssociationEventListener for CallbackEventListener<F>
where
    F: Fn(AssociationEvent) + Send + Sync,
{
    fn on_event(&self, event: AssociationEvent) {
        (self.callback)(event);
    }
}

/// Channel-based event listener
///
/// A listener that sends events to a tokio mpsc channel.
#[cfg(feature = "tokio")]
pub mod channel_listener {
    use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver};

    /// Channel-based event listener
    pub struct ChannelEventListener {
        tx: UnboundedSender<super::AssociationEvent>,
    }

    impl ChannelEventListener {
        /// Create a new channel-based event listener
        ///
        /// # Returns
        /// Returns the listener and a receiver for events.
        pub fn new() -> (Self, UnboundedReceiver<super::AssociationEvent>) {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            (Self { tx }, rx)
        }

        /// Send an event to the channel
        pub fn send(&self, event: super::AssociationEvent) -> Result<(), super::super::super::dlms_core::DlmsError> {
            self.tx
                .send(event)
                .map_err(|e| super::super::super::dlms_core::DlmsError::Other(e.to_string()))
        }
    }

    impl super::AssociationEventListener for ChannelEventListener {
        fn on_event(&self, event: super::AssociationEvent) {
            // Ignore send errors - the receiver might be dropped
            let _ = self.tx.send(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_types() {
        let event = AssociationEvent::Established { version: 6 };
        assert!(event.is_established());
        assert!(!event.is_failure());

        let event = AssociationEvent::Released;
        assert!(event.is_release_event());
        assert!(!event.is_failure());

        let event = AssociationEvent::Aborted {
            reason: AbortReason::Other(1),
        };
        assert!(event.is_termination());
        assert!(event.is_failure());
    }

    #[test]
    fn test_event_description() {
        let event = AssociationEvent::Established { version: 6 };
        assert!(event.description().contains("Association established"));
    }

    #[test]
    fn test_callback_listener() {
        let mut last_event = None;
        let listener = CallbackEventListener::new(|event| {
            last_event = Some(event);
        });

        let event = AssociationEvent::Released;
        listener.on_event(event);

        assert!(last_event.is_some());
        assert_eq!(last_event.unwrap(), AssociationEvent::Released);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_channel_listener() {
        use crate::association::events::channel_listener::ChannelEventListener;

        let (listener, mut rx) = ChannelEventListener::new();

        let event = AssociationEvent::Released;
        listener.on_event(event).unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received, AssociationEvent::Released);
    }
}
