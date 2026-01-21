//! Credit interface class (Class ID: 61)
//!
//! The Credit interface class manages credit information for prepaid meters.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: credit_available - Amount of credit currently available
//! - Attribute 3: credit_status - Status of the credit
//! - Attribute 4: credit_type - Type of credit (monetary, energy, etc.)
//! - Attribute 5: priority_level - Priority level for credit
//! - Attribute 6: unit_of_measure - Unit of measure for non-monetary credit
//! - Attribute 7: currency - Currency code for monetary credit
//! - Attribute 8: credit_threshold - Minimum credit threshold

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Credit Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CreditType {
    /// Monetary credit
    Monetary = 0,
    /// Energy based credit (kWh)
    Energy = 1,
    /// Volume based credit (m³)
    Volume = 2,
    /// Time based credit (hours)
    Time = 3,
    /// Custom credit type
    Custom = 4,
}

impl CreditType {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Monetary,
            1 => Self::Energy,
            2 => Self::Volume,
            3 => Self::Time,
            _ => Self::Custom,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Credit Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CreditStatusType {
    /// Credit available
    Available = 0,
    /// Credit low
    Low = 1,
    /// Credit exhausted
    Exhausted = 2,
    /// Credit error
    Error = 3,
}

impl CreditStatusType {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Available,
            1 => Self::Low,
            2 => Self::Exhausted,
            3 => Self::Error,
            _ => Self::Error,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if credit is available
    pub fn is_available(self) -> bool {
        matches!(self, Self::Available | Self::Low)
    }
}

/// Credit interface class (Class ID: 61)
///
/// Default OBIS: 0-0:61.0.0.255
///
/// This class manages credit information for prepaid meters.
#[derive(Debug, Clone)]
pub struct Credit {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Credit currently available
    credit_available: Arc<RwLock<i64>>,

    /// Credit status
    credit_status: Arc<RwLock<CreditStatusType>>,

    /// Credit type
    credit_type: Arc<RwLock<CreditType>>,

    /// Priority level (lower = higher priority)
    priority_level: Arc<RwLock<u8>>,

    /// Unit of measure for non-monetary credit
    unit_of_measure: Arc<RwLock<String>>,

    /// Currency code for monetary credit
    currency: Arc<RwLock<String>>,

    /// Credit threshold (warning level)
    credit_threshold: Arc<RwLock<i64>>,
}

impl Credit {
    /// Class ID for Credit
    pub const CLASS_ID: u16 = 61;

    /// Default OBIS code for Credit (0-0:61.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 61, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_CREDIT_AVAILABLE: u8 = 2;
    pub const ATTR_CREDIT_STATUS: u8 = 3;
    pub const ATTR_CREDIT_TYPE: u8 = 4;
    pub const ATTR_PRIORITY_LEVEL: u8 = 5;
    pub const ATTR_UNIT_OF_MEASURE: u8 = 6;
    pub const ATTR_CURRENCY: u8 = 7;
    pub const ATTR_CREDIT_THRESHOLD: u8 = 8;

