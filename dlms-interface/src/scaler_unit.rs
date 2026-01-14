//! Scaler Unit for Register interface class
//!
//! Scaler Unit is used in Register and Extended Register interface classes
//! to represent the scaling factor and unit of measurement for register values.
//!
//! # Structure
//!
//! Scaler Unit consists of:
//! - **scaler**: i8 (-128 to 127) - Scaling factor (10^scaler)
//! - **unit**: u8 - Unit code (e.g., 0x1B = W, 0x1E = Wh)
//!
//! # Unit Codes
//!
//! Common unit codes (from DLMS Green Book):
//! - 0x00: No unit
//! - 0x1B: W (Watt)
//! - 0x1E: Wh (Watt-hour)
//! - 0x23: V (Volt)
//! - 0x27: A (Ampere)
//! - 0x2B: Hz (Hertz)
//! - 0x2F: °C (Celsius)
//!
//! # Usage
//!
//! ```rust,no_run
//! use dlms_interface::ScalerUnit;
//!
//! // Create a ScalerUnit for energy (Wh) with scale factor 0
//! let scaler_unit = ScalerUnit::new(0, 0x1E);
//!
//! // Create a ScalerUnit for power (kW) with scale factor 3 (10^3 = 1000)
//! let scaler_unit = ScalerUnit::new(3, 0x1B);
//! ```

use dlms_core::{DlmsError, DlmsResult, DataObject};
use dlms_asn1::{AxdrDecoder, AxdrEncoder};

/// Scaler Unit structure
///
/// Represents the scaling factor and unit of measurement for register values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScalerUnit {
    /// Scaling factor (-128 to 127)
    /// The actual value is multiplied by 10^scaler
    /// Example: scaler = 3 means multiply by 1000 (10^3)
    pub scaler: i8,
    /// Unit code (0x00 = no unit, 0x1B = W, 0x1E = Wh, etc.)
    pub unit: u8,
}

impl ScalerUnit {
    /// Create a new ScalerUnit
    ///
    /// # Arguments
    /// * `scaler` - Scaling factor (-128 to 127)
    /// * `unit` - Unit code (0x00 to 0xFF)
    ///
    /// # Returns
    /// A new ScalerUnit instance
    pub fn new(scaler: i8, unit: u8) -> Self {
        Self { scaler, unit }
    }

    /// Create a ScalerUnit with no scaling and no unit
    pub fn none() -> Self {
        Self { scaler: 0, unit: 0x00 }
    }

    /// Get the scaling factor
    pub fn scaler(&self) -> i8 {
        self.scaler
    }

    /// Get the unit code
    pub fn unit(&self) -> u8 {
        self.unit
    }

    /// Apply scaling to a value
    ///
    /// # Arguments
    /// * `value` - The raw register value
    ///
    /// # Returns
    /// The scaled value (value * 10^scaler)
    ///
    /// # Example
    /// ```rust,no_run
    /// use dlms_interface::ScalerUnit;
    ///
    /// let scaler_unit = ScalerUnit::new(3, 0x1B); // kW
    /// let raw_value = 12345; // Raw value from register
    /// let scaled_value = scaler_unit.scale_value(raw_value as f64);
    /// // scaled_value = 12345.0 * 1000.0 = 12345000.0
    /// ```
    pub fn scale_value(&self, value: f64) -> f64 {
        value * 10_f64.powi(self.scaler as i32)
    }

    /// Reverse scaling (convert scaled value back to raw value)
    ///
    /// # Arguments
    /// * `scaled_value` - The scaled value
    ///
    /// # Returns
    /// The raw value (scaled_value / 10^scaler)
    pub fn unscale_value(&self, scaled_value: f64) -> f64 {
        scaled_value / 10_f64.powi(self.scaler as i32)
    }

    /// Encode to A-XDR format
    ///
    /// Encoding format:
    /// - scaler: Integer8
    /// - unit: Unsigned8
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();
        encoder.encode_integer8(self.scaler)?;
        encoder.encode_unsigned8(self.unit)?;
        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = AxdrDecoder::new(data);
        let scaler = decoder.decode_integer8()?;
        let unit = decoder.decode_unsigned8()?;
        Ok(Self::new(scaler, unit))
    }

    /// Convert to DataObject (Structure)
    pub fn to_data_object(&self) -> DataObject {
        DataObject::Structure(vec![
            DataObject::Integer8(self.scaler),
            DataObject::Unsigned8(self.unit),
        ])
    }

    /// Create from DataObject (Structure)
    pub fn from_data_object(obj: &DataObject) -> DlmsResult<Self> {
        match obj {
            DataObject::Structure(elements) => {
                if elements.len() != 2 {
                    return Err(DlmsError::InvalidData(format!(
                        "ScalerUnit structure must have 2 elements, got {}",
                        elements.len()
                    )));
                }

                let scaler = match &elements[0] {
                    DataObject::Integer8(v) => *v,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "ScalerUnit scaler must be Integer8".to_string(),
                        ));
                    }
                };

                let unit = match &elements[1] {
                    DataObject::Unsigned8(v) => *v,
                    _ => {
                        return Err(DlmsError::InvalidData(
                            "ScalerUnit unit must be Unsigned8".to_string(),
                        ));
                    }
                };

                Ok(Self::new(scaler, unit))
            }
            _ => Err(DlmsError::InvalidData(
                "ScalerUnit must be a Structure".to_string(),
            )),
        }
    }
}

/// Common unit codes (from DLMS Green Book)
pub mod units {
    /// No unit
    pub const NO_UNIT: u8 = 0x00;
    /// Watt (W)
    pub const WATT: u8 = 0x1B;
    /// Watt-hour (Wh)
    pub const WATT_HOUR: u8 = 0x1E;
    /// Volt (V)
    pub const VOLT: u8 = 0x23;
    /// Ampere (A)
    pub const AMPERE: u8 = 0x27;
    /// Hertz (Hz)
    pub const HERTZ: u8 = 0x2B;
    /// Celsius (°C)
    pub const CELSIUS: u8 = 0x2F;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaler_unit_creation() {
        let su = ScalerUnit::new(3, 0x1B);
        assert_eq!(su.scaler(), 3);
        assert_eq!(su.unit(), 0x1B);
    }

    #[test]
    fn test_scaler_unit_none() {
        let su = ScalerUnit::none();
        assert_eq!(su.scaler(), 0);
        assert_eq!(su.unit(), 0x00);
    }

    #[test]
    fn test_scale_value() {
        let su = ScalerUnit::new(3, 0x1B); // kW
        let raw = 12345.0;
        let scaled = su.scale_value(raw);
        assert!((scaled - 12345000.0).abs() < 0.001);
    }

    #[test]
    fn test_unscale_value() {
        let su = ScalerUnit::new(3, 0x1B); // kW
        let scaled = 12345000.0;
        let raw = su.unscale_value(scaled);
        assert!((raw - 12345.0).abs() < 0.001);
    }

    #[test]
    fn test_encode_decode() {
        let su = ScalerUnit::new(3, 0x1B);
        let encoded = su.encode().unwrap();
        let decoded = ScalerUnit::decode(&encoded).unwrap();
        assert_eq!(su, decoded);
    }

    #[test]
    fn test_to_from_data_object() {
        let su = ScalerUnit::new(3, 0x1B);
        let obj = su.to_data_object();
        let decoded = ScalerUnit::from_data_object(&obj).unwrap();
        assert_eq!(su, decoded);
    }
}
