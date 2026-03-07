//! Register interface class (Class ID: 3)
//!
//! The Register interface class represents a single register value with
//! scaling factor and unit information.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: value - The register value (Integer, Long, DoubleLong, etc.)
//! - Attribute 3: scaler_unit - ScalerUnit structure (scaler and unit)
//! - Attribute 4: status - Optional status value (Unsigned8)
//!
//! # Methods
//!
//! None
//!
//! # Usage
//!
//! ```rust,no_run
//! use dlms_interface::{Register, ScalerUnit};
//! use dlms_core::{ObisCode, DataObject};
//!
//! // Create a Register for energy (Wh)
//! let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
//! let value = DataObject::Unsigned32(12345);
//! let scaler_unit = ScalerUnit::new(0, 0x1E); // Wh
//! let register = Register::new(obis, value, scaler_unit, None);
//!
//! // Get the value
//! let current_value = register.value().await;
//! ```

use crate::scaler_unit::ScalerUnit;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use dlms_application::pdu::SelectiveAccessDescriptor;
use crate::CosemObject;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tokio::sync::Mutex;

/// Callback function type for value change notifications
pub type RegisterChangeCallback = Arc<dyn Fn(&DataObject) + Send + Sync>;

/// Register interface class (Class ID: 3)
///
/// Represents a single register value with scaling factor and unit information.
/// This is one of the most commonly used interface classes in DLMS/COSEM.
pub struct Register {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,
    /// The register value (Integer, Long, DoubleLong, etc.)
    value: Arc<RwLock<DataObject>>,
    /// Scaler and unit information
    scaler_unit: Arc<RwLock<ScalerUnit>>,
    /// Optional status value
    status: Arc<RwLock<Option<u8>>>,
    /// Change notification callbacks
    change_callbacks: Arc<Mutex<HashMap<String, RegisterChangeCallback>>>,
    /// Enable/disable change notifications
    notifications_enabled: Arc<RwLock<bool>>,
}

impl std::fmt::Debug for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Register")
            .field("logical_name", &self.logical_name)
            .field("value", &"<RwLock<DataObject>>")
            .field("scaler_unit", &"<RwLock<ScalerUnit>>")
            .field("status", &"<RwLock<Option<u8>>>")
            .field("change_callbacks", &"<callbacks>")
            .field("notifications_enabled", &"<RwLock<bool>>")
            .finish()
    }
}

impl Clone for Register {
    fn clone(&self) -> Self {
        Self {
            logical_name: self.logical_name,
            value: self.value.clone(),
            scaler_unit: self.scaler_unit.clone(),
            status: self.status.clone(),
            change_callbacks: self.change_callbacks.clone(),
            notifications_enabled: self.notifications_enabled.clone(),
        }
    }
}

