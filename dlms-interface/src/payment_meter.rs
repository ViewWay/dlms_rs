//! Payment Meter interface class (Class ID: 82)
//!
//! The Payment Meter interface class manages payment information for prepaid meters.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: payment_method - Payment method type
//! - Attribute 3: payment_status - Current payment status
//! - Attribute 4: total_payments - Total number of payments
//! - Attribute 5: total_amount_paid - Total amount paid
//! - Attribute 6: last_payment_amount - Amount of last payment
//! - Attribute 7: last_payment_date - Date of last payment
//! - Attribute 8: payment_reference - Reference for last payment

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Payment Method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PaymentMethod {
    /// Cash payment
    Cash = 0,
    /// Credit card payment
    CreditCard = 1,
    /// Direct debit
    DirectDebit = 2,
    /// Electronic transfer
    ElectronicTransfer = 3,
    /// Check/Cheque payment
    Check = 4,
    /// Mobile payment
    Mobile = 5,
    /// Token based payment
    Token = 6,
    /// Other payment method
    Other = 7,
}

impl PaymentMethod {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Cash,
            1 => Self::CreditCard,
            2 => Self::DirectDebit,
            3 => Self::ElectronicTransfer,
            4 => Self::Check,
            5 => Self::Mobile,
            6 => Self::Token,
            _ => Self::Other,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if electronic payment
    pub fn is_electronic(self) -> bool {
        matches!(self, Self::CreditCard | Self::DirectDebit | Self::ElectronicTransfer | Self::Mobile | Self::Token)
    }
}

/// Payment Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PaymentStatus {
    /// No payment due
    NoPaymentDue = 0,
    /// Payment pending
    PaymentPending = 1,
    /// Payment overdue
    PaymentOverdue = 2,
    /// Payment in progress
    PaymentInProgress = 3,
    /// Payment failed
    PaymentFailed = 4,
    /// Payment completed
    PaymentCompleted = 5,
}

impl PaymentStatus {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::NoPaymentDue,
            1 => Self::PaymentPending,
            2 => Self::PaymentOverdue,
            3 => Self::PaymentInProgress,
            4 => Self::PaymentFailed,
            5 => Self::PaymentCompleted,
            _ => Self::NoPaymentDue,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if payment is required
    pub fn is_payment_required(self) -> bool {
        matches!(self, Self::PaymentPending | Self::PaymentOverdue | Self::PaymentInProgress)
    }

    /// Check if payment was successful
    pub fn is_successful(self) -> bool {
        matches!(self, Self::NoPaymentDue | Self::PaymentCompleted)
    }
}

/// Payment Meter interface class (Class ID: 82)
///
/// Default OBIS: 0-0:82.0.0.255
///
/// This class manages payment information for prepaid meters.
#[derive(Debug, Clone)]
pub struct PaymentMeter {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Payment method
    payment_method: Arc<RwLock<PaymentMethod>>,

    /// Payment status
    payment_status: Arc<RwLock<PaymentStatus>>,

    /// Total number of payments
    total_payments: Arc<RwLock<u32>>,

    /// Total amount paid
    total_amount_paid: Arc<RwLock<i64>>,

    /// Last payment amount
    last_payment_amount: Arc<RwLock<i64>>,

    /// Last payment date (as Unix timestamp)
    last_payment_date: Arc<RwLock<Option<i64>>>,

    /// Reference for last payment
    payment_reference: Arc<RwLock<String>>,
}

impl PaymentMeter {
    /// Class ID for PaymentMeter
    pub const CLASS_ID: u16 = 82;

    /// Default OBIS code for PaymentMeter (0-0:82.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 82, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_PAYMENT_METHOD: u8 = 2;
    pub const ATTR_PAYMENT_STATUS: u8 = 3;
    pub const ATTR_TOTAL_PAYMENTS: u8 = 4;
    pub const ATTR_TOTAL_AMOUNT_PAID: u8 = 5;
    pub const ATTR_LAST_PAYMENT_AMOUNT: u8 = 6;
    pub const ATTR_LAST_PAYMENT_DATE: u8 = 7;
    pub const ATTR_PAYMENT_REFERENCE: u8 = 8;

