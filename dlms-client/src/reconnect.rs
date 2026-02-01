//! Auto-reconnection mechanism for DLMS/COSEM client
//!
//! This module provides automatic reconnection functionality for handling
//! connection failures and maintaining persistent connections to meters.

use dlms_core::{DlmsError, DlmsResult};
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Reconnection strategy
#[derive(Debug, Clone)]
pub enum ReconnectStrategy {
    /// No reconnection - fail immediately on connection loss
    None,
    /// Fixed delay between reconnection attempts
    FixedDelay(Duration),
    /// Exponential backoff with optional maximum delay
    ExponentialBackoff {
        /// Initial delay
        initial_delay: Duration,
        /// Maximum delay (optional)
        max_delay: Option<Duration>,
        /// Backoff multiplier
        multiplier: f64,
    },
    /// Custom strategy with defined delays
    Custom(Vec<Duration>),
}

impl ReconnectStrategy {
    /// Create a fixed delay strategy
    pub fn fixed(delay_ms: u64) -> Self {
        Self::FixedDelay(Duration::from_millis(delay_ms))
    }

    /// Create an exponential backoff strategy
    pub fn exponential_backoff(initial_delay_ms: u64, max_delay_ms: Option<u64>) -> Self {
        Self::ExponentialBackoff {
            initial_delay: Duration::from_millis(initial_delay_ms),
            max_delay: max_delay_ms.map(Duration::from_millis),
            multiplier: 2.0,
        }
    }

    /// Get the next delay for a given attempt number
    pub fn next_delay(&self, attempt: u32) -> Option<Duration> {
        match self {
            ReconnectStrategy::None => None,
            ReconnectStrategy::FixedDelay(delay) => Some(*delay),
            ReconnectStrategy::ExponentialBackoff {
                initial_delay,
                max_delay,
                multiplier,
            } => {
                let delay = initial_delay.as_millis() as f64 * multiplier.powi(attempt as i32 - 1);
                let delay = Duration::from_millis(delay as u64);
                if let Some(max) = max_delay {
                    Some(delay.min(*max))
                } else {
                    Some(delay)
                }
            }
            ReconnectStrategy::Custom(delays) => {
                let index = (attempt as usize).saturating_sub(1);
                delays.get(index).copied()
            }
        }
    }
}

impl Default for ReconnectStrategy {
    fn default() -> Self {
        Self::fixed(5000) // Default 5 second delay
    }
}

/// Connection state for reconnection tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconnectionState {
    /// Connected and operational
    Connected,
    /// Disconnected and will reconnect
    Disconnected,
    /// Reconnecting in progress
    Reconnecting,
    /// Failed - will not reconnect
    Failed,
}

/// Reconnection statistics
#[derive(Debug, Clone, Default)]
pub struct ReconnectionStats {
    /// Total number of reconnection attempts
    pub total_attempts: u32,
    /// Successful reconnections
    pub successful_reconnections: u32,
    /// Failed reconnections
    pub failed_reconnections: u32,
    /// Current reconnection attempt number
    pub current_attempt: u32,
    /// Last reconnection time (if any)
    pub last_reconnection_time: Option<Instant>,
    /// Total time spent reconnecting
    pub total_reconnect_time: Duration,
}

impl ReconnectionStats {
    /// Create a new stats instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.successful_reconnections as f64 / self.total_attempts as f64
        }
    }

    /// Get the average reconnection time
    pub fn avg_reconnect_time(&self) -> Option<Duration> {
        if self.successful_reconnections == 0 {
            None
        } else {
            Some(self.total_reconnect_time / self.successful_reconnections as u32)
        }
    }
}

/// Reconnection configuration
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Reconnection strategy
    pub strategy: ReconnectStrategy,
    /// Maximum number of reconnection attempts (None = infinite)
    pub max_attempts: Option<u32>,
    /// Connection timeout for each attempt
    pub connection_timeout: Duration,
    /// Whether to reset the attempt counter on successful connection
    pub reset_on_success: bool,
    /// Maximum time to spend trying to reconnect before giving up
    pub max_reconnect_time: Option<Duration>,
}

