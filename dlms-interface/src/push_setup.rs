//! Push Setup interface class (Class ID: 40)
//!
//! The Push Setup interface class configures push operations where
//! the meter initiates data transfer to the client (push mode).
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: push_object_list - List of objects to push
//! - Attribute 3: send_destination_and_method - Destination address and method
//! - Attribute 4: communication_window - Time window for push attempts
//! - Attribute 5: random_start_delay - Random delay before first push
//! - Attribute 6: number_of_push_attempts - Number of retry attempts
//! - Attribute 7: push_repeat_time - Time between repeated pushes
//!
//! # Methods
//!
//! - Method 1: push(id) - Initiate a push operation
//!
//! # Push Setup (Class ID: 40)
//!
//! This class enables the meter to proactively send data to a client,
//! useful for alarm notifications and regular reporting.

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{CosemObject, descriptor::ObisCodeExt};

/// Communication window for push operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommunicationWindow {
    /// Start time in minutes from midnight (0-1439)
    pub start_minute: u16,
    /// End time in minutes from midnight (0-1439)
    pub end_minute: u16,
}

impl CommunicationWindow {
    /// Create a new communication window
    pub fn new(start_minute: u16, end_minute: u16) -> DlmsResult<Self> {
        if start_minute > 1439 {
            return Err(DlmsError::InvalidData(
                "start_minute must be 0-1439".to_string(),
            ));
        }
        if end_minute > 1439 {
            return Err(DlmsError::InvalidData(
                "end_minute must be 0-1439".to_string(),
            ));
        }
        Ok(Self {
            start_minute,
            end_minute,
        })
    }

    /// Create from data object (array of 2 unsigned16)
    pub fn from_data_object(value: &DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Array(arr) if arr.len() >= 2 => {
                let start = match &arr[0] {
                    DataObject::Unsigned16(v) => *v,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Unsigned16 for start_minute".to_string(),
                        ))
                    }
                };
                let end = match &arr[1] {
                    DataObject::Unsigned16(v) => *v,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Unsigned16 for end_minute".to_string(),
                        ))
                    }
                };
                Self::new(start, end)
            }
            _ => Err(DlmsError::InvalidData(
                "Expected Array for CommunicationWindow".to_string(),
            )),
        }
    }

    /// Convert to data object
    pub fn to_data_object(&self) -> DataObject {
        DataObject::Array(vec![
            DataObject::Unsigned16(self.start_minute),
            DataObject::Unsigned16(self.end_minute),
        ])
    }

    /// Check if a given minute is within the window
    pub fn contains(&self, minute: u16) -> bool {
        if self.start_minute <= self.end_minute {
            minute >= self.start_minute && minute <= self.end_minute
        } else {
            // Window wraps around midnight
            minute >= self.start_minute || minute <= self.end_minute
        }
    }

    /// Duration of the window in minutes
    pub fn duration_minutes(&self) -> u16 {
        if self.start_minute <= self.end_minute {
            self.end_minute - self.start_minute
        } else {
            (1440 - self.start_minute) + self.end_minute
        }
    }
}

/// Push object definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushObjectDefinition {
    /// Class ID of the object
    pub class_id: u16,
    /// Logical name (OBIS code) of the object
    pub obis_code: ObisCode,
    /// Attribute index to push
    pub attribute_index: u8,
}

impl PushObjectDefinition {
    /// Create a new push object definition
    pub fn new(class_id: u16, obis_code: ObisCode, attribute_index: u8) -> Self {
        Self {
            class_id,
            obis_code,
            attribute_index,
        }
    }

    /// Create from data object (structure)
    pub fn from_data_object(value: &DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Array(arr) if arr.len() >= 3 => {
                let class_id = match &arr[0] {
                    DataObject::Unsigned16(v) => *v,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Unsigned16 for class_id".to_string(),
                        ))
                    }
                };
                let obis_code = match &arr[1] {
                    DataObject::OctetString(bytes) if bytes.len() == 6 => {
                        ObisCode::from_bytes(bytes)?
                    }
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected OctetString(6) for obis_code".to_string(),
                        ))
                    }
                };
                let attribute_index = match &arr[2] {
                    DataObject::Unsigned8(v) => *v,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected Unsigned8 for attribute_index".to_string(),
                        ))
                    }
                };
                Ok(Self {
                    class_id,
                    obis_code,
                    attribute_index,
                })
            }
            _ => Err(DlmsError::InvalidData(
                "Expected Array for PushObjectDefinition".to_string(),
            )),
        }
    }

    /// Convert to data object
    pub fn to_data_object(&self) -> DataObject {
        DataObject::Array(vec![
            DataObject::Unsigned16(self.class_id),
            DataObject::OctetString(self.obis_code.to_bytes().to_vec()),
            DataObject::Unsigned8(self.attribute_index),
        ])
    }
}