impl Register {
    /// Create a new Register object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `value` - Initial register value (must be a numeric type)
    /// * `scaler_unit` - ScalerUnit structure
    /// * `status` - Optional initial status value
    ///
    /// # Returns
    /// A new Register instance
    pub fn new(
        logical_name: ObisCode,
        value: DataObject,
        scaler_unit: ScalerUnit,
        status: Option<u8>,
    ) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(value)),
            scaler_unit: Arc::new(RwLock::new(scaler_unit)),
            status: Arc::new(RwLock::new(status)),
            change_callbacks: Arc::new(Mutex::new(HashMap::new())),
            notifications_enabled: Arc::new(RwLock::new(true)),
        }
    }

    /// Get the current value
    ///
    /// # Returns
    /// A copy of the current register value
    pub async fn value(&self) -> DataObject {
        self.value.read().await.clone()
    }

    /// Set the value
    ///
    /// # Arguments
    /// * `new_value` - New register value to set
    ///
    /// This will trigger change notification callbacks if the value differs.
    pub async fn set_value(&self, new_value: DataObject) {
        let old_value = self.value.read().await.clone();
        let value_changed = old_value != new_value;

        *self.value.write().await = new_value.clone();

        // Trigger change notifications if enabled and value changed
        if value_changed && *self.notifications_enabled.read().await {
            self.notify_change(&new_value).await;
        }
    }

    /// Get the scaler unit
    pub async fn scaler_unit(&self) -> ScalerUnit {
        *self.scaler_unit.read().await
    }

    /// Set the scaler unit
    pub async fn set_scaler_unit(&self, scaler_unit: ScalerUnit) {
        *self.scaler_unit.write().await = scaler_unit;
    }

    /// Get the status
    ///
    /// # Returns
    /// Optional status value
    pub async fn status(&self) -> Option<u8> {
        *self.status.read().await
    }

    /// Set the status
    ///
    /// # Arguments
    /// * `new_status` - New status value (None to clear)
    pub async fn set_status(&self, new_status: Option<u8>) {
        *self.status.write().await = new_status;
    }

    /// Get the logical name (OBIS code)
    pub fn logical_name(&self) -> ObisCode {
        self.logical_name
    }

    /// Get the scaled value as f64
    ///
    /// This applies the scaling factor to the register value.
    ///
    /// # Returns
    /// The scaled value, or error if value is not numeric
    pub async fn scaled_value(&self) -> DlmsResult<f64> {
        let value = self.value().await;
        let scaler_unit = self.scaler_unit().await;
        let numeric_value = match value {
            DataObject::Integer8(v) => v as f64,
            DataObject::Integer16(v) => v as f64,
            DataObject::Integer32(v) => v as f64,
            DataObject::Integer64(v) => v as f64,
            DataObject::Unsigned8(v) => v as f64,
            DataObject::Unsigned16(v) => v as f64,
            DataObject::Unsigned32(v) => v as f64,
            DataObject::Unsigned64(v) => v as f64,
            DataObject::Float32(v) => v as f64,
            DataObject::Float64(v) => v,
            _ => {
                return Err(DlmsError::InvalidData(
                    "Register value must be numeric".to_string(),
                ));
            }
        };
        Ok(scaler_unit.scale_value(numeric_value))
    }

    /// Register a callback for value change notifications
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this callback
    /// * `callback` - Function to call when value changes
    ///
    /// # Returns
    /// Ok(()) if registered, error if ID already exists
    pub async fn register_change_callback(&self, id: String, callback: RegisterChangeCallback) -> DlmsResult<()> {
        let mut callbacks = self.change_callbacks.lock().await;
        if callbacks.contains_key(&id) {
            return Err(DlmsError::InvalidData(format!(
                "Callback with id '{}' already exists",
                id
            )));
        }
        callbacks.insert(id, callback);
        Ok(())
    }

    /// Unregister a change callback
    ///
    /// # Arguments
    /// * `id` - Identifier of the callback to remove
    ///
    /// # Returns
    /// Ok(()) if removed, error if ID not found
    pub async fn unregister_change_callback(&self, id: &str) -> DlmsResult<()> {
        let mut callbacks = self.change_callbacks.lock().await;
        callbacks.remove(id).ok_or_else(|| {
            DlmsError::InvalidData(format!("Callback with id '{}' not found", id))
        })?;
        Ok(())
    }

    /// Enable or disable change notifications
    pub async fn set_notifications_enabled(&self, enabled: bool) {
        *self.notifications_enabled.write().await = enabled;
    }

    /// Check if change notifications are enabled
    pub async fn notifications_enabled(&self) -> bool {
        *self.notifications_enabled.read().await
    }

    /// Get the number of registered callbacks
    pub async fn callback_count(&self) -> usize {
        self.change_callbacks.lock().await.len()
    }

    /// Notify all registered callbacks of a value change
    async fn notify_change(&self, new_value: &DataObject) {
        let callbacks = self.change_callbacks.lock().await;
        for callback in callbacks.values() {
            callback(new_value);
        }
    }

    /// Add a value to the current value (for numeric registers)
    ///
    /// # Arguments
    /// * `delta` - Value to add (can be negative for subtraction)
    ///
    /// # Returns
    /// Ok(()) if successful, error if value is not numeric
    pub async fn add(&self, delta: i64) -> DlmsResult<()> {
        let current = self.value().await;
        let new_value = match current {
            DataObject::Integer8(v) => DataObject::Integer8((v as i64 + delta) as i8),
            DataObject::Integer16(v) => DataObject::Integer16((v as i64 + delta) as i16),
            DataObject::Integer32(v) => DataObject::Integer32((v as i64 + delta) as i32),
            DataObject::Integer64(v) => DataObject::Integer64(v + delta),
            DataObject::Unsigned8(v) => {
                if delta >= 0 {
                    DataObject::Unsigned8((v as i64 + delta) as u8)
                } else {
                    return Err(DlmsError::InvalidData(
                        "Cannot subtract from unsigned value".to_string(),
                    ));
                }
            }
            DataObject::Unsigned16(v) => {
                if delta >= 0 {
                    DataObject::Unsigned16((v as i64 + delta) as u16)
                } else {
                    return Err(DlmsError::InvalidData(
                        "Cannot subtract from unsigned value".to_string(),
                    ));
                }
            }
            DataObject::Unsigned32(v) => {
                if delta >= 0 {
                    DataObject::Unsigned32((v as i64 + delta) as u32)
                } else {
                    return Err(DlmsError::InvalidData(
                        "Cannot subtract from unsigned value".to_string(),
                    ));
                }
            }
            DataObject::Unsigned64(v) => {
                if delta >= 0 {
                    DataObject::Unsigned64((v as i64 + delta) as u64)
                } else {
                    return Err(DlmsError::InvalidData(
                        "Cannot subtract from unsigned value".to_string(),
                    ));
                }
            }
            DataObject::Float32(v) => DataObject::Float32(v + delta as f32),
            DataObject::Float64(v) => DataObject::Float64(v + delta as f64),
            _ => {
                return Err(DlmsError::InvalidData(
                    "Cannot add to non-numeric value".to_string(),
                ));
            }
        };
        self.set_value(new_value).await;
        Ok(())
    }

    /// Multiply the current value by a factor
    ///
    /// # Arguments
    /// * `factor` - Multiplication factor
    ///
    /// # Returns
    /// Ok(()) if successful, error if value is not numeric
    pub async fn multiply(&self, factor: f64) -> DlmsResult<()> {
        let current = self.value().await;
        let new_value = match current {
            DataObject::Integer8(v) => DataObject::Integer8((v as f64 * factor) as i8),
            DataObject::Integer16(v) => DataObject::Integer16((v as f64 * factor) as i16),
            DataObject::Integer32(v) => DataObject::Integer32((v as f64 * factor) as i32),
            DataObject::Integer64(v) => DataObject::Integer64((v as f64 * factor) as i64),
            DataObject::Unsigned8(v) => DataObject::Unsigned8((v as f64 * factor) as u8),
            DataObject::Unsigned16(v) => DataObject::Unsigned16((v as f64 * factor) as u16),
            DataObject::Unsigned32(v) => DataObject::Unsigned32((v as f64 * factor) as u32),
            DataObject::Unsigned64(v) => DataObject::Unsigned64((v as f64 * factor) as u64),
            DataObject::Float32(v) => DataObject::Float32(v * factor as f32),
            DataObject::Float64(v) => DataObject::Float64(v * factor),
            _ => {
                return Err(DlmsError::InvalidData(
                    "Cannot multiply non-numeric value".to_string(),
                ));
            }
        };
        self.set_value(new_value).await;
        Ok(())
    }

    /// Reset the value to zero
    pub async fn reset(&self) -> DlmsResult<()> {
        let current = self.value().await;
        let zero_value = match current {
            DataObject::Integer8(_) => DataObject::Integer8(0),
            DataObject::Integer16(_) => DataObject::Integer16(0),
            DataObject::Integer32(_) => DataObject::Integer32(0),
            DataObject::Integer64(_) => DataObject::Integer64(0),
            DataObject::Unsigned8(_) => DataObject::Unsigned8(0),
            DataObject::Unsigned16(_) => DataObject::Unsigned16(0),
            DataObject::Unsigned32(_) => DataObject::Unsigned32(0),
            DataObject::Unsigned64(_) => DataObject::Unsigned64(0),
            DataObject::Float32(_) => DataObject::Float32(0.0),
            DataObject::Float64(_) => DataObject::Float64(0.0),
            _ => {
                return Err(DlmsError::InvalidData(
                    "Cannot reset non-numeric value".to_string(),
                ));
            }
        };
        self.set_value(zero_value).await;
        Ok(())
    }

    /// Get the data type of the current value
    pub async fn value_type(&self) -> &'static str {
        match self.value().await {
            DataObject::Integer8(_) => "Integer8",
            DataObject::Integer16(_) => "Integer16",
            DataObject::Integer32(_) => "Integer32",
            DataObject::Integer64(_) => "Integer64",
            DataObject::Unsigned8(_) => "Unsigned8",
            DataObject::Unsigned16(_) => "Unsigned16",
            DataObject::Unsigned32(_) => "Unsigned32",
            DataObject::Unsigned64(_) => "Unsigned64",
            DataObject::Float32(_) => "Float32",
            DataObject::Float64(_) => "Float64",
            _ => "Other",
        }
    }

    /// Check if the current value is within a specified range
    ///
    /// # Arguments
    /// * `min` - Minimum allowed value (inclusive)
    /// * `max` - Maximum allowed value (inclusive)
    ///
    /// # Returns
    /// true if value is within range, false otherwise
    pub async fn is_within_range(&self, min: f64, max: f64) -> DlmsResult<bool> {
        let scaled = self.scaled_value().await?;
        Ok(scaled >= min && scaled <= max)
    }
}

