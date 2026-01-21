//! TCP/UDP Setup interface class (Class ID: 69)
//!
//! The TCP/UDP Setup interface class manages TCP/UDP network configuration.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: protocol_type - Protocol type (TCP/UDP)
//! - Attribute 3: port - Local port number
//! - Attribute 4: ip_address - IP address to bind to
//! - Attribute 5: client_port - Remote port for client mode
//! - Attribute 6: client_ip_address - Remote IP for client mode

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// IPv4 address type
pub type Ipv4Addr = [u8; 4];

/// Protocol Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ProtocolType {
    /// UDP protocol
    Udp = 0,
    /// TCP protocol
    Tcp = 1,
}

impl ProtocolType {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Udp,
            1 => Self::Tcp,
            _ => Self::Udp,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if this is UDP
    pub fn is_udp(self) -> bool {
        matches!(self, Self::Udp)
    }

    /// Check if this is TCP
    pub fn is_tcp(self) -> bool {
        matches!(self, Self::Tcp)
    }
}

/// Helper functions for IPv4 addresses
pub trait IpV4AddrHelper {
    /// Create an unspecified address (0.0.0.0)
    fn unspecified() -> Self;

    /// Create an IPv4 address from 4 octets
    fn new(a: u8, b: u8, c: u8, d: u8) -> Self;

    /// Create from bytes slice
    fn from_bytes(bytes: &[u8]) -> Option<Self> where Self: Sized;

    /// Convert to bytes
    fn to_bytes(&self) -> Vec<u8>;

    /// Convert to display string
    fn to_string(&self) -> String;

    /// Check if address is unspecified (0.0.0.0)
    fn is_unspecified(&self) -> bool;
}

impl IpV4AddrHelper for Ipv4Addr {
    fn unspecified() -> Self {
        [0, 0, 0, 0]
    }

    fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        [a, b, c, d]
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() == 4 {
            Some([bytes[0], bytes[1], bytes[2], bytes[3]])
        } else {
            None
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }

    fn to_string(&self) -> String {
        format!("{}.{}.{}.{}", self[0], self[1], self[2], self[3])
    }

    fn is_unspecified(&self) -> bool {
        self[0] == 0 && self[1] == 0 && self[2] == 0 && self[3] == 0
    }
}

/// TCP/UDP Setup interface class (Class ID: 69)
///
/// Default OBIS: 0-0:69.0.0.255
///
/// This class manages TCP/UDP network configuration.
#[derive(Debug, Clone)]
pub struct TcpUdpSetup {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Protocol type (TCP/UDP)
    protocol_type: Arc<RwLock<ProtocolType>>,

    /// Local port number
    port: Arc<RwLock<u16>>,

    /// IP address to bind to
    ip_address: Arc<RwLock<Ipv4Addr>>,

    /// Client port (for outgoing connections)
    client_port: Arc<RwLock<u16>>,

    /// Client IP address
    client_ip_address: Arc<RwLock<Ipv4Addr>>,
}

impl TcpUdpSetup {
    /// Class ID for TcpUdpSetup
    pub const CLASS_ID: u16 = 69;

    /// Default OBIS code for TcpUdpSetup (0-0:69.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 69, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_PROTOCOL_TYPE: u8 = 2;
    pub const ATTR_PORT: u8 = 3;
    pub const ATTR_IP_ADDRESS: u8 = 4;
    pub const ATTR_CLIENT_PORT: u8 = 5;
    pub const ATTR_CLIENT_IP_ADDRESS: u8 = 6;

