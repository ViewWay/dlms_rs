//! IP6 Setup interface class (Class ID: 66)
//!
//! The IP6 Setup interface class manages IPv6 network configuration for meters.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: ip_address_method - How IP address is assigned
//! - Attribute 3: ip_address - The IPv6 address
//! - Attribute 4: prefix_length - Network prefix length
//! - Attribute 5: gateway_address - Default gateway IPv6 address
//! - Attribute 6: primary_dns_server - Primary DNS server IPv6 address
//! - Attribute 7: secondary_dns_server - Secondary DNS server IPv6 address
//! - Attribute 8: subnet_mask - Subnet mask (IPv6 style)
//! - Attribute 9: multicast_address - Multicast IPv6 address
//! - Attribute 10: mtu_size - Maximum Transmission Unit size

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// IP Address Assignment Method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Ip6AddressMethod {
    /// Static IP address
    Static = 0,
    /// DHCPv6
    Dhcp = 1,
    /// SLAAC (Stateless Address Autoconfiguration)
    Slaac = 2,
    /// Auto IP (link-local)
    AutoIp = 3,
}

impl Ip6AddressMethod {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Static,
            1 => Self::Dhcp,
            2 => Self::Slaac,
            3 => Self::AutoIp,
            _ => Self::Static,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if this is an automatic method
    pub fn is_automatic(self) -> bool {
        matches!(self, Self::Dhcp | Self::Slaac | Self::AutoIp)
    }
}

/// IPv6 Address (128-bit)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv6Addr {
    /// 16 octets representing the IPv6 address
    pub octets: [u8; 16],
}

impl Ipv6Addr {
    /// Create a new IPv6 address from octets
    pub fn new(octets: [u8; 16]) -> Self {
        Self { octets }
    }

    /// Create from bytes slice
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() == 16 {
            let mut octets = [0u8; 16];
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

    /// Create unspecified address (::)
    pub fn unspecified() -> Self {
        Self { octets: [0u8; 16] }
    }

    /// Create localhost address (::1)
    pub fn localhost() -> Self {
        let mut octets = [0u8; 16];
        octets[15] = 1;
        Self { octets }
    }

    /// Check if address is unspecified
    pub fn is_unspecified(&self) -> bool {
        self.octets == [0u8; 16]
    }

    /// Check if address is localhost
    pub fn is_loopback(&self) -> bool {
        self.octets[..15] == [0u8; 15] && self.octets[15] == 1
    }
}

impl Default for Ipv6Addr {
    fn default() -> Self {
        Self::unspecified()
    }
}

impl std::fmt::Display for Ipv6Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Format IPv6 address in standard notation
        for i in 0..8 {
            let value = u16::from_be_bytes([
                self.octets[i * 2],
                self.octets[i * 2 + 1],
            ]);
            if i > 0 {
                write!(f, ":")?;
            }
            write!(f, "{:x}", value)?;
        }
        Ok(())
    }
}

/// IP6 Setup interface class (Class ID: 66)
///
/// Default OBIS: 0-0:66.0.0.255
///
/// This class manages IPv6 network configuration for meters.
#[derive(Debug, Clone)]
pub struct Ip6Setup {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// IP address assignment method
    address_method: Arc<RwLock<Ip6AddressMethod>>,

    /// IPv6 address
    ip_address: Arc<RwLock<Ipv6Addr>>,

    /// Network prefix length (0-128)
    prefix_length: Arc<RwLock<u8>>,

    /// Gateway IPv6 address
    gateway_address: Arc<RwLock<Ipv6Addr>>,

    /// Primary DNS server IPv6 address
    primary_dns: Arc<RwLock<Ipv6Addr>>,

    /// Secondary DNS server IPv6 address
    secondary_dns: Arc<RwLock<Ipv6Addr>>,

    /// MTU size (576-1500+ for jumbo frames)
    mtu_size: Arc<RwLock<u16>>,
}

impl Ip6Setup {
    /// Class ID for Ip6Setup
    pub const CLASS_ID: u16 = 66;

