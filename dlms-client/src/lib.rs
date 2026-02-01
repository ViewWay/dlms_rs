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
//! - [x] 对象浏览功能（ObjectBrowser）
//! - [x] 批量数据读取（BatchReader）
//! - [x] 批量数据写入（BatchWriter）
//! - [x] 高级客户端API（DlmsClient）
//! - [x] 类型安全的数据读写（TryFromDataObject/IntoDataObject）
//! - [x] 请求超时处理
//! - [x] 自动重试机制
//! - [x] 事件通知处理（EventHandler）
//!
//! ## 高级功能
//! - [x] 连接池管理
//! - [x] 自动重连机制（ReconnectManager）
//! - [x] 请求/响应超时处理
//! - [ ] 并发请求支持
//! - [ ] 请求队列管理
//! - [x] 客户端配置管理

pub mod connection;
pub mod browser;
pub mod batch_reader;
pub mod batch_writer;
pub mod block_transfer;
pub mod reconnect;
pub mod connection_pool;
pub mod event_handler;
pub mod client_api;

pub use connection::{
    Connection, ConnectionState, LnConnection, LnConnectionConfig,
    SnConnection, SnConnectionConfig, ConnectionBuilder,
};

pub use browser::{ObjectBrowser, CosemObjectDescriptor};
pub use batch_reader::{
    BatchReader, BatchReadResult, AttributeReadResult,
    AttributeReadError, AttributeReference,
};
pub use batch_writer::{
    BatchWriter, BatchWriteResult, AttributeWriteResult,
    AttributeWriteError, AttributeValue,
};
pub use block_transfer::{
    BlockTransferWriter, BlockTransferConfig, BlockTransferWritable,
};
pub use reconnect::{
    ReconnectManager, ReconnectConfig, ReconnectStrategy,
    ReconnectionState, ReconnectionStats,
};
pub use connection_pool::{
    ConnectionPool, ConnectionPoolConfig, ConnectionKey, ConnectionType,
    PoolStatistics, HealthChecker,
};
pub use event_handler::{
    EventHandler, EventNotification, EventFilter, EventCallback,
    EventListener, EventListenerConfig, EventStats,
};
pub use client_api::{
    DlmsClient, ClientConfig,
    TryFromDataObject, IntoDataObject,
};
