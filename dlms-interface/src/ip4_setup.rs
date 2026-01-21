//! IP4 Setup interface class (Class ID: 26)
//!
//! The IP4 Setup interface class manages IPv4 network configuration.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: ip_address - IPv4 address
//! - Attribute 3: subnet_mask - Subnet mask
//! - Attribute 4: gateway - Default gateway
//! - Attribute 5: primary_dns - Primary DNS server
//! - Attribute 6: secondary_dns - Secondary DNS server
//! - Attribute 7: ip_address_method - Address assignment method (static/DHCP)
//! - Attribute 8: dhcp_server - DHCP server address (when using DHCP)

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// IP Address Assignment Method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IpAddressMethod {
    /// Static IP address
    Static = 0,
    /// DHCP assigned IP address
    Dhcp = 1,
    /// Auto-IP (link-local)
    AutoIp = 2,
}

impl IpAddressMethod {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Static,
            1 => Self::Dhcp,
            2 => Self::AutoIp,
            _ => Self::Static,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if using DHCP
    pub fn is_dhcp(self) -> bool {
        matches!(self, Self::Dhcp)
    }

    /// Check if static
    pub fn is_static(self) -> bool {
        matches!(self, Self::Static)
    }
}

/// IPv4 address (4 bytes)
pub type Ipv4Addr = [u8; 4];

/// IP4 Setup interface class (Class ID: 26)
///
/// Default OBIS: 0-0:26.0.0.255
///
/// This class manages IPv4 network configuration.
#[derive(Debug, Clone)]
pub struct Ip4Setup {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// IPv4 address
    ip_address: Arc<RwLock<Ipv4Addr>>,

    /// Subnet mask
    subnet_mask: Arc<RwLock<Ipv4Addr>>,

    /// Default gateway
    gateway: Arc<RwLock<Ipv4Addr>>,

    /// Primary DNS server
    primary_dns: Arc<RwLock<Ipv4Addr>>,

    /// Secondary DNS server
    secondary_dns: Arc<RwLock<Ipv4Addr>>,

    /// IP address assignment method
    ip_address_method: Arc<RwLock<IpAddressMethod>>,

    /// DHCP server address
    dhcp_server: Arc<RwLock<Ipv4Addr>>,
}

impl Ip4Setup {
    /// Class ID for IP4 Setup
    pub const CLASS_ID: u16 = 26;

    /// Default OBIS code for IP4 Setup (0-0:26.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 26, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_IP_ADDRESS: u8 = 2;
    pub const ATTR_SUBNET_MASK: u8 = 3;
    pub const ATTR_GATEWAY: u8 = 4;
    pub const ATTR_PRIMARY_DNS: u8 = 5;
    pub const ATTR_SECONDARY_DNS: u8 = 6;
    pub const ATTR_IP_ADDRESS_METHOD: u8 = 7;
    pub const ATTR_DHCP_SERVER: u8 = 8;

    /// Create a new IP4 Setup object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            ip_address: Arc::new(RwLock::new([0, 0, 0, 0])),
            subnet_mask: Arc::new(RwLock::new([255, 255, 255, 0])),
            gateway: Arc::new(RwLock::new([0, 0, 0, 0])),
            primary_dns: Arc::new(RwLock::new([0, 0, 0, 0])),
            secondary_dns: Arc::new(RwLock::new([0, 0, 0, 0])),
            ip_address_method: Arc::new(RwLock::new(IpAddressMethod::Static)),
            dhcp_server: Arc::new(RwLock::new([0, 0, 0, 0])),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get the IP address
    pub async fn ip_address(&self) -> Ipv4Addr {
        *self.ip_address.read().await
    }

    /// Set the IP address
    pub async fn set_ip_address(&self, addr: Ipv4Addr) {
        *self.ip_address.write().await = addr;
    }

    /// Get the subnet mask
    pub async fn subnet_mask(&self) -> Ipv4Addr {
        *self.subnet_mask.read().await
    }

