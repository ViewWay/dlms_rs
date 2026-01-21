//! Value Display interface class (Class ID: 100)
//!
//! The Value Display interface class manages display of values on meter displays.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: value - The value to display
//! - Attribute 3: unit - Unit of the displayed value
//! - Attribute 4: display_format - Format for display (string format)
//! - Attribute 5: enabled - Whether display is enabled
//! - Attribute 6: refresh_rate - Display refresh rate in seconds

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Value Display interface class (Class ID: 100)
///
/// Default OBIS: 0-0:100.0.0.255
///
/// This class manages display of values on meter displays.
#[derive(Debug, Clone)]
pub struct ValueDisplay {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// The value to display
    value: Arc<RwLock<f64>>,

    /// Unit of the displayed value
    unit: Arc<RwLock<String>>,

    /// Format for display (e.g., "{:.2}")
    display_format: Arc<RwLock<String>>,

    /// Whether display is enabled
    enabled: Arc<RwLock<bool>>,

    /// Display refresh rate in seconds
    refresh_rate: Arc<RwLock<u16>>,
}

impl ValueDisplay {
    /// Class ID for ValueDisplay
    pub const CLASS_ID: u16 = 100;

    /// Default OBIS code for ValueDisplay (0-0:100.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 100, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_VALUE: u8 = 2;
    pub const ATTR_UNIT: u8 = 3;
    pub const ATTR_DISPLAY_FORMAT: u8 = 4;
    pub const ATTR_ENABLED: u8 = 5;
    pub const ATTR_REFRESH_RATE: u8 = 6;

    /// Create a new ValueDisplay object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(0.0)),
            unit: Arc::new(RwLock::new(String::new())),
            display_format: Arc::new(RwLock::new("{:.2}".to_string())),
            enabled: Arc::new(RwLock::new(true)),
            refresh_rate: Arc::new(RwLock::new(5)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Create with initial value and unit
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `value` - Initial value
    /// * `unit` - Unit string
    pub fn with_value(logical_name: ObisCode, value: f64, unit: String) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(value)),
            unit: Arc::new(RwLock::new(unit)),
            display_format: Arc::new(RwLock::new("{:.2}".to_string())),
            enabled: Arc::new(RwLock::new(true)),
            refresh_rate: Arc::new(RwLock::new(5)),
        }
    }

    /// Get the value
    pub async fn value(&self) -> f64 {
        *self.value.read().await
    }

    /// Set the value
    pub async fn set_value(&self, value: f64) {
        *self.value.write().await = value;
    }

    /// Get the unit
    pub async fn unit(&self) -> String {
        self.unit.read().await.clone()
    }

    /// Set the unit
    pub async fn set_unit(&self, unit: String) {
        *self.unit.write().await = unit;
    }

    /// Get the display format
    pub async fn display_format(&self) -> String {
        self.display_format.read().await.clone()
    }

    /// Set the display format
    pub async fn set_display_format(&self, format: String) {
        *self.display_format.write().await = format;
    }

    /// Get the enabled status
    pub async fn enabled(&self) -> bool {
        *self.enabled.read().await
    }

    /// Set the enabled status
    pub async fn set_enabled(&self, enabled: bool) {
        *self.enabled.write().await = enabled;
    }

    /// Enable the display
    pub async fn enable(&self) {
        self.set_enabled(true).await;
    }

    /// Disable the display
    pub async fn disable(&self) {
        self.set_enabled(false).await;
    }

    /// Get the refresh rate
    pub async fn refresh_rate(&self) -> u16 {
        *self.refresh_rate.read().await
    }

    /// Set the refresh rate
    pub async fn set_refresh_rate(&self, rate: u16) {
        *self.refresh_rate.write().await = rate;
    }

    /// Format the value for display
    pub async fn format_display(&self) -> String {
        let value = self.value().await;
        let unit = self.unit().await;
        let format_str = self.display_format().await;

        let formatted_value = Self::apply_format(value, &format_str);

        if unit.is_empty() {
            formatted_value
        } else {
            format!("{} {}", formatted_value, unit)
        }
    }

    /// Get formatted value (without unit)
    pub async fn formatted_value(&self) -> String {
        let value = self.value().await;
        let format_str = self.display_format().await;
        Self::apply_format(value, &format_str)
    }

    /// Apply format string to a value
    fn apply_format(value: f64, format_str: &str) -> String {
        // Parse common format patterns
        if format_str.contains("{:.2}") || format_str == "{:.2}" {
            format!("{:.2}", value)
        } else if format_str.contains("{:.1}") || format_str == "{:.1}" {
            format!("{:.1}", value)
        } else if format_str.contains("{:.0}") || format_str == "{:.0}" {
            format!("{:.0}", value)
        } else if format_str.contains("{:.3}") || format_str == "{:.3}" {
            format!("{:.3}", value)
        } else if format_str.contains("{}") || format_str == "{}" {
            format!("{}", value)
        } else {
            // Default to 2 decimal places if format string not recognized
            format!("{:.2}", value)
        }
    }

    /// Check if display is enabled
    pub async fn is_enabled(&self) -> bool {
        self.enabled().await
    }

    /// Update value and return formatted display string
    pub async fn update_and_format(&self, new_value: f64) -> String {
        self.set_value(new_value).await;
        self.format_display().await
    }
}

