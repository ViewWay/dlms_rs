# Modem Configuration 接口类完整实现规范

**Class ID: 27 | Version: 1 | 优先级: 🟢 低**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Modem Configuration 配置调制解调器参数。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | modem_type | enum | static | x + 0x08 | 调制解调器类型 |
| 3 | initialized | boolean | dynamic | x + 0x10 | 是否已初始化 |
| 4 | modem_initialization_strings | array | static | x + 0x18 | 初始化字符串列表 |
| 5 | modem_initialization_response_timeout | long_unsigned | static | x + 0x20 | 响应超时 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: modem_type

- **类型**: `enum`
- **访问**: static
- **Short Name**: `x + 0x08`

调制解调器类型

#### 属性 3: initialized

- **类型**: `boolean`
- **访问**: dynamic
- **Short Name**: `x + 0x10`

是否已初始化

#### 属性 4: modem_initialization_strings

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x18`

初始化字符串列表

#### 属性 5: modem_initialization_response_timeout

- **类型**: `long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x20`

响应超时

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Modem Configuration 接口类 (Class ID: 27, Version: 1)
/// 
/// Modem Configuration 配置调制解调器参数。
#[derive(Debug, Clone)]
pub struct ModemConfiguration {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 调制解调器类型
    pub modem_type: u8,
    /// 是否已初始化
    pub initialized: bool,
    /// 初始化字符串列表
    pub modem_initialization_strings: Vec<DlmsType>,
    /// 响应超时
    pub modem_initialization_response_timeout: u16,
}

impl ModemConfiguration {
    /// 创建新的 Modem Configuration 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            modem_type: 0,
            initialized: false,
            modem_initialization_strings: Vec::new(),
            modem_initialization_response_timeout: 0,
        }
    }
}

impl CosemObject for ModemConfiguration {
    const CLASS_ID: u16 = 27;
    const VERSION: u8 = 1;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.modem_type.clone().into()),
            3 => Ok(self.initialized.clone().into()),
            4 => Ok(self.modem_initialization_strings.clone().into()),
            5 => Ok(self.modem_initialization_response_timeout.clone().into()),
            _ => Err(CosemError::InvalidAttribute(attr_id)),
        }
    }

    fn set_attribute(&mut self, attr_id: u8, value: DlmsType) -> Result<(), CosemError> {
        match attr_id {
            3 => {
                self.initialized = value.try_into()?;
                Ok(())
            }
            _ => Err(CosemError::ReadOnlyAttribute(attr_id)),
        }
    }

    fn invoke_method(&mut self, method_id: u8, params: DlmsType) -> Result<DlmsType, CosemError> {
        match method_id {
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
    fn test_modem__configuration_class_id() {
        assert_eq!(ModemConfiguration::CLASS_ID, 27);
    }

    #[test]
    fn test_modem__configuration_version() {
        assert_eq!(ModemConfiguration::VERSION, 1);
    }

    #[test]
    fn test_modem__configuration_new() {
        let obis = ObisCode::from_str("0.0.27.0.0.255").unwrap();
        let obj = ModemConfiguration::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_modem__configuration_get_logical_name() {
        let obis = ObisCode::from_str("0.0.27.0.0.255").unwrap();
        let obj = ModemConfiguration::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_modem__configuration_get_modem_type() {
        let obj = ModemConfiguration::new(ObisCode::from_str("0.0.27.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_modem__configuration_get_initialized() {
        let obj = ModemConfiguration::new(ObisCode::from_str("0.0.27.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_modem__configuration_get_modem_initialization_strings() {
        let obj = ModemConfiguration::new(ObisCode::from_str("0.0.27.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `ModemConfiguration` 结构体
- [ ] 实现所有 5 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 27`
- [ ] `VERSION = 1`
- [ ] `get_attribute()` - 实现 5 个属性
- [ ] `set_attribute()` - 实现可写属性

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

*文件名: IC27_ModemConfiguration.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
