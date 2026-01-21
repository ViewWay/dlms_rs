//! Tariff interface class (Class ID: 65)
//!
//! The Tariff interface class manages tariff information for billing.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: tariff_id - Unique tariff identifier
//! - Attribute 3: tariff_name - Name/description of the tariff
//! - Attribute 4: unit_price - Price per unit
//! - Attribute 5: currency - Currency code
//! - Attribute 6: tariff_type - Type of tariff (fixed, tiered, etc.)
//! - Attribute 7: valid_from - Tariff validity start date
//! - Attribute 8: valid_until - Tariff validity end date

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Tariff Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TariffType {
    /// Fixed rate tariff
    Fixed = 0,
    /// Tiered rate tariff
    Tiered = 1,
    /// Time-of-use tariff
    TimeOfUse = 2,
    /// Seasonal tariff
    Seasonal = 3,
    /// Peak/off-peak tariff
    PeakOffPeak = 4,
    /// Custom tariff type
    Custom = 5,
}

impl TariffType {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Fixed,
            1 => Self::Tiered,
            2 => Self::TimeOfUse,
            3 => Self::Seasonal,
            4 => Self::PeakOffPeak,
            _ => Self::Custom,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if this is a tiered tariff
    pub fn is_tiered(self) -> bool {
        matches!(self, Self::Tiered | Self::TimeOfUse | Self::Seasonal | Self::PeakOffPeak)
    }
}

/// Tariff interface class (Class ID: 65)
///
/// Default OBIS: 0-0:65.0.0.255
///
/// This class manages tariff information for billing.
#[derive(Debug, Clone)]
pub struct Tariff {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Tariff identifier
    tariff_id: Arc<RwLock<String>>,

    /// Tariff name/description
    tariff_name: Arc<RwLock<String>>,

    /// Price per unit (in smallest currency unit)
    unit_price: Arc<RwLock<i64>>,

    /// Currency code
    currency: Arc<RwLock<String>>,

    /// Tariff type
    tariff_type: Arc<RwLock<TariffType>>,

    /// Validity start date (Unix timestamp)
    valid_from: Arc<RwLock<Option<i64>>>,

    /// Validity end date (Unix timestamp)
    valid_until: Arc<RwLock<Option<i64>>>,
}

impl Tariff {
    /// Class ID for Tariff
    pub const CLASS_ID: u16 = 65;

    /// Default OBIS code for Tariff (0-0:65.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 65, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_TARIFF_ID: u8 = 2;
    pub const ATTR_TARIFF_NAME: u8 = 3;
    pub const ATTR_UNIT_PRICE: u8 = 4;
    pub const ATTR_CURRENCY: u8 = 5;
    pub const ATTR_TARIFF_TYPE: u8 = 6;
    pub const ATTR_VALID_FROM: u8 = 7;
    pub const ATTR_VALID_UNTIL: u8 = 8;

    /// Create a new Tariff object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            tariff_id: Arc::new(RwLock::new(String::new())),
            tariff_name: Arc::new(RwLock::new(String::new())),
            unit_price: Arc::new(RwLock::new(0)),
            currency: Arc::new(RwLock::new(String::new())),
            tariff_type: Arc::new(RwLock::new(TariffType::Fixed)),
            valid_from: Arc::new(RwLock::new(None)),
            valid_until: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get the tariff ID
    pub async fn tariff_id(&self) -> String {
        self.tariff_id.read().await.clone()
    }

    /// Set the tariff ID
    pub async fn set_tariff_id(&self, id: String) {
        *self.tariff_id.write().await = id;
    }

    /// Get the tariff name
    pub async fn tariff_name(&self) -> String {
        self.tariff_name.read().await.clone()
    }

    /// Set the tariff name
    pub async fn set_tariff_name(&self, name: String) {
        *self.tariff_name.write().await = name;
    }

    /// Get the unit price
    pub async fn unit_price(&self) -> i64 {
        *self.unit_price.read().await
    }

    /// Set the unit price
    pub async fn set_unit_price(&self, price: i64) {
        *self.unit_price.write().await = price;
    }

    /// Get the currency
    pub async fn currency(&self) -> String {
        self.currency.read().await.clone()
    }

    /// Set the currency
    pub async fn set_currency(&self, currency: String) {
        *self.currency.write().await = currency;
    }

    /// Get the tariff type
    pub async fn tariff_type(&self) -> TariffType {
        *self.tariff_type.read().await
    }

    /// Set the tariff type
    pub async fn set_tariff_type(&self, tariff_type: TariffType) {
        *self.tariff_type.write().await = tariff_type;
    }

