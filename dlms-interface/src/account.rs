//! Account interface class (Class ID: 60)
//!
//! The Account interface class manages prepaid customer accounts for smart meters.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: account_id - Unique account identifier
//! - Attribute 3: current_credit - Current credit balance
//! - Attribute 4: credit_status - Credit status (enabled/disabled/low credit)
//! - Attribute 5: credit_threshold - Low credit threshold
//! - Attribute 6: currency - Currency code
//! - Attribute 7: low_credit_threshold - Low credit warning threshold
//! - Attribute 8: maximum_credit - Maximum allowed credit

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Credit Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CreditStatus {
    /// Credit available - service enabled
    Enabled = 0,
    /// Credit disabled - service cut off
    Disabled = 1,
    /// Low credit warning
    LowCredit = 2,
    /// Emergency credit active
    EmergencyCredit = 3,
}

impl CreditStatus {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Enabled,
            1 => Self::Disabled,
            2 => Self::LowCredit,
            3 => Self::EmergencyCredit,
            _ => Self::Disabled,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if service is enabled
    pub fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled | Self::LowCredit | Self::EmergencyCredit)
    }

    /// Check if low credit
    pub fn is_low_credit(self) -> bool {
        matches!(self, Self::LowCredit | Self::EmergencyCredit)
    }
}

/// Account interface class (Class ID: 60)
///
/// Default OBIS: 0-0:60.0.0.255
///
/// This class manages prepaid customer accounts.
#[derive(Debug, Clone)]
pub struct Account {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Account identifier
    account_id: Arc<RwLock<String>>,

    /// Current credit balance (in smallest currency unit)
    current_credit: Arc<RwLock<i64>>,

    /// Credit status
    credit_status: Arc<RwLock<CreditStatus>>,

    /// Credit threshold (zero or below means disabled)
    credit_threshold: Arc<RwLock<i64>>,

    /// Currency code (e.g., "USD", "EUR")
    currency: Arc<RwLock<String>>,

    /// Low credit warning threshold
    low_credit_threshold: Arc<RwLock<i64>>,

    /// Maximum allowed credit
    maximum_credit: Arc<RwLock<i64>>,
}

impl Account {
    /// Class ID for Account
    pub const CLASS_ID: u16 = 60;

    /// Default OBIS code for Account (0-0:60.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 60, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_ACCOUNT_ID: u8 = 2;
    pub const ATTR_CURRENT_CREDIT: u8 = 3;
    pub const ATTR_CREDIT_STATUS: u8 = 4;
    pub const ATTR_CREDIT_THRESHOLD: u8 = 5;
    pub const ATTR_CURRENCY: u8 = 6;
    pub const ATTR_LOW_CREDIT_THRESHOLD: u8 = 7;
    pub const ATTR_MAXIMUM_CREDIT: u8 = 8;

