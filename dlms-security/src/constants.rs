//! DLMS/COSEM security constants
//!
//! This module defines standard constants for DLMS/COSEM security, including
//! application context names and authentication mechanism names.
//!
//! These constants are based on the C++ reference implementation (COSEMSecurity.cpp)
//! and DLMS standard specifications.

/// Application Context Name for Logical Name Referencing without ciphering
///
/// OID: {2, 16, 756, 5, 8, 1, 1}
///
/// Used when:
/// - Logical Name (LN) addressing is used
/// - No encryption is required
pub const CONTEXT_LN_NO_CIPHER: &[u32] = &[2, 16, 756, 5, 8, 1, 1];

/// Application Context Name for Short Name Referencing without ciphering
///
/// OID: {2, 16, 756, 5, 8, 1, 2}
///
/// Used when:
/// - Short Name (SN) addressing is used
/// - No encryption is required
pub const CONTEXT_SN_NO_CIPHER: &[u32] = &[2, 16, 756, 5, 8, 1, 2];

/// Application Context Name for Logical Name Referencing with ciphering
///
/// OID: {2, 16, 756, 5, 8, 1, 3}
///
/// Used when:
/// - Logical Name (LN) addressing is used
/// - Encryption is required
pub const CONTEXT_LN_CIPHER: &[u32] = &[2, 16, 756, 5, 8, 1, 3];

/// Application Context Name for Short Name Referencing with ciphering
///
/// OID: {2, 16, 756, 5, 8, 1, 4}
///
/// Used when:
/// - Short Name (SN) addressing is used
/// - Encryption is required
pub const CONTEXT_SN_CIPHER: &[u32] = &[2, 16, 756, 5, 8, 1, 4];

/// Authentication Mechanism Name for Low Level Security
///
/// OID: {2, 16, 756, 5, 8, 2, 1}
///
/// Low Level Security uses password-based authentication without encryption.
/// The password is sent in plain text (or with simple encoding).
pub const MECHANISM_LOW_LEVEL: &[u32] = &[2, 16, 756, 5, 8, 2, 1];

/// Authentication Mechanism Name for High Level Security
///
/// OID: {2, 16, 756, 5, 8, 2, 5}
///
/// High Level Security uses cryptographic authentication (e.g., GMAC, HLS5-GMAC)
/// and may include encryption.
pub const MECHANISM_HIGH_LEVEL: &[u32] = &[2, 16, 756, 5, 8, 2, 5];

/// Check if an OID matches a known application context name
///
/// # Arguments
/// * `oid` - Object Identifier to check
///
/// # Returns
/// `Some(true)` if the OID matches a ciphering context, `Some(false)` if it matches
/// a non-ciphering context, or `None` if it doesn't match any known context.
pub fn is_ciphering_context(oid: &[u32]) -> Option<bool> {
    match oid {
        oid if oid == CONTEXT_LN_CIPHER || oid == CONTEXT_SN_CIPHER => Some(true),
        oid if oid == CONTEXT_LN_NO_CIPHER || oid == CONTEXT_SN_NO_CIPHER => Some(false),
        _ => None,
    }
}

/// Check if an OID matches a known authentication mechanism
///
/// # Arguments
/// * `oid` - Object Identifier to check
///
/// # Returns
/// `Some(true)` if the OID matches high-level security, `Some(false)` if it matches
/// low-level security, or `None` if it doesn't match any known mechanism.
pub fn is_high_level_security(oid: &[u32]) -> Option<bool> {
    match oid {
        oid if oid == MECHANISM_HIGH_LEVEL => Some(true),
        oid if oid == MECHANISM_LOW_LEVEL => Some(false),
        _ => None,
    }
}

/// Check if an OID uses Logical Name (LN) addressing
///
/// # Arguments
/// * `oid` - Object Identifier to check
///
/// # Returns
/// `true` if the OID matches an LN context, `false` otherwise.
pub fn is_logical_name_addressing(oid: &[u32]) -> bool {
    oid == CONTEXT_LN_NO_CIPHER || oid == CONTEXT_LN_CIPHER
}

/// Check if an OID uses Short Name (SN) addressing
///
/// # Arguments
/// * `oid` - Object Identifier to check
///
/// # Returns
/// `true` if the OID matches an SN context, `false` otherwise.
pub fn is_short_name_addressing(oid: &[u32]) -> bool {
    oid == CONTEXT_SN_NO_CIPHER || oid == CONTEXT_SN_CIPHER
}