    /// Set the subnet mask
    pub async fn set_subnet_mask(&self, mask: Ipv4Addr) {
        *self.subnet_mask.write().await = mask;
    }

    /// Get the gateway
    pub async fn gateway(&self) -> Ipv4Addr {
        *self.gateway.read().await
    }

    /// Set the gateway
    pub async fn set_gateway(&self, addr: Ipv4Addr) {
        *self.gateway.write().await = addr;
    }

    /// Get the primary DNS
    pub async fn primary_dns(&self) -> Ipv4Addr {
        *self.primary_dns.read().await
    }

    /// Set the primary DNS
    pub async fn set_primary_dns(&self, addr: Ipv4Addr) {
        *self.primary_dns.write().await = addr;
    }

    /// Get the secondary DNS
    pub async fn secondary_dns(&self) -> Ipv4Addr {
        *self.secondary_dns.read().await
    }

    /// Set the secondary DNS
    pub async fn set_secondary_dns(&self, addr: Ipv4Addr) {
        *self.secondary_dns.write().await = addr;
    }

    /// Get the IP address method
    pub async fn ip_address_method(&self) -> IpAddressMethod {
        *self.ip_address_method.read().await
    }

    /// Set the IP address method
    pub async fn set_ip_address_method(&self, method: IpAddressMethod) {
        *self.ip_address_method.write().await = method;
    }

    /// Get the DHCP server
    pub async fn dhcp_server(&self) -> Ipv4Addr {
        *self.dhcp_server.read().await
    }

    /// Set the DHCP server
    pub async fn set_dhcp_server(&self, addr: Ipv4Addr) {
        *self.dhcp_server.write().await = addr;
    }

    /// Configure with a specific IP address and subnet mask
    pub async fn configure_static(&self, ip: Ipv4Addr, mask: Ipv4Addr, gateway: Ipv4Addr) {
        self.set_ip_address(ip).await;
        self.set_subnet_mask(mask).await;
        self.set_gateway(gateway).await;
        self.set_ip_address_method(IpAddressMethod::Static).await;
    }

    /// Enable DHCP
    pub async fn enable_dhcp(&self) {
        self.set_ip_address_method(IpAddressMethod::Dhcp).await;
    }

    /// Encode IPv4 address as octet string
    fn encode_ipv4(addr: &Ipv4Addr) -> DataObject {
        DataObject::OctetString(addr.to_vec())
    }

    /// Decode octet string as IPv4 address
    fn decode_ipv4(value: &DataObject) -> DlmsResult<Ipv4Addr> {
        match value {
            DataObject::OctetString(bytes) if bytes.len() == 4 => {
                let mut addr = [0u8; 4];
                addr.copy_from_slice(&bytes[..4]);
                Ok(addr)
            }
            DataObject::OctetString(bytes) if bytes.len() >= 4 => {
                let mut addr = [0u8; 4];
                addr.copy_from_slice(&bytes[..4]);
                Ok(addr)
            }
            _ => Err(DlmsError::InvalidData(
                "Expected OctetString(4) for IPv4 address".to_string(),
            )),
        }
    }
}

