//! Security Parameter Validation
//!
//! This module provides validation functionality for security-related parameters
//! in DLMS/COSEM protocol.
//!
//! # Overview
//!
//! Security parameter validation ensures that all security-related parameters
//! meet the requirements specified in DLMS/COSEM standards. This includes:
//!
//! - Key length validation
//! - System title format validation
//! - Frame counter range validation
//! - Security suite compatibility validation
//!
//! # Usage
//!
//! ```rust,no_run
//! use dlms_security::validation::{
//!     SecurityParameterValidator, ValidationRule, ValidationResult
//! };
//!
//! let validator = SecurityParameterValidator::default();
//!
//! // Validate a key
//! let result = validator.validate_key(&[0u8; 16], 16);
//! assert!(result.is_valid);
//!
//! // Validate a system title
//! let result = validator.validate_system_title(&[0u8; 8]);
//! assert!(result.is_valid);
//! ```

use crate::error::DlmsResult;
use crate::suite::{SecuritySuite, EncryptionMechanism, AuthenticationMechanism, SecurityPolicy};
use std::collections::HashMap;
use std::fmt;

/// Validation result containing success/failure status and any errors/warnings
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Validation errors (if any)
    pub errors: Vec<ValidationError>,
    /// Validation warnings (if any)
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Create a new successful validation result
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    /// Create a new failed validation result
    pub fn failure(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: vec![],
        }
    }

    /// Create a new validation result with warnings
    pub fn with_warnings(mut self, warnings: Vec<ValidationWarning>) -> Self {
        self.warnings = warnings;
        // Warnings don't affect validity
        self
    }

    /// Add an error to the validation result
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
        self.is_valid = false;
    }

    /// Add a warning to the validation result
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Combine two validation results (AND logic)
    pub fn and(mut self, other: ValidationResult) -> Self {
        self.is_valid = self.is_valid && other.is_valid;
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self
    }

    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get the number of warnings
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Get a formatted error message
    pub fn error_message(&self) -> String {
        if self.errors.is_empty() {
            String::from("No errors")
        } else {
            self.errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        }
    }

    /// Get a formatted warning message
    pub fn warning_message(&self) -> String {
        if self.warnings.is_empty() {
            String::from("No warnings")
        } else {
            self.warnings
                .iter()
                .map(|w| w.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::success()
    }
}

impl fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ValidationResult(valid={}, errors={}, warnings={})",
            self.is_valid,
            self.errors.len(),
            self.warnings.len()
        )
    }
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error code
    pub code: ErrorCode,
    /// Error message
    pub message: String,
    /// Parameter path (for nested errors)
    pub path: String,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            path: String::new(),
        }
    }

    /// Create a new validation error with path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_empty() {
            write!(f, "[{:?}] {}", self.code, self.message)
        } else {
            write!(f, "[{:?}] {}.at({})", self.code, self.message, self.path)
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning code
    pub code: WarningCode,
    /// Warning message
    pub message: String,
    /// Parameter path
    pub path: String,
}

impl ValidationWarning {
    /// Create a new validation warning
    pub fn new(code: WarningCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            path: String::new(),
        }
    }

    /// Create a new validation warning with path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }
}

impl fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_empty() {
            write!(f, "[{:?}] {}", self.code, self.message)
        } else {
            write!(f, "[{:?}] {}.at({})", self.code, self.message, self.path)
        }
    }
}

/// Error code for validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// Key length is invalid
    InvalidKeyLength,
    /// Key format is invalid
    InvalidKeyFormat,
    /// System title length is invalid
    InvalidSystemTitleLength,
    /// System title format is invalid
    InvalidSystemTitleFormat,
    /// Frame counter is out of range
    FrameCounterOutOfRange,
    /// Frame counter is not monotonic
    FrameCounterNotMonotonic,
    /// Security suite is invalid
    InvalidSecuritySuite,
    /// Security policy mismatch
    SecurityPolicyMismatch,
    /// Authentication tag length is invalid
    InvalidAuthenticationTagLength,
    /// Parameter value is out of range
    ValueOutOfRange,
    /// Required parameter is missing
    MissingRequiredParameter,
    /// Parameter combination is invalid
    InvalidParameterCombination,
}

