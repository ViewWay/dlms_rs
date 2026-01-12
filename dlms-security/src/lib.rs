//! Security module for DLMS/COSEM protocol
//!
//! This crate provides security functionality including encryption and authentication.
//!
//! # TODO
//!
//! ## 加密功能
//! - [x] AES-GCM 加密/解密
//! - [x] Security Control 字节处理
//! - [x] 密钥派生函数（KDF）- 基础实现
//! - [x] 系统标题（System Title）管理
//! - [x] 帧计数器（Frame Counter）管理
//! - [x] 加密帧构建和解析（已集成System Title和Frame Counter）
//!
//! ## 认证功能
//! - [x] GMAC 认证
//! - [x] Low-level 认证（密码）
//! - [x] HLS5-GMAC 认证
//! - [ ] 认证挑战-响应流程
//! - [ ] 密钥协商机制
//! - [ ] 认证状态管理
//!
//! ## 密钥管理
//! - [x] AES 密钥生成
//! - [x] RFC 3394 密钥包装/解包
//! - [x] 密钥派生函数（KDF）- 基础实现
//! - [x] System Title管理
//! - [x] Frame Counter管理
//! - [x] xDLMS上下文管理
//! - [ ] 密钥存储和管理
//! - [ ] 密钥更新机制
//! - [ ] 主密钥（KEK）管理
//! - [ ] 密钥导出和导入
//!
//! ## 安全套件
//! - [x] Security Suite 配置
//! - [x] Security Policy 管理
//! - [ ] 安全套件协商
//! - [ ] 安全参数验证

pub mod error;
pub mod suite;
pub mod encryption;
pub mod authentication;
pub mod utils;
pub mod constants;
pub mod xdlms;
pub mod xdlms_frame;

pub use error::{DlmsError, DlmsResult};
pub use suite::{
    SecuritySuite, SecuritySuiteBuilder, SecurityPolicy, EncryptionMechanism, AuthenticationMechanism,
};
pub use encryption::{AesGcmEncryption, SecurityControl};
pub use authentication::{GmacAuth, LowAuth, Hls5GmacAuth};
pub use utils::{KeyId, generate_aes128_key, wrap_aes_rfc3394_key, unwrap_aes_rfc3394_key};
pub use constants::*;
pub use xdlms::{SystemTitle, FrameCounter, KeyDerivationFunction, XdlmsContext};
pub use xdlms_frame::{EncryptedFrameBuilder, EncryptedFrameParser};