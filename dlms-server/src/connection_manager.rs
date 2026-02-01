//! Connection manager for tracking and managing active client connections
//!
//! This module provides functionality to:
//! - Track all active client connections
//! - Limit maximum concurrent connections
//! - Monitor connection health
//! - Clean up stale connections

use crate::server::AssociationContext;
use dlms_core::{DlmsError, DlmsResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Connection information
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Client SAP address
    pub client_sap: u16,
    /// Client IP address (if available)
    pub client_address: Option<String>,
    /// Connection start time
    pub connected_at: Instant,
    /// Last activity time
    pub last_activity: Instant,
    /// Association context
    pub association: AssociationContext,
}

impl ConnectionInfo {
    /// Create a new connection info
    pub fn new(
        client_sap: u16,
        client_address: Option<String>,
        association: AssociationContext,
    ) -> Self {
        let now = Instant::now();
        Self {
            client_sap,
            client_address,
            connected_at: now,
            last_activity: now,
            association,
        }
    }

    /// Update last activity time
    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Get connection duration
    pub fn duration(&self) -> Duration {
        self.connected_at.elapsed()
    }

    /// Get idle time
    pub fn idle_time(&self) -> Duration {
        self.last_activity.elapsed()
    }

    /// Check if connection is stale (idle longer than timeout)
    pub fn is_stale(&self, timeout: Duration) -> bool {
        self.idle_time() > timeout
    }
}

/// Connection manager
///
/// Manages all active client connections with features for:
/// - Connection tracking
/// - Connection limiting
/// - Stale connection cleanup
pub struct ConnectionManager {
    /// Active connections indexed by client SAP
    connections: Arc<RwLock<HashMap<u16, ConnectionInfo>>>,

    /// Maximum number of concurrent connections
    max_connections: usize,

    /// Idle timeout before a connection is considered stale
    idle_timeout: Duration,
}