impl ReconnectConfig {
    /// Create a new reconnect config
    pub fn new(strategy: ReconnectStrategy) -> Self {
        Self {
            strategy,
            max_attempts: Some(5),
            connection_timeout: Duration::from_secs(30),
            reset_on_success: true,
            max_reconnect_time: None,
        }
    }

    /// Set the maximum number of reconnection attempts
    pub fn with_max_attempts(mut self, max: u32) -> Self {
        self.max_attempts = Some(max);
        self
    }

    /// Set no limit on reconnection attempts
    pub fn unlimited_attempts(mut self) -> Self {
        self.max_attempts = None;
        self
    }

    /// Set the connection timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Set whether to reset attempt counter on success
    pub fn with_reset_on_success(mut self, reset: bool) -> Self {
        self.reset_on_success = reset;
        self
    }
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self::new(ReconnectStrategy::default())
    }
}

/// Reconnection manager
///
/// Manages automatic reconnection logic and state tracking.
pub struct ReconnectManager {
    /// Configuration
    config: ReconnectConfig,
    /// Current state
    state: Arc<RwLock<ReconnectionState>>,
    /// Statistics
    stats: Arc<RwLock<ReconnectionStats>>,
    /// Time when reconnection started
    reconnect_start: Arc<Mutex<Option<Instant>>>,
}

impl ReconnectManager {
    /// Create a new reconnection manager
    pub fn new(config: ReconnectConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(ReconnectionState::Connected)),
            stats: Arc::new(RwLock::new(ReconnectionStats::new())),
            reconnect_start: Arc::new(Mutex::new(None)),
        }
    }

    /// Create with default configuration
    pub fn default_config() -> Self {
        Self::new(ReconnectConfig::default())
    }

    /// Get the current state
    pub async fn state(&self) -> ReconnectionState {
        *self.state.read().await
    }

    /// Set the current state
    pub async fn set_state(&self, state: ReconnectionState) {
        *self.state.write().await = state;
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        self.state().await == ReconnectionState::Connected
    }

    /// Get the statistics
    pub async fn stats(&self) -> ReconnectionStats {
        self.stats.read().await.clone()
    }

    /// Should attempt reconnection
    ///
    /// Returns true if reconnection should be attempted based on
    /// current state and configuration.
    pub async fn should_reconnect(&self) -> bool {
        let state = self.state().await;
        if state == ReconnectionState::Connected {
            return false;
        }

        let stats = self.stats.read().await;
        let current_attempt = stats.current_attempt;

        // Check max attempts
        if let Some(max) = self.config.max_attempts {
            if current_attempt >= max {
                return false;
            }
        }

        // Check max reconnect time
        if let Some(max_time) = self.config.max_reconnect_time {
            let start_guard = self.reconnect_start.lock().await;
            if let Some(start) = *start_guard {
                if start.elapsed() > max_time {
                    return false;
                }
            }
        }

        true
    }

    /// Get the delay before next reconnection attempt
    pub async fn next_delay(&self) -> Option<Duration> {
        let stats = self.stats.read().await;
        self.config.strategy.next_delay(stats.current_attempt)
    }

    /// Record a reconnection attempt
    pub async fn record_attempt(&self) {
        let mut stats = self.stats.write().await;
        stats.total_attempts += 1;
        stats.current_attempt += 1;

        // Set reconnect start time on first attempt
        if stats.current_attempt == 1 {
            let mut start = self.reconnect_start.lock().await;
            if start.is_none() {
                *start = Some(Instant::now());
            }
        }
    }

    /// Record a successful reconnection
    pub async fn record_success(&self) {
        let mut stats = self.stats.write().await;
        stats.successful_reconnections += 1;

        // Calculate time spent reconnecting
        let mut start = self.reconnect_start.lock().await;
        if let Some(start_time) = *start {
            stats.total_reconnect_time += start_time.elapsed();
            *start = None;
        }

        // Reset counter if configured
        if self.config.reset_on_success {
            stats.current_attempt = 0;
        }

        // Update state
        *self.state.write().await = ReconnectionState::Connected;
    }

    /// Record a failed reconnection
    pub async fn record_failure(&self) {
        let mut stats = self.stats.write().await;
        stats.failed_reconnections += 1;

        // Check if we should give up
        let should_give_up = if let Some(max) = self.config.max_attempts {
            stats.current_attempt >= max
        } else {
            false
        };

        if should_give_up {
            *self.state.write().await = ReconnectionState::Failed;
            // Clear reconnect start time
            *self.reconnect_start.lock().await = None;
        } else {
            *self.state.write().await = ReconnectionState::Disconnected;
        }
    }

    /// Record a disconnection
    pub async fn record_disconnection(&self) {
        *self.state.write().await = ReconnectionState::Disconnected;
    }

    /// Reset the reconnection state
    pub async fn reset(&self) {
        let mut stats = self.stats.write().await;
        stats.current_attempt = 0;
        *self.reconnect_start.lock().await = None;
        *self.state.write().await = ReconnectionState::Connected;
    }

    /// Calculate and wait for the next reconnection delay
    ///
    /// Returns true if reconnection should proceed, false if aborted.
    pub async fn wait_for_next_attempt(&self) -> DlmsResult<bool> {
        if !self.should_reconnect().await {
            return Err(DlmsError::Protocol(
                "Maximum reconnection attempts reached".to_string(),
            ));
        }

        let delay = self.next_delay().await.unwrap_or(Duration::ZERO);
        if delay > Duration::ZERO {
            tokio::time::sleep(delay).await;
        }

        Ok(true)
    }
}

