# Register Monitor 接口类完整实现规范

**Class ID: 21 | Version: 0 | 优先级: 🟡 中**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Register Monitor 监控寄存器值并触发动作。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | thresholds | array | static | x + 0x08 | 阈值列表 |
| 3 | monitored_value | capture_object_definition | static | x + 0x10 | 监控的对象 |
| 4 | actions | array | static | x + 0x18 | 触发的动作列表 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: thresholds

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x08`

阈值列表

#### 属性 3: monitored_value

- **类型**: `capture_object_definition`
- **访问**: static
- **Short Name**: `x + 0x10`

监控的对象

#### 属性 4: actions

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x18`

触发的动作列表

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Register Monitor 接口类 (Class ID: 21, Version: 0)
/// 
/// Register Monitor 监控寄存器值并触发动作。
#[derive(Debug, Clone)]
pub struct RegisterMonitor {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 阈值列表
    pub thresholds: Vec<DlmsType>,
    /// 监控的对象
    pub monitored_value: CaptureObjectDefinition,
    /// 触发的动作列表
    pub actions: Vec<DlmsType>,
}

impl RegisterMonitor {
    /// 创建新的 Register Monitor 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            thresholds: Vec::new(),
            monitored_value: Default::default(),
            actions: Vec::new(),
        }
    }
}

impl CosemObject for RegisterMonitor {
    const CLASS_ID: u16 = 21;
    const VERSION: u8 = 0;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.thresholds.clone().into()),
            3 => Ok(self.monitored_value.clone().into()),
            4 => Ok(self.actions.clone().into()),
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
    fn test_register__monitor_class_id() {
        assert_eq!(RegisterMonitor::CLASS_ID, 21);
    }

    #[test]
    fn test_register__monitor_version() {
        assert_eq!(RegisterMonitor::VERSION, 0);
    }

    #[test]
    fn test_register__monitor_new() {
        let obis = ObisCode::from_str("0.0.21.0.0.255").unwrap();
        let obj = RegisterMonitor::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_register__monitor_get_logical_name() {
        let obis = ObisCode::from_str("0.0.21.0.0.255").unwrap();
        let obj = RegisterMonitor::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_register__monitor_get_thresholds() {
        let obj = RegisterMonitor::new(ObisCode::from_str("0.0.21.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_register__monitor_get_monitored_value() {
        let obj = RegisterMonitor::new(ObisCode::from_str("0.0.21.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_register__monitor_get_actions() {
        let obj = RegisterMonitor::new(ObisCode::from_str("0.0.21.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `RegisterMonitor` 结构体
- [ ] 实现所有 4 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 21`
- [ ] `VERSION = 0`
- [ ] `get_attribute()` - 实现 4 个属性
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

*文件名: IC21_RegisterMonitor.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
