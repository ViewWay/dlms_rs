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
//!
//! ## 待实现
//! - [ ] 完整的 ApplicationContextNameList 编码/解码（SEQUENCE OF）
//! - [ ] 完整的 AssociateSourceDiagnostic CHOICE 支持
//! - [ ] AcseServiceUser 和 AcseServiceProvider 枚举定义
//! - [ ] 完整的 APTitle Form 1 支持（当前仅支持 Form 2）
//! - [ ] 完整的 AEQualifier Form 1 支持（当前仅支持 Form 2）
//! - [ ] 完整的 AuthenticationValue CHOICE 支持（当前仅支持 OCTET STRING）
//! - [ ] 常用的认证机制 OID 常量定义
//! - [ ] ACSE Requirements 位定义
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
