# Auto Connect 接口类完整实现规范

**Class ID: 29 | Version: 2 | 优先级: 🟢 低**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Auto Connect 配置自动连接功能。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | mode | enum | static | x + 0x08 | 连接模式 |
| 3 | repetitions | unsigned | static | x + 0x10 | 重试次数 |
| 4 | repetition_delay | long_unsigned | static | x + 0x18 | 重试间隔 |
| 5 | calling_window | structure | static | x + 0x20 | 呼叫时间窗口 |
| 6 | allowed_destinations | array | static | x + 0x28 | 允许的目的地列表 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: mode

- **类型**: `enum`
- **访问**: static
- **Short Name**: `x + 0x08`

连接模式

#### 属性 3: repetitions

- **类型**: `unsigned`
- **访问**: static
- **Short Name**: `x + 0x10`

重试次数

#### 属性 4: repetition_delay

- **类型**: `long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x18`

重试间隔

#### 属性 5: calling_window

- **类型**: `structure`
- **访问**: static
- **Short Name**: `x + 0x20`

呼叫时间窗口

#### 属性 6: allowed_destinations

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x28`

允许的目的地列表

---

## 3. 方法定义

| 方法 ID | 名称 | 必选/可选 | Short Name | 参数类型 | 说明 |
|---------|------|----------|------------|----------|------|
| 1 | set_mode | 必选 | x + 0x60 | enum | 设置连接模式 |


### 3.1 方法详细说明

#### 方法 1: set_mode

- **必选/可选**: 必选
- **Short Name**: `x + 0x60`
- **参数类型**: `enum`

设置连接模式

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Auto Connect 接口类 (Class ID: 29, Version: 2)
/// 
/// Auto Connect 配置自动连接功能。
#[derive(Debug, Clone)]
pub struct AutoConnect {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 连接模式
    pub mode: u8,
    /// 重试次数
    pub repetitions: u8,
    /// 重试间隔
    pub repetition_delay: u16,
    /// 呼叫时间窗口
    pub calling_window: Vec<DlmsType>,
    /// 允许的目的地列表
    pub allowed_destinations: Vec<DlmsType>,
}

impl AutoConnect {
    /// 创建新的 Auto Connect 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            mode: 0,
            repetitions: 0,
            repetition_delay: 0,
            calling_window: Vec::new(),
            allowed_destinations: Vec::new(),
        }
    }

    /// 方法 1: set_mode
    /// 
    /// 设置连接模式
    pub fn set_mode(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 set_mode
        // 参数类型: enum
        Ok(DlmsType::Null)
    }
}

impl CosemObject for AutoConnect {
    const CLASS_ID: u16 = 29;
    const VERSION: u8 = 2;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.mode.clone().into()),
            3 => Ok(self.repetitions.clone().into()),
            4 => Ok(self.repetition_delay.clone().into()),
            5 => Ok(self.calling_window.clone().into()),
            6 => Ok(self.allowed_destinations.clone().into()),
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
            1 => self.set_mode(params),
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
    fn test_auto__connect_class_id() {
        assert_eq!(AutoConnect::CLASS_ID, 29);
    }

    #[test]
    fn test_auto__connect_version() {
        assert_eq!(AutoConnect::VERSION, 2);
    }

    #[test]
    fn test_auto__connect_new() {
        let obis = ObisCode::from_str("0.0.29.0.0.255").unwrap();
        let obj = AutoConnect::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_auto__connect_get_logical_name() {
        let obis = ObisCode::from_str("0.0.29.0.0.255").unwrap();
        let obj = AutoConnect::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_auto__connect_get_mode() {
        let obj = AutoConnect::new(ObisCode::from_str("0.0.29.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_auto__connect_get_repetitions() {
        let obj = AutoConnect::new(ObisCode::from_str("0.0.29.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_auto__connect_get_repetition_delay() {
        let obj = AutoConnect::new(ObisCode::from_str("0.0.29.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }


    #[test]
    fn test_auto__connect_set_mode() {
        let mut obj = AutoConnect::new(ObisCode::from_str("0.0.29.0.0.255").unwrap());
        let result = obj.invoke_method(1, DlmsType::Null);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `AutoConnect` 结构体
- [ ] 实现所有 6 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 29`
- [ ] `VERSION = 2`
- [ ] `get_attribute()` - 实现 6 个属性
- [ ] `set_attribute()` - 实现可写属性

### 6.3 方法实现 (1 个)

- [ ] 方法 1: `set_mode()`

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

*文件名: IC29_AutoConnect.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
