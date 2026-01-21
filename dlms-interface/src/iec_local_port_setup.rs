//! IEC Local Port Setup interface class (Class ID: 19)
//!
//! The IEC Local Port Setup interface class configures local communication
//! port parameters according to IEC standards.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: default_baud - Default baud rate
//! - Attribute 3: default_parity - Default parity (0=none, 1=odd, 2=even)
//! - Attribute 4: default_stop_bits - Default stop bits (1 or 2)
//! - Attribute 5: data_bits - Number of data bits (typically 7 or 8)
//! - Attribute 6: mode - Mode of operation
//!
//! # IEC Local Port Setup (Class ID: 19)
//!
//! This class configures the local port for IEC 62056-21/IEC 61107 mode.
//! It is used for direct optical or electrical local communication.

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Parity enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Parity {
    None = 0,
    Odd = 1,
    Even = 2,
    Mark = 3,
    Space = 4,
}

impl Parity {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Odd,
            2 => Self::Even,
            3 => Self::Mark,
            4 => Self::Space,
            _ => Self::None,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PortMode {
    /// IEC 62056-21 mode E
    ModeE = 0,
    /// IEC 62056-21 mode B
    ModeB = 1,
    /// IEC 62056-21 mode C
    ModeC = 2,
    /// Reserved
    Reserved = 3,
}

impl PortMode {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::ModeB,
            2 => Self::ModeC,
            _ => Self::ModeE,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Baud rate enumeration (standard baud rates)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BaudRate {
    B300 = 300,
    B600 = 600,
    B1200 = 1200,
    B2400 = 2400,
    B4800 = 4800,
    B9600 = 9600,
    B19200 = 19200,
    B38400 = 38400,
    B57600 = 57600,
    B115200 = 115200,
}

impl BaudRate {
    /// Create from u32
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            300 => Some(Self::B300),
            600 => Some(Self::B600),
            1200 => Some(Self::B1200),
            2400 => Some(Self::B2400),
            4800 => Some(Self::B4800),
            9600 => Some(Self::B9600),
            19200 => Some(Self::B19200),
            38400 => Some(Self::B38400),
            57600 => Some(Self::B57600),
            115200 => Some(Self::B115200),
            _ => None,
        }
    }

    /// Convert to u32
    pub fn to_u32(self) -> u32 {
        self as u32
    }
}

/// IEC Local Port Setup interface class (Class ID: 19)
///
/// Default OBIS: 0-0:20.0.0.255
///
/// This class configures the local communication port for IEC mode.
#[derive(Debug, Clone)]
pub struct IecLocalPortSetup {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Default baud rate
    default_baud: Arc<RwLock<BaudRate>>,

    /// Default parity
    default_parity: Arc<RwLock<Parity>>,

    /// Default stop bits
    default_stop_bits: Arc<RwLock<u8>>,

    /// Data bits
    data_bits: Arc<RwLock<u8>>,

    /// Mode of operation
    mode: Arc<RwLock<PortMode>>,
}

impl IecLocalPortSetup {
    /// Class ID for IEC Local Port Setup
    pub const CLASS_ID: u16 = 19;

    /// Default OBIS code for IEC Local Port Setup (0-0:20.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 20, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_DEFAULT_BAUD: u8 = 2;
    pub const ATTR_DEFAULT_PARITY: u8 = 3;
    pub const ATTR_DEFAULT_STOP_BITS: u8 = 4;
    pub const ATTR_DATA_BITS: u8 = 5;
    pub const ATTR_MODE: u8 = 6;

    /// Create a new IEC Local Port Setup object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `default_baud` - Default baud rate
    /// * `default_parity` - Default parity setting
    /// * `default_stop_bits` - Default stop bits (1 or 2)
    /// * `data_bits` - Number of data bits (typically 7 or 8)
    /// * `mode` - Mode of operation
    pub fn new(
        logical_name: ObisCode,
        default_baud: BaudRate,
        default_parity: Parity,
        default_stop_bits: u8,
        data_bits: u8,
        mode: PortMode,
    ) -> Self {
        Self {
            logical_name,
            default_baud: Arc::new(RwLock::new(default_baud)),
            default_parity: Arc::new(RwLock::new(default_parity)),
            default_stop_bits: Arc::new(RwLock::new(default_stop_bits)),
            data_bits: Arc::new(RwLock::new(data_bits)),
            mode: Arc::new(RwLock::new(mode)),
        }
    }

