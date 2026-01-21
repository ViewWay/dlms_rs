//! LED Display interface class (Class ID: 49)
//!
//! The LED Display interface class manages LED display functionality for meters.
//!
//! # Attributes
//!
//! - Attribute 1: logical_name (OBIS code) - The logical name of the object
//! - Attribute 2: display_text - Current text being displayed
//! - Attribute 3: display_enabled - Whether the display is enabled
//! - Attribute 4: brightness_level - Display brightness level (0-100)
//! - Attribute 5: scroll_speed - Speed for scrolling text
//! - Attribute 6: blink_enabled - Whether blinking is enabled
//! - Attribute 7: display_mode - Current display mode

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Display Mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DisplayMode {
    /// Static display
    Static = 0,
    /// Scrolling left
    ScrollLeft = 1,
    /// Scrolling right
    ScrollRight = 2,
    /// Blinking
    Blink = 3,
    /// Flashing
    Flash = 4,
}

impl DisplayMode {
    /// Create from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Static,
            1 => Self::ScrollLeft,
            2 => Self::ScrollRight,
            3 => Self::Blink,
            4 => Self::Flash,
            _ => Self::Static,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if this is an animated mode
    pub fn is_animated(self) -> bool {
        matches!(self, Self::ScrollLeft | Self::ScrollRight | Self::Blink | Self::Flash)
    }
}

/// LED Display interface class (Class ID: 49)
///
/// Default OBIS: 0-0:49.0.0.255
///
/// This class manages LED display functionality for meters.
#[derive(Debug, Clone)]
pub struct LedDisplay {
    /// Logical name (OBIS code) of this object
    logical_name: ObisCode,

    /// Current text being displayed
    display_text: Arc<RwLock<String>>,

    /// Whether the display is enabled
    display_enabled: Arc<RwLock<bool>>,

    /// Display brightness level (0-100)
    brightness_level: Arc<RwLock<u8>>,

    /// Speed for scrolling text (1-10)
    scroll_speed: Arc<RwLock<u8>>,

    /// Whether blinking is enabled
    blink_enabled: Arc<RwLock<bool>>,

    /// Current display mode
    display_mode: Arc<RwLock<DisplayMode>>,
}

impl LedDisplay {
    /// Class ID for LedDisplay
    pub const CLASS_ID: u16 = 49;

    /// Default OBIS code for LedDisplay (0-0:49.0.0.255)
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 49, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_DISPLAY_TEXT: u8 = 2;
    pub const ATTR_DISPLAY_ENABLED: u8 = 3;
    pub const ATTR_BRIGHTNESS_LEVEL: u8 = 4;
    pub const ATTR_SCROLL_SPEED: u8 = 5;
    pub const ATTR_BLINK_ENABLED: u8 = 6;
    pub const ATTR_DISPLAY_MODE: u8 = 7;

