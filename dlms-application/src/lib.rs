//! DLMS/COSEM Application Layer
//!
//! This crate provides the DLMS/COSEM application layer protocol implementation,
//! including all PDU (Protocol Data Unit) types, service handlers, and addressing modes.
//!
//! # Overview
//!
//! The DLMS application layer defines the protocol data units (PDUs) exchanged
//! between client and server for reading/writing meter data, invoking methods,
//! and handling events.
//!
//! ## PDU Types
//!
//! The application layer supports several types of requests and responses:
//!
//! - **Initiate**: Connection establishment and capability negotiation
//! - **Get**: Read attribute values from COSEM objects
//! - **Set**: Write attribute values to COSEM objects
//! - **Action**: Invoke methods on COSEM objects
//! - **Access**: Combined GET/SET/ACTION operations in a single request
//! - **Event**: Notification of events from meter to client
//! - **Exception**: Error reporting
//!
//! # Get Request Example
//!
//! ```rust,ignore
//! use dlms_application::{GetRequest, GetRequestNormal, InvokeIdAndPriority};
//! use dlms_application::{CosemAttributeDescriptor, LogicalNameReference};
//! use dlms_core::ObisCode;
//!
//! // Create a normal GET request for attribute 2 (value) of a register
//! let invoke_id = InvokeIdAndPriority::new(1, false)?;
//! let descriptor = CosemAttributeDescriptor::LogicalName(LogicalNameReference {
//!     class_id: 3,
//!     instance_id: ObisCode::new(1, 1, 1, 8, 0, 255),
//!     id: 2,  // value attribute
//! });
//!
//! let request = GetRequest::Normal(GetRequestNormal::new(
//!     invoke_id,
//!     descriptor,
//!     None,  // No selective access
//! ));
//! ```
//!
//! # Set Request Example
//!
//! ```rust,ignore
//! use dlms_application::{SetRequest, SetRequestNormal, SetDataResult, InvokeIdAndPriority};
//! use dlms_application::{CosemAttributeDescriptor, LogicalNameReference};
//! use dlms_core::{ObisCode, DataObject};
//!
//! // Create a SET request to write a new value
//! let invoke_id = InvokeIdAndPriority::new(2, false)?;
//! let descriptor = CosemAttributeDescriptor::LogicalName(LogicalNameReference {
//!     class_id: 3,
//!     instance_id: ObisCode::new(1, 1, 1, 8, 0, 255),
//!     id: 2,
//! });
//!
//! let value = DataObject::new_unsigned32(12345);
//! let request = SetRequest::Normal(SetRequestNormal::new(
//!     invoke_id,
//!     descriptor,
//!     None,  // No selective access
//!     value,
//! ));
//! ```
//!
//! # Addressing Modes
//!
//! DLMS supports two addressing modes:
//!
//! - **Logical Name (LN)**: Uses OBIS code (A-E groups + F) for object identification
//! - **Short Name (SN)**: Uses 16-bit base name for compact addressing
//!
//! ```rust,ignore
//! use dlms_application::{CosemAttributeDescriptor, LogicalNameReference, ShortNameReference};
//! use dlms_core::ObisCode;
//!
//! // LN addressing
//! let ln_descriptor = CosemAttributeDescriptor::LogicalName(LogicalNameReference {
//!     class_id: 3,
//!     instance_id: ObisCode::new(1, 1, 1, 8, 0, 255),
//!     id: 2,
//! });
//!
//! // SN addressing
//! let sn_descriptor = CosemAttributeDescriptor::ShortName {
//!     base_name: 0x1000,
//!     id: 2,
//! };
//! ```
//!
//! # Selective Access
//!
//! Selective access allows reading/writing portions of array or structured data:
//!
//! ```rust,ignore
//! use dlms_application::{SelectiveAccessDescriptor, SelectiveAccess, AccessSelector};
//! use dlms_core::DataObject;
//!
//! // Read specific array indices
//! let selector = AccessSelector::Index(3);
//! let selective = SelectiveAccess::new(0, selector);
//!
//! let descriptor = SelectiveAccessDescriptor {
//!     access_selector: Some(selective),
//! };
//! ```
//!
//! # Conformance
//!
//! The Conformance bits indicate which DLMS features a device supports:
//!
//! ```rust,ignore
//! use dlms_application::{Conformance, ConformanceBit};
//!
//! let conformance = Conformance::new()
//!     .with(ConformanceBit::Get)
//!     .with(ConformanceBit::Set)
//!     .with(ConformanceBit::Action)
//!     .with(ConformanceBit::BlockTransfer);
//! ```
//!
//! # Module Structure
//!
//! - [`pdu`] - Protocol Data Unit definitions
//! - [`service`] - Service implementations (GET, SET, ACTION, Event)
//! - [`addressing`] - Addressing modes and selectors
//! - [`protocol_identification`] - Protocol identification for wrappers
//! - [`association`] - Association-related PDUs
//! - [`encrypted`] - Encrypted PDU handling
//! - [`sn_pdu`] - Short Name PDU format
//!
//! # Implementation Status
//!
//! ## PDU Encoding/Decoding
//! - [x] Initiate Request/Response - Connection establishment
//! - [x] Get Request (Normal, Next, WithList)
//! - [x] Get Response (Normal, WithDataBlock, WithList)
//! - [x] Set Request (Normal, WithFirstDataBlock, WithDataBlock, WithList)
//! - [x] Set Response (Normal, WithDataBlock, WithList)
//! - [x] Action Request/Response (Normal)
//! - [x] Access Request/Response - Combined operations
//! - [x] Event Notification - Push notifications
//! - [x] Exception Response - Error handling
//!
//! ## Services
//! - [x] GET service - Attribute reading
//! - [x] SET service - Attribute writing
//! - [x] ACTION service - Method invocation
//! - [x] Event Notification service - Event reporting
//!
//! ## Addressing
//! - [x] Logical Name (LN) addressing
//! - [x] Short Name (SN) addressing
//! - [x] Selective access (EntryIndex, DateRange, ValueRange, ParameterizedAccess)
//! - [x] Access selector descriptors

