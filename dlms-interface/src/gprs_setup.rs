//! GPRS Setup interface class (Class ID: 63)
//!
//! The GPRS Setup interface class manages GPRS network configuration for smart meters.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: apn - Access Point Name
//! - Attribute 3: pin - SIM PIN code
//! - Attribute 4: allowed_connections - Maximum allowed connections
//! - Attribute 5: quality_of_service - Quality of Service settings
//! - Attribute 6: enabled - Whether GPRS is enabled

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Quality of Service settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum QualityOfService {
    /// Best effort (no guarantee)
    BestEffort = 0,
    /// Low priority
    Low = 1,
    /// Normal priority
    Normal = 2,
    /// High priority
    High = 3,
}

impl QualityOfService {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::BestEffort,
            1 => Self::Low,
            2 => Self::Normal,
            3 => Self::High,
            _ => Self::BestEffort,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// GPRS Setup interface class (Class ID: 63)
///
/// Default OBIS: 0-0:63.0.0.255
///
/// This class manages GPRS network configuration for smart meters.
#[derive(Debug, Clone)]
pub struct GprsSetup {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Access Point Name
    apn: Arc<RwLock<String>>,

    /// SIM PIN code (stored as string for security)
    pin: Arc<RwLock<Option<String>>>,

    /// Maximum allowed connections
    allowed_connections: Arc<RwLock<u8>>,

    /// Quality of Service settings
    quality_of_service: Arc<RwLock<QualityOfService>>,

    /// Whether GPRS is enabled
    enabled: Arc<RwLock<bool>>,

    /// Current connection count
    connection_count: Arc<RwLock<u8>>,

    /// Username for APN authentication
    username: Arc<RwLock<Option<String>>>,

    /// Password for APN authentication
    password: Arc<RwLock<Option<String>>>,
}

impl GprsSetup {
    /// Class ID for GprsSetup
    pub const CLASS_ID: u16 = 63;

    /// Default OBIS code for GprsSetup (0-0:63.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 63, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_APN: u8 = 2;
    pub const ATTR_PIN: u8 = 3;
    pub const ATTR_ALLOWED_CONNECTIONS: u8 = 4;
    pub const ATTR_QUALITY_OF_SERVICE: u8 = 5;
    pub const ATTR_ENABLED: u8 = 6;

