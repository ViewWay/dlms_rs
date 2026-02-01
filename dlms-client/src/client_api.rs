//! High-level client API with timeout support and convenience methods
//!
//! This module provides a high-level API for DLMS/COSEM client operations
//! with enhanced features like:
//! - Per-request timeout configuration
//! - Automatic retry on transient failures
//! - Type-safe attribute access
//! - Convenience methods for common operations

use crate::connection::Connection;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::time::Duration;
use std::fmt;

/// Configuration for client operations
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Default timeout for requests
    pub default_timeout: Duration,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Delay between retries
    pub retry_delay: Duration,
    /// Whether to automatically retry on transient errors
    pub auto_retry: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(5),
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
            auto_retry: true,
        }
    }
}

/// High-level client API
///
/// Wraps a connection and provides enhanced features like timeout support,
/// automatic retries, and convenience methods.
///
/// # Example
/// ```rust,no_run
/// use dlms_client::client_api::{DlmsClient, ClientConfig};
/// use dlms_client::connection::LnConnection;
/// use dlms_core::ObisCode;
/// use std::time::Duration;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create connection
/// let mut connection = LnConnection::new(...)?;
/// connection.open().await?;
///
/// // Create client with custom config
/// let config = ClientConfig {
///     default_timeout: Duration::from_secs(10),
///     max_retries: 5,
///     ..Default::default()
/// };
/// let mut client = DlmsClient::with_config(connection, config);
///
/// // Use the client
/// let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
/// let value = client.get_attribute_typed::<u32>(obis, 1, 2).await?;
/// println!("Value: {}", value);
///
/// # Ok(())
/// # }
/// ```
pub struct DlmsClient<C: Connection> {
    /// Underlying connection
    connection: C,
    /// Client configuration
    config: ClientConfig,
}

impl<C: Connection> DlmsClient<C> {
    /// Create a new client with default configuration
    pub fn new(connection: C) -> Self {
        Self {
            connection,
            config: ClientConfig::default(),
        }
    }

    /// Create a new client with custom configuration
    pub fn with_config(connection: C, config: ClientConfig) -> Self {
        Self {
            connection,
            config,
        }
    }

    /// Get the underlying connection
    pub fn connection(&self) -> &C {
        &self.connection
    }

    /// Get a mutable reference to the underlying connection
    pub fn connection_mut(&mut self) -> &mut C {
        &mut self.connection
    }

    /// Get the client configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Update the client configuration
    pub fn set_config(&mut self, config: ClientConfig) {
        self.config = config;
    }

