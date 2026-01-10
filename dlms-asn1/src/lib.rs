//! ASN.1 processing module for DLMS/COSEM protocol
//!
//! This crate provides ASN.1 encoding/decoding functionality including
//! A-XDR, COSEM ASN.1, and ISO-ACSE layer definitions.
//!
//! # TODO
//!
//! ## A-XDR 编码/解码
//! - [x] 基本数据类型编码/解码
//! - [x] DataObject 编码/解码
//! - [ ] CompactArray 完整编码/解码支持
//! - [ ] 长度编码优化（支持长格式）
//! - [ ] 错误处理和恢复机制
//!
//! ## COSEM ASN.1
//! - [ ] 生成 COSEM ASN.1 结构定义
//! - [ ] 实现 COSEM 对象标识符编码/解码
//! - [ ] 实现 COSEM 方法调用编码/解码
//! - [ ] 实现 COSEM 属性访问编码/解码
//!
//! ## ISO-ACSE
//! - [x] 基础类型定义（AssociateResult, ReleaseRequestReason, etc.）
//! - [x] 辅助类型（APTitle, AEQualifier, AssociationInformation, etc.）
//! - [x] AARQ (Association Request) 编码/解码
//! - [x] AARE (Association Response) 编码/解码
//! - [x] RLRQ (Release Request) 编码/解码
//! - [x] RLRE (Release Response) 编码/解码
//! - [ ] ApplicationContextNameList 完整实现（SEQUENCE OF）
//! - [ ] AssociateSourceDiagnostic 完整 CHOICE 支持
//! - [ ] AcseServiceUser 和 AcseServiceProvider 枚举
//! - [ ] APTitle Form 1 支持
//! - [ ] AEQualifier Form 1 支持
//! - [ ] AuthenticationValue 完整 CHOICE 支持
//! - [ ] 常用认证机制 OID 常量
//! - [ ] ACSE Requirements 位定义
//! - [ ] ACSE 错误处理增强

pub mod error;
pub mod axdr;
pub mod ber;
pub mod cosem;
pub mod iso_acse;

pub use error::{DlmsError, DlmsResult};
pub use axdr::{AxdrEncoder, AxdrDecoder};
pub use axdr::types::{AxdrTag, LengthEncoding};
pub use ber::{BerEncoder, BerDecoder, BerTag, BerTagClass, BerLength};
pub use iso_acse::{AARQApdu, AAREApdu, RLRQApdu, RLREApdu, DLMS_APPLICATION_CONTEXT_NAME, DLMS_APPLICATION_CONTEXT_NAME_CIPHERED};