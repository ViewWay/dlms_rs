//! Limiter interface class (Class ID: 71)
//!
//! The Limiter interface class provides load limiting functionality
//! based on power or consumption thresholds.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: threshold_active - Active power threshold value
//! - Attribute 3: threshold_active_normal - Normal power threshold value
//! - Attribute 4: threshold_reactive - Reactive power threshold value
//! - Attribute 5: threshold_reactive_normal - Normal reactive threshold
//! - Attribute 6: action_threshold_over - Action when threshold exceeded
//! - Attribute 7: action_threshold_under - Action when threshold normal
//!
//! # Methods
//!
//! - Method 1: remote_disconnect() - Disconnect due to limit exceeded
//! - Method 2: remote_reconnect() - Reconnect when within limits
//!
//! # Limiter (Class ID: 71)
//!
//! This class monitors register values and can trigger disconnect/reconnect
//! actions when thresholds are exceeded or returned to normal levels.

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Limiter action configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LimiterAction {
    /// No action
    NoAction = 0,
    /// Disconnect load
    Disconnect = 1,
    /// Reconnect load
    Reconnect = 2,
    /// Send alarm
    SendAlarm = 3,
    /// Send alarm and disconnect
    SendAlarmAndDisconnect = 4,
}

impl LimiterAction {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::NoAction,
            1 => Self::Disconnect,
            2 => Self::Reconnect,
            3 => Self::SendAlarm,
            4 => Self::SendAlarmAndDisconnect,
            _ => Self::NoAction,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Limiter interface class (Class ID: 71)
///
/// Default OBIS: 0-0:97.0.0.255
///
/// This class provides load limiting based on power thresholds.
/// It monitors power consumption and can trigger actions.
#[derive(Debug, Clone)]
pub struct Limiter {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Active power threshold (limit value)
    threshold_active: Arc<RwLock<i64>>,

    /// Normal active power threshold (hysteresis)
    threshold_active_normal: Arc<RwLock<i64>>,

    /// Reactive power threshold
    threshold_reactive: Arc<RwLock<Option<i64>>>,

    /// Normal reactive power threshold
    threshold_reactive_normal: Arc<RwLock<Option<i64>>>,

    /// Action when threshold exceeded
    action_threshold_over: Arc<RwLock<LimiterAction>>,

    /// Action when returned to normal
    action_threshold_under: Arc<RwLock<LimiterAction>>,

    /// Current limiter status
    limit_active: Arc<RwLock<bool>>,
}

impl Limiter {
    /// Class ID for Limiter
    pub const CLASS_ID: u16 = 71;

    /// Default OBIS code for Limiter (0-0:97.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 97, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_THRESHOLD_ACTIVE: u8 = 2;
    pub const ATTR_THRESHOLD_ACTIVE_NORMAL: u8 = 3;
    pub const ATTR_THRESHOLD_REACTIVE: u8 = 4;
    pub const ATTR_THRESHOLD_REACTIVE_NORMAL: u8 = 5;
    pub const ATTR_ACTION_THRESHOLD_OVER: u8 = 6;
    pub const ATTR_ACTION_THRESHOLD_UNDER: u8 = 7;

    /// Method IDs
    pub const METHOD_REMOTE_DISCONNECT: u8 = 1;
    pub const METHOD_REMOTE_RECONNECT: u8 = 2;

