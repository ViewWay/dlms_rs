//! DLMS/COSEM server implementation
//!
//! This crate provides server-side functionality for DLMS/COSEM protocol.
//!
//! # Features
//!
//! - **Connection Management**: Multi-client connection handling with timeout management
//! - **Access Control**: ACL-based authorization for clients
//! - **Event Processing**: Event notification generation and subscription management
//! - **Request Statistics**: Comprehensive statistics tracking for monitoring and debugging
//! - **Block Transfer**: Support for large value transfers (GET and SET)
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use dlms_server::{DlmsServer, ServerConfig};
//! use dlms_interface::{Register, CosemObject};
//! use dlms_core::ObisCode;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create server configuration
//!     let config = ServerConfig {
//!         server_sap: 1,
//!         max_connections: 100,
//!         connection_idle_timeout_secs: 300,
//!         ..Default::default()
//!     };
//!
//!     let server = DlmsServer::with_config(config);
//!
//!     // Register a register object
//!     let register = Register::new(ObisCode::new(1, 1, 1, 8, 0, 255));
//!     server.register_object(Arc::new(register)).await?;
//!
//!     // Get server statistics
//!     let stats = server.get_connection_statistics().await;
//!     println!("Active connections: {}", stats.active_connections);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Request Statistics
//!
//! The server provides comprehensive request statistics tracking:
//!
//! ```rust,no_run
//! use dlms_server::{DlmsServer, request_stats::RequestType};
//!
//! // Start tracking a request
//! let tracker = server.record_request_start(RequestType::Get);
//!
//! // Process the request...
//!
//! // Record completion
//! server.record_request_completion(tracker, true, 100, 50);
//!
//! // Get formatted statistics
//! let summary = server.get_request_statistics().await.summary();
//! println!("{}", summary.format());
//! ```
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
//! - [x] 多客户端连接管理
//! - [x] 服务器状态管理
//!
//! ## 请求处理
//! - [x] GET 请求处理（基础实现）
//! - [x] SET 请求处理（基础实现）
//! - [x] ACTION 请求处理（基础实现）
//! - [x] Initiate Request 处理
//! - [x] 请求验证和授权
//! - [ ] 请求路由和分发（高级功能）
//! - [ ] 响应生成（完整实现）
//!
//! ## 对象管理
//! - [x] COSEM 对象注册表
//! - [x] 对象实例管理
//! - [x] 对象访问控制
//! - [x] 对象属性管理（通过CosemObject trait）
//! - [x] 对象方法实现（通过CosemObject trait）
//!
//! ## 事件处理
//! - [x] 事件通知生成
//! - [x] 事件订阅管理
//! - [x] 事件推送机制
//!
//! ## 高级功能
//! - [x] 服务器统计信息
//! - [ ] 日志和监控
//! - [ ] 性能优化
//! - [ ] 并发请求处理
//! - [x] Get Request Next/WithList 完整支持
//! - [x] Short Name 寻址支持

pub mod server;
pub mod server_state;
pub mod listener;
pub mod connection_manager;
pub mod access_control;
pub mod event;
pub mod set_block_transfer;
pub mod request_stats;
pub mod error;

pub use server::{DlmsServer, ServerConfig, AssociationContext};
pub use server_state::{ServerStateMachine, ServerState, ServerStatus, StateTransition};
pub use error::{DlmsServerError, ServerErrorCode};
pub use listener::ServerListener;
pub use connection_manager::{
    ConnectionManager, ConnectionInfo, ConnectionStatistics,
};
pub use access_control::{
    AccessControlManager, AccessControlList, AccessRule, AccessPermission, AclKey,
};
pub use event::{
    EventProcessor, DlmsEvent, EventSeverity, EventFilter,
    EventSubscription, EventNotification,
};
pub use request_stats::{
    RequestType, ServerRequestStats, ServerStatsSummary,
    RequestTracker, RequestTypeStats, PerformanceMetrics, ErrorStats,
};
pub use dlms_interface::CosemObject;