    /// Create a new TcpUdpSetup object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            protocol_type: Arc::new(RwLock::new(ProtocolType::Tcp)),
            port: Arc::new(RwLock::new(4059)), // Default DLMS port
            ip_address: Arc::new(RwLock::new(Ipv4Addr::unspecified())),
            client_port: Arc::new(RwLock::new(0)),
            client_ip_address: Arc::new(RwLock::new(Ipv4Addr::unspecified())),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Create with specific port
    pub fn with_port(logical_name: ObisCode, port: u16) -> Self {
        Self {
            logical_name,
            protocol_type: Arc::new(RwLock::new(ProtocolType::Tcp)),
            port: Arc::new(RwLock::new(port)),
            ip_address: Arc::new(RwLock::new(Ipv4Addr::unspecified())),
            client_port: Arc::new(RwLock::new(0)),
            client_ip_address: Arc::new(RwLock::new(Ipv4Addr::unspecified())),
        }
    }

    /// Get the protocol type
    pub async fn protocol_type(&self) -> ProtocolType {
        *self.protocol_type.read().await
    }

    /// Set the protocol type
    pub async fn set_protocol_type(&self, protocol_type: ProtocolType) {
        *self.protocol_type.write().await = protocol_type;
    }

    /// Get the local port
    pub async fn port(&self) -> u16 {
        *self.port.read().await
    }

    /// Set the local port
    pub async fn set_port(&self, port: u16) {
        *self.port.write().await = port;
    }

    /// Get the IP address
    pub async fn ip_address(&self) -> Ipv4Addr {
        *self.ip_address.read().await
    }

    /// Set the IP address
    pub async fn set_ip_address(&self, addr: Ipv4Addr) {
        *self.ip_address.write().await = addr;
    }

    /// Get the client port
    pub async fn client_port(&self) -> u16 {
        *self.client_port.read().await
    }

    /// Set the client port
    pub async fn set_client_port(&self, port: u16) {
        *self.client_port.write().await = port;
    }

    /// Get the client IP address
    pub async fn client_ip_address(&self) -> Ipv4Addr {
        *self.client_ip_address.read().await
    }

    /// Set the client IP address
    pub async fn set_client_ip_address(&self, addr: Ipv4Addr) {
        *self.client_ip_address.write().await = addr;
    }

    /// Check if using TCP
    pub async fn is_tcp(&self) -> bool {
        self.protocol_type().await.is_tcp()
    }

    /// Check if using UDP
    pub async fn is_udp(&self) -> bool {
        self.protocol_type().await.is_udp()
    }

    /// Get the endpoint as a string (e.g., "192.168.1.1:4059")
    pub async fn endpoint_string(&self) -> String {
        let addr = self.ip_address().await;
        format!("{}:{}", addr.to_string(), self.port().await)
    }

    /// Get the client endpoint as a string
    pub async fn client_endpoint_string(&self) -> String {
        let addr = self.client_ip_address().await;
        format!("{}:{}", addr.to_string(), self.client_port().await)
    }
}

