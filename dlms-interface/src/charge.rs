//! Charge interface class (Class ID: 62)
//!
//! The Charge interface class manages charging information for prepaid meters.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: total_amount_charged - Total amount charged
//! - Attribute 3: charge_type - Type of charge (fixed, rate-based, etc.)
//! - Attribute 4: priority_level - Priority level for charge application
//! - Attribute 5: unit_of_measure - Unit of measure for the charge
//! - Attribute 6: currency - Currency code
//! - Attribute 7: charge_per_unit - Cost per unit (for rate-based charges)
//! - Attribute 8: active - Whether the charge is active

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Charge Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChargeType {
    /// Fixed amount charge
    Fixed = 0,
    /// Rate-based charge (per unit)
    RateBased = 1,
    /// Tiered rate charge
    TieredRate = 2,
    /// Time-of-use charge
    TimeOfUse = 3,
    /// Custom charge type
    Custom = 4,
}

impl ChargeType {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Fixed,
            1 => Self::RateBased,
            2 => Self::TieredRate,
            3 => Self::TimeOfUse,
            _ => Self::Custom,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if rate-based
    pub fn is_rate_based(self) -> bool {
        matches!(self, Self::RateBased | Self::TieredRate | Self::TimeOfUse)
    }
}

/// Charge interface class (Class ID: 62)
///
/// Default OBIS: 0-0:62.0.0.255
///
/// This class manages charging information.
#[derive(Debug, Clone)]
pub struct Charge {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Total amount charged
    total_amount_charged: Arc<RwLock<i64>>,

    /// Charge type
    charge_type: Arc<RwLock<ChargeType>>,

    /// Priority level (lower = higher priority)
    priority_level: Arc<RwLock<u8>>,

    /// Unit of measure
    unit_of_measure: Arc<RwLock<String>>,

    /// Currency code
    currency: Arc<RwLock<String>>,

    /// Charge per unit (for rate-based charges)
    charge_per_unit: Arc<RwLock<i32>>,

    /// Whether the charge is active
    active: Arc<RwLock<bool>>,
}

impl Charge {
    /// Class ID for Charge
    pub const CLASS_ID: u16 = 62;

    /// Default OBIS code for Charge (0-0:62.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 62, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_TOTAL_AMOUNT_CHARGED: u8 = 2;
    pub const ATTR_CHARGE_TYPE: u8 = 3;
    pub const ATTR_PRIORITY_LEVEL: u8 = 4;
    pub const ATTR_UNIT_OF_MEASURE: u8 = 5;
    pub const ATTR_CURRENCY: u8 = 6;
    pub const ATTR_CHARGE_PER_UNIT: u8 = 7;
    pub const ATTR_ACTIVE: u8 = 8;