    /// Get the validity start date
    pub async fn valid_from(&self) -> Option<i64> {
        *self.valid_from.read().await
    }

    /// Set the validity start date
    pub async fn set_valid_from(&self, date: Option<i64>) {
        *self.valid_from.write().await = date;
    }

    /// Get the validity end date
    pub async fn valid_until(&self) -> Option<i64> {
        *self.valid_until.read().await
    }

    /// Set the validity end date
    pub async fn set_valid_until(&self, date: Option<i64>) {
        *self.valid_until.write().await = date;
    }

    /// Check if tariff is currently valid
    pub async fn is_valid(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        if let Some(from) = self.valid_from().await {
            if now < from {
                return false;
            }
        }

        if let Some(until) = self.valid_until().await {
            if now > until {
                return false;
            }
        }

        true
    }

    /// Calculate cost for a given quantity
    pub async fn calculate_cost(&self, quantity: i64) -> i64 {
        let price = self.unit_price().await;
        quantity * price
    }

    /// Check if tariff is tiered
    pub async fn is_tiered(&self) -> bool {
        self.tariff_type().await.is_tiered()
    }
}

#[async_trait]
impl CosemObject for Tariff {
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
            Self::ATTR_TARIFF_ID => {
                Ok(DataObject::OctetString(self.tariff_id().await.into_bytes()))
            }
            Self::ATTR_TARIFF_NAME => {
                Ok(DataObject::OctetString(self.tariff_name().await.into_bytes()))
            }
            Self::ATTR_UNIT_PRICE => {
                Ok(DataObject::Integer64(self.unit_price().await))
            }
            Self::ATTR_CURRENCY => {
                Ok(DataObject::OctetString(self.currency().await.into_bytes()))
            }
            Self::ATTR_TARIFF_TYPE => {
                Ok(DataObject::Enumerate(self.tariff_type().await.to_u8()))
            }
            Self::ATTR_VALID_FROM => {
                match self.valid_from().await {
                    Some(timestamp) => Ok(DataObject::Integer64(timestamp)),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_VALID_UNTIL => {
                match self.valid_until().await {
                    Some(timestamp) => Ok(DataObject::Integer64(timestamp)),
                    None => Ok(DataObject::Null),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Tariff has no attribute {}",
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
            Self::ATTR_TARIFF_ID => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let id = String::from_utf8_lossy(&bytes).to_string();
                        self.set_tariff_id(id).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for tariff_id".to_string(),
                    )),
                }
            }
            Self::ATTR_TARIFF_NAME => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let name = String::from_utf8_lossy(&bytes).to_string();
                        self.set_tariff_name(name).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for tariff_name".to_string(),
                    )),
                }
            }
            Self::ATTR_UNIT_PRICE => {
                match value {
                    DataObject::Integer64(price) => {
                        self.set_unit_price(price).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 for unit_price".to_string(),
                    )),
                }
            }
            Self::ATTR_CURRENCY => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let currency = String::from_utf8_lossy(&bytes).to_string();
                        self.set_currency(currency).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for currency".to_string(),
                    )),
                }
            }
            Self::ATTR_TARIFF_TYPE => {
                match value {
                    DataObject::Enumerate(tariff_type) => {
                        self.set_tariff_type(TariffType::from_u8(tariff_type)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for tariff_type".to_string(),
                    )),
                }
            }
            Self::ATTR_VALID_FROM => {
                match value {
                    DataObject::Integer64(timestamp) => {
                        self.set_valid_from(Some(timestamp)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_valid_from(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 or Null for valid_from".to_string(),
                    )),
                }
            }
            Self::ATTR_VALID_UNTIL => {
                match value {
                    DataObject::Integer64(timestamp) => {
                        self.set_valid_until(Some(timestamp)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_valid_until(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 or Null for valid_until".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Tariff has no attribute {}",
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
            "Tariff has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tariff_class_id() {
        let tariff = Tariff::with_default_obis();
        assert_eq!(tariff.class_id(), 65);
    }

    #[tokio::test]
    async fn test_tariff_obis_code() {
        let tariff = Tariff::with_default_obis();
        assert_eq!(tariff.obis_code(), Tariff::default_obis());
    }

    #[tokio::test]
    async fn test_tariff_type_from_u8() {
        assert_eq!(TariffType::from_u8(0), TariffType::Fixed);
        assert_eq!(TariffType::from_u8(1), TariffType::Tiered);
        assert_eq!(TariffType::from_u8(2), TariffType::TimeOfUse);
        assert_eq!(TariffType::from_u8(3), TariffType::Seasonal);
        assert_eq!(TariffType::from_u8(4), TariffType::PeakOffPeak);
    }

    #[tokio::test]
    async fn test_tariff_type_is_tiered() {
        assert!(!TariffType::Fixed.is_tiered());
        assert!(TariffType::Tiered.is_tiered());
        assert!(TariffType::TimeOfUse.is_tiered());
        assert!(TariffType::Seasonal.is_tiered());
        assert!(TariffType::PeakOffPeak.is_tiered());
    }

    #[tokio::test]
    async fn test_tariff_initial_state() {
        let tariff = Tariff::with_default_obis();
        assert_eq!(tariff.tariff_id().await, "");
        assert_eq!(tariff.tariff_name().await, "");
        assert_eq!(tariff.unit_price().await, 0);
        assert_eq!(tariff.tariff_type().await, TariffType::Fixed);
        assert_eq!(tariff.valid_from().await, None);
        assert_eq!(tariff.valid_until().await, None);
    }

    #[tokio::test]
    async fn test_tariff_set_tariff_id() {
        let tariff = Tariff::with_default_obis();
        tariff.set_tariff_id("TARIFF-001".to_string()).await;
        assert_eq!(tariff.tariff_id().await, "TARIFF-001");
    }

    #[tokio::test]
    async fn test_tariff_set_tariff_name() {
        let tariff = Tariff::with_default_obis();
        tariff.set_tariff_name("Residential Rate".to_string()).await;
        assert_eq!(tariff.tariff_name().await, "Residential Rate");
    }

    #[tokio::test]
    async fn test_tariff_set_unit_price() {
        let tariff = Tariff::with_default_obis();
        tariff.set_unit_price(150).await;
        assert_eq!(tariff.unit_price().await, 150);
    }

    #[tokio::test]
    async fn test_tariff_set_currency() {
        let tariff = Tariff::with_default_obis();
        tariff.set_currency("USD".to_string()).await;
        assert_eq!(tariff.currency().await, "USD");
    }

    #[tokio::test]
    async fn test_tariff_set_tariff_type() {
        let tariff = Tariff::with_default_obis();
        tariff.set_tariff_type(TariffType::TimeOfUse).await;
        assert_eq!(tariff.tariff_type().await, TariffType::TimeOfUse);
    }

    #[tokio::test]
    async fn test_tariff_set_validity_dates() {
        let tariff = Tariff::with_default_obis();
        tariff.set_valid_from(Some(1609459200)).await; // 2021-01-01
        tariff.set_valid_until(Some(1640995200)).await; // 2022-01-01

        assert_eq!(tariff.valid_from().await, Some(1609459200));
        assert_eq!(tariff.valid_until().await, Some(1640995200));
    }

    #[tokio::test]
    async fn test_tariff_calculate_cost() {
        let tariff = Tariff::with_default_obis();
        tariff.set_unit_price(10).await;

        let cost = tariff.calculate_cost(100).await;
        assert_eq!(cost, 1000); // 100 * 10
    }

    #[tokio::test]
    async fn test_tariff_is_tiered() {
        let tariff = Tariff::with_default_obis();
        assert!(!tariff.is_tiered().await);

        tariff.set_tariff_type(TariffType::Tiered).await;
        assert!(tariff.is_tiered().await);
    }

    #[tokio::test]
    async fn test_tariff_get_attributes() {
        let tariff = Tariff::with_default_obis();

        // Test tariff_type
        let result = tariff.get_attribute(6, None).await.unwrap();
        match result {
            DataObject::Enumerate(tt) => assert_eq!(tt, 0), // Fixed
            _ => panic!("Expected Enumerate"),
        }

        // Test unit_price
        let result = tariff.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Integer64(price) => assert_eq!(price, 0),
            _ => panic!("Expected Integer64"),
        }
    }

    #[tokio::test]
    async fn test_tariff_set_attributes() {
        let tariff = Tariff::with_default_obis();

        tariff.set_attribute(2, DataObject::OctetString(b"TARIFF-A".to_vec()), None)
            .await
            .unwrap();
        assert_eq!(tariff.tariff_id().await, "TARIFF-A");

        tariff.set_attribute(4, DataObject::Integer64(200), None)
            .await
            .unwrap();
        assert_eq!(tariff.unit_price().await, 200);
    }

    #[tokio::test]
    async fn test_tariff_read_only_logical_name() {
        let tariff = Tariff::with_default_obis();
        let result = tariff
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 65, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tariff_invalid_attribute() {
        let tariff = Tariff::with_default_obis();
        let result = tariff.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tariff_invalid_method() {
        let tariff = Tariff::with_default_obis();
        let result = tariff.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }
}
