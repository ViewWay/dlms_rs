//! Key Management
//!
//! This module provides comprehensive key management functionality for DLMS/COSEM
//! security operations.
//!
//! # Overview
//!
//! Key management is critical for secure DLMS/COSEM operations. This module provides:
//!
//! - **Key Storage**: Secure storage interface for keys
//! - **Key Rotation**: Automated key rotation with configurable policies
//! - **KEK Management**: Master key (Key Encryption Key) management
//! - **Key Lifecycle**: Complete key lifecycle from generation to destruction
//!
//! # Usage
//!
//! ```rust,no_run
//! use dlms_security::key_management::{
//!     KeyManager, KeyStorage, KeyRotationPolicy, ProtectedKey,
//! };
//! use std::time::Duration;
//!
//! // Create a key manager with in-memory storage
//! let mut key_manager = KeyManager::new(InMemoryKeyStorage::new());
//!
//! // Generate and store a master key
//! let kek = key_manager.generate_kek()?;
//!
//! // Derive and store session keys
//! let enc_key = key_manager.derive_encryption_key(&kek, &system_title)?;
//! key_manager.store_key("enc_key", &enc_key)?;
//!
//! // Check for key rotation
//! if key_manager.should_rotate_key("enc_key").await {
//!     key_manager.rotate_key("enc_key").await?;
//! }
//! ```

use crate::error::{DlmsError, DlmsResult};
use crate::xdlms::{SystemTitle, KeyDerivationFunction};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use rand::RngCore;

/// Key identifier
pub type KeyIdStr = String;

/// Protected key wrapper
///
/// Wraps key material with additional metadata for secure handling.
#[derive(Debug, Clone)]
pub struct ProtectedKey {
    /// The key material
    key: Vec<u8>,
    /// Key identifier
    id: KeyIdStr,
    /// Key type
    key_type: KeyType,
    /// Creation timestamp
    created_at: SystemTime,
    /// Last rotation timestamp
    last_rotated: Option<SystemTime>,
    /// Usage count
    usage_count: u64,
    /// Maximum usage count (0 for unlimited)
    max_usage: u64,
    /// Whether the key is mutable
    mutable: bool,
}

impl ProtectedKey {
    /// Create a new protected key
    pub fn new(
        key: Vec<u8>,
        id: KeyIdStr,
        key_type: KeyType,
    ) -> Self {
        Self {
            key,
            id,
            key_type,
            created_at: SystemTime::now(),
            last_rotated: None,
            usage_count: 0,
            max_usage: 0,
            mutable: true,
        }
    }

    /// Get the key material
    pub fn key(&self) -> &[u8] {
        &self.key
    }

    /// Get the key identifier
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the key type
    pub fn key_type(&self) -> KeyType {
        self.key_type
    }

    /// Get the creation time
    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }

    /// Get the last rotation time
    pub fn last_rotated(&self) -> Option<SystemTime> {
        self.last_rotated
    }

    /// Get the usage count
    pub fn usage_count(&self) -> u64 {
        self.usage_count
    }

    /// Check if the key is mutable
    pub fn is_mutable(&self) -> bool {
        self.mutable
    }

    /// Check if the key is expired based on age
    pub fn is_expired(&self, max_age: Duration) -> bool {
        self.created_at
            .elapsed()
            .unwrap_or(Duration::ZERO)
            > max_age
    }

    /// Check if the key has exceeded max usage
    pub fn is_exhausted(&self) -> bool {
        self.max_usage > 0 && self.usage_count >= self.max_usage
    }

    /// Increment usage count
    pub fn increment_usage(&mut self) {
        self.usage_count = self.usage_count.saturating_add(1);
    }

    /// Mark as rotated (update last_rotated timestamp)
    pub fn mark_as_rotated(&mut self) {
        self.last_rotated = Some(SystemTime::now());
        self.usage_count = 0;
    }

    /// Make key immutable
    pub fn make_immutable(&mut self) {
        self.mutable = false;
    }

    /// Check if key should be rotated
    pub fn needs_rotation(&self, policy: &KeyRotationPolicy) -> bool {
        // Check age
        if let Some(max_age) = policy.max_age {
            if self.is_expired(max_age) {
                return true;
            }
        }

        // Check usage
        if let Some(max_usage) = policy.max_usage {
            if self.usage_count >= max_usage {
                return true;
            }
        }

        false
    }

    /// Securely zero out the key material
    pub fn secure_zero(&mut self) {
        for byte in &mut self.key {
            *byte = 0;
        }
        self.mutable = false;
    }
}

