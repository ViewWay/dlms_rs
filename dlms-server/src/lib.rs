//! DLMS/COSEM server implementation
//!
//! This crate provides server-side functionality for DLMS/COSEM protocol.
//!
//! # TODO
//!
//! ## 服务器基础
//! - [x] 服务器配置管理
//! - [x] COSEM 对象注册表
//! - [x] 对象实例管理
//! - [x] 关联管理（Association）
//! - [x] 服务器启动和停止（基础实现）
//! - [x] 连接监听和接受（基础实现）
//! - [ ] 多客户端连接管理（部分实现，需要完善）
//! - [ ] 服务器状态管理
//!
//! ## 请求处理
//! - [x] GET 请求处理（基础实现）
//! - [x] SET 请求处理（基础实现）
//! - [x] ACTION 请求处理（基础实现）
//! - [x] Initiate Request 处理
//! - [ ] 请求验证和授权
//! - [ ] 请求路由和分发（高级功能）
//! - [ ] 响应生成（完整实现）
//!
//! ## 对象管理
//! - [x] COSEM 对象注册表
//! - [x] 对象实例管理
//! - [ ] 对象访问控制
//! - [x] 对象属性管理（通过CosemObject trait）
//! - [x] 对象方法实现（通过CosemObject trait）
//!
//! ## 事件处理
//! - [ ] 事件通知生成
//! - [ ] 事件订阅管理
//! - [ ] 事件推送机制
//!
//! ## 高级功能
//! - [ ] 服务器统计信息
//! - [ ] 日志和监控
//! - [ ] 性能优化
//! - [ ] 并发请求处理
//! - [ ] Get Request Next/WithList 完整支持
//! - [ ] Short Name 寻址支持

pub mod server;
pub mod listener;

pub use server::{DlmsServer, ServerConfig, CosemObject, AssociationContext};
pub use listener::ServerListener;