/// Push destination method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PushDestinationMethod {
    /// No destination
    None = 0,
    /// TCP
    Tcp = 1,
    /// UDP
    Udp = 2,
    /// SMS
    Sms = 3,
    /// FTP
    Ftp = 4,
    /// SMTP (email)
    Smtp = 5,
    /// HTTP
    Http = 6,
    /// HTTPS
    Https = 7,
}

impl PushDestinationMethod {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Tcp,
            2 => Self::Udp,
            3 => Self::Sms,
            4 => Self::Ftp,
            5 => Self::Smtp,
            6 => Self::Http,
            7 => Self::Https,
            _ => Self::None,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Push Setup interface class (Class ID: 40)
///
/// Default OBIS: 0-0:42.0.0.255
///
/// This class configures push operations for proactive data transfer.
#[derive(Debug, Clone)]
pub struct PushSetup {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// List of objects to push
    push_object_list: Arc<RwLock<Vec<PushObjectDefinition>>>,

    /// Send destination and method
    destination: Arc<RwLock<Option<String>>>,
    destination_method: Arc<RwLock<PushDestinationMethod>>,

    /// Communication window for push attempts
    communication_window: Arc<RwLock<Option<CommunicationWindow>>>,

    /// Random start delay before first push (seconds)
    random_start_delay: Arc<RwLock<u16>>,

    /// Number of push attempts
    number_of_push_attempts: Arc<RwLock<u8>>,

    /// Time between repeated push attempts (seconds)
    push_repeat_time: Arc<RwLock<u32>>,

    /// Enable push flag
    push_enabled: Arc<RwLock<bool>>,
}

impl PushSetup {
    /// Class ID for Push Setup
    pub const CLASS_ID: u16 = 40;

    /// Default OBIS code for Push Setup (0-0:42.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 42, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_PUSH_OBJECT_LIST: u8 = 2;
    pub const ATTR_SEND_DESTINATION_AND_METHOD: u8 = 3;
    pub const ATTR_COMMUNICATION_WINDOW: u8 = 4;
    pub const ATTR_RANDOM_START_DELAY: u8 = 5;
    pub const ATTR_NUMBER_OF_PUSH_ATTEMPTS: u8 = 6;
    pub const ATTR_PUSH_REPEAT_TIME: u8 = 7;

    /// Method IDs
    pub const METHOD_PUSH: u8 = 1;

