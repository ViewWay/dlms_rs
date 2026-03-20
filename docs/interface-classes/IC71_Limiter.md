# Limiter 接口类完整实现规范

**Class ID: 71 | Version: 0 | 优先级: 🟡 中**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Limiter 限制负载或功率。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | monitored_value | capture_object_definition | static | x + 0x08 | 监控的对象 |
| 3 | threshold_normal_value | any type | static | x + 0x10 | 正常阈值 |
| 4 | threshold_normal_under_over | enum | static | x + 0x18 | 阈值方向: 0=under, 1=over |
| 5 | min_over_threshold_duration | long_unsigned | static | x + 0x20 | 超过阈值最小时长（秒） |
| 6 | min_under_threshold_duration | long_unsigned | static | x + 0x28 | 低于阈值最小时长（秒） |
| 7 | emergency_profile | structure | static | x + 0x30 | 紧急配置 |
| 8 | emergency_profile_group_id_list | array | static | x + 0x38 | 紧急配置组 ID 列表 |
| 9 | emergency_profile_active | boolean | dynamic | x + 0x40 | 紧急配置是否激活 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: monitored_value

- **类型**: `capture_object_definition`
- **访问**: static
- **Short Name**: `x + 0x08`

监控的对象

#### 属性 3: threshold_normal_value

- **类型**: `any type`
- **访问**: static
- **Short Name**: `x + 0x10`

正常阈值

#### 属性 4: threshold_normal_under_over

- **类型**: `enum`
- **访问**: static
- **Short Name**: `x + 0x18`

阈值方向: 0=under, 1=over

#### 属性 5: min_over_threshold_duration

- **类型**: `long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x20`

超过阈值最小时长（秒）

#### 属性 6: min_under_threshold_duration

- **类型**: `long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x28`

低于阈值最小时长（秒）

#### 属性 7: emergency_profile

- **类型**: `structure`
- **访问**: static
- **Short Name**: `x + 0x30`

紧急配置

#### 属性 8: emergency_profile_group_id_list

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x38`

紧急配置组 ID 列表

#### 属性 9: emergency_profile_active

- **类型**: `boolean`
- **访问**: dynamic
- **Short Name**: `x + 0x40`

紧急配置是否激活

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Limiter 接口类 (Class ID: 71, Version: 0)
/// 
/// Limiter 限制负载或功率。
#[derive(Debug, Clone)]
pub struct Limiter {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 监控的对象
    pub monitored_value: CaptureObjectDefinition,
    /// 正常阈值
    pub threshold_normal_value: DlmsType,
    /// 阈值方向: 0=under, 1=over
    pub threshold_normal_under_over: u8,
    /// 超过阈值最小时长（秒）
    pub min_over_threshold_duration: u16,
    /// 低于阈值最小时长（秒）
    pub min_under_threshold_duration: u16,
    /// 紧急配置
    pub emergency_profile: Vec<DlmsType>,
    /// 紧急配置组 ID 列表
    pub emergency_profile_group_id_list: Vec<DlmsType>,
    /// 紧急配置是否激活
    pub emergency_profile_active: bool,
}

impl Limiter {
    /// 创建新的 Limiter 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            monitored_value: Default::default(),
            threshold_normal_value: DlmsType::Null,
            threshold_normal_under_over: 0,
            min_over_threshold_duration: 0,
            min_under_threshold_duration: 0,
            emergency_profile: Vec::new(),
            emergency_profile_group_id_list: Vec::new(),
            emergency_profile_active: false,
        }
    }
}

impl CosemObject for Limiter {
    const CLASS_ID: u16 = 71;
    const VERSION: u8 = 0;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.monitored_value.clone().into()),
            3 => Ok(self.threshold_normal_value.clone().into()),
            4 => Ok(self.threshold_normal_under_over.clone().into()),
            5 => Ok(self.min_over_threshold_duration.clone().into()),
            6 => Ok(self.min_under_threshold_duration.clone().into()),
            7 => Ok(self.emergency_profile.clone().into()),
            8 => Ok(self.emergency_profile_group_id_list.clone().into()),
            9 => Ok(self.emergency_profile_active.clone().into()),
            _ => Err(CosemError::InvalidAttribute(attr_id)),
        }
    }

    fn set_attribute(&mut self, attr_id: u8, value: DlmsType) -> Result<(), CosemError> {
        match attr_id {
            9 => {
                self.emergency_profile_active = value.try_into()?;
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
    fn test_limiter_class_id() {
        assert_eq!(Limiter::CLASS_ID, 71);
    }

    #[test]
    fn test_limiter_version() {
        assert_eq!(Limiter::VERSION, 0);
    }

    #[test]
    fn test_limiter_new() {
        let obis = ObisCode::from_str("0.0.71.0.0.255").unwrap();
        let obj = Limiter::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_limiter_get_logical_name() {
        let obis = ObisCode::from_str("0.0.71.0.0.255").unwrap();
        let obj = Limiter::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_limiter_get_monitored_value() {
        let obj = Limiter::new(ObisCode::from_str("0.0.71.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_limiter_get_threshold_normal_value() {
        let obj = Limiter::new(ObisCode::from_str("0.0.71.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_limiter_get_threshold_normal_under_over() {
        let obj = Limiter::new(ObisCode::from_str("0.0.71.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `Limiter` 结构体
- [ ] 实现所有 9 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 71`
- [ ] `VERSION = 0`
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

*文件名: IC71_Limiter.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
