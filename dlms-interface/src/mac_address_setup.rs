//! MAC Address Setup interface class (Class ID: 67)
//!
//! The MAC Address Setup interface class manages MAC address configuration
//! for network interfaces in meters.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: mac_address - The MAC address of the interface
//! - Attribute 3: mac_address_enabled - Whether MAC address is enabled

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// MAC Address (48-bit Ethernet address)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacAddr {
    /// 6 octets representing the MAC address
    pub octets: [u8; 6],
}

impl MacAddr {
    /// Create a new MAC address from octets
    pub fn new(octets: [u8; 6]) -> Self {
        Self { octets }
    }

    /// Create from bytes slice
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() == 6 {
            let mut octets = [0u8; 6];
            octets.copy_from_slice(bytes);
            Some(Self { octets })
        } else {
            None
        }
    }

    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.octets.to_vec()
    }

    /// Create a broadcast MAC address
    pub fn broadcast() -> Self {
        Self { octets: [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }
    }

    /// Check if this is a broadcast address
    pub fn is_broadcast(&self) -> bool {
        self.octets == [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    }

    /// Check if this is a multicast address
    pub fn is_multicast(&self) -> bool {
        self.octets[0] & 0x01 == 0x01
    }

    /// Check if this is a unicast address
    pub fn is_unicast(&self) -> bool {
        self.octets[0] & 0x01 == 0x00
    }

    /// Check if this is a locally administered address
    pub fn is_local(&self) -> bool {
        self.octets[0] & 0x02 == 0x02
    }
}

impl Default for MacAddr {
    fn default() -> Self {
        Self { octets: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00] }
    }
}

impl std::fmt::Display for MacAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.octets[0],
            self.octets[1],
            self.octets[2],
            self.octets[3],
            self.octets[4],
            self.octets[5]
        )
    }
}

/// MAC Address Setup interface class (Class ID: 67)
///
/// Default OBIS: 0-0:67.0.0.255
///
/// This class manages MAC address configuration for network interfaces.
#[derive(Debug, Clone)]
pub struct MacAddressSetup {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// MAC address of the interface
    mac_address: Arc<RwLock<MacAddr>>,

    /// Whether MAC address is enabled
    mac_enabled: Arc<RwLock<bool>>,
}

impl MacAddressSetup {
    /// Class ID for MacAddressSetup
    pub const CLASS_ID: u16 = 67;

    /// Default OBIS code for MacAddressSetup (0-0:67.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 67, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_MAC_ADDRESS: u8 = 2;
    pub const ATTR_MAC_ENABLED: u8 = 3;