    /// Create a new LedDisplay object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            display_text: Arc::new(RwLock::new(String::new())),
            display_enabled: Arc::new(RwLock::new(true)),
            brightness_level: Arc::new(RwLock::new(100)),
            scroll_speed: Arc::new(RwLock::new(5)),
            blink_enabled: Arc::new(RwLock::new(false)),
            display_mode: Arc::new(RwLock::new(DisplayMode::Static)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Get the display text
    pub async fn display_text(&self) -> String {
        self.display_text.read().await.clone()
    }

    /// Set the display text
    pub async fn set_display_text(&self, text: String) {
        *self.display_text.write().await = text;
    }

    /// Get whether the display is enabled
    pub async fn display_enabled(&self) -> bool {
        *self.display_enabled.read().await
    }

    /// Set whether the display is enabled
    pub async fn set_display_enabled(&self, enabled: bool) {
        *self.display_enabled.write().await = enabled;
    }

    /// Get the brightness level
    pub async fn brightness_level(&self) -> u8 {
        *self.brightness_level.read().await
    }

    /// Set the brightness level
    pub async fn set_brightness_level(&self, level: u8) {
        *self.brightness_level.write().await = level.min(100);
    }

    /// Get the scroll speed
    pub async fn scroll_speed(&self) -> u8 {
        *self.scroll_speed.read().await
    }

    /// Set the scroll speed
    pub async fn set_scroll_speed(&self, speed: u8) {
        *self.scroll_speed.write().await = speed.min(10).max(1);
    }

    /// Get whether blinking is enabled
    pub async fn blink_enabled(&self) -> bool {
        *self.blink_enabled.read().await
    }

    /// Set whether blinking is enabled
    pub async fn set_blink_enabled(&self, enabled: bool) {
        *self.blink_enabled.write().await = enabled;
    }

    /// Get the display mode
    pub async fn display_mode(&self) -> DisplayMode {
        *self.display_mode.read().await
    }

    /// Set the display mode
    pub async fn set_display_mode(&self, mode: DisplayMode) {
        *self.display_mode.write().await = mode;
    }

    /// Show text on the display
    pub async fn show(&self, text: String) {
        self.set_display_text(text).await;
        self.set_display_enabled(true).await;
    }

    /// Clear the display
    pub async fn clear(&self) {
        self.set_display_text(String::new()).await;
    }

    /// Turn off the display
    pub async fn turn_off(&self) {
        self.set_display_enabled(false).await;
    }

    /// Turn on the display
    pub async fn turn_on(&self) {
        self.set_display_enabled(true).await;
    }

    /// Set maximum brightness
    pub async fn max_brightness(&self) {
        self.set_brightness_level(100).await;
    }

    /// Set minimum brightness
    pub async fn min_brightness(&self) {
        self.set_brightness_level(1).await;
    }

    /// Check if display is animated
    pub async fn is_animated(&self) -> bool {
        self.display_mode().await.is_animated()
    }
}

#[async_trait]
impl CosemObject for LedDisplay {
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
            Self::ATTR_DISPLAY_TEXT => {
                Ok(DataObject::OctetString(self.display_text().await.into_bytes()))
            }
            Self::ATTR_DISPLAY_ENABLED => {
                Ok(DataObject::Boolean(self.display_enabled().await))
            }
            Self::ATTR_BRIGHTNESS_LEVEL => {
                Ok(DataObject::Unsigned8(self.brightness_level().await))
            }
            Self::ATTR_SCROLL_SPEED => {
                Ok(DataObject::Unsigned8(self.scroll_speed().await))
            }
            Self::ATTR_BLINK_ENABLED => {
                Ok(DataObject::Boolean(self.blink_enabled().await))
            }
            Self::ATTR_DISPLAY_MODE => {
                Ok(DataObject::Enumerate(self.display_mode().await.to_u8()))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "LedDisplay has no attribute {}",
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
            Self::ATTR_DISPLAY_TEXT => {
                match value {
                    DataObject::OctetString(bytes) => {
                        let text = String::from_utf8_lossy(&bytes).to_string();
                        self.set_display_text(text).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected OctetString for display_text".to_string(),
                    )),
                }
            }
            Self::ATTR_DISPLAY_ENABLED => {
                match value {
                    DataObject::Boolean(enabled) => {
                        self.set_display_enabled(enabled).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Boolean for display_enabled".to_string(),
                    )),
                }
            }
            Self::ATTR_BRIGHTNESS_LEVEL => {
                match value {
                    DataObject::Unsigned8(level) => {
                        self.set_brightness_level(level).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for brightness_level".to_string(),
                    )),
                }
            }
            Self::ATTR_SCROLL_SPEED => {
                match value {
                    DataObject::Unsigned8(speed) => {
                        self.set_scroll_speed(speed).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Unsigned8 for scroll_speed".to_string(),
                    )),
                }
            }
            Self::ATTR_BLINK_ENABLED => {
                match value {
                    DataObject::Boolean(enabled) => {
                        self.set_blink_enabled(enabled).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Boolean for blink_enabled".to_string(),
                    )),
                }
            }
            Self::ATTR_DISPLAY_MODE => {
                match value {
                    DataObject::Enumerate(mode) => {
                        self.set_display_mode(DisplayMode::from_u8(mode)).await;
                        Ok(())
                    }
                    _ => Err(DlmsError::InvalidData(
                        "Expected Enumerate for display_mode".to_string(),
                    )),
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "LedDisplay has no attribute {}",
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
            "LedDisplay has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_led_display_class_id() {
        let led = LedDisplay::with_default_obis();
        assert_eq!(led.class_id(), 49);
    }

    #[tokio::test]
    async fn test_led_display_obis_code() {
        let led = LedDisplay::with_default_obis();
        assert_eq!(led.obis_code(), LedDisplay::default_obis());
    }

    #[tokio::test]
    async fn test_display_mode_from_u8() {
        assert_eq!(DisplayMode::from_u8(0), DisplayMode::Static);
        assert_eq!(DisplayMode::from_u8(1), DisplayMode::ScrollLeft);
        assert_eq!(DisplayMode::from_u8(2), DisplayMode::ScrollRight);
        assert_eq!(DisplayMode::from_u8(3), DisplayMode::Blink);
        assert_eq!(DisplayMode::from_u8(4), DisplayMode::Flash);
    }

    #[tokio::test]
    async fn test_display_mode_is_animated() {
        assert!(!DisplayMode::Static.is_animated());
        assert!(DisplayMode::ScrollLeft.is_animated());
        assert!(DisplayMode::ScrollRight.is_animated());
        assert!(DisplayMode::Blink.is_animated());
        assert!(DisplayMode::Flash.is_animated());
    }

    #[tokio::test]
    async fn test_led_display_initial_state() {
        let led = LedDisplay::with_default_obis();
        assert_eq!(led.display_text().await, "");
        assert!(led.display_enabled().await);
        assert_eq!(led.brightness_level().await, 100);
        assert_eq!(led.scroll_speed().await, 5);
        assert!(!led.blink_enabled().await);
        assert_eq!(led.display_mode().await, DisplayMode::Static);
    }

    #[tokio::test]
    async fn test_led_display_set_display_text() {
        let led = LedDisplay::with_default_obis();
        led.set_display_text("Hello".to_string()).await;
        assert_eq!(led.display_text().await, "Hello");
    }

    #[tokio::test]
    async fn test_led_display_set_brightness_level() {
        let led = LedDisplay::with_default_obis();
        led.set_brightness_level(75).await;
        assert_eq!(led.brightness_level().await, 75);

        led.set_brightness_level(150).await;
        assert_eq!(led.brightness_level().await, 100); // Clamped to 100
    }

    #[tokio::test]
    async fn test_led_display_set_scroll_speed() {
        let led = LedDisplay::with_default_obis();
        led.set_scroll_speed(8).await;
        assert_eq!(led.scroll_speed().await, 8);

        led.set_scroll_speed(0).await;
        assert_eq!(led.scroll_speed().await, 1); // Minimum is 1

        led.set_scroll_speed(15).await;
        assert_eq!(led.scroll_speed().await, 10); // Maximum is 10
    }

    #[tokio::test]
    async fn test_led_display_set_display_mode() {
        let led = LedDisplay::with_default_obis();
        led.set_display_mode(DisplayMode::ScrollLeft).await;
        assert_eq!(led.display_mode().await, DisplayMode::ScrollLeft);
    }

    #[tokio::test]
    async fn test_led_display_show() {
        let led = LedDisplay::with_default_obis();
        led.show("Reading: 123.45".to_string()).await;

        assert_eq!(led.display_text().await, "Reading: 123.45");
        assert!(led.display_enabled().await);
    }

    #[tokio::test]
    async fn test_led_display_clear() {
        let led = LedDisplay::with_default_obis();
        led.show("Test".to_string()).await;
        led.clear().await;

        assert_eq!(led.display_text().await, "");
    }

    #[tokio::test]
    async fn test_led_display_turn_on_off() {
        let led = LedDisplay::with_default_obis();

        led.turn_off().await;
        assert!(!led.display_enabled().await);

        led.turn_on().await;
        assert!(led.display_enabled().await);
    }

    #[tokio::test]
    async fn test_led_display_max_min_brightness() {
        let led = LedDisplay::with_default_obis();

        led.max_brightness().await;
        assert_eq!(led.brightness_level().await, 100);

        led.min_brightness().await;
        assert_eq!(led.brightness_level().await, 1);
    }

    #[tokio::test]
    async fn test_led_display_is_animated() {
        let led = LedDisplay::with_default_obis();
        assert!(!led.is_animated().await);

        led.set_display_mode(DisplayMode::Blink).await;
        assert!(led.is_animated().await);
    }

    #[tokio::test]
    async fn test_led_display_get_attributes() {
        let led = LedDisplay::with_default_obis();

        // Test display_enabled
        let result = led.get_attribute(3, None).await.unwrap();
        match result {
            DataObject::Boolean(enabled) => assert!(enabled),
            _ => panic!("Expected Boolean"),
        }

        // Test brightness_level
        let result = led.get_attribute(4, None).await.unwrap();
        match result {
            DataObject::Unsigned8(level) => assert_eq!(level, 100),
            _ => panic!("Expected Unsigned8"),
        }
    }

    #[tokio::test]
    async fn test_led_display_set_attributes() {
        let led = LedDisplay::with_default_obis();

        led.set_attribute(2, DataObject::OctetString(b"Test".to_vec()), None)
            .await
            .unwrap();
        assert_eq!(led.display_text().await, "Test");

        led.set_attribute(7, DataObject::Enumerate(3), None) // Blink
            .await
            .unwrap();
        assert_eq!(led.display_mode().await, DisplayMode::Blink);
    }

    #[tokio::test]
    async fn test_led_display_read_only_logical_name() {
        let led = LedDisplay::with_default_obis();
        let result = led
            .set_attribute(1, DataObject::OctetString(vec![0, 0, 49, 0, 0, 1]), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_led_display_invalid_attribute() {
        let led = LedDisplay::with_default_obis();
        let result = led.get_attribute(99, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_led_display_invalid_method() {
        let led = LedDisplay::with_default_obis();
        let result = led.invoke_method(1, None, None).await;
        assert!(result.is_err());
    }
}
