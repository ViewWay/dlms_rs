//! Core types and utilities for DLMS/COSEM protocol
//!
//! This crate provides fundamental types, error handling, and utilities
//! used throughout the DLMS/COSEM implementation.

pub mod error;
pub mod obis_code;
pub mod datatypes;

pub use error::{DlmsError, DlmsResult};
pub use obis_code::ObisCode;
// Re-export datatypes when implemented
// pub use datatypes::*;
