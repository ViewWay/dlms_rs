//! Transport layer module for DLMS/COSEM protocol
//!
//! This crate provides transport layer implementations for TCP, UDP, and Serial communication.
//!
//! # TODO
//!
//! ## TCP 传输
//! - [x] TCP 连接管理
//! - [x] 异步读写操作
//! - [ ] 连接池管理
//! - [ ] 自动重连机制
//! - [ ] 超时处理优化
//!
//! ## UDP 传输
//! - [x] UDP 套接字管理
//! - [x] 异步读写操作
//! - [ ] 数据包分片和重组
//! - [ ] 数据包丢失检测
//!
//! ## Serial 传输
//! - [x] 串口连接管理
//! - [x] 异步读写操作
//! - [ ] 串口参数自动检测
//! - [ ] 流控制支持
//! - [ ] 多串口设备管理
//!
//! ## 通用功能
//! - [ ] 传输层统计信息
//! - [ ] 连接状态监控
//! - [ ] 错误恢复机制

pub mod error;
pub mod stream;
pub mod tcp;
pub mod udp;
pub mod serial;

pub use error::{DlmsError, DlmsResult};
pub use stream::{StreamAccessor, TransportLayer};
pub use tcp::{TcpTransport, TcpSettings};
pub use udp::{UdpTransport, UdpSettings, MAX_UDP_PAYLOAD_SIZE};
pub use serial::{SerialTransport, SerialSettings};
