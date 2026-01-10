//! Session layer module for DLMS/COSEM protocol
//!
//! This crate provides session layer implementations for HDLC and Wrapper protocols.
//!
//! # TODO
//!
//! ## HDLC 会话层
//! - [x] HDLC 地址编码/解码
//! - [x] HDLC 帧编码/解码
//! - [x] FCS 计算和验证
//! - [x] HDLC 连接管理
//! - [ ] HDLC 窗口管理（滑动窗口协议）
//! - [ ] HDLC 帧重传机制
//! - [ ] HDLC 连接建立和协商
//! - [ ] HDLC 参数协商（最大信息字段长度等）
//! - [ ] HDLC 帧分段和重组
//! - [ ] HDLC 错误恢复机制
//!
//! ## Wrapper 会话层
//! - [x] Wrapper 头部编码/解码
//! - [x] Wrapper PDU 编码/解码
//! - [x] Wrapper 会话管理
//! - [ ] Wrapper 连接建立流程
//! - [ ] Wrapper 错误处理
//!
//! ## 通用功能
//! - [ ] 会话层统计信息
//! - [ ] 会话状态管理
//! - [ ] 多会话支持

pub mod error;
pub mod hdlc;
pub mod wrapper;

pub use error::{DlmsError, DlmsResult};
pub use hdlc::*;
pub use wrapper::{WrapperSession, WrapperHeader, WrapperPdu, WRAPPER_HEADER_LENGTH};
