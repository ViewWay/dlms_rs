//! Profile Generic interface class (Class ID: 7)
//!
//! The Profile Generic interface class represents a log or buffer of data
//! stored in the meter, typically used for load profile, event log, or
//! historical data recording.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: buffer - The profile buffer (array of data records)
//! - Attribute 3: buffer_timestamp - Timestamp of the last buffer update
//! - Attribute 4: capture_objects - List of objects to capture
//! - Attribute 5: capture_period - Period in seconds between captures (0 = on demand)
//! - Attribute 6: sort_method - Sort method for the buffer (FIFO/LIFO)
//! - Attribute 7: sort_object - Object used for sorting (optional)
//! - Attribute 8: sort_attribute - Attribute of sort object (optional)
//! - Attribute 9: entries_in_use - Number of entries currently in use
//! - Attribute 10: profile_buffer_status - Status flags for the buffer
//!
//! # Methods
//!
//! - Method 1: reset() - Clear the buffer
//! - Method 2: capture() - Perform an immediate capture

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{
    datatypes::{CosemDateTime, CosemDateFormat},
    DlmsError, DlmsResult, ObisCode, DataObject,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{CosemObject, CosemObjectDescriptor, ObisCodeExt};

/// Sort method for the profile buffer (specific to Profile Generic)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileSortMethod {
    /// First In First Out
    Fifo = 0,
    /// Last In First Out
    Lifo = 1,
}

impl ProfileSortMethod {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ProfileSortMethod::Fifo),
            1 => Some(ProfileSortMethod::Lifo),
            _ => None,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Profile buffer status flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileBufferStatus {
    /// Capture is active
    CaptureActive = 0x01,
    /// Buffer is full
    BufferFull = 0x02,
    /// Data not available
    DataNotAvailable = 0x04,
}

impl ProfileBufferStatus {
    pub fn to_byte(statuses: &[ProfileBufferStatus]) -> u8 {
        let mut byte = 0u8;
        for status in statuses {
            byte |= *status as u8;
        }
        byte
    }

    pub fn from_byte(byte: u8) -> Vec<ProfileBufferStatus> {
        let mut statuses = Vec::new();
        if byte & ProfileBufferStatus::CaptureActive as u8 != 0 {
            statuses.push(ProfileBufferStatus::CaptureActive);
        }
        if byte & ProfileBufferStatus::BufferFull as u8 != 0 {
            statuses.push(ProfileBufferStatus::BufferFull);
        }
        if byte & ProfileBufferStatus::DataNotAvailable as u8 != 0 {
            statuses.push(ProfileBufferStatus::DataNotAvailable);
        }
        statuses
    }
}

/// A single entry in the profile buffer
#[derive(Debug, Clone)]
pub struct GenericProfileEntry {
    /// Timestamp of this entry
    pub timestamp: CosemDateTime,
    /// Data values captured for each capture object
    pub values: Vec<DataObject>,
}

impl GenericProfileEntry {
    /// Create a new profile entry
    pub fn new(timestamp: CosemDateTime, values: Vec<DataObject>) -> Self {
        Self { timestamp, values }
    }

    /// Encode as a DataObject structure
    pub fn encode(&self) -> DataObject {
        let mut fields = Vec::new();
        fields.push(DataObject::OctetString(self.timestamp.encode()));
        fields.extend(self.values.clone());
        DataObject::Structure(fields)
    }

    /// Decode from a DataObject structure
    pub fn decode(data: &DataObject) -> DlmsResult<Self> {
        match data {
            DataObject::Structure(fields) if !fields.is_empty() => {
                let timestamp = match &fields[0] {
                    DataObject::OctetString(bytes) if bytes.len() >= 12 => {
                        CosemDateTime::decode(bytes)?
                    }
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Expected timestamp as first field".to_string(),
                        ))
                    }
                };

                let values = fields[1..].to_vec();
                Ok(Self { timestamp, values })
            }
            _ => Err(DlmsError::InvalidData(
                "Expected Structure for GenericProfileEntry".to_string(),
            )),
        }
    }
}

/// Profile Generic interface class (Class ID: 7)
///
/// Default OBIS: 1-0:99.1.0.255 (typical for load profile)
///
/// This class represents a log or buffer of stored data in the meter.
/// It is commonly used for:
/// - Load profiles (energy consumption over time)
/// - Event logs (recording meter events)
/// - Historical data (billing data storage)
#[derive(Debug, Clone)]
pub struct ProfileGeneric {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// The profile buffer
    buffer: Arc<RwLock<Vec<GenericProfileEntry>>>,

