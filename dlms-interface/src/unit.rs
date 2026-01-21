//! Unit interface class (Class ID: 3 extended)
//!
//! The Unit interface class manages unit definitions and conversions.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: unit_id - The unit identifier
//! - Attribute 3: unit_name - Human-readable unit name
//! - Attribute 4: unit_symbol - Unit symbol (e.g., "kWh", "V")
//! - Attribute 5: scale_factor - Scaling factor for the unit

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Common unit identifiers (based on DLMS/COSEM standard)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UnitId {
    /// No unit
    None = 0,
    /// Year
    Year = 1,
    /// Month
    Month = 2,
    /// Week
    Week = 3,
    /// Day
    Day = 4,
    /// Hour
    Hour = 5,
    /// Minute
    Minute = 6,
    /// Second
    Second = 7,
    /// Ampere
    Ampere = 20,
    /// Volt
    Volt = 21,
    /// Volt squared
    VoltSquared = 22,
    /// Volt per meter
    VoltPerMeter = 23,
    /// Ampere per meter
    AmperePerMeter = 24,
    /// Ampere per square meter
    AmperePerSquareMeter = 25,
    /// Volt ampere
    VoltAmpere = 26,
    /// Watt
    Watt = 27,
    /// Volt ampere reactive
    VoltAmpereReactive = 28,
    /// Volt ampere hour
    VoltAmpereHour = 29,
    /// Watt hour
    WattHour = 30,
    /// Var hour
    VarHour = 31,
    /// Coulomb
    Coulomb = 32,
    /// Joule
    Joule = 33,
    /// Newton
    Newton = 34,
    /// Hertz
    Hertz = 35,
    /// Siemens
    Siemens = 36,
    /// Ohm
    Ohm = 37,
    /// Farad
    Farad = 38,
    /// Henry
    Henry = 39,
    /// Weber
    Weber = 40,
    /// Tesla
    Tesla = 41,
    /// Kelvin
    Kelvin = 42,
    /// Celsius
    Celsius = 43,
    /// Pascal
    Pascal = 44,
    /// Bar
    Bar = 45,
    /// Joule per Kelvin
    JoulePerKelvin = 46,
    /// Joule per kilogram Kelvin
    JoulePerKilogramKelvin = 47,
    /// Meter
    Meter = 62,
    /// Meter per second
    MeterPerSecond = 63,
    /// Cubic meter
    CubicMeter = 64,
    /// Cubic meter per second
    CubicMeterPerSecond = 65,
    /// Kilogram
    Kilogram = 66,
    /// Cubic meter per kilogram
    CubicMeterPerKilogram = 67,
    /// Joule per kilogram
    JoulePerKilogram = 68,
    /// Meter per second squared
    MeterPerSecondSquared = 69,
    /// Newton per meter
    NewtonPerMeter = 70,
    /// Cubic meter per hour
    CubicMeterPerHour = 71,
    /// Liter
    Liter = 72,
    /// Cubic meter per day
    CubicMeterPerDay = 73,
    /// Percentage
    Percent = 251,
    /// Parts per million
    Ppm = 252,
    /// Dimensionless
    Dimensionless = 255,
}