pub mod pdu;
pub mod service {
    pub mod get;
    pub mod set;
    pub mod action;
    pub mod event;

    pub use get::GetService;
    pub use set::SetService;
    pub use action::ActionService;
    pub use event::EventNotificationService;
}
pub mod addressing;
pub mod protocol_identification;
pub mod association;
pub mod encrypted;
pub mod sn_pdu;

pub use pdu::{
    InitiateRequest, InitiateResponse, Conformance, ConformanceEncodingMode,
    DLMS_VERSION_6, MAX_PDU_SIZE,
    GetRequest, GetResponse, GetRequestNormal, GetResponseNormal,
    SetRequest, SetResponse, SetRequestNormal, SetResponseNormal, SetDataResult,
    ActionRequest, ActionResponse, ActionRequestNormal, ActionResponseNormal, ActionResult,
    EventNotification, DataNotification, VariableNameSpecification,
    AccessRequest, AccessResponse, AccessRequestSpecification, AccessResponseSpecification,
    ExceptionResponse, ConfirmedServiceError, ServiceError,
    InvokeIdAndPriority, CosemAttributeDescriptor, CosemMethodDescriptor,
    SelectiveAccessDescriptor, GetDataResult,
};

// Re-export addressing types
pub use addressing::{LogicalNameReference, ShortNameReference, AccessSelector};

// Re-export error code constants
pub use pdu::data_access_result;
pub use pdu::action_result;

// Re-export protocol identification
pub use protocol_identification::{ProtocolIdentification, ProtocolInfo};

// Re-export encrypted PDU types
pub use encrypted::{
    SecurityControl, KeyType, EncryptedPduType,
    GlobalEncryptedPdu, DedicatedEncryptedPdu, EncryptedPdu,
};

// Re-export SN PDU types
pub use sn_pdu::{
    SnPduTag, ShortName,
    ReadRequest, ReadResponse,
    WriteRequest, WriteResponse,
    UnconfirmedWriteRequest, InformationReportRequest,
    SnPdu,
};