    /// Create a new MacAddressSetup object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            mac_address: Arc::new(RwLock::new(MacAddr::default())),
            mac_enabled: Arc::new(RwLock::new(true)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get the MAC address
    pub async fn mac_address(&self) -> MacAddr {
        self.mac_address.read().await.clone()
    }

    /// Set the MAC address
    pub async fn set_mac_address(&self, addr: MacAddr) {
        *self.mac_address.write().await = addr;
    }

    /// Set the MAC address from bytes
    pub async fn set_mac_address_from_bytes(&self, bytes: &[u8]) -> DlmsResult<()> {
        if let Some(addr) = MacAddr::from_bytes(bytes) {
            self.set_mac_address(addr).await;
            Ok(())
        } else {
            Err(DlmsError::InvalidData("Invalid MAC address bytes".to_string()))
        }
    }

    /// Get whether MAC address is enabled
    pub async fn mac_enabled(&self) -> bool {
        *self.mac_enabled.read().await
    }

    /// Set whether MAC address is enabled
    pub async fn set_mac_enabled(&self, enabled: bool) {
        *self.mac_enabled.write().await = enabled;
    }

    /// Check if MAC is broadcast
    pub async fn is_broadcast(&self) -> bool {
        self.mac_address().await.is_broadcast()
    }

    /// Check if MAC is multicast
    pub async fn is_multicast(&self) -> bool {
        self.mac_address().await.is_multicast()
    }

    /// Check if MAC is unicast
    pub async fn is_unicast(&self) -> bool {
        self.mac_address().await.is_unicast()
    }

    /// Check if MAC is locally administered
    pub async fn is_local(&self) -> bool {
        self.mac_address().await.is_local()
    }
}

#[async_trait]
impl CosemObject for MacAddressSetup {
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
            Self::ATTR_MAC_ADDRESS => {
                Ok(DataObject::OctetString(self.mac_address().await.to_bytes()))
            }
            Self::ATTR_MAC_ENABLED => {
                Ok(DataObject::Boolean(self.mac_enabled().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "MacAddressSetup has no attribute {}",
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
            Self::ATTR_MAC_ADDRESS => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_mac_address_from_bytes(&bytes).await?;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for mac_address".to_string(),
                    )),
                }
            }
            Self::ATTR_MAC_ENABLED => {
                match value {
                    DataObject::Boolean(enabled) => {
                        self.set_mac_enabled(enabled).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Boolean for mac_enabled".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "MacAddressSetup has no attribute {}",
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
            "MacAddressSetup has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mac_address_setup_class_id() {
        let setup = MacAddressSetup::with_default_obis();
        assert_eq!(setup.class_id(), 67);
    }

    #[tokio::test]
    async fn test_mac_address_setup_obis_code() {
        let setup = MacAddressSetup::with_default_obis();
        assert_eq!(setup.obis_code(), MacAddressSetup::default_obis());
    }

    #[tokio::test]
    async fn test_mac_addr_default() {
        let addr = MacAddr::default();
        assert!(!addr.is_broadcast());
        assert!(addr.is_unicast());
    }

    #[tokio::test]
    async fn test_mac_addr_broadcast() {
        let addr = MacAddr::broadcast();
        assert!(addr.is_broadcast());
        assert!(!addr.is_unicast());
        assert!(addr.is_multicast());
    }

    #[tokio::test]
    async fn test_mac_addr_from_bytes() {
        let bytes = vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        let addr = MacAddr::from_bytes(&bytes);
        assert!(addr.is_some());
        assert_eq!(addr.unwrap().octets, [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
    }

    #[tokio::test]
    async fn test_mac_addr_invalid_bytes() {
        let bytes = vec![0x00, 0x11, 0x22]; // Too short
        let addr = MacAddr::from_bytes(&bytes);
        assert!(addr.is_none());
    }

    #[tokio::test]
    async fn test_mac_addr_is_multicast() {
        let addr = MacAddr::new([0x01, 0x00, 0x5E, 0x00, 0x00, 0x01]);
        assert!(addr.is_multicast());
        assert!(!addr.is_broadcast());
    }

    #[tokio::test]
    async fn test_mac_addr_is_local() {
        let addr = MacAddr::new([0x02, 0x00, 0x5E, 0x00, 0x00, 0x01]);
        assert!(addr.is_local());
    }

    #[tokio::test]
    async fn test_mac_addr_to_bytes() {
        let addr = MacAddr::new([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        let bytes = addr.to_bytes();
        assert_eq!(bytes, vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    }

    #[tokio::test]
    async fn test_mac_addr_display() {
        let addr = MacAddr::new([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        assert_eq!(format!("{}", addr), "AA:BB:CC:DD:EE:FF");
    }

    #[tokio::test]
    async fn test_mac_address_setup_initial_state() {
        let setup = MacAddressSetup::with_default_obis();
        assert!(setup.mac_enabled().await);
        assert!(setup.is_unicast().await);
    }

    #[tokio::test]
    async fn test_mac_address_setup_set_mac_address() {
        let setup = MacAddressSetup::with_default_obis();
        let addr = MacAddr::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        setup.set_mac_address(addr.clone()).await;
        assert_eq!(setup.mac_address().await, addr);
    }

    #[tokio::test]
    async fn test_mac_address_setup_set_mac_address_from_bytes() {
        let setup = MacAddressSetup::with_default_obis();
        let bytes = vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        setup.set_mac_address_from_bytes(&bytes).await.unwrap();
        assert_eq!(setup.mac_address().await.octets, [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
    }

    #[tokio::test]
    async fn test_mac_address_setup_set_mac_enabled() {
        let setup = MacAddressSetup::with_default_obis();
        setup.set_mac_enabled(false).await;
        assert!(!setup.mac_enabled().await);
    }

    #[tokio::test]
    async fn test_mac_address_setup_is_broadcast() {
        let setup = MacAddressSetup::with_default_obis();
        setup.set_mac_address(MacAddr::broadcast()).await;
        assert!(setup.is_broadcast().await);
    }

    #[tokio::test]
    async fn test_mac_address_setup_is_multicast() {
        let setup = MacAddressSetup::with_default_obis();
        setup.set_mac_address(MacAddr::new([0x01, 0x00, 0x5E, 0x00, 0x00, 0x01])).await;
        assert!(setup.is_multicast().await);
    }

    #[tokio::test]
    async fn test_mac_address_setup_get_attributes() {
        let setup = MacAddressSetup::with_default_obis();

        // Test mac_enabled
        let result = setup.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Boolean(enabled) => assert!(enabled),
            _ => panic!("Expected Boolean"),
        }

        // Test mac_address
        let result = setup.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::OctetString(bytes) => assert_eq!(bytes.len(), 6),
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_mac_address_setup_set_attributes() {
        let setup = MacAddressSetup::with_default_obis();

        setup.set_attribute(2, DataObject::OctetString(vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06]), None)
            .await
            .unwrap();
        assert_eq!(setup.mac_address().await.octets, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);

        setup.set_attribute(3, DataObject::Boolean(false), None)
            .await
            .unwrap();
        assert!(!setup.mac_enabled().await);
    }

    #[tokio::test]
    async fn test_mac_address_setup_invalid_mac_bytes() {
        let setup = MacAddressSetup::with_default_obis();
        let result = setup.set_attribute(2, DataObject::OctetString(vec![1, 2, 3]), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mac_address_setup_read_only_logical_name() {
        let setup = MacAddressSetup::with_default_obis();
        let result = setup
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 67, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mac_address_setup_invalid_attribute() {
        let setup = MacAddressSetup::with_default_obis();
        let result = setup.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mac_address_setup_invalid_method() {
        let setup = MacAddressSetup::with_default_obis();
        let result = setup.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mac_address_setup_invalid_data_type() {
        let setup = MacAddressSetup::with_default_obis();
        let result = setup.set_attribute(3, DataObject::Unsigned8(1), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mac_addr_unicast_check() {
        let addr = MacAddr::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert!(addr.is_unicast());
        assert!(!addr.is_multicast());
        assert!(!addr.is_broadcast());
    }

    #[tokio::test]
    async fn test_mac_address_setup_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 67, 0, 0, 1);
        let setup = MacAddressSetup::new(obis);
        assert_eq!(setup.obis_code(), obis);
    }
}