    /// Create a new Credit object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `credit_type` - Type of credit
    pub fn new(logical_name: ObisCode, credit_type: CreditType) -> Self {
        Self {
            logical_name,
            credit_available: Arc::new(RwLock::new(0)),
            credit_status: Arc::new(RwLock::new(CreditStatusType::Exhausted)),
            credit_type: Arc::new(RwLock::new(credit_type)),
            priority_level: Arc::new(RwLock::new(0)),
            unit_of_measure: Arc::new(RwLock::new(String::new())),
            currency: Arc::new(RwLock::new(String::new())),
            credit_threshold: Arc::new(RwLock::new(100)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis(), CreditType::Monetary)
    }

    /// Get the credit available
    pub async fn credit_available(&self) -> i64 {
        *self.credit_available.read().await
    }

    /// Set the credit available
    pub async fn set_credit_available(&self, credit: i64) {
        *self.credit_available.write().await = credit;
        self.update_status().await;
    }

    /// Add credit
    pub async fn add_credit(&self, amount: i64) {
        let mut credit = self.credit_available.write().await;
        *credit += amount;
        drop(credit);
        self.update_status().await;
    }

    /// Consume/deduct credit
    pub async fn consume_credit(&self, amount: i64) -> DlmsResult<()> {
        let mut credit = self.credit_available.write().await;
        if *credit < amount {
            return Err(DlmsError::InvalidData("Insufficient credit".to_string()));
        }
        *credit -= amount;
        drop(credit);
        self.update_status().await;
        Ok(())
    }

    /// Update credit status based on available credit
    async fn update_status(&self) {
        let available = *self.credit_available.read().await;
        let threshold = *self.credit_threshold.read().await;

        let status = if available <= 0 {
            CreditStatusType::Exhausted
        } else if available < threshold {
            CreditStatusType::Low
        } else {
            CreditStatusType::Available
        };
        *self.credit_status.write().await = status;
    }

    /// Get the credit status
    pub async fn credit_status(&self) -> CreditStatusType {
        *self.credit_status.read().await
    }

    /// Get the credit type
    pub async fn credit_type(&self) -> CreditType {
        *self.credit_type.read().await
    }

    /// Set the credit type
    pub async fn set_credit_type(&self, credit_type: CreditType) {
        *self.credit_type.write().await = credit_type;
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

    /// Get the credit threshold
    pub async fn credit_threshold(&self) -> i64 {
        *self.credit_threshold.read().await
    }

    /// Set the credit threshold
    pub async fn set_credit_threshold(&self, threshold: i64) {
        *self.credit_threshold.write().await = threshold;
    }

    /// Check if credit is available
    pub async fn is_available(&self) -> bool {
        self.credit_status().await.is_available()
    }
}

#[async_trait]
impl CosemObject for Credit {
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
            Self::ATTR_CREDIT_AVAILABLE => {
                Ok(DataObject::Integer64(self.credit_available().await))
            }
            Self::ATTR_CREDIT_STATUS => {
                Ok(DataObject::Enumerate(self.credit_status().await.to_u8()))
            }
            Self::ATTR_CREDIT_TYPE => {
                Ok(DataObject::Enumerate(self.credit_type().await.to_u8()))
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
            Self::ATTR_CREDIT_THRESHOLD => {
                Ok(DataObject::Integer64(self.credit_threshold().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Credit has no attribute {}",
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
            Self::ATTR_CREDIT_AVAILABLE => {
                match value {
                    DataObject::Integer64(credit) => {
                        self.set_credit_available(credit).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 for credit_available".to_string(),
                    )),
                }
            }
            Self::ATTR_CREDIT_STATUS => {
                match value {
                    DataObject::Enumerate(status) => {
                        *self.credit_status.write().await = CreditStatusType::from_u8(status);
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for credit_status".to_string(),
                    )),
                }
            }
            Self::ATTR_CREDIT_TYPE => {
                match value {
                    DataObject::Enumerate(credit_type) => {
                        self.set_credit_type(CreditType::from_u8(credit_type)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for credit_type".to_string(),
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
            Self::ATTR_CREDIT_THRESHOLD => {
                match value {
                    DataObject::Integer64(threshold) => {
                        self.set_credit_threshold(threshold).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 for credit_threshold".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Credit has no attribute {}",
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
            "Credit has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_credit_class_id() {
        let c = Credit::with_default_obis();
        assert_eq!(c.class_id(), 61);
    }

    #[tokio::test]
    async fn test_credit_obis_code() {
        let c = Credit::with_default_obis();
        assert_eq!(c.obis_code(), Credit::default_obis());
    }

    #[tokio::test]
    async fn test_credit_type_from_u8() {
        assert_eq!(CreditType::from_u8(0), CreditType::Monetary);
        assert_eq!(CreditType::from_u8(1), CreditType::Energy);
        assert_eq!(CreditType::from_u8(2), CreditType::Volume);
        assert_eq!(CreditType::from_u8(3), CreditType::Time);
    }

    #[tokio::test]
    async fn test_credit_status_type_from_u8() {
        assert_eq!(CreditStatusType::from_u8(0), CreditStatusType::Available);
        assert_eq!(CreditStatusType::from_u8(1), CreditStatusType::Low);
        assert_eq!(CreditStatusType::from_u8(2), CreditStatusType::Exhausted);
        assert_eq!(CreditStatusType::from_u8(3), CreditStatusType::Error);
    }

    #[tokio::test]
    async fn test_credit_status_is_available() {
        assert!(CreditStatusType::Available.is_available());
        assert!(CreditStatusType::Low.is_available());
        assert!(!CreditStatusType::Exhausted.is_available());
    }

    #[tokio::test]
    async fn test_credit_initial_state() {
        let c = Credit::with_default_obis();
        assert_eq!(c.credit_available().await, 0);
        assert_eq!(c.credit_type().await, CreditType::Monetary);
        assert_eq!(c.priority_level().await, 0);
    }

    #[tokio::test]
    async fn test_credit_set_credit_available() {
        let c = Credit::with_default_obis();
        c.set_credit_available(100).await;
        assert_eq!(c.credit_available().await, 100);
        assert_eq!(c.credit_status().await, CreditStatusType::Available);

        c.set_credit_available(50).await;
        assert_eq!(c.credit_status().await, CreditStatusType::Low);

        c.set_credit_available(0).await;
        assert_eq!(c.credit_status().await, CreditStatusType::Exhausted);
    }

    #[tokio::test]
    async fn test_credit_add_credit() {
        let c = Credit::with_default_obis();
        c.add_credit(100).await;
        assert_eq!(c.credit_available().await, 100);

        c.add_credit(50).await;
        assert_eq!(c.credit_available().await, 150);
    }

    #[tokio::test]
    async fn test_credit_consume_credit() {
        let c = Credit::with_default_obis();
        c.set_credit_available(100).await;

        c.consume_credit(30).await.unwrap();
        assert_eq!(c.credit_available().await, 70);
    }

    #[tokio::test]
    async fn test_credit_consume_credit_insufficient() {
        let c = Credit::with_default_obis();
        c.set_credit_available(50).await;

        let result = c.consume_credit(100).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_credit_set_type() {
        let c = Credit::with_default_obis();
        c.set_credit_type(CreditType::Energy).await;
        assert_eq!(c.credit_type().await, CreditType::Energy);
    }

    #[tokio::test]
    async fn test_credit_set_priority_level() {
        let c = Credit::with_default_obis();
        c.set_priority_level(5).await;
        assert_eq!(c.priority_level().await, 5);
    }

    #[tokio::test]
    async fn test_credit_set_unit_of_measure() {
        let c = Credit::with_default_obis();
        c.set_unit_of_measure("kWh".to_string()).await;
        assert_eq!(c.unit_of_measure().await, "kWh");
    }

    #[tokio::test]
    async fn test_credit_set_currency() {
        let c = Credit::with_default_obis();
        c.set_currency("USD".to_string()).await;
        assert_eq!(c.currency().await, "USD");
    }

    #[tokio::test]
    async fn test_credit_set_threshold() {
        let c = Credit::with_default_obis();
        c.set_credit_threshold(200).await;
        assert_eq!(c.credit_threshold().await, 200);
    }

    #[tokio::test]
    async fn test_credit_is_available() {
        let c = Credit::with_default_obis();
        c.set_credit_available(100).await;
        assert!(c.is_available().await);

        c.set_credit_available(50).await;
        assert!(c.is_available().await);

        c.set_credit_available(0).await;
        assert!(!c.is_available().await);
    }

    #[tokio::test]
    async fn test_credit_get_attributes() {
        let c = Credit::with_default_obis();

        // Test credit_available
        let result = c.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Integer64(credit) => assert_eq!(credit, 0),
            _ => panic!("Expected Integer64"),
        }

        // Test credit_type
        let result = c.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Enumerate(ctype) => assert_eq!(ctype, 0), // Monetary
            _ => panic!("Expected Enumerate"),
        }
    }

    #[tokio::test]
    async fn test_credit_set_attributes() {
        let c = Credit::with_default_obis();

        c.set_attribute(2, DataObject::Integer64(100), None)
            .await
            .unwrap();
        assert_eq!(c.credit_available().await, 100);

        c.set_attribute(5, DataObject::Unsigned8(3), None)
            .await
            .unwrap();
        assert_eq!(c.priority_level().await, 3);
    }

    #[tokio::test]
    async fn test_credit_read_only_logical_name() {
        let c = Credit::with_default_obis();
        let result = c
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 61, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_credit_invalid_attribute() {
        let c = Credit::with_default_obis();
        let result = c.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_credit_invalid_method() {
        let c = Credit::with_default_obis();
        let result = c.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }
}
