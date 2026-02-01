//! Event notification handling for DLMS/COSEM client
//!
//! This module provides functionality for receiving and processing
//! event notifications from DLMS/COSEM devices.

use dlms_core::{DlmsError, DlmsResult, DataObject, ObisCode};
use dlms_application::pdu::DataNotification;
use dlms_application::sn_pdu::InformationReportRequest;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

/// Event notification received from a meter
#[derive(Debug, Clone)]
pub struct EventNotification {
    /// Source OBIS code (who sent this event)
    pub source: ObisCode,
    /// Attribute ID that triggered the event
    pub attribute_id: u8,
    /// Event data value
    pub value: DataObject,
    /// Timestamp when the event was received (client-side time)
    pub received_at: std::time::SystemTime,
    /// Optional event timestamp (device-side time, if available)
    pub event_timestamp: Option<std::time::SystemTime>,
}

impl EventNotification {
    /// Create a new event notification
    pub fn new(source: ObisCode, attribute_id: u8, value: DataObject) -> Self {
        Self {
            source,
            attribute_id,
            value,
            received_at: std::time::SystemTime::now(),
            event_timestamp: None,
        }
    }

    /// Create with event timestamp
    pub fn with_timestamp(
        source: ObisCode,
        attribute_id: u8,
        value: DataObject,
        event_timestamp: std::time::SystemTime,
    ) -> Self {
        Self {
            source,
            attribute_id,
            value,
            received_at: std::time::SystemTime::now(),
            event_timestamp: Some(event_timestamp),
        }
    }
}

/// Callback function type for event notifications
pub type EventCallback = Arc<dyn Fn(EventNotification) -> () + Send + Sync>;

/// Subscription filter for event notifications
#[derive(Debug, Clone)]
pub struct EventFilter {
    /// OBIS code filter (None means all)
    pub obis_filter: Option<ObisCode>,
    /// Attribute ID filter (None means all)
    pub attribute_filter: Option<u8>,
    /// Minimum value threshold (for numeric values)
    pub min_value: Option<DataObject>,
    /// Maximum value threshold (for numeric values)
    pub max_value: Option<DataObject>,
}

impl EventFilter {
    /// Create a new filter with OBIS code
    pub fn obis(obis: ObisCode) -> Self {
        Self {
            obis_filter: Some(obis),
            attribute_filter: None,
            min_value: None,
            max_value: None,
        }
    }

    /// Create a new filter with OBIS code and attribute ID
    pub fn attribute(obis: ObisCode, attribute_id: u8) -> Self {
        Self {
            obis_filter: Some(obis),
            attribute_filter: Some(attribute_id),
            min_value: None,
            max_value: None,
        }
    }

    /// Create a wildcard filter (matches all events)
    pub fn wildcard() -> Self {
        Self {
            obis_filter: None,
            attribute_filter: None,
            min_value: None,
            max_value: None,
        }
    }

    /// Add minimum value threshold
    pub fn with_min(mut self, min: DataObject) -> Self {
        self.min_value = Some(min);
        self
    }

    /// Add maximum value threshold
    pub fn with_max(mut self, max: DataObject) -> Self {
        self.max_value = Some(max);
        self
    }

    /// Check if an event matches this filter
    pub fn matches(&self, event: &EventNotification) -> bool {
        // Check OBIS filter
        if let Some(obis) = &self.obis_filter {
            if &event.source != obis {
                return false;
            }
        }

        // Check attribute filter
        if let Some(attr_id) = self.attribute_filter {
            if event.attribute_id != attr_id {
                return false;
            }
        }

        // Check min value (only for numeric types)
        if let Some(min) = &self.min_value {
            if !self.check_threshold(&event.value, min, true) {
                return false;
            }
        }

        // Check max value (only for numeric types)
        if let Some(max) = &self.max_value {
            if !self.check_threshold(&event.value, max, false) {
                return false;
            }
        }

        true
    }