impl UnitId {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Year,
            2 => Self::Month,
            3 => Self::Week,
            4 => Self::Day,
            5 => Self::Hour,
            6 => Self::Minute,
            7 => Self::Second,
            20 => Self::Ampere,
            21 => Self::Volt,
            22 => Self::VoltSquared,
            23 => Self::VoltPerMeter,
            24 => Self::AmperePerMeter,
            25 => Self::AmperePerSquareMeter,
            26 => Self::VoltAmpere,
            27 => Self::Watt,
            28 => Self::VoltAmpereReactive,
            29 => Self::VoltAmpereHour,
            30 => Self::WattHour,
            31 => Self::VarHour,
            32 => Self::Coulomb,
            33 => Self::Joule,
            34 => Self::Newton,
            35 => Self::Hertz,
            36 => Self::Siemens,
            37 => Self::Ohm,
            38 => Self::Farad,
            39 => Self::Henry,
            40 => Self::Weber,
            41 => Self::Tesla,
            42 => Self::Kelvin,
            43 => Self::Celsius,
            44 => Self::Pascal,
            45 => Self::Bar,
            46 => Self::JoulePerKelvin,
            47 => Self::JoulePerKilogramKelvin,
            62 => Self::Meter,
            63 => Self::MeterPerSecond,
            64 => Self::CubicMeter,
            65 => Self::CubicMeterPerSecond,
            66 => Self::Kilogram,
            67 => Self::CubicMeterPerKilogram,
            68 => Self::JoulePerKilogram,
            69 => Self::MeterPerSecondSquared,
            70 => Self::NewtonPerMeter,
            71 => Self::CubicMeterPerHour,
            72 => Self::Liter,
            73 => Self::CubicMeterPerDay,
            251 => Self::Percent,
            252 => Self::Ppm,
            255 => Self::Dimensionless,
            _ => Self::None,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Get the symbol for this unit
    pub fn symbol(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Year => "year",
            Self::Month => "month",
            Self::Week => "week",
            Self::Day => "d",
            Self::Hour => "h",
            Self::Minute => "min",
            Self::Second => "s",
            Self::Ampere => "A",
            Self::Volt => "V",
            Self::VoltSquared => "V²",
            Self::VoltPerMeter => "V/m",
            Self::AmperePerMeter => "A/m",
            Self::AmperePerSquareMeter => "A/m²",
            Self::VoltAmpere => "VA",
            Self::Watt => "W",
            Self::VoltAmpereReactive => "var",
            Self::VoltAmpereHour => "VAh",
            Self::WattHour => "Wh",
            Self::VarHour => "varh",
            Self::Coulomb => "C",
            Self::Joule => "J",
            Self::Newton => "N",
            Self::Hertz => "Hz",
            Self::Siemens => "S",
            Self::Ohm => "Ω",
            Self::Farad => "F",
            Self::Henry => "H",
            Self::Weber => "Wb",
            Self::Tesla => "T",
            Self::Kelvin => "K",
            Self::Celsius => "°C",
            Self::Pascal => "Pa",
            Self::Bar => "bar",
            Self::JoulePerKelvin => "J/K",
            Self::JoulePerKilogramKelvin => "J/(kg·K)",
            Self::Meter => "m",
            Self::MeterPerSecond => "m/s",
            Self::CubicMeter => "m³",
            Self::CubicMeterPerSecond => "m³/s",
            Self::Kilogram => "kg",
            Self::CubicMeterPerKilogram => "m³/kg",
            Self::JoulePerKilogram => "J/kg",
            Self::MeterPerSecondSquared => "m/s²",
            Self::NewtonPerMeter => "N/m",
            Self::CubicMeterPerHour => "m³/h",
            Self::Liter => "L",
            Self::CubicMeterPerDay => "m³/d",
            Self::Percent => "%",
            Self::Ppm => "ppm",
            Self::Dimensionless => "",
        }
    }

    /// Check if this is an energy unit
    pub fn is_energy_unit(self) -> bool {
        matches!(
            self,
            Self::WattHour | Self::VoltAmpereHour | Self::VarHour | Self::Joule
        )
    }

    /// Check if this is a power unit
    pub fn is_power_unit(self) -> bool {
        matches!(
            self,
            Self::Watt | Self::VoltAmpere | Self::VoltAmpereReactive
        )
    }

    /// Check if this is a voltage unit
    pub fn is_voltage_unit(self) -> bool {
        matches!(self, Self::Volt | Self::VoltSquared)
    }

    /// Check if this is a current unit
    pub fn is_current_unit(self) -> bool {
        matches!(self, Self::Ampere)
    }

    /// Check if this is a volume unit
    pub fn is_volume_unit(self) -> bool {
        matches!(
            self,
            Self::CubicMeter
                | Self::CubicMeterPerSecond
                | Self::CubicMeterPerHour
                | Self::CubicMeterPerDay
                | Self::Liter
        )
    }

    /// Check if this is a temperature unit
    pub fn is_temperature_unit(self) -> bool {
        matches!(self, Self::Kelvin | Self::Celsius)
    }
}

