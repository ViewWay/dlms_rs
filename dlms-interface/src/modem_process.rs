//! Modem Process interface class (Class ID: 30)
//!
//! The Modem Process interface class manages modem control and status monitoring.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: modem_enabled - Whether modem is enabled
//! - Attribute 3: modem_status - Current modem status
//! - Attribute 4: modem_port - Modem port identifier
//! - Attribute 5: baud_rate - Communication baud rate
//! - Attribute 6: modem_initialization_string - Initialization command
//! - Attribute 7: dial_string - Dial command string
//! - Attribute 8: connection_status - Connection status
//! - Attribute 9: connection_count - Number of connection attempts
//! - Attribute 10: max_connection_count - Maximum allowed connection attempts

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Modem Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ModemStatus {
    /// Modem is idle
    Idle = 0,
    /// Modem is dialing
    Dialing = 1,
    /// Modem is connected
    Connected = 2,
    /// Modem connection failed
    ConnectionFailed = 3,
    /// Modem is busy
    Busy = 4,
    /// Modem error occurred
    Error = 5,
    /// Modem is disabled
    Disabled = 6,
    /// Modem is initializing
    Initializing = 7,
}

impl ModemStatus {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Idle,
            1 => Self::Dialing,
            2 => Self::Connected,
            3 => Self::ConnectionFailed,
            4 => Self::Busy,
            5 => Self::Error,
            6 => Self::Disabled,
            _ => Self::Idle,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if modem is in a connected state
    pub fn is_connected(self) -> bool {
        matches!(self, Self::Connected)
    }

    /// Check if modem is in an error state
    pub fn is_error(self) -> bool {
        matches!(self, Self::ConnectionFailed | Self::Error)
    }

    /// Check if modem is available for use
    pub fn is_available(self) -> bool {
        matches!(self, Self::Idle | Self::Connected)
    }
}

/// Modem Connection Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ModemConnectionStatus {
    /// Not connected
    NotConnected = 0,
    /// Connecting
    Connecting = 1,
    /// Connected
    Connected = 2,
    /// Disconnecting
    Disconnecting = 3,
    /// Connection failed
    Failed = 4,
}

impl ModemConnectionStatus {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::NotConnected,
            1 => Self::Connecting,
            2 => Self::Connected,
            3 => Self::Disconnecting,
            _ => Self::Failed,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if currently connected
    pub fn is_connected(self) -> bool {
        matches!(self, Self::Connected)
    }

    /// Check if in transition state
    pub fn is_transitioning(self) -> bool {
        matches!(self, Self::Connecting | Self::Disconnecting)
    }
}

/// Modem Process interface class (Class ID: 30)
///
/// Default OBIS: 0-0:30.0.0.255
///
/// This class manages modem control and status monitoring.
#[derive(Debug, Clone)]
pub struct ModemProcess {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Whether modem is enabled
    modem_enabled: Arc<RwLock<bool>>,

    /// Current modem status
    modem_status: Arc<RwLock<ModemStatus>>,

    /// Modem port identifier
    modem_port: Arc<RwLock<String>>,

    /// Communication baud rate
    baud_rate: Arc<RwLock<u32>>,

    /// Initialization string
    init_string: Arc<RwLock<String>>,

    /// Dial string
    dial_string: Arc<RwLock<String>>,

    /// Connection status
    connection_status: Arc<RwLock<ModemConnectionStatus>>,

    /// Connection attempt count
    connection_count: Arc<RwLock<u16>>,

    /// Maximum allowed connection attempts
    max_connection_count: Arc<RwLock<u16>>,
}

impl ModemProcess {
    /// Class ID for ModemProcess
    pub const CLASS_ID: u16 = 30;

    /// Default OBIS code for ModemProcess (0-0:30.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 30, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_MODEM_ENABLED: u8 = 2;
    pub const ATTR_MODEM_STATUS: u8 = 3;
    pub const ATTR_MODEM_PORT: u8 = 4;
    pub const ATTR_BAUD_RATE: u8 = 5;
    pub const ATTR_INIT_STRING: u8 = 6;
    pub const ATTR_DIAL_STRING: u8 = 7;
    pub const ATTR_CONNECTION_STATUS: u8 = 8;
    pub const ATTR_CONNECTION_COUNT: u8 = 9;
    pub const ATTR_MAX_CONNECTION_COUNT: u8 = 10;

