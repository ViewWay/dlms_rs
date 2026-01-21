//! Auto Connect interface class (Class ID: 45)
//!
//! The Auto Connect interface class manages automatic connection settings
//! for communication modules (modems, GPRS, etc.).
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: mode - Operating mode (disabled, always, waiting for window)
//! - Attribute 3: repetition_delay - Delay between connection attempts (seconds)
//! - Attribute 4: repetition_number - Number of connection attempts
//! - Attribute 5: calling_window - Time window when connections are allowed
//! - Attribute 6: destination - Connection destination (phone number, IP, etc.)
//! - Attribute 7: status - Current connection status

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{
    datatypes::{CosemDateTime, CosemDateFormat},
    DlmsError, DlmsResult, ObisCode, DataObject,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Auto Connect Mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AutoConnectMode {
    /// Auto connect disabled
    Disabled = 0,
    /// Always attempt to connect
    Always = 1,
    /// Connect only within defined time windows
    WaitingForWindow = 2,
}

impl AutoConnectMode {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Disabled,
            1 => Self::Always,
            2 => Self::WaitingForWindow,
            _ => Self::Disabled,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if enabled
    pub fn is_enabled(self) -> bool {
        matches!(self, Self::Always | Self::WaitingForWindow)
    }
}

/// Connection Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ConnectionStatus {
    /// Not connected
    NotConnected = 0,
    /// Connecting in progress
    Connecting = 1,
    /// Connected
    Connected = 2,
    /// Connection failed
    Failed = 3,
}

impl ConnectionStatus {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::NotConnected,
            1 => Self::Connecting,
            2 => Self::Connected,
            3 => Self::Failed,
            _ => Self::NotConnected,
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

    /// Check if in progress
    pub fn is_in_progress(self) -> bool {
        matches!(self, Self::Connecting)
    }
}

/// Time Window for connections
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionTimeWindow {
    /// Window start time
    pub start_time: CosemDateTime,
    /// Window end time
    pub end_time: CosemDateTime,
    /// Days of week (bitmap: bit 0 = Monday, bit 6 = Sunday)
    pub days_of_week: u8,
}

impl ConnectionTimeWindow {
    /// Create a new time window
    pub fn new(start_time: CosemDateTime, end_time: CosemDateTime, days_of_week: u8) -> Self {
        Self {
            start_time,
            end_time,
            days_of_week,
        }
    }

    /// Convert to data object
    pub fn to_data_object(&self) -> DataObject {
        DataObject::Array(vec![
            DataObject::OctetString(self.start_time.encode()),
            DataObject::OctetString(self.end_time.encode()),
            DataObject::Unsigned8(self.days_of_week),
        ])
    }

    /// Create from data object
    pub fn from_data_object(value: &DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Array(arr) if arr.len() >= 3 => {
                let start_time = match &arr[0] {
                    DataObject::OctetString(bytes) if bytes.len() >= 12 => {
                        CosemDateTime::decode(bytes)?
                    }
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected OctetString for start_time".to_string(),
                        ))
                    }
                };
                let end_time = match &arr[1] {
                    DataObject::OctetString(bytes) if bytes.len() >= 12 => {
                        CosemDateTime::decode(bytes)?
                    }
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected OctetString for end_time".to_string(),
                        ))
                    }
                };
                let days_of_week = match &arr[2] {
                    DataObject::Unsigned8(d) => *d,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Unsigned8 for days_of_week".to_string(),
                        ))
                    }
                };
                Ok(Self {
                    start_time,
                    end_time,
                    days_of_week,
                })
            }
            _ => Err(DlmsError::InvalidData(
                "Expected Array for ConnectionTimeWindow".to_string(),
            )),
        }
    }
}

/// Auto Connect interface class (Class ID: 45)
///
/// Default OBIS: 0-0:45.0.0.255
///
/// This class manages automatic connection settings.
#[derive(Debug, Clone)]
pub struct AutoConnect {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Operating mode
    mode: Arc<RwLock<AutoConnectMode>>,

    /// Delay between connection attempts (seconds)
    repetition_delay: Arc<RwLock<u32>>,

    /// Number of connection attempts (0 = infinite)
    repetition_number: Arc<RwLock<u16>>,