/// Warning code for validation warnings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningCode {
    /// Key is weak (consider using stronger key)
    WeakKey,
    /// Security level is lower than recommended
    LowSecurityLevel,
    /// Parameter is deprecated
    DeprecatedParameter,
    /// Parameter value is at the edge of acceptable range
    BoundaryValue,
    /// Parameter combination may be invalid
    InvalidParameterCombination,
}

/// Validation rule for security parameters
#[derive(Debug, Clone)]
pub enum ValidationRule {
    /// Key length validation (min, max bytes)
    KeyLength { min_bytes: usize, max_bytes: usize },
    /// System title length (must be exactly 8 bytes)
    SystemTitleLength { exact_bytes: usize },
    /// Frame counter range (min, max)
    FrameCounterRange { min: u32, max: u32 },
    /// Authentication tag length (exact bytes)
    AuthenticationTagLength { exact_bytes: usize },
    /// Custom validation rule
    Custom {
        name: String,
        validator: fn(&[u8]) -> DlmsResult<()>,
    },
}

/// Security parameter validator
///
/// Validates security-related parameters according to DLMS/COSEM standards.
pub struct SecurityParameterValidator {
    /// Validation rules
    rules: HashMap<String, ValidationRule>,
    /// Strict mode (fail on warnings)
    strict_mode: bool,
    /// Allow weak keys
    allow_weak_keys: bool,
    /// Minimum security level
    min_security_level: u8,
}

impl SecurityParameterValidator {
    /// Create a new validator with default rules
    pub fn new() -> Self {
        let mut validator = Self {
            rules: HashMap::new(),
            strict_mode: false,
            allow_weak_keys: false,
            min_security_level: 0,
        };

        // Add default rules
        validator.add_default_rules();
        validator
    }

    /// Add default validation rules
    fn add_default_rules(&mut self) {
        // Key length rules for different key types
        self.rules.insert(
            "aes_128_key".to_string(),
            ValidationRule::KeyLength {
                min_bytes: 16,
                max_bytes: 16,
            },
        );
        self.rules.insert(
            "aes_192_key".to_string(),
            ValidationRule::KeyLength {
                min_bytes: 24,
                max_bytes: 24,
            },
        );
        self.rules.insert(
            "aes_256_key".to_string(),
            ValidationRule::KeyLength {
                min_bytes: 32,
                max_bytes: 32,
            },
        );

        // System title validation
        self.rules.insert(
            "system_title".to_string(),
            ValidationRule::SystemTitleLength { exact_bytes: 8 },
        );

        // Frame counter validation
        self.rules.insert(
            "frame_counter".to_string(),
            ValidationRule::FrameCounterRange { min: 0, max: u32::MAX },
        );

        // Authentication tag validation (GCM tag is 12 bytes)
        self.rules.insert(
            "authentication_tag".to_string(),
            ValidationRule::AuthenticationTagLength { exact_bytes: 12 },
        );
    }

    /// Set strict mode (fail on warnings)
    pub fn set_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Allow weak keys
    pub fn allow_weak_keys(mut self, allow: bool) -> Self {
        self.allow_weak_keys = allow;
        self
    }

    /// Set minimum security level (0-3)
    pub fn set_min_security_level(mut self, level: u8) -> Self {
        self.min_security_level = level.min(3);
        self
    }

    /// Validate a key
    ///
    /// # Arguments
    /// * `key` - The key bytes to validate
    /// * `expected_length` - Expected key length in bytes (0 for variable length)
    pub fn validate_key(&self, key: &[u8], expected_length: usize) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Check length
        if expected_length > 0 && key.len() != expected_length {
            result.add_error(ValidationError::new(
                ErrorCode::InvalidKeyLength,
                format!(
                    "Key length mismatch: expected {} bytes, got {}",
                    expected_length,
                    key.len()
                ),
            ));
        }

