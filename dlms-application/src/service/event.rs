//! Event Notification Service implementation for DLMS/COSEM
//!
//! This module provides high-level Event Notification service functionality for handling
//! asynchronous event notifications from COSEM objects.
//!
//! # Design Philosophy
//!
//! Event notifications are unconfirmed services - the server sends them without expecting
//! a response. This makes them different from GET/SET/ACTION services, which are confirmed
//! services with request/response pairs.
//!
//! # Features
//! - Event notification decoding
//! - Event filtering and processing
//! - Event handler registration
//!
//! # Usage
//!
//! ```rust,no_run
//! use dlms_application::service::EventNotificationService;
//! use dlms_application::pdu::EventNotification;
//!
//! // Create an event notification service
//! let service = EventNotificationService::new();
//!
//! // Decode an event notification from received bytes
//! let notification = service.decode_notification(&bytes)?;
//!
//! // Process the notification
//! service.process_notification(&notification)?;
//! ```

use crate::pdu::{EventNotification, CosemAttributeDescriptor};
use dlms_core::{DlmsError, DlmsResult, DataObject, ObisCode};

/// Event handler function type
///
/// Called when an event notification is received. The handler receives:
/// - `time`: Optional timestamp when the event occurred
/// - `attribute_descriptor`: The attribute that triggered the event
/// - `attribute_value`: The value of the attribute at the time of the event
///
/// # Return Value
/// Returns `Ok(())` if the event was handled successfully, or an error if processing failed.
pub type EventHandler = fn(
    time: Option<dlms_core::datatypes::CosemDateTime>,
    attribute_descriptor: &CosemAttributeDescriptor,
    attribute_value: &DataObject,
) -> DlmsResult<()>;

/// Event Notification Service for DLMS/COSEM
///
/// Provides high-level interface for handling event notifications, which are
/// asynchronous messages sent by the server when certain conditions are met.
///
/// # Why a Separate Service Layer?
/// Event notifications are fundamentally different from other services:
/// - **Unconfirmed**: No response is sent back to the server
/// - **Asynchronous**: Can arrive at any time, not in response to a request
/// - **Event-Driven**: Triggered by conditions in the server, not client requests
///
/// The service layer provides:
/// - **Decoding**: Parse event notifications from received bytes
/// - **Filtering**: Optionally filter events by attribute or OBIS code
/// - **Handling**: Register handlers for specific event types
/// - **Processing**: Extract and process event data
///
/// # Event Notification Flow
/// 1. Server detects an event condition (e.g., attribute value change, alarm condition)
/// 2. Server sends EventNotification PDU to client
/// 3. Client receives and decodes the notification
/// 4. Client processes the notification (handler, logging, etc.)
/// 5. No response is sent (unconfirmed service)
///
/// # Optimization Considerations
/// - Event notifications are typically infrequent, so performance is less critical
/// - Event handlers should be lightweight to avoid blocking the main communication channel
/// - Consider using a queue or ring buffer for high-frequency event scenarios
/// - Future enhancement: Add event filtering and subscription management
#[derive(Debug, Clone)]
pub struct EventNotificationService {
    /// Optional event handler function
    handler: Option<EventHandler>,
    /// Optional filter: only process events from this OBIS code
    filter_obis: Option<ObisCode>,
}

impl EventNotificationService {
    /// Create a new Event Notification service
    pub fn new() -> Self {
        Self {
            handler: None,
            filter_obis: None,
        }
    }

    /// Create a new Event Notification service with a handler
    ///
    /// # Arguments
    /// * `handler` - Function to call when an event notification is received
    pub fn with_handler(handler: EventHandler) -> Self {
        Self {
            handler: Some(handler),
            filter_obis: None,
        }
    }

    /// Set an event handler
    ///
    /// # Arguments
    /// * `handler` - Function to call when an event notification is received
    pub fn set_handler(&mut self, handler: EventHandler) {
        self.handler = Some(handler);
    }

    /// Set an OBIS code filter
    ///
    /// # Arguments
    /// * `obis` - Only process events from this OBIS code (None to disable filtering)
    ///
    /// # Why Filtering?
    /// In systems with many COSEM objects, filtering allows the client to only process
    /// events from specific objects, reducing processing overhead and improving responsiveness.
    pub fn set_filter(&mut self, obis: Option<ObisCode>) {
        self.filter_obis = obis;
    }

    /// Decode an event notification from bytes
    ///
    /// # Arguments
    /// * `data` - Raw bytes containing the EventNotification PDU
    ///
    /// # Returns
    /// Decoded `EventNotification` structure
    ///
    /// # Errors
    /// Returns error if the data cannot be decoded as a valid EventNotification PDU
    pub fn decode_notification(&self, data: &[u8]) -> DlmsResult<EventNotification> {
        EventNotification::decode(data)
    }