    /// Create a new ModemProcess object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            modem_enabled: Arc::new(RwLock::new(false)),
            modem_status: Arc::new(RwLock::new(ModemStatus::Disabled)),
            modem_port: Arc::new(RwLock::new(String::from("/dev/ttyUSB0"))),
            baud_rate: Arc::new(RwLock::new(115200)),
            init_string: Arc::new(RwLock::new(String::from("ATZ"))),
            dial_string: Arc::new(RwLock::new(String::new())),
            connection_status: Arc::new(RwLock::new(ModemConnectionStatus::NotConnected)),
            connection_count: Arc::new(RwLock::new(0)),
            max_connection_count: Arc::new(RwLock::new(3)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get whether modem is enabled
    pub async fn modem_enabled(&self) -> bool {
        *self.modem_enabled.read().await
    }

    /// Set whether modem is enabled
    pub async fn set_modem_enabled(&self, enabled: bool) {
        *self.modem_enabled.write().await = enabled;
        if !enabled {
            *self.modem_status.write().await = ModemStatus::Disabled;
        }
    }

    /// Get the modem status
    pub async fn modem_status(&self) -> ModemStatus {
        *self.modem_status.read().await
    }

    /// Set the modem status
    pub async fn set_modem_status(&self, status: ModemStatus) {
        *self.modem_status.write().await = status;
    }

    /// Get the modem port
    pub async fn modem_port(&self) -> String {
        self.modem_port.read().await.clone()
    }

    /// Set the modem port
    pub async fn set_modem_port(&self, port: String) {
        *self.modem_port.write().await = port;
    }

    /// Get the baud rate
    pub async fn baud_rate(&self) -> u32 {
        *self.baud_rate.read().await
    }

    /// Set the baud rate
    pub async fn set_baud_rate(&self, rate: u32) {
        *self.baud_rate.write().await = rate;
    }

    /// Get the initialization string
    pub async fn init_string(&self) -> String {
        self.init_string.read().await.clone()
    }

    /// Set the initialization string
    pub async fn set_init_string(&self, init: String) {
        *self.init_string.write().await = init;
    }

    /// Get the dial string
    pub async fn dial_string(&self) -> String {
        self.dial_string.read().await.clone()
    }

    /// Set the dial string
    pub async fn set_dial_string(&self, dial: String) {
        *self.dial_string.write().await = dial;
    }

    /// Get the connection status
    pub async fn connection_status(&self) -> ModemConnectionStatus {
        *self.connection_status.read().await
    }

    /// Set the connection status
    pub async fn set_connection_status(&self, status: ModemConnectionStatus) {
        *self.connection_status.write().await = status;
    }

    /// Get the connection count
    pub async fn connection_count(&self) -> u16 {
        *self.connection_count.read().await
    }

    /// Reset the connection count
    pub async fn reset_connection_count(&self) {
        *self.connection_count.write().await = 0;
    }

    /// Increment the connection count
    pub async fn increment_connection_count(&self) {
        *self.connection_count.write().await += 1;
    }

    /// Get the max connection count
    pub async fn max_connection_count(&self) -> u16 {
        *self.max_connection_count.read().await
    }

    /// Set the max connection count
    pub async fn set_max_connection_count(&self, max: u16) {
        *self.max_connection_count.write().await = max.max(1);
    }

    /// Check if max connection attempts reached
    pub async fn is_max_attempts_reached(&self) -> bool {
        self.connection_count().await >= self.max_connection_count().await
    }

    /// Enable the modem
    pub async fn enable(&self) {
        self.set_modem_enabled(true).await;
        if self.modem_status().await == ModemStatus::Disabled {
            self.set_modem_status(ModemStatus::Idle).await;
        }
    }

    /// Disable the modem
    pub async fn disable(&self) {
        self.set_modem_enabled(false).await;
        self.set_modem_status(ModemStatus::Disabled).await;
    }

    /// Check if modem is connected
    pub async fn is_connected(&self) -> bool {
        self.modem_status().await.is_connected()
    }

    /// Check if modem is available
    pub async fn is_available(&self) -> bool {
        self.modem_status().await.is_available()
    }

    /// Initiate a connection
    pub async fn connect(&self, phone_number: String) {
        self.set_dial_string(phone_number).await;
        self.set_modem_status(ModemStatus::Dialing).await;
        self.set_connection_status(ModemConnectionStatus::Connecting).await;
        self.increment_connection_count().await;
    }

    /// Disconnect
    pub async fn disconnect(&self) {
        self.set_connection_status(ModemConnectionStatus::Disconnecting).await;
        self.set_modem_status(ModemStatus::Idle).await;
        self.set_connection_status(ModemConnectionStatus::NotConnected).await;
        self.reset_connection_count().await;
    }
}

#[async_trait]
impl CosemObject for ModemProcess {
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
            Self::ATTR_MODEM_ENABLED => {
                Ok(DataObject::Boolean(self.modem_enabled().await))
            }
            Self::ATTR_MODEM_STATUS => {
                Ok(DataObject::Enumerate(self.modem_status().await.to_u8()))
            }
            Self::ATTR_MODEM_PORT => {
                Ok(DataObject::OctetString(self.modem_port().await.into_bytes()))
            }
            Self::ATTR_BAUD_RATE => {
                Ok(DataObject::Unsigned32(self.baud_rate().await))
            }
            Self::ATTR_INIT_STRING => {
                Ok(DataObject::OctetString(self.init_string().await.into_bytes()))
            }
            Self::ATTR_DIAL_STRING => {
                Ok(DataObject::OctetString(self.dial_string().await.into_bytes()))
            }
            Self::ATTR_CONNECTION_STATUS => {
                Ok(DataObject::Enumerate(self.connection_status().await.to_u8()))
            }
            Self::ATTR_CONNECTION_COUNT => {
                Ok(DataObject::Unsigned16(self.connection_count().await))
            }
            Self::ATTR_MAX_CONNECTION_COUNT => {
                Ok(DataObject::Unsigned16(self.max_connection_count().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "ModemProcess has no attribute {}",
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
            Self::ATTR_MODEM_ENABLED => {
                match value {
                    DataObject::Boolean(enabled) => {
                        self.set_modem_enabled(enabled).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Boolean for modem_enabled".to_string(),
                    )),
                }
            }
            Self::ATTR_MODEM_STATUS => {
                match value {
                    DataObject::Enumerate(status) => {
                        self.set_modem_status(ModemStatus::from_u8(status)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for modem_status".to_string(),
                    )),
                }
            }
            Self::ATTR_MODEM_PORT => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_modem_port(String::from_utf8_lossy(&bytes).to_string()).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for modem_port".to_string(),
                    )),
                }
            }
            Self::ATTR_BAUD_RATE => {
                match value {
                    DataObject::Unsigned32(rate) => {
                        self.set_baud_rate(rate).await;
                        Ok(())
                    }
                    DataObject::Unsigned16(rate) => {
                        self.set_baud_rate(rate as u32).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned32/Unsigned16 for baud_rate".to_string(),
                    )),
                }
            }
            Self::ATTR_INIT_STRING => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_init_string(String::from_utf8_lossy(&bytes).to_string()).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for init_string".to_string(),
                    )),
                }
            }
            Self::ATTR_DIAL_STRING => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_dial_string(String::from_utf8_lossy(&bytes).to_string()).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for dial_string".to_string(),
                    )),
                }
            }
            Self::ATTR_CONNECTION_STATUS => {
                match value {
                    DataObject::Enumerate(status) => {
                        self.set_connection_status(ModemConnectionStatus::from_u8(status)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for connection_status".to_string(),
                    )),
                }
            }
            Self::ATTR_CONNECTION_COUNT => {
                match value {
                    DataObject::Unsigned16(count) => {
                        *self.connection_count.write().await = count;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned16 for connection_count".to_string(),
                    )),
                }
            }
            Self::ATTR_MAX_CONNECTION_COUNT => {
                match value {
                    DataObject::Unsigned16(max) => {
                        self.set_max_connection_count(max).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned16 for max_connection_count".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "ModemProcess has no attribute {}",
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
            "ModemProcess has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_modem_process_class_id() {
        let modem = ModemProcess::with_default_obis();
        assert_eq!(modem.class_id(), 30);
    }

    #[tokio::test]
    async fn test_modem_process_obis_code() {
        let modem = ModemProcess::with_default_obis();
        assert_eq!(modem.obis_code(), ModemProcess::default_obis());
    }

    #[tokio::test]
    async fn test_modem_status_from_u8() {
        assert_eq!(ModemStatus::from_u8(0), ModemStatus::Idle);
        assert_eq!(ModemStatus::from_u8(1), ModemStatus::Dialing);
        assert_eq!(ModemStatus::from_u8(2), ModemStatus::Connected);
        assert_eq!(ModemStatus::from_u8(3), ModemStatus::ConnectionFailed);
        assert_eq!(ModemStatus::from_u8(4), ModemStatus::Busy);
        assert_eq!(ModemStatus::from_u8(5), ModemStatus::Error);
        assert_eq!(ModemStatus::from_u8(6), ModemStatus::Disabled);
    }

    #[tokio::test]
    async fn test_modem_status_is_connected() {
        assert!(ModemStatus::Connected.is_connected());
        assert!(!ModemStatus::Idle.is_connected());
        assert!(!ModemStatus::Dialing.is_connected());
    }

    #[tokio::test]
    async fn test_modem_status_is_error() {
        assert!(ModemStatus::ConnectionFailed.is_error());
        assert!(ModemStatus::Error.is_error());
        assert!(!ModemStatus::Connected.is_error());
    }

    #[tokio::test]
    async fn test_modem_status_is_available() {
        assert!(ModemStatus::Idle.is_available());
        assert!(ModemStatus::Connected.is_available());
        assert!(!ModemStatus::Busy.is_available());
        assert!(!ModemStatus::Disabled.is_available());
    }

    #[tokio::test]
    async fn test_connection_status_from_u8() {
        assert_eq!(ModemConnectionStatus::from_u8(0), ModemConnectionStatus::NotConnected);
        assert_eq!(ModemConnectionStatus::from_u8(1), ModemConnectionStatus::Connecting);
        assert_eq!(ModemConnectionStatus::from_u8(2), ModemConnectionStatus::Connected);
        assert_eq!(ModemConnectionStatus::from_u8(3), ModemConnectionStatus::Disconnecting);
    }

    #[tokio::test]
    async fn test_connection_status_is_connected() {
        assert!(ModemConnectionStatus::Connected.is_connected());
        assert!(!ModemConnectionStatus::NotConnected.is_connected());
    }

    #[tokio::test]
    async fn test_connection_status_is_transitioning() {
        assert!(ModemConnectionStatus::Connecting.is_transitioning());
        assert!(ModemConnectionStatus::Disconnecting.is_transitioning());
        assert!(!ModemConnectionStatus::Connected.is_transitioning());
    }

    #[tokio::test]
    async fn test_modem_process_initial_state() {
        let modem = ModemProcess::with_default_obis();
        assert!(!modem.modem_enabled().await);
        assert_eq!(modem.modem_status().await, ModemStatus::Disabled);
        assert_eq!(modem.baud_rate().await, 115200);
        assert_eq!(modem.connection_count().await, 0);
        assert_eq!(modem.max_connection_count().await, 3);
    }

    #[tokio::test]
    async fn test_modem_process_enable_disable() {
        let modem = ModemProcess::with_default_obis();
        assert_eq!(modem.modem_status().await, ModemStatus::Disabled);

        modem.enable().await;
        assert!(modem.modem_enabled().await);
        assert_eq!(modem.modem_status().await, ModemStatus::Idle);

        modem.disable().await;
        assert!(!modem.modem_enabled().await);
        assert_eq!(modem.modem_status().await, ModemStatus::Disabled);
    }

    #[tokio::test]
    async fn test_modem_process_set_modem_port() {
        let modem = ModemProcess::with_default_obis();
        modem.set_modem_port(String::from("/dev/ttyUSB1")).await;
        assert_eq!(modem.modem_port().await, "/dev/ttyUSB1");
    }

    #[tokio::test]
    async fn test_modem_process_set_baud_rate() {
        let modem = ModemProcess::with_default_obis();
        modem.set_baud_rate(57600).await;
        assert_eq!(modem.baud_rate().await, 57600);
    }

    #[tokio::test]
    async fn test_modem_process_set_init_string() {
        let modem = ModemProcess::with_default_obis();
        modem.set_init_string(String::from("AT&F")).await;
        assert_eq!(modem.init_string().await, "AT&F");
    }

    #[tokio::test]
    async fn test_modem_process_set_dial_string() {
        let modem = ModemProcess::with_default_obis();
        modem.set_dial_string(String::from("ATD12345678")).await;
        assert_eq!(modem.dial_string().await, "ATD12345678");
    }

    #[tokio::test]
    async fn test_modem_process_connection_count() {
        let modem = ModemProcess::with_default_obis();
        assert_eq!(modem.connection_count().await, 0);

        modem.increment_connection_count().await;
        assert_eq!(modem.connection_count().await, 1);

        modem.increment_connection_count().await;
        assert_eq!(modem.connection_count().await, 2);

        modem.reset_connection_count().await;
        assert_eq!(modem.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_modem_process_max_connection_count() {
        let modem = ModemProcess::with_default_obis();
        modem.set_max_connection_count(5).await;
        assert_eq!(modem.max_connection_count().await, 5);

        // Test minimum value
        modem.set_max_connection_count(0).await;
        assert_eq!(modem.max_connection_count().await, 1);
    }

    #[tokio::test]
    async fn test_modem_process_is_max_attempts_reached() {
        let modem = ModemProcess::with_default_obis();
        assert!(!modem.is_max_attempts_reached().await);

        *modem.connection_count.write().await = 3;
        assert!(modem.is_max_attempts_reached().await);
    }

    #[tokio::test]
    async fn test_modem_process_connect() {
        let modem = ModemProcess::with_default_obis();
        modem.enable().await;

        modem.connect(String::from("12345678")).await;
        assert_eq!(modem.modem_status().await, ModemStatus::Dialing);
        assert_eq!(modem.connection_status().await, ModemConnectionStatus::Connecting);
        assert_eq!(modem.dial_string().await, "12345678");
    }

    #[tokio::test]
    async fn test_modem_process_disconnect() {
        let modem = ModemProcess::with_default_obis();
        modem.enable().await;
        modem.connect(String::from("12345678")).await;
        modem.increment_connection_count().await;

        modem.disconnect().await;
        assert_eq!(modem.connection_status().await, ModemConnectionStatus::NotConnected);
        assert_eq!(modem.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_modem_process_is_connected() {
        let modem = ModemProcess::with_default_obis();
        assert!(!modem.is_connected().await);

        modem.set_modem_status(ModemStatus::Connected).await;
        assert!(modem.is_connected().await);
    }

    #[tokio::test]
    async fn test_modem_process_is_available() {
        let modem = ModemProcess::with_default_obis();
        assert!(!modem.is_available().await); // Disabled state

        modem.enable().await;
        assert!(modem.is_available().await); // Idle state

        modem.set_modem_status(ModemStatus::Connected).await;
        assert!(modem.is_available().await);

        modem.set_modem_status(ModemStatus::Busy).await;
        assert!(!modem.is_available().await);
    }

    #[tokio::test]
    async fn test_modem_process_get_attributes() {
        let modem = ModemProcess::with_default_obis();

        // Test modem_enabled
        let result = modem.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Boolean(enabled) => assert!(!enabled),
            _ => panic!("Expected Boolean"),
        }

        // Test modem_status
        let result = modem.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Enumerate(status) => assert_eq!(status, 6), // Disabled
            _ => panic!("Expected Enumerate"),
        }

        // Test baud_rate
        let result = modem.get_attribute(5, None).await.unwrap();
        match result {
            DataObject::Unsigned32(rate) => assert_eq!(rate, 115200),
            _ => panic!("Expected Unsigned32"),
        }
    }

    #[tokio::test]
    async fn test_modem_process_set_attributes() {
        let modem = ModemProcess::with_default_obis();

        modem.set_attribute(2, DataObject::Boolean(true), None)
            .await
            .unwrap();
        assert!(modem.modem_enabled().await);

        modem.set_attribute(5, DataObject::Unsigned32(57600), None)
            .await
            .unwrap();
        assert_eq!(modem.baud_rate().await, 57600);

        modem.set_attribute(5, DataObject::Unsigned16(9600), None)
            .await
            .unwrap();
        assert_eq!(modem.baud_rate().await, 9600);
    }

    #[tokio::test]
    async fn test_modem_process_read_only_logical_name() {
        let modem = ModemProcess::with_default_obis();
        let result = modem
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 30, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_modem_process_invalid_attribute() {
        let modem = ModemProcess::with_default_obis();
        let result = modem.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_modem_process_invalid_method() {
        let modem = ModemProcess::with_default_obis();
        let result = modem.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_modem_process_invalid_data_type() {
        let modem = ModemProcess::with_default_obis();
        let result = modem.set_attribute(2, DataObject::Unsigned8(1), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_modem_process_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 30, 0, 0, 1);
        let modem = ModemProcess::new(obis);
        assert_eq!(modem.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_modem_process_set_connection_status() {
        let modem = ModemProcess::with_default_obis();
        modem.set_connection_status(ModemConnectionStatus::Connecting).await;
        assert_eq!(modem.connection_status().await, ModemConnectionStatus::Connecting);
        assert!(modem.connection_status().await.is_transitioning());
    }
}