impl Drop for ProtectedKey {
    fn drop(&mut self) {
        // Zero out key material on drop
        self.secure_zero();
    }
}

/// Key type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyType {
    /// Master key (KEK - Key Encryption Key)
    MasterKek,
    /// Encryption key (unicast)
    EncryptionUnicast,
    /// Encryption key (broadcast)
    EncryptionBroadcast,
    /// Authentication key
    Authentication,
    /// Password
    Password,
    /// Derived key
    Derived,
    /// Wrapped key
    Wrapped,
}

impl fmt::Display for KeyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyType::MasterKek => write!(f, "MasterKEK"),
            KeyType::EncryptionUnicast => write!(f, "EncryptionUnicast"),
            KeyType::EncryptionBroadcast => write!(f, "EncryptionBroadcast"),
            KeyType::Authentication => write!(f, "Authentication"),
            KeyType::Password => write!(f, "Password"),
            KeyType::Derived => write!(f, "Derived"),
            KeyType::Wrapped => write!(f, "Wrapped"),
        }
    }
}

/// Key rotation policy
#[derive(Debug, Clone)]
pub struct KeyRotationPolicy {
    /// Maximum age of a key before rotation
    pub max_age: Option<Duration>,
    /// Maximum number of uses before rotation
    pub max_usage: Option<u64>,
    /// Whether automatic rotation is enabled
    pub auto_rotate: bool,
}

impl KeyRotationPolicy {
    /// Create a new key rotation policy
    pub fn new() -> Self {
        Self {
            max_age: None,
            max_usage: None,
            auto_rotate: false,
        }
    }

    /// Set maximum key age
    pub fn with_max_age(mut self, age: Duration) -> Self {
        self.max_age = Some(age);
        self
    }

    /// Set maximum usage count
    pub fn with_max_usage(mut self, usage: u64) -> Self {
        self.max_usage = Some(usage);
        self
    }

    /// Enable automatic rotation
    pub fn with_auto_rotate(mut self, enabled: bool) -> Self {
        self.auto_rotate = enabled;
        self
    }

    /// Create a policy with common defaults (90 days, 1M uses)
    pub fn with_common_defaults() -> Self {
        Self::new()
            .with_max_age(Duration::from_secs(90 * 24 * 60 * 60))
            .with_max_usage(1_000_000)
    }
}

impl Default for KeyRotationPolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// Key storage trait
///
/// Defines the interface for storing and retrieving keys securely.
pub trait KeyStorage: Send + Sync {
    /// Store a key
    fn store(&self, id: &str, key: &ProtectedKey) -> DlmsResult<()>;

    /// Retrieve a key
    fn retrieve(&self, id: &str) -> DlmsResult<ProtectedKey>;

    /// Delete a key
    fn delete(&self, id: &str) -> DlmsResult<()>;

    /// Check if a key exists
    fn exists(&self, id: &str) -> DlmsResult<bool>;

    /// List all key IDs
    fn list_keys(&self) -> DlmsResult<Vec<String>>;
}

/// In-memory key storage (for testing/non-persistent use)
#[derive(Debug, Clone)]
pub struct InMemoryKeyStorage {
    keys: Arc<RwLock<HashMap<String, ProtectedKey>>>,
}

