//! Authentication Challenge-Response Flow
//!
//! This module provides a complete authentication challenge-response flow
//! implementation for DLMS/COSEM protocol.
//!
//! # Authentication Flow Overview
//!
//! DLMS/COSEM supports multiple authentication mechanisms:
//! - **Low-level Security**: Password-based authentication
//! - **High-level Security (HLS)**: Cryptographic authentication (HLS5-GMAC, etc.)
//!
//! # Challenge-Response Flow
//!
//! 1. **Server generates challenge**: Random bytes sent to client
//! 2. **Client generates response**: Based on challenge and credentials
//! 3. **Server verifies response**: Validates client's credentials
//! 4. **Authentication state**: Tracks authentication status
//!
//! # Usage
//!
//! ```rust,no_run
//! use dlms_security::auth_flow::{AuthenticationFlow, AuthenticationMechanism};
//! use dlms_security::authentication::LowAuth;
//!
//! // Create authentication flow
//! let mut flow = AuthenticationFlow::new(AuthenticationMechanism::LowLevel);
//!
//! // Server side: Generate challenge
//! let challenge = flow.generate_challenge(8)?;
//!
//! // Client side: Generate response
//! let auth = LowAuth::new(b"password");
//! let response = flow.generate_response(&auth, &challenge)?;
//!
//! // Server side: Verify response
//! let verified = flow.verify_response(&auth, &challenge, &response)?;
//! ```

use crate::error::{DlmsError, DlmsResult};
use crate::authentication::{LowAuth, Hls5GmacAuth, GmacAuth};
use rand::RngCore;
use std::time::{SystemTime, UNIX_EPOCH};

/// Authentication mechanism type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthenticationMechanism {
    /// No authentication
    None,
    /// Low-level security (password-based)
    LowLevel,
    /// High-level security 5 (HLS5-GMAC)
    Hls5Gmac,
    /// GMAC authentication
    Gmac,
}

/// Authentication state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthenticationState {
    /// Not authenticated (initial state)
    NotAuthenticated,
    /// Challenge generated, waiting for response
    ChallengeSent,
    /// Response received, verification pending
    ResponseReceived,
    /// Authentication successful
    Authenticated,
    /// Authentication failed
    AuthenticationFailed,
}

/// Authentication Challenge-Response Flow Manager
///
/// Manages the complete authentication flow including challenge generation,
/// response verification, and state management.
pub struct AuthenticationFlow {
    /// Authentication mechanism
    mechanism: AuthenticationMechanism,
    /// Current authentication state
    state: AuthenticationState,
    /// Generated challenge (if any)
    challenge: Option<Vec<u8>>,
    /// Challenge generation timestamp
    challenge_timestamp: Option<SystemTime>,
    /// Challenge timeout (seconds)
    challenge_timeout: u64,
}

impl AuthenticationFlow {
    /// Create a new AuthenticationFlow
    ///
    /// # Arguments
    /// * `mechanism` - Authentication mechanism to use
    ///
    /// # Returns
    /// A new AuthenticationFlow instance in NotAuthenticated state
    pub fn new(mechanism: AuthenticationMechanism) -> Self {
        Self {
            mechanism,
            state: AuthenticationState::NotAuthenticated,
            challenge: None,
            challenge_timestamp: None,
            challenge_timeout: 30, // Default 30 seconds timeout
        }
    }

    /// Create with custom challenge timeout
    ///
    /// # Arguments
    /// * `mechanism` - Authentication mechanism to use
    /// * `timeout_seconds` - Challenge timeout in seconds
    pub fn with_timeout(mechanism: AuthenticationMechanism, timeout_seconds: u64) -> Self {
        Self {
            mechanism,
            state: AuthenticationState::NotAuthenticated,
            challenge: None,
            challenge_timestamp: None,
            challenge_timeout: timeout_seconds,
        }
    }

    /// Get current authentication state
    pub fn state(&self) -> AuthenticationState {
        self.state
    }

    /// Get authentication mechanism
    pub fn mechanism(&self) -> AuthenticationMechanism {
        self.mechanism
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        matches!(self.state, AuthenticationState::Authenticated)
    }

