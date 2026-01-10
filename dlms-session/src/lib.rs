//! Session layer module for DLMS/COSEM protocol
//!
//! This crate provides session layer implementations for HDLC and Wrapper protocols.

pub mod error;
pub mod hdlc;
pub mod wrapper;

pub use error::{DlmsError, DlmsResult};
pub use hdlc::*;
pub use wrapper::{WrapperSession, WrapperHeader, WrapperPdu, WRAPPER_HEADER_LENGTH};