/// Unit interface class (Class ID: 3 extended)
///
/// Default OBIS: 0-0:3.0.0.255
///
/// This class manages unit definitions and conversions.
#[derive(Debug, Clone)]
pub struct Unit {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// The unit identifier
    unit_id: Arc<RwLock<UnitId>>,

    /// Human-readable unit name
    unit_name: Arc<RwLock<String>>,

    /// Unit symbol
    unit_symbol: Arc<RwLock<String>>,

    /// Scaling factor (power of 10)
    scale_factor: Arc<RwLock<i8>>,
}

impl Unit {
    /// Class ID for Unit
    pub const CLASS_ID: u16 = 3;

    /// Default OBIS code for Unit (0-0:3.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 3, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_UNIT_ID: u8 = 2;
    pub const ATTR_UNIT_NAME: u8 = 3;
    pub const ATTR_UNIT_SYMBOL: u8 = 4;
    pub const ATTR_SCALE_FACTOR: u8 = 5;

    /// Create a new Unit object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            unit_id: Arc::new(RwLock::new(UnitId::None)),
            unit_name: Arc::new(RwLock::new(String::new())),
            unit_symbol: Arc::new(RwLock::new(String::new())),
            scale_factor: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Create with a specific unit ID
    pub fn with_unit(logical_name: ObisCode, unit_id: UnitId) -> Self {
        Self {
            logical_name,
            unit_id: Arc::new(RwLock::new(unit_id)),
            unit_name: Arc::new(RwLock::new(String::from(format!("{:?}", unit_id)))),
            unit_symbol: Arc::new(RwLock::new(String::from(unit_id.symbol()))),
            scale_factor: Arc::new(RwLock::new(0)),
        }
    }

    /// Get the unit ID
    pub async fn unit_id(&self) -> UnitId {
        *self.unit_id.read().await
    }

    /// Set the unit ID
    pub async fn set_unit_id(&self, unit_id: UnitId) {
        *self.unit_id.write().await = unit_id;
    }

    /// Get the unit name
    pub async fn unit_name(&self) -> String {
        self.unit_name.read().await.clone()
    }

    /// Set the unit name
    pub async fn set_unit_name(&self, name: String) {
        *self.unit_name.write().await = name;
    }

    /// Get the unit symbol
    pub async fn unit_symbol(&self) -> String {
        self.unit_symbol.read().await.clone()
    }

    /// Set the unit symbol
    pub async fn set_unit_symbol(&self, symbol: String) {
        *self.unit_symbol.write().await = symbol;
    }

    /// Get the scale factor
    pub async fn scale_factor(&self) -> i8 {
        *self.scale_factor.read().await
    }

    /// Set the scale factor
    pub async fn set_scale_factor(&self, factor: i8) {
        *self.scale_factor.write().await = factor;
    }

    /// Get the scaling multiplier (10^scale_factor)
    pub async fn scaling_multiplier(&self) -> f64 {
        10_f64.powi(self.scale_factor().await as i32)
    }

    /// Apply scaling to a value
    pub async fn apply_scaling(&self, value: f64) -> f64 {
        value * self.scaling_multiplier().await
    }

    /// Remove scaling from a value
    pub async fn remove_scaling(&self, value: f64) -> f64 {
        value / self.scaling_multiplier().await
    }