impl InMemoryKeyStorage {
    /// Create a new in-memory key storage
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryKeyStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyStorage for InMemoryKeyStorage {
    fn store(&self, id: &str, key: &ProtectedKey) -> DlmsResult<()> {
        let mut keys = self.keys.write().unwrap();
        keys.insert(id.to_string(), key.clone());
        Ok(())
    }

    fn retrieve(&self, id: &str) -> DlmsResult<ProtectedKey> {
        let keys = self.keys.read().unwrap();
        keys.get(id)
            .cloned()
            .ok_or_else(|| DlmsError::Security(format!("Key not found: {}", id)))
    }

    fn delete(&self, id: &str) -> DlmsResult<()> {
        let mut keys = self.keys.write().unwrap();
        keys.remove(id)
            .ok_or_else(|| DlmsError::Security(format!("Key not found: {}", id)))?;
        Ok(())
    }

    fn exists(&self, id: &str) -> DlmsResult<bool> {
        let keys = self.keys.read().unwrap();
        Ok(keys.contains_key(id))
    }

    fn list_keys(&self) -> DlmsResult<Vec<String>> {
        let keys = self.keys.read().unwrap();
        Ok(keys.keys().cloned().collect())
    }
}

/// Key manager
///
/// Central component for managing keys throughout their lifecycle.
pub struct KeyManager {
    /// Key storage backend
    storage: Arc<dyn KeyStorage>,
    /// Rotation policy
    rotation_policy: KeyRotationPolicy,
    /// KEK (master key) - stored separately for security
    kek: Option<ProtectedKey>,
}

impl KeyManager {
    /// Create a new key manager
    pub fn new(storage: Arc<dyn KeyStorage>) -> Self {
        Self {
            storage,
            rotation_policy: KeyRotationPolicy::default(),
            kek: None,
        }
    }

    /// Create with rotation policy
    pub fn with_rotation_policy(
        storage: Arc<dyn KeyStorage>,
        policy: KeyRotationPolicy,
    ) -> Self {
        Self {
            storage,
            rotation_policy: policy,
            kek: None,
        }
    }

    /// Generate a new KEK (master key)
    ///
    /// The KEK is used to derive other keys and should be kept highly secure.
    pub fn generate_kek(&mut self) -> DlmsResult<ProtectedKey> {
        let mut key_bytes = vec![0u8; 32]; // 256-bit KEK
        rand::thread_rng().fill_bytes(&mut key_bytes);

        let protected_key = ProtectedKey::new(
            key_bytes,
            "master_kek".to_string(),
            KeyType::MasterKek,
        );

        // Store KEK
        self.storage.store("master_kek", &protected_key)?;

        // Keep reference
        self.kek = Some(protected_key.clone());

        Ok(protected_key)
    }

    /// Get the KEK
    pub fn kek(&self) -> DlmsResult<&ProtectedKey> {
        self.kek
            .as_ref()
            .ok_or_else(|| DlmsError::Security("KEK not initialized".to_string()))
    }

    /// Load KEK from storage
    pub fn load_kek(&mut self) -> DlmsResult<()> {
        self.kek = Some(self.storage.retrieve("master_kek")?);
        Ok(())
    }

    /// Generate a random AES-128 key
    pub fn generate_key_128(&self) -> Vec<u8> {
        let mut key = vec![0u8; 16];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }

    /// Generate a random AES-256 key
    pub fn generate_key_256(&self) -> Vec<u8> {
        let mut key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }

    /// Derive an encryption key using KDF
    pub fn derive_encryption_key(
        &self,
        kek: &ProtectedKey,
        system_title: &SystemTitle,
    ) -> DlmsResult<Vec<u8>> {
        KeyDerivationFunction::derive_unicast_encryption_key(kek.key(), system_title)
    }

    /// Derive an authentication key using KDF
    pub fn derive_authentication_key(
        &self,
        kek: &ProtectedKey,
        system_title: &SystemTitle,
    ) -> DlmsResult<Vec<u8>> {
        KeyDerivationFunction::derive_unicast_authentication_key(kek.key(), system_title)
    }

    /// Store a key
    pub fn store_key(&self, id: &str, key: &[u8]) -> DlmsResult<()> {
        let protected_key = ProtectedKey::new(
            key.to_vec(),
            id.to_string(),
            KeyType::Derived,
        );
        self.storage.store(id, &protected_key)
    }