    /// Timestamp of last buffer update
    buffer_timestamp: Arc<RwLock<Option<CosemDateTime>>>,

    /// List of objects to capture
    capture_objects: Arc<RwLock<Vec<CosemObjectDescriptor>>>,

    /// Capture period in seconds (0 = on demand only)
    capture_period: Arc<RwLock<u32>>,

    /// Sort method for the buffer
    sort_method: Arc<RwLock<ProfileSortMethod>>,

    /// Status flags for the buffer
    buffer_status: Arc<RwLock<u8>>,

    /// Maximum size of the buffer
    max_buffer_size: usize,
}

impl ProfileGeneric {
    /// Class ID for Profile Generic
    pub const CLASS_ID: u16 = 7;

    /// Default OBIS code for Profile Generic (1-0:99.1.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(1, 0, 99, 1, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_BUFFER: u8 = 2;
    pub const ATTR_BUFFER_TIMESTAMP: u8 = 3;
    pub const ATTR_CAPTURE_OBJECTS: u8 = 4;
    pub const ATTR_CAPTURE_PERIOD: u8 = 5;
    pub const ATTR_SORT_METHOD: u8 = 6;
    pub const ATTR_SORT_OBJECT: u8 = 7;
    pub const ATTR_SORT_ATTRIBUTE: u8 = 8;
    pub const ATTR_ENTRIES_IN_USE: u8 = 9;
    pub const ATTR_PROFILE_BUFFER_STATUS: u8 = 10;

    /// Method IDs
    pub const METHOD_RESET: u8 = 1;
    pub const METHOD_CAPTURE: u8 = 2;

