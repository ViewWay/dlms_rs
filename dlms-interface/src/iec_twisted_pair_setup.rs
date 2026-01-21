//! IEC Twisted Pair Setup interface class (Class ID: 24)
//!
//! The IEC Twisted Pair Setup interface class configures communication
//! parameters for twisted pair (M-Bus style) connections according to
//! IEC 62056-31/IEC 870 standards.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: communication_speed - Communication speed (baud rate)
//! - Attribute 3: mode - Communication mode
//! - Attribute 4: protocol_select - Protocol selection
//! - Attribute 5: supported_communication_speeds - List of supported baud rates
//!
//! # IEC Twisted Pair Setup (Class ID: 24)
//!
//! This class configures twisted pair communication parameters, commonly
//! used for M-Bus and other twisted pair local connections.

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Communication mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CommunicationMode {
    /// Mode 1 - Direct connection
    Mode1 = 1,
    /// Mode 2 - Remote connection
    Mode2 = 2,
    /// Mode 3 - Reserved
    Mode3 = 3,
    /// Mode 4 - Reserved
    Mode4 = 4,
}

impl CommunicationMode {
    /// Create from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Mode1),
            2 => Some(Self::Mode2),
            3 => Some(Self::Mode3),
            4 => Some(Self::Mode4),
            _ => None,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Protocol selection enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ProtocolSelect {
    /// IEC 62056-46 (HDLC)
    Hdlc = 0,
    /// M-Bus according to IEC 870-5
    MBus = 1,
    /// Reserved
    Reserved = 2,
}

impl ProtocolSelect {
    /// Create from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Hdlc),
            1 => Some(Self::MBus),
            _ => Some(Self::Reserved),
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// IEC Twisted Pair Setup interface class (Class ID: 24)
///
/// Default OBIS: 0-0:25.0.0.255
///
/// This class configures twisted pair communication parameters.
#[derive(Debug, Clone)]
pub struct IecTwistedPairSetup {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Communication speed (baud rate)
    communication_speed: Arc<RwLock<u32>>,

    /// Communication mode
    mode: Arc<RwLock<CommunicationMode>>,

    /// Protocol selection
    protocol_select: Arc<RwLock<ProtocolSelect>>,

    /// Supported communication speeds
    supported_communication_speeds: Arc<RwLock<Vec<u32>>>,
}

impl IecTwistedPairSetup {
    /// Class ID for IEC Twisted Pair Setup
    pub const CLASS_ID: u16 = 24;

    /// Default OBIS code for IEC Twisted Pair Setup (0-0:25.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 25, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_COMMUNICATION_SPEED: u8 = 2;
    pub const ATTR_MODE: u8 = 3;
    pub const ATTR_PROTOCOL_SELECT: u8 = 4;
    pub const ATTR_SUPPORTED_COMMUNICATION_SPEEDS: u8 = 5;

    /// Create a new IEC Twisted Pair Setup object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `communication_speed` - Communication speed in baud
    /// * `mode` - Communication mode
    /// * `protocol_select` - Protocol selection
    /// * `supported_communication_speeds` - List of supported baud rates
    pub fn new(
        logical_name: ObisCode,
        communication_speed: u32,
        mode: CommunicationMode,
        protocol_select: ProtocolSelect,
        supported_communication_speeds: Vec<u32>,
    ) -> Self {
        Self {
            logical_name,
            communication_speed: Arc::new(RwLock::new(communication_speed)),
            mode: Arc::new(RwLock::new(mode)),
            protocol_select: Arc::new(RwLock::new(protocol_select)),
            supported_communication_speeds: Arc::new(RwLock::new(
                supported_communication_speeds,
            )),
        }
    }

    /// Create with default OBIS code and common settings
    pub fn with_default_obis() -> Self {
        Self::new(
            Self::default_obis(),
            2400,  // Common M-Bus speed
            CommunicationMode::Mode1,
            ProtocolSelect::MBus,
            vec![300, 600, 1200, 2400, 4800, 9600, 19200, 38400],
        )
    }

    /// Get the communication speed
    pub async fn communication_speed(&self) -> u32 {
        *self.communication_speed.read().await
    }

    /// Set the communication speed
    pub async fn set_communication_speed(&self, speed: u32) -> DlmsResult<()> {
        // Validate speed is in supported list
        let supported = self.supported_communication_speeds.read().await;
        if !supported.contains(&speed) {
            return Err(DlmsError::InvalidData(format!(
                "Speed {} not in supported list",
                speed
            )));
        }
        *self.communication_speed.write().await = speed;
        Ok(())
    }

