//! MBus Slave Port Setup interface class (Class ID: 25)
//!
//! The MBus Slave Port Setup interface class configures M-Bus
//! (Meter-Bus) slave port communication parameters according to
//! IEC 870-5 and EN 13757 standards.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: default_baud - Default baud rate
//! - Attribute 3: default_parity - Default parity (0=none, 1=odd, 2=even)
//! - Attribute 4: stop_bits - Stop bits (1 or 2)
//! - Attribute 5: data_bits - Data bits (typically 8)
//! - Attribute 6: response_time - Response time in bit periods
//!
//! # MBus Slave Port Setup (Class ID: 25)
//!
//! This class configures the M-Bus slave port for communication.
//! M-Bus is commonly used for utility metering in Europe.

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Parity enumeration for M-Bus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MBusParity {
    None = 0,
    Odd = 1,
    Even = 2,
}

impl MBusParity {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Odd,
            2 => Self::Even,
            _ => Self::None,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// MBus Slave Port Setup interface class (Class ID: 25)
///
/// Default OBIS: 0-0:26.0.0.255
///
/// This class configures M-Bus slave port communication parameters.
#[derive(Debug, Clone)]
pub struct MBusSlavePortSetup {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Default baud rate
    default_baud: Arc<RwLock<u32>>,

    /// Default parity
    default_parity: Arc<RwLock<MBusParity>>,

    /// Stop bits (1 or 2)
    stop_bits: Arc<RwLock<u8>>,

    /// Data bits (typically 8)
    data_bits: Arc<RwLock<u8>>,

    /// Response time in bit periods (11 bits is typical)
    response_time: Arc<RwLock<u8>>,
}

impl MBusSlavePortSetup {
    /// Class ID for MBus Slave Port Setup
    pub const CLASS_ID: u16 = 25;

    /// Default OBIS code for MBus Slave Port Setup (0-0:26.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 26, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_DEFAULT_BAUD: u8 = 2;
    pub const ATTR_DEFAULT_PARITY: u8 = 3;
    pub const ATTR_STOP_BITS: u8 = 4;
    pub const ATTR_DATA_BITS: u8 = 5;
    pub const ATTR_RESPONSE_TIME: u8 = 6;

    /// Create a new MBus Slave Port Setup object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `default_baud` - Default baud rate
    /// * `default_parity` - Default parity setting
    /// * `stop_bits` - Stop bits (1 or 2)
    /// * `data_bits` - Data bits
    /// * `response_time` - Response time in bit periods
    pub fn new(
        logical_name: ObisCode,
        default_baud: u32,
        default_parity: MBusParity,
        stop_bits: u8,
        data_bits: u8,
        response_time: u8,
    ) -> Self {
        Self {
            logical_name,
            default_baud: Arc::new(RwLock::new(default_baud)),
            default_parity: Arc::new(RwLock::new(default_parity)),
            stop_bits: Arc::new(RwLock::new(stop_bits)),
            data_bits: Arc::new(RwLock::new(data_bits)),
            response_time: Arc::new(RwLock::new(response_time)),
        }
    }

    /// Create with default OBIS code and standard M-Bus settings (2400, 8E1)
    pub fn with_default_obis() -> Self {
        Self::new(
            Self::default_obis(),
            2400,  // Standard M-Bus speed
            MBusParity::Even,
            1,
            8,
            11, // Standard response time
        )
    }

    /// Get the default baud rate
    pub async fn default_baud(&self) -> u32 {
        *self.default_baud.read().await
    }

    /// Set the default baud rate
    pub async fn set_default_baud(&self, baud: u32) {
        *self.default_baud.write().await = baud;
    }

    /// Get the default parity
    pub async fn default_parity(&self) -> MBusParity {
        *self.default_parity.read().await
    }

    /// Set the default parity
    pub async fn set_default_parity(&self, parity: MBusParity) {
        *self.default_parity.write().await = parity;
    }

    /// Get the stop bits
    pub async fn stop_bits(&self) -> u8 {
        *self.stop_bits.read().await
    }