    /// Create a new PaymentMeter object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            payment_method: Arc::new(RwLock::new(PaymentMethod::Cash)),
            payment_status: Arc::new(RwLock::new(PaymentStatus::NoPaymentDue)),
            total_payments: Arc::new(RwLock::new(0)),
            total_amount_paid: Arc::new(RwLock::new(0)),
            last_payment_amount: Arc::new(RwLock::new(0)),
            last_payment_date: Arc::new(RwLock::new(None)),
            payment_reference: Arc::new(RwLock::new(String::new())),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get the payment method
    pub async fn payment_method(&self) -> PaymentMethod {
        *self.payment_method.read().await
    }

    /// Set the payment method
    pub async fn set_payment_method(&self, method: PaymentMethod) {
        *self.payment_method.write().await = method;
    }

    /// Get the payment status
    pub async fn payment_status(&self) -> PaymentStatus {
        *self.payment_status.read().await
    }

    /// Set the payment status
    pub async fn set_payment_status(&self, status: PaymentStatus) {
        *self.payment_status.write().await = status;
    }

    /// Get the total number of payments
    pub async fn total_payments(&self) -> u32 {
        *self.total_payments.read().await
    }

    /// Get the total amount paid
    pub async fn total_amount_paid(&self) -> i64 {
        *self.total_amount_paid.read().await
    }

    /// Get the last payment amount
    pub async fn last_payment_amount(&self) -> i64 {
        *self.last_payment_amount.read().await
    }

    /// Get the last payment date
    pub async fn last_payment_date(&self) -> Option<i64> {
        *self.last_payment_date.read().await
    }

    /// Get the payment reference
    pub async fn payment_reference(&self) -> String {
        self.payment_reference.read().await.clone()
    }

    /// Set the payment reference
    pub async fn set_payment_reference(&self, reference: String) {
        *self.payment_reference.write().await = reference;
    }

    /// Record a payment
    pub async fn record_payment(&self, amount: i64, reference: String) {
        let mut total = self.total_payments.write().await;
        *total += 1;
        drop(total);

        *self.total_amount_paid.write().await += amount;
        *self.last_payment_amount.write().await = amount;
        *self.last_payment_date.write().await = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
        );
        *self.payment_reference.write().await = reference;
        *self.payment_status.write().await = PaymentStatus::PaymentCompleted;
    }

    /// Check if payment is required
    pub async fn is_payment_required(&self) -> bool {
        self.payment_status().await.is_payment_required()
    }

    /// Check if last payment was successful
    pub async fn is_last_payment_successful(&self) -> bool {
        self.payment_status().await.is_successful()
    }

    /// Reset payment statistics
    pub async fn reset_statistics(&self) {
        *self.total_payments.write().await = 0;
        *self.total_amount_paid.write().await = 0;
        *self.last_payment_amount.write().await = 0;
        *self.last_payment_date.write().await = None;
        *self.payment_reference.write().await = String::new();
        *self.payment_status.write().await = PaymentStatus::NoPaymentDue;
    }

    /// Set payment as pending
    pub async fn set_payment_pending(&self) {
        *self.payment_status.write().await = PaymentStatus::PaymentPending;
    }

    /// Set payment as failed
    pub async fn set_payment_failed(&self) {
        *self.payment_status.write().await = PaymentStatus::PaymentFailed;
    }

    /// Set payment as in progress
    pub async fn set_payment_in_progress(&self) {
        *self.payment_status.write().await = PaymentStatus::PaymentInProgress;
    }
}

