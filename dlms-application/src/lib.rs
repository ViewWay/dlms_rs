//! Application layer module for DLMS/COSEM protocol
//!
//! This crate provides application layer functionality including PDU handling and services.
//!
//! # TODO
//!
//! ## PDU (Protocol Data Unit)
//! - [x] Initiate Request PDU 编码/解码
//! - [x] Initiate Response PDU 编码/解码
//! - [x] Get Request PDU 编码/解码（Normal、Next和WithList已完整实现）
//! - [x] Get Response PDU 编码/解码（Normal、WithDataBlock和WithList已完整实现）
//! - [x] Get Request Next PDU 完整实现（已验证和完善）
//! - [x] Get Request WithList PDU 完整实现
//! - [x] Get Response WithDataBlock PDU 完整实现（已验证和完善）
//! - [x] Get Response WithList PDU 完整实现
//! - [x] Set Request PDU 编码/解码（Normal类型已实现）
//! - [x] Set Response PDU 编码/解码（Normal类型已实现）
//! - [x] Action Request PDU 编码/解码（Normal类型已实现）
//! - [x] Action Response PDU 编码/解码（Normal类型已实现）
//! - [x] Event Notification PDU 编码/解码
//! - [x] Access Request PDU 编码/解码（完整实现）
//! - [x] Access Response PDU 编码/解码（完整实现）
//! - [x] Exception Response PDU 编码/解码
//!
//! ## 服务层
//! - [x] GET 服务实现（基础功能已实现，支持WithList和WithDataBlock）
//! - [x] SET 服务实现（基础功能已实现）
//! - [x] ACTION 服务实现（基础功能已实现）
//! - [x] Event Notification 服务实现
//! - [x] 服务错误处理增强（使用标准错误码常量和描述）
//! - [x] 服务响应处理增强（支持WithList和WithDataBlock）
//!
//! ## 寻址
//! - [x] 逻辑名称（LN）寻址（LogicalNameReference）
//! - [x] 短名称（SN）寻址（ShortNameReference）
//! - [x] 类 ID 和属性/方法 ID 处理
//! - [x] OBIS 代码到对象引用转换
//! - [x] 访问选择器处理（AccessSelector，基础实现）
//! - [ ] 完整的访问选择器支持（日期范围等复杂选择器）

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

pub use pdu::{
    InitiateRequest, InitiateResponse, Conformance, DLMS_VERSION_6, MAX_PDU_SIZE,
    GetRequest, GetResponse, GetRequestNormal, GetResponseNormal,
    SetRequest, SetResponse, SetRequestNormal, SetResponseNormal, SetDataResult,
    ActionRequest, ActionResponse, ActionRequestNormal, ActionResponseNormal, ActionResult,
    EventNotification, AccessRequest, AccessResponse, AccessRequestSpecification, AccessResponseSpecification,
    ExceptionResponse,
    InvokeIdAndPriority, CosemAttributeDescriptor, CosemMethodDescriptor,
    SelectiveAccessDescriptor, GetDataResult,
};

// Re-export error code constants for convenience
pub use pdu::data_access_result;
pub use pdu::action_result;
