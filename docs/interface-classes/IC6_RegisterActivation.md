# Register Activation 接口类完整实现规范

**Class ID: 6 | Version: 0 | 优先级: 🟡 中**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Register Activation 控制寄存器的激活状态，用于管理费率切换。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | register_assignment | array | static | x + 0x08 | 关联的寄存器列表 |
| 3 | mask_list | array | static | x + 0x10 | 激活掩码列表 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: register_assignment

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x08`

关联的寄存器列表

#### 属性 3: mask_list

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x10`

激活掩码列表

---

## 3. 方法定义

| 方法 ID | 名称 | 必选/可选 | Short Name | 参数类型 | 说明 |
|---------|------|----------|------------|----------|------|
| 1 | add_register | 必选 | x + 0x60 | object_id | 添加寄存器到关联列表 |
| 2 | remove_register | 必选 | x + 0x68 | object_id | 从关联列表移除寄存器 |


### 3.1 方法详细说明

#### 方法 1: add_register

- **必选/可选**: 必选
- **Short Name**: `x + 0x60`
- **参数类型**: `object_id`

添加寄存器到关联列表

#### 方法 2: remove_register

- **必选/可选**: 必选
- **Short Name**: `x + 0x68`
- **参数类型**: `object_id`

从关联列表移除寄存器

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Register Activation 接口类 (Class ID: 6, Version: 0)
/// 
/// Register Activation 控制寄存器的激活状态，用于管理费率切换。
#[derive(Debug, Clone)]
pub struct RegisterActivation {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 关联的寄存器列表
    pub register_assignment: Vec<DlmsType>,
    /// 激活掩码列表
    pub mask_list: Vec<DlmsType>,
}

impl RegisterActivation {
    /// 创建新的 Register Activation 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            register_assignment: Vec::new(),
            mask_list: Vec::new(),
        }
    }

    /// 方法 1: add_register
    /// 
    /// 添加寄存器到关联列表
    pub fn add_register(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 add_register
        // 参数类型: object_id
        Ok(DlmsType::Null)
    }

    /// 方法 2: remove_register
    /// 
    /// 从关联列表移除寄存器
    pub fn remove_register(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 remove_register
        // 参数类型: object_id
        Ok(DlmsType::Null)
    }
}

impl CosemObject for RegisterActivation {
    const CLASS_ID: u16 = 6;
    const VERSION: u8 = 0;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.register_assignment.clone().into()),
            3 => Ok(self.mask_list.clone().into()),
            _ => Err(CosemError::InvalidAttribute(attr_id)),
        }
    }

    fn set_attribute(&mut self, attr_id: u8, value: DlmsType) -> Result<(), CosemError> {
        match attr_id {
            _ => Err(CosemError::ReadOnlyAttribute(attr_id)),
        }
    }

    fn invoke_method(&mut self, method_id: u8, params: DlmsType) -> Result<DlmsType, CosemError> {
        match method_id {
            1 => self.add_register(params),
            2 => self.remove_register(params),
            _ => Err(CosemError::InvalidMethod(method_id)),
        }
    }
}
```

---

## 5. 测试用例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register__activation_class_id() {
        assert_eq!(RegisterActivation::CLASS_ID, 6);
    }

    #[test]
    fn test_register__activation_version() {
        assert_eq!(RegisterActivation::VERSION, 0);
    }

    #[test]
    fn test_register__activation_new() {
        let obis = ObisCode::from_str("0.0.6.0.0.255").unwrap();
        let obj = RegisterActivation::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_register__activation_get_logical_name() {
        let obis = ObisCode::from_str("0.0.6.0.0.255").unwrap();
        let obj = RegisterActivation::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_register__activation_get_register_assignment() {
        let obj = RegisterActivation::new(ObisCode::from_str("0.0.6.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_register__activation_get_mask_list() {
        let obj = RegisterActivation::new(ObisCode::from_str("0.0.6.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_register__activation_add_register() {
        let mut obj = RegisterActivation::new(ObisCode::from_str("0.0.6.0.0.255").unwrap());
        let result = obj.invoke_method(1, DlmsType::Null);
        assert!(result.is_ok());
    }


    #[test]
    fn test_register__activation_remove_register() {
        let mut obj = RegisterActivation::new(ObisCode::from_str("0.0.6.0.0.255").unwrap());
        let result = obj.invoke_method(2, DlmsType::Null);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `RegisterActivation` 结构体
- [ ] 实现所有 3 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 6`
- [ ] `VERSION = 0`
- [ ] `get_attribute()` - 实现 3 个属性
- [ ] `set_attribute()` - 实现可写属性

### 6.3 方法实现 (2 个)

- [ ] 方法 1: `add_register()`
- [ ] 方法 2: `remove_register()`

### 6.4 测试

- [ ] 单元测试: 属性读写
- [ ] 单元测试: 方法调用
- [ ] 集成测试: 与 dlms_rs 集成

---

## 7. 相关文档

- Blue Book Edition 16 Part 2
- Green Book Edition 9
- [DLMS UA 1000-1](https://www.dlms.com)

---

*文件名: IC6_RegisterActivation.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
