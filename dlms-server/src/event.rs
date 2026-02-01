//! Event handling for DLMS/COSEM server
//!
//! This module provides event notification and subscription management:
//! - Event generation and notification
//! - Client subscription management
//! - Event filtering and routing
//! - Push notification support

use dlms_core::{DlmsResult, ObisCode, DataObject};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use std::time::{Duration, Instant};

/// Event severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EventSeverity {
    /// Informational event
    Info = 0,
    /// Warning event
    Warning = 1,
    /// Error event
    Error = 2,
    /// Critical event
    Critical = 3,
}

/// DLMS/COSEM event
///
/// Represents a single event that can be notified to clients.
#[derive(Debug, Clone)]
pub struct DlmsEvent {
    /// Unique event identifier
    pub id: u64,
    /// Event code (identifies event type)
    pub code: u16,
    /// Event severity
    pub severity: EventSeverity,
    /// Source object OBIS code
    pub source_obis: ObisCode,
    /// Event timestamp
    pub timestamp: Instant,
    /// Event data (optional)
    pub data: Option<DataObject>,
    /// Event message (human-readable)
    pub message: String,
}

impl DlmsEvent {
    /// Create a new event
    pub fn new(
        id: u64,
        code: u16,
        severity: EventSeverity,
        source_obis: ObisCode,
        message: String,
    ) -> Self {
        Self {
            id,
            code,
            severity,
            source_obis,
            timestamp: Instant::now(),
            data: None,
            message,
        }
    }

    /// Create an event with data
    pub fn with_data(
        id: u64,
        code: u16,
        severity: EventSeverity,
        source_obis: ObisCode,
        message: String,
        data: DataObject,
    ) -> Self {
        Self {
            id,
            code,
            severity,
            source_obis,
            timestamp: Instant::now(),
            data: Some(data),
            message,
        }
    }

    /// Create an info event
    pub fn info(id: u64, code: u16, source_obis: ObisCode, message: String) -> Self {
        Self::new(id, code, EventSeverity::Info, source_obis, message)
    }

    /// Create a warning event
    pub fn warning(id: u64, code: u16, source_obis: ObisCode, message: String) -> Self {
        Self::new(id, code, EventSeverity::Warning, source_obis, message)
    }

    /// Create an error event
    pub fn error(id: u64, code: u16, source_obis: ObisCode, message: String) -> Self {
        Self::new(id, code, EventSeverity::Error, source_obis, message)
    }

    /// Create a critical event
    pub fn critical(id: u64, code: u16, source_obis: ObisCode, message: String) -> Self {
        Self::new(id, code, EventSeverity::Critical, source_obis, message)
    }
}

/// Event filter
///
/// Used to filter events based on various criteria.
#[derive(Debug, Clone)]
pub struct EventFilter {
    /// Minimum severity to include
    pub min_severity: Option<EventSeverity>,
    /// Event codes to include (empty = all)
    pub event_codes: HashSet<u16>,
    /// Source OBIS codes to include (empty = all)
    pub source_obis: HashSet<ObisCode>,
}

impl EventFilter {
    /// Create a new filter that matches everything
    pub fn all() -> Self {
        Self {
            min_severity: None,
            event_codes: HashSet::new(),
            source_obis: HashSet::new(),
        }
    }

    /// Create a filter with minimum severity
    pub fn with_min_severity(min_severity: EventSeverity) -> Self {
        Self {
            min_severity: Some(min_severity),
            event_codes: HashSet::new(),
            source_obis: HashSet::new(),
        }
    }

    /// Add an event code to the filter
    pub fn add_event_code(&mut self, code: u16) {
        self.event_codes.insert(code);
    }

    /// Add a source OBIS to the filter
    pub fn add_source_obis(&mut self, obis: ObisCode) {
        self.source_obis.insert(obis);
    }

    /// Check if an event matches this filter
    pub fn matches(&self, event: &DlmsEvent) -> bool {
        // Check severity
        if let Some(min_severity) = self.min_severity {
            if event.severity < min_severity {
                return false;
            }
        }

        // Check event codes (if specified)
        if !self.event_codes.is_empty() && !self.event_codes.contains(&event.code) {
            return false;
        }

        // Check source OBIS (if specified)
        if !self.source_obis.is_empty() && !self.source_obis.contains(&event.source_obis) {
            return false;
        }

        true
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::all()
    }
}

