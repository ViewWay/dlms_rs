//! GSM Controller interface class (Class ID: 28)
//!
//! The GSM Controller interface class manages GSM/GPRS cellular communication for meters.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: gsm_enabled - Whether GSM is enabled
//! - Attribute 3: signal_strength - Current signal strength
//! - Attribute 4: operator_name - Mobile network operator name
//! - Attribute 5: apn - Access Point Name
//! - Attribute 6: connection_status - Current connection status
//! - Attribute 7: imei - International Mobile Equipment Identity
//! - Attribute 8: imsi - International Mobile Subscriber Identity

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Connection Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GsmConnectionStatus {
    /// Not connected
    Disconnected = 0,
    /// Connecting in progress
    Connecting = 1,
    /// Connected
    Connected = 2,
    /// Connection failed
    ConnectionFailed = 3,
    /// Roaming
    Roaming = 4,
    /// Searching for network
    Searching = 5,
}

impl GsmConnectionStatus {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Disconnected,
            1 => Self::Connecting,
            2 => Self::Connected,
            3 => Self::ConnectionFailed,
            4 => Self::Roaming,
            5 => Self::Searching,
            _ => Self::Disconnected,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if connected
    pub fn is_connected(self) -> bool {
        matches!(self, Self::Connected | Self::Roaming)
    }

    /// Check if connection is in progress
    pub fn is_connecting(self) -> bool {
        matches!(self, Self::Connecting | Self::Searching)
    }
}

/// Signal Strength
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SignalStrength {
    /// No signal
    NoSignal = 0,
    /// Very weak signal (< -100 dBm)
    VeryWeak = 1,
    /// Weak signal (-100 to -95 dBm)
    Weak = 2,
    /// Moderate signal (-95 to -85 dBm)
    Moderate = 3,
    /// Good signal (-85 to -75 dBm)
    Good = 4,
    /// Excellent signal (> -75 dBm)
    Excellent = 5,
    /// Unknown signal strength
    Unknown = 255,
}

impl SignalStrength {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::NoSignal,
            1 => Self::VeryWeak,
            2 => Self::Weak,
            3 => Self::Moderate,
            4 => Self::Good,
            5 => Self::Excellent,
            255 => Self::Unknown,
            _ => Self::Unknown,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if signal is usable
    pub fn is_usable(self) -> bool {
        matches!(self, Self::Moderate | Self::Good | Self::Excellent)
    }
}

/// GSM Controller interface class (Class ID: 28)
///
/// Default OBIS: 0-0:28.0.0.255
///
/// This class manages GSM/GPRS cellular communication for meters.
#[derive(Debug, Clone)]
pub struct GsmController {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Whether GSM is enabled
    gsm_enabled: Arc<RwLock<bool>>,

    /// Current signal strength
    signal_strength: Arc<RwLock<SignalStrength>>,

    /// Mobile network operator name
    operator_name: Arc<RwLock<String>>,

    /// Access Point Name
    apn: Arc<RwLock<String>>,

    /// Connection status
    connection_status: Arc<RwLock<GsmConnectionStatus>>,

    /// IMEI (International Mobile Equipment Identity)
    imei: Arc<RwLock<String>>,

    /// IMSI (International Mobile Subscriber Identity)
    imsi: Arc<RwLock<String>>,
}

impl GsmController {
    /// Class ID for GsmController
    pub const CLASS_ID: u16 = 28;

    /// Default OBIS code for GsmController (0-0:28.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 28, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_GSM_ENABLED: u8 = 2;
    pub const ATTR_SIGNAL_STRENGTH: u8 = 3;
    pub const ATTR_OPERATOR_NAME: u8 = 4;
    pub const ATTR_APN: u8 = 5;
    pub const ATTR_CONNECTION_STATUS: u8 = 6;
    pub const ATTR_IMEI: u8 = 7;
    pub const ATTR_IMSI: u8 = 8;