#[async_trait::async_trait]
impl CosemObject for Register {
    fn class_id(&self) -> u16 {
        3 // Register interface class ID
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
            1 => {
                // Attribute 1: logical_name (OBIS code)
                Ok(DataObject::OctetString(self.logical_name.to_bytes().to_vec()))
            }
            2 => {
                // Attribute 2: value
                Ok(self.value().await)
            }
            3 => {
                // Attribute 3: scaler_unit
                Ok(self.scaler_unit().await.to_data_object())
            }
            4 => {
                // Attribute 4: status (optional)
                match self.status().await {
                    Some(s) => Ok(DataObject::Unsigned8(s)),
                    None => Ok(DataObject::Null),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Register interface class has no attribute {}",
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
            1 => {
                // Attribute 1: logical_name is read-only
                Err(DlmsError::AccessDenied(
                    "Attribute 1 (logical_name) is read-only".to_string(),
                ))
            }
            2 => {
                // Attribute 2: value
                self.set_value(value).await;
                Ok(())
            }
            3 => {
                // Attribute 3: scaler_unit
                let scaler_unit = ScalerUnit::from_data_object(&value)?;
                self.set_scaler_unit(scaler_unit).await;
                Ok(())
            }
            4 => {
                // Attribute 4: status
                let status = match value {
                    DataObject::Null => None,
                    DataObject::Unsigned8(s) => Some(s),
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "Status must be Unsigned8 or Null".to_string(),
                        ));
                    }
                };
                self.set_status(status).await;
                Ok(())
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Register interface class has no attribute {}",
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
            "Register interface class has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_creation() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned32(12345);
        let scaler_unit = ScalerUnit::new(0, 0x1E); // Wh
        let register = Register::new(obis, value.clone(), scaler_unit, Some(0));

        assert_eq!(register.class_id(), 3);
        assert_eq!(register.obis_code(), obis);
        assert_eq!(register.value().await, value);
        assert_eq!(register.status().await, Some(0));
    }