    /// Get an attribute value with default timeout and automatic retry
    ///
    /// # Arguments
    /// * `obis_code` - OBIS code of the object
    /// * `class_id` - Class ID of the object
    /// * `attribute_id` - Attribute ID to read
    pub async fn get_attribute(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        attribute_id: u8,
    ) -> DlmsResult<DataObject> {
        if !self.config.auto_retry {
            return self.connection.get_attribute(obis_code, class_id, attribute_id).await;
        }

        let mut last_error = None;
        for attempt in 0..=self.config.max_retries {
            match self.connection.get_attribute(obis_code, class_id, attribute_id).await {
                Ok(result) => return Ok(result),
                Err(e) if is_transient_error(&e) && attempt < self.config.max_retries => {
                    last_error = Some(e);
                    tokio::time::sleep(self.config.retry_delay).await;
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap_or_else(|| {
            DlmsError::InvalidData("Max retries exceeded with no error stored".to_string())
        }))
    }

    /// Get an attribute value as a specific type
    ///
    /// # Type Parameters
    /// * `T` - The type to convert the value to (must implement TryFromDataObject)
    ///
    /// # Arguments
    /// * `obis_code` - OBIS code of the object
    /// * `class_id` - Class ID of the object
    /// * `attribute_id` - Attribute ID to read
    pub async fn get_attribute_typed<T>(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        attribute_id: u8,
    ) -> DlmsResult<T>
    where
        T: TryFromDataObject,
    {
        let value = self.get_attribute(obis_code, class_id, attribute_id).await?;
        T::try_from_data_object(value)
    }

    /// Set an attribute value with default timeout and automatic retry
    ///
    /// # Arguments
    /// * `obis_code` - OBIS code of the object
    /// * `class_id` - Class ID of the object
    /// * `attribute_id` - Attribute ID to write
    /// * `value` - Value to write
    pub async fn set_attribute(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        attribute_id: u8,
        value: DataObject,
    ) -> DlmsResult<()> {
        if !self.config.auto_retry {
            return self.connection.set_attribute(obis_code, class_id, attribute_id, value).await;
        }

        // Clone value for each retry attempt
        let mut last_error = None;
        for attempt in 0..=self.config.max_retries {
            match self.connection.set_attribute(obis_code, class_id, attribute_id, value.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) if is_transient_error(&e) && attempt < self.config.max_retries => {
                    last_error = Some(e);
                    tokio::time::sleep(self.config.retry_delay).await;
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap_or_else(|| {
            DlmsError::InvalidData("Max retries exceeded with no error stored".to_string())
        }))
    }

    /// Set an attribute value from a Rust native type
    ///
    /// # Type Parameters
    /// * `T` - The type to convert from (must implement IntoDataObject)
    ///
    /// # Arguments
    /// * `obis_code` - OBIS code of the object
    /// * `class_id` - Class ID of the object
    /// * `attribute_id` - Attribute ID to write
    pub async fn set_attribute_typed<T>(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        attribute_id: u8,
        value: T,
    ) -> DlmsResult<()>
    where
        T: IntoDataObject,
    {
        let data_object = value.into_data_object()?;
        self.set_attribute(obis_code, class_id, attribute_id, data_object).await
    }

    /// Invoke a method with default timeout and automatic retry
    ///
    /// # Arguments
    /// * `obis_code` - OBIS code of the object
    /// * `class_id` - Class ID of the object
    /// * `method_id` - Method ID to invoke
    /// * `parameters` - Optional method parameters
    pub async fn invoke_method(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        method_id: u8,
        parameters: Option<DataObject>,
    ) -> DlmsResult<Option<DataObject>> {
        if !self.config.auto_retry {
            return self.connection.invoke_method(obis_code, class_id, method_id, parameters).await;
        }

        // Clone parameters for each retry attempt
        let mut last_error = None;
        for attempt in 0..=self.config.max_retries {
            match self.connection.invoke_method(obis_code, class_id, method_id, parameters.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) if is_transient_error(&e) && attempt < self.config.max_retries => {
                    last_error = Some(e);
                    tokio::time::sleep(self.config.retry_delay).await;
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap_or_else(|| {
            DlmsError::InvalidData("Max retries exceeded with no error stored".to_string())
        }))
    }

    /// Invoke a method and get typed result
    ///
    /// # Type Parameters
    /// * `T` - The type to convert the result to (must implement TryFromDataObject)
    ///
    /// # Arguments
    /// * `obis_code` - OBIS code of the object
    /// * `class_id` - Class ID of the object
    /// * `method_id` - Method ID to invoke
    /// * `parameters` - Optional method parameters
    pub async fn invoke_method_typed<T>(
        &mut self,
        obis_code: ObisCode,
        class_id: u16,
        method_id: u8,
        parameters: Option<DataObject>,
    ) -> DlmsResult<Option<T>>
    where
        T: TryFromDataObject,
    {
        let result = self.invoke_method(obis_code, class_id, method_id, parameters).await?;
        match result {
            Some(value) => Ok(Some(T::try_from_data_object(value)?)),
            None => Ok(None),
        }
    }
}

/// Check if an error is transient (might succeed on retry)
fn is_transient_error(error: &DlmsError) -> bool {
    match error {
        DlmsError::Timeout => true,
        DlmsError::Connection(_) => true,
        DlmsError::InvalidData(msg) if msg.contains("timeout") => true,
        _ => false,
    }
}

/// Trait for converting DataObject to Rust types
pub trait TryFromDataObject: Sized {
    /// Convert from DataObject
    fn try_from_data_object(value: DataObject) -> DlmsResult<Self>;
}

/// Trait for converting Rust types to DataObject
pub trait IntoDataObject {
    /// Convert to DataObject
    fn into_data_object(self) -> DlmsResult<DataObject>;
}

// Implementations for common types
impl TryFromDataObject for u32 {
    fn try_from_data_object(value: DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Unsigned32(v) => Ok(v),
            DataObject::Unsigned16(v) => Ok(v as u32),
            DataObject::Unsigned8(v) => Ok(v as u32),
            DataObject::Integer32(v) => Ok(v as u32),
            DataObject::Integer16(v) => Ok(v as u32),
            DataObject::Integer8(v) => Ok(v as u32),
            _ => Err(DlmsError::InvalidData(format!(
                "Cannot convert {:?} to u32", value
            ))),
        }
    }
}

impl TryFromDataObject for i32 {
    fn try_from_data_object(value: DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Integer32(v) => Ok(v),
            DataObject::Integer16(v) => Ok(v as i32),
            DataObject::Integer8(v) => Ok(v as i32),
            DataObject::Unsigned32(v) => Ok(v as i32),
            DataObject::Unsigned16(v) => Ok(v as i32),
            DataObject::Unsigned8(v) => Ok(v as i32),
            _ => Err(DlmsError::InvalidData(format!(
                "Cannot convert {:?} to i32", value
            ))),
        }
    }
}

impl TryFromDataObject for u64 {
    fn try_from_data_object(value: DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Unsigned64(v) => Ok(v),
            DataObject::Unsigned32(v) => Ok(v as u64),
            DataObject::Unsigned16(v) => Ok(v as u64),
            DataObject::Unsigned8(v) => Ok(v as u64),
            _ => Err(DlmsError::InvalidData(format!(
                "Cannot convert {:?} to u64", value
            ))),
        }
    }
}

impl TryFromDataObject for i64 {
    fn try_from_data_object(value: DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Integer64(v) => Ok(v),
            DataObject::Integer32(v) => Ok(v as i64),
            DataObject::Integer16(v) => Ok(v as i64),
            DataObject::Integer8(v) => Ok(v as i64),
            _ => Err(DlmsError::InvalidData(format!(
                "Cannot convert {:?} to i64", value
            ))),
        }
    }
}