    /// Process an event notification
    ///
    /// This method:
    /// 1. Checks if the event matches the filter (if set)
    /// 2. Calls the registered handler (if set)
    /// 3. Returns the notification for further processing
    ///
    /// # Arguments
    /// * `notification` - The event notification to process
    ///
    /// # Returns
    /// The notification (for chaining or further processing)
    ///
    /// # Errors
    /// Returns error if the handler returns an error
    pub fn process_notification<'a>(&'a self, notification: &'a EventNotification) -> DlmsResult<&'a EventNotification> {
        // Check filter if set
        if let Some(ref filter_obis) = self.filter_obis {
            match &notification.cosem_attribute_descriptor {
                crate::pdu::CosemAttributeDescriptor::LogicalName(ln_ref) => {
                    if &ln_ref.instance_id != filter_obis {
                        // Event doesn't match filter, skip processing
                        return Ok(notification);
                    }
                }
                crate::pdu::CosemAttributeDescriptor::ShortName { .. } => {
                    // Short name addressing doesn't have OBIS code, skip filter check
                    // or implement short name filtering if needed
                }
            }
        }

        // Call handler if set
        if let Some(handler) = self.handler {
            handler(
                notification.time.clone(),
                &notification.cosem_attribute_descriptor,
                &notification.attribute_value,
            )?;
        }

        Ok(notification)
    }

    /// Decode and process an event notification in one step
    ///
    /// # Arguments
    /// * `data` - Raw bytes containing the EventNotification PDU
    ///
    /// # Returns
    /// Decoded `EventNotification` structure
    ///
    /// # Errors
    /// Returns error if decoding fails or if the handler returns an error
    pub fn decode_and_process(&self, data: &[u8]) -> DlmsResult<EventNotification> {
        let notification = self.decode_notification(data)?;
        self.process_notification(&notification)?;
        Ok(notification)
    }

    /// Extract the attribute value from an event notification
    ///
    /// # Arguments
    /// * `notification` - The event notification
    ///
    /// # Returns
    /// The attribute value that triggered the event
    pub fn extract_attribute_value(notification: &EventNotification) -> &DataObject {
        &notification.attribute_value
    }

    /// Extract the OBIS code from an event notification
    ///
    /// # Arguments
    /// * `notification` - The event notification
    ///
    /// # Returns
    /// The OBIS code of the object that triggered the event, or None if using short name addressing
    pub fn extract_obis_code(notification: &EventNotification) -> Option<&ObisCode> {
        match &notification.cosem_attribute_descriptor {
            crate::pdu::CosemAttributeDescriptor::LogicalName(ln_ref) => Some(&ln_ref.instance_id),
            crate::pdu::CosemAttributeDescriptor::ShortName { .. } => None,
        }
    }

    /// Extract the class ID from an event notification
    ///
    /// # Arguments
    /// * `notification` - The event notification
    ///
    /// # Returns
    /// The class ID of the object that triggered the event
    pub fn extract_class_id(notification: &EventNotification) -> u16 {
        match &notification.cosem_attribute_descriptor {
            crate::pdu::CosemAttributeDescriptor::LogicalName(ln_ref) => ln_ref.class_id,
            crate::pdu::CosemAttributeDescriptor::ShortName { class_id, .. } => *class_id,
        }
    }

    /// Extract the attribute ID from an event notification
    ///
    /// # Arguments
    /// * `notification` - The event notification
    ///
    /// # Returns
    /// The attribute ID that triggered the event
    pub fn extract_attribute_id(notification: &EventNotification) -> u8 {
        match &notification.cosem_attribute_descriptor {
            crate::pdu::CosemAttributeDescriptor::LogicalName(ln_ref) => ln_ref.id,
            crate::pdu::CosemAttributeDescriptor::ShortName { reference, .. } => reference.id,
        }
    }
}

impl Default for EventNotificationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdu::CosemAttributeDescriptor;
    use dlms_core::datatypes::CosemDateTime;

    #[test]
    fn test_event_notification_service_new() {
        let service = EventNotificationService::new();
        assert!(service.handler.is_none());
        assert!(service.filter_obis.is_none());
    }

    #[test]
    fn test_event_notification_service_decode() {
        let service = EventNotificationService::new();
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let attr_desc = CosemAttributeDescriptor::new_logical_name(1, obis, 2).unwrap();
        let value = DataObject::new_unsigned32(12345);
        let notification = EventNotification::new(None, attr_desc, value);
        
        let encoded = notification.encode().unwrap();
        let decoded = service.decode_notification(&encoded).unwrap();
        
        assert_eq!(decoded.attribute_value, notification.attribute_value);
    }

    #[test]
    fn test_event_notification_service_extract() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let attr_desc = CosemAttributeDescriptor::new_logical_name(1, obis, 2).unwrap();
        let value = DataObject::new_unsigned32(12345);
        let notification = EventNotification::new(None, attr_desc, value);
        
        assert_eq!(EventNotificationService::extract_class_id(&notification), 1);
        assert_eq!(EventNotificationService::extract_attribute_id(&notification), 2);
        assert_eq!(EventNotificationService::extract_obis_code(&notification), Some(&obis));
    }
}
