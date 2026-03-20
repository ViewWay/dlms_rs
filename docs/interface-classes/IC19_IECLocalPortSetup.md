# IEC Local Port Setup 接口类完整实现规范

**Class ID: 19 | Version: 1 | 优先级: 🟢 低**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

IEC Local Port Setup 配置本地串口参数。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | default_mode | enum | static | x + 0x08 | 默认模式 |
| 3 | default_baud | enum | static | x + 0x10 | 默认波特率 |
| 4 | baud | enum | dynamic | x + 0x18 | 当前波特率 |
| 5 | local_port_state | enum | dynamic | x + 0x20 | 端口状态 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: default_mode

- **类型**: `enum`
- **访问**: static
- **Short Name**: `x + 0x08`

默认模式

#### 属性 3: default_baud

- **类型**: `enum`
- **访问**: static
- **Short Name**: `x + 0x10`

默认波特率

#### 属性 4: baud

- **类型**: `enum`
- **访问**: dynamic
- **Short Name**: `x + 0x18`

当前波特率

#### 属性 5: local_port_state

- **类型**: `enum`
- **访问**: dynamic
- **Short Name**: `x + 0x20`

端口状态

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// IEC Local Port Setup 接口类 (Class ID: 19, Version: 1)
/// 
/// IEC Local Port Setup 配置本地串口参数。
#[derive(Debug, Clone)]
pub struct IecLocalPortSetup {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 默认模式
    pub default_mode: u8,
    /// 默认波特率
    pub default_baud: u8,
    /// 当前波特率
    pub baud: u8,
    /// 端口状态
    pub local_port_state: u8,
}

impl IecLocalPortSetup {
    /// 创建新的 IEC Local Port Setup 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            default_mode: 0,
            default_baud: 0,
            baud: 0,
            local_port_state: 0,
        }
    }
}

impl CosemObject for IecLocalPortSetup {
    const CLASS_ID: u16 = 19;
    const VERSION: u8 = 1;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.default_mode.clone().into()),
            3 => Ok(self.default_baud.clone().into()),
            4 => Ok(self.baud.clone().into()),
            5 => Ok(self.local_port_state.clone().into()),
            _ => Err(CosemError::InvalidAttribute(attr_id)),
        }
    }

    fn set_attribute(&mut self, attr_id: u8, value: DlmsType) -> Result<(), CosemError> {
        match attr_id {
            4 => {
                self.baud = value.try_into()?;
                Ok(())
            }
            5 => {
                self.local_port_state = value.try_into()?;
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
    fn test_iec__local__port__setup_class_id() {
        assert_eq!(IecLocalPortSetup::CLASS_ID, 19);
    }

    #[test]
    fn test_iec__local__port__setup_version() {
        assert_eq!(IecLocalPortSetup::VERSION, 1);
    }

    #[test]
    fn test_iec__local__port__setup_new() {
        let obis = ObisCode::from_str("0.0.19.0.0.255").unwrap();
        let obj = IecLocalPortSetup::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_iec__local__port__setup_get_logical_name() {
        let obis = ObisCode::from_str("0.0.19.0.0.255").unwrap();
        let obj = IecLocalPortSetup::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_iec__local__port__setup_get_default_mode() {
        let obj = IecLocalPortSetup::new(ObisCode::from_str("0.0.19.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_iec__local__port__setup_get_default_baud() {
        let obj = IecLocalPortSetup::new(ObisCode::from_str("0.0.19.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_iec__local__port__setup_get_baud() {
        let obj = IecLocalPortSetup::new(ObisCode::from_str("0.0.19.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `IecLocalPortSetup` 结构体
- [ ] 实现所有 5 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 19`
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

*文件名: IC19_IecLocalPortSetup.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
