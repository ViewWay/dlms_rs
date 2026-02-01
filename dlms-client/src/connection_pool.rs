//! Connection pool management for DLMS/COSEM client
//!
//! This module provides connection pooling functionality for efficient
//! reuse of DLMS/COSEM connections.

use dlms_core::DlmsResult;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections in the pool
    pub max_size: usize,
    /// Minimum number of idle connections to maintain
    pub min_idle: usize,
    /// Maximum idle time before a connection is closed
    pub max_idle_time: Duration,
    /// Maximum lifetime of a connection
    pub max_lifetime: Duration,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Whether to enable health checking
    pub enable_health_check: bool,
    /// Health check interval
    pub health_check_interval: Duration,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_size: 10,
            min_idle: 2,
            max_idle_time: Duration::from_secs(300),  // 5 minutes
            max_lifetime: Duration::from_secs(3600),  // 1 hour
            connection_timeout: Duration::from_secs(30),
            enable_health_check: true,
            health_check_interval: Duration::from_secs(60),  // 1 minute
        }
    }
}

impl ConnectionPoolConfig {
    /// Create a new configuration with custom max size
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }

    /// Set the minimum idle connections
    pub fn with_min_idle(mut self, min_idle: usize) -> Self {
        self.min_idle = min_idle;
        self
    }

    /// Set the maximum idle time
    pub fn with_max_idle_time(mut self, max_idle_time: Duration) -> Self {
        self.max_idle_time = max_idle_time;
        self
    }

    /// Set the maximum lifetime
    pub fn with_max_lifetime(mut self, max_lifetime: Duration) -> Self {
        self.max_lifetime = max_lifetime;
        self
    }

    /// Enable or disable health checking
    pub fn with_health_check(mut self, enable: bool) -> Self {
        self.enable_health_check = enable;
        self
    }

    /// Set the health check interval
    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }
}

/// Key for identifying connections in the pool
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConnectionKey {
    /// Unique identifier for the connection endpoint
    pub endpoint: String,
    /// Connection type identifier
    pub connection_type: ConnectionType,
}

impl ConnectionKey {
    /// Create a new connection key
    pub fn new(endpoint: String, connection_type: ConnectionType) -> Self {
        Self {
            endpoint,
            connection_type,
        }
    }

    /// Create a TCP HDLC connection key
    pub fn tcp_hdlc(host: String, port: u16) -> Self {
        Self::new(format!("{}:{}", host, port), ConnectionType::TcpHdlc)
    }

    /// Create a TCP Wrapper connection key
    pub fn tcp_wrapper(host: String, port: u16) -> Self {
        Self::new(format!("{}:{}", host, port), ConnectionType::TcpWrapper)
    }

    /// Create a Serial connection key (default to HDLC)
    pub fn serial(port: String) -> Self {
        Self::new(port, ConnectionType::SerialHdlc)
    }
}

/// Connection type
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConnectionType {
    /// TCP with HDLC framing
    TcpHdlc,
    /// TCP with Wrapper protocol
    TcpWrapper,
    /// Serial port with HDLC framing
    SerialHdlc,
    /// Serial port with Wrapper protocol
    SerialWrapper,
}

impl std::fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionType::TcpHdlc => write!(f, "tcp-hdlc"),
            ConnectionType::TcpWrapper => write!(f, "tcp-wrapper"),
            ConnectionType::SerialHdlc => write!(f, "serial-hdlc"),
            ConnectionType::SerialWrapper => write!(f, "serial-wrapper"),
        }
    }
}

/// Connection pool statistics
#[derive(Debug, Clone, Default)]
pub struct PoolStatistics {
    /// Total number of connections currently in the pool
    pub total_connections: usize,
    /// Number of active (in-use) connections
    pub active_connections: usize,
    /// Number of idle connections available
    pub idle_connections: usize,
    /// Total number of connections created since pool start
    pub total_created: u64,
    /// Total number of connections closed
    pub total_closed: u64,
    /// Number of failed connection attempts
    pub failed_attempts: u64,
    /// Number of successful connection acquisitions
    pub successful_acquisitions: u64,
    /// Number of times a connection was reused from the pool
    pub connection_reuses: u64,
    /// Average wait time in milliseconds for acquiring a connection
    pub avg_wait_time_ms: u64,
}