/// Trait for reconnectable connections
#[async_trait::async_trait]
pub trait ReconnectableConnection: Send + Sync {
    /// Connect using the stored configuration
    async fn connect(&mut self) -> DlmsResult<()>;

    /// Disconnect
    async fn disconnect(&mut self) -> DlmsResult<()>;

    /// Check if the connection is alive
    async fn is_alive(&self) -> bool;

    /// Get the connection state
    async fn connection_state(&self) -> ConnectionState;
}

/// Reconnection state for use with the manager
pub use crate::ConnectionState;

/// Auto-reconnecting connection wrapper
///
/// Wraps a connection and provides automatic reconnection on failure.
pub struct AutoReconnectConnection<C> {
    /// Inner connection
    inner: C,
    /// Reconnection manager
    manager: Arc<ReconnectManager>,
}

impl<C> AutoReconnectConnection<C>
where
    C: ReconnectableConnection + Send + Sync,
{
    /// Create a new auto-reconnecting connection
    pub fn new(inner: C, config: ReconnectConfig) -> Self {
        Self {
            inner,
            manager: Arc::new(ReconnectManager::new(config)),
        }
    }

    /// Get the reconnection manager
    pub fn manager(&self) -> &Arc<ReconnectManager> {
        &self.manager
    }

    /// Execute an operation with automatic reconnection
    pub async fn execute_with_reconnect<F, R>(
        &mut self,
        operation: F,
    ) -> DlmsResult<R>
    where
        F: Fn(&mut C) -> futures::future::BoxFuture<'_, DlmsResult<R>> + Send + Sync,
    {
        // Check if we need to reconnect
        if !self.inner.is_alive().await {
            self.reconnect().await?;
        }

        // Try the operation
        match operation(&mut self.inner).await {
            Ok(result) => Ok(result),
            Err(_e) => {
                // Record disconnection and try to reconnect
                self.manager.record_disconnection().await;

                // Try to reconnect
                self.reconnect().await?;

                // Retry the operation once
                operation(&mut self.inner).await
            }
        }
    }

    /// Perform reconnection
    async fn reconnect(&mut self) -> DlmsResult<()> {
        self.manager.set_state(ReconnectionState::Reconnecting).await;

        loop {
            // Check if we should continue trying
            if !self.manager.should_reconnect().await {
                return Err(DlmsError::Protocol(
                    "Reconnection failed: maximum attempts reached".to_string(),
                ));
            }

            // Wait for the next attempt
            self.manager.wait_for_next_attempt().await?;

            // Record attempt
            self.manager.record_attempt().await;

            // Try to connect
            match self.inner.connect().await {
                Ok(()) => {
                    self.manager.record_success().await;
                    return Ok(());
                }
                Err(_e) => {
                    self.manager.record_failure().await;
                    // Continue loop to try again
                }
            }
        }
    }

    /// Get the inner connection
    pub fn inner(&self) -> &C {
        &self.inner
    }

    /// Get mutable reference to the inner connection
    pub fn inner_mut(&mut self) -> &mut C {
        &mut self.inner
    }

    /// Disconnect and reset the reconnection manager
    pub async fn disconnect(&mut self) -> DlmsResult<()> {
        self.inner.disconnect().await?;
        self.manager.reset().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconnect_strategy_fixed_delay() {
        let strategy = ReconnectStrategy::fixed(1000);
        assert_eq!(strategy.next_delay(1), Some(Duration::from_millis(1000)));
        assert_eq!(strategy.next_delay(2), Some(Duration::from_millis(1000)));
        assert_eq!(strategy.next_delay(3), Some(Duration::from_millis(1000)));
    }

    #[test]
    fn test_reconnect_strategy_exponential_backoff() {
        let strategy = ReconnectStrategy::exponential_backoff(1000, Some(8000));
        assert_eq!(strategy.next_delay(1), Some(Duration::from_millis(1000)));
        assert_eq!(strategy.next_delay(2), Some(Duration::from_millis(2000)));
        assert_eq!(strategy.next_delay(3), Some(Duration::from_millis(4000)));
        assert_eq!(strategy.next_delay(4), Some(Duration::from_millis(8000))); // Max
        assert_eq!(strategy.next_delay(5), Some(Duration::from_millis(8000))); // Still max
    }

    #[test]
    fn test_reconnect_strategy_none() {
        let strategy = ReconnectStrategy::None;
        assert_eq!(strategy.next_delay(1), None);
    }

    #[test]
    fn test_reconnect_strategy_custom() {
        let delays = vec![
            Duration::from_millis(100),
            Duration::from_millis(500),
            Duration::from_millis(2000),
        ];
        let strategy = ReconnectStrategy::Custom(delays.clone());
        assert_eq!(strategy.next_delay(1), Some(delays[0]));
        assert_eq!(strategy.next_delay(2), Some(delays[1]));
        assert_eq!(strategy.next_delay(3), Some(delays[2]));
        assert_eq!(strategy.next_delay(4), None); // Out of bounds
    }

    #[test]
    fn test_reconnection_stats() {
        let mut stats = ReconnectionStats::new();
        assert_eq!(stats.total_attempts, 0);
        assert_eq!(stats.success_rate(), 0.0);
        assert!(stats.avg_reconnect_time().is_none());

        stats.total_attempts = 10;
        stats.successful_reconnections = 8;
        assert!((stats.success_rate() - 0.8).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_reconnect_manager_state() {
        let manager = ReconnectManager::default_config();

        assert_eq!(manager.state().await, ReconnectionState::Connected);
        assert!(manager.is_connected().await);

        manager.set_state(ReconnectionState::Disconnected).await;
        assert_eq!(manager.state().await, ReconnectionState::Disconnected);
        assert!(!manager.is_connected().await);
    }

    #[tokio::test]
    async fn test_reconnect_manager_attempts() {
        let config = ReconnectConfig::new(ReconnectStrategy::fixed(100))
            .with_max_attempts(3);
        let manager = ReconnectManager::new(config);

        manager.set_state(ReconnectionState::Disconnected).await;

        // Should allow reconnection initially
        assert!(manager.should_reconnect().await);

        // After max attempts, should not allow
        for _ in 0..3 {
            manager.record_attempt().await;
        }
        assert!(!manager.should_reconnect().await);
    }

    #[tokio::test]
    async fn test_reconnect_manager_reset() {
        let manager = ReconnectManager::default_config();

        manager.record_attempt().await;
        let stats = manager.stats().await;
        assert_eq!(stats.current_attempt, 1);

        manager.reset().await;
        let stats = manager.stats().await;
        assert_eq!(stats.current_attempt, 0);
        assert_eq!(manager.state().await, ReconnectionState::Connected);
    }

    #[test]
    fn test_reconnect_config_builder() {
        let config = ReconnectConfig::new(ReconnectStrategy::fixed(1000))
            .with_max_attempts(10)
            .with_timeout(Duration::from_secs(60))
            .with_reset_on_success(false);

        assert!(matches!(config.strategy, ReconnectStrategy::FixedDelay(_)));
        assert_eq!(config.max_attempts, Some(10));
        assert_eq!(config.connection_timeout, Duration::from_secs(60));
        assert!(!config.reset_on_success);
    }
}