    /// Retrieve a key
    pub fn retrieve_key(&self, id: &str) -> DlmsResult<Vec<u8>> {
        let protected_key = self.storage.retrieve(id)?;
        Ok(protected_key.key().to_vec())
    }

    /// Delete a key
    pub fn delete_key(&self, id: &str) -> DlmsResult<()> {
        self.storage.delete(id)
    }

    /// Check if a key exists
    pub fn key_exists(&self, id: &str) -> DlmsResult<bool> {
        self.storage.exists(id)
    }

    /// List all stored key IDs
    pub fn list_keys(&self) -> DlmsResult<Vec<String>> {
        self.storage.list_keys()
    }

    /// Check if a key needs rotation
    pub fn should_rotate_key(&self, id: &str) -> DlmsResult<bool> {
        match self.storage.retrieve(id) {
            Ok(key) => Ok(key.needs_rotation(&self.rotation_policy)),
            Err(_) => Ok(false),
        }
    }

    /// Rotate a key
    pub fn rotate_key(&self, id: &str) -> DlmsResult<Vec<u8>> {
        let old_key = self.storage.retrieve(id)?;

        // Generate new key of same length
        let mut new_key_bytes = vec![0u8; old_key.key().len()];
        rand::thread_rng().fill_bytes(&mut new_key_bytes);

        // Create new protected key
        let mut new_key = ProtectedKey::new(
            new_key_bytes,
            id.to_string(),
            old_key.key_type(),
        );
        new_key.mark_as_rotated();

        // Store new key
        self.storage.store(id, &new_key)?;

        Ok(new_key.key().to_vec())
    }

    /// Rotate all keys that need rotation
    pub fn rotate_due_keys(&self) -> DlmsResult<Vec<String>> {
        let keys = self.list_keys()?;
        let mut rotated = vec![];

        for id in keys {
            if self.should_rotate_key(&id)? {
                self.rotate_key(&id)?;
                rotated.push(id);
            }
        }

        Ok(rotated)
    }

    /// Update rotation policy
    pub fn set_rotation_policy(&mut self, policy: KeyRotationPolicy) {
        self.rotation_policy = policy;
    }

    /// Get rotation policy
    pub fn rotation_policy(&self) -> &KeyRotationPolicy {
        &self.rotation_policy
    }

    /// Derive and store session keys for a connection
    pub fn setup_session_keys(
        &self,
        system_title: &SystemTitle,
    ) -> DlmsResult<SessionKeys> {
        let kek = self.kek()?;

        // Derive encryption key
        let enc_key = self.derive_encryption_key(kek, system_title)?;

        // Derive authentication key
        let auth_key = self.derive_authentication_key(kek, system_title)?;

        // Store with temporary IDs
        self.store_key("session_enc", &enc_key)?;
        self.store_key("session_auth", &auth_key)?;

        Ok(SessionKeys {
            encryption_key: enc_key,
            authentication_key: auth_key,
        })
    }

    /// Clear session keys
    pub fn clear_session_keys(&self) -> DlmsResult<()> {
        self.delete_key("session_enc")?;
        self.delete_key("session_auth")?;
        Ok(())
    }
}

impl<S> From<S> for KeyManager
where
    S: KeyStorage + Send + Sync + 'static,
{
    fn from(storage: S) -> Self {
        Self::new(Arc::new(storage))
    }
}

/// Session keys (temporary keys for a connection)
#[derive(Debug, Clone)]
pub struct SessionKeys {
    /// Encryption key
    pub encryption_key: Vec<u8>,
    /// Authentication key
    pub authentication_key: Vec<u8>,
}

/// Key generator utility
pub struct KeyGenerator;

impl KeyGenerator {
    /// Generate a random key of specified length
    pub fn generate(length: usize) -> Vec<u8> {
        let mut key = vec![0u8; length];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }

    /// Generate an AES-128 key (16 bytes)
    pub fn aes_128() -> Vec<u8> {
        Self::generate(16)
    }