    /// Check if a value passes a threshold
    fn check_threshold(&self, value: &DataObject, threshold: &DataObject, is_min: bool) -> bool {
        match (value, threshold) {
            (DataObject::Unsigned8(v), DataObject::Unsigned8(t)) => {
                if is_min { *v >= *t } else { *v <= *t }
            }
            (DataObject::Unsigned16(v), DataObject::Unsigned16(t)) => {
                if is_min { *v >= *t } else { *v <= *t }
            }
            (DataObject::Unsigned32(v), DataObject::Unsigned32(t)) => {
                if is_min { *v >= *t } else { *v <= *t }
            }
            (DataObject::Unsigned64(v), DataObject::Unsigned64(t)) => {
                if is_min { *v >= *t } else { *v <= *t }
            }
            (DataObject::Integer8(v), DataObject::Integer8(t)) => {
                if is_min { *v >= *t } else { *v <= *t }
            }
            (DataObject::Integer16(v), DataObject::Integer16(t)) => {
                if is_min { *v >= *t } else { *v <= *t }
            }
            (DataObject::Integer32(v), DataObject::Integer32(t)) => {
                if is_min { *v >= *t } else { *v <= *t }
            }
            (DataObject::Integer64(v), DataObject::Integer64(t)) => {
                if is_min { *v >= *t } else { *v <= *t }
            }
            _ => true, // Non-numeric values pass threshold checks
        }
    }
}

/// Subscription information
struct Subscription {
    /// Filter for this subscription
    filter: EventFilter,
    /// Callback to invoke when event matches
    callback: EventCallback,
    /// Whether this subscription is active
    active: bool,
}

/// Event notification handler
///
/// Manages event subscriptions and dispatches notifications
/// to registered callbacks.
pub struct EventHandler {
    /// Subscriptions indexed by subscription ID
    subscriptions: Arc<RwLock<HashMap<u64, Subscription>>>,
    /// Next subscription ID
    next_id: Arc<RwLock<u64>>,
    /// Event notification sender for async processing
    event_tx: mpsc::UnboundedSender<EventNotification>,
    /// Event statistics
    stats: Arc<RwLock<EventStats>>,
}