    /// Create a new Account object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            account_id: Arc::new(RwLock::new(String::new())),
            current_credit: Arc::new(RwLock::new(0)),
            credit_status: Arc::new(RwLock::new(CreditStatus::Disabled)),
            credit_threshold: Arc::new(RwLock::new(0)),
            currency: Arc::new(RwLock::new(String::new())),
            low_credit_threshold: Arc::new(RwLock::new(100)),
            maximum_credit: Arc::new(RwLock::new(i64::MAX)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get the account ID
    pub async fn account_id(&self) -> String {
        self.account_id.read().await.clone()
    }

    /// Set the account ID
    pub async fn set_account_id(&self, id: String) {
        *self.account_id.write().await = id;
    }

    /// Get the current credit
    pub async fn current_credit(&self) -> i64 {
        *self.current_credit.read().await
    }

    /// Set the current credit
    pub async fn set_current_credit(&self, credit: i64) {
        *self.current_credit.write().await = credit;
        self.update_status().await;
    }

    /// Add credit to the account
    pub async fn add_credit(&self, amount: i64) -> DlmsResult<()> {
        let max = *self.maximum_credit.read().await;
        let mut current = self.current_credit.write().await;
        let new_credit = *current + amount;
        if new_credit > max {
            return Err(DlmsError::InvalidData("Credit exceeds maximum".to_string()));
        }
        *current = new_credit;
        drop(current);
        self.update_status().await;
        Ok(())
    }

    /// Consume/deduct credit from the account
    pub async fn consume_credit(&self, amount: i64) -> DlmsResult<()> {
        let mut current = self.current_credit.write().await;
        let new_credit = *current - amount;
        if new_credit < *self.credit_threshold.read().await {
            return Err(DlmsError::InvalidData("Insufficient credit".to_string()));
        }
        *current = new_credit;
        drop(current);
        self.update_status().await;
        Ok(())
    }

    /// Update credit status based on current credit
    async fn update_status(&self) {
        let current = *self.current_credit.read().await;
        let threshold = *self.credit_threshold.read().await;
        let low_threshold = *self.low_credit_threshold.read().await;

        let status = if current < threshold {
            CreditStatus::Disabled
        } else if current < low_threshold {
            CreditStatus::LowCredit
        } else {
            CreditStatus::Enabled
        };
        *self.credit_status.write().await = status;
    }

    /// Get the credit status
    pub async fn credit_status(&self) -> CreditStatus {
        *self.credit_status.read().await
    }

    /// Get the credit threshold
    pub async fn credit_threshold(&self) -> i64 {
        *self.credit_threshold.read().await
    }

    /// Set the credit threshold
    pub async fn set_credit_threshold(&self, threshold: i64) {
        *self.credit_threshold.write().await = threshold;
        self.update_status().await;
    }

    /// Get the currency
    pub async fn currency(&self) -> String {
        self.currency.read().await.clone()
    }

    /// Set the currency
    pub async fn set_currency(&self, currency: String) {
        *self.currency.write().await = currency;
    }

    /// Get the low credit threshold
    pub async fn low_credit_threshold(&self) -> i64 {
        *self.low_credit_threshold.read().await
    }

    /// Set the low credit threshold
    pub async fn set_low_credit_threshold(&self, threshold: i64) {
        *self.low_credit_threshold.write().await = threshold;
    }

    /// Get the maximum credit
    pub async fn maximum_credit(&self) -> i64 {
        *self.maximum_credit.read().await
    }

    /// Set the maximum credit
    pub async fn set_maximum_credit(&self, max: i64) {
        *self.maximum_credit.write().await = max;
    }

    /// Check if service is enabled
    pub async fn is_service_enabled(&self) -> bool {
        self.credit_status().await.is_enabled()
    }

    /// Check if low credit warning should be shown
    pub async fn is_low_credit(&self) -> bool {
        self.credit_status().await.is_low_credit()
    }
}

#[async_trait]
impl CosemObject for Account {
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
            Self::ATTR_ACCOUNT_ID => {
                Ok(DataObject::OctetString(self.account_id().await.into_bytes()))
            }
            Self::ATTR_CURRENT_CREDIT => {
                Ok(DataObject::Integer64(self.current_credit().await))
            }
            Self::ATTR_CREDIT_STATUS => {
                Ok(DataObject::Enumerate(self.credit_status().await.to_u8()))
            }
            Self::ATTR_CREDIT_THRESHOLD => {
                Ok(DataObject::Integer64(self.credit_threshold().await))
            }
            Self::ATTR_CURRENCY => {
                Ok(DataObject::OctetString(self.currency().await.into_bytes()))
            }
            Self::ATTR_LOW_CREDIT_THRESHOLD => {
                Ok(DataObject::Integer64(self.low_credit_threshold().await))
            }
            Self::ATTR_MAXIMUM_CREDIT => {
                Ok(DataObject::Integer64(self.maximum_credit().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Account has no attribute {}",
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
            Self::ATTR_ACCOUNT_ID => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let id = String::from_utf8_lossy(&bytes).to_string();
                        self.set_account_id(id).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for account_id".to_string(),
                    )),
                }
            }
            Self::ATTR_CURRENT_CREDIT => {
                match value {
                    DataObject::Integer64(credit) => {
                        self.set_current_credit(credit).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 for current_credit".to_string(),
                    )),
                }
            }
            Self::ATTR_CREDIT_STATUS => {
                match value {
                    DataObject::Enumerate(status) => {
                        *self.credit_status.write().await = CreditStatus::from_u8(status);
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for credit_status".to_string(),
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
            Self::ATTR_LOW_CREDIT_THRESHOLD => {
                match value {
                    DataObject::Integer64(threshold) => {
                        self.set_low_credit_threshold(threshold).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 for low_credit_threshold".to_string(),
                    )),
                }
            }
            Self::ATTR_MAXIMUM_CREDIT => {
                match value {
                    DataObject::Integer64(max) => {
                        self.set_maximum_credit(max).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 for maximum_credit".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Account has no attribute {}",
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
            "Account has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_account_class_id() {
        let acc = Account::with_default_obis();
        assert_eq!(acc.class_id(), 60);
    }

    #[tokio::test]
    async fn test_account_obis_code() {
        let acc = Account::with_default_obis();
        assert_eq!(acc.obis_code(), Account::default_obis());
    }

    #[tokio::test]
    async fn test_credit_status_from_u8() {
        assert_eq!(CreditStatus::from_u8(0), CreditStatus::Enabled);
        assert_eq!(CreditStatus::from_u8(1), CreditStatus::Disabled);
        assert_eq!(CreditStatus::from_u8(2), CreditStatus::LowCredit);
        assert_eq!(CreditStatus::from_u8(3), CreditStatus::EmergencyCredit);
    }

    #[tokio::test]
    async fn test_credit_status_is_enabled() {
        assert!(CreditStatus::Enabled.is_enabled());
        assert!(CreditStatus::LowCredit.is_enabled());
        assert!(CreditStatus::EmergencyCredit.is_enabled());
        assert!(!CreditStatus::Disabled.is_enabled());
    }

    #[tokio::test]
    async fn test_credit_status_is_low_credit() {
        assert!(CreditStatus::LowCredit.is_low_credit());
        assert!(CreditStatus::EmergencyCredit.is_low_credit());
        assert!(!CreditStatus::Enabled.is_low_credit());
    }

    #[tokio::test]
    async fn test_account_initial_state() {
        let acc = Account::with_default_obis();
        assert_eq!(acc.current_credit().await, 0);
        assert_eq!(acc.credit_status().await, CreditStatus::Disabled);
        assert_eq!(acc.low_credit_threshold().await, 100);
    }

    #[tokio::test]
    async fn test_account_set_account_id() {
        let acc = Account::with_default_obis();
        acc.set_account_id("ACCT-12345".to_string()).await;
        assert_eq!(acc.account_id().await, "ACCT-12345");
    }

    #[tokio::test]
    async fn test_account_set_current_credit() {
        let acc = Account::with_default_obis();
        acc.set_credit_threshold(0).await;
        acc.set_low_credit_threshold(50).await;

        acc.set_current_credit(100).await;
        assert_eq!(acc.current_credit().await, 100);
        assert_eq!(acc.credit_status().await, CreditStatus::Enabled);

        acc.set_current_credit(30).await;
        assert_eq!(acc.credit_status().await, CreditStatus::LowCredit);

        acc.set_current_credit(-10).await;
        assert_eq!(acc.credit_status().await, CreditStatus::Disabled);
    }

    #[tokio::test]
    async fn test_account_add_credit() {
        let acc = Account::with_default_obis();
        acc.set_maximum_credit(1000).await;

        acc.add_credit(100).await.unwrap();
        assert_eq!(acc.current_credit().await, 100);

        acc.add_credit(50).await.unwrap();
        assert_eq!(acc.current_credit().await, 150);
    }

    #[tokio::test]
    async fn test_account_add_credit_exceeds_max() {
        let acc = Account::with_default_obis();
        acc.set_maximum_credit(100).await;

        acc.add_credit(50).await.unwrap();
        let result = acc.add_credit(100).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_account_consume_credit() {
        let acc = Account::with_default_obis();
        acc.set_credit_threshold(0).await;

        acc.set_current_credit(100).await;
        acc.consume_credit(30).await.unwrap();
        assert_eq!(acc.current_credit().await, 70);
    }

    #[tokio::test]
    async fn test_account_consume_credit_insufficient() {
        let acc = Account::with_default_obis();
        acc.set_credit_threshold(50).await;

        acc.set_current_credit(100).await;
        let result = acc.consume_credit(60).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_account_set_currency() {
        let acc = Account::with_default_obis();
        acc.set_currency("USD".to_string()).await;
        assert_eq!(acc.currency().await, "USD");
    }

    #[tokio::test]
    async fn test_account_set_thresholds() {
        let acc = Account::with_default_obis();

        acc.set_credit_threshold(10).await;
        assert_eq!(acc.credit_threshold().await, 10);

        acc.set_low_credit_threshold(50).await;
        assert_eq!(acc.low_credit_threshold().await, 50);

        acc.set_maximum_credit(1000).await;
        assert_eq!(acc.maximum_credit().await, 1000);
    }

    #[tokio::test]
    async fn test_account_is_service_enabled() {
        let acc = Account::with_default_obis();
        // Set positive threshold so that 0 credit means disabled
        acc.set_credit_threshold(10).await;

        assert!(!acc.is_service_enabled().await);

        acc.set_current_credit(100).await;
        assert!(acc.is_service_enabled().await);
    }

    #[tokio::test]
    async fn test_account_is_low_credit() {
        let acc = Account::with_default_obis();
        acc.set_credit_threshold(0).await;
        acc.set_low_credit_threshold(50).await;

        acc.set_current_credit(100).await;
        assert!(!acc.is_low_credit().await);

        acc.set_current_credit(30).await;
        assert!(acc.is_low_credit().await);
    }

    #[tokio::test]
    async fn test_account_get_attributes() {
        let acc = Account::with_default_obis();

        // Test current_credit
        let result = acc.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Integer64(credit) => assert_eq!(credit, 0),
            _ => panic!("Expected Integer64"),
        }

        // Test credit_status
        let result = acc.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Enumerate(status) => assert_eq!(status, 1), // Disabled
            _ => panic!("Expected Enumerate"),
        }
    }

    #[tokio::test]
    async fn test_account_set_attributes() {
        let acc = Account::with_default_obis();

        acc.set_attribute(2, DataObject::OctetString(b"ACCT-123".to_vec()), None)
            .await
            .unwrap();
        assert_eq!(acc.account_id().await, "ACCT-123");

        acc.set_attribute(3, DataObject::Integer64(100), None)
            .await
            .unwrap();
        assert_eq!(acc.current_credit().await, 100);

        acc.set_attribute(6, DataObject::OctetString(b"EUR".to_vec()), None)
            .await
            .unwrap();
        assert_eq!(acc.currency().await, "EUR");
    }

    #[tokio::test]
    async fn test_account_read_only_logical_name() {
        let acc = Account::with_default_obis();
        let result = acc
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 60, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_account_invalid_attribute() {
        let acc = Account::with_default_obis();
        let result = acc.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_account_invalid_method() {
        let acc = Account::with_default_obis();
        let result = acc.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }
}