#[async_trait]
impl CosemObject for TcpUdpSetup {
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
            Self::ATTR_PROTOCOL_TYPE => {
                Ok(DataObject::Enumerate(self.protocol_type().await.to_u8()))
            }
            Self::ATTR_PORT => {
                Ok(DataObject::Unsigned16(self.port().await))
            }
            Self::ATTR_IP_ADDRESS => {
                Ok(DataObject::OctetString(self.ip_address().await.to_bytes()))
            }
            Self::ATTR_CLIENT_PORT => {
                Ok(DataObject::Unsigned16(self.client_port().await))
            }
            Self::ATTR_CLIENT_IP_ADDRESS => {
                Ok(DataObject::OctetString(self.client_ip_address().await.to_bytes()))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "TcpUdpSetup has no attribute {}",
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
            Self::ATTR_PROTOCOL_TYPE => {
                match value {
                    DataObject::Enumerate(protocol) => {
                        self.set_protocol_type(ProtocolType::from_u8(protocol)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for protocol_type".to_string(),
                    )),
                }
            }
            Self::ATTR_PORT => {
                match value {
                    DataObject::Unsigned16(port) => {
                        self.set_port(port).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned16 for port".to_string(),
                    )),
                }
            }
            Self::ATTR_IP_ADDRESS => {
                match value {
                    DataObject::OctetString(bytes) => {
                        if let Some(addr) = Ipv4Addr::from_bytes(&bytes) {
                            self.set_ip_address(addr).await;
                            Ok(())
                        } else {
                            Err(DlmsError::InvalidData("Invalid IPv4 address".to_string()))
                        }
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for ip_address".to_string(),
                    )),
                }
            }
            Self::ATTR_CLIENT_PORT => {
                match value {
                    DataObject::Unsigned16(port) => {
                        self.set_client_port(port).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned16 for client_port".to_string(),
                    )),
                }
            }
            Self::ATTR_CLIENT_IP_ADDRESS => {
                match value {
                    DataObject::OctetString(bytes) => {
                        if let Some(addr) = Ipv4Addr::from_bytes(&bytes) {
                            self.set_client_ip_address(addr).await;
                            Ok(())
                        } else {
                            Err(DlmsError::InvalidData("Invalid IPv4 address".to_string()))
                        }
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for client_ip_address".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "TcpUdpSetup has no attribute {}",
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
            "TcpUdpSetup has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tcp_udp_setup_class_id() {
        let setup = TcpUdpSetup::with_default_obis();
        assert_eq!(setup.class_id(), 69);
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_obis_code() {
        let setup = TcpUdpSetup::with_default_obis();
        assert_eq!(setup.obis_code(), TcpUdpSetup::default_obis());
    }

    #[tokio::test]
    async fn test_protocol_type_from_u8() {
        assert_eq!(ProtocolType::from_u8(0), ProtocolType::Udp);
        assert_eq!(ProtocolType::from_u8(1), ProtocolType::Tcp);
    }

    #[tokio::test]
    async fn test_protocol_type_is_udp() {
        assert!(ProtocolType::Udp.is_udp());
        assert!(!ProtocolType::Tcp.is_udp());
    }

    #[tokio::test]
    async fn test_protocol_type_is_tcp() {
        assert!(ProtocolType::Tcp.is_tcp());
        assert!(!ProtocolType::Udp.is_tcp());
    }

    #[tokio::test]
    async fn test_ipv4_addr_helper_unspecified() {
        let addr = Ipv4Addr::unspecified();
        assert!(addr.is_unspecified());
        assert_eq!(addr, [0, 0, 0, 0]);
    }

    #[tokio::test]
    async fn test_ipv4_addr_helper_new() {
        let addr = Ipv4Addr::new(192, 168, 1, 100);
        assert_eq!(addr, [192, 168, 1, 100]);
    }

    #[tokio::test]
    async fn test_ipv4_addr_helper_from_bytes() {
        let addr = Ipv4Addr::from_bytes(&[192, 168, 1, 100]);
        assert!(addr.is_some());
        assert_eq!(addr.unwrap(), [192, 168, 1, 100]);
    }

    #[tokio::test]
    async fn test_ipv4_addr_helper_from_bytes_invalid() {
        let addr = Ipv4Addr::from_bytes(&[192, 168]);
        assert!(addr.is_none());
    }

    #[tokio::test]
    async fn test_ipv4_addr_helper_to_bytes() {
        let addr = [192, 168, 1, 100];
        assert_eq!(addr.to_bytes(), vec![192, 168, 1, 100]);
    }

    #[tokio::test]
    async fn test_ipv4_addr_helper_to_string() {
        let addr = [192, 168, 1, 100];
        assert_eq!(addr.to_string(), "192.168.1.100");
    }

    #[tokio::test]
    async fn test_ipv4_addr_helper_is_unspecified() {
        assert!([0, 0, 0, 0].is_unspecified());
        assert!(![192, 168, 1, 1].is_unspecified());
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_initial_state() {
        let setup = TcpUdpSetup::with_default_obis();
        assert_eq!(setup.protocol_type().await, ProtocolType::Tcp);
        assert_eq!(setup.port().await, 4059);
        assert!(setup.ip_address().await.is_unspecified());
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_set_protocol_type() {
        let setup = TcpUdpSetup::with_default_obis();
        setup.set_protocol_type(ProtocolType::Udp).await;
        assert_eq!(setup.protocol_type().await, ProtocolType::Udp);
        assert!(setup.is_udp().await);
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_set_port() {
        let setup = TcpUdpSetup::with_default_obis();
        setup.set_port(8080).await;
        assert_eq!(setup.port().await, 8080);
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_set_ip_address() {
        let setup = TcpUdpSetup::with_default_obis();
        let addr = [192, 168, 1, 100];
        setup.set_ip_address(addr).await;
        assert_eq!(setup.ip_address().await, [192, 168, 1, 100]);
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_set_client_port() {
        let setup = TcpUdpSetup::with_default_obis();
        setup.set_client_port(9000).await;
        assert_eq!(setup.client_port().await, 9000);
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_set_client_ip() {
        let setup = TcpUdpSetup::with_default_obis();
        let addr = [10, 0, 0, 1];
        setup.set_client_ip_address(addr).await;
        assert_eq!(setup.client_ip_address().await, [10, 0, 0, 1]);
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_is_tcp() {
        let setup = TcpUdpSetup::with_default_obis();
        assert!(setup.is_tcp().await);
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_endpoint_string() {
        let setup = TcpUdpSetup::with_default_obis();
        setup.set_ip_address([192, 168, 1, 100]).await;
        setup.set_port(8080).await;
        assert_eq!(setup.endpoint_string().await, "192.168.1.100:8080");
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_client_endpoint_string() {
        let setup = TcpUdpSetup::with_default_obis();
        setup.set_client_ip_address([10, 0, 0, 1]).await;
        setup.set_client_port(9000).await;
        assert_eq!(setup.client_endpoint_string().await, "10.0.0.1:9000");
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_with_port() {
        let setup = TcpUdpSetup::with_port(ObisCode::new(0, 0, 69, 0, 0, 255), 8080);
        assert_eq!(setup.port().await, 8080);
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_get_attributes() {
        let setup = TcpUdpSetup::with_default_obis();

        // Test protocol_type
        let result = setup.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Enumerate(protocol) => assert_eq!(protocol, 1), // TCP
            _ => panic!("Expected Enumerate"),
        }

        // Test port
        let result = setup.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Unsigned16(port) => assert_eq!(port, 4059),
            _ => panic!("Expected Unsigned16"),
        }
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_set_attributes() {
        let setup = TcpUdpSetup::with_default_obis();

        setup.set_attribute(2, DataObject::Enumerate(0), None) // UDP
            .await
            .unwrap();
        assert!(setup.is_udp().await);

        setup.set_attribute(3, DataObject::Unsigned16(9000), None)
            .await
            .unwrap();
        assert_eq!(setup.port().await, 9000);
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_set_ip_attribute() {
        let setup = TcpUdpSetup::with_default_obis();
        setup.set_attribute(4, DataObject::OctetString(vec![192, 168, 1, 1]), None)
            .await
            .unwrap();
        assert_eq!(setup.ip_address().await, [192, 168, 1, 1]);
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_read_only_logical_name() {
        let setup = TcpUdpSetup::with_default_obis();
        let result = setup
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 69, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_invalid_attribute() {
        let setup = TcpUdpSetup::with_default_obis();
        let result = setup.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_invalid_method() {
        let setup = TcpUdpSetup::with_default_obis();
        let result = setup.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_invalid_ip_bytes() {
        let setup = TcpUdpSetup::with_default_obis();
        let result = setup.set_attribute(4, DataObject::OctetString(vec![1, 2]), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tcp_udp_setup_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 69, 0, 0, 1);
        let setup = TcpUdpSetup::new(obis);
        assert_eq!(setup.obis_code(), obis);
    }
}