/// Event handler statistics
#[derive(Debug, Clone, Default)]
pub struct EventStats {
    /// Total events received
    pub total_received: u64,
    /// Total events processed
    pub total_processed: u64,
    /// Total events filtered (not matching any subscription)
    pub total_filtered: u64,
    /// Total callbacks invoked
    pub callbacks_invoked: u64,
    /// Current active subscriptions
    pub active_subscriptions: usize,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new() -> Self {
        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<EventNotification>();
        let subscriptions = Arc::new(RwLock::new(HashMap::<u64, Subscription>::new()));
        let stats = Arc::new(RwLock::new(EventStats::default()));

        // Spawn event processing task
        let subs_clone = subscriptions.clone();
        let stats_clone = stats.clone();
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                let mut s = stats_clone.write().await;
                s.total_received += 1;
                drop(s);

                // Process subscriptions
                let mut matched = false;
                let mut callbacks_invoked = 0;

                {
                    let subs = subs_clone.read().await;
                    for (_, sub) in subs.iter() {
                        if sub.active && sub.filter.matches(&event) {
                            matched = true;
                            // Invoke callback
                            (sub.callback)(event.clone());
                            callbacks_invoked += 1;
                        }
                    }
                }

                let mut s = stats_clone.write().await;
                s.total_processed += 1;
                if matched {
                    s.callbacks_invoked += callbacks_invoked;
                } else {
                    s.total_filtered += 1;
                }
            }
        });

        Self {
            subscriptions,
            next_id: Arc::new(RwLock::new(1)),
            event_tx,
            stats,
        }
    }

    /// Subscribe to event notifications
    ///
    /// Returns a subscription ID that can be used to unsubscribe later.
    ///
    /// # Arguments
    /// * `filter` - Event filter to match notifications
    /// * `callback` - Function to call when a matching event is received
    pub async fn subscribe(&self, filter: EventFilter, callback: EventCallback) -> u64 {
        let id = {
            let mut next_id = self.next_id.write().await;
            let id = *next_id;
            *next_id = next_id.wrapping_add(1);
            id
        };

        let subscription = Subscription {
            filter,
            callback,
            active: true,
        };

        let mut subs = self.subscriptions.write().await;
        subs.insert(id, subscription);

        // Update stats
        let mut stats = self.stats.write().await;
        stats.active_subscriptions = subs.len();

        id
    }

    /// Unsubscribe from event notifications
    ///
    /// # Arguments
    /// * `subscription_id` - ID returned from `subscribe()`
    pub async fn unsubscribe(&self, subscription_id: u64) -> bool {
        let mut subs = self.subscriptions.write().await;
        let removed = subs.remove(&subscription_id).is_some();

        if removed {
            let mut stats = self.stats.write().await;
            stats.active_subscriptions = subs.len();
        }

        removed
    }

    /// Pause a subscription (stop receiving events but keep subscription)
    pub async fn pause_subscription(&self, subscription_id: u64) -> bool {
        let mut subs = self.subscriptions.write().await;
        if let Some(sub) = subs.get_mut(&subscription_id) {
            sub.active = false;
            true
        } else {
            false
        }
    }

    /// Resume a paused subscription
    pub async fn resume_subscription(&self, subscription_id: u64) -> bool {
        let mut subs = self.subscriptions.write().await;
        if let Some(sub) = subs.get_mut(&subscription_id) {
            sub.active = true;
            true
        } else {
            false
        }
    }

    /// Process a DataNotification PDU
    ///
    /// This extracts event data from a DataNotification and dispatches
    /// it to matching subscriptions.
    pub fn handle_data_notification(&self, notification: DataNotification) -> DlmsResult<()> {
        // For DataNotification, we need to extract the variable name
        // This is typically a COSEM attribute reference
        let (obis, attr_id) = Self::extract_variable_info(&notification)?;

        let event = EventNotification::new(obis, attr_id, notification.data_value);

        self.event_tx.send(event)
            .map_err(|e| DlmsError::Protocol(format!("Failed to send event: {}", e)))?;

        Ok(())
    }

    /// Process an InformationReport PDU (SN addressing)
    ///
    /// This extracts event data from an InformationReport and dispatches
    /// it to matching subscriptions.
    pub fn handle_information_report(&self, report: InformationReportRequest) -> DlmsResult<()> {
        // For InformationReport, extract the variable reference and value
        let (obis, attr_id) = Self::extract_info_variable_info(&report)?;

        // InformationReport has a single data value
        let event = EventNotification::new(obis, attr_id, report.data);

        self.event_tx.send(event)
            .map_err(|e| DlmsError::Protocol(format!("Failed to send event: {}", e)))?;

        Ok(())
    }

    /// Manually emit an event (for testing or internal use)
    pub fn emit_event(&self, event: EventNotification) -> DlmsResult<()> {
        self.event_tx.send(event)
            .map_err(|e| DlmsError::Protocol(format!("Failed to send event: {}", e)))?;

        Ok(())
    }

    /// Get event statistics
    pub async fn stats(&self) -> EventStats {
        self.stats.read().await.clone()
    }

    /// Clear all statistics
    pub async fn clear_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = EventStats::default();
        stats.active_subscriptions = self.subscriptions.read().await.len();
    }

    /// Unsubscribe all subscriptions
    pub async fn unsubscribe_all(&self) {
        let mut subs = self.subscriptions.write().await;
        subs.clear();
        let mut stats = self.stats.write().await;
        stats.active_subscriptions = 0;
    }

    /// Extract variable information from DataNotification
    fn extract_variable_info(notification: &DataNotification) -> DlmsResult<(ObisCode, u8)> {
        // Try to get OBIS and attribute ID from variable_name_specification
        if let Some(var_spec) = &notification.variable_name_specification {
            match var_spec {
                dlms_application::pdu::VariableNameSpecification::CosemAttribute(attr) => {
                    match attr {
                        dlms_application::pdu::CosemAttributeDescriptor::LogicalName(ln_ref) => {
                            Ok((ln_ref.instance_id, ln_ref.id))
                        }
                        dlms_application::pdu::CosemAttributeDescriptor::ShortName { reference: sn_ref, .. } => {
                            // For Short Name, we don't have OBIS code directly
                            // Return a default OBIS and use the id from SN reference
                            Ok((ObisCode::new(0, 0, 1, 0, 0, 255), sn_ref.id))
                        }
                    }
                }
                dlms_application::pdu::VariableNameSpecification::Structure(_) => {
                    // Complex structure - need to parse further
                    // For now, return a default
                    Ok((ObisCode::new(0, 0, 0, 0, 0, 0), 2))
                }
            }
        } else {
            // No variable name spec - use default
            Ok((ObisCode::new(0, 0, 0, 0, 0, 0), 2))
        }
    }

    /// Extract variable information from InformationReport
    fn extract_info_variable_info(_report: &InformationReportRequest) -> DlmsResult<(ObisCode, u8)> {
        // InformationReport uses short name addressing
        // We need to map the base_name to OBIS code
        // For now, return a default
        Ok((ObisCode::new(0, 0, 1, 0, 0, 255), 2))
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Event listener configuration
#[derive(Debug, Clone)]
pub struct EventListenerConfig {
    /// Buffer size for event channel
    pub buffer_size: usize,
    /// Whether to enable automatic event processing
    pub auto_process: bool,
    /// Maximum number of subscriptions (0 = unlimited)
    pub max_subscriptions: usize,
}

impl Default for EventListenerConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            auto_process: true,
            max_subscriptions: 0,
        }
    }
}