    /// Create a new Push Setup object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `destination` - Optional destination address
    /// * `destination_method` - Method for sending push data
    pub fn new(
        logical_name: ObisCode,
        destination: Option<String>,
        destination_method: PushDestinationMethod,
    ) -> Self {
        Self {
            logical_name,
            push_object_list: Arc::new(RwLock::new(Vec::new())),
            destination: Arc::new(RwLock::new(destination)),
            destination_method: Arc::new(RwLock::new(destination_method)),
            communication_window: Arc::new(RwLock::new(None)),
            random_start_delay: Arc::new(RwLock::new(0)),
            number_of_push_attempts: Arc::new(RwLock::new(3)),
            push_repeat_time: Arc::new(RwLock::new(60)),
            push_enabled: Arc::new(RwLock::new(true)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis(), None, PushDestinationMethod::Tcp)
    }

    /// Get the push object list
    pub async fn push_object_list(&self) -> Vec<PushObjectDefinition> {
        self.push_object_list.read().await.clone()
    }

    /// Add an object to the push list
    pub async fn add_push_object(&self, object: PushObjectDefinition) {
        self.push_object_list.write().await.push(object);
    }

    /// Remove an object from the push list by index
    pub async fn remove_push_object(&self, index: usize) -> DlmsResult<()> {
        let mut list = self.push_object_list.write().await;
        if index >= list.len() {
            return Err(DlmsError::InvalidData("Index out of bounds".to_string()));
        }
        list.remove(index);
        Ok(())
    }

    /// Clear all push objects
    pub async fn clear_push_objects(&self) {
        self.push_object_list.write().await.clear();
    }

    /// Get the destination
    pub async fn destination(&self) -> Option<String> {
        self.destination.read().await.clone()
    }

    /// Set the destination
    pub async fn set_destination(&self, destination: Option<String>) {
        *self.destination.write().await = destination;
    }

    /// Get the destination method
    pub async fn destination_method(&self) -> PushDestinationMethod {
        *self.destination_method.read().await
    }

    /// Set the destination method
    pub async fn set_destination_method(&self, method: PushDestinationMethod) {
        *self.destination_method.write().await = method;
    }

    /// Get the communication window
    pub async fn communication_window(&self) -> Option<CommunicationWindow> {
        self.communication_window.read().await.clone()
    }

    /// Set the communication window
    pub async fn set_communication_window(&self, window: Option<CommunicationWindow>) {
        *self.communication_window.write().await = window;
    }

    /// Get the random start delay
    pub async fn random_start_delay(&self) -> u16 {
        *self.random_start_delay.read().await
    }

    /// Set the random start delay
    pub async fn set_random_start_delay(&self, delay: u16) {
        *self.random_start_delay.write().await = delay;
    }

    /// Get the number of push attempts
    pub async fn number_of_push_attempts(&self) -> u8 {
        *self.number_of_push_attempts.read().await
    }

    /// Set the number of push attempts
    pub async fn set_number_of_push_attempts(&self, attempts: u8) {
        *self.number_of_push_attempts.write().await = attempts;
    }

    /// Get the push repeat time
    pub async fn push_repeat_time(&self) -> u32 {
        *self.push_repeat_time.read().await
    }

    /// Set the push repeat time
    pub async fn set_push_repeat_time(&self, time: u32) {
        *self.push_repeat_time.write().await = time;
    }

    /// Check if push is enabled
    pub async fn is_push_enabled(&self) -> bool {
        *self.push_enabled.read().await
    }

    /// Enable or disable push
    pub async fn set_push_enabled(&self, enabled: bool) {
        *self.push_enabled.write().await = enabled;
    }

    /// Push - initiate a push operation
    ///
    /// This corresponds to Method 1
    pub async fn push(&self, push_object_id: u8) -> DlmsResult<()> {
        if !self.is_push_enabled().await {
            return Err(DlmsError::AccessDenied("Push is disabled".to_string()));
        }
        if self.destination().await.is_none() {
            return Err(DlmsError::InvalidData("No destination configured".to_string()));
        }
        // In a real implementation, this would trigger the actual push
        // For now, we just validate the parameters
        if push_object_id == 0 || push_object_id > self.push_object_list().await.len() as u8 {
            return Err(DlmsError::InvalidData("Invalid push_object_id".to_string()));
        }
        Ok(())
    }

    /// Check if current time is within communication window
    pub async fn is_within_communication_window(&self, current_minute: u16) -> bool {
        match self.communication_window().await {
            Some(window) => window.contains(current_minute),
            None => true, // No window restriction
        }
    }
}

#[async_trait]
impl CosemObject for PushSetup {
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
            Self::ATTR_PUSH_OBJECT_LIST => {
                let objects = self.push_object_list().await;
                let data: Vec<DataObject> =
                    objects.iter().map(|o| o.to_data_object()).collect();
                Ok(DataObject::Array(data))
            }
            Self::ATTR_SEND_DESTINATION_AND_METHOD => {
                let dest = self.destination().await.unwrap_or_default();
                let method = self.destination_method().await.to_u8();
                Ok(DataObject::Array(vec![
                    DataObject::OctetString(dest.into_bytes()),
                    DataObject::Enumerate(method),
                ]))
            }
            Self::ATTR_COMMUNICATION_WINDOW => {
                match self.communication_window().await {
                    Some(window) => Ok(window.to_data_object()),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_RANDOM_START_DELAY => {
                Ok(DataObject::Unsigned16(self.random_start_delay().await))
            }
            Self::ATTR_NUMBER_OF_PUSH_ATTEMPTS => {
                Ok(DataObject::Unsigned8(self.number_of_push_attempts().await))
            }
            Self::ATTR_PUSH_REPEAT_TIME => {
                Ok(DataObject::Unsigned32(self.push_repeat_time().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Push Setup has no attribute {}",
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
            Self::ATTR_PUSH_OBJECT_LIST => {
                match value {
                    DataObject::Array(arr) => {
                        self.clear_push_objects().await;
                        for item in arr {
                            let obj = PushObjectDefinition::from_data_object(&item)?;
                            self.add_push_object(obj).await;
                        }
                        Ok(())
                    }
                    DataObject::Null => {
                        self.clear_push_objects().await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Array for push_object_list".to_string(),
                    )),
                }
            }
            Self::ATTR_SEND_DESTINATION_AND_METHOD => {
                match value {
                    DataObject::Array(arr) if arr.len() >= 2 => {
                        let dest = match &arr[0] {
                            DataObject::OctetString(bytes) => {
                                String::from_utf8(bytes.clone()).unwrap_or_default()
                            }
                            DataObject::Null => String::new(),
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Expected OctetString for destination".to_string(),
                                ))
                            }
                        };
                        let method = match &arr[1] {
                            DataObject::Enumerate(m) => PushDestinationMethod::from_u8(*m),
                            DataObject::Unsigned8(m) => PushDestinationMethod::from_u8(*m),
                            _ => {
                                return Err(DlmsError::InvalidData(
                                    "Expected Enumerate for method".to_string(),
                                ))
                            }
                        };
                        self.set_destination(if dest.is_empty() { None } else { Some(dest) }).await;
                        self.set_destination_method(method).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Array for send_destination_and_method".to_string(),
                    )),
                }
            }
            Self::ATTR_COMMUNICATION_WINDOW => {
                match value {
                    DataObject::Array(_) => {
                        let window = CommunicationWindow::from_data_object(&value)?;
                        self.set_communication_window(Some(window)).await;
                        Ok(())
                    }
                    DataObject::Null => {
                        self.set_communication_window(None).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Array or Null for communication_window".to_string(),
                    )),
                }
            }
            Self::ATTR_RANDOM_START_DELAY => {
                if let DataObject::Unsigned16(delay) = value {
                    self.set_random_start_delay(delay).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned16 for random_start_delay".to_string(),
                    ))
                }
            }
            Self::ATTR_NUMBER_OF_PUSH_ATTEMPTS => {
                if let DataObject::Unsigned8(attempts) = value {
                    self.set_number_of_push_attempts(attempts).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for number_of_push_attempts".to_string(),
                    ))
                }
            }
            Self::ATTR_PUSH_REPEAT_TIME => {
                if let DataObject::Unsigned32(time) = value {
                    self.set_push_repeat_time(time).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned32 for push_repeat_time".to_string(),
                    ))
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Push Setup has no attribute {}",
                attribute_id
            ))),
        }
    }

    async fn invoke_method(
        &self,
        method_id: u8,
        parameters: Option<DataObject>,
        _selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<Option<DataObject>> {
        match method_id {
            Self::METHOD_PUSH => {
                let push_id = match parameters {
                    Some(DataObject::Unsigned8(id)) => id,
                    Some(DataObject::Unsigned16(id)) => id as u8,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Method 1 expects Unsigned8 parameter".to_string(),
                        ))
                    }
                };
                self.push(push_id).await?;
                Ok(None)
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Push Setup has no method {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_push_setup_class_id() {
        let setup = PushSetup::with_default_obis();
        assert_eq!(setup.class_id(), 40);
    }

    #[tokio::test]
    async fn test_push_setup_obis_code() {
        let setup = PushSetup::with_default_obis();
        assert_eq!(setup.obis_code(), PushSetup::default_obis());
    }

    #[tokio::test]
    async fn test_push_setup_initial_state() {
        let setup = PushSetup::with_default_obis();
        assert!(setup.push_object_list().await.is_empty());
        assert!(setup.destination().await.is_none());
        assert_eq!(setup.destination_method().await, PushDestinationMethod::Tcp);
        assert_eq!(setup.number_of_push_attempts().await, 3);
        assert_eq!(setup.push_repeat_time().await, 60);
        assert!(setup.is_push_enabled().await);
    }

    #[tokio::test]
    async fn test_push_setup_add_object() {
        let setup = PushSetup::with_default_obis();
        let obj = PushObjectDefinition::new(3, ObisCode::new(1, 1, 1, 1, 1, 255), 2);
        setup.add_push_object(obj.clone()).await;

        let list = setup.push_object_list().await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0], obj);
    }

    #[tokio::test]
    async fn test_push_setup_remove_object() {
        let setup = PushSetup::with_default_obis();
        setup
            .add_push_object(PushObjectDefinition::new(3, ObisCode::new(1, 1, 1, 1, 1, 255), 2))
            .await;
        setup
            .add_push_object(PushObjectDefinition::new(3, ObisCode::new(1, 1, 1, 1, 2, 255), 2))
            .await;

        setup.remove_push_object(0).await.unwrap();
        assert_eq!(setup.push_object_list().await.len(), 1);
    }

    #[tokio::test]
    async fn test_push_setup_clear_objects() {
        let setup = PushSetup::with_default_obis();
        setup
            .add_push_object(PushObjectDefinition::new(3, ObisCode::new(1, 1, 1, 1, 1, 255), 2))
            .await;
        setup.clear_push_objects().await;
        assert!(setup.push_object_list().await.is_empty());
    }

    #[tokio::test]
    async fn test_push_setup_set_destination() {
        let setup = PushSetup::with_default_obis();
        setup.set_destination(Some("example.com:8080".to_string())).await;
        assert_eq!(setup.destination().await, Some("example.com:8080".to_string()));
    }

    #[tokio::test]
    async fn test_push_setup_set_method() {
        let setup = PushSetup::with_default_obis();
        setup.set_destination_method(PushDestinationMethod::Https).await;
        assert_eq!(setup.destination_method().await, PushDestinationMethod::Https);
    }

    #[tokio::test]
    async fn test_push_setup_communication_window() {
        let setup = PushSetup::with_default_obis();
        let window = CommunicationWindow::new(600, 1200).unwrap(); // 10:00 to 20:00
        setup.set_communication_window(Some(window.clone())).await;

        let retrieved = setup.communication_window().await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), window);
    }

    #[tokio::test]
    async fn test_communication_window_contains() {
        let window = CommunicationWindow::new(600, 1200).unwrap();
        assert!(window.contains(900)); // 15:00
        assert!(!window.contains(300)); // 05:00
    }

    #[tokio::test]
    async fn test_communication_window_wraps_midnight() {
        let window = CommunicationWindow::new(1320, 120).unwrap(); // 22:00 to 02:00
        assert!(window.contains(60)); // 01:00
        assert!(!window.contains(720)); // 12:00
    }

    #[tokio::test]
    async fn test_communication_window_invalid() {
        assert!(CommunicationWindow::new(1440, 100).is_err());
        assert!(CommunicationWindow::new(100, 1440).is_err());
    }

    #[tokio::test]
    async fn test_push_setup_push_disabled() {
        let setup = PushSetup::with_default_obis();
        setup.set_push_enabled(false).await;
        let result = setup.push(1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_push_setup_push_no_destination() {
        let setup = PushSetup::with_default_obis();
        let obj = PushObjectDefinition::new(3, ObisCode::new(1, 1, 1, 1, 1, 255), 2);
        setup.add_push_object(obj).await;
        let result = setup.push(1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_push_setup_push_success() {
        let setup = PushSetup::with_default_obis();
        setup.set_destination(Some("example.com:8080".to_string())).await;
        let obj = PushObjectDefinition::new(3, ObisCode::new(1, 1, 1, 1, 1, 255), 2);
        setup.add_push_object(obj).await;
        assert!(setup.push(1).await.is_ok());
    }

    #[tokio::test]
    async fn test_push_setup_get_logical_name() {
        let setup = PushSetup::with_default_obis();
        let result = setup.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_push_setup_get_push_object_list() {
        let setup = PushSetup::with_default_obis();
        let result = setup.get_attribute(2, None).await.unwrap();

        match result {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 0);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_push_setup_get_destination() {
        let setup = PushSetup::with_default_obis();
        setup.set_destination(Some("test.com".to_string())).await;
        let result = setup.get_attribute(3, None).await.unwrap();

        match result {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 2);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_push_setup_get_communication_window() {
        let setup = PushSetup::with_default_obis();
        let result = setup.get_attribute(4, None).await.unwrap();
        assert_eq!(result, DataObject::Null);

        let window = CommunicationWindow::new(600, 1200).unwrap();
        setup.set_communication_window(Some(window)).await;
        let result = setup.get_attribute(4, None).await.unwrap();

        match result {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 2);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_push_setup_get_repeat_time() {
        let setup = PushSetup::with_default_obis();
        let result = setup.get_attribute(7, None).await.unwrap();
        assert_eq!(result, DataObject::Unsigned32(60));
    }

    #[tokio::test]
    async fn test_push_setup_set_object_list_via_attribute() {
        let setup = PushSetup::with_default_obis();
        let obj_data = DataObject::Array(vec![
            DataObject::Unsigned16(3),
            DataObject::OctetString(vec![1, 1, 1, 1, 1, 255]),
            DataObject::Unsigned8(2),
        ]);
        setup
            .set_attribute(2, DataObject::Array(vec![obj_data]), None)
            .await
            .unwrap();

        assert_eq!(setup.push_object_list().await.len(), 1);
    }

    #[tokio::test]
    async fn test_push_setup_set_destination_via_attribute() {
        let setup = PushSetup::with_default_obis();
        setup
            .set_attribute(
                3,
                DataObject::Array(vec![
                    DataObject::OctetString(b"test.com".to_vec()),
                    DataObject::Enumerate(6), // HTTP
                ]),
                None,
            )
            .await
            .unwrap();

        assert_eq!(setup.destination().await, Some("test.com".to_string()));
        assert_eq!(setup.destination_method().await, PushDestinationMethod::Http);
    }

    #[tokio::test]
    async fn test_push_setup_set_repeat_time_via_attribute() {
        let setup = PushSetup::with_default_obis();
        setup
            .set_attribute(7, DataObject::Unsigned32(120), None)
            .await
            .unwrap();
        assert_eq!(setup.push_repeat_time().await, 120);
    }

    #[tokio::test]
    async fn test_push_setup_read_only_logical_name() {
        let setup = PushSetup::with_default_obis();
        let result = setup
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 42, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_push_setup_method_push() {
        let setup = PushSetup::with_default_obis();
        setup.set_destination(Some("test.com".to_string())).await;
        setup
            .add_push_object(PushObjectDefinition::new(3, ObisCode::new(1, 1, 1, 1, 1, 255), 2))
            .await;
        let result = setup.invoke_method(1, Some(DataObject::Unsigned8(1)), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_push_setup_invalid_attribute() {
        let setup = PushSetup::with_default_obis();
        let result = setup.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_push_setup_invalid_method() {
        let setup = PushSetup::with_default_obis();
        let result = setup.invoke_method(99, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_push_destination_method_from_u8() {
        assert_eq!(PushDestinationMethod::from_u8(0), PushDestinationMethod::None);
        assert_eq!(PushDestinationMethod::from_u8(1), PushDestinationMethod::Tcp);
        assert_eq!(PushDestinationMethod::from_u8(2), PushDestinationMethod::Udp);
        assert_eq!(PushDestinationMethod::from_u8(3), PushDestinationMethod::Sms);
        assert_eq!(PushDestinationMethod::from_u8(4), PushDestinationMethod::Ftp);
        assert_eq!(PushDestinationMethod::from_u8(5), PushDestinationMethod::Smtp);
        assert_eq!(PushDestinationMethod::from_u8(6), PushDestinationMethod::Http);
        assert_eq!(PushDestinationMethod::from_u8(7), PushDestinationMethod::Https);
        assert_eq!(PushDestinationMethod::from_u8(99), PushDestinationMethod::None);
    }

    #[tokio::test]
    async fn test_push_object_definition_to_data_object() {
        let obj = PushObjectDefinition::new(3, ObisCode::new(1, 1, 1, 1, 1, 255), 2);
        let data = obj.to_data_object();

        match data {
            DataObject::Array(arr) => {
                assert_eq!(arr.len(), 3);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_push_object_definition_from_data_object() {
        let data = DataObject::Array(vec![
            DataObject::Unsigned16(3),
            DataObject::OctetString(vec![1, 1, 1, 1, 1, 255]),
            DataObject::Unsigned8(2),
        ]);

        let obj = PushObjectDefinition::from_data_object(&data).unwrap();
        assert_eq!(obj.class_id, 3);
        assert_eq!(obj.attribute_index, 2);
    }

    #[tokio::test]
    async fn test_push_setup_is_within_window() {
        let setup = PushSetup::with_default_obis();
        assert!(setup.is_within_communication_window(720).await); // 12:00, no window restriction

        let window = CommunicationWindow::new(600, 1200).unwrap();
        setup.set_communication_window(Some(window)).await;
        assert!(setup.is_within_communication_window(900).await);
        assert!(!setup.is_within_communication_window(300).await);
    }
}