    /// Create with default OBIS code and common settings (9600, 8E1)
    pub fn with_default_obis() -> Self {
        Self::new(
            Self::default_obis(),
            BaudRate::B9600,
            Parity::Even,
            1,
            8,
            PortMode::ModeE,
        )
    }

    /// Get the default baud rate
    pub async fn default_baud(&self) -> BaudRate {
        *self.default_baud.read().await
    }

    /// Set the default baud rate
    pub async fn set_default_baud(&self, baud: BaudRate) {
        *self.default_baud.write().await = baud;
    }

    /// Get the default parity
    pub async fn default_parity(&self) -> Parity {
        *self.default_parity.read().await
    }

    /// Set the default parity
    pub async fn set_default_parity(&self, parity: Parity) {
        *self.default_parity.write().await = parity;
    }

    /// Get the default stop bits
    pub async fn default_stop_bits(&self) -> u8 {
        *self.default_stop_bits.read().await
    }

    /// Set the default stop bits
    pub async fn set_default_stop_bits(&self, stop_bits: u8) -> DlmsResult<()> {
        if stop_bits != 1 && stop_bits != 2 {
            return Err(DlmsError::InvalidData(
                "Stop bits must be 1 or 2".to_string(),
            ));
        }
        *self.default_stop_bits.write().await = stop_bits;
        Ok(())
    }

    /// Get the data bits
    pub async fn data_bits(&self) -> u8 {
        *self.data_bits.read().await
    }

    /// Set the data bits
    pub async fn set_data_bits(&self, bits: u8) -> DlmsResult<()> {
        if bits < 5 || bits > 8 {
            return Err(DlmsError::InvalidData(
                "Data bits must be between 5 and 8".to_string(),
            ));
        }
        *self.data_bits.write().await = bits;
        Ok(())
    }

    /// Get the mode
    pub async fn mode(&self) -> PortMode {
        *self.mode.read().await
    }

    /// Set the mode
    pub async fn set_mode(&self, mode: PortMode) {
        *self.mode.write().await = mode;
    }

    /// Get configuration as a tuple
    pub async fn config(&self) -> (BaudRate, Parity, u8, u8, PortMode) {
        (
            self.default_baud().await,
            self.default_parity().await,
            self.default_stop_bits().await,
            self.data_bits().await,
            self.mode().await,
        )
    }

    /// Set common configuration 8N1 (8 data bits, no parity, 1 stop bit)
    pub async fn set_8n1(&self, baud: BaudRate) {
        self.set_default_baud(baud).await;
        self.set_default_parity(Parity::None).await;
        self.set_default_stop_bits(1).await.unwrap();
        self.set_data_bits(8).await.unwrap();
    }

    /// Set common configuration 8E1 (8 data bits, even parity, 1 stop bit)
    pub async fn set_8e1(&self, baud: BaudRate) {
        self.set_default_baud(baud).await;
        self.set_default_parity(Parity::Even).await;
        self.set_default_stop_bits(1).await.unwrap();
        self.set_data_bits(8).await.unwrap();
    }

    /// Set common configuration 8O1 (8 data bits, odd parity, 1 stop bit)
    pub async fn set_8o1(&self, baud: BaudRate) {
        self.set_default_baud(baud).await;
        self.set_default_parity(Parity::Odd).await;
        self.set_default_stop_bits(1).await.unwrap();
        self.set_data_bits(8).await.unwrap();
    }
}

