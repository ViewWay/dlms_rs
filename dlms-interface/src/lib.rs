//! COSEM interface classes module for DLMS/COSEM protocol
//!
//! This crate provides COSEM interface class definitions and implementations.
//!
//! # TODO
//!
//! ## 接口类定义
//! - [ ] Data 接口类（Class ID: 1）
//! - [ ] Register 接口类（Class ID: 3）
//! - [ ] Extended Register 接口类（Class ID: 4）
//! - [ ] Demand Register 接口类（Class ID: 5）
//! - [ ] Register Activation 接口类（Class ID: 6）
//! - [ ] Profile Generic 接口类（Class ID: 7）
//! - [ ] Clock 接口类（Class ID: 8）
//! - [ ] Script Table 接口类（Class ID: 9）
//! - [ ] Schedule 接口类（Class ID: 10）
//! - [ ] Special Days Table 接口类（Class ID: 11）
//! - [ ] Association Short Name 接口类（Class ID: 12）
//! - [ ] Association Logical Name 接口类（Class ID: 15）
//! - [ ] SAP Assignment 接口类（Class ID: 17）
//! - [ ] Image Transfer 接口类（Class ID: 18）
//! - [ ] IEC Local Port Setup 接口类（Class ID: 19）
//! - [ ] Activity Calendar 接口类（Class ID: 20）
//! - [ ] Register Monitor 接口类（Class ID: 21）
//! - [ ] Single Action Schedule 接口类（Class ID: 22）
//! - [ ] IEC HDLC Setup 接口类（Class ID: 23）
//! - [ ] IEC  twisted pair setup 接口类（Class ID: 24）
//! - [ ] MBus Slave Port Setup 接口类（Class ID: 25）
//! - [ ] Security Setup 接口类（Class ID: 64）
//! - [ ] Disconnect Control 接口类（Class ID: 70）
//! - [ ] Limiter 接口类（Class ID: 71）
//! - [ ] Push Setup 接口类（Class ID: 40）
//! - [ ] 更多接口类...
//!
//! ## 属性处理
//! - [ ] 属性访问器实现
//! - [ ] 属性值验证
//! - [ ] 属性访问权限检查
//!
//! ## 方法处理
//! - [ ] 方法调用实现
//! - [ ] 方法参数验证
//! - [ ] 方法返回值处理
//!
//! ## 宏系统
//! - [ ] 接口类定义宏
//! - [ ] 属性定义宏
//! - [ ] 方法定义宏

pub mod attribute;
pub mod method;
pub mod macros;