    /// Create a new GsmController object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            gsm_enabled: Arc::new(RwLock::new(false)),
            signal_strength: Arc::new(RwLock::new(SignalStrength::Unknown)),
            operator_name: Arc::new(RwLock::new(String::new())),
            apn: Arc::new(RwLock::new(String::new())),
            connection_status: Arc::new(RwLock::new(GsmConnectionStatus::Disconnected)),
            imei: Arc::new(RwLock::new(String::new())),
            imsi: Arc::new(RwLock::new(String::new())),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get whether GSM is enabled
    pub async fn gsm_enabled(&self) -> bool {
        *self.gsm_enabled.read().await
    }

    /// Set whether GSM is enabled
    pub async fn set_gsm_enabled(&self, enabled: bool) {
        *self.gsm_enabled.write().await = enabled;
    }

    /// Get the signal strength
    pub async fn signal_strength(&self) -> SignalStrength {
        *self.signal_strength.read().await
    }

    /// Set the signal strength
    pub async fn set_signal_strength(&self, strength: SignalStrength) {
        *self.signal_strength.write().await = strength;
    }

    /// Get the operator name
    pub async fn operator_name(&self) -> String {
        self.operator_name.read().await.clone()
    }

    /// Set the operator name
    pub async fn set_operator_name(&self, name: String) {
        *self.operator_name.write().await = name;
    }

    /// Get the APN
    pub async fn apn(&self) -> String {
        self.apn.read().await.clone()
    }

    /// Set the APN
    pub async fn set_apn(&self, apn: String) {
        *self.apn.write().await = apn;
    }

    /// Get the connection status
    pub async fn connection_status(&self) -> GsmConnectionStatus {
        *self.connection_status.read().await
    }

    /// Set the connection status
    pub async fn set_connection_status(&self, status: GsmConnectionStatus) {
        *self.connection_status.write().await = status;
    }

    /// Get the IMEI
    pub async fn imei(&self) -> String {
        self.imei.read().await.clone()
    }

    /// Set the IMEI
    pub async fn set_imei(&self, imei: String) {
        *self.imei.write().await = imei;
    }

    /// Get the IMSI
    pub async fn imsi(&self) -> String {
        self.imsi.read().await.clone()
    }

    /// Set the IMSI
    pub async fn set_imsi(&self, imsi: String) {
        *self.imsi.write().await = imsi;
    }

    /// Check if GSM is enabled
    pub async fn is_enabled(&self) -> bool {
        self.gsm_enabled().await
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        self.connection_status().await.is_connected()
    }

    /// Check if signal is usable
    pub async fn is_signal_usable(&self) -> bool {
        self.signal_strength().await.is_usable()
    }

    /// Check if connecting
    pub async fn is_connecting(&self) -> bool {
        self.connection_status().await.is_connecting()
    }

    /// Connect to GSM network
    pub async fn connect(&self) {
        *self.connection_status.write().await = GsmConnectionStatus::Connecting;
    }

    /// Disconnect from GSM network
    pub async fn disconnect(&self) {
        *self.connection_status.write().await = GsmConnectionStatus::Disconnected;
    }

    /// Mark as connected
    pub async fn mark_connected(&self) {
        *self.connection_status.write().await = GsmConnectionStatus::Connected;
    }

    /// Mark as roaming
    pub async fn mark_roaming(&self) {
        *self.connection_status.write().await = GsmConnectionStatus::Roaming;
    }

    /// Mark connection as failed
    pub async fn mark_connection_failed(&self) {
        *self.connection_status.write().await = GsmConnectionStatus::ConnectionFailed;
    }

    /// Search for network
    pub async fn search_network(&self) {
        *self.connection_status.write().await = GsmConnectionStatus::Searching;
    }

    /// Validate IMEI format (basic check - should be 15 digits)
    pub async fn validate_imei(&self) -> bool {
        let imei = self.imei().await;
        imei.len() == 15 && imei.chars().all(|c| c.is_ascii_digit())
    }

    /// Validate IMSI format (basic check - should be up to 15 digits)
    pub async fn validate_imsi(&self) -> bool {
        let imsi = self.imsi().await;
        !imsi.is_empty() && imsi.len() <= 15 && imsi.chars().all(|c| c.is_ascii_digit())
    }
}