impl ConnectionManager {
    /// Create a new connection manager
    ///
    /// # Arguments
    /// * `max_connections` - Maximum number of concurrent connections (0 = unlimited)
    /// * `idle_timeout` - Idle timeout before connection is considered stale
    pub fn new(max_connections: usize, idle_timeout: Duration) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            max_connections,
            idle_timeout,
        }
    }

    /// Create with default settings
    ///
    /// Defaults:
    /// - max_connections: 100
    /// - idle_timeout: 5 minutes
    pub fn with_defaults() -> Self {
        Self::new(100, Duration::from_secs(300))
    }

    /// Register a new connection
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    /// * `client_address` - Optional client IP address
    /// * `association` - Association context
    ///
    /// # Errors
    /// Returns error if maximum connections limit is reached
    pub async fn register_connection(
        &self,
        client_sap: u16,
        client_address: Option<String>,
        association: AssociationContext,
    ) -> DlmsResult<()> {
        let mut connections = self.connections.write().await;

        // Check if connection limit is reached
        if self.max_connections > 0 && connections.len() >= self.max_connections {
            // Check if this is a reconnection from existing client
            if !connections.contains_key(&client_sap) {
                return Err(DlmsError::Connection(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    format!(
                        "Maximum connections limit reached: {}",
                        self.max_connections
                    ),
                )));
            }
        }

        let info = ConnectionInfo::new(client_sap, client_address, association);
        connections.insert(client_sap, info);

        Ok(())
    }

    /// Unregister a connection
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    pub async fn unregister_connection(&self, client_sap: u16) {
        let mut connections = self.connections.write().await;
        connections.remove(&client_sap);
    }

    /// Update connection activity
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    pub async fn update_activity(&self, client_sap: u16) -> DlmsResult<()> {
        let mut connections = self.connections.write().await;

        if let Some(info) = connections.get_mut(&client_sap) {
            info.update_activity();
            Ok(())
        } else {
            Err(DlmsError::InvalidData(format!(
                "Connection not found for SAP: {}",
                client_sap
            )))
        }
    }

    /// Get connection info
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    ///
    /// # Returns
    /// Connection info if found, None otherwise
    pub async fn get_connection(&self, client_sap: u16) -> Option<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections.get(&client_sap).cloned()
    }

    /// Get all active connections
    ///
    /// # Returns
    /// Vector of all active connection info
    pub async fn get_all_connections(&self) -> Vec<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections.values().cloned().collect()
    }

    /// Get active connection count
    ///
    /// # Returns
    /// Number of active connections
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    /// Check if a connection exists
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    ///
    /// # Returns
    /// true if connection exists, false otherwise
    pub async fn has_connection(&self, client_sap: u16) -> bool {
        let connections = self.connections.read().await;
        connections.contains_key(&client_sap)
    }

    /// Clean up stale connections
    ///
    /// Removes connections that have been idle longer than the configured timeout.
    ///
    /// # Returns
    /// Number of connections removed
    pub async fn cleanup_stale_connections(&self) -> usize {
        let mut connections = self.connections.write().await;
        let initial_count = connections.len();

        connections
            .retain(|_, info| !info.is_stale(self.idle_timeout));

        initial_count - connections.len()
    }

    /// Force disconnect a client
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    ///
    /// # Returns
    /// true if connection was removed, false if it didn't exist
    pub async fn disconnect_client(&self, client_sap: u16) -> bool {
        let mut connections = self.connections.write().await;
        connections.remove(&client_sap).is_some()
    }

    /// Get connections older than specified duration
    ///
    /// # Arguments
    /// * `duration` - Minimum connection duration
    ///
    /// # Returns
    /// Vector of connection info for connections older than specified duration
    pub async fn get_connections_older_than(&self, duration: Duration) -> Vec<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections
            .values()
            .filter(|info| info.duration() > duration)
            .cloned()
            .collect()
    }

    /// Get connections idle longer than specified duration
    ///
    /// # Arguments
    /// * `duration` - Minimum idle duration
    ///
    /// # Returns
    /// Vector of connection info for connections idle longer than specified duration
    pub async fn get_idle_connections(&self, duration: Duration) -> Vec<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections
            .values()
            .filter(|info| info.idle_time() > duration)
            .cloned()
            .collect()
    }

    /// Get connection statistics
    ///
    /// # Returns
    /// Connection statistics including count, oldest/newest connections, etc.
    pub async fn get_statistics(&self) -> ConnectionStatistics {
        let connections = self.connections.read().await;

        if connections.is_empty() {
            return ConnectionStatistics::default();
        }

        let count = connections.len();
        let now = Instant::now();

        let oldest = connections
            .values()
            .min_by_key(|info| info.connected_at)
            .map(|info| now.duration_since(info.connected_at));
        let newest = connections
            .values()
            .max_by_key(|info| info.connected_at)
            .map(|info| now.duration_since(info.connected_at));

        let longest_idle = connections
            .values()
            .max_by_key(|info| info.last_activity)
            .map(|info| info.idle_time());

        let shortest_idle = connections
            .values()
            .min_by_key(|info| info.last_activity)
            .map(|info| info.idle_time());

        ConnectionStatistics {
            active_connections: count,
            max_connections: if self.max_connections == 0 {
                None
            } else {
                Some(self.max_connections)
            },
            oldest_connection_duration: oldest,
            newest_connection_duration: newest,
            longest_idle_time: longest_idle,
            shortest_idle_time: shortest_idle,
        }
    }
}

/// Connection statistics
#[derive(Debug, Clone, Default)]
pub struct ConnectionStatistics {
    /// Number of active connections
    pub active_connections: usize,
    /// Maximum allowed connections (None = unlimited)
    pub max_connections: Option<usize>,
    /// Duration of the oldest connection
    pub oldest_connection_duration: Option<Duration>,
    /// Duration of the newest connection
    pub newest_connection_duration: Option<Duration>,
    /// Longest idle time among all connections
    pub longest_idle_time: Option<Duration>,
    /// Shortest idle time among all connections
    pub shortest_idle_time: Option<Duration>,
}

