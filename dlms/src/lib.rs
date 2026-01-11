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
//! - HDLC 会话层（地址、帧、FCS、HCS、连接、窗口管理、帧重传、SNRM/UA握手、DISC/DM/UA释放、分段重组、LLC Header、统计信息、状态机）
//! - Wrapper 会话层
//! - 安全层（加密、认证、密钥管理、xDLMS基础功能）
//! - xDLMS（System Title、Frame Counter、KDF、xDLMS Context）
//!
//! ## ✅ 已完成（继续完善中）
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
//!   - ✅ LnConnection 完整实现（支持HDLC和Wrapper）
//!   - ✅ SnConnection 完整实现（支持HDLC和Wrapper）
//!   - ✅ ConnectionBuilder 实现（支持TCP和Serial，LN和SN）
//!   - ✅ GET/SET/ACTION 操作完整实现
//!   - ✅ 完整连接建立流程（传输层+会话层+应用层集成）
//!   - ✅ 完整连接关闭流程
//! - 服务器实现
//!   - ✅ 服务器基础框架（DlmsServer、对象管理、关联管理）
//!   - ✅ 服务器监听器（TCP连接监听、客户端接受、并发处理）
//!   - ✅ GET/SET/ACTION请求处理（基础实现）
//!   - ✅ Initiate Request处理
//!
//! ## 📋 待实现（详细列表见 TODO.md）
//! 
//! ### 高优先级
//! - [ ] 服务器端SNRM/UA握手实现
//! - [ ] 请求解析和路由
//! - [ ] 加密帧构建和解析
//! - [ ] 帧计数器验证
//! - [ ] 完整的访问选择器支持
//! 
//! ### 中优先级
//! - [ ] ISO-ACSE高级功能（ApplicationContextNameList、完整CHOICE支持等）
//! - [ ] 服务器高级功能（访问控制、事件处理、统计信息等）
//! - [ ] 客户端高级功能（对象浏览、连接池、自动重连等）
//! - [ ] 安全层高级功能（认证挑战-响应、密钥协商、密钥管理等）
//! 
//! ### 低优先级
//! - [ ] COSEM ASN.1 结构
//! - [ ] 接口类实现（Data、Register、Profile Generic、Clock等）
//! - [ ] 传输层优化（连接池、自动重连、统计信息等）
//! - [ ] 性能优化和代码质量提升
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
