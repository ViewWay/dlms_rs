//! Application layer module for DLMS/COSEM protocol
//!
//! This crate provides application layer functionality including PDU handling and services.
//!
//! # TODO
//!
//! ## PDU (Protocol Data Unit)
//! - [x] Initiate Request PDU 编码/解码
//! - [x] Initiate Response PDU 编码/解码
//! - [ ] Get Request PDU 编码/解码
//! - [ ] Get Response PDU 编码/解码
//! - [ ] Set Request PDU 编码/解码
//! - [ ] Set Response PDU 编码/解码
//! - [ ] Action Request PDU 编码/解码
//! - [ ] Action Response PDU 编码/解码
//! - [ ] Event Notification PDU 编码/解码
//! - [ ] Access Request PDU 编码/解码
//! - [ ] Access Response PDU 编码/解码
//! - [ ] Exception Response PDU 编码/解码
//!
//! ## 服务层
//! - [ ] GET 服务实现
//! - [ ] SET 服务实现
//! - [ ] ACTION 服务实现
//! - [ ] Event Notification 服务实现
//! - [ ] 服务错误处理
//! - [ ] 服务响应处理
//!
//! ## 寻址
//! - [ ] 逻辑名称（LN）寻址
//! - [ ] 短名称（SN）寻址
//! - [ ] 类 ID 和属性/方法 ID 处理
//! - [ ] OBIS 代码到对象引用转换
//! - [ ] 访问选择器处理

pub mod pdu;
pub mod service;
pub mod addressing;

pub use pdu::{InitiateRequest, InitiateResponse, Conformance, DLMS_VERSION_6, MAX_PDU_SIZE};