/// Dedicated event listener for receiving events from a connection
///
/// This provides a channel-based API for receiving events asynchronously.
pub struct EventListener {
    /// Event receiver channel
    event_rx: mpsc::UnboundedReceiver<EventNotification>,
    /// Filter for this listener
    filter: EventFilter,
    /// Whether listener is active
    active: Arc<RwLock<bool>>,
}

impl EventListener {
    /// Create a new event listener
    pub fn new(filter: EventFilter) -> (Self, mpsc::UnboundedSender<EventNotification>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let listener = Self {
            event_rx,
            filter,
            active: Arc::new(RwLock::new(true)),
        };

        (listener, event_tx)
    }

    /// Receive the next event (blocking)
    pub async fn recv(&mut self) -> Option<EventNotification> {
        while self.is_active().await {
            match self.event_rx.recv().await {
                Some(event) => {
                    if self.filter.matches(&event) {
                        return Some(event);
                    }
                    // Event doesn't match filter, continue waiting
                }
                None => return None, // Channel closed
            }
        }
        None
    }

    /// Try to receive an event without blocking
    pub fn try_recv(&mut self) -> Option<EventNotification> {
        if !self.is_active_blocking() {
            return None;
        }

        while let Ok(event) = self.event_rx.try_recv() {
            if self.filter.matches(&event) {
                return Some(event);
            }
            // Event doesn't match filter, try next
        }
        None
    }

    /// Check if listener is active
    async fn is_active(&self) -> bool {
        *self.active.read().await
    }

    /// Check if listener is active (blocking version)
    fn is_active_blocking(&self) -> bool {
        // Use blocking try_read for non-async context
        // This is a simplified version - in real code you'd want proper async handling
        true
    }

    /// Stop the listener
    pub async fn stop(&self) {
        *self.active.write().await = false;
    }