/// Event subscription
///
/// Represents a client's subscription to events.
#[derive(Debug, Clone)]
pub struct EventSubscription {
    /// Client SAP address
    pub client_sap: u16,
    /// Event filter
    pub filter: EventFilter,
    /// Whether subscription is active
    pub active: bool,
    /// Subscription start time
    pub subscribed_at: Instant,
}

impl EventSubscription {
    /// Create a new subscription
    pub fn new(client_sap: u16, filter: EventFilter) -> Self {
        Self {
            client_sap,
            filter,
            active: true,
            subscribed_at: Instant::now(),
        }
    }

    /// Deactivate the subscription
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Activate the subscription
    pub fn activate(&mut self) {
        self.active = true;
        self.subscribed_at = Instant::now();
    }

    /// Check if the subscription is active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Event notification
///
/// Represents an event being sent to a client.
#[derive(Debug, Clone)]
pub struct EventNotification {
    /// The event being notified
    pub event: DlmsEvent,
    /// Target client SAP
    pub client_sap: u16,
    /// Whether this notification has been sent
    pub sent: bool,
    /// When the notification was created
    pub created_at: Instant,
}

impl EventNotification {
    /// Create a new event notification
    pub fn new(event: DlmsEvent, client_sap: u16) -> Self {
        Self {
            event,
            client_sap,
            sent: false,
            created_at: Instant::now(),
        }
    }
}

/// Event processor configuration
#[derive(Debug, Clone)]
pub struct EventProcessorConfig {
    /// Maximum number of pending notifications per client
    pub max_pending_notifications: usize,
    /// Event buffer size
    pub event_buffer_size: usize,
    /// Notification timeout
    pub notification_timeout: Duration,
}

impl Default for EventProcessorConfig {
    fn default() -> Self {
        Self {
            max_pending_notifications: 100,
            event_buffer_size: 1000,
            notification_timeout: Duration::from_secs(30),
        }
    }
}

/// Event processor
///
/// Manages event generation, subscription, and notification delivery.
pub struct EventProcessor {
    /// Event subscriptions indexed by client SAP
    subscriptions: Arc<RwLock<HashMap<u16, EventSubscription>>>,
    /// Pending notifications
    notifications: Arc<RwLock<Vec<EventNotification>>>,
    /// Event channel for receiving events
    event_tx: mpsc::Sender<DlmsEvent>,
    /// Next event ID
    next_event_id: Arc<RwLock<u64>>,
    /// Configuration
    config: EventProcessorConfig,
}

impl EventProcessor {
    /// Create a new event processor
    pub fn new() -> Self {
        let (event_tx, _event_rx) = mpsc::channel(1000);

        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            notifications: Arc::new(RwLock::new(Vec::new())),
            event_tx,
            next_event_id: Arc::new(RwLock::new(1)),
            config: EventProcessorConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: EventProcessorConfig) -> Self {
        let (event_tx, _event_rx) = mpsc::channel(config.event_buffer_size);

        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            notifications: Arc::new(RwLock::new(Vec::new())),
            event_tx,
            next_event_id: Arc::new(RwLock::new(1)),
            config,
        }
    }

    /// Get the event sender
    ///
    /// Can be used by other parts of the server to send events.
    pub fn event_sender(&self) -> mpsc::Sender<DlmsEvent> {
        self.event_tx.clone()
    }

    /// Subscribe a client to events
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    /// * `filter` - Event filter to apply
    pub async fn subscribe(&self, client_sap: u16, filter: EventFilter) -> DlmsResult<()> {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(client_sap, EventSubscription::new(client_sap, filter));
        Ok(())
    }

    /// Unsubscribe a client from events
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    pub async fn unsubscribe(&self, client_sap: u16) {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(&client_sap);
    }

    /// Check if a client is subscribed
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    pub async fn is_subscribed(&self, client_sap: u16) -> bool {
        let subscriptions = self.subscriptions.read().await;
        subscriptions
            .get(&client_sap)
            .map(|s| s.is_active())
            .unwrap_or(false)
    }