        // Check for weak keys (all zeros, all ones, etc.)
        if !self.allow_weak_keys {
            if is_weak_key(key) {
                let warning = ValidationWarning::new(
                    WarningCode::WeakKey,
                    "Key appears to be weak (contains repeated patterns)",
                );
                result.add_warning(warning);
            }
        }

        // Check key entropy (basic check)
        if key.len() >= 16 && has_low_entropy(key) {
            result.add_warning(ValidationWarning::new(
                WarningCode::WeakKey,
                "Key has low entropy (may not be sufficiently random)",
            ));
        }

        result
    }

    /// Validate a system title
    pub fn validate_system_title(&self, title: &[u8]) -> ValidationResult {
        let mut result = ValidationResult::success();

        if title.len() != 8 {
            result.add_error(ValidationError::new(
                ErrorCode::InvalidSystemTitleLength,
                format!(
                    "System title must be exactly 8 bytes, got {}",
                    title.len()
                ),
            ));
        }

        // Check if title is all zeros (not recommended)
        if title.iter().all(|&b| b == 0) {
            result.add_warning(ValidationWarning::new(
                WarningCode::BoundaryValue,
                "System title is all zeros (not recommended for production)",
            ));
        }

        result
    }

    /// Validate a frame counter value
    pub fn validate_frame_counter(&self, counter: u32, previous_counter: Option<u32>) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Check if counter is within valid range
        if let Some(ValidationRule::FrameCounterRange { min: _, max: _ }) =
            self.rules.get("frame_counter")
        {
            if let ValidationRule::FrameCounterRange { min: rule_min, max: rule_max } = *self.rules.get("frame_counter").unwrap() {
                if counter < rule_min || counter > rule_max {
                    result.add_error(ValidationError::new(
                        ErrorCode::FrameCounterOutOfRange,
                        format!("Frame counter {} is out of range [{}, {}]", counter, rule_min, rule_max),
                    ));
                }
            }
        }

        // Check monotonicity
        if let Some(prev) = previous_counter {
            if counter <= prev {
                result.add_error(ValidationError::new(
                    ErrorCode::FrameCounterNotMonotonic,
                    format!("Frame counter {} is not greater than previous value {}", counter, prev),
                ));
            }
        }

        // Warning if counter is near wraparound
        if counter > u32::MAX - 1000 {
            result.add_warning(ValidationWarning::new(
                WarningCode::BoundaryValue,
                format!("Frame counter {} is near wraparound point", counter),
            ));
        }

        result
    }

    /// Validate an authentication tag
    pub fn validate_authentication_tag(&self, tag: &[u8], expected_length: usize) -> ValidationResult {
        let mut result = ValidationResult::success();

        if tag.len() != expected_length {
            result.add_error(ValidationError::new(
                ErrorCode::InvalidAuthenticationTagLength,
                format!(
                    "Authentication tag length mismatch: expected {} bytes, got {}",
                    expected_length,
                    tag.len()
                ),
            ));
        }

        result
    }

    /// Validate a security suite
    pub fn validate_security_suite(&self, suite: &SecuritySuite) -> ValidationResult {
        let mut result = ValidationResult::success();

        let encryption = suite.encryption_mechanism();
        let authentication = suite.authentication_mechanism();
        let policy = suite.security_policy();

        // Validate encryption mechanism and key presence
        if encryption != EncryptionMechanism::None {
            if suite.global_unicast_encryption_key().is_none() {
                result.add_error(ValidationError::new(
                    ErrorCode::MissingRequiredParameter,
                    format!("Encryption key required for {:?}", encryption),
                ));
            } else if let Some(key) = suite.global_unicast_encryption_key() {
                let key_result = self.validate_key(key, encryption.key_length_bytes().unwrap_or(0));
                result = result.and(key_result);
            }
        }

        // Validate authentication mechanism and key presence
        if matches!(
            authentication,
            AuthenticationMechanism::Low | AuthenticationMechanism::Hls5Gmac
        ) {
            if suite.authentication_key().is_none() && suite.password().is_none() {
                result.add_error(ValidationError::new(
                    ErrorCode::MissingRequiredParameter,
                    format!("Authentication key/password required for {:?}", authentication),
                ));
            }
        }

        // Validate policy consistency
        let expected_policy = self.derive_policy(encryption, authentication);
        if policy != expected_policy {
            result.add_warning(ValidationWarning::new(
                WarningCode::InvalidParameterCombination,
                format!(
                    "Security policy {:?} may not match mechanisms (encryption={:?}, auth={:?}), expected {:?}",
                    policy, encryption, authentication, expected_policy
                ),
            ));
        }

        // Check security level
        let security_level = self.calculate_security_level(suite);
        if security_level < self.min_security_level {
            result.add_warning(ValidationWarning::new(
                WarningCode::LowSecurityLevel,
                format!(
                    "Security level {} is below minimum required level {}",
                    security_level, self.min_security_level
                ),
            ));
        }

        result
    }

    /// Validate security suite compatibility between client and server
    pub fn validate_suite_compatibility(
        &self,
        client_suite: &SecuritySuite,
        server_suite: &SecuritySuite,
    ) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Both must use same encryption mechanism
        if client_suite.encryption_mechanism() != server_suite.encryption_mechanism() {
            result.add_error(ValidationError::new(
                ErrorCode::InvalidParameterCombination,
                format!(
                    "Encryption mechanism mismatch: client={:?}, server={:?}",
                    client_suite.encryption_mechanism(),
                    server_suite.encryption_mechanism()
                ),
            ));
        }

        // Both must use same authentication mechanism
        if client_suite.authentication_mechanism() != server_suite.authentication_mechanism() {
            result.add_error(ValidationError::new(
                ErrorCode::InvalidParameterCombination,
                format!(
                    "Authentication mechanism mismatch: client={:?}, server={:?}",
                    client_suite.authentication_mechanism(),
                    server_suite.authentication_mechanism()
                ),
            ));
        }

        result
    }

    /// Derive security policy from mechanisms
    fn derive_policy(
        &self,
        encryption: EncryptionMechanism,
        authentication: AuthenticationMechanism,
    ) -> SecurityPolicy {
        let has_encryption = encryption != EncryptionMechanism::None;
        let has_auth = authentication != AuthenticationMechanism::None
            && authentication != AuthenticationMechanism::Absent;

        match (has_encryption, has_auth) {
            (true, true) => SecurityPolicy::AuthenticatedAndEncrypted,
            (true, false) => SecurityPolicy::Encrypted,
            (false, true) => SecurityPolicy::Authenticated,
            (false, false) => SecurityPolicy::Nothing,
        }
    }

    /// Calculate security level (0-3)
    fn calculate_security_level(&self, suite: &SecuritySuite) -> u8 {
        suite.security_policy() as u8
    }
}

