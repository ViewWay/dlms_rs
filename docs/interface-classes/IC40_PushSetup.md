# Push Setup 接口类完整实现规范

**Class ID: 40 | Version: 3 | 优先级: 🟡 中**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Push Setup 配置数据推送功能，用于主动上报数据。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | push_object_list | array | static | x + 0x08 | 要推送的对象列表 |
| 3 | send_destination_and_method | structure | static | x + 0x10 | 发送目标和方法 |
| 4 | randomisation_interval_of_initial_delivery | long_unsigned | static | x + 0x18 | 初始投递随机间隔 |
| 5 | number_of_retries | unsigned | static | x + 0x20 | 重试次数 |
| 6 | recurrence_delay | long_unsigned | static | x + 0x28 | 重复延迟 |
| 7 | allowed_send_methods | bit_string | static | x + 0x30 | 允许的发送方法 |
| 8 | allowed_sap_list | array | static | x + 0x38 | 允许的 SAP 列表 |
| 9 | max_send_delay | long_unsigned | static | x + 0x40 | 最大发送延迟 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: push_object_list

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x08`

要推送的对象列表

#### 属性 3: send_destination_and_method

- **类型**: `structure`
- **访问**: static
- **Short Name**: `x + 0x10`

发送目标和方法

#### 属性 4: randomisation_interval_of_initial_delivery

- **类型**: `long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x18`

初始投递随机间隔

#### 属性 5: number_of_retries

- **类型**: `unsigned`
- **访问**: static
- **Short Name**: `x + 0x20`

重试次数

#### 属性 6: recurrence_delay

- **类型**: `long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x28`

重复延迟

#### 属性 7: allowed_send_methods

- **类型**: `bit_string`
- **访问**: static
- **Short Name**: `x + 0x30`

允许的发送方法

#### 属性 8: allowed_sap_list

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x38`

允许的 SAP 列表

#### 属性 9: max_send_delay

- **类型**: `long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x40`

最大发送延迟

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Push Setup 接口类 (Class ID: 40, Version: 3)
/// 
/// Push Setup 配置数据推送功能，用于主动上报数据。
#[derive(Debug, Clone)]
pub struct PushSetup {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 要推送的对象列表
    pub push_object_list: Vec<DlmsType>,
    /// 发送目标和方法
    pub send_destination_and_method: Vec<DlmsType>,
    /// 初始投递随机间隔
    pub randomisation_interval_of_initial_delivery: u16,
    /// 重试次数
    pub number_of_retries: u8,
    /// 重复延迟
    pub recurrence_delay: u16,
    /// 允许的发送方法
    pub allowed_send_methods: Vec<u8>,
    /// 允许的 SAP 列表
    pub allowed_sap_list: Vec<DlmsType>,
    /// 最大发送延迟
    pub max_send_delay: u16,
}

impl PushSetup {
    /// 创建新的 Push Setup 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            push_object_list: Vec::new(),
            send_destination_and_method: Vec::new(),
            randomisation_interval_of_initial_delivery: 0,
            number_of_retries: 0,
            recurrence_delay: 0,
            allowed_send_methods: Vec::new(),
            allowed_sap_list: Vec::new(),
            max_send_delay: 0,
        }
    }
}

impl CosemObject for PushSetup {
    const CLASS_ID: u16 = 40;
    const VERSION: u8 = 3;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.push_object_list.clone().into()),
            3 => Ok(self.send_destination_and_method.clone().into()),
            4 => Ok(self.randomisation_interval_of_initial_delivery.clone().into()),
            5 => Ok(self.number_of_retries.clone().into()),
            6 => Ok(self.recurrence_delay.clone().into()),
            7 => Ok(self.allowed_send_methods.clone().into()),
            8 => Ok(self.allowed_sap_list.clone().into()),
            9 => Ok(self.max_send_delay.clone().into()),
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
    fn test_push__setup_class_id() {
        assert_eq!(PushSetup::CLASS_ID, 40);
    }

    #[test]
    fn test_push__setup_version() {
        assert_eq!(PushSetup::VERSION, 3);
    }

    #[test]
    fn test_push__setup_new() {
        let obis = ObisCode::from_str("0.0.40.0.0.255").unwrap();
        let obj = PushSetup::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_push__setup_get_logical_name() {
        let obis = ObisCode::from_str("0.0.40.0.0.255").unwrap();
        let obj = PushSetup::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_push__setup_get_push_object_list() {
        let obj = PushSetup::new(ObisCode::from_str("0.0.40.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_push__setup_get_send_destination_and_method() {
        let obj = PushSetup::new(ObisCode::from_str("0.0.40.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_push__setup_get_randomisation_interval_of_initial_delivery() {
        let obj = PushSetup::new(ObisCode::from_str("0.0.40.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `PushSetup` 结构体
- [ ] 实现所有 9 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 40`
- [ ] `VERSION = 3`
- [ ] `get_attribute()` - 实现 9 个属性
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

*文件名: IC40_PushSetup.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