#[async_trait]
impl CosemObject for Ip4Setup {
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
            Self::ATTR_IP_ADDRESS => {
                Ok(Self::encode_ipv4(&self.ip_address().await))
            }
            Self::ATTR_SUBNET_MASK => {
                Ok(Self::encode_ipv4(&self.subnet_mask().await))
            }
            Self::ATTR_GATEWAY => {
                Ok(Self::encode_ipv4(&self.gateway().await))
            }
            Self::ATTR_PRIMARY_DNS => {
                Ok(Self::encode_ipv4(&self.primary_dns().await))
            }
            Self::ATTR_SECONDARY_DNS => {
                Ok(Self::encode_ipv4(&self.secondary_dns().await))
            }
            Self::ATTR_IP_ADDRESS_METHOD => {
                Ok(DataObject::Enumerate(self.ip_address_method().await.to_u8()))
            }
            Self::ATTR_DHCP_SERVER => {
                Ok(Self::encode_ipv4(&self.dhcp_server().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "IP4 Setup has no attribute {}",
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
            Self::ATTR_IP_ADDRESS => {
                let addr = Self::decode_ipv4(&value)?;
                self.set_ip_address(addr).await;
                Ok(())
            }
            Self::ATTR_SUBNET_MASK => {
                let mask = Self::decode_ipv4(&value)?;
                self.set_subnet_mask(mask).await;
                Ok(())
            }
            Self::ATTR_GATEWAY => {
                let addr = Self::decode_ipv4(&value)?;
                self.set_gateway(addr).await;
                Ok(())
            }
            Self::ATTR_PRIMARY_DNS => {
                let addr = Self::decode_ipv4(&value)?;
                self.set_primary_dns(addr).await;
                Ok(())
            }
            Self::ATTR_SECONDARY_DNS => {
                let addr = Self::decode_ipv4(&value)?;
                self.set_secondary_dns(addr).await;
                Ok(())
            }
            Self::ATTR_IP_ADDRESS_METHOD => {
                match value {
                    DataObject::Enumerate(method) => {
                        self.set_ip_address_method(IpAddressMethod::from_u8(method)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for ip_address_method".to_string(),
                    )),
                }
            }
            Self::ATTR_DHCP_SERVER => {
                let addr = Self::decode_ipv4(&value)?;
                self.set_dhcp_server(addr).await;
                Ok(())
            }
            _ => Err(DlmsError::InvalidData(format!(
                "IP4 Setup has no attribute {}",
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
            "IP4 Setup has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(a: u8, b: u8, c: u8, d: u8) -> Ipv4Addr {
        [a, b, c, d]
    }

    #[tokio::test]
    async fn test_ip4_setup_class_id() {
        let setup = Ip4Setup::with_default_obis();
        assert_eq!(setup.class_id(), 26);
    }

    #[tokio::test]
    async fn test_ip4_setup_obis_code() {
        let setup = Ip4Setup::with_default_obis();
        assert_eq!(setup.obis_code(), Ip4Setup::default_obis());
    }

    #[tokio::test]
    async fn test_ip_address_method_from_u8() {
        assert_eq!(IpAddressMethod::from_u8(0), IpAddressMethod::Static);
        assert_eq!(IpAddressMethod::from_u8(1), IpAddressMethod::Dhcp);
        assert_eq!(IpAddressMethod::from_u8(2), IpAddressMethod::AutoIp);
        assert_eq!(IpAddressMethod::from_u8(99), IpAddressMethod::Static);
    }

    #[tokio::test]
    async fn test_ip_address_method_to_u8() {
        assert_eq!(IpAddressMethod::Static.to_u8(), 0);
        assert_eq!(IpAddressMethod::Dhcp.to_u8(), 1);
        assert_eq!(IpAddressMethod::AutoIp.to_u8(), 2);
    }

    #[tokio::test]
    async fn test_ip_address_method_is_dhcp() {
        assert!(IpAddressMethod::Dhcp.is_dhcp());
        assert!(!IpAddressMethod::Static.is_dhcp());
    }

    #[tokio::test]
    async fn test_ip_address_method_is_static() {
        assert!(IpAddressMethod::Static.is_static());
        assert!(!IpAddressMethod::Dhcp.is_static());
    }

    #[tokio::test]
    async fn test_ip4_setup_initial_state() {
        let setup = Ip4Setup::with_default_obis();
        assert_eq!(setup.ip_address().await, [0, 0, 0, 0]);
        assert_eq!(setup.subnet_mask().await, [255, 255, 255, 0]);
        assert_eq!(setup.ip_address_method().await, IpAddressMethod::Static);
    }

    #[tokio::test]
    async fn test_ip4_setup_set_ip_address() {
        let setup = Ip4Setup::with_default_obis();
        setup.set_ip_address(ip(192, 168, 1, 100)).await;
        assert_eq!(setup.ip_address().await, [192, 168, 1, 100]);
    }

    #[tokio::test]
    async fn test_ip4_setup_set_subnet_mask() {
        let setup = Ip4Setup::with_default_obis();
        setup.set_subnet_mask(ip(255, 255, 255, 128)).await;
        assert_eq!(setup.subnet_mask().await, [255, 255, 255, 128]);
    }

    #[tokio::test]
    async fn test_ip4_setup_set_gateway() {
        let setup = Ip4Setup::with_default_obis();
        setup.set_gateway(ip(192, 168, 1, 1)).await;
        assert_eq!(setup.gateway().await, [192, 168, 1, 1]);
    }

    #[tokio::test]
    async fn test_ip4_setup_set_dns() {
        let setup = Ip4Setup::with_default_obis();
        setup.set_primary_dns(ip(8, 8, 8, 8)).await;
        setup.set_secondary_dns(ip(8, 8, 4, 4)).await;
        assert_eq!(setup.primary_dns().await, [8, 8, 8, 8]);
        assert_eq!(setup.secondary_dns().await, [8, 8, 4, 4]);
    }

    #[tokio::test]
    async fn test_ip4_setup_configure_static() {
        let setup = Ip4Setup::with_default_obis();
        setup.configure_static(ip(192, 168, 1, 100), ip(255, 255, 255, 0), ip(192, 168, 1, 1)).await;

        assert_eq!(setup.ip_address().await, [192, 168, 1, 100]);
        assert_eq!(setup.subnet_mask().await, [255, 255, 255, 0]);
        assert_eq!(setup.gateway().await, [192, 168, 1, 1]);
        assert_eq!(setup.ip_address_method().await, IpAddressMethod::Static);
    }

    #[tokio::test]
    async fn test_ip4_setup_enable_dhcp() {
        let setup = Ip4Setup::with_default_obis();
        setup.enable_dhcp().await;
        assert_eq!(setup.ip_address_method().await, IpAddressMethod::Dhcp);
    }

    #[tokio::test]
    async fn test_ip4_setup_get_attributes() {
        let setup = Ip4Setup::with_default_obis();

        // Test logical_name
        let result = setup.get_attribute(1, None).await.unwrap();
        match result {
            DataObject::OctetString(bytes) => assert_eq!(bytes.len(), 6),
            _ => panic!("Expected OctetString"),
        }

        // Test ip_address
        let result = setup.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::OctetString(bytes) => assert_eq!(bytes, vec![0, 0, 0, 0]),
            _ => panic!("Expected OctetString"),
        }

        // Test ip_address_method
        let result = setup.get_attribute(7, None).await.unwrap();
        match result {
            DataObject::Enumerate(method) => assert_eq!(method, 0), // Static
            _ => panic!("Expected Enumerate"),
        }
    }

    #[tokio::test]
    async fn test_ip4_setup_set_attributes() {
        let setup = Ip4Setup::with_default_obis();

        setup
            .set_attribute(2, DataObject::OctetString(vec![192, 168, 1, 100]), None)
            .await
            .unwrap();
        assert_eq!(setup.ip_address().await, [192, 168, 1, 100]);

        setup
            .set_attribute(3, DataObject::OctetString(vec![255, 255, 255, 0]), None)
            .await
            .unwrap();
        assert_eq!(setup.subnet_mask().await, [255, 255, 255, 0]);

        setup
            .set_attribute(7, DataObject::Enumerate(1), None)
            .await
            .unwrap();
        assert_eq!(setup.ip_address_method().await, IpAddressMethod::Dhcp);
    }

    #[tokio::test]
    async fn test_ip4_setup_read_only_logical_name() {
        let setup = Ip4Setup::with_default_obis();
        let result = setup
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 26, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ip4_setup_invalid_attribute() {
        let setup = Ip4Setup::with_default_obis();
        let result = setup.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ip4_setup_invalid_method() {
        let setup = Ip4Setup::with_default_obis();
        let result = setup.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ip4_setup_invalid_ip_address() {
        let setup = Ip4Setup::with_default_obis();
        let result = setup
            .set_attribute(2, DataObject::OctetString(vec![1, 2]), None)
            .await;
        assert!(result.is_err());
    }
}