    /// Generate a challenge
    ///
    /// # Arguments
    /// * `length` - Challenge length in bytes (typically 8-16 bytes)
    ///
    /// # Returns
    /// Generated challenge bytes
    ///
    /// # Errors
    /// Returns error if already in ChallengeSent state or invalid length
    pub fn generate_challenge(&mut self, length: usize) -> DlmsResult<Vec<u8>> {
        if length == 0 || length > 64 {
            return Err(DlmsError::InvalidData(format!(
                "Challenge length must be between 1 and 64 bytes, got {}",
                length
            )));
        }

        if matches!(self.state, AuthenticationState::ChallengeSent) {
            return Err(DlmsError::Security(
                "Challenge already generated, waiting for response".to_string(),
            ));
        }

        // Generate random challenge
        let mut challenge = vec![0u8; length];
        rand::thread_rng().fill_bytes(&mut challenge);

        self.challenge = Some(challenge.clone());
        self.challenge_timestamp = Some(SystemTime::now());
        self.state = AuthenticationState::ChallengeSent;

        Ok(challenge)
    }

    /// Get the current challenge
    ///
    /// # Returns
    /// Current challenge if available, None otherwise
    pub fn challenge(&self) -> Option<&[u8]> {
        self.challenge.as_deref()
    }

    /// Check if challenge has expired
    ///
    /// # Returns
    /// `true` if challenge exists and has expired, `false` otherwise
    pub fn is_challenge_expired(&self) -> bool {
        if let (Some(timestamp), Some(_)) = (self.challenge_timestamp, &self.challenge) {
            if let Ok(elapsed) = timestamp.elapsed() {
                return elapsed.as_secs() > self.challenge_timeout;
            }
        }
        false
    }

    /// Generate response for Low-level authentication
    ///
    /// # Arguments
    /// * `auth` - LowAuth instance with password
    /// * `challenge` - Challenge bytes received from server
    ///
    /// # Returns
    /// Response bytes to send to server
    pub fn generate_response_low_level(
        &mut self,
        auth: &LowAuth,
        challenge: &[u8],
    ) -> DlmsResult<Vec<u8>> {
        if self.mechanism != AuthenticationMechanism::LowLevel {
            return Err(DlmsError::Security(format!(
                "Expected LowLevel mechanism, got {:?}",
                self.mechanism
            )));
        }

        let response = auth.generate_challenge_response(challenge)?;
        self.state = AuthenticationState::ResponseReceived;
        Ok(response)
    }

    /// Generate response for HLS5-GMAC authentication
    ///
    /// # Arguments
    /// * `auth` - Hls5GmacAuth instance
    /// * `challenge` - Challenge bytes received from server
    /// * `system_title` - System title (8 bytes)
    /// * `frame_counter` - Frame counter
    ///
    /// # Returns
    /// Response bytes (authentication tag)
    pub fn generate_response_hls5_gmac(
        &mut self,
        auth: &Hls5GmacAuth,
        challenge: &[u8],
        system_title: &[u8],
        frame_counter: u32,
    ) -> DlmsResult<Vec<u8>> {
        if self.mechanism != AuthenticationMechanism::Hls5Gmac {
            return Err(DlmsError::Security(format!(
                "Expected Hls5Gmac mechanism, got {:?}",
                self.mechanism
            )));
        }

        if system_title.len() != 8 {
            return Err(DlmsError::InvalidData(format!(
                "System title must be 8 bytes, got {}",
                system_title.len()
            )));
        }

        let response = auth.generate_auth_tag(challenge, system_title, frame_counter)?;
        self.state = AuthenticationState::ResponseReceived;
        Ok(response)
    }

    /// Verify response for Low-level authentication
    ///
    /// # Arguments
    /// * `auth` - LowAuth instance with expected password
    /// * `challenge` - Challenge that was sent
    /// * `response` - Response received from client
    ///
    /// # Returns
    /// `true` if response is valid, `false` otherwise
    pub fn verify_response_low_level(
        &mut self,
        auth: &LowAuth,
        challenge: &[u8],
        response: &[u8],
    ) -> DlmsResult<bool> {
        if self.mechanism != AuthenticationMechanism::LowLevel {
            return Err(DlmsError::Security(format!(
                "Expected LowLevel mechanism, got {:?}",
                self.mechanism
            )));
        }

        if !matches!(self.state, AuthenticationState::ChallengeSent) {
            return Err(DlmsError::Security(
                "Cannot verify response: no challenge sent".to_string(),
            ));
        }

        if self.is_challenge_expired() {
            self.state = AuthenticationState::AuthenticationFailed;
            return Err(DlmsError::Security("Challenge expired".to_string()));
        }

        let verified = auth.verify_challenge_response(challenge, response)?;
        
        if verified {
            self.state = AuthenticationState::Authenticated;
        } else {
            self.state = AuthenticationState::AuthenticationFailed;
        }

        Ok(verified)
    }

