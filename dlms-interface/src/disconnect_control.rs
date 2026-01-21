//! Disconnect Control interface class (Class ID: 70)
//!
//! The Disconnect Control interface class provides remote disconnect
//! and reconnect control for the meter's load switch.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: output_state - Current state of the output (true=connected)
//!
//! # Methods
//!
//! - Method 1: remote_disconnect() - Disconnect the load
//! - Method 2: remote_reconnect() - Reconnect the load
//!
//! # Disconnect Control (Class ID: 70)
//!
//! This class is used for smart metering remote disconnect/reconnect
//! functionality, allowing utilities to control service remotely.

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Output state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OutputState {
    /// Disconnected
    Disconnected = 0,
    /// Connected
    Connected = 1,
    /// Ready for reconnect
    ReadyForReconnect = 2,
    /// Disconnect in progress
    DisconnectInProgress = 3,
    /// Reconnect in progress
    ReconnectInProgress = 4,
}

impl OutputState {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Disconnected,
            1 => Self::Connected,
            2 => Self::ReadyForReconnect,
            3 => Self::DisconnectInProgress,
            4 => Self::ReconnectInProgress,
            _ => Self::Disconnected,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if connected
    pub fn is_connected(self) -> bool {
        matches!(self, Self::Connected)
    }

    /// Check if disconnected
    pub fn is_disconnected(self) -> bool {
        matches!(self, Self::Disconnected | Self::ReadyForReconnect)
    }

    /// Check if in transition
    pub fn is_in_transition(self) -> bool {
        matches!(self, Self::DisconnectInProgress | Self::ReconnectInProgress)
    }
}

/// Disconnect Control interface class (Class ID: 70)
///
/// Default OBIS: 0-0:96.1.0.255
///
/// This class provides remote disconnect/reconnect control for smart meters.
/// It's essential for prepaid metering and remote service management.
#[derive(Debug, Clone)]
pub struct DisconnectControl {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Current output state
    output_state: Arc<RwLock<OutputState>>,

    /// Whether disconnect is enabled (can be disabled for safety)
    disconnect_enabled: Arc<RwLock<bool>>,

    /// Whether reconnect is enabled
    reconnect_enabled: Arc<RwLock<bool>>,
}

impl DisconnectControl {
    /// Class ID for Disconnect Control
    pub const CLASS_ID: u16 = 70;

    /// Default OBIS code for Disconnect Control (0-0:96.1.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 96, 1, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_OUTPUT_STATE: u8 = 2;

    /// Method IDs
    pub const METHOD_REMOTE_DISCONNECT: u8 = 1;
    pub const METHOD_REMOTE_RECONNECT: u8 = 2;

    /// Create a new Disconnect Control object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `output_state` - Initial output state
    pub fn new(logical_name: ObisCode, output_state: OutputState) -> Self {
        Self {
            logical_name,
            output_state: Arc::new(RwLock::new(output_state)),
            disconnect_enabled: Arc::new(RwLock::new(true)),
            reconnect_enabled: Arc::new(RwLock::new(true)),
        }
    }

    /// Create with default OBIS code (connected state)
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis(), OutputState::Connected)
    }

    /// Get the current output state
    pub async fn output_state(&self) -> OutputState {
        *self.output_state.read().await
    }

    /// Set the output state directly
    pub async fn set_output_state(&self, state: OutputState) {
        *self.output_state.write().await = state;
    }

    /// Check if disconnect is enabled
    pub async fn is_disconnect_enabled(&self) -> bool {
        *self.disconnect_enabled.read().await
    }

    /// Enable or disable disconnect functionality
    pub async fn set_disconnect_enabled(&self, enabled: bool) {
        *self.disconnect_enabled.write().await = enabled;
    }

    /// Check if reconnect is enabled
    pub async fn is_reconnect_enabled(&self) -> bool {
        *self.reconnect_enabled.read().await
    }

    /// Enable or disable reconnect functionality
    pub async fn set_reconnect_enabled(&self, enabled: bool) {
        *self.reconnect_enabled.write().await = enabled;
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        self.output_state().await.is_connected()
    }

    /// Remote disconnect - disconnect the load
    ///
    /// This corresponds to Method 1
    pub async fn remote_disconnect(&self) -> DlmsResult<()> {
        if !self.is_disconnect_enabled().await {
            return Err(DlmsError::AccessDenied(
                "Remote disconnect is disabled".to_string(),
            ));
        }
        if self.output_state().await.is_disconnected() {
            return Err(DlmsError::InvalidData(
                "Already disconnected".to_string(),
            ));
        }
        self.set_output_state(OutputState::DisconnectInProgress).await;
        // In a real implementation, this would trigger the actual disconnect
        // For now, we simulate immediate completion
        self.set_output_state(OutputState::Disconnected).await;
        Ok(())
    }

    /// Remote reconnect - reconnect the load
    ///
    /// This corresponds to Method 2
    pub async fn remote_reconnect(&self) -> DlmsResult<()> {
        if !self.is_reconnect_enabled().await {
            return Err(DlmsError::AccessDenied(
                "Remote reconnect is disabled".to_string(),
            ));
        }
        if self.output_state().await.is_connected() {
            return Err(DlmsError::InvalidData("Already connected".to_string()));
        }
        self.set_output_state(OutputState::ReconnectInProgress).await;
        // In a real implementation, this would trigger the actual reconnect
        // For now, we simulate immediate completion
        self.set_output_state(OutputState::Connected).await;
        Ok(())
    }

    /// Get both enabled states
    pub async fn enabled_states(&self) -> (bool, bool) {
        (self.is_disconnect_enabled().await, self.is_reconnect_enabled().await)
    }
}

