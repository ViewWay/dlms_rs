//! Server state management
//!
//! This module provides state machine functionality for managing the DLMS/COSEM server
//! lifecycle, including startup, operation, and graceful shutdown.

use crate::error::{ServerErrorCode, DlmsServerError};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Server state in the lifecycle state machine
///
/// # State Transitions
/// ```text
///      ┌─────────┐
///      │  Idle   │
///      └────┬────┘
///           │ start()
///           ▼
///      ┌─────────┐
///      │Starting │
///      └────┬────┘
///           │ [ready]
///           ▼
///      ┌─────────┐    shutdown()
/// ──▶  │ Running │ ──────────────────────┐
///      └────┬────┘                        │
///           │                             │
///      ┌────┴────┐                        │
///      │  Error  │                        │
///      └────┬────┘                        ▼
///           │                    ┌─────────────┐
///           └───────────────────▶│   Stopping  │
///                                 └──────┬──────┘
///                                        │ [done]
///                                        ▼
///                                  ┌─────────┐
///                                  │ Stopped │
///                                  └─────────┘
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// Server is created but not started
    Idle,
    /// Server is initializing
    Starting,
    /// Server is running and accepting connections
    Running,
    /// Server is shutting down (graceful shutdown in progress)
    Stopping,
    /// Server has stopped
    Stopped,
    /// Server encountered an error
    Error(ServerErrorCode),
}

impl ServerState {
    /// Check if this state allows accepting new connections
    pub fn can_accept_connections(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if this state allows processing requests
    pub fn can_process_requests(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if this state is a terminal state (no further transitions)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Stopped | Self::Error(_))
    }

    /// Check if this state is operational (server can do useful work)
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Starting | Self::Running)
    }

    /// Get the state name as a string
    pub fn name(&self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Starting => "Starting",
            Self::Running => "Running",
            Self::Stopping => "Stopping",
            Self::Stopped => "Stopped",
            Self::Error(_code) => "Error",
        }
    }
}

impl std::fmt::Display for ServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error(code) => write!(f, "Error({})", code),
            _ => write!(f, "{}", self.name()),
        }
    }
}

/// State transition information
#[derive(Debug, Clone)]
pub struct StateTransition {
    /// The previous state
    pub from: ServerState,
    /// The new state
    pub to: ServerState,
    /// When the transition occurred
    pub timestamp: Instant,
    /// Reason for the transition
    pub reason: String,
}

impl StateTransition {
    /// Create a new state transition record
    pub fn new(from: ServerState, to: ServerState, reason: impl Into<String>) -> Self {
        Self {
            from,
            to,
            timestamp: Instant::now(),
            reason: reason.into(),
        }
    }
}

/// Callback type for state change notifications
pub type StateChangeCallback = Arc<dyn Fn(ServerState, ServerState) + Send + Sync>;

/// Server state machine
///
/// Manages server lifecycle states and transitions with validation.
pub struct ServerStateMachine {
    /// Current state
    state: Arc<RwLock<ServerState>>,
    /// State transition history
    history: Arc<RwLock<Vec<StateTransition>>>,
    /// Maximum history size
    max_history: usize,
    /// Optional state change callback
    callback: Arc<RwLock<Option<StateChangeCallback>>>,
}