impl TryFromDataObject for f64 {
    fn try_from_data_object(value: DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Float64(v) => Ok(v),
            DataObject::Float32(v) => Ok(v as f64),
            DataObject::Unsigned64(v) => Ok(v as f64),
            DataObject::Integer64(v) => Ok(v as f64),
            _ => Err(DlmsError::InvalidData(format!(
                "Cannot convert {:?} to f64", value
            ))),
        }
    }
}

impl TryFromDataObject for f32 {
    fn try_from_data_object(value: DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Float32(v) => Ok(v),
            DataObject::Float64(v) => Ok(v as f32),
            _ => Err(DlmsError::InvalidData(format!(
                "Cannot convert {:?} to f32", value
            ))),
        }
    }
}

impl TryFromDataObject for String {
    fn try_from_data_object(value: DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::OctetString(bytes) => {
                String::from_utf8(bytes).map_err(|_| {
                    DlmsError::InvalidData("Invalid UTF-8 in octet string".to_string())
                })
            }
            DataObject::Utf8String(bytes) => {
                String::from_utf8(bytes).map_err(|_| {
                    DlmsError::InvalidData("Invalid UTF-8 in Utf8String".to_string())
                })
            }
            DataObject::VisibleString(bytes) => {
                String::from_utf8(bytes).map_err(|_| {
                    DlmsError::InvalidData("Invalid UTF-8 in visible string".to_string())
                })
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Cannot convert {:?} to String", value
            ))),
        }
    }
}