#[async_trait]
impl CosemObject for IecLocalPortSetup {
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
            Self::ATTR_DEFAULT_BAUD => {
                Ok(DataObject::Unsigned32(self.default_baud().await.to_u32()))
            }
            Self::ATTR_DEFAULT_PARITY => {
                Ok(DataObject::Enumerate(self.default_parity().await.to_u8()))
            }
            Self::ATTR_DEFAULT_STOP_BITS => {
                Ok(DataObject::Unsigned8(self.default_stop_bits().await))
            }
            Self::ATTR_DATA_BITS => {
                Ok(DataObject::Unsigned8(self.data_bits().await))
            }
            Self::ATTR_MODE => {
                Ok(DataObject::Enumerate(self.mode().await.to_u8()))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "IEC Local Port Setup has no attribute {}",
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
            Self::ATTR_DEFAULT_BAUD => {
                if let DataObject::Unsigned32(baud) = value {
                    if let Some(rate) = BaudRate::from_u32(baud) {
                        self.set_default_baud(rate).await;
                        Ok(())
                    } else {
                        Err(DlmsError::InvalidData(format!(
                            "Invalid baud rate: {}",
                            baud
                        )))
                    }
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned32 for default_baud".to_string(),
                    ))
                }
            }
            Self::ATTR_DEFAULT_PARITY => {
                let parity = match value {
                    DataObject::Enumerate(p) => Parity::from_u8(p),
                    DataObject::Unsigned8(p) => Parity::from_u8(p),
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Enum for default_parity".to_string(),
                        ))
                    }
                };
                self.set_default_parity(parity).await;
                Ok(())
            }
            Self::ATTR_DEFAULT_STOP_BITS => {
                if let DataObject::Unsigned8(bits) = value {
                    self.set_default_stop_bits(bits).await
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for default_stop_bits".to_string(),
                    ))
                }
            }
            Self::ATTR_DATA_BITS => {
                if let DataObject::Unsigned8(bits) = value {
                    self.set_data_bits(bits).await
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for data_bits".to_string(),
                    ))
                }
            }
            Self::ATTR_MODE => {
                let mode = match value {
                    DataObject::Enumerate(m) => PortMode::from_u8(m),
                    DataObject::Unsigned8(m) => PortMode::from_u8(m),
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Enum for mode".to_string(),
                        ))
                    }
                };
                self.set_mode(mode).await;
                Ok(())
            }
            _ => Err(DlmsError::InvalidData(format!(
                "IEC Local Port Setup has no attribute {}",
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
            "IEC Local Port Setup has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_iec_local_port_setup_class_id() {
        let setup = IecLocalPortSetup::with_default_obis();
        assert_eq!(setup.class_id(), 19);
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_obis_code() {
        let setup = IecLocalPortSetup::with_default_obis();
        assert_eq!(setup.obis_code(), IecLocalPortSetup::default_obis());
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_defaults() {
        let setup = IecLocalPortSetup::with_default_obis();
        assert_eq!(setup.default_baud().await, BaudRate::B9600);
        assert_eq!(setup.default_parity().await, Parity::Even);
        assert_eq!(setup.default_stop_bits().await, 1);
        assert_eq!(setup.data_bits().await, 8);
        assert_eq!(setup.mode().await, PortMode::ModeE);
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_set_baud() {
        let setup = IecLocalPortSetup::with_default_obis();
        setup.set_default_baud(BaudRate::B115200).await;
        assert_eq!(setup.default_baud().await, BaudRate::B115200);
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_set_parity() {
        let setup = IecLocalPortSetup::with_default_obis();
        setup.set_default_parity(Parity::Odd).await;
        assert_eq!(setup.default_parity().await, Parity::Odd);
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_set_stop_bits() {
        let setup = IecLocalPortSetup::with_default_obis();
        setup.set_default_stop_bits(2).await.unwrap();
        assert_eq!(setup.default_stop_bits().await, 2);
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_invalid_stop_bits() {
        let setup = IecLocalPortSetup::with_default_obis();
        let result = setup.set_default_stop_bits(3).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_set_data_bits() {
        let setup = IecLocalPortSetup::with_default_obis();
        setup.set_data_bits(7).await.unwrap();
        assert_eq!(setup.data_bits().await, 7);
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_invalid_data_bits() {
        let setup = IecLocalPortSetup::with_default_obis();
        let result = setup.set_data_bits(9).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_set_mode() {
        let setup = IecLocalPortSetup::with_default_obis();
        setup.set_mode(PortMode::ModeB).await;
        assert_eq!(setup.mode().await, PortMode::ModeB);
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_config() {
        let setup = IecLocalPortSetup::with_default_obis();
        let (baud, parity, stop, data, mode) = setup.config().await;
        assert_eq!(baud, BaudRate::B9600);
        assert_eq!(parity, Parity::Even);
        assert_eq!(stop, 1);
        assert_eq!(data, 8);
        assert_eq!(mode, PortMode::ModeE);
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_set_8n1() {
        let setup = IecLocalPortSetup::with_default_obis();
        setup.set_8n1(BaudRate::B115200).await;
        assert_eq!(setup.default_baud().await, BaudRate::B115200);
        assert_eq!(setup.default_parity().await, Parity::None);
        assert_eq!(setup.default_stop_bits().await, 1);
        assert_eq!(setup.data_bits().await, 8);
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_set_8e1() {
        let setup = IecLocalPortSetup::with_default_obis();
        setup.set_8e1(BaudRate::B19200).await;
        assert_eq!(setup.default_baud().await, BaudRate::B19200);
        assert_eq!(setup.default_parity().await, Parity::Even);
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_set_8o1() {
        let setup = IecLocalPortSetup::with_default_obis();
        setup.set_8o1(BaudRate::B4800).await;
        assert_eq!(setup.default_baud().await, BaudRate::B4800);
        assert_eq!(setup.default_parity().await, Parity::Odd);
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_get_logical_name() {
        let setup = IecLocalPortSetup::with_default_obis();
        let result = setup.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_get_baud() {
        let setup = IecLocalPortSetup::with_default_obis();
        let result = setup.get_attribute(2, None).await.unwrap();
        assert_eq!(result, DataObject::Unsigned32(9600));
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_set_baud_via_attribute() {
        let setup = IecLocalPortSetup::with_default_obis();
        setup
            .set_attribute(2, DataObject::Unsigned32(115200), None)
            .await
            .unwrap();
        assert_eq!(setup.default_baud().await, BaudRate::B115200);
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_set_invalid_baud() {
        let setup = IecLocalPortSetup::with_default_obis();
        let result = setup
            .set_attribute(2, DataObject::Unsigned16(12345), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_read_only_logical_name() {
        let setup = IecLocalPortSetup::with_default_obis();
        let result = setup
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 20, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_invalid_attribute() {
        let setup = IecLocalPortSetup::with_default_obis();
        let result = setup.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iec_local_port_setup_invalid_method() {
        let setup = IecLocalPortSetup::with_default_obis();
        let result = setup.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parity_from_u8() {
        assert_eq!(Parity::from_u8(0), Parity::None);
        assert_eq!(Parity::from_u8(1), Parity::Odd);
        assert_eq!(Parity::from_u8(2), Parity::Even);
        assert_eq!(Parity::from_u8(3), Parity::Mark);
        assert_eq!(Parity::from_u8(4), Parity::Space);
        assert_eq!(Parity::from_u8(255), Parity::None); // Invalid defaults to None
    }

    #[tokio::test]
    async fn test_baud_rate_from_u32() {
        assert_eq!(BaudRate::from_u32(9600), Some(BaudRate::B9600));
        assert_eq!(BaudRate::from_u32(115200), Some(BaudRate::B115200));
        assert_eq!(BaudRate::from_u32(12345), None);
    }

    #[tokio::test]
    async fn test_port_mode_from_u8() {
        assert_eq!(PortMode::from_u8(0), PortMode::ModeE);
        assert_eq!(PortMode::from_u8(1), PortMode::ModeB);
        assert_eq!(PortMode::from_u8(2), PortMode::ModeC);
        assert_eq!(PortMode::from_u8(3), PortMode::ModeE); // Invalid defaults to ModeE
    }
}