    /// Resume the listener
    pub async fn start(&self) {
        *self.active.write().await = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_event_filter_wildcard() {
        let filter = EventFilter::wildcard();
        let event = EventNotification::new(
            ObisCode::new(1, 1, 1, 1, 1, 1),
            2,
            DataObject::Unsigned8(42),
        );
        assert!(filter.matches(&event));
    }

    #[test]
    fn test_event_filter_obis() {
        let filter = EventFilter::obis(ObisCode::new(1, 1, 1, 1, 1, 1));
        let event = EventNotification::new(
            ObisCode::new(1, 1, 1, 1, 1, 1),
            2,
            DataObject::Unsigned8(42),
        );
        assert!(filter.matches(&event));

        let event2 = EventNotification::new(
            ObisCode::new(2, 2, 2, 2, 2, 2),
            2,
            DataObject::Unsigned8(42),
        );
        assert!(!filter.matches(&event2));
    }

    #[test]
    fn test_event_filter_attribute() {
        let filter = EventFilter::attribute(ObisCode::new(1, 1, 1, 1, 1, 1), 2);
        let event = EventNotification::new(
            ObisCode::new(1, 1, 1, 1, 1, 1),
            2,
            DataObject::Unsigned8(42),
        );
        assert!(filter.matches(&event));

        let event2 = EventNotification::new(
            ObisCode::new(1, 1, 1, 1, 1, 1),
            3, // Different attribute
            DataObject::Unsigned8(42),
        );
        assert!(!filter.matches(&event2));
    }

    #[test]
    fn test_event_filter_min_value() {
        let filter = EventFilter::wildcard().with_min(DataObject::Unsigned8(50));
        let event = EventNotification::new(
            ObisCode::new(1, 1, 1, 1, 1, 1),
            2,
            DataObject::Unsigned8(42),
        );
        assert!(!filter.matches(&event)); // 42 < 50

        let event2 = EventNotification::new(
            ObisCode::new(1, 1, 1, 1, 1, 1),
            2,
            DataObject::Unsigned8(60),
        );
        assert!(filter.matches(&event2)); // 60 >= 50
    }

    #[test]
    fn test_event_filter_max_value() {
        let filter = EventFilter::wildcard().with_max(DataObject::Unsigned8(50));
        let event = EventNotification::new(
            ObisCode::new(1, 1, 1, 1, 1, 1),
            2,
            DataObject::Unsigned8(60),
        );
        assert!(!filter.matches(&event)); // 60 > 50

        let event2 = EventNotification::new(
            ObisCode::new(1, 1, 1, 1, 1, 1),
            2,
            DataObject::Unsigned8(42),
        );
        assert!(filter.matches(&event2)); // 42 <= 50
    }

    #[tokio::test]
    async fn test_event_handler_subscribe() {
        let handler = EventHandler::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let callback = Arc::new(move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let filter = EventFilter::wildcard();
        handler.subscribe(filter, callback).await;

        let stats = handler.stats().await;
        assert_eq!(stats.active_subscriptions, 1);
    }

    #[tokio::test]
    async fn test_event_handler_unsubscribe() {
        let handler = EventHandler::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let callback = Arc::new(move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let filter = EventFilter::wildcard();
        let id = handler.subscribe(filter, callback).await;

        assert!(handler.unsubscribe(id).await);
        let stats = handler.stats().await;
        assert_eq!(stats.active_subscriptions, 0);
    }

    #[tokio::test]
    async fn test_event_handler_emit() {
        let handler = EventHandler::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let callback: EventCallback = Arc::new(move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        handler.subscribe(EventFilter::wildcard(), callback).await;

        let event = EventNotification::new(
            ObisCode::new(1, 1, 1, 1, 1, 1),
            2,
            DataObject::Unsigned8(42),
        );

        handler.emit_event(event).unwrap();

        // Give some time for the event to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        assert_eq!(counter.load(Ordering::SeqCst), 1);

        let stats = handler.stats().await;
        assert_eq!(stats.total_received, 1);
        assert_eq!(stats.total_processed, 1);
        assert_eq!(stats.callbacks_invoked, 1);
    }

    #[tokio::test]
    async fn test_event_handler_filtering() {
        let handler = EventHandler::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let callback: EventCallback = Arc::new(move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Only subscribe to OBIS 1.1.1.1.1.1
        handler.subscribe(
            EventFilter::obis(ObisCode::new(1, 1, 1, 1, 1, 1)),
            callback,
        ).await;

        // Send event with matching OBIS
        let event1 = EventNotification::new(
            ObisCode::new(1, 1, 1, 1, 1, 1),
            2,
            DataObject::Unsigned8(42),
        );
        handler.emit_event(event1).unwrap();

        // Send event with different OBIS
        let event2 = EventNotification::new(
            ObisCode::new(2, 2, 2, 2, 2, 2),
            2,
            DataObject::Unsigned8(42),
        );
        handler.emit_event(event2).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Only the matching event should trigger callback
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        let stats = handler.stats().await;
        assert_eq!(stats.total_received, 2);
        assert_eq!(stats.callbacks_invoked, 1);
        assert_eq!(stats.total_filtered, 1);
    }

    #[tokio::test]
    async fn test_event_handler_pause_resume() {
        let handler = EventHandler::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let callback: EventCallback = Arc::new(move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let filter = EventFilter::wildcard();
        let id = handler.subscribe(filter, callback).await;

        // Pause subscription
        handler.pause_subscription(id).await;

        let event = EventNotification::new(
            ObisCode::new(1, 1, 1, 1, 1, 1),
            2,
            DataObject::Unsigned8(42),
        );
        handler.emit_event(event).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 0);

        // Resume subscription
        handler.resume_subscription(id).await;

        let event2 = EventNotification::new(
            ObisCode::new(1, 1, 1, 1, 1, 1),
            2,
            DataObject::Unsigned8(43),
        );
        handler.emit_event(event2).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_event_handler_clear_stats() {
        let handler = EventHandler::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let callback: EventCallback = Arc::new(move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        handler.subscribe(EventFilter::wildcard(), callback).await;

        let event = EventNotification::new(
            ObisCode::new(1, 1, 1, 1, 1, 1),
            2,
            DataObject::Unsigned8(42),
        );
        handler.emit_event(event).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        handler.clear_stats().await;

        let stats = handler.stats().await;
        assert_eq!(stats.total_received, 0);
        assert_eq!(stats.callbacks_invoked, 0);
    }

    #[tokio::test]
    async fn test_event_listener() {
        let filter = EventFilter::obis(ObisCode::new(1, 1, 1, 1, 1, 1));
        let (mut listener, sender) = EventListener::new(filter);

        // Send an event
        let event = EventNotification::new(
            ObisCode::new(1, 1, 1, 1, 1, 1),
            2,
            DataObject::Unsigned8(42),
        );
        sender.send(event).unwrap();

        // Receive the event with timeout
        let received = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            listener.recv(),
        ).await;

        assert!(received.is_ok());
        let received_event = received.unwrap().unwrap();
        assert_eq!(received_event.attribute_id, 2);
    }
}
