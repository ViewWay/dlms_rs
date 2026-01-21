//! Login interface class (Class ID: 93)
//!
//! The Login interface class manages user authentication and login sessions.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: current_user - Currently logged in user
//! - Attribute 3: authentication_status - Status of authentication
//! - Attribute 4: max_login_attempts - Maximum number of login attempts
//! - Attribute 5: remaining_attempts - Remaining login attempts
//! - Attribute 6: session_timeout - Session timeout in seconds

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Authentication Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AuthenticationStatus {
    /// Not authenticated
    NotAuthenticated = 0,
    /// Authentication in progress
    InProgress = 1,
    /// Authenticated
    Authenticated = 2,
    /// Authentication failed
    Failed = 3,
    /// Account locked
    Locked = 4,
    /// Session expired
    Expired = 5,
}

impl AuthenticationStatus {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::NotAuthenticated,
            1 => Self::InProgress,
            2 => Self::Authenticated,
            3 => Self::Failed,
            4 => Self::Locked,
            _ => Self::NotAuthenticated,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if authenticated
    pub fn is_authenticated(self) -> bool {
        matches!(self, Self::Authenticated)
    }

    /// Check if authentication failed
    pub fn is_failed(self) -> bool {
        matches!(self, Self::Failed | Self::Locked | Self::Expired)
    }

    /// Check if authentication is in progress
    pub fn is_in_progress(self) -> bool {
        matches!(self, Self::InProgress)
    }
}

/// Login interface class (Class ID: 93)
///
/// Default OBIS: 0-0:93.0.0.255
///
/// This class manages user authentication and login sessions.
#[derive(Debug, Clone)]
pub struct Login {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Currently logged in user
    current_user: Arc<RwLock<Option<String>>>,

    /// Status of authentication
    authentication_status: Arc<RwLock<AuthenticationStatus>>,

    /// Maximum number of login attempts
    max_login_attempts: Arc<RwLock<u8>>,

    /// Remaining login attempts
    remaining_attempts: Arc<RwLock<u8>>,

    /// Session timeout in seconds
    session_timeout: Arc<RwLock<u32>>,

    /// Login attempt counter
    attempt_counter: Arc<RwLock<u8>>,
}

impl Login {
    /// Class ID for Login
    pub const CLASS_ID: u16 = 93;

    /// Default OBIS code for Login (0-0:93.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 93, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_CURRENT_USER: u8 = 2;
    pub const ATTR_AUTHENTICATION_STATUS: u8 = 3;
    pub const ATTR_MAX_LOGIN_ATTEMPTS: u8 = 4;
    pub const ATTR_REMAINING_ATTEMPTS: u8 = 5;
    pub const ATTR_SESSION_TIMEOUT: u8 = 6;