    /// Default OBIS code for Ip6Setup (0-0:66.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 66, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_ADDRESS_METHOD: u8 = 2;
    pub const ATTR_IP_ADDRESS: u8 = 3;
    pub const ATTR_PREFIX_LENGTH: u8 = 4;
    pub const ATTR_GATEWAY_ADDRESS: u8 = 5;
    pub const ATTR_PRIMARY_DNS: u8 = 6;
    pub const ATTR_SECONDARY_DNS: u8 = 7;
    pub const ATTR_MTU_SIZE: u8 = 10;

    /// Create a new Ip6Setup object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            address_method: Arc::new(RwLock::new(Ip6AddressMethod::Slaac)),
            ip_address: Arc::new(RwLock::new(Ipv6Addr::unspecified())),
            prefix_length: Arc::new(RwLock::new(64)),
            gateway_address: Arc::new(RwLock::new(Ipv6Addr::unspecified())),
            primary_dns: Arc::new(RwLock::new(Ipv6Addr::unspecified())),
            secondary_dns: Arc::new(RwLock::new(Ipv6Addr::unspecified())),
            mtu_size: Arc::new(RwLock::new(1500)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get the address assignment method
    pub async fn address_method(&self) -> Ip6AddressMethod {
        *self.address_method.read().await
    }

    /// Set the address assignment method
    pub async fn set_address_method(&self, method: Ip6AddressMethod) {
        *self.address_method.write().await = method;
    }

    /// Get the IPv6 address
    pub async fn ip_address(&self) -> Ipv6Addr {
        self.ip_address.read().await.clone()
    }

    /// Set the IPv6 address
    pub async fn set_ip_address(&self, addr: Ipv6Addr) {
        *self.ip_address.write().await = addr;
    }

    /// Set the IPv6 address from bytes
    pub async fn set_ip_address_from_bytes(&self, bytes: &[u8]) -> DlmsResult<()> {
        if let Some(addr) = Ipv6Addr::from_bytes(bytes) {
            self.set_ip_address(addr).await;
            Ok(())
        } else {
            Err(DlmsError::InvalidData("Invalid IPv6 address bytes".to_string()))
        }
    }

    /// Get the prefix length
    pub async fn prefix_length(&self) -> u8 {
        *self.prefix_length.read().await
    }

    /// Set the prefix length
    pub async fn set_prefix_length(&self, length: u8) {
        *self.prefix_length.write().await = length.min(128);
    }

    /// Get the gateway address
    pub async fn gateway_address(&self) -> Ipv6Addr {
        self.gateway_address.read().await.clone()
    }

    /// Set the gateway address
    pub async fn set_gateway_address(&self, addr: Ipv6Addr) {
        *self.gateway_address.write().await = addr;
    }

    /// Get the primary DNS server address
    pub async fn primary_dns(&self) -> Ipv6Addr {
        self.primary_dns.read().await.clone()
    }

    /// Set the primary DNS server address
    pub async fn set_primary_dns(&self, addr: Ipv6Addr) {
        *self.primary_dns.write().await = addr;
    }

    /// Get the secondary DNS server address
    pub async fn secondary_dns(&self) -> Ipv6Addr {
        self.secondary_dns.read().await.clone()
    }

    /// Set the secondary DNS server address
    pub async fn set_secondary_dns(&self, addr: Ipv6Addr) {
        *self.secondary_dns.write().await = addr;
    }

    /// Get the MTU size
    pub async fn mtu_size(&self) -> u16 {
        *self.mtu_size.read().await
    }

    /// Set the MTU size
    pub async fn set_mtu_size(&self, size: u16) {
        *self.mtu_size.write().await = size.max(576);
    }

    /// Check if using static IP
    pub async fn is_static(&self) -> bool {
        matches!(self.address_method().await, Ip6AddressMethod::Static)
    }

    /// Check if using automatic IP assignment
    pub async fn is_automatic(&self) -> bool {
        self.address_method().await.is_automatic()
    }

    /// Check if has a valid IP configured
    pub async fn has_valid_ip(&self) -> bool {
        !self.ip_address().await.is_unspecified()
    }
}

#[async_trait]
impl CosemObject for Ip6Setup {
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
            Self::ATTR_ADDRESS_METHOD => {
                Ok(DataObject::Enumerate(self.address_method().await.to_u8()))
            }
            Self::ATTR_IP_ADDRESS => {
                Ok(DataObject::OctetString(self.ip_address().await.to_bytes()))
            }
            Self::ATTR_PREFIX_LENGTH => {
                Ok(DataObject::Unsigned8(self.prefix_length().await))
            }
            Self::ATTR_GATEWAY_ADDRESS => {
                Ok(DataObject::OctetString(self.gateway_address().await.to_bytes()))
            }
            Self::ATTR_PRIMARY_DNS => {
                Ok(DataObject::OctetString(self.primary_dns().await.to_bytes()))
            }
            Self::ATTR_SECONDARY_DNS => {
                Ok(DataObject::OctetString(self.secondary_dns().await.to_bytes()))
            }
            Self::ATTR_MTU_SIZE => {
                Ok(DataObject::Unsigned16(self.mtu_size().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Ip6Setup has no attribute {}",
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
            Self::ATTR_ADDRESS_METHOD => {
                match value {
                    DataObject::Enumerate(method) => {
                        self.set_address_method(Ip6AddressMethod::from_u8(method)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for address_method".to_string(),
                    )),
                }
            }
            Self::ATTR_IP_ADDRESS => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_ip_address_from_bytes(&bytes).await?;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for ip_address".to_string(),
                    )),
                }
            }
            Self::ATTR_PREFIX_LENGTH => {
                match value {
                    DataObject::Unsigned8(length) => {
                        self.set_prefix_length(length).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for prefix_length".to_string(),
                    )),
                }
            }
            Self::ATTR_GATEWAY_ADDRESS => {
                match value {
                    DataObject::OctetString(bytes) => {
                        if let Some(addr) = Ipv6Addr::from_bytes(&bytes) {
                            self.set_gateway_address(addr).await;
                            Ok(())
                        } else {
                            Err(DlmsError::InvalidData("Invalid IPv6 address".to_string()))
                        }
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for gateway_address".to_string(),
                    )),
                }
            }
            Self::ATTR_PRIMARY_DNS => {
                match value {
                    DataObject::OctetString(bytes) => {
                        if let Some(addr) = Ipv6Addr::from_bytes(&bytes) {
                            self.set_primary_dns(addr).await;
                            Ok(())
                        } else {
                            Err(DlmsError::InvalidData("Invalid IPv6 address".to_string()))
                        }
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for primary_dns".to_string(),
                    )),
                }
            }
            Self::ATTR_SECONDARY_DNS => {
                match value {
                    DataObject::OctetString(bytes) => {
                        if let Some(addr) = Ipv6Addr::from_bytes(&bytes) {
                            self.set_secondary_dns(addr).await;
                            Ok(())
                        } else {
                            Err(DlmsError::InvalidData("Invalid IPv6 address".to_string()))
                        }
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for secondary_dns".to_string(),
                    )),
                }
            }
            Self::ATTR_MTU_SIZE => {
                match value {
                    DataObject::Unsigned16(size) => {
                        self.set_mtu_size(size).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned16 for mtu_size".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Ip6Setup has no attribute {}",
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
            "Ip6Setup has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ip6_setup_class_id() {
        let setup = Ip6Setup::with_default_obis();
        assert_eq!(setup.class_id(), 66);
    }

    #[tokio::test]
    async fn test_ip6_setup_obis_code() {
        let setup = Ip6Setup::with_default_obis();
        assert_eq!(setup.obis_code(), Ip6Setup::default_obis());
    }

    #[tokio::test]
    async fn test_ip6_address_method_from_u8() {
        assert_eq!(Ip6AddressMethod::from_u8(0), Ip6AddressMethod::Static);
        assert_eq!(Ip6AddressMethod::from_u8(1), Ip6AddressMethod::Dhcp);
        assert_eq!(Ip6AddressMethod::from_u8(2), Ip6AddressMethod::Slaac);
        assert_eq!(Ip6AddressMethod::from_u8(3), Ip6AddressMethod::AutoIp);
    }

    #[tokio::test]
    async fn test_ip6_address_method_is_automatic() {
        assert!(!Ip6AddressMethod::Static.is_automatic());
        assert!(Ip6AddressMethod::Dhcp.is_automatic());
        assert!(Ip6AddressMethod::Slaac.is_automatic());
        assert!(Ip6AddressMethod::AutoIp.is_automatic());
    }

    #[tokio::test]
    async fn test_ipv6_addr_unspecified() {
        let addr = Ipv6Addr::unspecified();
        assert!(addr.is_unspecified());
        assert!(!addr.is_loopback());
    }

    #[tokio::test]
    async fn test_ipv6_addr_localhost() {
        let addr = Ipv6Addr::localhost();
        assert!(!addr.is_unspecified());
        assert!(addr.is_loopback());
    }

    #[tokio::test]
    async fn test_ipv6_addr_from_bytes() {
        let bytes = vec![
            0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        let addr = Ipv6Addr::from_bytes(&bytes);
        assert!(addr.is_some());
        assert!(!addr.unwrap().is_unspecified());
    }

    #[tokio::test]
    async fn test_ipv6_addr_invalid_bytes() {
        let bytes = vec![0x20, 0x01, 0x0d]; // Too short
        let addr = Ipv6Addr::from_bytes(&bytes);
        assert!(addr.is_none());
    }

    #[tokio::test]
    async fn test_ipv6_addr_to_bytes() {
        let addr = Ipv6Addr::localhost();
        let bytes = addr.to_bytes();
        assert_eq!(bytes.len(), 16);
        assert_eq!(bytes[15], 1);
    }

    #[tokio::test]
    async fn test_ip6_setup_initial_state() {
        let setup = Ip6Setup::with_default_obis();
        assert_eq!(setup.address_method().await, Ip6AddressMethod::Slaac);
        assert!(setup.ip_address().await.is_unspecified());
        assert_eq!(setup.prefix_length().await, 64);
        assert_eq!(setup.mtu_size().await, 1500);
    }

    #[tokio::test]
    async fn test_ip6_setup_set_address_method() {
        let setup = Ip6Setup::with_default_obis();
        setup.set_address_method(Ip6AddressMethod::Static).await;
        assert_eq!(setup.address_method().await, Ip6AddressMethod::Static);
        assert!(setup.is_static().await);
    }

    #[tokio::test]
    async fn test_ip6_setup_set_ip_address() {
        let setup = Ip6Setup::with_default_obis();
        let addr = Ipv6Addr::localhost();
        setup.set_ip_address(addr.clone()).await;
        assert_eq!(setup.ip_address().await, addr);
    }

    #[tokio::test]
    async fn test_ip6_setup_set_ip_address_from_bytes() {
        let setup = Ip6Setup::with_default_obis();
        let bytes = vec![
            0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        setup.set_ip_address_from_bytes(&bytes).await.unwrap();
        assert!(setup.has_valid_ip().await);
    }

    #[tokio::test]
    async fn test_ip6_setup_set_prefix_length() {
        let setup = Ip6Setup::with_default_obis();
        setup.set_prefix_length(48).await;
        assert_eq!(setup.prefix_length().await, 48);

        // Test clamping to max 128
        setup.set_prefix_length(200).await;
        assert_eq!(setup.prefix_length().await, 128);
    }

    #[tokio::test]
    async fn test_ip6_setup_set_gateway() {
        let setup = Ip6Setup::with_default_obis();
        let gateway = Ipv6Addr::localhost();
        setup.set_gateway_address(gateway.clone()).await;
        assert_eq!(setup.gateway_address().await, gateway);
    }

    #[tokio::test]
    async fn test_ip6_setup_set_dns_servers() {
        let setup = Ip6Setup::with_default_obis();
        let dns1 = Ipv6Addr::localhost();
        let dns2 = Ipv6Addr::new([0xfe, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);

        setup.set_primary_dns(dns1.clone()).await;
        setup.set_secondary_dns(dns2.clone()).await;

        assert_eq!(setup.primary_dns().await, dns1);
        assert_eq!(setup.secondary_dns().await, dns2);
    }

    #[tokio::test]
    async fn test_ip6_setup_set_mtu_size() {
        let setup = Ip6Setup::with_default_obis();
        setup.set_mtu_size(9000).await;
        assert_eq!(setup.mtu_size().await, 9000);

        // Test minimum value
        setup.set_mtu_size(100).await;
        assert_eq!(setup.mtu_size().await, 576);
    }

    #[tokio::test]
    async fn test_ip6_setup_is_automatic() {
        let setup = Ip6Setup::with_default_obis();
        assert!(setup.is_automatic().await);

        setup.set_address_method(Ip6AddressMethod::Static).await;
        assert!(!setup.is_automatic().await);
    }

    #[tokio::test]
    async fn test_ip6_setup_has_valid_ip() {
        let setup = Ip6Setup::with_default_obis();
        assert!(!setup.has_valid_ip().await);

        setup.set_ip_address(Ipv6Addr::localhost()).await;
        assert!(setup.has_valid_ip().await);
    }

    #[tokio::test]
    async fn test_ip6_setup_get_attributes() {
        let setup = Ip6Setup::with_default_obis();

        // Test address_method
        let result = setup.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Enumerate(method) => assert_eq!(method, 2), // Slaac
            _ => panic!("Expected Enumerate"),
        }

        // Test prefix_length
        let result = setup.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Unsigned8(length) => assert_eq!(length, 64),
            _ => panic!("Expected Unsigned8"),
        }

        // Test mtu_size
        let result = setup.get_attribute(10, None).await.unwrap();
        match result {
            DataObject::Unsigned16(mtu) => assert_eq!(mtu, 1500),
            _ => panic!("Expected Unsigned16"),
        }
    }

    #[tokio::test]
    async fn test_ip6_setup_set_attributes() {
        let setup = Ip6Setup::with_default_obis();

        setup.set_attribute(2, DataObject::Enumerate(0), None) // Static
            .await
            .unwrap();
        assert_eq!(setup.address_method().await, Ip6AddressMethod::Static);

        setup.set_attribute(4, DataObject::Unsigned8(56), None)
            .await
            .unwrap();
        assert_eq!(setup.prefix_length().await, 56);
    }

    #[tokio::test]
    async fn test_ip6_setup_set_ip_address_attribute() {
        let setup = Ip6Setup::with_default_obis();
        let bytes = vec![
            0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];

        setup.set_attribute(3, DataObject::OctetString(bytes), None)
            .await
            .unwrap();
        assert!(setup.has_valid_ip().await);
    }

    #[tokio::test]
    async fn test_ip6_setup_invalid_ip_bytes() {
        let setup = Ip6Setup::with_default_obis();
        let result = setup.set_attribute(3, DataObject::OctetString(vec![1, 2, 3]), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ip6_setup_read_only_logical_name() {
        let setup = Ip6Setup::with_default_obis();
        let result = setup
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 66, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ip6_setup_invalid_attribute() {
        let setup = Ip6Setup::with_default_obis();
        let result = setup.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ip6_setup_invalid_method() {
        let setup = Ip6Setup::with_default_obis();
        let result = setup.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ipv6_addr_default() {
        let addr = Ipv6Addr::default();
        assert!(addr.is_unspecified());
    }

    #[tokio::test]
    async fn test_ip6_setup_invalid_data_type_for_method() {
        let setup = Ip6Setup::with_default_obis();
        let result = setup.set_attribute(2, DataObject::Boolean(true), None).await;
        assert!(result.is_err());
    }
}