    /// Get the communication mode
    pub async fn mode(&self) -> CommunicationMode {
        *self.mode.read().await
    }

    /// Set the communication mode
    pub async fn set_mode(&self, mode: CommunicationMode) {
        *self.mode.write().await = mode;
    }

    /// Get the protocol selection
    pub async fn protocol_select(&self) -> ProtocolSelect {
        *self.protocol_select.read().await
    }

    /// Set the protocol selection
    pub async fn set_protocol_select(&self, protocol: ProtocolSelect) {
        *self.protocol_select.write().await = protocol;
    }

    /// Get the supported communication speeds
    pub async fn supported_communication_speeds(&self) -> Vec<u32> {
        self.supported_communication_speeds.read().await.clone()
    }

    /// Check if a speed is supported
    pub async fn is_speed_supported(&self, speed: u32) -> bool {
        let supported = self.supported_communication_speeds.read().await;
        supported.contains(&speed)
    }

    /// Get configuration as a tuple
    pub async fn config(&self) -> (u32, CommunicationMode, ProtocolSelect) {
        (
            self.communication_speed().await,
            self.mode().await,
            self.protocol_select().await,
        )
    }

    /// Set default settings (2400 baud, Mode1, M-Bus)
    pub async fn set_defaults(&self) -> DlmsResult<()> {
        self.set_communication_speed(2400).await?;
        self.set_mode(CommunicationMode::Mode1).await;
        self.set_protocol_select(ProtocolSelect::MBus).await;
        Ok(())
    }
}

#[async_trait]
impl CosemObject for IecTwistedPairSetup {
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
                Ok(DataObject::Unsigned32(self.communication_speed().await))
            }
            Self::ATTR_MODE => {
                Ok(DataObject::Enumerate(self.mode().await.to_u8()))
            }
            Self::ATTR_PROTOCOL_SELECT => {
                Ok(DataObject::Enumerate(self.protocol_select().await.to_u8()))
            }
            Self::ATTR_SUPPORTED_COMMUNICATION_SPEEDS => {
                let speeds = self.supported_communication_speeds().await;
                let speed_objs: Vec<DataObject> =
                    speeds.into_iter().map(DataObject::Unsigned32).collect();
                Ok(DataObject::Array(speed_objs))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "IEC Twisted Pair Setup has no attribute {}",
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
                if let DataObject::Unsigned32(speed) = value {
                    self.set_communication_speed(speed).await
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned32 for communication_speed".to_string(),
                    ))
                }
            }
            Self::ATTR_MODE => {
                let mode = match value {
                    DataObject::Enumerate(m) => CommunicationMode::from_u8(m),
                    DataObject::Unsigned8(m) => CommunicationMode::from_u8(m),
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Enumerate for mode".to_string(),
                        ))
                    }
                };
                if let Some(m) = mode {
                    self.set_mode(m).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData("Invalid mode value".to_string()))
                }
            }
            Self::ATTR_PROTOCOL_SELECT => {
                let protocol = match value {
                    DataObject::Enumerate(p) => ProtocolSelect::from_u8(p),
                    DataObject::Unsigned8(p) => ProtocolSelect::from_u8(p),
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Enumerate for protocol_select".to_string(),
                        ))
                    }
                };
                // from_u8 always returns Some (with Reserved as fallback), so unwrap is safe
                self.set_protocol_select(protocol.unwrap_or(ProtocolSelect::Reserved)).await;
                Ok(())
            }
            Self::ATTR_SUPPORTED_COMMUNICATION_SPEEDS => {
                // Read-only attribute
                Err(DlmsError::AccessDenied(
                    "Attribute 5 (supported_communication_speeds) is read-only".to_string(),
                ))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "IEC Twisted Pair Setup has no attribute {}",
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
            "IEC Twisted Pair Setup has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_class_id() {
        let setup = IecTwistedPairSetup::with_default_obis();
        assert_eq!(setup.class_id(), 24);
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_obis_code() {
        let setup = IecTwistedPairSetup::with_default_obis();
        assert_eq!(setup.obis_code(), IecTwistedPairSetup::default_obis());
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_defaults() {
        let setup = IecTwistedPairSetup::with_default_obis();
        assert_eq!(setup.communication_speed().await, 2400);
        assert_eq!(setup.mode().await, CommunicationMode::Mode1);
        assert_eq!(setup.protocol_select().await, ProtocolSelect::MBus);
        assert_eq!(setup.supported_communication_speeds().await.len(), 8);
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_set_communication_speed() {
        let setup = IecTwistedPairSetup::with_default_obis();
        setup.set_communication_speed(9600).await.unwrap();
        assert_eq!(setup.communication_speed().await, 9600);
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_set_unsupported_speed() {
        let setup = IecTwistedPairSetup::with_default_obis();
        let result = setup.set_communication_speed(12345).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_set_mode() {
        let setup = IecTwistedPairSetup::with_default_obis();
        setup.set_mode(CommunicationMode::Mode2).await;
        assert_eq!(setup.mode().await, CommunicationMode::Mode2);
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_set_protocol_select() {
        let setup = IecTwistedPairSetup::with_default_obis();
        setup.set_protocol_select(ProtocolSelect::Hdlc).await;
        assert_eq!(setup.protocol_select().await, ProtocolSelect::Hdlc);
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_is_speed_supported() {
        let setup = IecTwistedPairSetup::with_default_obis();
        assert!(setup.is_speed_supported(2400).await);
        assert!(setup.is_speed_supported(9600).await);
        assert!(!setup.is_speed_supported(12345).await);
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_config() {
        let setup = IecTwistedPairSetup::with_default_obis();
        let (speed, mode, protocol) = setup.config().await;
        assert_eq!(speed, 2400);
        assert_eq!(mode, CommunicationMode::Mode1);
        assert_eq!(protocol, ProtocolSelect::MBus);
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_set_defaults() {
        let setup = IecTwistedPairSetup::new(
            IecTwistedPairSetup::default_obis(),
            9600,
            CommunicationMode::Mode2,
            ProtocolSelect::Hdlc,
            vec![2400, 9600],
        );

        setup.set_defaults().await.unwrap();

        assert_eq!(setup.communication_speed().await, 2400);
        assert_eq!(setup.mode().await, CommunicationMode::Mode1);
        assert_eq!(setup.protocol_select().await, ProtocolSelect::MBus);
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_get_logical_name() {
        let setup = IecTwistedPairSetup::with_default_obis();
        let result = setup.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_get_communication_speed() {
        let setup = IecTwistedPairSetup::with_default_obis();
        let result = setup.get_attribute(2, None).await.unwrap();
        assert_eq!(result, DataObject::Unsigned32(2400));
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_set_communication_speed_via_attribute() {
        let setup = IecTwistedPairSetup::with_default_obis();
        setup
            .set_attribute(2, DataObject::Unsigned32(9600), None)
            .await
            .unwrap();
        assert_eq!(setup.communication_speed().await, 9600);
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_set_mode_via_attribute() {
        let setup = IecTwistedPairSetup::with_default_obis();
        setup
            .set_attribute(3, DataObject::Enumerate(2), None)
            .await
            .unwrap();
        assert_eq!(setup.mode().await, CommunicationMode::Mode2);
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_set_invalid_mode() {
        let setup = IecTwistedPairSetup::with_default_obis();
        let result = setup.set_attribute(3, DataObject::Enumerate(99), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_get_supported_speeds() {
        let setup = IecTwistedPairSetup::with_default_obis();
        let result = setup.get_attribute(5, None).await.unwrap();

        match result {
            DataObject::Array(speeds) => {
                assert_eq!(speeds.len(), 8);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_read_only_logical_name() {
        let setup = IecTwistedPairSetup::with_default_obis();
        let result = setup
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 25, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_read_only_supported_speeds() {
        let setup = IecTwistedPairSetup::with_default_obis();
        let result = setup
            .set_attribute(
                5,
                DataObject::Array(vec![DataObject::Unsigned32(2400)]),
                None,
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_invalid_attribute() {
        let setup = IecTwistedPairSetup::with_default_obis();
        let result = setup.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iec_twisted_pair_setup_invalid_method() {
        let setup = IecTwistedPairSetup::with_default_obis();
        let result = setup.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_communication_mode_from_u8() {
        assert_eq!(
            CommunicationMode::from_u8(1),
            Some(CommunicationMode::Mode1)
        );
        assert_eq!(
            CommunicationMode::from_u8(2),
            Some(CommunicationMode::Mode2)
        );
        assert_eq!(CommunicationMode::from_u8(99), None);
    }

    #[tokio::test]
    async fn test_protocol_select_from_u8() {
        assert_eq!(ProtocolSelect::from_u8(0), Some(ProtocolSelect::Hdlc));
        assert_eq!(ProtocolSelect::from_u8(1), Some(ProtocolSelect::MBus));
        assert_eq!(ProtocolSelect::from_u8(2), Some(ProtocolSelect::Reserved));
    }
}