    #[tokio::test]
    async fn test_register_get_attribute() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned32(12345);
        let scaler_unit = ScalerUnit::new(3, 0x1B); // kW
        let register = Register::new(obis, value.clone(), scaler_unit, Some(0));

        // Get attribute 1 (logical_name)
        let attr1 = register.get_attribute(1, None).await.unwrap();
        if let DataObject::OctetString(bytes) = attr1 {
            assert_eq!(bytes, obis.to_bytes());
        } else {
            panic!("Attribute 1 should be OctetString");
        }

        // Get attribute 2 (value)
        let attr2 = register.get_attribute(2, None).await.unwrap();
        assert_eq!(attr2, value);

        // Get attribute 3 (scaler_unit)
        let attr3 = register.get_attribute(3, None).await.unwrap();
        let decoded_scaler_unit = ScalerUnit::from_data_object(&attr3).unwrap();
        assert_eq!(decoded_scaler_unit, register.scaler_unit().await);

        // Get attribute 4 (status)
        let attr4 = register.get_attribute(4, None).await.unwrap();
        if let DataObject::Unsigned8(s) = attr4 {
            assert_eq!(s, 0);
        } else {
            panic!("Attribute 4 should be Unsigned8");
        }
    }

    #[tokio::test]
    async fn test_register_set_attribute() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let initial_value = DataObject::Unsigned32(12345);
        let scaler_unit = ScalerUnit::new(0, 0x1E);
        let register = Register::new(obis, initial_value, scaler_unit, None);

        // Set attribute 2 (value)
        let new_value = DataObject::Unsigned32(67890);
        register.set_attribute(2, new_value.clone(), None).await.unwrap();

        // Verify the value was updated
        let current_value = register.get_attribute(2, None).await.unwrap();
        assert_eq!(current_value, new_value);

        // Set attribute 4 (status)
        register
            .set_attribute(4, DataObject::Unsigned8(1), None)
            .await
            .unwrap();
        assert_eq!(register.status().await, Some(1));

        // Clear status
        register.set_attribute(4, DataObject::Null, None).await.unwrap();
        assert_eq!(register.status().await, None);
    }

    #[tokio::test]
    async fn test_register_scaled_value() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned32(12345);
        let scaler_unit = ScalerUnit::new(3, 0x1B); // kW (scale factor 3)
        let register = Register::new(obis, value, scaler_unit, None);

        let scaled = register.scaled_value().await.unwrap();
        // 12345 * 10^3 = 12345000
        assert!((scaled - 12345000.0).abs() < 0.001);
        
        // Test setting scaler_unit
        let new_scaler_unit = ScalerUnit::new(0, 0x1E);
        register.set_scaler_unit(new_scaler_unit).await;
        let scaled2 = register.scaled_value().await.unwrap();
        // 12345 * 10^0 = 12345
        assert!((scaled2 - 12345.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_register_invalid_attribute() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned32(12345);
        let scaler_unit = ScalerUnit::new(0, 0x1E);
        let register = Register::new(obis, value, scaler_unit, None);

        // Try to get non-existent attribute
        let result = register.get_attribute(99, None).await;
        assert!(result.is_err());

        // Try to set non-existent attribute
        let result = register.set_attribute(99, DataObject::Integer32(0), None).await;
        assert!(result.is_err());
    }

    // Tests for enhanced functionality

    #[tokio::test]
    async fn test_register_add() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned32(100);
        let scaler_unit = ScalerUnit::new(0, 0x1E);
        let register = Register::new(obis, value, scaler_unit, None);

        register.add(50).await.unwrap();
        let current = register.value().await;
        match current {
            DataObject::Unsigned32(v) => assert_eq!(v, 150),
            _ => panic!("Expected Unsigned32"),
        }
    }

    #[tokio::test]
    async fn test_register_multiply() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned32(100);
        let scaler_unit = ScalerUnit::new(0, 0x1E);
        let register = Register::new(obis, value, scaler_unit, None);

        register.multiply(2.0).await.unwrap();
        let current = register.value().await;
        match current {
            DataObject::Unsigned32(v) => assert_eq!(v, 200),
            _ => panic!("Expected Unsigned32"),
        }
    }

    #[tokio::test]
    async fn test_register_reset() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned32(12345);
        let scaler_unit = ScalerUnit::new(0, 0x1E);
        let register = Register::new(obis, value, scaler_unit, None);

        register.reset().await.unwrap();
        let current = register.value().await;
        match current {
            DataObject::Unsigned32(v) => assert_eq!(v, 0),
            _ => panic!("Expected Unsigned32"),
        }
    }

    #[tokio::test]
    async fn test_register_value_type() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned32(12345);
        let scaler_unit = ScalerUnit::new(0, 0x1E);
        let register = Register::new(obis, value, scaler_unit, None);

        assert_eq!(register.value_type().await, "Unsigned32");
    }

    #[tokio::test]
    async fn test_register_is_within_range() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned32(50);
        let scaler_unit = ScalerUnit::new(0, 0x1E);
        let register = Register::new(obis, value, scaler_unit, None);

        assert!(register.is_within_range(0.0, 100.0).await.unwrap());
        assert!(!register.is_within_range(100.0, 200.0).await.unwrap());
    }

    #[tokio::test]
    async fn test_register_change_callback() {
        use std::sync::atomic::{AtomicU32, Ordering};

        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned32(100);
        let scaler_unit = ScalerUnit::new(0, 0x1E);
        let register = Register::new(obis, value, scaler_unit, None);

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        register.register_change_callback(
            "test_callback".to_string(),
            Arc::new(move |_new_value| {
                call_count_clone.fetch_add(1, Ordering::SeqCst);
            }),
        ).await.unwrap();

        register.set_value(DataObject::Unsigned32(200)).await;
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        assert_eq!(register.callback_count().await, 1);

        // Unregister and verify no more calls
        register.unregister_change_callback("test_callback").await.unwrap();
        register.set_value(DataObject::Unsigned32(300)).await;
        assert_eq!(call_count.load(Ordering::SeqCst), 1); // Still 1, not 2
    }

    #[tokio::test]
    async fn test_register_notifications_disabled() {
        use std::sync::atomic::{AtomicU32, Ordering};

        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned32(100);
        let scaler_unit = ScalerUnit::new(0, 0x1E);
        let register = Register::new(obis, value, scaler_unit, None);

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        register.register_change_callback(
            "test_callback".to_string(),
            Arc::new(move |_new_value| {
                call_count_clone.fetch_add(1, Ordering::SeqCst);
            }),
        ).await.unwrap();

        // Disable notifications
        register.set_notifications_enabled(false).await;
        assert!(!register.notifications_enabled().await);

        register.set_value(DataObject::Unsigned32(200)).await;
        assert_eq!(call_count.load(Ordering::SeqCst), 0); // No call made

        // Re-enable
        register.set_notifications_enabled(true).await;
        register.set_value(DataObject::Unsigned32(300)).await;
        assert_eq!(call_count.load(Ordering::SeqCst), 1); // Call made
    }

    #[tokio::test]
    async fn test_register_no_duplicate_callback_id() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned32(100);
        let scaler_unit = ScalerUnit::new(0, 0x1E);
        let register = Register::new(obis, value, scaler_unit, None);

        // Use a function instead of empty closure so Fn implements for any lifetime (&DataObject)
        fn noop(_: &DataObject) {}
        let callback = Arc::new(noop) as RegisterChangeCallback;
        register.register_change_callback("test".to_string(), callback.clone()).await.unwrap();

        // Try to register with same ID
        let result = register.register_change_callback("test".to_string(), callback).await;
        assert!(result.is_err());
    }
}
