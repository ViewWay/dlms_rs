//! Token Gateway interface class (Class ID: 81)
//!
//! The Token Gateway interface class manages token-based transactions for prepaid meters.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: token_id - Unique token identifier
//! - Attribute 3: token_status - Status of the token
//! - Attribute 4: token_amount - Amount associated with the token
//! - Attribute 5: token_currency - Currency code for the token
//! - Attribute 6: token_expiry_date - Token expiration date
//! - Attribute 7: token_issue_date - Token issue date
//! - Attribute 8: token_type - Type of token

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use dlms_core::datatypes::{CosemDate, CosemDateFormat};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Token Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TokenStatus {
    /// Token is valid and ready to use
    Valid = 0,
    /// Token has been used
    Used = 1,
    /// Token has expired
    Expired = 2,
    /// Token is invalid
    Invalid = 3,
    /// Token is pending processing
    Pending = 4,
}

impl TokenStatus {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Valid,
            1 => Self::Used,
            2 => Self::Expired,
            3 => Self::Invalid,
            4 => Self::Pending,
            _ => Self::Invalid,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if token can be used
    pub fn is_usable(self) -> bool {
        matches!(self, Self::Valid | Self::Pending)
    }
}

/// Token Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TokenType {
    /// Credit token - adds credit to account
    Credit = 0,
    /// Emergency credit token
    EmergencyCredit = 1,
    /// Test token
    Test = 2,
    /// Settlement token
    Settlement = 3,
    /// Refund token
    Refund = 4,
    /// Custom token type
    Custom = 5,
}

impl TokenType {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Credit,
            1 => Self::EmergencyCredit,
            2 => Self::Test,
            3 => Self::Settlement,
            4 => Self::Refund,
            _ => Self::Custom,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if this is a credit token
    pub fn is_credit(self) -> bool {
        matches!(self, Self::Credit | Self::EmergencyCredit)
    }
}

/// Token Gateway interface class (Class ID: 81)
///
/// Default OBIS: 0-0:81.0.0.255
///
/// This class manages token-based transactions for prepaid meters.
#[derive(Debug, Clone)]
pub struct TokenGateway {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Token identifier
    token_id: Arc<RwLock<String>>,

    /// Token status
    token_status: Arc<RwLock<TokenStatus>>,

    /// Token amount (in smallest currency unit)
    token_amount: Arc<RwLock<i64>>,

    /// Token currency code
    token_currency: Arc<RwLock<String>>,

    /// Token expiry date
    token_expiry_date: Arc<RwLock<Option<CosemDate>>>,

    /// Token issue date
    token_issue_date: Arc<RwLock<Option<CosemDate>>>,

    /// Token type
    token_type: Arc<RwLock<TokenType>>,
}

impl TokenGateway {
    /// Class ID for TokenGateway
    pub const CLASS_ID: u16 = 81;

    /// Default OBIS code for TokenGateway (0-0:81.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 81, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_TOKEN_ID: u8 = 2;
    pub const ATTR_TOKEN_STATUS: u8 = 3;
    pub const ATTR_TOKEN_AMOUNT: u8 = 4;
    pub const ATTR_TOKEN_CURRENCY: u8 = 5;
    pub const ATTR_TOKEN_EXPIRY_DATE: u8 = 6;
    pub const ATTR_TOKEN_ISSUE_DATE: u8 = 7;
    pub const ATTR_TOKEN_TYPE: u8 = 8;