    /// Time windows for connections
    calling_windows: Arc<RwLock<Vec<ConnectionTimeWindow>>>,

    /// Connection destination
    destination: Arc<RwLock<String>>,

    /// Current connection status
    status: Arc<RwLock<ConnectionStatus>>,
}

impl AutoConnect {
    /// Class ID for Auto Connect
    pub const CLASS_ID: u16 = 45;

    /// Default OBIS code for Auto Connect (0-0:45.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 45, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_MODE: u8 = 2;
    pub const ATTR_REPETITION_DELAY: u8 = 3;
    pub const ATTR_REPETITION_NUMBER: u8 = 4;
    pub const ATTR_CALLING_WINDOW: u8 = 5;
    pub const ATTR_DESTINATION: u8 = 6;
    pub const ATTR_STATUS: u8 = 7;

    /// Create a new Auto Connect object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            mode: Arc::new(RwLock::new(AutoConnectMode::Disabled)),
            repetition_delay: Arc::new(RwLock::new(60)),
            repetition_number: Arc::new(RwLock::new(0)),
            calling_windows: Arc::new(RwLock::new(Vec::new())),
            destination: Arc::new(RwLock::new(String::new())),
            status: Arc::new(RwLock::new(ConnectionStatus::NotConnected)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get the mode
    pub async fn mode(&self) -> AutoConnectMode {
        *self.mode.read().await
    }

    /// Set the mode
    pub async fn set_mode(&self, mode: AutoConnectMode) {
        *self.mode.write().await = mode;
    }

    /// Get the repetition delay
    pub async fn repetition_delay(&self) -> u32 {
        *self.repetition_delay.read().await
    }

    /// Set the repetition delay
    pub async fn set_repetition_delay(&self, delay: u32) {
        *self.repetition_delay.write().await = delay;
    }

    /// Get the repetition number
    pub async fn repetition_number(&self) -> u16 {
        *self.repetition_number.read().await
    }

    /// Set the repetition number
    pub async fn set_repetition_number(&self, number: u16) {
        *self.repetition_number.write().await = number;
    }

    /// Get the calling windows
    pub async fn calling_windows(&self) -> Vec<ConnectionTimeWindow> {
        self.calling_windows.read().await.clone()
    }

    /// Add a calling window
    pub async fn add_calling_window(&self, window: ConnectionTimeWindow) {
        self.calling_windows.write().await.push(window);
    }

    /// Clear all calling windows
    pub async fn clear_calling_windows(&self) {
        self.calling_windows.write().await.clear();
    }

    /// Get the destination
    pub async fn destination(&self) -> String {
        self.destination.read().await.clone()
    }

    /// Set the destination
    pub async fn set_destination(&self, dest: String) {
        *self.destination.write().await = dest;
    }

    /// Get the status
    pub async fn status(&self) -> ConnectionStatus {
        *self.status.read().await
    }

    /// Set the status
    pub async fn set_status(&self, status: ConnectionStatus) {
        *self.status.write().await = status;
    }

    /// Enable auto connect (always mode)
    pub async fn enable(&self) {
        self.set_mode(AutoConnectMode::Always).await;
    }

    /// Disable auto connect
    pub async fn disable(&self) {
        self.set_mode(AutoConnectMode::Disabled).await;
    }

    /// Check if auto connect is enabled
    pub async fn is_enabled(&self) -> bool {
        self.mode().await.is_enabled()
    }
}

#[async_trait]
impl CosemObject for AutoConnect {
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
            Self::ATTR_MODE => {
                Ok(DataObject::Enumerate(self.mode().await.to_u8()))
            }
            Self::ATTR_REPETITION_DELAY => {
                Ok(DataObject::Unsigned32(self.repetition_delay().await))
            }
            Self::ATTR_REPETITION_NUMBER => {
                Ok(DataObject::Unsigned16(self.repetition_number().await))
            }
            Self::ATTR_CALLING_WINDOW => {
                let windows = self.calling_windows().await;
                let data: Vec<DataObject> = windows
                    .iter()
                    .map(|w| w.to_data_object())
                    .collect();
                Ok(DataObject::Array(data))
            }
            Self::ATTR_DESTINATION => {
                Ok(DataObject::OctetString(self.destination().await.into_bytes()))
            }
            Self::ATTR_STATUS => {
                Ok(DataObject::Enumerate(self.status().await.to_u8()))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Auto Connect has no attribute {}",
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
            Self::ATTR_MODE => {
                match value {
                    DataObject::Enumerate(mode) => {
                        self.set_mode(AutoConnectMode::from_u8(mode)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for mode".to_string(),
                    )),
                }
            }
            Self::ATTR_REPETITION_DELAY => {
                match value {
                    DataObject::Unsigned32(delay) => {
                        self.set_repetition_delay(delay).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned32 for repetition_delay".to_string(),
                    )),
                }
            }
            Self::ATTR_REPETITION_NUMBER => {
                match value {
                    DataObject::Unsigned16(number) => {
                        self.set_repetition_number(number).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned16 for repetition_number".to_string(),
                    )),
                }
            }
            Self::ATTR_CALLING_WINDOW => {
                match value {
                    DataObject::Array(arr) => {
                        self.clear_calling_windows().await;
                        for item in arr {
                            if let Ok(window) = ConnectionTimeWindow::from_data_object(&item) {
                                self.add_calling_window(window).await;
                            }
                        }
                        Ok(())
                    }
                    DataObject::Null => {
                        self.clear_calling_windows().await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Array for calling_window".to_string(),
                    )),
                }
            }
            Self::ATTR_DESTINATION => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let dest = String::from_utf8_lossy(&bytes).to_string();
                        self.set_destination(dest).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for destination".to_string(),
                    )),
                }
            }
            Self::ATTR_STATUS => {
                match value {
                    DataObject::Enumerate(status) => {
                        self.set_status(ConnectionStatus::from_u8(status)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for status".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Auto Connect has no attribute {}",
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
        Err(DlmsError::InvalidData(format!(
            "Auto Connect has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auto_connect_class_id() {
        let ac = AutoConnect::with_default_obis();
        assert_eq!(ac.class_id(), 45);
    }

    #[tokio::test]
    async fn test_auto_connect_obis_code() {
        let ac = AutoConnect::with_default_obis();
        assert_eq!(ac.obis_code(), AutoConnect::default_obis());
    }

    #[tokio::test]
    async fn test_auto_connect_mode_from_u8() {
        assert_eq!(AutoConnectMode::from_u8(0), AutoConnectMode::Disabled);
        assert_eq!(AutoConnectMode::from_u8(1), AutoConnectMode::Always);
        assert_eq!(AutoConnectMode::from_u8(2), AutoConnectMode::WaitingForWindow);
    }

    #[tokio::test]
    async fn test_auto_connect_mode_is_enabled() {
        assert!(AutoConnectMode::Always.is_enabled());
        assert!(AutoConnectMode::WaitingForWindow.is_enabled());
        assert!(!AutoConnectMode::Disabled.is_enabled());
    }

    #[tokio::test]
    async fn test_connection_status_from_u8() {
        assert_eq!(ConnectionStatus::from_u8(0), ConnectionStatus::NotConnected);
        assert_eq!(ConnectionStatus::from_u8(1), ConnectionStatus::Connecting);
        assert_eq!(ConnectionStatus::from_u8(2), ConnectionStatus::Connected);
        assert_eq!(ConnectionStatus::from_u8(3), ConnectionStatus::Failed);
    }

    #[tokio::test]
    async fn test_connection_status_is_connected() {
        assert!(ConnectionStatus::Connected.is_connected());
        assert!(!ConnectionStatus::NotConnected.is_connected());
    }

    #[tokio::test]
    async fn test_auto_connect_initial_state() {
        let ac = AutoConnect::with_default_obis();
        assert_eq!(ac.mode().await, AutoConnectMode::Disabled);
        assert_eq!(ac.repetition_delay().await, 60);
        assert_eq!(ac.repetition_number().await, 0);
        assert!(ac.destination().await.is_empty());
        assert_eq!(ac.status().await, ConnectionStatus::NotConnected);
    }

    #[tokio::test]
    async fn test_auto_connect_set_mode() {
        let ac = AutoConnect::with_default_obis();
        ac.set_mode(AutoConnectMode::Always).await;
        assert_eq!(ac.mode().await, AutoConnectMode::Always);
    }

    #[tokio::test]
    async fn test_auto_connect_enable_disable() {
        let ac = AutoConnect::with_default_obis();

        ac.enable().await;
        assert!(ac.is_enabled().await);

        ac.disable().await;
        assert!(!ac.is_enabled().await);
    }

    #[tokio::test]
    async fn test_auto_connect_set_repetition_delay() {
        let ac = AutoConnect::with_default_obis();
        ac.set_repetition_delay(120).await;
        assert_eq!(ac.repetition_delay().await, 120);
    }

    #[tokio::test]
    async fn test_auto_connect_set_repetition_number() {
        let ac = AutoConnect::with_default_obis();
        ac.set_repetition_number(5).await;
        assert_eq!(ac.repetition_number().await, 5);
    }

    #[tokio::test]
    async fn test_auto_connect_set_destination() {
        let ac = AutoConnect::with_default_obis();
        ac.set_destination("example.com".to_string()).await;
        assert_eq!(ac.destination().await, "example.com");
    }

    #[tokio::test]
    async fn test_auto_connect_set_status() {
        let ac = AutoConnect::with_default_obis();
        ac.set_status(ConnectionStatus::Connected).await;
        assert_eq!(ac.status().await, ConnectionStatus::Connected);
    }

    #[tokio::test]
    async fn test_auto_connect_add_calling_window() {
        let ac = AutoConnect::with_default_obis();
        let start = CosemDateTime::new(2024, 1, 1, 8, 0, 0, 0, &[]).unwrap();
        let end = CosemDateTime::new(2024, 1, 1, 18, 0, 0, 0, &[]).unwrap();
        let window = ConnectionTimeWindow::new(start.clone(), end, 0x7F);

        ac.add_calling_window(window).await;
        assert_eq!(ac.calling_windows().await.len(), 1);
    }

    #[tokio::test]
    async fn test_auto_connect_clear_calling_windows() {
        let ac = AutoConnect::with_default_obis();
        let start = CosemDateTime::new(2024, 1, 1, 8, 0, 0, 0, &[]).unwrap();
        let end = CosemDateTime::new(2024, 1, 1, 18, 0, 0, 0, &[]).unwrap();
        let window = ConnectionTimeWindow::new(start, end, 0x7F);

        ac.add_calling_window(window).await;
        assert_eq!(ac.calling_windows().await.len(), 1);

        ac.clear_calling_windows().await;
        assert!(ac.calling_windows().await.is_empty());
    }

    #[tokio::test]
    async fn test_connection_time_window_to_data_object() {
        let start = CosemDateTime::new(2024, 1, 1, 8, 0, 0, 0, &[]).unwrap();
        let end = CosemDateTime::new(2024, 1, 1, 18, 0, 0, 0, &[]).unwrap();
        let window = ConnectionTimeWindow::new(start, end, 0x7F);

        let data = window.to_data_object();
        match data {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 3);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_auto_connect_get_attributes() {
        let ac = AutoConnect::with_default_obis();

        // Test mode
        let result = ac.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Enumerate(mode) => assert_eq!(mode, 0), // Disabled
            _ => panic!("Expected Enumerate"),
        }

        // Test repetition_delay
        let result = ac.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Unsigned32(delay) => assert_eq!(delay, 60),
            _ => panic!("Expected Unsigned32"),
        }
    }

    #[tokio::test]
    async fn test_auto_connect_set_attributes() {
        let ac = AutoConnect::with_default_obis();

        ac.set_attribute(2, DataObject::Enumerate(1), None)
            .await
            .unwrap();
        assert_eq!(ac.mode().await, AutoConnectMode::Always);

        ac.set_attribute(6, DataObject::OctetString(b"test.com".to_vec()), None)
            .await
            .unwrap();
        assert_eq!(ac.destination().await, "test.com");
    }

    #[tokio::test]
    async fn test_auto_connect_read_only_logical_name() {
        let ac = AutoConnect::with_default_obis();
        let result = ac
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 45, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auto_connect_invalid_attribute() {
        let ac = AutoConnect::with_default_obis();
        let result = ac.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auto_connect_invalid_method() {
        let ac = AutoConnect::with_default_obis();
        let result = ac.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }
}