    /// Create a new Limiter object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `threshold_active` - Active power threshold
    /// * `threshold_active_normal` - Normal active power threshold
    pub fn new(logical_name: ObisCode, threshold_active: i64, threshold_active_normal: i64) -> Self {
        Self {
            logical_name,
            threshold_active: Arc::new(RwLock::new(threshold_active)),
            threshold_active_normal: Arc::new(RwLock::new(threshold_active_normal)),
            threshold_reactive: Arc::new(RwLock::new(None)),
            threshold_reactive_normal: Arc::new(RwLock::new(None)),
            action_threshold_over: Arc::new(RwLock::new(LimiterAction::Disconnect)),
            action_threshold_under: Arc::new(RwLock::new(LimiterAction::Reconnect)),
            limit_active: Arc::new(RwLock::new(false)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis(threshold_active: i64, threshold_active_normal: i64) -> Self {
        Self::new(Self::default_obis(), threshold_active, threshold_active_normal)
    }

    /// Get the active power threshold
    pub async fn threshold_active(&self) -> i64 {
        *self.threshold_active.read().await
    }

    /// Set the active power threshold
    pub async fn set_threshold_active(&self, threshold: i64) {
        *self.threshold_active.write().await = threshold;
    }

    /// Get the normal active power threshold
    pub async fn threshold_active_normal(&self) -> i64 {
        *self.threshold_active_normal.read().await
    }

    /// Set the normal active power threshold
    pub async fn set_threshold_active_normal(&self, threshold: i64) {
        *self.threshold_active_normal.write().await = threshold;
    }

    /// Get the reactive power threshold
    pub async fn threshold_reactive(&self) -> Option<i64> {
        *self.threshold_reactive.read().await
    }

    /// Set the reactive power threshold
    pub async fn set_threshold_reactive(&self, threshold: Option<i64>) {
        *self.threshold_reactive.write().await = threshold;
    }

    /// Get the normal reactive power threshold
    pub async fn threshold_reactive_normal(&self) -> Option<i64> {
        *self.threshold_reactive_normal.read().await
    }

    /// Set the normal reactive power threshold
    pub async fn set_threshold_reactive_normal(&self, threshold: Option<i64>) {
        *self.threshold_reactive_normal.write().await = threshold;
    }

    /// Get action when threshold exceeded
    pub async fn action_threshold_over(&self) -> LimiterAction {
        *self.action_threshold_over.read().await
    }

    /// Set action when threshold exceeded
    pub async fn set_action_threshold_over(&self, action: LimiterAction) {
        *self.action_threshold_over.write().await = action;
    }

    /// Get action when returned to normal
    pub async fn action_threshold_under(&self) -> LimiterAction {
        *self.action_threshold_under.read().await
    }

    /// Set action when returned to normal
    pub async fn set_action_threshold_under(&self, action: LimiterAction) {
        *self.action_threshold_under.write().await = action;
    }

    /// Check if limit is currently active
    pub async fn is_limit_active(&self) -> bool {
        *self.limit_active.read().await
    }

    /// Set limit active status
    pub async fn set_limit_active(&self, active: bool) {
        *self.limit_active.write().await = active;
    }

    /// Check if active power value exceeds threshold
    pub async fn check_active_power(&self, current_value: i64) -> bool {
        if self.is_limit_active().await {
            // Check if value is now within normal threshold
            current_value <= self.threshold_active_normal().await
        } else {
            // Check if value exceeds limit threshold
            current_value > self.threshold_active().await
        }
    }

    /// Update limiter with current active power value
    /// Returns true if state changed
    pub async fn update_active_power(&self, current_value: i64) -> bool {
        let was_limited = self.is_limit_active().await;
        let is_limited = current_value > self.threshold_active().await;

        if was_limited != is_limited {
            self.set_limit_active(is_limited).await;
            true
        } else {
            false
        }
    }

    /// Remote disconnect - disconnect due to limit exceeded
    ///
    /// This corresponds to Method 1
    pub async fn remote_disconnect(&self) -> DlmsResult<()> {
        self.set_limit_active(true).await;
        Ok(())
    }

    /// Remote reconnect - reconnect when within limits
    ///
    /// This corresponds to Method 2
    pub async fn remote_reconnect(&self) -> DlmsResult<()> {
        self.set_limit_active(false).await;
        Ok(())
    }

    /// Get all thresholds as a tuple
    pub async fn thresholds(&self) -> (i64, i64, Option<i64>, Option<i64>) {
        (
            self.threshold_active().await,
            self.threshold_active_normal().await,
            self.threshold_reactive().await,
            self.threshold_reactive_normal().await,
        )
    }
}

#[async_trait]
impl CosemObject for Limiter {
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
            Self::ATTR_THRESHOLD_ACTIVE => {
                Ok(DataObject::Integer64(self.threshold_active().await))
            }
            Self::ATTR_THRESHOLD_ACTIVE_NORMAL => {
                Ok(DataObject::Integer64(self.threshold_active_normal().await))
            }
            Self::ATTR_THRESHOLD_REACTIVE => {
                match self.threshold_reactive().await {
                    Some(v) => Ok(DataObject::Integer64(v)),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_THRESHOLD_REACTIVE_NORMAL => {
                match self.threshold_reactive_normal().await {
                    Some(v) => Ok(DataObject::Integer64(v)),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_ACTION_THRESHOLD_OVER => {
                Ok(DataObject::Enumerate(self.action_threshold_over().await.to_u8()))
            }
            Self::ATTR_ACTION_THRESHOLD_UNDER => {
                Ok(DataObject::Enumerate(self.action_threshold_under().await.to_u8()))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Limiter has no attribute {}",
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
            Self::ATTR_THRESHOLD_ACTIVE => {
                if let DataObject::Integer64(v) = value {
                    self.set_threshold_active(v).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Integer64 for threshold_active".to_string(),
                    ))
                }
            }
            Self::ATTR_THRESHOLD_ACTIVE_NORMAL => {
                if let DataObject::Integer64(v) = value {
                    self.set_threshold_active_normal(v).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Integer64 for threshold_active_normal".to_string(),
                    ))
                }
            }
            Self::ATTR_THRESHOLD_REACTIVE => {
                match value {
                    DataObject::Integer64(v) => {
                        self.set_threshold_reactive(Some(v)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_threshold_reactive(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 or Null for threshold_reactive".to_string(),
                    )),
                }
            }
            Self::ATTR_THRESHOLD_REACTIVE_NORMAL => {
                match value {
                    DataObject::Integer64(v) => {
                        self.set_threshold_reactive_normal(Some(v)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_threshold_reactive_normal(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer64 or Null for threshold_reactive_normal".to_string(),
                    )),
                }
            }
            Self::ATTR_ACTION_THRESHOLD_OVER => {
                let action = match value {
                    DataObject::Enumerate(a) => LimiterAction::from_u8(a),
                    DataObject::Unsigned8(a) => LimiterAction::from_u8(a),
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Enumerate for action_threshold_over".to_string(),
                        ))
                    }
                };
                self.set_action_threshold_over(action).await;
                Ok(())
            }
            Self::ATTR_ACTION_THRESHOLD_UNDER => {
                let action = match value {
                    DataObject::Enumerate(a) => LimiterAction::from_u8(a),
                    DataObject::Unsigned8(a) => LimiterAction::from_u8(a),
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Enumerate for action_threshold_under".to_string(),
                        ))
                    }
                };
                self.set_action_threshold_under(action).await;
                Ok(())
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Limiter has no attribute {}",
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
        match method_id {
            Self::METHOD_REMOTE_DISCONNECT => {
                self.remote_disconnect().await?;
                Ok(None)
            }
            Self::METHOD_REMOTE_RECONNECT => {
                self.remote_reconnect().await?;
                Ok(None)
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Limiter has no method {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_limiter_class_id() {
        let limiter = Limiter::with_default_obis(1000, 900);
        assert_eq!(limiter.class_id(), 71);
    }

    #[tokio::test]
    async fn test_limiter_obis_code() {
        let limiter = Limiter::with_default_obis(1000, 900);
        assert_eq!(limiter.obis_code(), Limiter::default_obis());
    }

    #[tokio::test]
    async fn test_limiter_initial_state() {
        let limiter = Limiter::with_default_obis(1000, 900);
        assert_eq!(limiter.threshold_active().await, 1000);
        assert_eq!(limiter.threshold_active_normal().await, 900);
        assert!(!limiter.is_limit_active().await);
        assert_eq!(limiter.action_threshold_over().await, LimiterAction::Disconnect);
        assert_eq!(limiter.action_threshold_under().await, LimiterAction::Reconnect);
    }

    #[tokio::test]
    async fn test_limiter_set_thresholds() {
        let limiter = Limiter::with_default_obis(1000, 900);
        limiter.set_threshold_active(2000).await;
        limiter.set_threshold_active_normal(1800).await;

        assert_eq!(limiter.threshold_active().await, 2000);
        assert_eq!(limiter.threshold_active_normal().await, 1800);
    }

    #[tokio::test]
    async fn test_limiter_reactive_thresholds() {
        let limiter = Limiter::with_default_obis(1000, 900);

        assert!(limiter.threshold_reactive().await.is_none());
        limiter.set_threshold_reactive(Some(500)).await;
        assert_eq!(limiter.threshold_reactive().await, Some(500));

        limiter.set_threshold_reactive_normal(Some(400)).await;
        assert_eq!(limiter.threshold_reactive_normal().await, Some(400));
    }

    #[tokio::test]
    async fn test_limiter_update_active_power() {
        let limiter = Limiter::with_default_obis(1000, 900);

        // Value below threshold
        assert!(!limiter.update_active_power(800).await);
        assert!(!limiter.is_limit_active().await);

        // Value exceeds threshold
        assert!(limiter.update_active_power(1100).await);
        assert!(limiter.is_limit_active().await);

        // Value back to normal
        assert!(limiter.update_active_power(850).await);
        assert!(!limiter.is_limit_active().await);
    }

    #[tokio::test]
    async fn test_limiter_remote_disconnect() {
        let limiter = Limiter::with_default_obis(1000, 900);
        limiter.remote_disconnect().await.unwrap();
        assert!(limiter.is_limit_active().await);
    }

    #[tokio::test]
    async fn test_limiter_remote_reconnect() {
        let limiter = Limiter::with_default_obis(1000, 900);
        limiter.set_limit_active(true).await;
        limiter.remote_reconnect().await.unwrap();
        assert!(!limiter.is_limit_active().await);
    }

    #[tokio::test]
    async fn test_limiter_set_actions() {
        let limiter = Limiter::with_default_obis(1000, 900);

        limiter.set_action_threshold_over(LimiterAction::SendAlarm).await;
        assert_eq!(limiter.action_threshold_over().await, LimiterAction::SendAlarm);

        limiter
            .set_action_threshold_under(LimiterAction::NoAction)
            .await;
        assert_eq!(
            limiter.action_threshold_under().await,
            LimiterAction::NoAction
        );
    }

    #[tokio::test]
    async fn test_limiter_get_logical_name() {
        let limiter = Limiter::with_default_obis(1000, 900);
        let result = limiter.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_limiter_get_thresholds() {
        let limiter = Limiter::with_default_obis(1000, 900);
        let result = limiter.get_attribute(2, None).await.unwrap();
        assert_eq!(result, DataObject::Integer64(1000));

        let result = limiter.get_attribute(3, None).await.unwrap();
        assert_eq!(result, DataObject::Integer64(900));
    }

    #[tokio::test]
    async fn test_limiter_get_reactive_thresholds() {
        let limiter = Limiter::with_default_obis(1000, 900);

        let result = limiter.get_attribute(4, None).await.unwrap();
        assert_eq!(result, DataObject::Null);

        limiter.set_threshold_reactive(Some(500)).await;
        let result = limiter.get_attribute(4, None).await.unwrap();
        assert_eq!(result, DataObject::Integer64(500));
    }

    #[tokio::test]
    async fn test_limiter_set_thresholds_via_attribute() {
        let limiter = Limiter::with_default_obis(1000, 900);
        limiter
            .set_attribute(2, DataObject::Integer64(1500), None)
            .await
            .unwrap();
        assert_eq!(limiter.threshold_active().await, 1500);
    }

    #[tokio::test]
    async fn test_limiter_set_actions_via_attribute() {
        let limiter = Limiter::with_default_obis(1000, 900);
        limiter
            .set_attribute(6, DataObject::Enumerate(3), None)
            .await
            .unwrap();
        assert_eq!(limiter.action_threshold_over().await, LimiterAction::SendAlarm);
    }

    #[tokio::test]
    async fn test_limiter_read_only_logical_name() {
        let limiter = Limiter::with_default_obis(1000, 900);
        let result = limiter
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 97, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_limiter_method_disconnect() {
        let limiter = Limiter::with_default_obis(1000, 900);
        limiter.invoke_method(1, None, None).await.unwrap();
        assert!(limiter.is_limit_active().await);
    }

    #[tokio::test]
    async fn test_limiter_method_reconnect() {
        let limiter = Limiter::with_default_obis(1000, 900);
        limiter.set_limit_active(true).await;
        limiter.invoke_method(2, None, None).await.unwrap();
        assert!(!limiter.is_limit_active().await);
    }

    #[tokio::test]
    async fn test_limiter_invalid_attribute() {
        let limiter = Limiter::with_default_obis(1000, 900);
        let result = limiter.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_limiter_invalid_method() {
        let limiter = Limiter::with_default_obis(1000, 900);
        let result = limiter.invoke_method(99, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_limiter_thresholds_tuple() {
        let limiter = Limiter::with_default_obis(1000, 900);
        limiter.set_threshold_reactive(Some(500)).await;
        limiter.set_threshold_reactive_normal(Some(400)).await;

        let (active, active_normal, reactive, reactive_normal) = limiter.thresholds().await;
        assert_eq!(active, 1000);
        assert_eq!(active_normal, 900);
        assert_eq!(reactive, Some(500));
        assert_eq!(reactive_normal, Some(400));
    }

    #[tokio::test]
    async fn test_limiter_action_from_u8() {
        assert_eq!(LimiterAction::from_u8(0), LimiterAction::NoAction);
        assert_eq!(LimiterAction::from_u8(1), LimiterAction::Disconnect);
        assert_eq!(LimiterAction::from_u8(2), LimiterAction::Reconnect);
        assert_eq!(LimiterAction::from_u8(3), LimiterAction::SendAlarm);
        assert_eq!(
            LimiterAction::from_u8(4),
            LimiterAction::SendAlarmAndDisconnect
        );
        assert_eq!(LimiterAction::from_u8(99), LimiterAction::NoAction);
    }
}
