//! Modem Configuration interface class (Class ID: 29)
//!
//! The Modem Configuration interface class manages modem settings for meters.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: communication_speed - Communication speed (baud rate)
//! - Attribute 3: modem_initialization_string - Modem init command
//! - Attribute 4: modem_phone_number - Phone number for dialing
//! - Attribute 5: connection_timeout - Connection timeout in seconds
//! - Attribute 6: modem_response_timeout - Response timeout in seconds
//! - Attribute 7: connection_status - Current connection status
//! - Attribute 8: error_control - Error control protocol

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Baud Rate for communication
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ModemBaudRate {
    /// 300 baud
    B300 = 300,
    /// 600 baud
    B600 = 600,
    /// 1200 baud
    B1200 = 1200,
    /// 2400 baud
    B2400 = 2400,
    /// 4800 baud
    B4800 = 4800,
    /// 9600 baud
    B9600 = 9600,
    /// 19200 baud
    B19200 = 19200,
    /// 38400 baud
    B38400 = 38400,
    /// 57600 baud
    B57600 = 57600,
}

impl ModemBaudRate {
    /// Create from u16
    pub fn from_u16(value: u16) -> Self {
        match value {
            300 => Self::B300,
            600 => Self::B600,
            1200 => Self::B1200,
            2400 => Self::B2400,
            4800 => Self::B4800,
            9600 => Self::B9600,
            19200 => Self::B19200,
            38400 => Self::B38400,
            57600 => Self::B57600,
            _ => Self::B9600, // Default to 9600
        }
    }

    /// Convert to u16
    pub fn to_u16(self) -> u16 {
        self as u16
    }
}

/// Error Control Protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ErrorControl {
    /// No error control
    None = 0,
    /// V.42 error control
    V42 = 1,
    /// MNP (Microcom Networking Protocol)
    MNP = 2,
    /// V.42 and MNP
    V42AndMNP = 3,
}

impl ErrorControl {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::V42,
            2 => Self::MNP,
            3 => Self::V42AndMNP,
            _ => Self::None,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if error control is enabled
    pub fn is_enabled(self) -> bool {
        matches!(self, Self::V42 | Self::MNP | Self::V42AndMNP)
    }
}

/// Modem Configuration interface class (Class ID: 29)
///
/// Default OBIS: 0-0:29.0.0.255
///
/// This class manages modem settings for meters.
#[derive(Debug, Clone)]
pub struct ModemConfiguration {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Communication speed (baud rate)
    communication_speed: Arc<RwLock<ModemBaudRate>>,

    /// Modem initialization string
    modem_initialization_string: Arc<RwLock<String>>,

    /// Modem phone number
    modem_phone_number: Arc<RwLock<String>>,

    /// Connection timeout in seconds
    connection_timeout: Arc<RwLock<u8>>,

    /// Modem response timeout in seconds
    modem_response_timeout: Arc<RwLock<u8>>,

    /// Connection status
    connection_status: Arc<RwLock<bool>>,

    /// Error control protocol
    error_control: Arc<RwLock<ErrorControl>>,
}

impl ModemConfiguration {
    /// Class ID for ModemConfiguration
    pub const CLASS_ID: u16 = 29;

    /// Default OBIS code for ModemConfiguration (0-0:29.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 29, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_COMMUNICATION_SPEED: u8 = 2;
    pub const ATTR_MODEM_INITIALIZATION_STRING: u8 = 3;
    pub const ATTR_MODEM_PHONE_NUMBER: u8 = 4;
    pub const ATTR_CONNECTION_TIMEOUT: u8 = 5;
    pub const ATTR_MODEM_RESPONSE_TIMEOUT: u8 = 6;
    pub const ATTR_CONNECTION_STATUS: u8 = 7;
    pub const ATTR_ERROR_CONTROL: u8 = 8;