/// Pool entry tracking connection state
#[derive(Debug)]
struct PoolEntry {
    /// Connection state identifier (opaque to the pool)
    connection_id: u64,
    /// When this connection was created
    created_at: Instant,
    /// When this connection was last used
    last_used: Instant,
    /// Whether the connection is currently in use
    in_use: bool,
}

impl PoolEntry {
    /// Create a new pool entry
    fn new(connection_id: u64) -> Self {
        let now = Instant::now();
        Self {
            connection_id,
            created_at: now,
            last_used: now,
            in_use: false,
        }
    }

    /// Check if the entry is expired based on config
    fn is_expired(&self, config: &ConnectionPoolConfig) -> bool {
        // Check lifetime
        if self.created_at.elapsed() > config.max_lifetime {
            return true;
        }

        // Check idle time (only if not in use)
        if !self.in_use && self.last_used.elapsed() > config.max_idle_time {
            return true;
        }

        false
    }

    /// Mark the entry as in use
    fn mark_in_use(&mut self) {
        self.in_use = true;
        self.last_used = Instant::now();
    }

    /// Mark the entry as available
    fn mark_available(&mut self) {
        self.in_use = false;
        self.last_used = Instant::now();
    }
}

/// Connection pool manager
///
/// Manages a pool of reusable connections with health checking
/// and automatic cleanup of idle/expired connections.
///
/// # Example
///
/// ```no_run
/// use dlms_client::connection_pool::{ConnectionPool, ConnectionPoolConfig, ConnectionKey};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = ConnectionPoolConfig::default()
///         .with_max_size(20)
///         .with_max_idle_time(Duration::from_secs(600));
///
///     let pool = ConnectionPool::new(config);
///
///     let key = ConnectionKey::tcp_hdlc("192.168.1.100".to_string(), 4059);
///
///     // The pool tracks connections but actual connection creation
///     // is handled by the connection manager
///
///     let stats = pool.stats().await;
///     println!("Pool stats: {:?}", stats);
///
///     Ok(())
/// }
/// ```
pub struct ConnectionPool {
    /// Pool configuration
    config: ConnectionPoolConfig,
    /// Map of connection key to pool entries
    entries: Arc<RwLock<std::collections::HashMap<ConnectionKey, Vec<PoolEntry>>>>,
    /// Semaphore for limiting concurrent connections
    semaphore: Arc<Semaphore>,
    /// Statistics
    stats: Arc<RwLock<PoolStatistics>>,
    /// Next connection ID
    next_id: Arc<RwLock<u64>>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(config: ConnectionPoolConfig) -> Self {
        let max_size = config.max_size;
        let entries = Arc::new(RwLock::new(std::collections::HashMap::new()));
        let stats = Arc::new(RwLock::new(PoolStatistics::default()));
        let semaphore = Arc::new(Semaphore::new(max_size));
        let next_id = Arc::new(RwLock::new(0));

        // Start health check task if enabled
        if config.enable_health_check {
            let entries_clone = entries.clone();
            let stats_clone = stats.clone();
            let config_clone = config.clone();
            let interval = config.health_check_interval;

            tokio::spawn(async move {
                let mut ticker = tokio::time::interval(interval);
                // Skip the immediate first tick
                ticker.tick().await;
                loop {
                    ticker.tick().await;
                    ConnectionPool::perform_health_check(&entries_clone, &stats_clone, &config_clone).await;
                }
            });
        }

        Self {
            config,
            entries,
            semaphore,
            stats,
            next_id,
        }
    }

    /// Create a connection pool with default configuration
    pub fn default_config() -> Self {
        Self::new(ConnectionPoolConfig::default())
    }

