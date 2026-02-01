//! ISO-ACSE ASN.1 definitions
//!
//! This module provides ISO-ACSE (Association Control Service Element) structures
//! for DLMS/COSEM protocol. ISO-ACSE is used to establish and release associations
//! between application entities.
//!
//! # Main PDU Types
//!
//! - **AARQ**: Association Request (Application tag 0)
//! - **AARE**: Association Response (Application tag 1)
//! - **RLRQ**: Release Request (Application tag 2)
//! - **RLRE**: Release Response (Application tag 3)
//!
//! # Encoding Format
//!
//! All ISO-ACSE PDUs are encoded using BER (Basic Encoding Rules) as specified
//! in ITU-T X.690. This is different from A-XDR used in the COSEM application layer.
//!
//! # Usage
//!
//! ```rust,no_run
//! use dlms_asn1::iso_acse::{AARQApdu, AAREApdu};
//!
//! // Create AARQ
//! let aarq = AARQApdu::new(vec![1, 0, 17, 0, 0, 128, 0, 1]); // DLMS context
//! let encoded = aarq.encode()?;
//!
//! // Decode AARE
//! let aare = AAREApdu::decode(&data)?;
//! ```
//!
//! # TODO
//!
//! ## 已实现
//! - [x] AARQ (Association Request) 结构定义和编码/解码
//! - [x] AARE (Association Response) 结构定义和编码/解码
//! - [x] RLRQ (Release Request) 结构定义和编码/解码
//! - [x] RLRE (Release Response) 结构定义和编码/解码
//! - [x] 基础类型（AssociateResult, ReleaseRequestReason, ReleaseResponseReason）
//! - [x] 基础辅助类型（APTitle, AEQualifier, APInvocationIdentifier, etc.）
//! - [x] APTitle Form 1 和 Form 2 完整支持（OID 和 OCTET STRING）
//! - [x] AEQualifier Form 1 和 Form 2 完整支持（INTEGER 和 OCTET STRING）
//! - [x] ApplicationContextNameList 完整编码/解码（SEQUENCE OF）
//! - [x] AssociateSourceDiagnostic 完整 CHOICE 支持
//! - [x] AcseServiceUser 和 AcseServiceProvider 枚举定义
//! - [x] AuthenticationValue 完整 CHOICE 支持（所有数据类型变体）
//! - [x] 认证机制 OID 常量定义（Low-level, HLS5-GMAC, SHA-256等）
//! - [x] ACSE Requirements 位定义（builder模式）
//!
//! ## 待实现
//! - [ ] 单元测试覆盖所有字段组合
//! - [ ] 错误处理和验证增强

pub mod types;
pub mod pdu;

pub use types::*;
pub use pdu::{AARQApdu, AAREApdu, RLRQApdu, RLREApdu};

/// DLMS Application Context Name OID
///
/// This is the standard OID for DLMS/COSEM application context:
/// {1 0 17 0 0 128 0 1}
///
/// # Why This Constant?
/// This OID is used in all DLMS/COSEM associations. Providing it as a constant
/// prevents errors from manual OID construction and improves code readability.
pub const DLMS_APPLICATION_CONTEXT_NAME: &[u32] = &[1, 0, 17, 0, 0, 128, 0, 1];

/// DLMS Application Context Name OID (Ciphered)
///
/// This is the OID for DLMS/COSEM application context with encryption:
/// {1 0 17 0 0 128 0 2}
pub const DLMS_APPLICATION_CONTEXT_NAME_CIPHERED: &[u32] = &[1, 0, 17, 0, 0, 128, 0, 2];