    /// Create a new TokenGateway object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            token_id: Arc::new(RwLock::new(String::new())),
            token_status: Arc::new(RwLock::new(TokenStatus::Invalid)),
            token_amount: Arc::new(RwLock::new(0)),
            token_currency: Arc::new(RwLock::new(String::new())),
            token_expiry_date: Arc::new(RwLock::new(None)),
            token_issue_date: Arc::new(RwLock::new(None)),
            token_type: Arc::new(RwLock::new(TokenType::Credit)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get the token ID
    pub async fn token_id(&self) -> String {
        self.token_id.read().await.clone()
    }

    /// Set the token ID
    pub async fn set_token_id(&self, id: String) {
        *self.token_id.write().await = id;
    }

    /// Get the token status
    pub async fn token_status(&self) -> TokenStatus {
        *self.token_status.read().await
    }

    /// Set the token status
    pub async fn set_token_status(&self, status: TokenStatus) {
        *self.token_status.write().await = status;
    }

    /// Get the token amount
    pub async fn token_amount(&self) -> i64 {
        *self.token_amount.read().await
    }

    /// Set the token amount
    pub async fn set_token_amount(&self, amount: i64) {
        *self.token_amount.write().await = amount;
    }

    /// Get the token currency
    pub async fn token_currency(&self) -> String {
        self.token_currency.read().await.clone()
    }

    /// Set the token currency
    pub async fn set_token_currency(&self, currency: String) {
        *self.token_currency.write().await = currency;
    }

    /// Get the token expiry date
    pub async fn token_expiry_date(&self) -> Option<CosemDate> {
        self.token_expiry_date.read().await.clone()
    }

    /// Set the token expiry date
    pub async fn set_token_expiry_date(&self, date: Option<CosemDate>) {
        *self.token_expiry_date.write().await = date;
    }

    /// Get the token issue date
    pub async fn token_issue_date(&self) -> Option<CosemDate> {
        self.token_issue_date.read().await.clone()
    }

    /// Set the token issue date
    pub async fn set_token_issue_date(&self, date: Option<CosemDate>) {
        *self.token_issue_date.write().await = date;
    }

    /// Get the token type
    pub async fn token_type(&self) -> TokenType {
        *self.token_type.read().await
    }

    /// Set the token type
    pub async fn set_token_type(&self, token_type: TokenType) {
        *self.token_type.write().await = token_type;
    }

    /// Check if token can be used
    pub async fn is_usable(&self) -> bool {
        self.token_status().await.is_usable()
    }

    /// Mark token as used
    pub async fn mark_as_used(&self) {
        *self.token_status.write().await = TokenStatus::Used;
    }

    /// Validate token (check expiry and status)
    pub async fn validate(&self, current_date: CosemDate) -> bool {
        let status = self.token_status().await;
        if !status.is_usable() {
            return false;
        }

        if let Some(expiry) = self.token_expiry_date().await {
            // Check if token has expired
            // Note: This is a simplified check - proper date comparison would be needed
            if expiry.encode()[0] < current_date.encode()[0] {
                *self.token_status.write().await = TokenStatus::Expired;
                return false;
            }
        }

        true
    }

    /// Load a new token
    pub async fn load_token(&self, token_id: String, amount: i64, token_type: TokenType) {
        self.set_token_id(token_id).await;
        self.set_token_amount(amount).await;
        self.set_token_type(token_type).await;
        self.set_token_status(TokenStatus::Valid).await;
    }

    /// Clear token data
    pub async fn clear(&self) {
        self.set_token_id(String::new()).await;
        self.set_token_amount(0).await;
        self.set_token_status(TokenStatus::Invalid).await;
        self.set_token_expiry_date(None).await;
        self.set_token_issue_date(None).await;
    }
}

#[async_trait]
impl CosemObject for TokenGateway {
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
            Self::ATTR_TOKEN_ID => {
                Ok(DataObject::OctetString(self.token_id().await.into_bytes()))
            }
            Self::ATTR_TOKEN_STATUS => {
                Ok(DataObject::Enumerate(self.token_status().await.to_u8()))
            }
            Self::ATTR_TOKEN_AMOUNT => {
                Ok(DataObject::Integer64(self.token_amount().await))
            }
            Self::ATTR_TOKEN_CURRENCY => {
                Ok(DataObject::OctetString(self.token_currency().await.into_bytes()))
            }
            Self::ATTR_TOKEN_EXPIRY_DATE => {
                match self.token_expiry_date().await {
                    Some(date) => Ok(DataObject::OctetString(date.encode().to_vec())),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_TOKEN_ISSUE_DATE => {
                match self.token_issue_date().await {
                    Some(date) => Ok(DataObject::OctetString(date.encode().to_vec())),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_TOKEN_TYPE => {
                Ok(DataObject::Enumerate(self.token_type().await.to_u8()))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "TokenGateway has no attribute {}",
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
            Self::ATTR_TOKEN_ID => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let id = String::from_utf8_lossy(&bytes).to_string();
                        self.set_token_id(id).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for token_id".to_string(),
                    )),
                }
            }
            Self::ATTR_TOKEN_STATUS => {
                match value {
                    DataObject::Enumerate(status) => {
                        self.set_token_status(TokenStatus::from_u8(status)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for token_status".to_string(),
                    )),
                }
            }
            Self::ATTR_TOKEN_AMOUNT => {
                match value {
                    DataObject::Integer64(amount) => {
                        self.set_token_amount(amount).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 for token_amount".to_string(),
                    )),
                }
            }
            Self::ATTR_TOKEN_CURRENCY => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let currency = String::from_utf8_lossy(&bytes).to_string();
                        self.set_token_currency(currency).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for token_currency".to_string(),
                    )),
                }
            }
            Self::ATTR_TOKEN_EXPIRY_DATE => {
                match value {
                    DataObject::OctetString(bytes) => {
                        if bytes.is_empty() {
                            self.set_token_expiry_date(None).await;
                        } else if let Ok(date) = CosemDate::decode(&bytes) {
                            self.set_token_expiry_date(Some(date)).await;
                        }
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_token_expiry_date(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString or Null for token_expiry_date".to_string(),
                    )),
                }
            }
            Self::ATTR_TOKEN_ISSUE_DATE => {
                match value {
                    DataObject::OctetString(bytes) => {
                        if bytes.is_empty() {
                            self.set_token_issue_date(None).await;
                        } else if let Ok(date) = CosemDate::decode(&bytes) {
                            self.set_token_issue_date(Some(date)).await;
                        }
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_token_issue_date(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString or Null for token_issue_date".to_string(),
                    )),
                }
            }
            Self::ATTR_TOKEN_TYPE => {
                match value {
                    DataObject::Enumerate(token_type) => {
                        self.set_token_type(TokenType::from_u8(token_type)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for token_type".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "TokenGateway has no attribute {}",
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
            "TokenGateway has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_gateway_class_id() {
        let tg = TokenGateway::with_default_obis();
        assert_eq!(tg.class_id(), 81);
    }

    #[tokio::test]
    async fn test_token_gateway_obis_code() {
        let tg = TokenGateway::with_default_obis();
        assert_eq!(tg.obis_code(), TokenGateway::default_obis());
    }

    #[tokio::test]
    async fn test_token_status_from_u8() {
        assert_eq!(TokenStatus::from_u8(0), TokenStatus::Valid);
        assert_eq!(TokenStatus::from_u8(1), TokenStatus::Used);
        assert_eq!(TokenStatus::from_u8(2), TokenStatus::Expired);
        assert_eq!(TokenStatus::from_u8(3), TokenStatus::Invalid);
        assert_eq!(TokenStatus::from_u8(4), TokenStatus::Pending);
    }

    #[tokio::test]
    async fn test_token_status_is_usable() {
        assert!(TokenStatus::Valid.is_usable());
        assert!(TokenStatus::Pending.is_usable());
        assert!(!TokenStatus::Used.is_usable());
        assert!(!TokenStatus::Expired.is_usable());
        assert!(!TokenStatus::Invalid.is_usable());
    }

    #[tokio::test]
    async fn test_token_type_from_u8() {
        assert_eq!(TokenType::from_u8(0), TokenType::Credit);
        assert_eq!(TokenType::from_u8(1), TokenType::EmergencyCredit);
        assert_eq!(TokenType::from_u8(2), TokenType::Test);
        assert_eq!(TokenType::from_u8(3), TokenType::Settlement);
        assert_eq!(TokenType::from_u8(4), TokenType::Refund);
    }

    #[tokio::test]
    async fn test_token_type_is_credit() {
        assert!(TokenType::Credit.is_credit());
        assert!(TokenType::EmergencyCredit.is_credit());
        assert!(!TokenType::Test.is_credit());
        assert!(!TokenType::Settlement.is_credit());
        assert!(!TokenType::Refund.is_credit());
    }

    #[tokio::test]
    async fn test_token_gateway_initial_state() {
        let tg = TokenGateway::with_default_obis();
        assert_eq!(tg.token_id().await, "");
        assert_eq!(tg.token_status().await, TokenStatus::Invalid);
        assert_eq!(tg.token_amount().await, 0);
        assert_eq!(tg.token_type().await, TokenType::Credit);
    }

    #[tokio::test]
    async fn test_token_gateway_set_token_id() {
        let tg = TokenGateway::with_default_obis();
        tg.set_token_id("TOKEN-12345".to_string()).await;
        assert_eq!(tg.token_id().await, "TOKEN-12345");
    }

    #[tokio::test]
    async fn test_token_gateway_set_token_status() {
        let tg = TokenGateway::with_default_obis();
        tg.set_token_status(TokenStatus::Valid).await;
        assert_eq!(tg.token_status().await, TokenStatus::Valid);
    }

    #[tokio::test]
    async fn test_token_gateway_set_token_amount() {
        let tg = TokenGateway::with_default_obis();
        tg.set_token_amount(5000).await;
        assert_eq!(tg.token_amount().await, 5000);
    }

    #[tokio::test]
    async fn test_token_gateway_set_token_currency() {
        let tg = TokenGateway::with_default_obis();
        tg.set_token_currency("USD".to_string()).await;
        assert_eq!(tg.token_currency().await, "USD");
    }

    #[tokio::test]
    async fn test_token_gateway_set_token_type() {
        let tg = TokenGateway::with_default_obis();
        tg.set_token_type(TokenType::EmergencyCredit).await;
        assert_eq!(tg.token_type().await, TokenType::EmergencyCredit);
    }

    #[tokio::test]
    async fn test_token_gateway_is_usable() {
        let tg = TokenGateway::with_default_obis();

        assert!(!tg.is_usable().await);

        tg.set_token_status(TokenStatus::Valid).await;
        assert!(tg.is_usable().await);

        tg.set_token_status(TokenStatus::Pending).await;
        assert!(tg.is_usable().await);

        tg.set_token_status(TokenStatus::Used).await;
        assert!(!tg.is_usable().await);
    }

    #[tokio::test]
    async fn test_token_gateway_mark_as_used() {
        let tg = TokenGateway::with_default_obis();
        tg.set_token_status(TokenStatus::Valid).await;
        assert!(tg.is_usable().await);

        tg.mark_as_used().await;
        assert_eq!(tg.token_status().await, TokenStatus::Used);
        assert!(!tg.is_usable().await);
    }

    #[tokio::test]
    async fn test_token_gateway_load_token() {
        let tg = TokenGateway::with_default_obis();
        tg.load_token("TOKEN-999".to_string(), 10000, TokenType::Credit).await;

        assert_eq!(tg.token_id().await, "TOKEN-999");
        assert_eq!(tg.token_amount().await, 10000);
        assert_eq!(tg.token_type().await, TokenType::Credit);
        assert_eq!(tg.token_status().await, TokenStatus::Valid);
    }

    #[tokio::test]
    async fn test_token_gateway_clear() {
        let tg = TokenGateway::with_default_obis();
        tg.load_token("TOKEN-999".to_string(), 10000, TokenType::Credit).await;

        tg.clear().await;

        assert_eq!(tg.token_id().await, "");
        assert_eq!(tg.token_amount().await, 0);
        assert_eq!(tg.token_status().await, TokenStatus::Invalid);
    }

    #[tokio::test]
    async fn test_token_gateway_get_attributes() {
        let tg = TokenGateway::with_default_obis();
        tg.load_token("T1".to_string(), 100, TokenType::Credit).await;

        // Test token_id
        let result = tg.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::OctetString(bytes) => assert_eq!(String::from_utf8_lossy(&bytes), "T1"),
            _ => panic!("Expected OctetString"),
        }

        // Test token_amount
        let result = tg.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Integer64(amount) => assert_eq!(amount, 100),
            _ => panic!("Expected Integer64"),
        }

        // Test token_status
        let result = tg.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Enumerate(status) => assert_eq!(status, 0), // Valid
            _ => panic!("Expected Enumerate"),
        }
    }

    #[tokio::test]
    async fn test_token_gateway_set_attributes() {
        let tg = TokenGateway::with_default_obis();

        tg.set_attribute(2, DataObject::OctetString(b"NEW-TOKEN".to_vec()), None)
            .await
            .unwrap();
        assert_eq!(tg.token_id().await, "NEW-TOKEN");

        tg.set_attribute(4, DataObject::Integer64(250), None)
            .await
            .unwrap();
        assert_eq!(tg.token_amount().await, 250);

        tg.set_attribute(3, DataObject::Enumerate(1), None) // Used
            .await
            .unwrap();
        assert_eq!(tg.token_status().await, TokenStatus::Used);
    }

    #[tokio::test]
    async fn test_token_gateway_read_only_logical_name() {
        let tg = TokenGateway::with_default_obis();
        let result = tg
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 81, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_token_gateway_invalid_attribute() {
        let tg = TokenGateway::with_default_obis();
        let result = tg.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_token_gateway_invalid_method() {
        let tg = TokenGateway::with_default_obis();
        let result = tg.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_token_gateway_null_dates() {
        let tg = TokenGateway::with_default_obis();

        // Default dates are None, should return Null DataObject
        let result = tg.get_attribute(6, None).await.unwrap();
        assert!(matches!(result, DataObject::Null));

        let result = tg.get_attribute(7, None).await.unwrap();
        assert!(matches!(result, DataObject::Null));
    }
}
