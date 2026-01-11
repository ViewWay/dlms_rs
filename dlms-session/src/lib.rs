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
//! - [x] HCS (Header Check Sequence) 计算和验证
//! - [x] HDLC 连接管理
//! - [x] HDLC 窗口管理（滑动窗口协议）
//! - [x] HDLC 帧重传机制
//! - [x] HDLC 连接建立和协商（SNRM/UA握手）
//! - [x] HDLC 连接释放（DISC/DM/UA握手）
//! - [x] HDLC 参数协商（最大信息字段长度等）
//! - [x] HDLC 帧分段和重组（自动RR帧发送）
//! - [x] LLC Header支持
//! - [x] HDLC统计信息收集
//! - [x] 状态机管理
//! - [ ] HDLC 错误恢复机制（高级功能）
//!
//! ## Wrapper 会话层
//! - [x] Wrapper 头部编码/解码
//! - [x] Wrapper PDU 编码/解码
//! - [x] Wrapper 会话管理
//! - [ ] Wrapper 连接建立流程
//! - [ ] Wrapper 错误处理
//!
//! ## 通用功能
//! - [x] 会话层统计信息（HDLC统计）
//! - [x] 会话状态管理（HDLC状态机）
//! - [ ] 多会话支持（高级功能）

pub mod error;
pub mod hdlc;
pub mod wrapper;

pub use error::{DlmsError, DlmsResult};
pub use hdlc::*;
pub use wrapper::{WrapperSession, WrapperHeader, WrapperPdu, WRAPPER_HEADER_LENGTH};