    /// Generate an AES-192 key (24 bytes)
    pub fn aes_192() -> Vec<u8> {
        Self::generate(24)
    }

    /// Generate an AES-256 key (32 bytes)
    pub fn aes_256() -> Vec<u8> {
        Self::generate(32)
    }

    /// Generate a key from password using PBKDF2
    ///
    /// This is a simplified version - in production, use proper password hashing.
    pub fn from_password(password: &[u8], salt: &[u8], iterations: u32, length: usize) -> Vec<u8> {
        use ring::pbkdf2;
        use std::num::NonZeroU32;

        let iterations = NonZeroU32::new(iterations).unwrap_or_else(|| NonZeroU32::new(100_000).unwrap());
        let mut key = vec![0u8; length];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            iterations,
            salt,
            password,
            &mut key,
        );
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protected_key_creation() {
        let key = ProtectedKey::new(
            vec![1, 2, 3, 4, 5, 6, 7, 8],
            "test_key".to_string(),
            KeyType::EncryptionUnicast,
        );

        assert_eq!(key.id(), "test_key");
        assert_eq!(key.key_type(), KeyType::EncryptionUnicast);
        assert_eq!(key.usage_count(), 0);
        assert!(key.is_mutable());
    }

    #[test]
    fn test_protected_key_expiration() {
        let key = ProtectedKey::new(
            vec![0u8; 16],
            "test_key".to_string(),
            KeyType::EncryptionUnicast,
        );

        // Should not be expired immediately
        assert!(!key.is_expired(Duration::from_secs(60)));

        // Simulate old key (mock)
        // In real test, would need to manipulate created_at
    }

    #[test]
    fn test_protected_key_exhaustion() {
        let mut key = ProtectedKey::new(
            vec![0u8; 16],
            "test_key".to_string(),
            KeyType::EncryptionUnicast,
        );

        // Set max usage to 5
        key.max_usage = 5;

        // Use 5 times
        for _ in 0..5 {
            key.increment_usage();
        }

        assert!(key.is_exhausted());
    }

    #[test]
    fn test_protected_key_rotation() {
        let mut key = ProtectedKey::new(
            vec![0u8; 16],
            "test_key".to_string(),
            KeyType::EncryptionUnicast,
        );

        let policy = KeyRotationPolicy::new()
            .with_max_age(Duration::from_secs(3600))
            .with_max_usage(100);

        // New key should not need rotation
        assert!(!key.needs_rotation(&policy));

        // After using it 100 times, it should need rotation
        for _ in 0..100 {
            key.increment_usage();
        }
        assert!(key.needs_rotation(&policy));

        key.mark_as_rotated();
        assert!(key.last_rotated().is_some());
        assert_eq!(key.usage_count(), 0);
        // After rotation, should not need rotation again
        assert!(!key.needs_rotation(&policy));
    }

    #[test]
    fn test_key_rotation_policy() {
        let policy = KeyRotationPolicy::new()
            .with_max_age(Duration::from_secs(86400))
            .with_max_usage(1000)
            .with_auto_rotate(true);

        assert_eq!(policy.max_age, Some(Duration::from_secs(86400)));
        assert_eq!(policy.max_usage, Some(1000));
        assert!(policy.auto_rotate);
    }

    #[test]
    fn test_key_rotation_policy_defaults() {
        let policy = KeyRotationPolicy::with_common_defaults();

        assert!(policy.max_age.is_some());
        assert!(policy.max_usage.is_some());
    }

    #[test]
    fn test_in_memory_storage() {
        let storage = InMemoryKeyStorage::new();
        let key = ProtectedKey::new(
            vec![1, 2, 3, 4],
            "test".to_string(),
            KeyType::EncryptionUnicast,
        );

        // Store
        storage.store("test", &key).unwrap();

        // Exists
        assert!(storage.exists("test").unwrap());

        // Retrieve
        let retrieved = storage.retrieve("test").unwrap();
        assert_eq!(retrieved.key(), &[1, 2, 3, 4]);

        // List
        let keys = storage.list_keys().unwrap();
        assert!(keys.contains(&"test".to_string()));

        // Delete
        storage.delete("test").unwrap();
        assert!(!storage.exists("test").unwrap());
    }