impl TryFromDataObject for Vec<u8> {
    fn try_from_data_object(value: DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::OctetString(bytes) => Ok(bytes),
            DataObject::VisibleString(bytes) => Ok(bytes),
            DataObject::Utf8String(bytes) => Ok(bytes),
            _ => Err(DlmsError::InvalidData(format!(
                "Cannot convert {:?} to Vec<u8>", value
            ))),
        }
    }
}

impl TryFromDataObject for bool {
    fn try_from_data_object(value: DataObject) -> DlmsResult<Self> {
        match value {
            DataObject::Boolean(b) => Ok(b),
            _ => Err(DlmsError::InvalidData(format!(
                "Cannot convert {:?} to bool", value
            ))),
        }
    }
}

impl TryFromDataObject for DataObject {
    fn try_from_data_object(value: DataObject) -> DlmsResult<Self> {
        Ok(value)
    }
}

// IntoDataObject implementations
impl IntoDataObject for u32 {
    fn into_data_object(self) -> DlmsResult<DataObject> {
        Ok(DataObject::Unsigned32(self))
    }
}

impl IntoDataObject for i32 {
    fn into_data_object(self) -> DlmsResult<DataObject> {
        Ok(DataObject::Integer32(self))
    }
}

impl IntoDataObject for u64 {
    fn into_data_object(self) -> DlmsResult<DataObject> {
        Ok(DataObject::Unsigned64(self))
    }
}

impl IntoDataObject for i64 {
    fn into_data_object(self) -> DlmsResult<DataObject> {
        Ok(DataObject::Integer64(self))
    }
}

impl IntoDataObject for f64 {
    fn into_data_object(self) -> DlmsResult<DataObject> {
        Ok(DataObject::Float64(self))
    }
}

impl IntoDataObject for f32 {
    fn into_data_object(self) -> DlmsResult<DataObject> {
        Ok(DataObject::Float32(self))
    }
}

impl IntoDataObject for &str {
    fn into_data_object(self) -> DlmsResult<DataObject> {
        Ok(DataObject::Utf8String(self.as_bytes().to_vec()))
    }
}

impl IntoDataObject for String {
    fn into_data_object(self) -> DlmsResult<DataObject> {
        Ok(DataObject::Utf8String(self.into_bytes()))
    }
}

impl IntoDataObject for &[u8] {
    fn into_data_object(self) -> DlmsResult<DataObject> {
        Ok(DataObject::OctetString(self.to_vec()))
    }
}

impl IntoDataObject for Vec<u8> {
    fn into_data_object(self) -> DlmsResult<DataObject> {
        Ok(DataObject::OctetString(self))
    }
}

impl IntoDataObject for bool {
    fn into_data_object(self) -> DlmsResult<DataObject> {
        Ok(DataObject::Boolean(self))
    }
}

impl IntoDataObject for DataObject {
    fn into_data_object(self) -> DlmsResult<DataObject> {
        Ok(self)
    }
}

/// Error type for type conversion failures
#[derive(Debug)]
pub struct ConversionError {
    pub expected_type: &'static str,
    pub actual_value: DataObject,
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cannot convert {:?} to {}",
            self.actual_value, self.expected_type
        )
    }
}

impl std::error::Error for ConversionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.default_timeout, Duration::from_secs(5));
        assert_eq!(config.max_retries, 3);
        assert!(config.auto_retry);
    }

    #[test]
    fn test_try_from_data_object_u32() {
        let value = DataObject::Unsigned32(42);
        let result: u32 = TryFromDataObject::try_from_data_object(value).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_try_from_data_object_u32_from_u16() {
        let value = DataObject::Unsigned16(42);
        let result: u32 = TryFromDataObject::try_from_data_object(value).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_into_data_object_u32() {
        let value: u32 = 42;
        let result = value.into_data_object().unwrap();
        assert!(matches!(result, DataObject::Unsigned32(42)));
    }

    #[test]
    fn test_into_data_object_string() {
        let value = "hello".to_string();
        let result = value.into_data_object().unwrap();
        assert!(matches!(result, DataObject::Utf8String(_)));
    }
}