    /// Create a new Login object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            current_user: Arc::new(RwLock::new(None)),
            authentication_status: Arc::new(RwLock::new(AuthenticationStatus::NotAuthenticated)),
            max_login_attempts: Arc::new(RwLock::new(3)),
            remaining_attempts: Arc::new(RwLock::new(3)),
            session_timeout: Arc::new(RwLock::new(300)), // 5 minutes default
            attempt_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Create with specific configuration
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `max_attempts` - Maximum login attempts
    /// * `timeout` - Session timeout in seconds
    pub fn with_config(logical_name: ObisCode, max_attempts: u8, timeout: u32) -> Self {
        Self {
            logical_name,
            current_user: Arc::new(RwLock::new(None)),
            authentication_status: Arc::new(RwLock::new(AuthenticationStatus::NotAuthenticated)),
            max_login_attempts: Arc::new(RwLock::new(max_attempts)),
            remaining_attempts: Arc::new(RwLock::new(max_attempts)),
            session_timeout: Arc::new(RwLock::new(timeout)),
            attempt_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Get the current user
    pub async fn current_user(&self) -> Option<String> {
        self.current_user.read().await.clone()
    }

    /// Get the authentication status
    pub async fn authentication_status(&self) -> AuthenticationStatus {
        *self.authentication_status.read().await
    }

    /// Get the max login attempts
    pub async fn max_login_attempts(&self) -> u8 {
        *self.max_login_attempts.read().await
    }

    /// Set the max login attempts
    pub async fn set_max_login_attempts(&self, attempts: u8) {
        *self.max_login_attempts.write().await = attempts;
        // Reset remaining attempts to new max
        let status = self.authentication_status().await;
        if !status.is_authenticated() && !status.is_failed() {
            *self.remaining_attempts.write().await = attempts;
        }
    }

    /// Get the remaining attempts
    pub async fn remaining_attempts(&self) -> u8 {
        *self.remaining_attempts.read().await
    }

    /// Get the session timeout
    pub async fn session_timeout(&self) -> u32 {
        *self.session_timeout.read().await
    }

    /// Set the session timeout
    pub async fn set_session_timeout(&self, timeout: u32) {
        *self.session_timeout.write().await = timeout;
    }

    /// Check if a user is logged in
    pub async fn is_logged_in(&self) -> bool {
        self.authentication_status().await.is_authenticated()
    }

    /// Check if account is locked
    pub async fn is_locked(&self) -> bool {
        matches!(
            self.authentication_status().await,
            AuthenticationStatus::Locked
        )
    }

    /// Attempt login (simulated)
    pub async fn login(&self, username: &str, _password: &str) -> DlmsResult<()> {
        let status = self.authentication_status().await;
        if status.is_authenticated() {
            return Err(DlmsError::InvalidData(
                "User already logged in".to_string(),
            ));
        }

        if self.is_locked().await {
            return Err(DlmsError::AccessDenied(
                "Account is locked".to_string(),
            ));
        }

        let remaining = self.remaining_attempts().await;
        if remaining == 0 {
            *self.authentication_status.write().await = AuthenticationStatus::Locked;
            return Err(DlmsError::AccessDenied("Account locked".to_string()));
        }

        // Simulate login attempt (in real implementation, verify password)
        // For testing, we accept any non-empty username/password
        let success = !username.is_empty();

        if success {
            *self.current_user.write().await = Some(username.to_string());
            *self.authentication_status.write().await = AuthenticationStatus::Authenticated;
            *self.remaining_attempts.write().await = self.max_login_attempts().await;
            *self.attempt_counter.write().await = 0;
            Ok(())
        } else {
            *self.attempt_counter.write().await += 1;
            let attempts_left = remaining.saturating_sub(1);
            *self.remaining_attempts.write().await = attempts_left;

            if attempts_left == 0 {
                *self.authentication_status.write().await = AuthenticationStatus::Locked;
                Err(DlmsError::AccessDenied("Account locked".to_string()))
            } else {
                *self.authentication_status.write().await = AuthenticationStatus::Failed;
                Err(DlmsError::AccessDenied(format!(
                    "Authentication failed. {} attempts remaining",
                    attempts_left
                )))
            }
        }
    }

    /// Logout
    pub async fn logout(&self) {
        *self.current_user.write().await = None;
        *self.authentication_status.write().await = AuthenticationStatus::NotAuthenticated;
        *self.remaining_attempts.write().await = self.max_login_attempts().await;
        *self.attempt_counter.write().await = 0;
    }

    /// Reset login attempts (admin function)
    pub async fn reset_attempts(&self) {
        *self.remaining_attempts.write().await = self.max_login_attempts().await;
        *self.attempt_counter.write().await = 0;
        if self.is_locked().await {
            *self.authentication_status.write().await = AuthenticationStatus::NotAuthenticated;
        }
    }

    /// Get attempt counter
    pub async fn attempt_counter(&self) -> u8 {
        *self.attempt_counter.read().await
    }

    /// Check if session has expired (simulated - in real impl, track last activity)
    pub async fn is_session_expired(&self) -> bool {
        matches!(
            self.authentication_status().await,
            AuthenticationStatus::Expired
        )
    }

    /// Set authentication status directly
    pub async fn set_authentication_status(&self, status: AuthenticationStatus) {
        *self.authentication_status.write().await = status;
    }
}

#[async_trait]
impl CosemObject for Login {
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
            Self::ATTR_CURRENT_USER => {
                match self.current_user().await {
                    Some(user) => Ok(DataObject::OctetString(user.into_bytes())),
                    None => Ok(DataObject::OctetString(Vec::new())),
                }
            }
            Self::ATTR_AUTHENTICATION_STATUS => {
                Ok(DataObject::Enumerate(self.authentication_status().await.to_u8()))
            }
            Self::ATTR_MAX_LOGIN_ATTEMPTS => {
                Ok(DataObject::Unsigned8(self.max_login_attempts().await))
            }
            Self::ATTR_REMAINING_ATTEMPTS => {
                Ok(DataObject::Unsigned8(self.remaining_attempts().await))
            }
            Self::ATTR_SESSION_TIMEOUT => {
                Ok(DataObject::Unsigned32(self.session_timeout().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Login has no attribute {}",
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
            Self::ATTR_CURRENT_USER => {
                match value {
                    DataObject::OctetString(bytes) => {
                        if bytes.is_empty() {
                            // Logout
                            self.logout().await;
                        } else {
                            let user = String::from_utf8_lossy(&bytes).to_string();
                            // This is a simplified login - in real impl, use separate method
                            *self.current_user.write().await = Some(user);
                        }
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for current_user".to_string(),
                    )),
                }
            }
            Self::ATTR_AUTHENTICATION_STATUS => {
                match value {
                    DataObject::Enumerate(status) => {
                        self.set_authentication_status(AuthenticationStatus::from_u8(status)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for authentication_status".to_string(),
                    )),
                }
            }
            Self::ATTR_MAX_LOGIN_ATTEMPTS => {
                match value {
                    DataObject::Unsigned8(attempts) => {
                        self.set_max_login_attempts(attempts).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for max_login_attempts".to_string(),
                    )),
                }
            }
            Self::ATTR_REMAINING_ATTEMPTS => {
                match value {
                    DataObject::Unsigned8(attempts) => {
                        *self.remaining_attempts.write().await = attempts;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for remaining_attempts".to_string(),
                    )),
                }
            }
            Self::ATTR_SESSION_TIMEOUT => {
                match value {
                    DataObject::Unsigned32(timeout) => {
                        self.set_session_timeout(timeout).await;
                        Ok(())
                    }
                    DataObject::Unsigned16(timeout) => {
                        self.set_session_timeout(timeout as u32).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned32/Unsigned16 for session_timeout".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Login has no attribute {}",
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
            "Login has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_login_class_id() {
        let login = Login::with_default_obis();
        assert_eq!(login.class_id(), 93);
    }

    #[tokio::test]
    async fn test_login_obis_code() {
        let login = Login::with_default_obis();
        assert_eq!(login.obis_code(), Login::default_obis());
    }

    #[tokio::test]
    async fn test_login_initial_state() {
        let login = Login::with_default_obis();
        assert!(!login.is_logged_in().await);
        assert!(!login.is_locked().await);
        assert_eq!(login.current_user().await, None);
        assert_eq!(login.authentication_status().await, AuthenticationStatus::NotAuthenticated);
        assert_eq!(login.max_login_attempts().await, 3);
        assert_eq!(login.remaining_attempts().await, 3);
        assert_eq!(login.session_timeout().await, 300);
    }

    #[tokio::test]
    async fn test_login_with_config() {
        let login = Login::with_config(ObisCode::new(0, 0, 93, 0, 0, 255), 5, 600);
        assert_eq!(login.max_login_attempts().await, 5);
        assert_eq!(login.remaining_attempts().await, 5);
        assert_eq!(login.session_timeout().await, 600);
    }

    #[tokio::test]
    async fn test_login_success() {
        let login = Login::with_default_obis();
        login.login("user1", "password").await.unwrap();
        assert!(login.is_logged_in().await);
        assert_eq!(login.current_user().await, Some("user1".to_string()));
        assert_eq!(login.authentication_status().await, AuthenticationStatus::Authenticated);
        assert_eq!(login.remaining_attempts().await, 3); // Reset after success
    }

    #[tokio::test]
    async fn test_login_already_logged_in() {
        let login = Login::with_default_obis();
        login.login("user1", "password").await.unwrap();
        let result = login.login("user2", "password").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_login_empty_username() {
        let login = Login::with_default_obis();
        let result = login.login("", "password").await;
        assert!(result.is_err());
        assert_eq!(login.remaining_attempts().await, 2);
    }

    #[tokio::test]
    async fn test_login_attempts_decrement() {
        let login = Login::with_default_obis();
        let _ = login.login("", "password1").await;
        assert_eq!(login.remaining_attempts().await, 2);
        let _ = login.login("", "password2").await;
        assert_eq!(login.remaining_attempts().await, 1);
    }

    #[tokio::test]
    async fn test_login_locked_after_max_attempts() {
        let login = Login::with_config(ObisCode::new(0, 0, 93, 0, 0, 255), 3, 300);
        let _ = login.login("", "password1").await;
        let _ = login.login("", "password2").await;
        let result = login.login("", "password3").await;
        assert!(result.is_err());
        assert!(login.is_locked().await);
        assert_eq!(login.authentication_status().await, AuthenticationStatus::Locked);
    }

    #[tokio::test]
    async fn test_login_while_locked() {
        let login = Login::with_config(ObisCode::new(0, 0, 93, 0, 0, 255), 2, 300);
        let _ = login.login("", "password1").await;
        let _ = login.login("", "password2").await;
        let result = login.login("valid", "password").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_logout() {
        let login = Login::with_default_obis();
        login.login("user1", "password").await.unwrap();
        login.logout().await;
        assert!(!login.is_logged_in().await);
        assert_eq!(login.current_user().await, None);
        assert_eq!(login.authentication_status().await, AuthenticationStatus::NotAuthenticated);
        assert_eq!(login.remaining_attempts().await, 3); // Reset
    }

    #[tokio::test]
    async fn test_reset_attempts() {
        let login = Login::with_config(ObisCode::new(0, 0, 93, 0, 0, 255), 3, 300);
        let _ = login.login("", "password1").await;
        assert_eq!(login.remaining_attempts().await, 2);
        login.reset_attempts().await;
        assert_eq!(login.remaining_attempts().await, 3);
    }

    #[tokio::test]
    async fn test_reset_attempts_unlocks() {
        let login = Login::with_config(ObisCode::new(0, 0, 93, 0, 0, 255), 2, 300);
        let _ = login.login("", "password1").await;
        let _ = login.login("", "password2").await;
        assert!(login.is_locked().await);
        login.reset_attempts().await;
        assert!(!login.is_locked().await);
    }

    #[tokio::test]
    async fn test_set_max_login_attempts() {
        let login = Login::with_default_obis();
        login.set_max_login_attempts(5).await;
        assert_eq!(login.max_login_attempts().await, 5);
        assert_eq!(login.remaining_attempts().await, 5);
    }

    #[tokio::test]
    async fn test_set_session_timeout() {
        let login = Login::with_default_obis();
        login.set_session_timeout(900).await;
        assert_eq!(login.session_timeout().await, 900);
    }

    #[tokio::test]
    async fn test_authentication_status_from_u8() {
        assert_eq!(AuthenticationStatus::from_u8(0), AuthenticationStatus::NotAuthenticated);
        assert_eq!(AuthenticationStatus::from_u8(1), AuthenticationStatus::InProgress);
        assert_eq!(AuthenticationStatus::from_u8(2), AuthenticationStatus::Authenticated);
        assert_eq!(AuthenticationStatus::from_u8(3), AuthenticationStatus::Failed);
        assert_eq!(AuthenticationStatus::from_u8(4), AuthenticationStatus::Locked);
    }

    #[tokio::test]
    async fn test_authentication_status_is_authenticated() {
        assert!(AuthenticationStatus::Authenticated.is_authenticated());
        assert!(!AuthenticationStatus::NotAuthenticated.is_authenticated());
    }

    #[tokio::test]
    async fn test_authentication_status_is_failed() {
        assert!(AuthenticationStatus::Failed.is_failed());
        assert!(AuthenticationStatus::Locked.is_failed());
        assert!(AuthenticationStatus::Expired.is_failed());
        assert!(!AuthenticationStatus::Authenticated.is_failed());
    }

    #[tokio::test]
    async fn test_authentication_status_is_in_progress() {
        assert!(AuthenticationStatus::InProgress.is_in_progress());
        assert!(!AuthenticationStatus::Authenticated.is_in_progress());
    }

    #[tokio::test]
    async fn test_get_attributes() {
        let login = Login::with_default_obis();

        // Test authentication_status
        let result = login.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Enumerate(status) => assert_eq!(status, 0), // NotAuthenticated
            _ => panic!("Expected Enumerate"),
        }

        // Test max_login_attempts
        let result = login.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Unsigned8(attempts) => assert_eq!(attempts, 3),
            _ => panic!("Expected Unsigned8"),
        }

        // Test session_timeout
        let result = login.get_attribute(6, None).await.unwrap();
        match result {
            DataObject::Unsigned32(timeout) => assert_eq!(timeout, 300),
            _ => panic!("Expected Unsigned32"),
        }
    }

    #[tokio::test]
    async fn test_set_attributes() {
        let login = Login::with_default_obis();

        login.set_attribute(4, DataObject::Unsigned8(5), None)
            .await
            .unwrap();
        assert_eq!(login.max_login_attempts().await, 5);

        login.set_attribute(6, DataObject::Unsigned32(600), None)
            .await
            .unwrap();
        assert_eq!(login.session_timeout().await, 600);
    }

    #[tokio::test]
    async fn test_set_session_timeout_u16() {
        let login = Login::with_default_obis();
        login.set_attribute(6, DataObject::Unsigned16(900), None)
            .await
            .unwrap();
        assert_eq!(login.session_timeout().await, 900);
    }

    #[tokio::test]
    async fn test_set_current_user_logout() {
        let login = Login::with_default_obis();
        login.login("user1", "password").await.unwrap();
        login.set_attribute(2, DataObject::OctetString(Vec::new()), None)
            .await
            .unwrap();
        assert!(!login.is_logged_in().await);
    }

    #[tokio::test]
    async fn test_read_only_logical_name() {
        let login = Login::with_default_obis();
        let result = login
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 93, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_attribute() {
        let login = Login::with_default_obis();
        let result = login.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_method() {
        let login = Login::with_default_obis();
        let result = login.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 93, 0, 0, 1);
        let login = Login::new(obis);
        assert_eq!(login.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_attempt_counter() {
        let login = Login::with_default_obis();
        assert_eq!(login.attempt_counter().await, 0);
        let _ = login.login("", "password1").await;
        assert_eq!(login.attempt_counter().await, 1);
    }
}