    #[test]
    fn test_key_manager() {
        let storage = InMemoryKeyStorage::new();
        let mut manager = KeyManager::new(Arc::new(storage));

        // Generate KEK
        let kek = manager.generate_kek().unwrap();
        assert_eq!(kek.key_type(), KeyType::MasterKek);
        assert!(manager.kek().is_ok());

        // Store and retrieve key
        let key_data = vec![1u8; 16];
        manager.store_key("my_key", &key_data).unwrap();
        let retrieved = manager.retrieve_key("my_key").unwrap();
        assert_eq!(retrieved, key_data);

        // Check existence
        assert!(manager.key_exists("my_key").unwrap());

        // List keys
        let keys = manager.list_keys().unwrap();
        assert!(keys.contains(&"my_key".to_string()));
    }

    #[test]
    fn test_key_rotation() {
        let storage = Arc::new(InMemoryKeyStorage::new());
        let manager = KeyManager::with_rotation_policy(
            storage.clone(),
            KeyRotationPolicy::new().with_max_usage(5),
        );

        // Store a key
        let key_data = vec![1u8; 16];
        manager.store_key("rotate_test", &key_data).unwrap();

        // Key doesn't need rotation yet (usage count is 0)
        assert!(!manager.should_rotate_key("rotate_test").unwrap());

        // Set usage manually via direct storage access
        let key = storage.retrieve("rotate_test").unwrap();
        let mut mutable_key = ProtectedKey::new(
            key.key().to_vec(),
            key.id().to_string(),
            key.key_type(),
        );
        for _ in 0..5 {
            mutable_key.increment_usage();
        }
        storage.store("rotate_test", &mutable_key).unwrap();

        // Now it needs rotation
        assert!(manager.should_rotate_key("rotate_test").unwrap());

        // Rotate
        let new_key = manager.rotate_key("rotate_test").unwrap();
        assert_eq!(new_key.len(), 16);
        assert_ne!(new_key, key_data);
    }

    #[test]
    fn test_session_keys() {
        let storage = InMemoryKeyStorage::new();
        let mut manager = KeyManager::new(Arc::new(storage));

        // Setup KEK
        manager.generate_kek().unwrap();

        // Create system title
        let system_title = SystemTitle::new([1, 2, 3, 4, 5, 6, 7, 8]);

        // Setup session keys
        let session = manager.setup_session_keys(&system_title).unwrap();

        assert_eq!(session.encryption_key.len(), 16);
        assert_eq!(session.authentication_key.len(), 16);

        // Keys are stored
        assert!(manager.key_exists("session_enc").unwrap());
        assert!(manager.key_exists("session_auth").unwrap());

        // Clear session keys
        manager.clear_session_keys().unwrap();
        assert!(!manager.key_exists("session_enc").unwrap());
        assert!(!manager.key_exists("session_auth").unwrap());
    }

    #[test]
    fn test_key_generator() {
        let key_128 = KeyGenerator::aes_128();
        assert_eq!(key_128.len(), 16);

        let key_256 = KeyGenerator::aes_256();
        assert_eq!(key_256.len(), 32);

        let custom = KeyGenerator::generate(24);
        assert_eq!(custom.len(), 24);
    }

    #[test]
    fn test_key_generator_from_password() {
        let password = b"test_password";
        let salt = [0u8; 16];
        let key = KeyGenerator::from_password(password, &salt, 10000, 32);

        assert_eq!(key.len(), 32);
        // Same input should produce same output
        let key2 = KeyGenerator::from_password(password, &salt, 10000, 32);
        assert_eq!(key, key2);
    }

    #[test]
    fn test_key_type_display() {
        assert_eq!(KeyType::MasterKek.to_string(), "MasterKEK");
        assert_eq!(KeyType::Authentication.to_string(), "Authentication");
    }
}