#[async_trait]
impl CosemObject for GsmController {
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
            Self::ATTR_GSM_ENABLED => {
                Ok(DataObject::Boolean(self.gsm_enabled().await))
            }
            Self::ATTR_SIGNAL_STRENGTH => {
                Ok(DataObject::Enumerate(self.signal_strength().await.to_u8()))
            }
            Self::ATTR_OPERATOR_NAME => {
                Ok(DataObject::OctetString(self.operator_name().await.into_bytes()))
            }
            Self::ATTR_APN => {
                Ok(DataObject::OctetString(self.apn().await.into_bytes()))
            }
            Self::ATTR_CONNECTION_STATUS => {
                Ok(DataObject::Enumerate(self.connection_status().await.to_u8()))
            }
            Self::ATTR_IMEI => {
                Ok(DataObject::OctetString(self.imei().await.into_bytes()))
            }
            Self::ATTR_IMSI => {
                Ok(DataObject::OctetString(self.imsi().await.into_bytes()))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "GsmController has no attribute {}",
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
            Self::ATTR_GSM_ENABLED => {
                match value {
                    DataObject::Boolean(enabled) => {
                        self.set_gsm_enabled(enabled).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Boolean for gsm_enabled".to_string(),
                    )),
                }
            }
            Self::ATTR_SIGNAL_STRENGTH => {
                match value {
                    DataObject::Enumerate(strength) => {
                        self.set_signal_strength(SignalStrength::from_u8(strength)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for signal_strength".to_string(),
                    )),
                }
            }
            Self::ATTR_OPERATOR_NAME => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let name = String::from_utf8_lossy(&bytes).to_string();
                        self.set_operator_name(name).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for operator_name".to_string(),
                    )),
                }
            }
            Self::ATTR_APN => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let apn = String::from_utf8_lossy(&bytes).to_string();
                        self.set_apn(apn).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for apn".to_string(),
                    )),
                }
            }
            Self::ATTR_CONNECTION_STATUS => {
                match value {
                    DataObject::Enumerate(status) => {
                        self.set_connection_status(GsmConnectionStatus::from_u8(status)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for connection_status".to_string(),
                    )),
                }
            }
            Self::ATTR_IMEI => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let imei = String::from_utf8_lossy(&bytes).to_string();
                        self.set_imei(imei).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for imei".to_string(),
                    )),
                }
            }
            Self::ATTR_IMSI => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let imsi = String::from_utf8_lossy(&bytes).to_string();
                        self.set_imsi(imsi).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for imsi".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "GsmController has no attribute {}",
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
            "GsmController has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gsm_controller_class_id() {
        let gsm = GsmController::with_default_obis();
        assert_eq!(gsm.class_id(), 28);
    }

    #[tokio::test]
    async fn test_gsm_controller_obis_code() {
        let gsm = GsmController::with_default_obis();
        assert_eq!(gsm.obis_code(), GsmController::default_obis());
    }

    #[tokio::test]
    async fn test_gsm_connection_status_from_u8() {
        assert_eq!(GsmConnectionStatus::from_u8(0), GsmConnectionStatus::Disconnected);
        assert_eq!(GsmConnectionStatus::from_u8(1), GsmConnectionStatus::Connecting);
        assert_eq!(GsmConnectionStatus::from_u8(2), GsmConnectionStatus::Connected);
        assert_eq!(GsmConnectionStatus::from_u8(3), GsmConnectionStatus::ConnectionFailed);
        assert_eq!(GsmConnectionStatus::from_u8(4), GsmConnectionStatus::Roaming);
        assert_eq!(GsmConnectionStatus::from_u8(5), GsmConnectionStatus::Searching);
    }

    #[tokio::test]
    async fn test_gsm_connection_status_is_connected() {
        assert!(GsmConnectionStatus::Connected.is_connected());
        assert!(GsmConnectionStatus::Roaming.is_connected());
        assert!(!GsmConnectionStatus::Disconnected.is_connected());
        assert!(!GsmConnectionStatus::Connecting.is_connected());
        assert!(!GsmConnectionStatus::ConnectionFailed.is_connected());
        assert!(!GsmConnectionStatus::Searching.is_connected());
    }

    #[tokio::test]
    async fn test_gsm_connection_status_is_connecting() {
        assert!(GsmConnectionStatus::Connecting.is_connecting());
        assert!(GsmConnectionStatus::Searching.is_connecting());
        assert!(!GsmConnectionStatus::Connected.is_connecting());
        assert!(!GsmConnectionStatus::Disconnected.is_connecting());
    }

    #[tokio::test]
    async fn test_signal_strength_from_u8() {
        assert_eq!(SignalStrength::from_u8(0), SignalStrength::NoSignal);
        assert_eq!(SignalStrength::from_u8(1), SignalStrength::VeryWeak);
        assert_eq!(SignalStrength::from_u8(2), SignalStrength::Weak);
        assert_eq!(SignalStrength::from_u8(3), SignalStrength::Moderate);
        assert_eq!(SignalStrength::from_u8(4), SignalStrength::Good);
        assert_eq!(SignalStrength::from_u8(5), SignalStrength::Excellent);
        assert_eq!(SignalStrength::from_u8(255), SignalStrength::Unknown);
    }

    #[tokio::test]
    async fn test_signal_strength_is_usable() {
        assert!(SignalStrength::Moderate.is_usable());
        assert!(SignalStrength::Good.is_usable());
        assert!(SignalStrength::Excellent.is_usable());
        assert!(!SignalStrength::NoSignal.is_usable());
        assert!(!SignalStrength::VeryWeak.is_usable());
        assert!(!SignalStrength::Weak.is_usable());
        assert!(!SignalStrength::Unknown.is_usable());
    }

    #[tokio::test]
    async fn test_gsm_controller_initial_state() {
        let gsm = GsmController::with_default_obis();
        assert!(!gsm.gsm_enabled().await);
        assert_eq!(gsm.signal_strength().await, SignalStrength::Unknown);
        assert_eq!(gsm.operator_name().await, "");
        assert_eq!(gsm.apn().await, "");
        assert_eq!(gsm.connection_status().await, GsmConnectionStatus::Disconnected);
        assert_eq!(gsm.imei().await, "");
        assert_eq!(gsm.imsi().await, "");
    }

    #[tokio::test]
    async fn test_gsm_controller_set_gsm_enabled() {
        let gsm = GsmController::with_default_obis();
        gsm.set_gsm_enabled(true).await;
        assert!(gsm.gsm_enabled().await);
    }

    #[tokio::test]
    async fn test_gsm_controller_set_signal_strength() {
        let gsm = GsmController::with_default_obis();
        gsm.set_signal_strength(SignalStrength::Excellent).await;
        assert_eq!(gsm.signal_strength().await, SignalStrength::Excellent);
    }

    #[tokio::test]
    async fn test_gsm_controller_set_operator_name() {
        let gsm = GsmController::with_default_obis();
        gsm.set_operator_name("Vodafone".to_string()).await;
        assert_eq!(gsm.operator_name().await, "Vodafone");
    }

    #[tokio::test]
    async fn test_gsm_controller_set_apn() {
        let gsm = GsmController::with_default_obis();
        gsm.set_apn("internet.com".to_string()).await;
        assert_eq!(gsm.apn().await, "internet.com");
    }

    #[tokio::test]
    async fn test_gsm_controller_set_imei() {
        let gsm = GsmController::with_default_obis();
        gsm.set_imei("123456789012345".to_string()).await;
        assert_eq!(gsm.imei().await, "123456789012345");
    }

    #[tokio::test]
    async fn test_gsm_controller_set_imsi() {
        let gsm = GsmController::with_default_obis();
        gsm.set_imsi("310260123456789".to_string()).await;
        assert_eq!(gsm.imsi().await, "310260123456789");
    }

    #[tokio::test]
    async fn test_gsm_controller_is_connected() {
        let gsm = GsmController::with_default_obis();

        assert!(!gsm.is_connected().await);

        gsm.set_connection_status(GsmConnectionStatus::Connected).await;
        assert!(gsm.is_connected().await);

        gsm.set_connection_status(GsmConnectionStatus::Roaming).await;
        assert!(gsm.is_connected().await);
    }

    #[tokio::test]
    async fn test_gsm_controller_is_signal_usable() {
        let gsm = GsmController::with_default_obis();

        assert!(!gsm.is_signal_usable().await);

        gsm.set_signal_strength(SignalStrength::Good).await;
        assert!(gsm.is_signal_usable().await);
    }

    #[tokio::test]
    async fn test_gsm_controller_connect() {
        let gsm = GsmController::with_default_obis();
        gsm.connect().await;
        assert_eq!(gsm.connection_status().await, GsmConnectionStatus::Connecting);
    }

    #[tokio::test]
    async fn test_gsm_controller_disconnect() {
        let gsm = GsmController::with_default_obis();
        gsm.mark_connected().await;
        gsm.disconnect().await;
        assert_eq!(gsm.connection_status().await, GsmConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn test_gsm_controller_mark_connected() {
        let gsm = GsmController::with_default_obis();
        gsm.mark_connected().await;
        assert_eq!(gsm.connection_status().await, GsmConnectionStatus::Connected);
    }

    #[tokio::test]
    async fn test_gsm_controller_mark_roaming() {
        let gsm = GsmController::with_default_obis();
        gsm.mark_roaming().await;
        assert_eq!(gsm.connection_status().await, GsmConnectionStatus::Roaming);
    }

    #[tokio::test]
    async fn test_gsm_controller_mark_connection_failed() {
        let gsm = GsmController::with_default_obis();
        gsm.mark_connection_failed().await;
        assert_eq!(gsm.connection_status().await, GsmConnectionStatus::ConnectionFailed);
    }

    #[tokio::test]
    async fn test_gsm_controller_search_network() {
        let gsm = GsmController::with_default_obis();
        gsm.search_network().await;
        assert_eq!(gsm.connection_status().await, GsmConnectionStatus::Searching);
    }

    #[tokio::test]
    async fn test_gsm_controller_validate_imei() {
        let gsm = GsmController::with_default_obis();

        assert!(!gsm.validate_imei().await); // Empty

        gsm.set_imei("123456789012345".to_string()).await; // 15 digits
        assert!(gsm.validate_imei().await);

        gsm.set_imei("12345".to_string()).await; // Too short
        assert!(!gsm.validate_imei().await);
    }

    #[tokio::test]
    async fn test_gsm_controller_validate_imsi() {
        let gsm = GsmController::with_default_obis();

        assert!(!gsm.validate_imsi().await); // Empty

        gsm.set_imsi("310260123456789".to_string()).await; // 15 digits
        assert!(gsm.validate_imsi().await);

        gsm.set_imsi("12345".to_string()).await; // 5 digits
        assert!(gsm.validate_imsi().await);
    }

    #[tokio::test]
    async fn test_gsm_controller_get_attributes() {
        let gsm = GsmController::with_default_obis();

        // Test gsm_enabled
        let result = gsm.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Boolean(enabled) => assert!(!enabled),
            _ => panic!("Expected Boolean"),
        }

        // Test signal_strength
        let result = gsm.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Enumerate(strength) => assert_eq!(strength, 255), // Unknown
            _ => panic!("Expected Enumerate"),
        }

        // Test connection_status
        let result = gsm.get_attribute(6, None).await.unwrap();
        match result {
            DataObject::Enumerate(status) => assert_eq!(status, 0), // Disconnected
            _ => panic!("Expected Enumerate"),
        }
    }

    #[tokio::test]
    async fn test_gsm_controller_set_attributes() {
        let gsm = GsmController::with_default_obis();

        gsm.set_attribute(2, DataObject::Boolean(true), None)
            .await
            .unwrap();
        assert!(gsm.gsm_enabled().await);

        gsm.set_attribute(3, DataObject::Enumerate(4), None) // Good
            .await
            .unwrap();
        assert_eq!(gsm.signal_strength().await, SignalStrength::Good);

        gsm.set_attribute(5, DataObject::OctetString(b"apn.example.com".to_vec()), None)
            .await
            .unwrap();
        assert_eq!(gsm.apn().await, "apn.example.com");
    }

    #[tokio::test]
    async fn test_gsm_controller_read_only_logical_name() {
        let gsm = GsmController::with_default_obis();
        let result = gsm
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 28, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_gsm_controller_invalid_attribute() {
        let gsm = GsmController::with_default_obis();
        let result = gsm.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_gsm_controller_invalid_method() {
        let gsm = GsmController::with_default_obis();
        let result = gsm.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }
}