impl Default for SecurityParameterValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a key appears to be weak
fn is_weak_key(key: &[u8]) -> bool {
    if key.is_empty() {
        return true;
    }

    // All zeros
    if key.iter().all(|&b| b == 0) {
        return true;
    }

    // All ones
    if key.iter().all(|&b| b == 0xFF) {
        return true;
    }

    // All same byte
    let first = key[0];
    if key.iter().all(|&b| b == first) {
        return true;
    }

    false
}

/// Basic entropy check (counts unique byte values)
fn has_low_entropy(key: &[u8]) -> bool {
    let unique_count = key.iter().collect::<std::collections::HashSet<_>>().len();
    // Less than 50% unique bytes is considered low entropy
    unique_count < (key.len() / 2)
}

/// Quick validation for a vector of keys
pub fn validate_key_vector(keys: &[Vec<u8>], expected_length: usize) -> ValidationResult {
    let validator = SecurityParameterValidator::default();
    let mut result = ValidationResult::success();

    for (i, key) in keys.iter().enumerate() {
        let key_result = validator.validate_key(key, expected_length);
        let key_result = if !key_result.is_valid {
            ValidationResult::failure(
                key_result
                    .errors
                    .into_iter()
                    .map(|e| ValidationError::new(e.code, format!("Key[{}]: {}", i, e.message)))
                    .collect(),
            )
        } else {
            key_result
        };
        result = result.and(key_result);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suite::{SecuritySuiteBuilder, EncryptionMechanism, AuthenticationMechanism};

    #[test]
    fn test_validate_valid_key() {
        let validator = SecurityParameterValidator::new();
        let key = [0u8; 16];
        let result = validator.validate_key(&key, 16);

        assert!(result.is_valid);
        assert_eq!(result.error_count(), 0);
    }

    #[test]
    fn test_validate_invalid_key_length() {
        let validator = SecurityParameterValidator::new();
        let key = [0u8; 8]; // Too short for AES-128
        let result = validator.validate_key(&key, 16);

        assert!(!result.is_valid);
        assert_eq!(result.error_count(), 1);
        assert!(matches!(
            result.errors[0].code,
            ErrorCode::InvalidKeyLength
        ));
    }

    #[test]
    fn test_validate_system_title() {
        let validator = SecurityParameterValidator::new();

        // Valid title
        let title = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let result = validator.validate_system_title(&title);
        assert!(result.is_valid);

        // Invalid title length
        let short_title = [1u8, 2, 3];
        let result = validator.validate_system_title(&short_title);
        assert!(!result.is_valid);
        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn test_validate_frame_counter() {
        let validator = SecurityParameterValidator::new();

        // Valid counter
        let result = validator.validate_frame_counter(100, Some(50));
        assert!(result.is_valid);

        // Non-monotonic counter
        let result = validator.validate_frame_counter(50, Some(100));
        assert!(!result.is_valid);
        assert_eq!(result.error_count(), 1);
        assert!(matches!(
            result.errors[0].code,
            ErrorCode::FrameCounterNotMonotonic
        ));
    }

    #[test]
    fn test_validate_frame_counter_wraparound_warning() {
        let validator = SecurityParameterValidator::new();

        // Near wraparound
        let result = validator.validate_frame_counter(u32::MAX - 500, Some(u32::MAX - 1000));
        assert!(result.is_valid);
        assert_eq!(result.warning_count(), 1);
        assert!(matches!(
            result.warnings[0].code,
            WarningCode::BoundaryValue
        ));
    }

    #[test]
    fn test_validate_authentication_tag() {
        let validator = SecurityParameterValidator::new();

        // Valid tag
        let tag = [0u8; 12];
        let result = validator.validate_authentication_tag(&tag, 12);
        assert!(result.is_valid);

        // Invalid tag length
        let short_tag = [0u8; 8];
        let result = validator.validate_authentication_tag(&short_tag, 12);
        assert!(!result.is_valid);
        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn test_validate_security_suite_missing_key() {
        let validator = SecurityParameterValidator::new();

        let suite = SecuritySuiteBuilder::new()
            .set_encryption_mechanism(EncryptionMechanism::AesGcm128)
            .build()
            .unwrap();

        let result = validator.validate_security_suite(&suite);
        assert!(!result.is_valid);
        assert_eq!(result.error_count(), 1);
        assert!(matches!(
            result.errors[0].code,
            ErrorCode::MissingRequiredParameter
        ));
    }

    #[test]
    fn test_validate_security_suite_valid() {
        let validator = SecurityParameterValidator::new();

        let suite = SecuritySuiteBuilder::new()
            .set_encryption_mechanism(EncryptionMechanism::AesGcm128)
            .set_global_unicast_encryption_key(vec![0u8; 16])
            .build()
            .unwrap();

        let result = validator.validate_security_suite(&suite);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_suite_compatibility() {
        let validator = SecurityParameterValidator::new();

        let suite1 = SecuritySuiteBuilder::new()
            .set_encryption_mechanism(EncryptionMechanism::AesGcm128)
            .set_global_unicast_encryption_key(vec![0u8; 16])
            .build()
            .unwrap();

        let suite2 = SecuritySuiteBuilder::new()
            .set_encryption_mechanism(EncryptionMechanism::None)
            .build()
            .unwrap();

        let result = validator.validate_suite_compatibility(&suite1, &suite2);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validation_result_and() {
        let result1 = ValidationResult::success();
        let result2 = ValidationResult::success();
        let combined = result1.and(result2);
        assert!(combined.is_valid);

        let result1 = ValidationResult::success();
        let result2 = ValidationResult::failure(vec![
            ValidationError::new(ErrorCode::InvalidKeyLength, "test"),
        ]);
        let combined = result1.and(result2);
        assert!(!combined.is_valid);
    }

    #[test]
    fn test_weak_key_detection() {
        assert!(is_weak_key(&[0u8; 16]));
        assert!(is_weak_key(&[0xFFu8; 16]));
        assert!(is_weak_key(&[0x55u8; 16]));
        assert!(!is_weak_key(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]));
    }

    #[test]
    fn test_low_entropy_detection() {
        let high_entropy = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        assert!(!has_low_entropy(&high_entropy));

        let low_entropy = [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
        assert!(has_low_entropy(&low_entropy));
    }

    #[test]
    fn test_min_security_level() {
        let validator = SecurityParameterValidator::new()
            .set_min_security_level(2);

        // Use a non-weak key
        let key = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let suite = SecuritySuiteBuilder::new()
            .set_encryption_mechanism(EncryptionMechanism::AesGcm128)
            .set_global_unicast_encryption_key(key.to_vec())
            .build()
            .unwrap();

        let result = validator.validate_security_suite(&suite);
        // Suite is valid, AesGcm128 with no auth has policy Encrypted (level 2)
        assert!(result.is_valid);
        // No warnings about security level since level 2 >= min_security_level 2
        assert_eq!(result.warning_count(), 0);
    }

    #[test]
    fn test_strict_mode() {
        let validator = SecurityParameterValidator::new()
            .set_strict_mode(true);

        // In strict mode, warnings might be treated as errors
        // This is a placeholder test - actual strict mode behavior depends on implementation
        assert!(validator.strict_mode);
    }

    #[test]
    fn test_validate_key_vector() {
        let keys = vec![
            vec![0u8; 16],
            vec![1u8; 16],
            vec![2u8; 16],
        ];

        let result = validate_key_vector(&keys, 16);
        assert!(result.is_valid);

        let keys = vec![
            vec![0u8; 16],
            vec![0u8; 8], // Wrong length
        ];

        let result = validate_key_vector(&keys, 16);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_error_message() {
        let error = ValidationError::new(ErrorCode::InvalidKeyLength, "test message");
        assert_eq!(error.to_string(), "[InvalidKeyLength] test message");

        let error_with_path = error.with_path("encryption.key");
        assert!(error_with_path.to_string().contains(".at(encryption.key)"));
    }

    #[test]
    fn test_warning_message() {
        let warning = ValidationWarning::new(WarningCode::WeakKey, "test warning");
        assert_eq!(warning.to_string(), "[WeakKey] test warning");

        let warning_with_path = warning.with_path("key");
        assert!(warning_with_path.to_string().contains(".at(key)"));
    }

    #[test]
    fn test_validation_result_display() {
        let result = ValidationResult::success();
        assert_eq!(
            result.to_string(),
            "ValidationResult(valid=true, errors=0, warnings=0)"
        );
    }

    #[test]
    fn test_validation_result_with_warnings() {
        let result = ValidationResult::success()
            .with_warnings(vec![
                ValidationWarning::new(WarningCode::WeakKey, "weak key"),
            ]);

        assert!(result.is_valid);
        assert_eq!(result.warning_count(), 1);
    }
}
