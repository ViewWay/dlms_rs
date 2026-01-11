//! DLMS/COSEM client implementation
//!
//! This crate provides client-side functionality for connecting to and
//! communicating with DLMS/COSEM devices.
//!
//! # TODO
//!
//! ## 连接管理
//! - [x] 连接构建器（Builder）模式实现
//! - [x] TCP 连接构建器（基础实现）
//! - [x] Serial 连接构建器（基础实现）
//! - [x] 逻辑名称（LN）连接实现（完整实现，支持HDLC和Wrapper）
//! - [x] 短名称（SN）连接实现（完整实现，支持HDLC和Wrapper）
//! - [x] 连接建立流程（完整实现，包括传输层、会话层、应用层）
//! - [x] 连接关闭和清理（完整实现）
//! - [x] 连接状态管理
//!
//! ## 客户端功能
//! - [x] GET 操作实现（基础框架完成，需要完整会话层支持）
//! - [x] SET 操作实现（基础框架完成，需要完整会话层支持）
//! - [x] ACTION 操作实现（基础框架完成，需要完整会话层支持）
//! - [ ] 对象浏览功能
//! - [ ] 数据读取功能
//! - [ ] 数据写入功能
//! - [ ] 方法调用功能
//! - [ ] 事件通知处理
//!
//! ## 高级功能
//! - [ ] 连接池管理
//! - [ ] 自动重连机制
//! - [ ] 请求/响应超时处理
//! - [ ] 并发请求支持
//! - [ ] 请求队列管理
//! - [ ] 客户端配置管理

pub mod connection;

pub use connection::{
    Connection, ConnectionState, LnConnection, LnConnectionConfig,
    SnConnection, SnConnectionConfig, ConnectionBuilder,
};
