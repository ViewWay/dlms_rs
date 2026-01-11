//! jDLMS - Rust implementation of DLMS/COSEM protocol
//!
//! This library provides a complete implementation of the DLMS/COSEM
//! communication standard for smart meter communication.
//!
//! # Architecture
//!
//! This library is organized as a workspace with multiple crates:
//!
//! - `dlms-core`: Core types, error handling, and utilities
//! - `dlms-asn1`: ASN.1 encoding/decoding
//! - `dlms-transport`: Transport layer (TCP, UDP, Serial)
//! - `dlms-session`: Session layer (HDLC, Wrapper)
//! - `dlms-security`: Security layer (encryption, authentication)
//! - `dlms-application`: Application layer (PDU, services)
//! - `dlms-interface`: COSEM interface classes
//! - `dlms-client`: Client implementation
//! - `dlms-server`: Server implementation
//!
//! # Implementation Status
//!
//! ## ✅ 已完成
//! - 核心数据类型（DataObject, BitString, CosemDate/Time/DateTime, CompactArray）
//! - A-XDR 编码/解码
//! - 传输层（TCP, UDP, Serial）
//! - HDLC 会话层（地址、帧、FCS、连接）
//! - Wrapper 会话层
//! - 安全层（加密、认证、密钥管理）
//!
//! ## 🚧 进行中
//! - 应用层（PDU、服务）
//!   - ✅ Initiate Request/Response PDU
//!   - ✅ Get Request/Response PDU (Normal, WithList, Next, WithDataBlock) - 完整实现
//!   - ✅ Set Request/Response PDU (Normal类型)
//!   - ✅ Action Request/Response PDU (Normal类型)
//!   - ✅ Event Notification PDU
//!   - ✅ Access Request/Response PDU - 完整实现
//!   - ✅ Exception Response PDU
//!   - ✅ GET/SET/ACTION/Event Notification 服务层（完整功能）
//! - 客户端连接管理
//!   - ✅ Connection trait 定义
//!   - ✅ LnConnection 基础结构
//!   - ✅ ConnectionBuilder 实现（支持TCP和Serial）
//!   - ✅ GET/SET/ACTION 操作框架（需要完整会话层集成）
//!   - ⏳ 完整连接建立流程（传输层+会话层+应用层集成）
//!
//! ## 📋 待实现
//! - ISO-ACSE 层（✅ 基础实现完成，部分高级功能待实现）
//! - COSEM ASN.1 结构
//! - 接口类实现
//! - 服务器实现
//!
//! # Usage
//!
//! ```no_run
//! use dlms::client::ConnectionBuilder;
//! ```
//!
//! # Examples
//!
//! See the `examples/` directory for usage examples.

// Re-export core types
pub use dlms_core::{DlmsError, DlmsResult, ObisCode};
pub use dlms_core::datatypes::*;

// Re-export client API
pub mod client {
    pub use dlms_client::*;
}

// Re-export server API
pub mod server {
    pub use dlms_server::*;
}

// Re-export interface classes
pub mod interface {
    pub use dlms_interface::*;
}