#[async_trait]
impl CosemObject for PaymentMeter {
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
            Self::ATTR_PAYMENT_METHOD => {
                Ok(DataObject::Enumerate(self.payment_method().await.to_u8()))
            }
            Self::ATTR_PAYMENT_STATUS => {
                Ok(DataObject::Enumerate(self.payment_status().await.to_u8()))
            }
            Self::ATTR_TOTAL_PAYMENTS => {
                Ok(DataObject::Unsigned32(self.total_payments().await))
            }
            Self::ATTR_TOTAL_AMOUNT_PAID => {
                Ok(DataObject::Integer64(self.total_amount_paid().await))
            }
            Self::ATTR_LAST_PAYMENT_AMOUNT => {
                Ok(DataObject::Integer64(self.last_payment_amount().await))
            }
            Self::ATTR_LAST_PAYMENT_DATE => {
                match self.last_payment_date().await {
                    Some(timestamp) => Ok(DataObject::Integer64(timestamp)),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_PAYMENT_REFERENCE => {
                Ok(DataObject::OctetString(self.payment_reference().await.into_bytes()))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "PaymentMeter has no attribute {}",
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
            Self::ATTR_PAYMENT_METHOD => {
                match value {
                    DataObject::Enumerate(method) => {
                        self.set_payment_method(PaymentMethod::from_u8(method)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for payment_method".to_string(),
                    )),
                }
            }
            Self::ATTR_PAYMENT_STATUS => {
                match value {
                    DataObject::Enumerate(status) => {
                        self.set_payment_status(PaymentStatus::from_u8(status)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for payment_status".to_string(),
                    )),
                }
            }
            Self::ATTR_TOTAL_PAYMENTS => {
                match value {
                    DataObject::Unsigned32(count) => {
                        *self.total_payments.write().await = count;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned32 for total_payments".to_string(),
                    )),
                }
            }
            Self::ATTR_TOTAL_AMOUNT_PAID => {
                match value {
                    DataObject::Integer64(amount) => {
                        *self.total_amount_paid.write().await = amount;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 for total_amount_paid".to_string(),
                    )),
                }
            }
            Self::ATTR_LAST_PAYMENT_AMOUNT => {
                match value {
                    DataObject::Integer64(amount) => {
                        *self.last_payment_amount.write().await = amount;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 for last_payment_amount".to_string(),
                    )),
                }
            }
            Self::ATTR_LAST_PAYMENT_DATE => {
                match value {
                    DataObject::Integer64(timestamp) => {
                        *self.last_payment_date.write().await = Some(timestamp);
                        Ok(())
                    }
                    DataObject::Null => {
                        *self.last_payment_date.write().await = None;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 or Null for last_payment_date".to_string(),
                    )),
                }
            }
            Self::ATTR_PAYMENT_REFERENCE => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let reference = String::from_utf8_lossy(&bytes).to_string();
                        self.set_payment_reference(reference).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for payment_reference".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "PaymentMeter has no attribute {}",
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
            "PaymentMeter has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_payment_meter_class_id() {
        let pm = PaymentMeter::with_default_obis();
        assert_eq!(pm.class_id(), 82);
    }

    #[tokio::test]
    async fn test_payment_meter_obis_code() {
        let pm = PaymentMeter::with_default_obis();
        assert_eq!(pm.obis_code(), PaymentMeter::default_obis());
    }

    #[tokio::test]
    async fn test_payment_method_from_u8() {
        assert_eq!(PaymentMethod::from_u8(0), PaymentMethod::Cash);
        assert_eq!(PaymentMethod::from_u8(1), PaymentMethod::CreditCard);
        assert_eq!(PaymentMethod::from_u8(2), PaymentMethod::DirectDebit);
        assert_eq!(PaymentMethod::from_u8(3), PaymentMethod::ElectronicTransfer);
        assert_eq!(PaymentMethod::from_u8(4), PaymentMethod::Check);
        assert_eq!(PaymentMethod::from_u8(5), PaymentMethod::Mobile);
        assert_eq!(PaymentMethod::from_u8(6), PaymentMethod::Token);
        assert_eq!(PaymentMethod::from_u8(7), PaymentMethod::Other);
    }

    #[tokio::test]
    async fn test_payment_method_is_electronic() {
        assert!(!PaymentMethod::Cash.is_electronic());
        assert!(PaymentMethod::CreditCard.is_electronic());
        assert!(PaymentMethod::DirectDebit.is_electronic());
        assert!(PaymentMethod::ElectronicTransfer.is_electronic());
        assert!(!PaymentMethod::Check.is_electronic());
        assert!(PaymentMethod::Mobile.is_electronic());
        assert!(PaymentMethod::Token.is_electronic());
    }

    #[tokio::test]
    async fn test_payment_status_from_u8() {
        assert_eq!(PaymentStatus::from_u8(0), PaymentStatus::NoPaymentDue);
        assert_eq!(PaymentStatus::from_u8(1), PaymentStatus::PaymentPending);
        assert_eq!(PaymentStatus::from_u8(2), PaymentStatus::PaymentOverdue);
        assert_eq!(PaymentStatus::from_u8(3), PaymentStatus::PaymentInProgress);
        assert_eq!(PaymentStatus::from_u8(4), PaymentStatus::PaymentFailed);
        assert_eq!(PaymentStatus::from_u8(5), PaymentStatus::PaymentCompleted);
    }

    #[tokio::test]
    async fn test_payment_status_is_payment_required() {
        assert!(!PaymentStatus::NoPaymentDue.is_payment_required());
        assert!(PaymentStatus::PaymentPending.is_payment_required());
        assert!(PaymentStatus::PaymentOverdue.is_payment_required());
        assert!(PaymentStatus::PaymentInProgress.is_payment_required());
        assert!(!PaymentStatus::PaymentFailed.is_payment_required());
        assert!(!PaymentStatus::PaymentCompleted.is_payment_required());
    }

    #[tokio::test]
    async fn test_payment_status_is_successful() {
        assert!(PaymentStatus::NoPaymentDue.is_successful());
        assert!(!PaymentStatus::PaymentPending.is_successful());
        assert!(!PaymentStatus::PaymentOverdue.is_successful());
        assert!(!PaymentStatus::PaymentInProgress.is_successful());
        assert!(!PaymentStatus::PaymentFailed.is_successful());
        assert!(PaymentStatus::PaymentCompleted.is_successful());
    }

    #[tokio::test]
    async fn test_payment_meter_initial_state() {
        let pm = PaymentMeter::with_default_obis();
        assert_eq!(pm.payment_method().await, PaymentMethod::Cash);
        assert_eq!(pm.payment_status().await, PaymentStatus::NoPaymentDue);
        assert_eq!(pm.total_payments().await, 0);
        assert_eq!(pm.total_amount_paid().await, 0);
        assert_eq!(pm.last_payment_amount().await, 0);
    }

    #[tokio::test]
    async fn test_payment_meter_set_payment_method() {
        let pm = PaymentMeter::with_default_obis();
        pm.set_payment_method(PaymentMethod::CreditCard).await;
        assert_eq!(pm.payment_method().await, PaymentMethod::CreditCard);
    }

    #[tokio::test]
    async fn test_payment_meter_set_payment_status() {
        let pm = PaymentMeter::with_default_obis();
        pm.set_payment_status(PaymentStatus::PaymentPending).await;
        assert_eq!(pm.payment_status().await, PaymentStatus::PaymentPending);
    }

    #[tokio::test]
    async fn test_payment_meter_record_payment() {
        let pm = PaymentMeter::with_default_obis();

        pm.record_payment(5000, "REF-001".to_string()).await;

        assert_eq!(pm.total_payments().await, 1);
        assert_eq!(pm.total_amount_paid().await, 5000);
        assert_eq!(pm.last_payment_amount().await, 5000);
        assert_eq!(pm.payment_reference().await, "REF-001");
        assert_eq!(pm.payment_status().await, PaymentStatus::PaymentCompleted);
        assert!(pm.last_payment_date().await.is_some());
    }

    #[tokio::test]
    async fn test_payment_meter_record_multiple_payments() {
        let pm = PaymentMeter::with_default_obis();

        pm.record_payment(1000, "REF-001".to_string()).await;
        pm.record_payment(2000, "REF-002".to_string()).await;

        assert_eq!(pm.total_payments().await, 2);
        assert_eq!(pm.total_amount_paid().await, 3000);
        assert_eq!(pm.last_payment_amount().await, 2000);
        assert_eq!(pm.payment_reference().await, "REF-002");
    }

    #[tokio::test]
    async fn test_payment_meter_is_payment_required() {
        let pm = PaymentMeter::with_default_obis();

        assert!(!pm.is_payment_required().await);

        pm.set_payment_status(PaymentStatus::PaymentPending).await;
        assert!(pm.is_payment_required().await);

        pm.set_payment_status(PaymentStatus::PaymentOverdue).await;
        assert!(pm.is_payment_required().await);

        pm.set_payment_status(PaymentStatus::PaymentCompleted).await;
        assert!(!pm.is_payment_required().await);
    }

    #[tokio::test]
    async fn test_payment_meter_is_last_payment_successful() {
        let pm = PaymentMeter::with_default_obis();

        assert!(pm.is_last_payment_successful().await);

        pm.set_payment_status(PaymentStatus::PaymentFailed).await;
        assert!(!pm.is_last_payment_successful().await);

        pm.record_payment(1000, "REF-001".to_string()).await;
        assert!(pm.is_last_payment_successful().await);
    }

    #[tokio::test]
    async fn test_payment_meter_reset_statistics() {
        let pm = PaymentMeter::with_default_obis();

        pm.record_payment(5000, "REF-001".to_string()).await;
        assert_eq!(pm.total_payments().await, 1);

        pm.reset_statistics().await;

        assert_eq!(pm.total_payments().await, 0);
        assert_eq!(pm.total_amount_paid().await, 0);
        assert_eq!(pm.last_payment_amount().await, 0);
        assert_eq!(pm.payment_status().await, PaymentStatus::NoPaymentDue);
        assert_eq!(pm.payment_reference().await, "");
    }

    #[tokio::test]
    async fn test_payment_meter_set_payment_pending() {
        let pm = PaymentMeter::with_default_obis();
        pm.set_payment_pending().await;
        assert_eq!(pm.payment_status().await, PaymentStatus::PaymentPending);
    }

    #[tokio::test]
    async fn test_payment_meter_set_payment_failed() {
        let pm = PaymentMeter::with_default_obis();
        pm.set_payment_failed().await;
        assert_eq!(pm.payment_status().await, PaymentStatus::PaymentFailed);
    }

    #[tokio::test]
    async fn test_payment_meter_set_payment_in_progress() {
        let pm = PaymentMeter::with_default_obis();
        pm.set_payment_in_progress().await;
        assert_eq!(pm.payment_status().await, PaymentStatus::PaymentInProgress);
    }

    #[tokio::test]
    async fn test_payment_meter_get_attributes() {
        let pm = PaymentMeter::with_default_obis();

        // Test payment_method
        let result = pm.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Enumerate(method) => assert_eq!(method, 0), // Cash
            _ => panic!("Expected Enumerate"),
        }

        // Test payment_status
        let result = pm.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Enumerate(status) => assert_eq!(status, 0), // NoPaymentDue
            _ => panic!("Expected Enumerate"),
        }

        // Test total_payments
        let result = pm.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Unsigned32(count) => assert_eq!(count, 0),
            _ => panic!("Expected Unsigned32"),
        }
    }

    #[tokio::test]
    async fn test_payment_meter_set_attributes() {
        let pm = PaymentMeter::with_default_obis();

        pm.set_attribute(2, DataObject::Enumerate(1), None) // CreditCard
            .await
            .unwrap();
        assert_eq!(pm.payment_method().await, PaymentMethod::CreditCard);

        pm.set_attribute(3, DataObject::Enumerate(1), None) // PaymentPending
            .await
            .unwrap();
        assert_eq!(pm.payment_status().await, PaymentStatus::PaymentPending);

        pm.set_attribute(4, DataObject::Unsigned32(5), None)
            .await
            .unwrap();
        assert_eq!(pm.total_payments().await, 5);
    }

    #[tokio::test]
    async fn test_payment_meter_read_only_logical_name() {
        let pm = PaymentMeter::with_default_obis();
        let result = pm
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 82, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_payment_meter_invalid_attribute() {
        let pm = PaymentMeter::with_default_obis();
        let result = pm.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_payment_meter_invalid_method() {
        let pm = PaymentMeter::with_default_obis();
        let result = pm.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_payment_meter_null_last_payment_date() {
        let pm = PaymentMeter::with_default_obis();

        // Default date is None, should return Null DataObject
        let result = pm.get_attribute(7, None).await.unwrap();
        assert!(matches!(result, DataObject::Null));
    }
}
