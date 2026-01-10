//! Core types and utilities for DLMS/COSEM protocol
//!
//! This crate provides fundamental types, error handling, and utilities
//! used throughout the DLMS/COSEM implementation.
//!
//! # TODO
//!
//! - [ ] 完善数据类型单元测试
//! - [ ] 实现更多 COSEM 日期/时间格式支持
//! - [ ] 添加数据类型验证和约束检查
//! - [ ] 实现 OBIS 代码解析和验证工具
//! - [ ] 添加数据类型转换工具函数

pub mod error;
pub mod obis_code;
pub mod datatypes;

pub use error::{DlmsError, DlmsResult};
pub use obis_code::ObisCode;
pub use datatypes::*;
