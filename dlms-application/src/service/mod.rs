//! Service layer for DLMS/COSEM application layer
//!
//! This module provides high-level service interfaces for DLMS/COSEM operations:
//! - **GET Service**: Read attribute values
//! - **SET Service**: Write attribute values
//! - **ACTION Service**: Invoke methods on objects
//! - **Event Notification Service**: Handle asynchronous event notifications
//!
//! # Architecture
//!
//! The service layer sits above the PDU layer and provides:
//! - High-level APIs for common operations
//! - Automatic PDU encoding/decoding
//! - Error handling and result conversion
//! - Request/response correlation (invoke ID management)
//!
//! # Usage
//!
//! ```rust,no_run
//! use dlms_application::service::GetService;
//! use dlms_application::pdu::{GetRequest, CosemAttributeDescriptor};
//!
//! // Create a GET service
//! let service = GetService::new();
//!
//! // Create a request
//! let attr_desc = CosemAttributeDescriptor::new_logical_name(1, obis_code, 2)?;
//! let request = service.create_request(attr_desc, None)?;
//!
//! // Send request and receive response
//! let response = service.send_request(request)?;
//! ```

pub mod get;
pub mod set;
pub mod action;
pub mod event;

pub use get::GetService;
pub use set::SetService;
pub use action::ActionService;
pub use event::EventNotificationService;