#[async_trait]
impl CosemObject for DisconnectControl {
    fn class_id(&self) -> u16 {
        Self::CLASS_ID
    }

    fn obis_code(&self) -> ObisCode {
        self.logical_name
    }

    async fn get_attribute(
        &self,
        attribute_id: u8,
        _selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<DataObject> {
        match attribute_id {
            Self::ATTR_LOGICAL_NAME => {
                Ok(DataObject::OctetString(self.logical_name.to_bytes().to_vec()))
            }
            Self::ATTR_OUTPUT_STATE => {
                Ok(DataObject::Enumerate(self.output_state().await.to_u8()))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Disconnect Control has no attribute {}",
                attribute_id
            ))),
        }
    }

    async fn set_attribute(
        &self,
        attribute_id: u8,
        value: DataObject,
        _selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<()> {
        match attribute_id {
            Self::ATTR_LOGICAL_NAME => {
                Err(DlmsError::AccessDenied(
                    "Attribute 1 (logical_name) is read-only".to_string(),
                ))
            }
            Self::ATTR_OUTPUT_STATE => {
                // Output state is controlled via methods, not direct attribute writes
                // However, we allow it for configuration purposes
                let state = match value {
                    DataObject::Enumerate(s) => OutputState::from_u8(s),
                    DataObject::Unsigned8(s) => OutputState::from_u8(s),
                    DataObject::Boolean(b) => {
                        if b {
                            OutputState::Connected
                        } else {
                            OutputState::Disconnected
                        }
                    }
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Enumerate, Unsigned8, or Boolean for output_state".to_string(),
                        ))
                    }
                };
                self.set_output_state(state).await;
                Ok(())
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Disconnect Control has no attribute {}",
                attribute_id
            ))),
        }
    }

    async fn invoke_method(
        &self,
        method_id: u8,
        _parameters: Option<DataObject>,
        _selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<Option<DataObject>> {
        match method_id {
            Self::METHOD_REMOTE_DISCONNECT => {
                self.remote_disconnect().await?;
                Ok(None)
            }
            Self::METHOD_REMOTE_RECONNECT => {
                self.remote_reconnect().await?;
                Ok(None)
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Disconnect Control has no method {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_disconnect_control_class_id() {
        let control = DisconnectControl::with_default_obis();
        assert_eq!(control.class_id(), 70);
    }

    #[tokio::test]
    async fn test_disconnect_control_obis_code() {
        let control = DisconnectControl::with_default_obis();
        assert_eq!(control.obis_code(), DisconnectControl::default_obis());
    }

    #[tokio::test]
    async fn test_disconnect_control_initial_state() {
        let control = DisconnectControl::with_default_obis();
        assert_eq!(control.output_state().await, OutputState::Connected);
        assert!(control.is_connected().await);
    }

    #[tokio::test]
    async fn test_disconnect_control_remote_disconnect() {
        let control = DisconnectControl::with_default_obis();
        control.remote_disconnect().await.unwrap();
        assert_eq!(control.output_state().await, OutputState::Disconnected);
        assert!(!control.is_connected().await);
    }

    #[tokio::test]
    async fn test_disconnect_control_remote_reconnect() {
        let control = DisconnectControl::new(
            DisconnectControl::default_obis(),
            OutputState::Disconnected,
        );
        control.remote_reconnect().await.unwrap();
        assert_eq!(control.output_state().await, OutputState::Connected);
        assert!(control.is_connected().await);
    }

    #[tokio::test]
    async fn test_disconnect_control_double_disconnect() {
        let control = DisconnectControl::with_default_obis();
        control.remote_disconnect().await.unwrap();
        let result = control.remote_disconnect().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_disconnect_control_double_reconnect() {
        let control = DisconnectControl::with_default_obis();
        let result = control.remote_reconnect().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_disconnect_control_disconnect_disabled() {
        let control = DisconnectControl::with_default_obis();
        control.set_disconnect_enabled(false).await;
        let result = control.remote_disconnect().await;
        assert!(result.is_err());
        assert!(matches!(result, Err(DlmsError::AccessDenied(_))));
    }

    #[tokio::test]
    async fn test_disconnect_control_reconnect_disabled() {
        let control = DisconnectControl::new(
            DisconnectControl::default_obis(),
            OutputState::Disconnected,
        );
        control.set_reconnect_enabled(false).await;
        let result = control.remote_reconnect().await;
        assert!(result.is_err());
        assert!(matches!(result, Err(DlmsError::AccessDenied(_))));
    }

    #[tokio::test]
    async fn test_disconnect_control_disconnect_reconnect_cycle() {
        let control = DisconnectControl::with_default_obis();

        // Initially connected
        assert!(control.is_connected().await);

        // Disconnect
        control.remote_disconnect().await.unwrap();
        assert!(!control.is_connected().await);

        // Reconnect
        control.remote_reconnect().await.unwrap();
        assert!(control.is_connected().await);
    }

    #[tokio::test]
    async fn test_disconnect_control_get_logical_name() {
        let control = DisconnectControl::with_default_obis();
        let result = control.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_disconnect_control_get_output_state() {
        let control = DisconnectControl::with_default_obis();
        let result = control.get_attribute(2, None).await.unwrap();
        assert_eq!(result, DataObject::Enumerate(1)); // Connected = 1

        control.set_output_state(OutputState::Disconnected).await;
        let result = control.get_attribute(2, None).await.unwrap();
        assert_eq!(result, DataObject::Enumerate(0)); // Disconnected = 0
    }

    #[tokio::test]
    async fn test_disconnect_control_set_output_state_via_attribute() {
        let control = DisconnectControl::with_default_obis();
        control
            .set_attribute(2, DataObject::Enumerate(0), None)
            .await
            .unwrap();
        assert_eq!(control.output_state().await, OutputState::Disconnected);
    }

    #[tokio::test]
    async fn test_disconnect_control_set_output_state_boolean() {
        let control = DisconnectControl::with_default_obis();
        control
            .set_attribute(2, DataObject::Boolean(false), None)
            .await
            .unwrap();
        assert!(!control.is_connected().await);
    }

    #[tokio::test]
    async fn test_disconnect_control_read_only_logical_name() {
        let control = DisconnectControl::with_default_obis();
        let result = control
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 96, 1, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_disconnect_control_method_disconnect() {
        let control = DisconnectControl::with_default_obis();
        let result = control.invoke_method(1, None, None).await;
        assert!(result.is_ok());
        assert!(!control.is_connected().await);
    }

    #[tokio::test]
    async fn test_disconnect_control_method_reconnect() {
        let control = DisconnectControl::new(
            DisconnectControl::default_obis(),
            OutputState::Disconnected,
        );
        let result = control.invoke_method(2, None, None).await;
        assert!(result.is_ok());
        assert!(control.is_connected().await);
    }

    #[tokio::test]
    async fn test_disconnect_control_invalid_method() {
        let control = DisconnectControl::with_default_obis();
        let result = control.invoke_method(99, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_disconnect_control_invalid_attribute() {
        let control = DisconnectControl::with_default_obis();
        let result = control.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_output_state_from_u8() {
        assert_eq!(OutputState::from_u8(0), OutputState::Disconnected);
        assert_eq!(OutputState::from_u8(1), OutputState::Connected);
        assert_eq!(OutputState::from_u8(2), OutputState::ReadyForReconnect);
        assert_eq!(OutputState::from_u8(3), OutputState::DisconnectInProgress);
        assert_eq!(OutputState::from_u8(4), OutputState::ReconnectInProgress);
        assert_eq!(OutputState::from_u8(99), OutputState::Disconnected); // Invalid defaults to Disconnected
    }

    #[tokio::test]
    async fn test_output_state_is_connected() {
        assert!(OutputState::Connected.is_connected());
        assert!(!OutputState::Disconnected.is_connected());
        assert!(!OutputState::ReadyForReconnect.is_connected());
    }

    #[tokio::test]
    async fn test_output_state_is_disconnected() {
        assert!(OutputState::Disconnected.is_disconnected());
        assert!(OutputState::ReadyForReconnect.is_disconnected());
        assert!(!OutputState::Connected.is_disconnected());
    }

    #[tokio::test]
    async fn test_output_state_is_in_transition() {
        assert!(OutputState::DisconnectInProgress.is_in_transition());
        assert!(OutputState::ReconnectInProgress.is_in_transition());
        assert!(!OutputState::Connected.is_in_transition());
    }

    #[tokio::test]
    async fn test_disconnect_control_enabled_states() {
        let control = DisconnectControl::with_default_obis();
        let (disconnect, reconnect) = control.enabled_states().await;
        assert!(disconnect);
        assert!(reconnect);

        control.set_disconnect_enabled(false).await;
        let (disconnect, reconnect) = control.enabled_states().await;
        assert!(!disconnect);
        assert!(reconnect);
    }
}