    /// Create a new GprsSetup object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            apn: Arc::new(RwLock::new(String::new())),
            pin: Arc::new(RwLock::new(None)),
            allowed_connections: Arc::new(RwLock::new(1)),
            quality_of_service: Arc::new(RwLock::new(QualityOfService::Normal)),
            enabled: Arc::new(RwLock::new(false)),
            connection_count: Arc::new(RwLock::new(0)),
            username: Arc::new(RwLock::new(None)),
            password: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Create with APN
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `apn` - Access Point Name
    pub fn with_apn(logical_name: ObisCode, apn: String) -> Self {
        Self {
            logical_name,
            apn: Arc::new(RwLock::new(apn)),
            pin: Arc::new(RwLock::new(None)),
            allowed_connections: Arc::new(RwLock::new(1)),
            quality_of_service: Arc::new(RwLock::new(QualityOfService::Normal)),
            enabled: Arc::new(RwLock::new(false)),
            connection_count: Arc::new(RwLock::new(0)),
            username: Arc::new(RwLock::new(None)),
            password: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the APN
    pub async fn apn(&self) -> String {
        self.apn.read().await.clone()
    }

    /// Set the APN
    pub async fn set_apn(&self, apn: String) {
        *self.apn.write().await = apn;
    }

    /// Get the PIN
    pub async fn pin(&self) -> Option<String> {
        self.pin.read().await.clone()
    }

    /// Set the PIN
    pub async fn set_pin(&self, pin: String) {
        *self.pin.write().await = Some(pin);
    }

    /// Clear the PIN
    pub async fn clear_pin(&self) {
        *self.pin.write().await = None;
    }

    /// Get the allowed connections
    pub async fn allowed_connections(&self) -> u8 {
        *self.allowed_connections.read().await
    }

    /// Set the allowed connections
    pub async fn set_allowed_connections(&self, count: u8) {
        *self.allowed_connections.write().await = count;
    }

    /// Get the quality of service
    pub async fn quality_of_service(&self) -> QualityOfService {
        *self.quality_of_service.read().await
    }

    /// Set the quality of service
    pub async fn set_quality_of_service(&self, qos: QualityOfService) {
        *self.quality_of_service.write().await = qos;
    }

    /// Get the enabled status
    pub async fn enabled(&self) -> bool {
        *self.enabled.read().await
    }

    /// Set the enabled status
    pub async fn set_enabled(&self, enabled: bool) {
        *self.enabled.write().await = enabled;
    }

    /// Enable GPRS
    pub async fn enable(&self) {
        self.set_enabled(true).await;
    }

    /// Disable GPRS
    pub async fn disable(&self) {
        self.set_enabled(false).await;
    }

    /// Get the current connection count
    pub async fn connection_count(&self) -> u8 {
        *self.connection_count.read().await
    }

    /// Increment connection count
    pub async fn increment_connection_count(&self) {
        *self.connection_count.write().await += 1;
    }

    /// Decrement connection count
    pub async fn decrement_connection_count(&self) {
        let count = self.connection_count().await;
        if count > 0 {
            *self.connection_count.write().await = count - 1;
        }
    }

    /// Get the username
    pub async fn username(&self) -> Option<String> {
        self.username.read().await.clone()
    }

    /// Set the username
    pub async fn set_username(&self, username: String) {
        *self.username.write().await = Some(username);
    }

    /// Clear the username
    pub async fn clear_username(&self) {
        *self.username.write().await = None;
    }

    /// Get the password
    pub async fn password(&self) -> Option<String> {
        self.password.read().await.clone()
    }

    /// Set the password
    pub async fn set_password(&self, password: String) {
        *self.password.write().await = Some(password);
    }

    /// Clear the password
    pub async fn clear_password(&self) {
        *self.password.write().await = None;
    }

    /// Check if a new connection can be established
    pub async fn can_connect(&self) -> bool {
        self.enabled().await
            && self.connection_count().await < self.allowed_connections().await
    }

    /// Check if credentials are configured
    pub async fn has_credentials(&self) -> bool {
        !self.apn().await.is_empty()
    }

    /// Check if APN authentication is configured
    pub async fn has_auth(&self) -> bool {
        self.username().await.is_some() && self.password().await.is_some()
    }

    /// Get the maximum allowed connections
    pub async fn max_connections(&self) -> u8 {
        self.allowed_connections().await
    }

    /// Check if at maximum connections
    pub async fn is_at_max_connections(&self) -> bool {
        self.connection_count().await >= self.allowed_connections().await
    }

    /// Get available connection slots
    pub async fn available_connections(&self) -> u8 {
        let max = self.allowed_connections().await;
        let current = self.connection_count().await;
        max.saturating_sub(current)
    }
}

#[async_trait]
impl CosemObject for GprsSetup {
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
            Self::ATTR_APN => {
                Ok(DataObject::OctetString(self.apn().await.into_bytes()))
            }
            Self::ATTR_PIN => {
                match self.pin().await {
                    Some(pin) => Ok(DataObject::OctetString(pin.into_bytes())),
                    None => Ok(DataObject::OctetString(Vec::new())),
                }
            }
            Self::ATTR_ALLOWED_CONNECTIONS => {
                Ok(DataObject::Unsigned8(self.allowed_connections().await))
            }
            Self::ATTR_QUALITY_OF_SERVICE => {
                Ok(DataObject::Enumerate(self.quality_of_service().await.to_u8()))
            }
            Self::ATTR_ENABLED => {
                Ok(DataObject::Boolean(self.enabled().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "GprsSetup has no attribute {}",
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
            Self::ATTR_APN => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_apn(String::from_utf8_lossy(&bytes).to_string()).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for apn".to_string(),
                    )),
                }
            }
            Self::ATTR_PIN => {
                match value {
                    DataObject::OctetString(bytes) => {
                        if bytes.is_empty() {
                            self.clear_pin().await;
                        } else {
                            self.set_pin(String::from_utf8_lossy(&bytes).to_string()).await;
                        }
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for pin".to_string(),
                    )),
                }
            }
            Self::ATTR_ALLOWED_CONNECTIONS => {
                match value {
                    DataObject::Unsigned8(count) => {
                        self.set_allowed_connections(count).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for allowed_connections".to_string(),
                    )),
                }
            }
            Self::ATTR_QUALITY_OF_SERVICE => {
                match value {
                    DataObject::Enumerate(qos) => {
                        self.set_quality_of_service(QualityOfService::from_u8(qos)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for quality_of_service".to_string(),
                    )),
                }
            }
            Self::ATTR_ENABLED => {
                match value {
                    DataObject::Boolean(enabled) => {
                        self.set_enabled(enabled).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Boolean for enabled".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "GprsSetup has no attribute {}",
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
            "GprsSetup has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gprs_setup_class_id() {
        let gprs = GprsSetup::with_default_obis();
        assert_eq!(gprs.class_id(), 63);
    }

    #[tokio::test]
    async fn test_gprs_setup_obis_code() {
        let gprs = GprsSetup::with_default_obis();
        assert_eq!(gprs.obis_code(), GprsSetup::default_obis());
    }

    #[tokio::test]
    async fn test_gprs_setup_initial_state() {
        let gprs = GprsSetup::with_default_obis();
        assert!(!gprs.enabled().await);
        assert_eq!(gprs.apn().await, "");
        assert_eq!(gprs.pin().await, None);
        assert_eq!(gprs.allowed_connections().await, 1);
        assert_eq!(gprs.quality_of_service().await, QualityOfService::Normal);
        assert_eq!(gprs.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_gprs_setup_with_apn() {
        let gprs = GprsSetup::with_apn(ObisCode::new(0, 0, 63, 0, 0, 255), "internet.example.com".to_string());
        assert_eq!(gprs.apn().await, "internet.example.com");
    }

    #[tokio::test]
    async fn test_gprs_setup_set_apn() {
        let gprs = GprsSetup::with_default_obis();
        gprs.set_apn("internet.example.com".to_string()).await;
        assert_eq!(gprs.apn().await, "internet.example.com");
    }

    #[tokio::test]
    async fn test_gprs_setup_set_pin() {
        let gprs = GprsSetup::with_default_obis();
        gprs.set_pin("1234".to_string()).await;
        assert_eq!(gprs.pin().await, Some("1234".to_string()));
    }

    #[tokio::test]
    async fn test_gprs_setup_clear_pin() {
        let gprs = GprsSetup::with_default_obis();
        gprs.set_pin("1234".to_string()).await;
        gprs.clear_pin().await;
        assert_eq!(gprs.pin().await, None);
    }

    #[tokio::test]
    async fn test_gprs_setup_set_allowed_connections() {
        let gprs = GprsSetup::with_default_obis();
        gprs.set_allowed_connections(5).await;
        assert_eq!(gprs.allowed_connections().await, 5);
    }

    #[tokio::test]
    async fn test_gprs_setup_set_quality_of_service() {
        let gprs = GprsSetup::with_default_obis();
        gprs.set_quality_of_service(QualityOfService::High).await;
        assert_eq!(gprs.quality_of_service().await, QualityOfService::High);
    }

    #[tokio::test]
    async fn test_gprs_setup_enable_disable() {
        let gprs = GprsSetup::with_default_obis();
        assert!(!gprs.enabled().await);

        gprs.enable().await;
        assert!(gprs.enabled().await);

        gprs.disable().await;
        assert!(!gprs.enabled().await);
    }

    #[tokio::test]
    async fn test_gprs_setup_increment_connection_count() {
        let gprs = GprsSetup::with_default_obis();
        assert_eq!(gprs.connection_count().await, 0);

        gprs.increment_connection_count().await;
        assert_eq!(gprs.connection_count().await, 1);

        gprs.increment_connection_count().await;
        assert_eq!(gprs.connection_count().await, 2);
    }

    #[tokio::test]
    async fn test_gprs_setup_decrement_connection_count() {
        let gprs = GprsSetup::with_default_obis();
        gprs.increment_connection_count().await;
        gprs.increment_connection_count().await;
        assert_eq!(gprs.connection_count().await, 2);

        gprs.decrement_connection_count().await;
        assert_eq!(gprs.connection_count().await, 1);
    }

    #[tokio::test]
    async fn test_gprs_setup_decrement_connection_count_no_underflow() {
        let gprs = GprsSetup::with_default_obis();
        gprs.decrement_connection_count().await;
        assert_eq!(gprs.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_gprs_setup_set_username_password() {
        let gprs = GprsSetup::with_default_obis();
        gprs.set_username("user".to_string()).await;
        gprs.set_password("pass".to_string()).await;
        assert_eq!(gprs.username().await, Some("user".to_string()));
        assert_eq!(gprs.password().await, Some("pass".to_string()));
    }

    #[tokio::test]
    async fn test_gprs_setup_clear_username_password() {
        let gprs = GprsSetup::with_default_obis();
        gprs.set_username("user".to_string()).await;
        gprs.set_password("pass".to_string()).await;
        gprs.clear_username().await;
        gprs.clear_password().await;
        assert_eq!(gprs.username().await, None);
        assert_eq!(gprs.password().await, None);
    }

    #[tokio::test]
    async fn test_gprs_setup_can_connect() {
        let gprs = GprsSetup::with_default_obis();
        gprs.enable().await;
        gprs.set_allowed_connections(2).await;
        assert!(gprs.can_connect().await);

        gprs.increment_connection_count().await;
        assert!(gprs.can_connect().await);

        gprs.increment_connection_count().await;
        assert!(!gprs.can_connect().await);
    }

    #[tokio::test]
    async fn test_gprs_setup_can_connect_when_disabled() {
        let gprs = GprsSetup::with_default_obis();
        // Don't enable
        assert!(!gprs.can_connect().await);
    }

    #[tokio::test]
    async fn test_gprs_setup_has_credentials() {
        let gprs = GprsSetup::with_default_obis();
        assert!(!gprs.has_credentials().await);

        gprs.set_apn("internet.example.com".to_string()).await;
        assert!(gprs.has_credentials().await);
    }

    #[tokio::test]
    async fn test_gprs_setup_has_auth() {
        let gprs = GprsSetup::with_default_obis();
        assert!(!gprs.has_auth().await);

        gprs.set_username("user".to_string()).await;
        gprs.set_password("pass".to_string()).await;
        assert!(gprs.has_auth().await);
    }

    #[tokio::test]
    async fn test_gprs_setup_is_at_max_connections() {
        let gprs = GprsSetup::with_default_obis();
        gprs.set_allowed_connections(2).await;
        assert!(!gprs.is_at_max_connections().await);

        gprs.increment_connection_count().await;
        gprs.increment_connection_count().await;
        assert!(gprs.is_at_max_connections().await);
    }

    #[tokio::test]
    async fn test_gprs_setup_available_connections() {
        let gprs = GprsSetup::with_default_obis();
        gprs.set_allowed_connections(5).await;
        assert_eq!(gprs.available_connections().await, 5);

        gprs.increment_connection_count().await;
        gprs.increment_connection_count().await;
        assert_eq!(gprs.available_connections().await, 3);
    }

    #[tokio::test]
    async fn test_quality_of_service_from_u8() {
        assert_eq!(QualityOfService::from_u8(0), QualityOfService::BestEffort);
        assert_eq!(QualityOfService::from_u8(1), QualityOfService::Low);
        assert_eq!(QualityOfService::from_u8(2), QualityOfService::Normal);
        assert_eq!(QualityOfService::from_u8(3), QualityOfService::High);
    }

    #[tokio::test]
    async fn test_gprs_setup_get_attributes() {
        let gprs = GprsSetup::with_default_obis();

        // Test apn
        let result = gprs.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::OctetString(bytes) => assert!(bytes.is_empty()),
            _ => panic!("Expected OctetString"),
        }

        // Test allowed_connections
        let result = gprs.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Unsigned8(count) => assert_eq!(count, 1),
            _ => panic!("Expected Unsigned8"),
        }

        // Test quality_of_service
        let result = gprs.get_attribute(5, None).await.unwrap();
        match result {
            DataObject::Enumerate(qos) => assert_eq!(qos, 2), // Normal
            _ => panic!("Expected Enumerate"),
        }
    }

    #[tokio::test]
    async fn test_gprs_setup_set_attributes() {
        let gprs = GprsSetup::with_default_obis();

        gprs.set_attribute(2, DataObject::OctetString(b"internet.example.com".to_vec()), None)
            .await
            .unwrap();
        assert_eq!(gprs.apn().await, "internet.example.com");

        gprs.set_attribute(4, DataObject::Unsigned8(3), None)
            .await
            .unwrap();
        assert_eq!(gprs.allowed_connections().await, 3);

        gprs.set_attribute(6, DataObject::Boolean(true), None)
            .await
            .unwrap();
        assert!(gprs.enabled().await);
    }

    #[tokio::test]
    async fn test_gprs_setup_set_pin_empty_clears() {
        let gprs = GprsSetup::with_default_obis();
        gprs.set_pin("1234".to_string()).await;
        gprs.set_attribute(3, DataObject::OctetString(Vec::new()), None)
            .await
            .unwrap();
        assert_eq!(gprs.pin().await, None);
    }

    #[tokio::test]
    async fn test_gprs_setup_read_only_logical_name() {
        let gprs = GprsSetup::with_default_obis();
        let result = gprs
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 63, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_gprs_setup_invalid_attribute() {
        let gprs = GprsSetup::with_default_obis();
        let result = gprs.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_gprs_setup_invalid_method() {
        let gprs = GprsSetup::with_default_obis();
        let result = gprs.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_gprs_setup_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 63, 0, 0, 1);
        let gprs = GprsSetup::new(obis);
        assert_eq!(gprs.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_gprs_setup_max_connections() {
        let gprs = GprsSetup::with_default_obis();
        gprs.set_allowed_connections(10).await;
        assert_eq!(gprs.max_connections().await, 10);
    }
}