impl ConnectionStatistics {
    /// Check if connections are at capacity
    pub fn is_at_capacity(&self) -> bool {
        if let Some(max) = self.max_connections {
            self.active_connections >= max
        } else {
            false
        }
    }

    /// Get connection utilization percentage
    ///
    /// # Returns
    /// Utilization as a percentage (0-100), or None if unlimited
    pub fn utilization_percent(&self) -> Option<f64> {
        self.max_connections.map(|max| {
            (self.active_connections as f64 / max as f64) * 100.0
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dlms_security::SecuritySuite;

    fn create_test_association() -> AssociationContext {
        AssociationContext {
            client_sap: 1,
            server_sap: 16,
            security_options: SecuritySuite::default(),
            conformance: dlms_application::pdu::Conformance::default(),
            max_pdu_size: 1024,
            dlms_version: 6,
        }
    }

    #[tokio::test]
    async fn test_register_connection() {
        let manager = ConnectionManager::with_defaults();
        let association = create_test_association();

        let result = manager
            .register_connection(1, Some("127.0.0.1".to_string()), association)
            .await;

        assert!(result.is_ok());
        assert_eq!(manager.connection_count().await, 1);
    }

    #[tokio::test]
    async fn test_unregister_connection() {
        let manager = ConnectionManager::with_defaults();
        let association = create_test_association();

        manager
            .register_connection(1, Some("127.0.0.1".to_string()), association)
            .await
            .unwrap();

        manager.unregister_connection(1).await;
        assert_eq!(manager.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_max_connections_limit() {
        let manager = ConnectionManager::new(2, Duration::from_secs(300));
        let association = create_test_association();

        // Register first connection
        assert!(manager
            .register_connection(1, Some("127.0.0.1".to_string()), association.clone())
            .await
            .is_ok());

        // Register second connection
        assert!(manager
            .register_connection(2, Some("127.0.0.2".to_string()), association.clone())
            .await
            .is_ok());

        // Third connection should fail
        assert!(manager
            .register_connection(3, Some("127.0.0.3".to_string()), association)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_update_activity() {
        let manager = ConnectionManager::with_defaults();
        let association = create_test_association();

        manager
            .register_connection(1, None, association)
            .await
            .unwrap();

        // Initial idle time should be very small
        tokio::time::sleep(Duration::from_millis(10)).await;
        let conn = manager.get_connection(1).await.unwrap();
        assert!(conn.idle_time() < Duration::from_millis(50));

        // Update activity
        manager.update_activity(1).await.unwrap();

        // Idle time should be reset
        let conn = manager.get_connection(1).await.unwrap();
        assert!(conn.idle_time() < Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_cleanup_stale_connections() {
        let manager = ConnectionManager::new(10, Duration::from_millis(100));
        let association = create_test_association();

        // Register a connection
        manager
            .register_connection(1, None, association)
            .await
            .unwrap();

        // Wait for connection to become stale
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Cleanup should remove the stale connection
        let removed = manager.cleanup_stale_connections().await;
        assert_eq!(removed, 1);
        assert_eq!(manager.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_disconnect_client() {
        let manager = ConnectionManager::with_defaults();
        let association = create_test_association();

        manager
            .register_connection(1, None, association)
            .await
            .unwrap();

        assert!(manager.disconnect_client(1).await);
        assert!(!manager.disconnect_client(1).await); // Already disconnected
    }

    #[tokio::test]
    async fn test_connection_statistics() {
        let manager = ConnectionManager::new(5, Duration::from_secs(300));
        let association = create_test_association();

        manager
            .register_connection(1, Some("127.0.0.1".to_string()), association.clone())
            .await
            .unwrap();
        manager
            .register_connection(2, Some("127.0.0.2".to_string()), association)
            .await
            .unwrap();

        let stats = manager.get_statistics().await;
        assert_eq!(stats.active_connections, 2);
        assert_eq!(stats.max_connections, Some(5));
        assert_eq!(stats.utilization_percent(), Some(40.0));
        assert!(!stats.is_at_capacity());
    }
}
