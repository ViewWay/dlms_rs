//! IEC HDLC Setup interface class (Class ID: 23)
//!
//! The IEC HDLC Setup interface class configures HDLC protocol parameters
//! for communication according to IEC 62056-46/IEC 61334 standards.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: communication_speed - Communication speed (baud rate)
//! - Attribute 3: window_size_transmission - Window size for transmission (1-7)
//! - Attribute 4: window_size_reception - Window size for reception (1-7)
//! - Attribute 5: maximum_information_length - Maximum info field length (128 or 512)
//! - Attribute 6: supported_communication_speeds - List of supported baud rates
//!
//! # IEC HDLC Setup (Class ID: 23)
//!
//! This class configures HDLC protocol parameters for local communication.
//! It works in conjunction with the HDLC transport layer.

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Information length enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum InformationLength {
    L128 = 128,
    L256 = 256,
    L512 = 512,
    L1024 = 1024,
}

impl InformationLength {
    /// Create from u16
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            128 => Some(Self::L128),
            256 => Some(Self::L256),
            512 => Some(Self::L512),
            1024 => Some(Self::L1024),
            _ => None,
        }
    }

    /// Convert to u16
    pub fn to_u16(self) -> u16 {
        self as u16
    }
}

/// IEC HDLC Setup interface class (Class ID: 23)
///
/// Default OBIS: 0-0:24.0.0.255
///
/// This class configures HDLC protocol parameters for local communication.
#[derive(Debug, Clone)]
pub struct IecHdlcSetup {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Communication speed (baud rate)
    communication_speed: Arc<RwLock<u32>>,

    /// Window size for transmission (1-7 for HDLC)
    window_size_transmission: Arc<RwLock<u8>>,

    /// Window size for reception (1-7 for HDLC)
    window_size_reception: Arc<RwLock<u8>>,

    /// Maximum information field length
    maximum_information_length: Arc<RwLock<InformationLength>>,

    /// Supported communication speeds
    supported_communication_speeds: Arc<RwLock<Vec<u32>>>,
}

impl IecHdlcSetup {
    /// Class ID for IEC HDLC Setup
    pub const CLASS_ID: u16 = 23;

    /// Default OBIS code for IEC HDLC Setup (0-0:24.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 24, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_COMMUNICATION_SPEED: u8 = 2;
    pub const ATTR_WINDOW_SIZE_TRANSMISSION: u8 = 3;
    pub const ATTR_WINDOW_SIZE_RECEPTION: u8 = 4;
    pub const ATTR_MAXIMUM_INFORMATION_LENGTH: u8 = 5;
    pub const ATTR_SUPPORTED_COMMUNICATION_SPEEDS: u8 = 6;

    /// Create a new IEC HDLC Setup object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `communication_speed` - Communication speed in baud
    /// * `window_size_transmission` - Window size for transmission (1-7)
    /// * `window_size_reception` - Window size for reception (1-7)
    /// * `maximum_information_length` - Maximum info field length
    /// * `supported_communication_speeds` - List of supported baud rates
    pub fn new(
        logical_name: ObisCode,
        communication_speed: u32,
        window_size_transmission: u8,
        window_size_reception: u8,
        maximum_information_length: InformationLength,
        supported_communication_speeds: Vec<u32>,
    ) -> Self {
        Self {
            logical_name,
            communication_speed: Arc::new(RwLock::new(communication_speed)),
            window_size_transmission: Arc::new(RwLock::new(window_size_transmission)),
            window_size_reception: Arc::new(RwLock::new(window_size_reception)),
            maximum_information_length: Arc::new(RwLock::new(maximum_information_length)),
            supported_communication_speeds: Arc::new(RwLock::new(supported_communication_speeds)),
        }
    }