impl ServerStateMachine {
    /// Create a new state machine in Idle state
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ServerState::Idle)),
            history: Arc::new(RwLock::new(Vec::new())),
            max_history: 100,
            callback: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the current state
    pub async fn current_state(&self) -> ServerState {
        *self.state.read().await
    }

    /// Check if the server is in a specific state
    pub async fn is_state(&self, state: ServerState) -> bool {
        self.current_state().await == state
    }

    /// Check if the server can accept connections
    pub async fn can_accept_connections(&self) -> bool {
        self.current_state().await.can_accept_connections()
    }

    /// Check if the server can process requests
    pub async fn can_process_requests(&self) -> bool {
        self.current_state().await.can_process_requests()
    }

    /// Transition to a new state
    ///
    /// # Arguments
    /// * `new_state` - The target state
    /// * `reason` - Reason for the transition
    ///
    /// # Errors
    /// Returns error if the transition is not valid
    pub async fn transition_to(
        &self,
        new_state: ServerState,
        reason: impl Into<String>,
    ) -> Result<(), DlmsServerError> {
        let current = self.current_state().await;
        let reason = reason.into();

        // Validate state transition
        self.validate_transition(current, new_state)?;

        // Perform transition
        {
            let mut state = self.state.write().await;
            *state = new_state;
        }

        // Record transition
        let transition = StateTransition::new(current, new_state, reason.clone());
        self.record_transition(transition).await;

        // Invoke callback if registered
        let callback = self.callback.read().await.clone();
        if let Some(cb) = callback {
            cb(current, new_state);
        }

        Ok(())
    }

    /// Validate that a state transition is allowed
    fn validate_transition(
        &self,
        from: ServerState,
        to: ServerState,
    ) -> Result<(), DlmsServerError> {
        // Same state is always valid (no-op)
        if from == to {
            return Ok(());
        }

        match (from, to) {
            // From Idle
            (ServerState::Idle, ServerState::Starting) => Ok(()),
            (ServerState::Idle, ServerState::Error(_)) => Ok(()),

            // From Starting
            (ServerState::Starting, ServerState::Running) => Ok(()),
            (ServerState::Starting, ServerState::Error(_)) => Ok(()),
            (ServerState::Starting, ServerState::Stopping) => Ok(()),

            // From Running
            (ServerState::Running, ServerState::Stopping) => Ok(()),
            (ServerState::Running, ServerState::Error(_)) => Ok(()),

            // From Stopping
            (ServerState::Stopping, ServerState::Stopped) => Ok(()),
            (ServerState::Stopping, ServerState::Error(_)) => Ok(()),

            // From Stopped or Error - no valid transitions
            (ServerState::Stopped, _) | (ServerState::Error(_), _) => {
                Err(DlmsServerError::InvalidStateTransition {
                    from: from.to_string(),
                    to: to.to_string(),
                })
            }

            // All other transitions are invalid
            _ => Err(DlmsServerError::InvalidStateTransition {
                from: from.to_string(),
                to: to.to_string(),
            }),
        }
    }

    /// Record a state transition in history
    async fn record_transition(&self, transition: StateTransition) {
        let mut history = self.history.write().await;
        history.push(transition);

        // Trim history if needed
        let len = history.len();
        if len > self.max_history {
            history.drain(0..len - self.max_history);
        }
    }

    /// Get state transition history
    pub async fn history(&self) -> Vec<StateTransition> {
        self.history.read().await.clone()
    }

    /// Clear state transition history
    pub async fn clear_history(&self) {
        self.history.write().await.clear();
    }

    /// Set a callback to be invoked on state changes
    pub async fn set_state_change_callback(&self, callback: StateChangeCallback) {
        let mut cb = self.callback.write().await;
        *cb = Some(callback);
    }

    /// Remove the state change callback
    pub async fn remove_state_change_callback(&self) {
        let mut cb = self.callback.write().await;
        *cb = None;
    }

    /// Start the server (Idle -> Starting -> Running)
    ///
    /// This is a convenience method that performs the startup transitions.
    pub async fn start(&self) -> Result<(), DlmsServerError> {
        if !self.is_state(ServerState::Idle).await {
            return Err(DlmsServerError::InvalidState {
                expected: "Idle".to_string(),
                actual: self.current_state().await.to_string(),
            });
        }

        self.transition_to(ServerState::Starting, "Server start requested").await?;

        // In a real implementation, you would perform initialization here
        // For now, we immediately transition to Running
        self.transition_to(ServerState::Running, "Initialization complete").await?;

        Ok(())
    }

    /// Stop the server (Running -> Stopping -> Stopped)
    ///
    /// This is a convenience method that performs the shutdown transitions.
    pub async fn stop(&self) -> Result<(), DlmsServerError> {
        let current = self.current_state().await;

        if !matches!(current, ServerState::Running | ServerState::Starting) {
            return Err(DlmsServerError::InvalidState {
                expected: "Running or Starting".to_string(),
                actual: current.to_string(),
            });
        }

        self.transition_to(ServerState::Stopping, "Server stop requested").await?;

        // In a real implementation, you would perform graceful shutdown here
        self.transition_to(ServerState::Stopped, "Shutdown complete").await?;

        Ok(())
    }

    /// Set error state
    ///
    /// Transitions to Error state from any non-terminal state.
    pub async fn set_error(&self, error: ServerErrorCode, reason: impl Into<String>) {
        let current = self.current_state().await;

        // Can only transition to error from non-terminal states
        if !current.is_terminal() {
            let _ = self
                .transition_to(ServerState::Error(error), reason)
                .await;
        }
    }

    /// Check if the server is in a healthy state
    pub async fn is_healthy(&self) -> bool {
        matches!(
            self.current_state().await,
            ServerState::Idle | ServerState::Starting | ServerState::Running
        )
    }
}

impl Default for ServerStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// Server status information
#[derive(Debug, Clone)]
pub struct ServerStatus {
    /// Current server state
    pub state: ServerState,
    /// Whether the server can accept connections
    pub can_accept_connections: bool,
    /// Whether the server can process requests
    pub can_process_requests: bool,
    /// Whether the server is healthy
    pub is_healthy: bool,
    /// Last state change timestamp
    pub last_state_change: Option<Instant>,
    /// Number of state transitions
    pub total_transitions: usize,
}