    /// Get the standard symbol (from UnitId enum)
    pub async fn standard_symbol(&self) -> &'static str {
        self.unit_id().await.symbol()
    }

    /// Check if this is an energy unit
    pub async fn is_energy_unit(&self) -> bool {
        self.unit_id().await.is_energy_unit()
    }

    /// Check if this is a power unit
    pub async fn is_power_unit(&self) -> bool {
        self.unit_id().await.is_power_unit()
    }

    /// Check if this is a voltage unit
    pub async fn is_voltage_unit(&self) -> bool {
        self.unit_id().await.is_voltage_unit()
    }

    /// Check if this is a current unit
    pub async fn is_current_unit(&self) -> bool {
        self.unit_id().await.is_current_unit()
    }

    /// Check if this is a volume unit
    pub async fn is_volume_unit(&self) -> bool {
        self.unit_id().await.is_volume_unit()
    }

    /// Check if this is a temperature unit
    pub async fn is_temperature_unit(&self) -> bool {
        self.unit_id().await.is_temperature_unit()
    }
}

#[async_trait]
impl CosemObject for Unit {
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
            Self::ATTR_UNIT_ID => {
                Ok(DataObject::Enumerate(self.unit_id().await.to_u8()))
            }
            Self::ATTR_UNIT_NAME => {
                Ok(DataObject::OctetString(self.unit_name().await.into_bytes()))
            }
            Self::ATTR_UNIT_SYMBOL => {
                Ok(DataObject::OctetString(self.unit_symbol().await.into_bytes()))
            }
            Self::ATTR_SCALE_FACTOR => {
                Ok(DataObject::Integer8(self.scale_factor().await))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Unit has no attribute {}",
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
            Self::ATTR_UNIT_ID => {
                match value {
                    DataObject::Enumerate(id) => {
                        self.set_unit_id(UnitId::from_u8(id)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for unit_id".to_string(),
                    )),
                }
            }
            Self::ATTR_UNIT_NAME => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_unit_name(String::from_utf8_lossy(&bytes).to_string()).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for unit_name".to_string(),
                    )),
                }
            }
            Self::ATTR_UNIT_SYMBOL => {
                match value {
                    DataObject::OctetString(bytes) => {
                        self.set_unit_symbol(String::from_utf8_lossy(&bytes).to_string()).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for unit_symbol".to_string(),
                    )),
                }
            }
            Self::ATTR_SCALE_FACTOR => {
                match value {
                    DataObject::Integer8(factor) => {
                        self.set_scale_factor(factor).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Integer8 for scale_factor".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Unit has no attribute {}",
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
            "Unit has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unit_class_id() {
        let unit = Unit::with_default_obis();
        assert_eq!(unit.class_id(), 3);
    }

    #[tokio::test]
    async fn test_unit_obis_code() {
        let unit = Unit::with_default_obis();
        assert_eq!(unit.obis_code(), Unit::default_obis());
    }

    #[tokio::test]
    async fn test_unit_id_from_u8() {
        assert_eq!(UnitId::from_u8(0), UnitId::None);
        assert_eq!(UnitId::from_u8(27), UnitId::Watt);
        assert_eq!(UnitId::from_u8(30), UnitId::WattHour);
        assert_eq!(UnitId::from_u8(21), UnitId::Volt);
        assert_eq!(UnitId::from_u8(20), UnitId::Ampere);
    }

    #[tokio::test]
    async fn test_unit_id_symbol() {
        assert_eq!(UnitId::Watt.symbol(), "W");
        assert_eq!(UnitId::WattHour.symbol(), "Wh");
        assert_eq!(UnitId::Volt.symbol(), "V");
        assert_eq!(UnitId::Ampere.symbol(), "A");
        assert_eq!(UnitId::CubicMeter.symbol(), "m³");
        assert_eq!(UnitId::Percent.symbol(), "%");
    }

    #[tokio::test]
    async fn test_unit_id_is_energy_unit() {
        assert!(UnitId::WattHour.is_energy_unit());
        assert!(UnitId::Joule.is_energy_unit());
        assert!(!UnitId::Watt.is_energy_unit());
        assert!(!UnitId::Volt.is_energy_unit());
    }

    #[tokio::test]
    async fn test_unit_id_is_power_unit() {
        assert!(UnitId::Watt.is_power_unit());
        assert!(UnitId::VoltAmpere.is_power_unit());
        assert!(!UnitId::WattHour.is_power_unit());
    }

    #[tokio::test]
    async fn test_unit_id_is_voltage_unit() {
        assert!(UnitId::Volt.is_voltage_unit());
        assert!(UnitId::VoltSquared.is_voltage_unit());
        assert!(!UnitId::Ampere.is_voltage_unit());
    }

    #[tokio::test]
    async fn test_unit_id_is_current_unit() {
        assert!(UnitId::Ampere.is_current_unit());
        assert!(!UnitId::Volt.is_current_unit());
    }

    #[tokio::test]
    async fn test_unit_id_is_volume_unit() {
        assert!(UnitId::CubicMeter.is_volume_unit());
        assert!(UnitId::Liter.is_volume_unit());
        assert!(!UnitId::Meter.is_volume_unit());
    }

    #[tokio::test]
    async fn test_unit_id_is_temperature_unit() {
        assert!(UnitId::Kelvin.is_temperature_unit());
        assert!(UnitId::Celsius.is_temperature_unit());
        assert!(!UnitId::Pascal.is_temperature_unit());
    }

    #[tokio::test]
    async fn test_unit_initial_state() {
        let unit = Unit::with_default_obis();
        assert_eq!(unit.unit_id().await, UnitId::None);
        assert_eq!(unit.scale_factor().await, 0);
    }

    #[tokio::test]
    async fn test_unit_with_unit_id() {
        let unit = Unit::with_unit(ObisCode::new(0, 0, 3, 0, 0, 255), UnitId::WattHour);
        assert_eq!(unit.unit_id().await, UnitId::WattHour);
        assert!(unit.is_energy_unit().await);
        assert_eq!(unit.unit_symbol().await, "Wh");
    }

    #[tokio::test]
    async fn test_unit_set_unit_id() {
        let unit = Unit::with_default_obis();
        unit.set_unit_id(UnitId::WattHour).await;
        assert_eq!(unit.unit_id().await, UnitId::WattHour);
    }

    #[tokio::test]
    async fn test_unit_set_unit_name() {
        let unit = Unit::with_default_obis();
        unit.set_unit_name(String::from("Kilowatt Hour")).await;
        assert_eq!(unit.unit_name().await, "Kilowatt Hour");
    }

    #[tokio::test]
    async fn test_unit_set_unit_symbol() {
        let unit = Unit::with_default_obis();
        unit.set_unit_symbol(String::from("kWh")).await;
        assert_eq!(unit.unit_symbol().await, "kWh");
    }

    #[tokio::test]
    async fn test_unit_set_scale_factor() {
        let unit = Unit::with_default_obis();
        unit.set_scale_factor(3).await;
        assert_eq!(unit.scale_factor().await, 3);
        assert_eq!(unit.scaling_multiplier().await, 1000.0);
    }

    #[tokio::test]
    async fn test_unit_apply_scaling() {
        let unit = Unit::with_default_obis();
        unit.set_scale_factor(2).await; // Scale by 100
        assert_eq!(unit.scaling_multiplier().await, 100.0);

        let scaled = unit.apply_scaling(5.0).await;
        assert!((scaled - 500.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_unit_remove_scaling() {
        let unit = Unit::with_default_obis();
        unit.set_scale_factor(2).await; // Scale by 100

        let unscaled = unit.remove_scaling(500.0).await;
        assert!((unscaled - 5.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_unit_negative_scale_factor() {
        let unit = Unit::with_default_obis();
        unit.set_scale_factor(-2).await; // Scale by 0.01
        assert_eq!(unit.scaling_multiplier().await, 0.01);

        let scaled = unit.apply_scaling(500.0).await;
        assert!((scaled - 5.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_unit_is_energy_unit() {
        let unit = Unit::with_unit(ObisCode::new(0, 0, 3, 0, 0, 255), UnitId::WattHour);
        assert!(unit.is_energy_unit().await);
    }

    #[tokio::test]
    async fn test_unit_is_power_unit() {
        let unit = Unit::with_unit(ObisCode::new(0, 0, 3, 0, 0, 255), UnitId::Watt);
        assert!(unit.is_power_unit().await);
    }

    #[tokio::test]
    async fn test_unit_is_voltage_unit() {
        let unit = Unit::with_unit(ObisCode::new(0, 0, 3, 0, 0, 255), UnitId::Volt);
        assert!(unit.is_voltage_unit().await);
    }

    #[tokio::test]
    async fn test_unit_is_current_unit() {
        let unit = Unit::with_unit(ObisCode::new(0, 0, 3, 0, 0, 255), UnitId::Ampere);
        assert!(unit.is_current_unit().await);
    }

    #[tokio::test]
    async fn test_unit_is_volume_unit() {
        let unit = Unit::with_unit(ObisCode::new(0, 0, 3, 0, 0, 255), UnitId::CubicMeter);
        assert!(unit.is_volume_unit().await);
    }

    #[tokio::test]
    async fn test_unit_is_temperature_unit() {
        let unit = Unit::with_unit(ObisCode::new(0, 0, 3, 0, 0, 255), UnitId::Celsius);
        assert!(unit.is_temperature_unit().await);
    }

    #[tokio::test]
    async fn test_unit_get_attributes() {
        let unit = Unit::with_unit(ObisCode::new(0, 0, 3, 0, 0, 255), UnitId::Watt);

        // Test unit_id
        let result = unit.get_attribute(2, None).await.unwrap();
        match result {
            DataObject::Enumerate(id) => assert_eq!(id, 27), // Watt
            _ => panic!("Expected Enumerate"),
        }

        // Test scale_factor
        let result = unit.get_attribute(5, None).await.unwrap();
        match result {
            DataObject::Integer8(factor) => assert_eq!(factor, 0),
            _ => panic!("Expected Integer8"),
        }
    }

    #[tokio::test]
    async fn test_unit_set_attributes() {
        let unit = Unit::with_default_obis();

        unit.set_attribute(2, DataObject::Enumerate(30), None) // WattHour
            .await
            .unwrap();
        assert_eq!(unit.unit_id().await, UnitId::WattHour);

        unit.set_attribute(5, DataObject::Integer8(-1), None)
            .await
            .unwrap();
        assert_eq!(unit.scale_factor().await, -1);
    }

    #[tokio::test]
    async fn test_unit_read_only_logical_name() {
        let unit = Unit::with_default_obis();
        let result = unit
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 3, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unit_invalid_attribute() {
        let unit = Unit::with_default_obis();
        let result = unit.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unit_invalid_method() {
        let unit = Unit::with_default_obis();
        let result = unit.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unit_with_custom_obis() {
        let obis = ObisCode::new(1, 1, 3, 0, 0, 1);
        let unit = Unit::new(obis);
        assert_eq!(unit.obis_code(), obis);
    }

    #[tokio::test]
    async fn test_unit_standard_symbol() {
        let unit = Unit::with_unit(ObisCode::new(0, 0, 3, 0, 0, 255), UnitId::VoltAmpere);
        assert_eq!(unit.standard_symbol().await, "VA");
    }

    #[tokio::test]
    async fn test_unit_get_attributes_unit_name() {
        let unit = Unit::with_default_obis();
        unit.set_unit_name(String::from("Test Unit")).await;

        let result = unit.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(String::from_utf8_lossy(&bytes), "Test Unit");
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_unit_get_attributes_unit_symbol() {
        let unit = Unit::with_default_obis();
        unit.set_unit_symbol(String::from("kW")).await;

        let result = unit.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(String::from_utf8_lossy(&bytes), "kW");
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_unit_unknown_id_fallback() {
        let id = UnitId::from_u8(255); // Dimensionless
        assert_eq!(id, UnitId::Dimensionless);

        let id = UnitId::from_u8(200); // Unknown
        assert_eq!(id, UnitId::None);
    }
}