#[async_trait]
impl CosemObject for ValueDisplay {
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
            Self::ATTR_VALUE => {
                Ok(DataObject::Float64(self.value().await))
            }
            Self::ATTR_UNIT => {
                Ok(DataObject::OctetString(self.unit().await.into_bytes()))
            }
            Self::ATTR_DISPLAY_FORMAT => {
                Ok(DataObject::OctetString(self.display_format().await.into_bytes()))
            }
            Self::ATTR_ENABLED => {
                Ok(DataObject::Boolean(self.enabled().await))
            }
            Self::ATTR_REFRESH_RATE => {
                Ok(DataObject::Unsigned16(self.refresh_rate().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "ValueDisplay has no attribute {}",
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
            Self::ATTR_VALUE => {
                match value {
                    DataObject::Float64(v) => {
                        self.set_value(v).await;
                        Ok(())
                    }
                    DataObject::Float32(v) => {
                        self.set_value(v as f64).await;
                        Ok(())
                    }
                    DataObject::Integer64(v) => {
                        self.set_value(v as f64).await;
                        Ok(())
                    }
                    DataObject::Integer32(v) => {
                        self.set_value(v as f64).await;
                        Ok(())
                    }
                    DataObject::Integer16(v) => {
                        self.set_value(v as f64).await;
                        Ok(())
                    }
                    DataObject::Integer8(v) => {
                        self.set_value(v as f64).await;
                        Ok(())
                    }
                    DataObject::Unsigned64(v) => {
                        self.set_value(v as f64).await;
                        Ok(())
                    }
                    DataObject::Unsigned32(v) => {
                        self.set_value(v as f64).await;
                        Ok(())
                    }
                    DataObject::Unsigned16(v) => {
                        self.set_value(v as f64).await;
                        Ok(())
                    }
                    DataObject::Unsigned8(v) => {
                        self.set_value(v as f64).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected numeric type for value".to_string(),
                    )),
                }
            }
            Self::ATTR_UNIT => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_unit(String::from_utf8_lossy(&bytes).to_string()).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for unit".to_string(),
                    )),
                }
            }
            Self::ATTR_DISPLAY_FORMAT => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_display_format(String::from_utf8_lossy(&bytes).to_string()).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for display_format".to_string(),
                    )),
                }
            }
            Self::ATTR_ENABLED => {
                match value {
                    DataObject::Boolean(enabled) => {
                        self.set_enabled(enabled).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Boolean for enabled".to_string(),
                    )),
                }
            }
            Self::ATTR_REFRESH_RATE => {
                match value {
                    DataObject::Unsigned16(rate) => {
                        self.set_refresh_rate(rate).await;
                        Ok(())
                    }
                    DataObject::Unsigned8(rate) => {
                        self.set_refresh_rate(rate as u16).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned16/Unsigned8 for refresh_rate".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "ValueDisplay has no attribute {}",
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
            "ValueDisplay has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_value_display_class_id() {
        let vd = ValueDisplay::with_default_obis();
        assert_eq!(vd.class_id(), 100);
    }

    #[tokio::test]
    async fn test_value_display_obis_code() {
        let vd = ValueDisplay::with_default_obis();
        assert_eq!(vd.obis_code(), ValueDisplay::default_obis());
    }

    #[tokio::test]
    async fn test_value_display_initial_state() {
        let vd = ValueDisplay::with_default_obis();
        assert_eq!(vd.value().await, 0.0);
        assert_eq!(vd.unit().await, "");
        assert_eq!(vd.display_format().await, "{:.2}");
        assert!(vd.enabled().await);
        assert_eq!(vd.refresh_rate().await, 5);
    }

    #[tokio::test]
    async fn test_value_display_with_value() {
        let vd = ValueDisplay::with_value(ObisCode::new(0, 0, 100, 0, 0, 255), 123.456, "kWh".to_string());
        assert_eq!(vd.value().await, 123.456);
        assert_eq!(vd.unit().await, "kWh");
    }

    #[tokio::test]
    async fn test_value_display_set_value() {
        let vd = ValueDisplay::with_default_obis();
        vd.set_value(42.5).await;
        assert_eq!(vd.value().await, 42.5);
    }

    #[tokio::test]
    async fn test_value_display_set_unit() {
        let vd = ValueDisplay::with_default_obis();
        vd.set_unit("kWh".to_string()).await;
        assert_eq!(vd.unit().await, "kWh");
    }

    #[tokio::test]
    async fn test_value_display_set_display_format() {
        let vd = ValueDisplay::with_default_obis();
        vd.set_display_format("{:.3}".to_string()).await;
        assert_eq!(vd.display_format().await, "{:.3}");
    }

    #[tokio::test]
    async fn test_value_display_enable_disable() {
        let vd = ValueDisplay::with_default_obis();
        assert!(vd.is_enabled().await);

        vd.disable().await;
        assert!(!vd.is_enabled().await);

        vd.enable().await;
        assert!(vd.is_enabled().await);
    }

    #[tokio::test]
    async fn test_value_display_set_refresh_rate() {
        let vd = ValueDisplay::with_default_obis();
        vd.set_refresh_rate(10).await;
        assert_eq!(vd.refresh_rate().await, 10);
    }

    #[tokio::test]
    async fn test_value_display_format_display() {
        let vd = ValueDisplay::with_value(ObisCode::new(0, 0, 100, 0, 0, 255), 123.456, "kWh".to_string());
        let formatted = vd.format_display().await;
        // Default format is "{:.2}"
        assert!(formatted.contains("123.46"));
        assert!(formatted.contains("kWh"));
    }

    #[tokio::test]
    async fn test_value_display_format_display_no_unit() {
        let vd = ValueDisplay::with_value(ObisCode::new(0, 0, 100, 0, 0, 255), 123.456, "".to_string());
        let formatted = vd.format_display().await;
        assert!(formatted.contains("123.46"));
    }

    #[tokio::test]
    async fn test_value_display_formatted_value() {
        let vd = ValueDisplay::with_value(ObisCode::new(0, 0, 100, 0, 0, 255), 123.456, "kWh".to_string());
        let formatted = vd.formatted_value().await;
        assert_eq!(formatted, "123.46");
        assert!(!formatted.contains("kWh"));
    }

    #[tokio::test]
    async fn test_value_display_update_and_format() {
        let vd = ValueDisplay::with_value(ObisCode::new(0, 0, 100, 0, 0, 255), 100.0, "kWh".to_string());
        let formatted = vd.update_and_format(250.789).await;
        assert_eq!(vd.value().await, 250.789);
        assert!(formatted.contains("250.79"));
    }

    #[tokio::test]
    async fn test_value_display_custom_format() {
        let vd = ValueDisplay::with_value(ObisCode::new(0, 0, 100, 0, 0, 255), 123.456, "V".to_string());
        vd.set_display_format("{:.1}".to_string()).await;
        let formatted = vd.format_display().await;
        assert!(formatted.contains("123.5"));
    }

    #[tokio::test]
    async fn test_value_display_integer_value() {
        let vd = ValueDisplay::with_default_obis();
        vd.set_value(42.0).await;
        assert_eq!(vd.formatted_value().await, "42.00");
    }

    #[tokio::test]
    async fn test_value_display_negative_value() {
        let vd = ValueDisplay::with_default_obis();
        vd.set_value(-123.456).await;
        assert!(vd.formatted_value().await.contains("-123.46"));
    }

    #[tokio::test]
    async fn test_value_display_get_attributes() {
        let vd = ValueDisplay::with_default_obis();

        // Test value
        let result = vd.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Float64(v) => assert_eq!(v, 0.0),
            _ => panic!("Expected Float64"),
        }

        // Test enabled
        let result = vd.get_attribute(5, None).await.unwrap();
        match result {
            DataObject::Boolean(enabled) => assert!(enabled),
            _ => panic!("Expected Boolean"),
        }
    }

    #[tokio::test]
    async fn test_value_display_set_attributes() {
        let vd = ValueDisplay::with_default_obis();

        vd.set_attribute(2, DataObject::Float64(42.5), None)
            .await
            .unwrap();
        assert_eq!(vd.value().await, 42.5);

        vd.set_attribute(3, DataObject::OctetString(b"kWh".to_vec()), None)
            .await
            .unwrap();
        assert_eq!(vd.unit().await, "kWh");
    }

    #[tokio::test]
    async fn test_value_display_set_value_from_integer() {
        let vd = ValueDisplay::with_default_obis();
        vd.set_attribute(2, DataObject::Integer32(100), None)
            .await
            .unwrap();
        assert_eq!(vd.value().await, 100.0);
    }

    #[tokio::test]
    async fn test_value_display_set_value_from_unsigned() {
        let vd = ValueDisplay::with_default_obis();
        vd.set_attribute(2, DataObject::Unsigned16(50), None)
            .await
            .unwrap();
        assert_eq!(vd.value().await, 50.0);
    }

    #[tokio::test]
    async fn test_value_display_set_value_from_float32() {
        let vd = ValueDisplay::with_default_obis();
        vd.set_attribute(2, DataObject::Float32(12.34), None)
            .await
            .unwrap();
        assert!((vd.value().await - 12.34).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_value_display_set_refresh_rate_u8() {
        let vd = ValueDisplay::with_default_obis();
        vd.set_attribute(6, DataObject::Unsigned8(15), None)
            .await
            .unwrap();
        assert_eq!(vd.refresh_rate().await, 15);
    }

    #[tokio::test]
    async fn test_value_display_read_only_logical_name() {
        let vd = ValueDisplay::with_default_obis();
        let result = vd
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 100, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_value_display_invalid_attribute() {
        let vd = ValueDisplay::with_default_obis();
        let result = vd.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_value_display_invalid_method() {
        let vd = ValueDisplay::with_default_obis();
        let result = vd.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_value_display_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 100, 0, 0, 1);
        let vd = ValueDisplay::new(obis);
        assert_eq!(vd.obis_code(), obis);
    }
}