    /// Get all subscriptions
    pub async fn get_subscriptions(&self) -> Vec<EventSubscription> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.values().cloned().collect()
    }

    /// Get subscription count
    pub async fn subscription_count(&self) -> usize {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.len()
    }

    /// Generate and publish an event
    ///
    /// # Arguments
    /// * `code` - Event code
    /// * `severity` - Event severity
    /// * `source_obis` - Source object OBIS code
    /// * `message` - Event message
    ///
    /// # Returns
    /// The generated event ID
    pub async fn publish_event(
        &self,
        code: u16,
        severity: EventSeverity,
        source_obis: ObisCode,
        message: String,
    ) -> DlmsResult<u64> {
        let id = {
            let mut next_id = self.next_event_id.write().await;
            let id = *next_id;
            *next_id += 1;
            id
        };

        let event = DlmsEvent::new(id, code, severity, source_obis, message);
        self.publish(event).await?;

        Ok(id)
    }

    /// Publish an event to all matching subscribers
    ///
    /// # Arguments
    /// * `event` - The event to publish
    pub async fn publish(&self, event: DlmsEvent) -> DlmsResult<()> {
        let subscriptions = self.subscriptions.read().await;

        for (client_sap, subscription) in subscriptions.iter() {
            if !subscription.is_active() {
                continue;
            }

            if !subscription.filter.matches(&event) {
                continue;
            }

            let notification = EventNotification::new(event.clone(), *client_sap);

            let mut notifications = self.notifications.write().await;
            if notifications.len() < self.config.event_buffer_size {
                notifications.push(notification);
            }
            // Note: If buffer is full, events are dropped (could log warning)
        }

        Ok(())
    }

    /// Get pending notifications for a client
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    ///
    /// # Returns
    /// Vector of pending notifications for the client
    pub async fn get_pending_notifications(&self, client_sap: u16) -> Vec<EventNotification> {
        let mut notifications = self.notifications.write().await;
        let pending: Vec<_> = notifications
            .iter()
            .filter(|n| n.client_sap == client_sap && !n.sent)
            .cloned()
            .collect();

        // Mark as sent
        for notification in &pending {
            if let Some(n) = notifications.iter_mut().find(|n| n.event.id == notification.event.id) {
                n.sent = true;
            }
        }

        pending
    }

    /// Mark notifications as sent
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    /// * `event_ids` - Event IDs to mark as sent
    pub async fn mark_sent(&self, client_sap: u16, event_ids: &[u64]) {
        let mut notifications = self.notifications.write().await;
        for notification in notifications.iter_mut() {
            if notification.client_sap == client_sap && event_ids.contains(&notification.event.id) {
                notification.sent = true;
            }
        }
    }

    /// Clean up old sent notifications
    ///
    /// Removes notifications that were sent more than the timeout ago.
    pub async fn cleanup_old_notifications(&self) -> usize {
        let mut notifications = self.notifications.write().await;
        let initial_count = notifications.len();

        notifications.retain(|n| {
            if n.sent {
                n.created_at.elapsed() < self.config.notification_timeout
            } else {
                true
            }
        });

        initial_count - notifications.len()
    }

    /// Get all pending notifications count
    pub async fn pending_notification_count(&self) -> usize {
        let notifications = self.notifications.read().await;
        notifications.iter().filter(|n| !n.sent).count()
    }

    /// Clear all notifications for a client
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    pub async fn clear_client_notifications(&self, client_sap: u16) {
        let mut notifications = self.notifications.write().await;
        notifications.retain(|n| n.client_sap != client_sap);
    }
}

