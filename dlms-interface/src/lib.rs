//! COSEM interface classes module for DLMS/COSEM protocol
//!
//! This crate provides COSEM interface class definitions and implementations.
//!
//! # TODO
//!
//! ## 接口类定义
//! - [x] Data 接口类（Class ID: 1） - 已实现
//! - [x] Register 接口类（Class ID: 3） - 已实现（包含ScalerUnit支持）
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

use dlms_core::{DlmsResult, ObisCode, DataObject};
use dlms_application::pdu::SelectiveAccessDescriptor;
use async_trait::async_trait;

pub mod attribute;
pub mod method;
pub mod macros;
pub mod data;
pub mod scaler_unit;
pub mod register;

pub use data::Data;
pub use scaler_unit::{ScalerUnit, units};
pub use register::Register;

/// COSEM Object trait
///
/// This trait defines the interface that all COSEM objects must implement.
/// It provides methods for getting and setting attributes, and invoking methods.
///
/// # Why This Trait?
/// Using a trait allows:
/// - **Polymorphism**: Same code works with different object types
/// - **Extensibility**: Easy to add new object types
/// - **Testability**: Easy to mock objects for testing
#[async_trait]
pub trait CosemObject: Send + Sync {
    /// Get the class ID of this object
    fn class_id(&self) -> u16;

    /// Get the OBIS code (logical name) of this object
    fn obis_code(&self) -> ObisCode;

    /// Get an attribute value
    ///
    /// # Arguments
    /// * `attribute_id` - Attribute ID to read (1-255)
    /// * `selective_access` - Optional selective access descriptor
    ///
    /// # Returns
    /// The attribute value as a `DataObject`, or error if attribute doesn't exist
    async fn get_attribute(
        &self,
        attribute_id: u8,
        selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<DataObject>;

    /// Set an attribute value
    ///
    /// # Arguments
    /// * `attribute_id` - Attribute ID to write (1-255)
    /// * `value` - Value to write
    /// * `selective_access` - Optional selective access descriptor
    ///
    /// # Returns
    /// `Ok(())` if successful, error otherwise
    async fn set_attribute(
        &self,
        attribute_id: u8,
        value: DataObject,
        selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<()>;

    /// Invoke a method
    ///
    /// # Arguments
    /// * `method_id` - Method ID to invoke (1-255)
    /// * `parameters` - Method parameters (optional)
    /// * `selective_access` - Optional selective access descriptor
    ///
    /// # Returns
    /// The method return value as a `DataObject`, or error if method doesn't exist or fails
    async fn invoke_method(
        &self,
        method_id: u8,
        parameters: Option<DataObject>,
        selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<Option<DataObject>>;
}