    /// Create a new Charge object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `charge_type` - Type of charge
    pub fn new(logical_name: ObisCode, charge_type: ChargeType) -> Self {
        Self {
            logical_name,
            total_amount_charged: Arc::new(RwLock::new(0)),
            charge_type: Arc::new(RwLock::new(charge_type)),
            priority_level: Arc::new(RwLock::new(0)),
            unit_of_measure: Arc::new(RwLock::new(String::new())),
            currency: Arc::new(RwLock::new(String::new())),
            charge_per_unit: Arc::new(RwLock::new(0)),
            active: Arc::new(RwLock::new(false)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis(), ChargeType::Fixed)
    }

    /// Get the total amount charged
    pub async fn total_amount_charged(&self) -> i64 {
        *self.total_amount_charged.read().await
    }

    /// Set the total amount charged
    pub async fn set_total_amount_charged(&self, amount: i64) {
        *self.total_amount_charged.write().await = amount;
    }

    /// Add to the charged amount
    pub async fn add_charge(&self, amount: i64) {
        let mut charged = self.total_amount_charged.write().await;
        *charged += amount;
    }

    /// Get the charge type
    pub async fn charge_type(&self) -> ChargeType {
        *self.charge_type.read().await
    }

    /// Set the charge type
    pub async fn set_charge_type(&self, charge_type: ChargeType) {
        *self.charge_type.write().await = charge_type;
    }

    /// Get the priority level
    pub async fn priority_level(&self) -> u8 {
        *self.priority_level.read().await
    }

    /// Set the priority level
    pub async fn set_priority_level(&self, level: u8) {
        *self.priority_level.write().await = level;
    }

    /// Get the unit of measure
    pub async fn unit_of_measure(&self) -> String {
        self.unit_of_measure.read().await.clone()
    }

    /// Set the unit of measure
    pub async fn set_unit_of_measure(&self, unit: String) {
        *self.unit_of_measure.write().await = unit;
    }

    /// Get the currency
    pub async fn currency(&self) -> String {
        self.currency.read().await.clone()
    }

    /// Set the currency
    pub async fn set_currency(&self, currency: String) {
        *self.currency.write().await = currency;
    }

    /// Get the charge per unit
    pub async fn charge_per_unit(&self) -> i32 {
        *self.charge_per_unit.read().await
    }

    /// Set the charge per unit
    pub async fn set_charge_per_unit(&self, rate: i32) {
        *self.charge_per_unit.write().await = rate;
    }

    /// Get whether the charge is active
    pub async fn is_active(&self) -> bool {
        *self.active.read().await
    }

    /// Set whether the charge is active
    pub async fn set_active(&self, active: bool) {
        *self.active.write().await = active;
    }

    /// Activate the charge
    pub async fn activate(&self) {
        self.set_active(true).await;
    }

    /// Deactivate the charge
    pub async fn deactivate(&self) {
        self.set_active(false).await;
    }

    /// Calculate charge for a given quantity (for rate-based charges)
    pub async fn calculate_charge(&self, quantity: i64) -> i64 {
        let rate = self.charge_per_unit().await as i64;
        quantity * rate
    }

    /// Reset the charged amount to zero
    pub async fn reset(&self) {
        *self.total_amount_charged.write().await = 0;
    }
}

#[async_trait]
impl CosemObject for Charge {
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
            Self::ATTR_TOTAL_AMOUNT_CHARGED => {
                Ok(DataObject::Integer64(self.total_amount_charged().await))
            }
            Self::ATTR_CHARGE_TYPE => {
                Ok(DataObject::Enumerate(self.charge_type().await.to_u8()))
            }
            Self::ATTR_PRIORITY_LEVEL => {
                Ok(DataObject::Unsigned8(self.priority_level().await))
            }
            Self::ATTR_UNIT_OF_MEASURE => {
                Ok(DataObject::OctetString(self.unit_of_measure().await.into_bytes()))
            }
            Self::ATTR_CURRENCY => {
                Ok(DataObject::OctetString(self.currency().await.into_bytes()))
            }
            Self::ATTR_CHARGE_PER_UNIT => {
                Ok(DataObject::Integer32(self.charge_per_unit().await))
            }
            Self::ATTR_ACTIVE => {
                Ok(DataObject::Boolean(self.is_active().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Charge has no attribute {}",
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
            Self::ATTR_TOTAL_AMOUNT_CHARGED => {
                match value {
                    DataObject::Integer64(amount) => {
                        self.set_total_amount_charged(amount).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 for total_amount_charged".to_string(),
                    )),
                }
            }
            Self::ATTR_CHARGE_TYPE => {
                match value {
                    DataObject::Enumerate(charge_type) => {
                        self.set_charge_type(ChargeType::from_u8(charge_type)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for charge_type".to_string(),
                    )),
                }
            }
            Self::ATTR_PRIORITY_LEVEL => {
                match value {
                    DataObject::Unsigned8(level) => {
                        self.set_priority_level(level).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for priority_level".to_string(),
                    )),
                }
            }
            Self::ATTR_UNIT_OF_MEASURE => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let unit = String::from_utf8_lossy(&bytes).to_string();
                        self.set_unit_of_measure(unit).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for unit_of_measure".to_string(),
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
            Self::ATTR_CHARGE_PER_UNIT => {
                match value {
                    DataObject::Integer32(rate) => {
                        self.set_charge_per_unit(rate).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer32 for charge_per_unit".to_string(),
                    )),
                }
            }
            Self::ATTR_ACTIVE => {
                match value {
                    DataObject::Boolean(active) => {
                        self.set_active(active).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Boolean for active".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Charge has no attribute {}",
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
            "Charge has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_charge_class_id() {
        let c = Charge::with_default_obis();
        assert_eq!(c.class_id(), 62);
    }

    #[tokio::test]
    async fn test_charge_obis_code() {
        let c = Charge::with_default_obis();
        assert_eq!(c.obis_code(), Charge::default_obis());
    }

    #[tokio::test]
    async fn test_charge_type_from_u8() {
        assert_eq!(ChargeType::from_u8(0), ChargeType::Fixed);
        assert_eq!(ChargeType::from_u8(1), ChargeType::RateBased);
        assert_eq!(ChargeType::from_u8(2), ChargeType::TieredRate);
        assert_eq!(ChargeType::from_u8(3), ChargeType::TimeOfUse);
    }

    #[tokio::test]
    async fn test_charge_type_is_rate_based() {
        assert!(ChargeType::RateBased.is_rate_based());
        assert!(ChargeType::TieredRate.is_rate_based());
        assert!(ChargeType::TimeOfUse.is_rate_based());
        assert!(!ChargeType::Fixed.is_rate_based());
    }

    #[tokio::test]
    async fn test_charge_initial_state() {
        let c = Charge::with_default_obis();
        assert_eq!(c.total_amount_charged().await, 0);
        assert_eq!(c.charge_type().await, ChargeType::Fixed);
        assert!(!c.is_active().await);
    }

    #[tokio::test]
    async fn test_charge_set_total_amount_charged() {
        let c = Charge::with_default_obis();
        c.set_total_amount_charged(1000).await;
        assert_eq!(c.total_amount_charged().await, 1000);
    }

    #[tokio::test]
    async fn test_charge_add_charge() {
        let c = Charge::with_default_obis();
        c.add_charge(100).await;
        assert_eq!(c.total_amount_charged().await, 100);

        c.add_charge(50).await;
        assert_eq!(c.total_amount_charged().await, 150);
    }

    #[tokio::test]
    async fn test_charge_set_type() {
        let c = Charge::with_default_obis();
        c.set_charge_type(ChargeType::RateBased).await;
        assert_eq!(c.charge_type().await, ChargeType::RateBased);
    }

    #[tokio::test]
    async fn test_charge_set_charge_per_unit() {
        let c = Charge::with_default_obis();
        c.set_charge_per_unit(150).await;
        assert_eq!(c.charge_per_unit().await, 150);
    }

    #[tokio::test]
    async fn test_charge_activate_deactivate() {
        let c = Charge::with_default_obis();

        c.activate().await;
        assert!(c.is_active().await);

        c.deactivate().await;
        assert!(!c.is_active().await);
    }

    #[tokio::test]
    async fn test_charge_calculate_charge() {
        let c = Charge::with_default_obis();
        c.set_charge_per_unit(10).await;

        let charge = c.calculate_charge(100).await;
        assert_eq!(charge, 1000); // 100 * 10 = 1000
    }

    #[tokio::test]
    async fn test_charge_reset() {
        let c = Charge::with_default_obis();
        c.add_charge(500).await;
        assert_eq!(c.total_amount_charged().await, 500);

        c.reset().await;
        assert_eq!(c.total_amount_charged().await, 0);
    }

    #[tokio::test]
    async fn test_charge_set_unit_of_measure() {
        let c = Charge::with_default_obis();
        c.set_unit_of_measure("kWh".to_string()).await;
        assert_eq!(c.unit_of_measure().await, "kWh");
    }

    #[tokio::test]
    async fn test_charge_set_currency() {
        let c = Charge::with_default_obis();
        c.set_currency("USD".to_string()).await;
        assert_eq!(c.currency().await, "USD");
    }

    #[tokio::test]
    async fn test_charge_set_priority_level() {
        let c = Charge::with_default_obis();
        c.set_priority_level(3).await;
        assert_eq!(c.priority_level().await, 3);
    }

    #[tokio::test]
    async fn test_charge_get_attributes() {
        let c = Charge::with_default_obis();

        // Test total_amount_charged
        let result = c.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Integer64(amount) => assert_eq!(amount, 0),
            _ => panic!("Expected Integer64"),
        }

        // Test charge_type
        let result = c.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Enumerate(ctype) => assert_eq!(ctype, 0), // Fixed
            _ => panic!("Expected Enumerate"),
        }

        // Test active
        let result = c.get_attribute(8, None).await.unwrap();
        match result {
            DataObject::Boolean(active) => assert!(!active),
            _ => panic!("Expected Boolean"),
        }
    }

    #[tokio::test]
    async fn test_charge_set_attributes() {
        let c = Charge::with_default_obis();

        c.set_attribute(2, DataObject::Integer64(500), None)
            .await
            .unwrap();
        assert_eq!(c.total_amount_charged().await, 500);

        c.set_attribute(7, DataObject::Integer32(100), None)
            .await
            .unwrap();
        assert_eq!(c.charge_per_unit().await, 100);

        c.set_attribute(8, DataObject::Boolean(true), None)
            .await
            .unwrap();
        assert!(c.is_active().await);
    }

    #[tokio::test]
    async fn test_charge_read_only_logical_name() {
        let c = Charge::with_default_obis();
        let result = c
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 62, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_charge_invalid_attribute() {
        let c = Charge::with_default_obis();
        let result = c.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_charge_invalid_method() {
        let c = Charge::with_default_obis();
        let result = c.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }
}