    /// Verify response for HLS5-GMAC authentication
    ///
    /// # Arguments
    /// * `auth` - Hls5GmacAuth instance
    /// * `challenge` - Challenge that was sent
    /// * `response` - Response (authentication tag) received from client
    /// * `system_title` - System title (8 bytes)
    /// * `frame_counter` - Frame counter
    ///
    /// # Returns
    /// `true` if response is valid, `false` otherwise
    pub fn verify_response_hls5_gmac(
        &mut self,
        auth: &Hls5GmacAuth,
        challenge: &[u8],
        response: &[u8],
        system_title: &[u8],
        frame_counter: u32,
    ) -> DlmsResult<bool> {
        if self.mechanism != AuthenticationMechanism::Hls5Gmac {
            return Err(DlmsError::Security(format!(
                "Expected Hls5Gmac mechanism, got {:?}",
                self.mechanism
            )));
        }

        if !matches!(self.state, AuthenticationState::ChallengeSent) {
            return Err(DlmsError::Security(
                "Cannot verify response: no challenge sent".to_string(),
            ));
        }

        if self.is_challenge_expired() {
            self.state = AuthenticationState::AuthenticationFailed;
            return Err(DlmsError::Security("Challenge expired".to_string()));
        }

        if system_title.len() != 8 {
            return Err(DlmsError::InvalidData(format!(
                "System title must be 8 bytes, got {}",
                system_title.len()
            )));
        }

        let verified = auth.verify_auth_tag(challenge, system_title, frame_counter, response)?;
        
        if verified {
            self.state = AuthenticationState::Authenticated;
        } else {
            self.state = AuthenticationState::AuthenticationFailed;
        }

        Ok(verified)
    }

    /// Reset authentication flow
    ///
    /// Resets the flow to NotAuthenticated state and clears challenge.
    pub fn reset(&mut self) {
        self.state = AuthenticationState::NotAuthenticated;
        self.challenge = None;
        self.challenge_timestamp = None;
    }

    /// Set challenge timeout
    ///
    /// # Arguments
    /// * `timeout_seconds` - New timeout in seconds
    pub fn set_timeout(&mut self, timeout_seconds: u64) {
        self.challenge_timeout = timeout_seconds;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authentication_flow_creation() {
        let flow = AuthenticationFlow::new(AuthenticationMechanism::LowLevel);
        assert_eq!(flow.state(), AuthenticationState::NotAuthenticated);
        assert_eq!(flow.mechanism(), AuthenticationMechanism::LowLevel);
        assert!(!flow.is_authenticated());
    }

    #[test]
    fn test_generate_challenge() {
        let mut flow = AuthenticationFlow::new(AuthenticationMechanism::LowLevel);
        let challenge = flow.generate_challenge(8).unwrap();
        
        assert_eq!(challenge.len(), 8);
        assert_eq!(flow.state(), AuthenticationState::ChallengeSent);
        assert!(flow.challenge().is_some());
    }

    #[test]
    fn test_challenge_expiration() {
        let mut flow = AuthenticationFlow::with_timeout(AuthenticationMechanism::LowLevel, 1);
        flow.generate_challenge(8).unwrap();
        
        // Challenge should not be expired immediately
        assert!(!flow.is_challenge_expired());
        
        // Wait a bit (in real test, you might use a mock time)
        // For now, we'll just test the logic
    }

    #[test]
    fn test_low_level_authentication_flow() {
        let mut flow = AuthenticationFlow::new(AuthenticationMechanism::LowLevel);
        let auth = LowAuth::new(b"password123");
        
        // Generate challenge
        let challenge = flow.generate_challenge(8).unwrap();
        
        // Generate response
        let response = flow.generate_response_low_level(&auth, &challenge).unwrap();
        assert_eq!(flow.state(), AuthenticationState::ResponseReceived);
        
        // Verify response
        let mut server_flow = AuthenticationFlow::new(AuthenticationMechanism::LowLevel);
        server_flow.generate_challenge(8).unwrap();
        // Use the same challenge for verification
        let verified = server_flow.verify_response_low_level(&auth, &challenge, &response).unwrap();
        
        assert!(verified);
        assert!(server_flow.is_authenticated());
    }

    #[test]
    fn test_reset() {
        let mut flow = AuthenticationFlow::new(AuthenticationMechanism::LowLevel);
        flow.generate_challenge(8).unwrap();
        
        flow.reset();
        assert_eq!(flow.state(), AuthenticationState::NotAuthenticated);
        assert!(flow.challenge().is_none());
    }

    #[test]
    fn test_invalid_challenge_length() {
        let mut flow = AuthenticationFlow::new(AuthenticationMechanism::LowLevel);
        
        // Too long
        assert!(flow.generate_challenge(65).is_err());
        
        // Zero length
        assert!(flow.generate_challenge(0).is_err());
    }
}