    /// Set the stop bits
    pub async fn set_stop_bits(&self, stop_bits: u8) -> DlmsResult<()> {
        if stop_bits != 1 && stop_bits != 2 {
            return Err(DlmsError::InvalidData(
                "Stop bits must be 1 or 2".to_string(),
            ));
        }
        *self.stop_bits.write().await = stop_bits;
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

    /// Get the response time in bit periods
    pub async fn response_time(&self) -> u8 {
        *self.response_time.read().await
    }

    /// Set the response time
    pub async fn set_response_time(&self, time: u8) {
        *self.response_time.write().await = time;
    }

    /// Get configuration as a tuple
    pub async fn config(&self) -> (u32, MBusParity, u8, u8, u8) {
        (
            self.default_baud().await,
            self.default_parity().await,
            self.stop_bits().await,
            self.data_bits().await,
            self.response_time().await,
        )
    }

    /// Set standard M-Bus configuration 8E1 (8 data bits, even parity, 1 stop bit)
    pub async fn set_8e1(&self, baud: u32) {
        self.set_default_baud(baud).await;
        self.set_default_parity(MBusParity::Even).await;
        self.set_stop_bits(1).await.unwrap();
        self.set_data_bits(8).await.unwrap();
    }

    /// Set standard M-Bus configuration 8N1 (8 data bits, no parity, 1 stop bit)
    pub async fn set_8n1(&self, baud: u32) {
        self.set_default_baud(baud).await;
        self.set_default_parity(MBusParity::None).await;
        self.set_stop_bits(1).await.unwrap();
        self.set_data_bits(8).await.unwrap();
    }

    /// Set standard M-Bus configuration 8O1 (8 data bits, odd parity, 1 stop bit)
    pub async fn set_8o1(&self, baud: u32) {
        self.set_default_baud(baud).await;
        self.set_default_parity(MBusParity::Odd).await;
        self.set_stop_bits(1).await.unwrap();
        self.set_data_bits(8).await.unwrap();
    }
}

#[async_trait]
impl CosemObject for MBusSlavePortSetup {
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
                Ok(DataObject::Unsigned32(self.default_baud().await))
            }
            Self::ATTR_DEFAULT_PARITY => {
                Ok(DataObject::Enumerate(self.default_parity().await.to_u8()))
            }
            Self::ATTR_STOP_BITS => {
                Ok(DataObject::Unsigned8(self.stop_bits().await))
            }
            Self::ATTR_DATA_BITS => {
                Ok(DataObject::Unsigned8(self.data_bits().await))
            }
            Self::ATTR_RESPONSE_TIME => {
                Ok(DataObject::Unsigned8(self.response_time().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "MBus Slave Port Setup has no attribute {}",
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
                    self.set_default_baud(baud).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned32 for default_baud".to_string(),
                    ))
                }
            }
            Self::ATTR_DEFAULT_PARITY => {
                let parity = match value {
                    DataObject::Enumerate(p) => MBusParity::from_u8(p),
                    DataObject::Unsigned8(p) => MBusParity::from_u8(p),
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Enumerate for default_parity".to_string(),
                        ))
                    }
                };
                self.set_default_parity(parity).await;
                Ok(())
            }
            Self::ATTR_STOP_BITS => {
                if let DataObject::Unsigned8(bits) = value {
                    self.set_stop_bits(bits).await
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for stop_bits".to_string(),
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
            Self::ATTR_RESPONSE_TIME => {
                if let DataObject::Unsigned8(time) = value {
                    self.set_response_time(time).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for response_time".to_string(),
                    ))
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "MBus Slave Port Setup has no attribute {}",
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
            "MBus Slave Port Setup has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mbus_slave_port_setup_class_id() {
        let setup = MBusSlavePortSetup::with_default_obis();
        assert_eq!(setup.class_id(), 25);
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_obis_code() {
        let setup = MBusSlavePortSetup::with_default_obis();
        assert_eq!(setup.obis_code(), MBusSlavePortSetup::default_obis());
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_defaults() {
        let setup = MBusSlavePortSetup::with_default_obis();
        assert_eq!(setup.default_baud().await, 2400);
        assert_eq!(setup.default_parity().await, MBusParity::Even);
        assert_eq!(setup.stop_bits().await, 1);
        assert_eq!(setup.data_bits().await, 8);
        assert_eq!(setup.response_time().await, 11);
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_set_baud() {
        let setup = MBusSlavePortSetup::with_default_obis();
        setup.set_default_baud(9600).await;
        assert_eq!(setup.default_baud().await, 9600);
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_set_parity() {
        let setup = MBusSlavePortSetup::with_default_obis();
        setup.set_default_parity(MBusParity::Odd).await;
        assert_eq!(setup.default_parity().await, MBusParity::Odd);
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_set_stop_bits() {
        let setup = MBusSlavePortSetup::with_default_obis();
        setup.set_stop_bits(2).await.unwrap();
        assert_eq!(setup.stop_bits().await, 2);
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_invalid_stop_bits() {
        let setup = MBusSlavePortSetup::with_default_obis();
        let result = setup.set_stop_bits(3).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_set_data_bits() {
        let setup = MBusSlavePortSetup::with_default_obis();
        setup.set_data_bits(7).await.unwrap();
        assert_eq!(setup.data_bits().await, 7);
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_invalid_data_bits() {
        let setup = MBusSlavePortSetup::with_default_obis();
        let result = setup.set_data_bits(9).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_set_response_time() {
        let setup = MBusSlavePortSetup::with_default_obis();
        setup.set_response_time(20).await;
        assert_eq!(setup.response_time().await, 20);
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_config() {
        let setup = MBusSlavePortSetup::with_default_obis();
        let (baud, parity, stop, data, resp) = setup.config().await;
        assert_eq!(baud, 2400);
        assert_eq!(parity, MBusParity::Even);
        assert_eq!(stop, 1);
        assert_eq!(data, 8);
        assert_eq!(resp, 11);
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_set_8e1() {
        let setup = MBusSlavePortSetup::with_default_obis();
        setup.set_8e1(4800).await;
        assert_eq!(setup.default_baud().await, 4800);
        assert_eq!(setup.default_parity().await, MBusParity::Even);
        assert_eq!(setup.stop_bits().await, 1);
        assert_eq!(setup.data_bits().await, 8);
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_set_8n1() {
        let setup = MBusSlavePortSetup::with_default_obis();
        setup.set_8n1(9600).await;
        assert_eq!(setup.default_baud().await, 9600);
        assert_eq!(setup.default_parity().await, MBusParity::None);
        assert_eq!(setup.data_bits().await, 8);
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_set_8o1() {
        let setup = MBusSlavePortSetup::with_default_obis();
        setup.set_8o1(19200).await;
        assert_eq!(setup.default_baud().await, 19200);
        assert_eq!(setup.default_parity().await, MBusParity::Odd);
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_get_logical_name() {
        let setup = MBusSlavePortSetup::with_default_obis();
        let result = setup.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_get_baud() {
        let setup = MBusSlavePortSetup::with_default_obis();
        let result = setup.get_attribute(2, None).await.unwrap();
        assert_eq!(result, DataObject::Unsigned32(2400));
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_get_parity() {
        let setup = MBusSlavePortSetup::with_default_obis();
        let result = setup.get_attribute(3, None).await.unwrap();
        assert_eq!(result, DataObject::Enumerate(2)); // Even = 2
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_set_baud_via_attribute() {
        let setup = MBusSlavePortSetup::with_default_obis();
        setup
            .set_attribute(2, DataObject::Unsigned32(9600), None)
            .await
            .unwrap();
        assert_eq!(setup.default_baud().await, 9600);
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_set_parity_via_attribute() {
        let setup = MBusSlavePortSetup::with_default_obis();
        setup
            .set_attribute(3, DataObject::Enumerate(1), None)
            .await
            .unwrap();
        assert_eq!(setup.default_parity().await, MBusParity::Odd);
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_set_response_time_via_attribute() {
        let setup = MBusSlavePortSetup::with_default_obis();
        setup
            .set_attribute(6, DataObject::Unsigned8(20), None)
            .await
            .unwrap();
        assert_eq!(setup.response_time().await, 20);
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_read_only_logical_name() {
        let setup = MBusSlavePortSetup::with_default_obis();
        let result = setup
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 26, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_invalid_attribute() {
        let setup = MBusSlavePortSetup::with_default_obis();
        let result = setup.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mbus_slave_port_setup_invalid_method() {
        let setup = MBusSlavePortSetup::with_default_obis();
        let result = setup.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mbus_parity_from_u8() {
        assert_eq!(MBusParity::from_u8(0), MBusParity::None);
        assert_eq!(MBusParity::from_u8(1), MBusParity::Odd);
        assert_eq!(MBusParity::from_u8(2), MBusParity::Even);
        assert_eq!(MBusParity::from_u8(255), MBusParity::None); // Invalid defaults to None
    }
}