    /// Acquire a permit to create/use a connection
    ///
    /// This should be called before creating a new connection.
    /// Returns `None` if the pool is at capacity.
    pub async fn try_acquire(&self) -> Option<tokio::sync::SemaphorePermit<'_>> {
        self.semaphore.try_acquire().ok()
    }

    /// Acquire a permit to create/use a connection (waits if necessary)
    pub async fn acquire(&self) -> tokio::sync::SemaphorePermit<'_> {
        self.semaphore.acquire().await
            .unwrap_or_else(|_| unreachable!("Semaphore never closes"))
    }

    /// Register a new connection with the pool
    ///
    /// Returns the connection ID that should be used to track this connection.
    pub async fn register_connection(&self, key: ConnectionKey) -> u64 {
        let mut entries = self.entries.write().await;
        let mut next_id = self.next_id.write().await;
        let connection_id = *next_id;
        *next_id = next_id.wrapping_add(1);

        let entry_list = entries.entry(key).or_insert_with(Vec::new);
        entry_list.push(PoolEntry::new(connection_id));

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_created += 1;
        stats.total_connections += 1;
        stats.idle_connections += 1;

        connection_id
    }

    /// Mark a connection as in use
    ///
    /// Returns `true` if the connection was found and marked.
    pub async fn mark_in_use(&self, key: &ConnectionKey, connection_id: u64) -> bool {
        let found = {
            let mut entries = self.entries.write().await;

            if let Some(entry_list) = entries.get_mut(key) {
                if let Some(entry) = entry_list.iter_mut().find(|e| e.connection_id == connection_id) {
                    if !entry.in_use {
                        entry.mark_in_use();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        };

        if found {
            // Update stats after releasing entries lock
            let mut stats = self.stats.write().await;
            stats.active_connections += 1;
            stats.idle_connections = stats.idle_connections.saturating_sub(1);
            stats.successful_acquisitions += 1;
        }

        found
    }

    /// Mark a connection as available (return it to the pool)
    ///
    /// Returns `true` if the connection was found and marked.
    pub async fn return_connection(&self, key: &ConnectionKey, connection_id: u64) -> bool {
        let found = {
            let mut entries = self.entries.write().await;

            if let Some(entry_list) = entries.get_mut(key) {
                if let Some(entry) = entry_list.iter_mut().find(|e| e.connection_id == connection_id) {
                    entry.mark_available();
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        if found {
            // Update stats after releasing entries lock
            let mut stats = self.stats.write().await;
            stats.active_connections = stats.active_connections.saturating_sub(1);
            stats.idle_connections += 1;
        }

        found
    }

    /// Remove a connection from the pool
    ///
    /// Returns `true` if the connection was found and removed.
    pub async fn remove_connection(&self, key: &ConnectionKey, connection_id: u64) -> bool {
        let mut entries = self.entries.write().await;

        if let Some(entry_list) = entries.get_mut(key) {
            let initial_len = entry_list.len();
            entry_list.retain(|e| e.connection_id != connection_id);

            if entry_list.len() < initial_len {
                // Update stats
                let mut stats = self.stats.write().await;
                stats.total_closed += 1;
                stats.total_connections = stats.total_connections.saturating_sub(1);

                // Clean up empty entry lists
                if entry_list.is_empty() {
                    entries.remove(key);
                }

                return true;
            }
        }

        false
    }

    /// Check if an idle connection is available for the given key
    pub async fn has_idle_connection(&self, key: &ConnectionKey) -> bool {
        let entries = self.entries.read().await;

        entries.get(key)
            .map(|list| list.iter().any(|e| !e.in_use && !e.is_expired(&self.config)))
            .unwrap_or(false)
    }

    /// Get the number of connections for a specific key
    pub async fn connection_count(&self, key: &ConnectionKey) -> usize {
        let entries = self.entries.read().await;

        entries.get(key)
            .map(|list| list.len())
            .unwrap_or(0)
    }

    /// Get an idle connection ID for the given key, if available
    ///
    /// This returns the first available (not in use, not expired) connection ID.
    pub async fn get_idle_connection(&self, key: &ConnectionKey) -> Option<u64> {
        let connection_id = {
            let mut entries = self.entries.write().await;

            if let Some(entry_list) = entries.get_mut(key) {
                // Find the index of the first idle, non-expired entry
                let idx = entry_list.iter().position(|e| !e.in_use && !e.is_expired(&self.config));

                if let Some(idx) = idx {
                    let entry = &mut entry_list[idx];
                    entry.mark_in_use();
                    Some(entry.connection_id)
                } else {
                    None
                }
            } else {
                None
            }
        };

        if connection_id.is_some() {
            // Update stats after releasing entries lock
            let mut stats = self.stats.write().await;
            stats.active_connections += 1;
            stats.idle_connections = stats.idle_connections.saturating_sub(1);
            stats.successful_acquisitions += 1;
            stats.connection_reuses += 1;
        }

        connection_id
    }

    /// Perform health check on all connections
    async fn perform_health_check(
        entries: &Arc<RwLock<std::collections::HashMap<ConnectionKey, Vec<PoolEntry>>>>,
        stats: &Arc<RwLock<PoolStatistics>>,
        config: &ConnectionPoolConfig,
    ) {
        let mut total_closed: u64 = 0;

        {
            let mut entries = entries.write().await;

            for entry_list in entries.values_mut() {
                let initial_len = entry_list.len();

                // Remove expired entries
                entry_list.retain(|entry| {
                    // Keep in-use entries even if expired (will be cleaned when returned)
                    if entry.in_use {
                        return true;
                    }

                    !entry.is_expired(config)
                });

                total_closed += (initial_len - entry_list.len()) as u64;
            }
        }

        // Update stats outside of entries write lock
        let mut s = stats.write().await;
        s.total_closed += total_closed;

        // Recalculate counts
        let entries = entries.read().await;
        s.total_connections = entries.values().map(|v| v.len()).sum();
        s.active_connections = entries.values()
            .flat_map(|v| v.iter())
            .filter(|e| e.in_use)
            .count();
        s.idle_connections = s.total_connections - s.active_connections;
    }

    /// Clean up expired and idle connections
    ///
    /// Returns the number of connections removed.
    pub async fn cleanup(&self) -> usize {
        let mut total_removed: usize = 0;
        let mut total_closed: u64 = 0;
        let mut total_connections: usize = 0;

        {
            let mut entries = self.entries.write().await;

            for entry_list in entries.values_mut() {
                let initial_len = entry_list.len();

                // Remove expired entries and extra idle entries (beyond min_idle)
                // First, count idle entries
                let idle_count = entry_list.iter().filter(|e| !e.in_use).count();
                let keep_count = self.config.min_idle.min(idle_count);

                // Create a list of entries to remove
                let mut idle_seen = 0;
                let mut to_remove = Vec::new();

                for (idx, entry) in entry_list.iter().enumerate() {
                    if !entry.in_use {
                        idle_seen += 1;
                        // Remove if we have more idle than needed and this is beyond the keep count
                        if idle_seen > keep_count {
                            to_remove.push(idx);
                        } else if entry.is_expired(&self.config) {
                            to_remove.push(idx);
                        }
                    }
                }

                // Remove in reverse order to maintain indices
                for idx in to_remove.into_iter().rev() {
                    entry_list.remove(idx);
                }

                let removed = initial_len - entry_list.len();
                total_removed += removed;
                total_closed += removed as u64;
                total_connections += entry_list.len();
            }

            // Remove empty entry lists
            entries.retain(|_, list| !list.is_empty());
        }

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_closed += total_closed;
        stats.total_connections = total_connections;

        total_removed
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStatistics {
        self.stats.read().await.clone()
    }

    /// Close all connections in the pool
    ///
    /// This clears all entries from the pool. The actual connections
    /// should be closed by the caller.
    pub async fn close(&self) -> DlmsResult<()> {
        let mut entries = self.entries.write().await;
        let total: usize = entries.values().map(|v| v.len()).sum();

        entries.clear();

        let mut stats = self.stats.write().await;
        stats.total_connections = 0;
        stats.active_connections = 0;
        stats.idle_connections = 0;
        stats.total_closed += total as u64;

        Ok(())
    }
}

impl Drop for ConnectionPool {
    fn drop(&mut self) {
        // Pool is being dropped, cleanup will happen automatically
        // as Arcs are dropped
    }
}

/// Health checker for connection validation
pub struct HealthChecker {
    /// Timeout for health check operations
    pub timeout: Duration,
    /// Number of retries before giving up
    pub max_retries: usize,
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            max_retries: 3,
        }
    }
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(timeout: Duration, max_retries: usize) -> Self {
        Self {
            timeout,
            max_retries,
        }
    }

    /// Check if a connection is healthy with a health check function
    pub async fn check<F, Fut>(&self, health_check: F) -> bool
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let mut retries = 0;

        while retries < self.max_retries {
            let result = tokio::time::timeout(self.timeout, health_check()).await;

            if let Ok(healthy) = result {
                if healthy {
                    return true;
                }
            }

            retries += 1;

            if retries < self.max_retries {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_key() {
        let key1 = ConnectionKey::tcp_hdlc("127.0.0.1".to_string(), 4059);
        let key2 = ConnectionKey::tcp_hdlc("127.0.0.1".to_string(), 4059);
        let key3 = ConnectionKey::tcp_wrapper("127.0.0.1".to_string(), 4059);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_connection_key_serial() {
        let key1 = ConnectionKey::serial("/dev/ttyUSB0".to_string());
        let key2 = ConnectionKey::serial("/dev/ttyUSB1".to_string());

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_connection_type_display() {
        assert_eq!(ConnectionType::TcpHdlc.to_string(), "tcp-hdlc");
        assert_eq!(ConnectionType::TcpWrapper.to_string(), "tcp-wrapper");
        assert_eq!(ConnectionType::SerialHdlc.to_string(), "serial-hdlc");
        assert_eq!(ConnectionType::SerialWrapper.to_string(), "serial-wrapper");
    }

    #[test]
    fn test_pool_config_builder() {
        let config = ConnectionPoolConfig::default()
            .with_max_size(20)
            .with_min_idle(5)
            .with_max_idle_time(Duration::from_secs(600))
            .with_max_lifetime(Duration::from_secs(7200))
            .with_health_check(false);

        assert_eq!(config.max_size, 20);
        assert_eq!(config.min_idle, 5);
        assert_eq!(config.max_idle_time, Duration::from_secs(600));
        assert_eq!(config.max_lifetime, Duration::from_secs(7200));
        assert_eq!(config.enable_health_check, false);
    }

    #[test]
    fn test_pool_config_default() {
        let config = ConnectionPoolConfig::default();

        assert_eq!(config.max_size, 10);
        assert_eq!(config.min_idle, 2);
        assert_eq!(config.max_idle_time, Duration::from_secs(300));
        assert_eq!(config.max_lifetime, Duration::from_secs(3600));
        assert_eq!(config.enable_health_check, true);
        assert_eq!(config.health_check_interval, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_pool_stats() {
        let pool = ConnectionPool::default_config();
        let stats = pool.stats().await;

        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.idle_connections, 0);
        assert_eq!(stats.total_created, 0);
    }

    #[tokio::test]
    async fn test_pool_register() {
        let pool = ConnectionPool::default_config();
        let key = ConnectionKey::tcp_hdlc("127.0.0.1".to_string(), 4059);

        let id1 = pool.register_connection(key.clone()).await;
        let id2 = pool.register_connection(key.clone()).await;

        assert!(id1 < id2);
        assert_eq!(pool.connection_count(&key).await, 2);

        let stats = pool.stats().await;
        assert_eq!(stats.total_connections, 2);
        assert_eq!(stats.total_created, 2);
    }

    #[tokio::test]
    async fn test_pool_mark_in_use() {
        let pool = ConnectionPool::default_config();
        let key = ConnectionKey::tcp_hdlc("127.0.0.1".to_string(), 4059);

        let id = pool.register_connection(key.clone()).await;

        assert!(pool.mark_in_use(&key, id).await);
        assert!(!pool.mark_in_use(&key, id).await); // Already in use

        let stats = pool.stats().await;
        assert_eq!(stats.active_connections, 1);
        assert_eq!(stats.idle_connections, 0); // After marking in use, idle should be 0
        assert_eq!(stats.total_connections, 1); // total = active + idle
    }

    #[tokio::test]
    async fn test_pool_return_connection() {
        let pool = ConnectionPool::default_config();
        let key = ConnectionKey::tcp_hdlc("127.0.0.1".to_string(), 4059);

        let id = pool.register_connection(key.clone()).await;
        pool.mark_in_use(&key, id).await;

        assert!(pool.return_connection(&key, id).await);

        let stats = pool.stats().await;
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.idle_connections, 1); // After return, should have 1 idle
        assert_eq!(stats.total_connections, 1); // total = active + idle
    }

    #[tokio::test]
    async fn test_pool_remove_connection() {
        let pool = ConnectionPool::default_config();
        let key = ConnectionKey::tcp_hdlc("127.0.0.1".to_string(), 4059);

        let id = pool.register_connection(key.clone()).await;

        assert!(pool.remove_connection(&key, id).await);
        assert_eq!(pool.connection_count(&key).await, 0);

        let stats = pool.stats().await;
        assert_eq!(stats.total_closed, 1);
    }

    #[tokio::test]
    async fn test_pool_has_idle_connection() {
        let pool = ConnectionPool::default_config();
        let key = ConnectionKey::tcp_hdlc("127.0.0.1".to_string(), 4059);

        assert!(!pool.has_idle_connection(&key).await);

        pool.register_connection(key.clone()).await;
        assert!(pool.has_idle_connection(&key).await);

        // Get the idle connection
        let id = pool.get_idle_connection(&key).await;
        assert!(id.is_some());

        // No more idle connections
        assert!(!pool.has_idle_connection(&key).await);
    }

    #[tokio::test]
    async fn test_pool_cleanup() {
        let config = ConnectionPoolConfig {
            max_size: 10,
            min_idle: 0,
            max_idle_time: Duration::from_millis(100),
            max_lifetime: Duration::from_secs(3600),
            connection_timeout: Duration::from_secs(30),
            enable_health_check: false,
            health_check_interval: Duration::from_secs(60),
        };

        let pool = ConnectionPool::new(config);
        let key = ConnectionKey::tcp_hdlc("127.0.0.1".to_string(), 4059);

        pool.register_connection(key.clone()).await;

        // Wait for idle time to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        let removed = pool.cleanup().await;
        assert_eq!(removed, 1);
        assert_eq!(pool.connection_count(&key).await, 0);
    }

    #[tokio::test]
    async fn test_pool_close() {
        let pool = ConnectionPool::default_config();
        let key = ConnectionKey::tcp_hdlc("127.0.0.1".to_string(), 4059);

        pool.register_connection(key.clone()).await;
        pool.register_connection(key.clone()).await;

        assert!(pool.close().await.is_ok());
        assert_eq!(pool.connection_count(&key).await, 0);
    }

    #[test]
    fn test_health_checker_default() {
        let checker = HealthChecker::default();

        assert_eq!(checker.timeout, Duration::from_secs(5));
        assert_eq!(checker.max_retries, 3);
    }

    #[test]
    fn test_health_checker_new() {
        let checker = HealthChecker::new(Duration::from_secs(10), 5);

        assert_eq!(checker.timeout, Duration::from_secs(10));
        assert_eq!(checker.max_retries, 5);
    }

    #[tokio::test]
    async fn test_health_checker_check() {
        let checker = HealthChecker::new(Duration::from_millis(100), 2);

        // Test with healthy check
        let result = checker.check(|| async { true }).await;
        assert!(result);

        // Test with unhealthy check
        let result = checker.check(|| async { false }).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_pool_acquire_semaphore() {
        let pool = ConnectionPool::default_config();

        // Should be able to acquire up to max_size permits
        {
            let _permit1 = pool.acquire().await;
            let _permit2 = pool.acquire().await;

            // Try acquire should succeed
            assert!(pool.try_acquire().await.is_some());
        }

        // After dropping, permits should be available again
        {
            let _permit1 = pool.acquire().await;
            let _permit2 = pool.acquire().await;
            let _permit3 = pool.acquire().await;
        }
    }

    #[tokio::test]
    async fn test_pool_get_idle_reuses_connection() {
        let pool = ConnectionPool::default_config();
        let key = ConnectionKey::tcp_hdlc("127.0.0.1".to_string(), 4059);

        // Register a connection
        let id = pool.register_connection(key.clone()).await;

        // Get the idle connection (marks as in use)
        let retrieved_id = pool.get_idle_connection(&key).await;
        assert_eq!(retrieved_id, Some(id));

        // Return it to the pool
        pool.return_connection(&key, id).await;

        // Get it again (should be reused)
        let stats_before = pool.stats().await;
        let retrieved_id2 = pool.get_idle_connection(&key).await;
        assert_eq!(retrieved_id2, Some(id));

        let stats_after = pool.stats().await;
        assert_eq!(stats_after.connection_reuses, stats_before.connection_reuses + 1);
    }
}
