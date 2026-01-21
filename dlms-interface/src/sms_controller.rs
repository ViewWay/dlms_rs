//! SMS Controller interface class (Class ID: 27)
//!
//! The SMS Controller interface class manages SMS communication for meters.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: sms_enabled - Whether SMS is enabled
//! - Attribute 3: phone_number - Phone number for SMS
//! - Attribute 4: message_template - Message template for SMS
//! - Attribute 5: send_status - Status of last send operation
//! - Attribute 6: send_count - Number of messages sent
//! - Attribute 7: receive_count - Number of messages received
//! - Attribute 8: max_message_size - Maximum message size

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// SMS Send Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SmsSendStatus {
    /// No message sent
    Idle = 0,
    /// Send in progress
    Sending = 1,
    /// Send successful
    Success = 2,
    /// Send failed
    Failed = 3,
    /// Send timeout
    Timeout = 4,
}

impl SmsSendStatus {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Idle,
            1 => Self::Sending,
            2 => Self::Success,
            3 => Self::Failed,
            4 => Self::Timeout,
            _ => Self::Idle,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if send was successful
    pub fn is_successful(self) -> bool {
        matches!(self, Self::Success)
    }

    /// Check if send failed
    pub fn is_failed(self) -> bool {
        matches!(self, Self::Failed | Self::Timeout)
    }
}

/// SMS Controller interface class (Class ID: 27)
///
/// Default OBIS: 0-0:27.0.0.255
///
/// This class manages SMS communication for meters.
#[derive(Debug, Clone)]
pub struct SmsController {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Whether SMS is enabled
    sms_enabled: Arc<RwLock<bool>>,

    /// Phone number for SMS
    phone_number: Arc<RwLock<String>>,

    /// Message template for SMS
    message_template: Arc<RwLock<String>>,

    /// Status of last send operation
    send_status: Arc<RwLock<SmsSendStatus>>,

    /// Number of messages sent
    send_count: Arc<RwLock<u32>>,

    /// Number of messages received
    receive_count: Arc<RwLock<u32>>,

    /// Maximum message size
    max_message_size: Arc<RwLock<u16>>,
}

impl SmsController {
    /// Class ID for SmsController
    pub const CLASS_ID: u16 = 27;

    /// Default OBIS code for SmsController (0-0:27.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 27, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_SMS_ENABLED: u8 = 2;
    pub const ATTR_PHONE_NUMBER: u8 = 3;
    pub const ATTR_MESSAGE_TEMPLATE: u8 = 4;
    pub const ATTR_SEND_STATUS: u8 = 5;
    pub const ATTR_SEND_COUNT: u8 = 6;
    pub const ATTR_RECEIVE_COUNT: u8 = 7;
    pub const ATTR_MAX_MESSAGE_SIZE: u8 = 8;