    /// Create a new Profile Generic object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `max_buffer_size` - Maximum number of entries in the buffer
    /// * `capture_period` - Period in seconds between captures (0 = on demand)
    /// * `sort_method` - Sort method (FIFO or LIFO)
    pub fn new(
        logical_name: ObisCode,
        max_buffer_size: usize,
        capture_period: u32,
        sort_method: ProfileSortMethod,
    ) -> Self {
        Self {
            logical_name,
            buffer: Arc::new(RwLock::new(Vec::with_capacity(max_buffer_size))),
            buffer_timestamp: Arc::new(RwLock::new(None)),
            capture_objects: Arc::new(RwLock::new(Vec::new())),
            capture_period: Arc::new(RwLock::new(capture_period)),
            sort_method: Arc::new(RwLock::new(sort_method)),
            buffer_status: Arc::new(RwLock::new(0)),
            max_buffer_size,
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis(max_buffer_size: usize) -> Self {
        Self::new(
            Self::default_obis(),
            max_buffer_size,
            900, // 15 minutes default
            ProfileSortMethod::Fifo,
        )
    }

    /// Get the buffer entries
    pub async fn buffer(&self) -> Vec<GenericProfileEntry> {
        self.buffer.read().await.clone()
    }

    /// Get the number of entries in use
    pub async fn entries_in_use(&self) -> u32 {
        self.buffer.read().await.len() as u32
    }

    /// Get the buffer timestamp
    pub async fn buffer_timestamp(&self) -> Option<CosemDateTime> {
        self.buffer_timestamp.read().await.clone()
    }

    /// Get capture objects
    pub async fn capture_objects(&self) -> Vec<CosemObjectDescriptor> {
        self.capture_objects.read().await.clone()
    }

    /// Set capture objects
    pub async fn set_capture_objects(&self, objects: Vec<CosemObjectDescriptor>) {
        *self.capture_objects.write().await = objects;
    }

    /// Add a capture object
    pub async fn add_capture_object(&self, object: CosemObjectDescriptor) {
        self.capture_objects.write().await.push(object);
    }

    /// Get capture period
    pub async fn capture_period(&self) -> u32 {
        *self.capture_period.read().await
    }

    /// Set capture period
    pub async fn set_capture_period(&self, period: u32) {
        *self.capture_period.write().await = period;
    }

    /// Get sort method
    pub async fn sort_method(&self) -> ProfileSortMethod {
        *self.sort_method.read().await
    }

    /// Set sort method
    pub async fn set_sort_method(&self, method: ProfileSortMethod) {
        *self.sort_method.write().await = method;
    }

    /// Get buffer status
    pub async fn buffer_status(&self) -> u8 {
        *self.buffer_status.read().await
    }

    /// Set buffer status
    pub async fn set_buffer_status(&self, status: u8) {
        *self.buffer_status.write().await = status;
    }

    /// Clear the buffer (reset)
    pub async fn reset(&self) -> DlmsResult<()> {
        self.buffer.write().await.clear();
        *self.buffer_timestamp.write().await = None;
        // Clear buffer full flag
        let mut status = self.buffer_status.write().await;
        *status &= !(ProfileBufferStatus::BufferFull as u8);
        Ok(())
    }

    /// Perform an immediate capture
    pub async fn capture(&self, values: Vec<DataObject>) -> DlmsResult<()> {
        let timestamp = CosemDateTime::new(2024, 1, 1, 0, 0, 0, 0, &[]).unwrap();
        self.capture_with_timestamp(timestamp, values).await
    }

    /// Perform a capture with a specific timestamp
    pub async fn capture_with_timestamp(
        &self,
        timestamp: CosemDateTime,
        values: Vec<DataObject>,
    ) -> DlmsResult<()> {
        let entry = GenericProfileEntry::new(timestamp.clone(), values);

        let mut buffer = self.buffer.write().await;
        let sort_method = *self.sort_method.read().await;

        // Check if buffer is full
        if buffer.len() >= self.max_buffer_size {
            if sort_method == ProfileSortMethod::Fifo {
                // Remove oldest entry
                buffer.remove(0);
            } else {
                // LIFO: buffer is full, can't add more without overwriting
                // For simplicity, remove the first entry
                buffer.remove(0);
            }
        }

        // Add new entry
        if sort_method == ProfileSortMethod::Fifo {
            buffer.push(entry);
        } else {
            // LIFO: add at beginning
            buffer.insert(0, entry);
        }

        // Update timestamp
        *self.buffer_timestamp.write().await = Some(timestamp);

        // Update status flags
        let mut status = self.buffer_status.write().await;
        *status |= ProfileBufferStatus::CaptureActive as u8;
        if buffer.len() >= self.max_buffer_size {
            *status |= ProfileBufferStatus::BufferFull as u8;
        }

        Ok(())
    }

    /// Encode the buffer as a DataObject (array of structures)
    async fn encode_buffer(&self) -> DataObject {
        let buffer = self.buffer.read().await;
        let mut entries = Vec::new();

        for entry in buffer.iter() {
            entries.push(entry.encode());
        }

        DataObject::Array(entries)
    }

    /// Encode capture objects as a DataObject (array of structures)
    async fn encode_capture_objects(&self) -> DataObject {
        let objects = self.capture_objects.read().await;
        let mut descriptors = Vec::new();

        for obj in objects.iter() {
            let mut fields = Vec::new();
            fields.push(DataObject::Unsigned16(obj.class_id));
            fields.push(DataObject::OctetString(obj.logical_name.to_bytes().to_vec()));
            fields.push(DataObject::Unsigned8(obj.version));
            descriptors.push(DataObject::Structure(fields));
        }

        DataObject::Array(descriptors)
    }
}

#[async_trait]
impl CosemObject for ProfileGeneric {
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
            Self::ATTR_BUFFER => {
                Ok(self.encode_buffer().await)
            }
            Self::ATTR_BUFFER_TIMESTAMP => {
                match self.buffer_timestamp.read().await.as_ref() {
                    Some(ts) => Ok(DataObject::OctetString(ts.encode())),
                    None => Ok(DataObject::Null),
                }
            }
            Self::ATTR_CAPTURE_OBJECTS => {
                Ok(self.encode_capture_objects().await)
            }
            Self::ATTR_CAPTURE_PERIOD => {
                Ok(DataObject::Unsigned32(self.capture_period().await))
            }
            Self::ATTR_SORT_METHOD => {
                Ok(DataObject::Unsigned8(self.sort_method().await.to_u8()))
            }
            Self::ATTR_SORT_OBJECT => {
                // No sort object supported in this implementation
                Ok(DataObject::Null)
            }
            Self::ATTR_SORT_ATTRIBUTE => {
                // No sort attribute supported
                Ok(DataObject::Unsigned8(0))
            }
            Self::ATTR_ENTRIES_IN_USE => {
                Ok(DataObject::Unsigned32(self.entries_in_use().await))
            }
            Self::ATTR_PROFILE_BUFFER_STATUS => {
                Ok(DataObject::Unsigned8(self.buffer_status().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Profile Generic has no attribute {}",
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
            Self::ATTR_BUFFER => {
                // Buffer is typically managed through capture operations
                Err(DlmsError::AccessDenied(
                    "Attribute 2 (buffer) is read-only".to_string(),
                ))
            }
            Self::ATTR_BUFFER_TIMESTAMP => {
                // Buffer timestamp is set automatically
                Err(DlmsError::AccessDenied(
                    "Attribute 3 (buffer_timestamp) is read-only".to_string(),
                ))
            }
            Self::ATTR_CAPTURE_OBJECTS => {
                // Parse and set capture objects
                match value {
                    DataObject::Array(entries) => {
                        let mut objects = Vec::new();
                        for entry in entries {
                            if let DataObject::Structure(fields) = entry {
                                if fields.len() >= 3 {
                                    let class_id = match &fields[0] {
                                        DataObject::Unsigned16(id) => *id,
                                        _ => {
                                            return Err(DlmsError::InvalidData(
                                                "Expected class_id as Unsigned16".to_string(),
                                            ))
                                        }
                                    };
                                    let logical_name = match &fields[1] {
                                        DataObject::OctetString(bytes) if bytes.len() == 6 => {
                                            ObisCode::from_bytes(bytes)?
                                        }
                                        _ => {
                                            return Err(DlmsError::InvalidData(
                                                "Expected logical_name as 6-byte octet string".to_string(),
                                            ))
                                        }
                                    };
                                    let version = match &fields[2] {
                                        DataObject::Unsigned8(v) => *v,
                                        _ => {
                                            return Err(DlmsError::InvalidData(
                                                "Expected version as Unsigned8".to_string(),
                                            ))
                                        }
                                    };
                                    objects.push(CosemObjectDescriptor::new(class_id, logical_name, version));
                                }
                            }
                        }
                        self.set_capture_objects(objects).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Array for capture_objects".to_string(),
                    )),
                }
            }
            Self::ATTR_CAPTURE_PERIOD => {
                if let DataObject::Unsigned32(period) = value {
                    self.set_capture_period(period).await;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned32 for capture_period".to_string(),
                    ))
                }
            }
            Self::ATTR_SORT_METHOD => {
                if let DataObject::Unsigned8(method) = value {
                    if let Some(sort_method) = ProfileSortMethod::from_u8(method) {
                        self.set_sort_method(sort_method).await;
                        Ok(())
                    } else {
                        Err(DlmsError::InvalidData(
                            "Invalid sort method value".to_string(),
                        ))
                    }
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for sort_method".to_string(),
                    ))
                }
            }
            Self::ATTR_ENTRIES_IN_USE => {
                Err(DlmsError::AccessDenied(
                    "Attribute 9 (entries_in_use) is read-only".to_string(),
                ))
            }
            Self::ATTR_PROFILE_BUFFER_STATUS => {
                Err(DlmsError::AccessDenied(
                    "Attribute 10 (profile_buffer_status) is read-only".to_string(),
                ))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Profile Generic has no attribute {}",
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
            Self::METHOD_RESET => {
                self.reset().await?;
                Ok(None)
            }
            Self::METHOD_CAPTURE => {
                // Capture with default values (empty for this implementation)
                self.capture(Vec::new()).await?;
                Ok(None)
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Profile Generic has no method {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_profile_generic_class_id() {
        let profile = ProfileGeneric::with_default_obis(100);
        assert_eq!(profile.class_id(), 7);
    }

    #[tokio::test]
    async fn test_profile_generic_obis_code() {
        let profile = ProfileGeneric::with_default_obis(100);
        assert_eq!(profile.obis_code(), ProfileGeneric::default_obis());
    }

    #[tokio::test]
    async fn test_profile_generic_get_logical_name() {
        let profile = ProfileGeneric::with_default_obis(100);
        let result = profile.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_profile_generic_capture_period() {
        let profile = ProfileGeneric::with_default_obis(100);
        assert_eq!(profile.capture_period().await, 900); // 15 minutes

        profile.set_attribute(5, DataObject::Unsigned32(3600), None).await.unwrap();
        assert_eq!(profile.capture_period().await, 3600); // 1 hour
    }

    #[tokio::test]
    async fn test_profile_generic_sort_method() {
        let profile = ProfileGeneric::with_default_obis(100);
        assert_eq!(profile.sort_method().await, ProfileSortMethod::Fifo);

        profile.set_attribute(6, DataObject::Unsigned8(1), None).await.unwrap();
        assert_eq!(profile.sort_method().await, ProfileSortMethod::Lifo);
    }

    #[tokio::test]
    async fn test_profile_generic_entries_in_use() {
        let profile = ProfileGeneric::with_default_obis(10);
        assert_eq!(profile.entries_in_use().await, 0);

        let timestamp = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        profile.capture_with_timestamp(timestamp, vec![DataObject::Unsigned32(100)]).await.unwrap();

        assert_eq!(profile.entries_in_use().await, 1);
    }

    #[tokio::test]
    async fn test_profile_generic_reset() {
        let profile = ProfileGeneric::with_default_obis(10);
        let timestamp = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        profile.capture_with_timestamp(timestamp, vec![DataObject::Unsigned32(100)]).await.unwrap();

        assert_eq!(profile.entries_in_use().await, 1);

        profile.invoke_method(1, None, None).await.unwrap();

        assert_eq!(profile.entries_in_use().await, 0);
        assert!(profile.buffer_timestamp().await.is_none());
    }

    #[tokio::test]
    async fn test_profile_generic_method_capture() {
        let profile = ProfileGeneric::with_default_obis(10);

        profile.invoke_method(2, None, None).await.unwrap();

        assert_eq!(profile.entries_in_use().await, 1);
    }

    #[tokio::test]
    async fn test_profile_generic_buffer_full() {
        let profile = ProfileGeneric::with_default_obis(3); // Small buffer
        let timestamp = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();

        // Fill the buffer
        for _ in 0..5 {
            profile.capture_with_timestamp(timestamp.clone(), vec![DataObject::Unsigned32(0)]).await.unwrap();
        }

        // With FIFO, buffer should have max 3 entries
        assert_eq!(profile.entries_in_use().await, 3);

        // Buffer full flag should be set
        assert!(profile.buffer_status().await & ProfileBufferStatus::BufferFull as u8 != 0);
    }

    #[tokio::test]
    async fn test_profile_generic_invalid_attribute() {
        let profile = ProfileGeneric::with_default_obis(100);
        let result = profile.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_profile_generic_buffer_encoding() {
        let profile = ProfileGeneric::with_default_obis(10);
        let timestamp = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();
        profile.capture_with_timestamp(timestamp.clone(), vec![DataObject::Unsigned32(100)]).await.unwrap();

        let result = profile.get_attribute(2, None).await.unwrap();

        match result {
            DataObject::Array(entries) => {
                assert_eq!(entries.len(), 1);
                match &entries[0] {
                    DataObject::Structure(fields) => {
                        assert!(!fields.is_empty());
                    }
                    _ => panic!("Expected Structure"),
                }
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_profile_generic_capture_objects() {
        let profile = ProfileGeneric::with_default_obis(100);
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let descriptor = CosemObjectDescriptor::new(3, obis, 0);

        profile.add_capture_object(descriptor).await;

        let result = profile.get_attribute(4, None).await.unwrap();

        match result {
            DataObject::Array(entries) => {
                assert_eq!(entries.len(), 1);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[tokio::test]
    async fn test_profile_generic_set_capture_objects() {
        let profile = ProfileGeneric::with_default_obis(100);

        // Create capture object array
        let mut obj_fields = Vec::new();
        obj_fields.push(DataObject::Unsigned16(3)); // Register
        obj_fields.push(DataObject::OctetString(vec![0x01, 0x01, 0x01, 0x08, 0x00, 0xFF]));
        obj_fields.push(DataObject::Unsigned8(0));

        let value = DataObject::Array(vec![DataObject::Structure(obj_fields)]);

        profile.set_attribute(4, value, None).await.unwrap();

        let objects = profile.capture_objects().await;
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].class_id, 3);
    }

    #[tokio::test]
    async fn test_profile_generic_status_flags() {
        let profile = ProfileGeneric::with_default_obis(10);

        // Initially no flags
        assert_eq!(profile.buffer_status().await, 0);

        // Set capture active flag
        profile.set_buffer_status(ProfileBufferStatus::CaptureActive as u8).await;

        let result = profile.get_attribute(10, None).await.unwrap();
        match result {
            DataObject::Unsigned8(status) => {
                assert_eq!(status, ProfileBufferStatus::CaptureActive as u8);
            }
            _ => panic!("Expected Unsigned8"),
        }
    }

    #[tokio::test]
    async fn test_profile_generic_lifo_behavior() {
        let profile = ProfileGeneric::new(ProfileGeneric::default_obis(), 10, 0, ProfileSortMethod::Lifo);
        let timestamp = CosemDateTime::new(2024, 6, 15, 12, 0, 0, 0, &[]).unwrap();

        // Add entries
        for i in 0..3 {
            profile.capture_with_timestamp(timestamp.clone(), vec![DataObject::Unsigned32(i)]).await.unwrap();
        }

        let buffer = profile.buffer().await;
        // LIFO: last entry should be first
        match &buffer[0].values[0] {
            DataObject::Unsigned32(v) => {
                // With LIFO, the most recently added is at the front
                assert_eq!(*v, 2);
            }
            _ => panic!("Expected Unsigned32"),
        }
    }
}