impl ServerStatus {
    /// Get current server status from state machine
    pub async fn from_state_machine(state_machine: &ServerStateMachine) -> Self {
        let state = state_machine.current_state().await;
        let history = state_machine.history().await;
        let last_change = history.last().map(|t| t.timestamp);

        Self {
            can_accept_connections: state.can_accept_connections(),
            can_process_requests: state.can_process_requests(),
            is_healthy: state_machine.is_healthy().await,
            state,
            last_state_change: last_change,
            total_transitions: history.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initial_state() {
        let sm = ServerStateMachine::new();
        assert_eq!(sm.current_state().await, ServerState::Idle);
        assert!(sm.is_healthy().await);
    }

    #[tokio::test]
    async fn test_valid_transitions() {
        let sm = ServerStateMachine::new();

        // Idle -> Starting
        sm.transition_to(ServerState::Starting, "test").await.unwrap();
        assert_eq!(sm.current_state().await, ServerState::Starting);

        // Starting -> Running
        sm.transition_to(ServerState::Running, "test").await.unwrap();
        assert_eq!(sm.current_state().await, ServerState::Running);

        // Running -> Stopping
        sm.transition_to(ServerState::Stopping, "test").await.unwrap();
        assert_eq!(sm.current_state().await, ServerState::Stopping);

        // Stopping -> Stopped
        sm.transition_to(ServerState::Stopped, "test").await.unwrap();
        assert_eq!(sm.current_state().await, ServerState::Stopped);
    }

    #[tokio::test]
    async fn test_invalid_transitions() {
        let sm = ServerStateMachine::new();

        // Idle -> Running (should fail - must go through Starting)
        assert!(sm
            .transition_to(ServerState::Running, "test")
            .await
            .is_err());

        // Idle -> Stopping (should fail)
        assert!(sm
            .transition_to(ServerState::Stopping, "test")
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_error_from_non_terminal() {
        let sm = ServerStateMachine::new();

        // Idle -> Error
        sm.set_error(ServerErrorCode::IoError, "Test error").await;
        assert!(matches!(
            sm.current_state().await,
            ServerState::Error(ServerErrorCode::IoError)
        ));
    }

    #[tokio::test]
    async fn test_no_transition_from_stopped() {
        let sm = ServerStateMachine::new();

        // Go to Stopped state
        sm.transition_to(ServerState::Starting, "test").await.unwrap();
        sm.transition_to(ServerState::Running, "test").await.unwrap();
        sm.transition_to(ServerState::Stopping, "test").await.unwrap();
        sm.transition_to(ServerState::Stopped, "test").await.unwrap();

        // Stopped -> Running should fail
        assert!(sm
            .transition_to(ServerState::Running, "test")
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_start_method() {
        let sm = ServerStateMachine::new();
        sm.start().await.unwrap();
        assert_eq!(sm.current_state().await, ServerState::Running);
        assert!(sm.can_accept_connections().await);
    }

    #[tokio::test]
    async fn test_stop_method() {
        let sm = ServerStateMachine::new();
        sm.start().await.unwrap();
        sm.stop().await.unwrap();
        assert_eq!(sm.current_state().await, ServerState::Stopped);
        assert!(!sm.can_accept_connections().await);
    }

    #[tokio::test]
    async fn test_history_tracking() {
        let sm = ServerStateMachine::new();

        sm.transition_to(ServerState::Starting, "reason1").await.unwrap();
        sm.transition_to(ServerState::Running, "reason2").await.unwrap();

        let history = sm.history().await;
        assert_eq!(history.len(), 2); // 2 transitions
        assert_eq!(history[0].from, ServerState::Idle);
        assert_eq!(history[0].to, ServerState::Starting);
        assert_eq!(history[0].reason, "reason1");
        assert_eq!(history[1].to, ServerState::Running);
        assert_eq!(history[1].reason, "reason2");
    }

    #[tokio::test]
    async fn test_state_callback() {
        let sm = ServerStateMachine::new();
        let callback_called = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let cb = {
            let callback_called = callback_called.clone();
            Arc::new(move |_from, _to| {
                callback_called.store(true, std::sync::atomic::Ordering::SeqCst);
            }) as StateChangeCallback
        };

        sm.set_state_change_callback(cb).await;
        sm.transition_to(ServerState::Starting, "test").await.unwrap();

        assert!(callback_called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_server_status() {
        let sm = ServerStateMachine::new();
        sm.start().await.unwrap();

        let status = ServerStatus::from_state_machine(&sm).await;
        assert_eq!(status.state, ServerState::Running);
        assert!(status.can_accept_connections);
        assert!(status.can_process_requests);
        assert!(status.is_healthy);
        assert!(status.total_transitions > 0);
    }

    #[tokio::test]
    async fn test_error_code_from_value() {
        assert_eq!(
            ServerErrorCode::from_value(1),
            Some(ServerErrorCode::InitializationFailed)
        );
        assert_eq!(
            ServerErrorCode::from_value(10),
            Some(ServerErrorCode::Timeout)
        );
        assert_eq!(ServerErrorCode::from_value(99), None);
    }

    #[tokio::test]
    async fn test_error_code_display() {
        let code = ServerErrorCode::BindFailed;
        assert_eq!(code.to_string(), "Failed to bind to address");
        assert_eq!(code.value(), 2);
    }
}