impl Default for EventProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_obis() -> ObisCode {
        ObisCode::new(1, 1, 1, 8, 0, 255)
    }

    #[test]
    fn test_event_filter() {
        let obis = create_test_obis();
        let event = DlmsEvent::warning(1, 100, obis, "Test event".to_string());

        // Filter that matches everything
        let filter = EventFilter::all();
        assert!(filter.matches(&event));

        // Filter with minimum severity
        let filter = EventFilter::with_min_severity(EventSeverity::Error);
        assert!(!filter.matches(&event)); // Warning < Error

        let filter = EventFilter::with_min_severity(EventSeverity::Info);
        assert!(filter.matches(&event)); // Warning >= Info
    }

    #[test]
    fn test_event_filter_codes() {
        let obis = create_test_obis();
        let event = DlmsEvent::warning(1, 100, obis, "Test event".to_string());

        let mut filter = EventFilter::all();
        filter.add_event_code(100);
        assert!(filter.matches(&event));

        let mut filter = EventFilter::all();
        filter.add_event_code(200);
        assert!(!filter.matches(&event));
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let subscription = EventSubscription::new(1, EventFilter::all());
        assert!(subscription.is_active());

        let mut sub = subscription;
        sub.deactivate();
        assert!(!sub.is_active());

        sub.activate();
        assert!(sub.is_active());
    }

    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let processor = EventProcessor::new();

        // Subscribe client 1
        processor.subscribe(1, EventFilter::all()).await.unwrap();
        assert!(processor.is_subscribed(1).await);
        assert_eq!(processor.subscription_count().await, 1);

        // Unsubscribe
        processor.unsubscribe(1).await;
        assert!(!processor.is_subscribed(1).await);
        assert_eq!(processor.subscription_count().await, 0);
    }

    #[tokio::test]
    async fn test_publish_event() {
        let processor = EventProcessor::new();
        let obis = create_test_obis();

        // Subscribe client 1
        processor.subscribe(1, EventFilter::all()).await.unwrap();

        // Publish event
        let id = processor
            .publish_event(100, EventSeverity::Warning, obis, "Test event".to_string())
            .await
            .unwrap();

        // Check pending notifications
        let pending = processor.get_pending_notifications(1).await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].event.id, id);
        assert!(!pending[0].sent);
    }

    #[tokio::test]
    async fn test_event_filtering() {
        let processor = EventProcessor::new();
        let obis1 = ObisCode::new(1, 1, 1, 8, 0, 255);
        let obis2 = ObisCode::new(1, 1, 1, 9, 0, 255);

        // Subscribe client 1 with filter for obis1 only
        let mut filter = EventFilter::all();
        filter.add_source_obis(obis1);
        processor.subscribe(1, filter).await.unwrap();

        // Publish event from obis1
        processor
            .publish_event(100, EventSeverity::Warning, obis1, "Event 1".to_string())
            .await
            .unwrap();

        // Publish event from obis2
        processor
            .publish_event(200, EventSeverity::Warning, obis2, "Event 2".to_string())
            .await
            .unwrap();

        // Only obis1 event should be received
        let pending = processor.get_pending_notifications(1).await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].event.code, 100);
    }

    #[tokio::test]
    async fn test_mark_sent() {
        let processor = EventProcessor::new();
        let obis = create_test_obis();

        processor.subscribe(1, EventFilter::all()).await.unwrap();

        let id = processor
            .publish_event(100, EventSeverity::Warning, obis, "Test".to_string())
            .await
            .unwrap();

        // Get pending (marks as sent)
        let pending = processor.get_pending_notifications(1).await;
        assert_eq!(pending.len(), 1);
        assert!(!pending[0].sent);

        // Mark as sent
        processor.mark_sent(1, &[id]).await;

        // Should be marked
        let all_notifications = processor.notifications.read().await;
        let notification = all_notifications.iter().find(|n| n.event.id == id).unwrap();
        assert!(notification.sent);
    }

    #[tokio::test]
    async fn test_cleanup_old_notifications() {
        let processor = EventProcessor::with_config(EventProcessorConfig {
            notification_timeout: Duration::from_millis(100),
            ..Default::default()
        });
        let obis = create_test_obis();

        processor.subscribe(1, EventFilter::all()).await.unwrap();

        processor
            .publish_event(100, EventSeverity::Warning, obis, "Test".to_string())
            .await
            .unwrap();

        // Mark as sent
        let pending = processor.get_pending_notifications(1).await;
        processor.mark_sent(1, &[pending[0].event.id]).await;

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Cleanup
        let removed = processor.cleanup_old_notifications().await;
        assert_eq!(removed, 1);
    }
}