    /// Create with default OBIS code and common settings
    pub fn with_default_obis() -> Self {
        Self::new(
            Self::default_obis(),
            9600,
            1,  // Window size 1 (standard)
            1,  // Window size 1 (standard)
            InformationLength::L128,
            vec![300, 600, 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200],
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

    /// Get the window size for transmission
    pub async fn window_size_transmission(&self) -> u8 {
        *self.window_size_transmission.read().await
    }

    /// Set the window size for transmission
    pub async fn set_window_size_transmission(&self, size: u8) -> DlmsResult<()> {
        if size < 1 || size > 7 {
            return Err(DlmsError::InvalidData(
                "Window size must be between 1 and 7".to_string(),
            ));
        }
        *self.window_size_transmission.write().await = size;
        Ok(())
    }

    /// Get the window size for reception
    pub async fn window_size_reception(&self) -> u8 {
        *self.window_size_reception.read().await
    }

    /// Set the window size for reception
    pub async fn set_window_size_reception(&self, size: u8) -> DlmsResult<()> {
        if size < 1 || size > 7 {
            return Err(DlmsError::InvalidData(
                "Window size must be between 1 and 7".to_string(),
            ));
        }
        *self.window_size_reception.write().await = size;
        Ok(())
    }

    /// Get the maximum information length
    pub async fn maximum_information_length(&self) -> InformationLength {
        *self.maximum_information_length.read().await
    }

    /// Set the maximum information length
    pub async fn set_maximum_information_length(&self, length: InformationLength) {
        *self.maximum_information_length.write().await = length;
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
    pub async fn config(&self) -> (u32, u8, u8, InformationLength) {
        (
            self.communication_speed().await,
            self.window_size_transmission().await,
            self.window_size_reception().await,
            self.maximum_information_length().await,
        )
    }

    /// Set default HDLC settings (9600 baud, window size 1, 128-byte info length)
    pub async fn set_defaults(&self) -> DlmsResult<()> {
        self.set_communication_speed(9600).await?;
        self.set_window_size_transmission(1).await?;
        self.set_window_size_reception(1).await?;
        self.set_maximum_information_length(InformationLength::L128).await;
        Ok(())
    }
}

#[async_trait]
impl CosemObject for IecHdlcSetup {
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
            Self::ATTR_WINDOW_SIZE_TRANSMISSION => {
                Ok(DataObject::Unsigned8(self.window_size_transmission().await))
            }
            Self::ATTR_WINDOW_SIZE_RECEPTION => {
                Ok(DataObject::Unsigned8(self.window_size_reception().await))
            }
            Self::ATTR_MAXIMUM_INFORMATION_LENGTH => {
                Ok(DataObject::Unsigned16(
                    self.maximum_information_length().await.to_u16(),
                ))
            }
            Self::ATTR_SUPPORTED_COMMUNICATION_SPEEDS => {
                let speeds = self.supported_communication_speeds().await;
                let speed_objs: Vec<DataObject> =
                    speeds.into_iter().map(DataObject::Unsigned32).collect();
                Ok(DataObject::Array(speed_objs))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "IEC HDLC Setup has no attribute {}",
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
            Self::ATTR_WINDOW_SIZE_TRANSMISSION => {
                if let DataObject::Unsigned8(size) = value {
                    self.set_window_size_transmission(size).await
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for window_size_transmission".to_string(),
                    ))
                }
            }
            Self::ATTR_WINDOW_SIZE_RECEPTION => {
                if let DataObject::Unsigned8(size) = value {
                    self.set_window_size_reception(size).await
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for window_size_reception".to_string(),
                    ))
                }
            }
            Self::ATTR_MAXIMUM_INFORMATION_LENGTH => {
                if let DataObject::Unsigned16(length) = value {
                    if let Some(info_len) = InformationLength::from_u16(length) {
                        self.set_maximum_information_length(info_len).await;
                        Ok(())
                    } else {
                        Err(DlmsError::InvalidData(format!(
                            "Invalid information length: {}",
                            length
                        )))
                    }
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned16 for maximum_information_length".to_string(),
                    ))
                }
            }
            Self::ATTR_SUPPORTED_COMMUNICATION_SPEEDS => {
                // Read-only attribute
                Err(DlmsError::AccessDenied(
                    "Attribute 6 (supported_communication_speeds) is read-only".to_string(),
                ))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "IEC HDLC Setup has no attribute {}",
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
            "IEC HDLC Setup has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_iec_hdlc_setup_class_id() {
        let setup = IecHdlcSetup::with_default_obis();
        assert_eq!(setup.class_id(), 23);
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_obis_code() {
        let setup = IecHdlcSetup::with_default_obis();
        assert_eq!(setup.obis_code(), IecHdlcSetup::default_obis());
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_defaults() {
        let setup = IecHdlcSetup::with_default_obis();
        assert_eq!(setup.communication_speed().await, 9600);
        assert_eq!(setup.window_size_transmission().await, 1);
        assert_eq!(setup.window_size_reception().await, 1);
        assert_eq!(
            setup.maximum_information_length().await,
            InformationLength::L128
        );
        assert_eq!(setup.supported_communication_speeds().await.len(), 10);
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_set_communication_speed() {
        let setup = IecHdlcSetup::with_default_obis();
        setup.set_communication_speed(19200).await.unwrap();
        assert_eq!(setup.communication_speed().await, 19200);
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_set_unsupported_speed() {
        let setup = IecHdlcSetup::with_default_obis();
        let result = setup.set_communication_speed(12345).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_set_window_size_transmission() {
        let setup = IecHdlcSetup::with_default_obis();
        setup.set_window_size_transmission(3).await.unwrap();
        assert_eq!(setup.window_size_transmission().await, 3);
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_set_invalid_window_size_transmission() {
        let setup = IecHdlcSetup::with_default_obis();
        let result = setup.set_window_size_transmission(8).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_set_window_size_reception() {
        let setup = IecHdlcSetup::with_default_obis();
        setup.set_window_size_reception(2).await.unwrap();
        assert_eq!(setup.window_size_reception().await, 2);
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_set_invalid_window_size_reception() {
        let setup = IecHdlcSetup::with_default_obis();
        let result = setup.set_window_size_reception(0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_set_maximum_information_length() {
        let setup = IecHdlcSetup::with_default_obis();
        setup
            .set_maximum_information_length(InformationLength::L512)
            .await;
        assert_eq!(
            setup.maximum_information_length().await,
            InformationLength::L512
        );
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_is_speed_supported() {
        let setup = IecHdlcSetup::with_default_obis();
        assert!(setup.is_speed_supported(9600).await);
        assert!(setup.is_speed_supported(115200).await);
        assert!(!setup.is_speed_supported(12345).await);
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_config() {
        let setup = IecHdlcSetup::with_default_obis();
        let (speed, tx_win, rx_win, info_len) = setup.config().await;
        assert_eq!(speed, 9600);
        assert_eq!(tx_win, 1);
        assert_eq!(rx_win, 1);
        assert_eq!(info_len, InformationLength::L128);
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_set_defaults() {
        let setup = IecHdlcSetup::new(
            IecHdlcSetup::default_obis(),
            19200,
            2,
            2,
            InformationLength::L512,
            vec![300, 9600, 19200],
        );

        setup.set_defaults().await.unwrap();

        assert_eq!(setup.communication_speed().await, 9600);
        assert_eq!(setup.window_size_transmission().await, 1);
        assert_eq!(setup.window_size_reception().await, 1);
        assert_eq!(
            setup.maximum_information_length().await,
            InformationLength::L128
        );
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_get_logical_name() {
        let setup = IecHdlcSetup::with_default_obis();
        let result = setup.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_get_communication_speed() {
        let setup = IecHdlcSetup::with_default_obis();
        let result = setup.get_attribute(2, None).await.unwrap();
        assert_eq!(result, DataObject::Unsigned32(9600));
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_set_communication_speed_via_attribute() {
        let setup = IecHdlcSetup::with_default_obis();
        setup
            .set_attribute(2, DataObject::Unsigned32(19200), None)
            .await
            .unwrap();
        assert_eq!(setup.communication_speed().await, 19200);
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_get_supported_speeds() {
        let setup = IecHdlcSetup::with_default_obis();
        let result = setup.get_attribute(6, None).await.unwrap();

        match result {
            DataObject::Array(speeds) => {
                assert_eq!(speeds.len(), 10);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_read_only_logical_name() {
        let setup = IecHdlcSetup::with_default_obis();
        let result = setup
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 24, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_read_only_supported_speeds() {
        let setup = IecHdlcSetup::with_default_obis();
        let result = setup
            .set_attribute(
                6,
                DataObject::Array(vec![DataObject::Unsigned32(9600)]),
                None,
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_invalid_attribute() {
        let setup = IecHdlcSetup::with_default_obis();
        let result = setup.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_iec_hdlc_setup_invalid_method() {
        let setup = IecHdlcSetup::with_default_obis();
        let result = setup.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_information_length_from_u16() {
        assert_eq!(
            InformationLength::from_u16(128),
            Some(InformationLength::L128)
        );
        assert_eq!(
            InformationLength::from_u16(512),
            Some(InformationLength::L512)
        );
        assert_eq!(InformationLength::from_u16(123), None);
    }
}