    /// Create a new SmsController object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            sms_enabled: Arc::new(RwLock::new(false)),
            phone_number: Arc::new(RwLock::new(String::new())),
            message_template: Arc::new(RwLock::new(String::new())),
            send_status: Arc::new(RwLock::new(SmsSendStatus::Idle)),
            send_count: Arc::new(RwLock::new(0)),
            receive_count: Arc::new(RwLock::new(0)),
            max_message_size: Arc::new(RwLock::new(160)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get whether SMS is enabled
    pub async fn sms_enabled(&self) -> bool {
        *self.sms_enabled.read().await
    }

    /// Set whether SMS is enabled
    pub async fn set_sms_enabled(&self, enabled: bool) {
        *self.sms_enabled.write().await = enabled;
    }

    /// Get the phone number
    pub async fn phone_number(&self) -> String {
        self.phone_number.read().await.clone()
    }

    /// Set the phone number
    pub async fn set_phone_number(&self, number: String) {
        *self.phone_number.write().await = number;
    }

    /// Get the message template
    pub async fn message_template(&self) -> String {
        self.message_template.read().await.clone()
    }

    /// Set the message template
    pub async fn set_message_template(&self, template: String) {
        *self.message_template.write().await = template;
    }

    /// Get the send status
    pub async fn send_status(&self) -> SmsSendStatus {
        *self.send_status.read().await
    }

    /// Set the send status
    pub async fn set_send_status(&self, status: SmsSendStatus) {
        *self.send_status.write().await = status;
    }

    /// Get the send count
    pub async fn send_count(&self) -> u32 {
        *self.send_count.read().await
    }

    /// Get the receive count
    pub async fn receive_count(&self) -> u32 {
        *self.receive_count.read().await
    }

    /// Get the maximum message size
    pub async fn max_message_size(&self) -> u16 {
        *self.max_message_size.read().await
    }

    /// Set the maximum message size
    pub async fn set_max_message_size(&self, size: u16) {
        *self.max_message_size.write().await = size;
    }

    /// Increment send count
    pub async fn increment_send_count(&self) {
        let mut count = self.send_count.write().await;
        *count += 1;
    }

    /// Increment receive count
    pub async fn increment_receive_count(&self) {
        let mut count = self.receive_count.write().await;
        *count += 1;
    }

    /// Reset counters
    pub async fn reset_counters(&self) {
        *self.send_count.write().await = 0;
        *self.receive_count.write().await = 0;
    }

    /// Check if SMS is enabled
    pub async fn is_enabled(&self) -> bool {
        self.sms_enabled().await
    }

    /// Format a message using the template
    pub async fn format_message(&self, data: &str) -> String {
        let template = self.message_template().await;
        template.replace("{}", data)
    }

    /// Prepare to send (set status to Sending)
    pub async fn prepare_send(&self) {
        *self.send_status.write().await = SmsSendStatus::Sending;
    }

    /// Mark send as successful
    pub async fn mark_send_success(&self) {
        *self.send_status.write().await = SmsSendStatus::Success;
        self.increment_send_count().await;
    }

    /// Mark send as failed
    pub async fn mark_send_failed(&self) {
        *self.send_status.write().await = SmsSendStatus::Failed;
    }

    /// Mark send as timeout
    pub async fn mark_send_timeout(&self) {
        *self.send_status.write().await = SmsSendStatus::Timeout;
    }

    /// Validate phone number format (basic check)
    pub async fn validate_phone_number(&self) -> bool {
        let number = self.phone_number().await;
        // Basic validation: should only contain digits, +, -, spaces, and parentheses
        number.chars().all(|c| c.is_ascii_digit() || matches!(c, '+') | matches!(c, '-') | matches!(c, ' ') | matches!(c, '(') | matches!(c, ')'))
            && !number.is_empty()
            && number.len() <= 20
    }

    /// Clear phone number
    pub async fn clear_phone_number(&self) {
        *self.phone_number.write().await = String::new();
    }
}

#[async_trait]
impl CosemObject for SmsController {
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
            Self::ATTR_SMS_ENABLED => {
                Ok(DataObject::Boolean(self.sms_enabled().await))
            }
            Self::ATTR_PHONE_NUMBER => {
                Ok(DataObject::OctetString(self.phone_number().await.into_bytes()))
            }
            Self::ATTR_MESSAGE_TEMPLATE => {
                Ok(DataObject::OctetString(self.message_template().await.into_bytes()))
            }
            Self::ATTR_SEND_STATUS => {
                Ok(DataObject::Enumerate(self.send_status().await.to_u8()))
            }
            Self::ATTR_SEND_COUNT => {
                Ok(DataObject::Unsigned32(self.send_count().await))
            }
            Self::ATTR_RECEIVE_COUNT => {
                Ok(DataObject::Unsigned32(self.receive_count().await))
            }
            Self::ATTR_MAX_MESSAGE_SIZE => {
                Ok(DataObject::Unsigned16(self.max_message_size().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "SmsController has no attribute {}",
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
            Self::ATTR_SMS_ENABLED => {
                match value {
                    DataObject::Boolean(enabled) => {
                        self.set_sms_enabled(enabled).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Boolean for sms_enabled".to_string(),
                    )),
                }
            }
            Self::ATTR_PHONE_NUMBER => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let number = String::from_utf8_lossy(&bytes).to_string();
                        self.set_phone_number(number).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for phone_number".to_string(),
                    )),
                }
            }
            Self::ATTR_MESSAGE_TEMPLATE => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let template = String::from_utf8_lossy(&bytes).to_string();
                        self.set_message_template(template).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for message_template".to_string(),
                    )),
                }
            }
            Self::ATTR_SEND_STATUS => {
                match value {
                    DataObject::Enumerate(status) => {
                        self.set_send_status(SmsSendStatus::from_u8(status)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for send_status".to_string(),
                    )),
                }
            }
            Self::ATTR_SEND_COUNT => {
                match value {
                    DataObject::Unsigned32(count) => {
                        *self.send_count.write().await = count;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned32 for send_count".to_string(),
                    )),
                }
            }
            Self::ATTR_RECEIVE_COUNT => {
                match value {
                    DataObject::Unsigned32(count) => {
                        *self.receive_count.write().await = count;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned32 for receive_count".to_string(),
                    )),
                }
            }
            Self::ATTR_MAX_MESSAGE_SIZE => {
                match value {
                    DataObject::Unsigned16(size) => {
                        self.set_max_message_size(size).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned16 for max_message_size".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "SmsController has no attribute {}",
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
            "SmsController has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sms_controller_class_id() {
        let sms = SmsController::with_default_obis();
        assert_eq!(sms.class_id(), 27);
    }

    #[tokio::test]
    async fn test_sms_controller_obis_code() {
        let sms = SmsController::with_default_obis();
        assert_eq!(sms.obis_code(), SmsController::default_obis());
    }

    #[tokio::test]
    async fn test_sms_send_status_from_u8() {
        assert_eq!(SmsSendStatus::from_u8(0), SmsSendStatus::Idle);
        assert_eq!(SmsSendStatus::from_u8(1), SmsSendStatus::Sending);
        assert_eq!(SmsSendStatus::from_u8(2), SmsSendStatus::Success);
        assert_eq!(SmsSendStatus::from_u8(3), SmsSendStatus::Failed);
        assert_eq!(SmsSendStatus::from_u8(4), SmsSendStatus::Timeout);
    }

    #[tokio::test]
    async fn test_sms_send_status_is_successful() {
        assert!(SmsSendStatus::Success.is_successful());
        assert!(!SmsSendStatus::Idle.is_successful());
        assert!(!SmsSendStatus::Sending.is_successful());
        assert!(!SmsSendStatus::Failed.is_successful());
        assert!(!SmsSendStatus::Timeout.is_successful());
    }

    #[tokio::test]
    async fn test_sms_send_status_is_failed() {
        assert!(SmsSendStatus::Failed.is_failed());
        assert!(SmsSendStatus::Timeout.is_failed());
        assert!(!SmsSendStatus::Idle.is_failed());
        assert!(!SmsSendStatus::Sending.is_failed());
        assert!(!SmsSendStatus::Success.is_failed());
    }

    #[tokio::test]
    async fn test_sms_controller_initial_state() {
        let sms = SmsController::with_default_obis();
        assert!(!sms.sms_enabled().await);
        assert_eq!(sms.phone_number().await, "");
        assert_eq!(sms.message_template().await, "");
        assert_eq!(sms.send_status().await, SmsSendStatus::Idle);
        assert_eq!(sms.send_count().await, 0);
        assert_eq!(sms.receive_count().await, 0);
        assert_eq!(sms.max_message_size().await, 160);
    }

    #[tokio::test]
    async fn test_sms_controller_set_sms_enabled() {
        let sms = SmsController::with_default_obis();
        sms.set_sms_enabled(true).await;
        assert!(sms.sms_enabled().await);
    }

    #[tokio::test]
    async fn test_sms_controller_set_phone_number() {
        let sms = SmsController::with_default_obis();
        sms.set_phone_number("+1234567890".to_string()).await;
        assert_eq!(sms.phone_number().await, "+1234567890");
    }

    #[tokio::test]
    async fn test_sms_controller_set_message_template() {
        let sms = SmsController::with_default_obis();
        sms.set_message_template("Meter reading: {}".to_string()).await;
        assert_eq!(sms.message_template().await, "Meter reading: {}");
    }

    #[tokio::test]
    async fn test_sms_controller_increment_send_count() {
        let sms = SmsController::with_default_obis();
        assert_eq!(sms.send_count().await, 0);

        sms.increment_send_count().await;
        assert_eq!(sms.send_count().await, 1);

        sms.increment_send_count().await;
        assert_eq!(sms.send_count().await, 2);
    }

    #[tokio::test]
    async fn test_sms_controller_increment_receive_count() {
        let sms = SmsController::with_default_obis();
        assert_eq!(sms.receive_count().await, 0);

        sms.increment_receive_count().await;
        assert_eq!(sms.receive_count().await, 1);
    }

    #[tokio::test]
    async fn test_sms_controller_reset_counters() {
        let sms = SmsController::with_default_obis();
        sms.increment_send_count().await;
        sms.increment_receive_count().await;

        sms.reset_counters().await;

        assert_eq!(sms.send_count().await, 0);
        assert_eq!(sms.receive_count().await, 0);
    }

    #[tokio::test]
    async fn test_sms_controller_format_message() {
        let sms = SmsController::with_default_obis();
        sms.set_message_template("Value: {} kWh".to_string()).await;

        let formatted = sms.format_message("123.45").await;
        assert_eq!(formatted, "Value: 123.45 kWh");
    }

    #[tokio::test]
    async fn test_sms_controller_prepare_send() {
        let sms = SmsController::with_default_obis();
        sms.prepare_send().await;
        assert_eq!(sms.send_status().await, SmsSendStatus::Sending);
    }

    #[tokio::test]
    async fn test_sms_controller_mark_send_success() {
        let sms = SmsController::with_default_obis();
        sms.prepare_send().await;
        sms.mark_send_success().await;

        assert_eq!(sms.send_status().await, SmsSendStatus::Success);
        assert_eq!(sms.send_count().await, 1);
    }

    #[tokio::test]
    async fn test_sms_controller_mark_send_failed() {
        let sms = SmsController::with_default_obis();
        sms.prepare_send().await;
        sms.mark_send_failed().await;

        assert_eq!(sms.send_status().await, SmsSendStatus::Failed);
        assert_eq!(sms.send_count().await, 0); // Failed doesn't increment count
    }

    #[tokio::test]
    async fn test_sms_controller_mark_send_timeout() {
        let sms = SmsController::with_default_obis();
        sms.prepare_send().await;
        sms.mark_send_timeout().await;

        assert_eq!(sms.send_status().await, SmsSendStatus::Timeout);
    }

    #[tokio::test]
    async fn test_sms_controller_validate_phone_number() {
        let sms = SmsController::with_default_obis();

        assert!(!sms.validate_phone_number().await); // Empty

        sms.set_phone_number("+1234567890".to_string()).await;
        assert!(sms.validate_phone_number().await);

        sms.set_phone_number("(555) 123-4567".to_string()).await;
        assert!(sms.validate_phone_number().await);

        sms.set_phone_number("invalid".to_string()).await;
        assert!(!sms.validate_phone_number().await);
    }

    #[tokio::test]
    async fn test_sms_controller_clear_phone_number() {
        let sms = SmsController::with_default_obis();
        sms.set_phone_number("+1234567890".to_string()).await;

        sms.clear_phone_number().await;
        assert_eq!(sms.phone_number().await, "");
    }

    #[tokio::test]
    async fn test_sms_controller_get_attributes() {
        let sms = SmsController::with_default_obis();

        // Test sms_enabled
        let result = sms.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Boolean(enabled) => assert!(!enabled),
            _ => panic!("Expected Boolean"),
        }

        // Test send_count
        let result = sms.get_attribute(6, None).await.unwrap();
        match result {
            DataObject::Unsigned32(count) => assert_eq!(count, 0),
            _ => panic!("Expected Unsigned32"),
        }

        // Test send_status
        let result = sms.get_attribute(5, None).await.unwrap();
        match result {
            DataObject::Enumerate(status) => assert_eq!(status, 0), // Idle
            _ => panic!("Expected Enumerate"),
        }
    }

    #[tokio::test]
    async fn test_sms_controller_set_attributes() {
        let sms = SmsController::with_default_obis();

        sms.set_attribute(2, DataObject::Boolean(true), None)
            .await
            .unwrap();
        assert!(sms.sms_enabled().await);

        sms.set_attribute(3, DataObject::OctetString(b"+9876543210".to_vec()), None)
            .await
            .unwrap();
        assert_eq!(sms.phone_number().await, "+9876543210");

        sms.set_attribute(8, DataObject::Unsigned16(140), None)
            .await
            .unwrap();
        assert_eq!(sms.max_message_size().await, 140);
    }

    #[tokio::test]
    async fn test_sms_controller_read_only_logical_name() {
        let sms = SmsController::with_default_obis();
        let result = sms
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 27, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sms_controller_invalid_attribute() {
        let sms = SmsController::with_default_obis();
        let result = sms.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sms_controller_invalid_method() {
        let sms = SmsController::with_default_obis();
        let result = sms.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }
}