    /// Create a new ModemConfiguration object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            communication_speed: Arc::new(RwLock::new(ModemBaudRate::B9600)),
            modem_initialization_string: Arc::new(RwLock::new(String::new())),
            modem_phone_number: Arc::new(RwLock::new(String::new())),
            connection_timeout: Arc::new(RwLock::new(30)),
            modem_response_timeout: Arc::new(RwLock::new(10)),
            connection_status: Arc::new(RwLock::new(false)),
            error_control: Arc::new(RwLock::new(ErrorControl::None)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get the communication speed
    pub async fn communication_speed(&self) -> ModemBaudRate {
        *self.communication_speed.read().await
    }

    /// Set the communication speed
    pub async fn set_communication_speed(&self, speed: ModemBaudRate) {
        *self.communication_speed.write().await = speed;
    }

    /// Get the modem initialization string
    pub async fn modem_initialization_string(&self) -> String {
        self.modem_initialization_string.read().await.clone()
    }

    /// Set the modem initialization string
    pub async fn set_modem_initialization_string(&self, init_string: String) {
        *self.modem_initialization_string.write().await = init_string;
    }

    /// Get the modem phone number
    pub async fn modem_phone_number(&self) -> String {
        self.modem_phone_number.read().await.clone()
    }

    /// Set the modem phone number
    pub async fn set_modem_phone_number(&self, number: String) {
        *self.modem_phone_number.write().await = number;
    }

    /// Get the connection timeout
    pub async fn connection_timeout(&self) -> u8 {
        *self.connection_timeout.read().await
    }

    /// Set the connection timeout
    pub async fn set_connection_timeout(&self, timeout: u8) {
        *self.connection_timeout.write().await = timeout;
    }

    /// Get the modem response timeout
    pub async fn modem_response_timeout(&self) -> u8 {
        *self.modem_response_timeout.read().await
    }

    /// Set the modem response timeout
    pub async fn set_modem_response_timeout(&self, timeout: u8) {
        *self.modem_response_timeout.write().await = timeout;
    }

    /// Get the connection status
    pub async fn connection_status(&self) -> bool {
        *self.connection_status.read().await
    }

    /// Set the connection status
    pub async fn set_connection_status(&self, status: bool) {
        *self.connection_status.write().await = status;
    }

    /// Get the error control protocol
    pub async fn error_control(&self) -> ErrorControl {
        *self.error_control.read().await
    }

    /// Set the error control protocol
    pub async fn set_error_control(&self, error_control: ErrorControl) {
        *self.error_control.write().await = error_control;
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        self.connection_status().await
    }

    /// Check if error control is enabled
    pub async fn is_error_control_enabled(&self) -> bool {
        self.error_control().await.is_enabled()
    }

    /// Connect the modem
    pub async fn connect(&self) {
        *self.connection_status.write().await = true;
    }

    /// Disconnect the modem
    pub async fn disconnect(&self) {
        *self.connection_status.write().await = false;
    }

    /// Reset to default configuration
    pub async fn reset_to_defaults(&self) {
        *self.communication_speed.write().await = ModemBaudRate::B9600;
        *self.modem_initialization_string.write().await = String::new();
        *self.connection_timeout.write().await = 30;
        *self.modem_response_timeout.write().await = 10;
        *self.error_control.write().await = ErrorControl::None;
    }
}

#[async_trait]
impl CosemObject for ModemConfiguration {
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
            Self::ATTR_COMMUNICATION_SPEED => {
                Ok(DataObject::Unsigned16(self.communication_speed().await.to_u16()))
            }
            Self::ATTR_MODEM_INITIALIZATION_STRING => {
                Ok(DataObject::OctetString(self.modem_initialization_string().await.into_bytes()))
            }
            Self::ATTR_MODEM_PHONE_NUMBER => {
                Ok(DataObject::OctetString(self.modem_phone_number().await.into_bytes()))
            }
            Self::ATTR_CONNECTION_TIMEOUT => {
                Ok(DataObject::Unsigned8(self.connection_timeout().await))
            }
            Self::ATTR_MODEM_RESPONSE_TIMEOUT => {
                Ok(DataObject::Unsigned8(self.modem_response_timeout().await))
            }
            Self::ATTR_CONNECTION_STATUS => {
                Ok(DataObject::Boolean(self.connection_status().await))
            }
            Self::ATTR_ERROR_CONTROL => {
                Ok(DataObject::Enumerate(self.error_control().await.to_u8()))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "ModemConfiguration has no attribute {}",
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
            Self::ATTR_COMMUNICATION_SPEED => {
                match value {
                    DataObject::Unsigned16(speed) => {
                        self.set_communication_speed(ModemBaudRate::from_u16(speed)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned16 for communication_speed".to_string(),
                    )),
                }
            }
            Self::ATTR_MODEM_INITIALIZATION_STRING => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let init = String::from_utf8_lossy(&bytes).to_string();
                        self.set_modem_initialization_string(init).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for modem_initialization_string".to_string(),
                    )),
                }
            }
            Self::ATTR_MODEM_PHONE_NUMBER => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let number = String::from_utf8_lossy(&bytes).to_string();
                        self.set_modem_phone_number(number).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for modem_phone_number".to_string(),
                    )),
                }
            }
            Self::ATTR_CONNECTION_TIMEOUT => {
                match value {
                    DataObject::Unsigned8(timeout) => {
                        self.set_connection_timeout(timeout).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for connection_timeout".to_string(),
                    )),
                }
            }
            Self::ATTR_MODEM_RESPONSE_TIMEOUT => {
                match value {
                    DataObject::Unsigned8(timeout) => {
                        self.set_modem_response_timeout(timeout).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for modem_response_timeout".to_string(),
                    )),
                }
            }
            Self::ATTR_CONNECTION_STATUS => {
                match value {
                    DataObject::Boolean(status) => {
                        self.set_connection_status(status).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Boolean for connection_status".to_string(),
                    )),
                }
            }
            Self::ATTR_ERROR_CONTROL => {
                match value {
                    DataObject::Enumerate(error_control) => {
                        self.set_error_control(ErrorControl::from_u8(error_control)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for error_control".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "ModemConfiguration has no attribute {}",
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
            "ModemConfiguration has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_modem_configuration_class_id() {
        let mc = ModemConfiguration::with_default_obis();
        assert_eq!(mc.class_id(), 29);
    }

    #[tokio::test]
    async fn test_modem_configuration_obis_code() {
        let mc = ModemConfiguration::with_default_obis();
        assert_eq!(mc.obis_code(), ModemConfiguration::default_obis());
    }

    #[tokio::test]
    async fn test_modem_baud_rate_from_u16() {
        assert_eq!(ModemBaudRate::from_u16(300), ModemBaudRate::B300);
        assert_eq!(ModemBaudRate::from_u16(9600), ModemBaudRate::B9600);
        assert_eq!(ModemBaudRate::from_u16(57600), ModemBaudRate::B57600);
        assert_eq!(ModemBaudRate::from_u16(12345), ModemBaudRate::B9600); // Default
    }

    #[tokio::test]
    async fn test_modem_baud_rate_to_u16() {
        assert_eq!(ModemBaudRate::B1200.to_u16(), 1200);
        assert_eq!(ModemBaudRate::B9600.to_u16(), 9600);
        assert_eq!(ModemBaudRate::B57600.to_u16(), 57600);
    }

    #[tokio::test]
    async fn test_error_control_from_u8() {
        assert_eq!(ErrorControl::from_u8(0), ErrorControl::None);
        assert_eq!(ErrorControl::from_u8(1), ErrorControl::V42);
        assert_eq!(ErrorControl::from_u8(2), ErrorControl::MNP);
        assert_eq!(ErrorControl::from_u8(3), ErrorControl::V42AndMNP);
    }

    #[tokio::test]
    async fn test_error_control_is_enabled() {
        assert!(!ErrorControl::None.is_enabled());
        assert!(ErrorControl::V42.is_enabled());
        assert!(ErrorControl::MNP.is_enabled());
        assert!(ErrorControl::V42AndMNP.is_enabled());
    }

    #[tokio::test]
    async fn test_modem_configuration_initial_state() {
        let mc = ModemConfiguration::with_default_obis();
        assert_eq!(mc.communication_speed().await, ModemBaudRate::B9600);
        assert_eq!(mc.modem_initialization_string().await, "");
        assert_eq!(mc.modem_phone_number().await, "");
        assert_eq!(mc.connection_timeout().await, 30);
        assert_eq!(mc.modem_response_timeout().await, 10);
        assert!(!mc.connection_status().await);
        assert_eq!(mc.error_control().await, ErrorControl::None);
    }

    #[tokio::test]
    async fn test_modem_configuration_set_communication_speed() {
        let mc = ModemConfiguration::with_default_obis();
        mc.set_communication_speed(ModemBaudRate::B57600).await;
        assert_eq!(mc.communication_speed().await, ModemBaudRate::B57600);
    }

    #[tokio::test]
    async fn test_modem_configuration_set_modem_initialization_string() {
        let mc = ModemConfiguration::with_default_obis();
        mc.set_modem_initialization_string("ATZ".to_string()).await;
        assert_eq!(mc.modem_initialization_string().await, "ATZ");
    }

    #[tokio::test]
    async fn test_modem_configuration_set_modem_phone_number() {
        let mc = ModemConfiguration::with_default_obis();
        mc.set_modem_phone_number("+1234567890".to_string()).await;
        assert_eq!(mc.modem_phone_number().await, "+1234567890");
    }

    #[tokio::test]
    async fn test_modem_configuration_set_timeouts() {
        let mc = ModemConfiguration::with_default_obis();
        mc.set_connection_timeout(60).await;
        mc.set_modem_response_timeout(20).await;

        assert_eq!(mc.connection_timeout().await, 60);
        assert_eq!(mc.modem_response_timeout().await, 20);
    }

    #[tokio::test]
    async fn test_modem_configuration_set_error_control() {
        let mc = ModemConfiguration::with_default_obis();
        mc.set_error_control(ErrorControl::V42).await;
        assert_eq!(mc.error_control().await, ErrorControl::V42);
    }

    #[tokio::test]
    async fn test_modem_configuration_is_connected() {
        let mc = ModemConfiguration::with_default_obis();
        assert!(!mc.is_connected().await);

        mc.connect().await;
        assert!(mc.is_connected().await);

        mc.disconnect().await;
        assert!(!mc.is_connected().await);
    }

    #[tokio::test]
    async fn test_modem_configuration_is_error_control_enabled() {
        let mc = ModemConfiguration::with_default_obis();
        assert!(!mc.is_error_control_enabled().await);

        mc.set_error_control(ErrorControl::V42).await;
        assert!(mc.is_error_control_enabled().await);
    }

    #[tokio::test]
    async fn test_modem_configuration_reset_to_defaults() {
        let mc = ModemConfiguration::with_default_obis();
        mc.set_communication_speed(ModemBaudRate::B57600).await;
        mc.set_connection_timeout(90).await;
        mc.set_error_control(ErrorControl::V42).await;

        mc.reset_to_defaults().await;

        assert_eq!(mc.communication_speed().await, ModemBaudRate::B9600);
        assert_eq!(mc.connection_timeout().await, 30);
        assert_eq!(mc.error_control().await, ErrorControl::None);
    }

    #[tokio::test]
    async fn test_modem_configuration_get_attributes() {
        let mc = ModemConfiguration::with_default_obis();

        // Test communication_speed
        let result = mc.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Unsigned16(speed) => assert_eq!(speed, 9600),
            _ => panic!("Expected Unsigned16"),
        }

        // Test connection_timeout
        let result = mc.get_attribute(5, None).await.unwrap();
        match result {
            DataObject::Unsigned8(timeout) => assert_eq!(timeout, 30),
            _ => panic!("Expected Unsigned8"),
        }

        // Test error_control
        let result = mc.get_attribute(8, None).await.unwrap();
        match result {
            DataObject::Enumerate(ec) => assert_eq!(ec, 0), // None
            _ => panic!("Expected Enumerate"),
        }
    }

    #[tokio::test]
    async fn test_modem_configuration_set_attributes() {
        let mc = ModemConfiguration::with_default_obis();

        mc.set_attribute(2, DataObject::Unsigned16(57600), None)
            .await
            .unwrap();
        assert_eq!(mc.communication_speed().await, ModemBaudRate::B57600);

        mc.set_attribute(3, DataObject::OctetString(b"AT&F".to_vec()), None)
            .await
            .unwrap();
        assert_eq!(mc.modem_initialization_string().await, "AT&F");

        mc.set_attribute(7, DataObject::Boolean(true), None)
            .await
            .unwrap();
        assert!(mc.connection_status().await);
    }

    #[tokio::test]
    async fn test_modem_configuration_read_only_logical_name() {
        let mc = ModemConfiguration::with_default_obis();
        let result = mc
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 29, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_modem_configuration_invalid_attribute() {
        let mc = ModemConfiguration::with_default_obis();
        let result = mc.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_modem_configuration_invalid_method() {
        let mc = ModemConfiguration::with_default_obis();
        let result = mc.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }
}
