# Auto Answer 接口类完整实现规范

**Class ID: 28 | Version: 2 | 优先级: 🟢 低**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Auto Answer 配置自动应答功能。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | mode | enum | static | x + 0x08 | 应答模式 |
| 3 | listening_window | structure | static | x + 0x10 | 监听时间窗口 |
| 4 | number_of_calls | unsigned | dynamic | x + 0x18 | 呼叫计数 |
| 5 | number_of_rings | unsigned | static | x + 0x20 | 振铃次数 |
| 6 | answered | boolean | dynamic | x + 0x28 | 是否已应答 |


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

应答模式

#### 属性 3: listening_window

- **类型**: `structure`
- **访问**: static
- **Short Name**: `x + 0x10`

监听时间窗口

#### 属性 4: number_of_calls

- **类型**: `unsigned`
- **访问**: dynamic
- **Short Name**: `x + 0x18`

呼叫计数

#### 属性 5: number_of_rings

- **类型**: `unsigned`
- **访问**: static
- **Short Name**: `x + 0x20`

振铃次数

#### 属性 6: answered

- **类型**: `boolean`
- **访问**: dynamic
- **Short Name**: `x + 0x28`

是否已应答

---

## 3. 方法定义

| 方法 ID | 名称 | 必选/可选 | Short Name | 参数类型 | 说明 |
|---------|------|----------|------------|----------|------|
| 1 | set_mode | 必选 | x + 0x60 | enum | 设置应答模式 |


### 3.1 方法详细说明

#### 方法 1: set_mode

- **必选/可选**: 必选
- **Short Name**: `x + 0x60`
- **参数类型**: `enum`

设置应答模式

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Auto Answer 接口类 (Class ID: 28, Version: 2)
/// 
/// Auto Answer 配置自动应答功能。
#[derive(Debug, Clone)]
pub struct AutoAnswer {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 应答模式
    pub mode: u8,
    /// 监听时间窗口
    pub listening_window: Vec<DlmsType>,
    /// 呼叫计数
    pub number_of_calls: u8,
    /// 振铃次数
    pub number_of_rings: u8,
    /// 是否已应答
    pub answered: bool,
}

impl AutoAnswer {
    /// 创建新的 Auto Answer 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            mode: 0,
            listening_window: Vec::new(),
            number_of_calls: 0,
            number_of_rings: 0,
            answered: false,
        }
    }

    /// 方法 1: set_mode
    /// 
    /// 设置应答模式
    pub fn set_mode(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 set_mode
        // 参数类型: enum
        Ok(DlmsType::Null)
    }
}

impl CosemObject for AutoAnswer {
    const CLASS_ID: u16 = 28;
    const VERSION: u8 = 2;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.mode.clone().into()),
            3 => Ok(self.listening_window.clone().into()),
            4 => Ok(self.number_of_calls.clone().into()),
            5 => Ok(self.number_of_rings.clone().into()),
            6 => Ok(self.answered.clone().into()),
            _ => Err(CosemError::InvalidAttribute(attr_id)),
        }
    }

    fn set_attribute(&mut self, attr_id: u8, value: DlmsType) -> Result<(), CosemError> {
        match attr_id {
            4 => {
                self.number_of_calls = value.try_into()?;
                Ok(())
            }
            6 => {
                self.answered = value.try_into()?;
                Ok(())
            }
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
    fn test_auto__answer_class_id() {
        assert_eq!(AutoAnswer::CLASS_ID, 28);
    }

    #[test]
    fn test_auto__answer_version() {
        assert_eq!(AutoAnswer::VERSION, 2);
    }

    #[test]
    fn test_auto__answer_new() {
        let obis = ObisCode::from_str("0.0.28.0.0.255").unwrap();
        let obj = AutoAnswer::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_auto__answer_get_logical_name() {
        let obis = ObisCode::from_str("0.0.28.0.0.255").unwrap();
        let obj = AutoAnswer::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_auto__answer_get_mode() {
        let obj = AutoAnswer::new(ObisCode::from_str("0.0.28.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_auto__answer_get_listening_window() {
        let obj = AutoAnswer::new(ObisCode::from_str("0.0.28.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_auto__answer_get_number_of_calls() {
        let obj = AutoAnswer::new(ObisCode::from_str("0.0.28.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }


    #[test]
    fn test_auto__answer_set_mode() {
        let mut obj = AutoAnswer::new(ObisCode::from_str("0.0.28.0.0.255").unwrap());
        let result = obj.invoke_method(1, DlmsType::Null);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `AutoAnswer` 结构体
- [ ] 实现所有 6 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 28`
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

*文件名: IC28_AutoAnswer.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
