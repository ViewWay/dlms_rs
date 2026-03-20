# Activity Calendar 接口类完整实现规范

**Class ID: 20 | Version: 0 | 优先级: 🟡 中**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Activity Calendar 管理费率时段和日历。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | calendar_name_active | visible_string | dynamic | x + 0x08 | - |
| 3 | calendar_name_passive | visible_string | static | x + 0x10 | - |
| 4 | season_list_active | array | dynamic | x + 0x18 | - |
| 5 | season_list_passive | array | static | x + 0x20 | - |
| 6 | week_profile_list_active | array | dynamic | x + 0x28 | - |
| 7 | week_profile_list_passive | array | static | x + 0x30 | - |
| 8 | day_profile_list_active | array | dynamic | x + 0x38 | - |
| 9 | day_profile_list_passive | array | static | x + 0x40 | - |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: calendar_name_active

- **类型**: `visible_string`
- **访问**: dynamic
- **Short Name**: `x + 0x08`

待补充

#### 属性 3: calendar_name_passive

- **类型**: `visible_string`
- **访问**: static
- **Short Name**: `x + 0x10`

待补充

#### 属性 4: season_list_active

- **类型**: `array`
- **访问**: dynamic
- **Short Name**: `x + 0x18`

待补充

#### 属性 5: season_list_passive

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x20`

待补充

#### 属性 6: week_profile_list_active

- **类型**: `array`
- **访问**: dynamic
- **Short Name**: `x + 0x28`

待补充

#### 属性 7: week_profile_list_passive

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x30`

待补充

#### 属性 8: day_profile_list_active

- **类型**: `array`
- **访问**: dynamic
- **Short Name**: `x + 0x38`

待补充

#### 属性 9: day_profile_list_passive

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x40`

待补充

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Activity Calendar 接口类 (Class ID: 20, Version: 0)
/// 
/// Activity Calendar 管理费率时段和日历。
#[derive(Debug, Clone)]
pub struct ActivityCalendar {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// calendar_name_active
    pub calendar_name_active: String,
    /// calendar_name_passive
    pub calendar_name_passive: String,
    /// season_list_active
    pub season_list_active: Vec<DlmsType>,
    /// season_list_passive
    pub season_list_passive: Vec<DlmsType>,
    /// week_profile_list_active
    pub week_profile_list_active: Vec<DlmsType>,
    /// week_profile_list_passive
    pub week_profile_list_passive: Vec<DlmsType>,
    /// day_profile_list_active
    pub day_profile_list_active: Vec<DlmsType>,
    /// day_profile_list_passive
    pub day_profile_list_passive: Vec<DlmsType>,
}

impl ActivityCalendar {
    /// 创建新的 Activity Calendar 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            calendar_name_active: String::new(),
            calendar_name_passive: String::new(),
            season_list_active: Vec::new(),
            season_list_passive: Vec::new(),
            week_profile_list_active: Vec::new(),
            week_profile_list_passive: Vec::new(),
            day_profile_list_active: Vec::new(),
            day_profile_list_passive: Vec::new(),
        }
    }
}

impl CosemObject for ActivityCalendar {
    const CLASS_ID: u16 = 20;
    const VERSION: u8 = 0;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.calendar_name_active.clone().into()),
            3 => Ok(self.calendar_name_passive.clone().into()),
            4 => Ok(self.season_list_active.clone().into()),
            5 => Ok(self.season_list_passive.clone().into()),
            6 => Ok(self.week_profile_list_active.clone().into()),
            7 => Ok(self.week_profile_list_passive.clone().into()),
            8 => Ok(self.day_profile_list_active.clone().into()),
            9 => Ok(self.day_profile_list_passive.clone().into()),
            _ => Err(CosemError::InvalidAttribute(attr_id)),
        }
    }

    fn set_attribute(&mut self, attr_id: u8, value: DlmsType) -> Result<(), CosemError> {
        match attr_id {
            2 => {
                self.calendar_name_active = value.try_into()?;
                Ok(())
            }
            4 => {
                self.season_list_active = value.try_into()?;
                Ok(())
            }
            6 => {
                self.week_profile_list_active = value.try_into()?;
                Ok(())
            }
            8 => {
                self.day_profile_list_active = value.try_into()?;
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
    fn test_activity__calendar_class_id() {
        assert_eq!(ActivityCalendar::CLASS_ID, 20);
    }

    #[test]
    fn test_activity__calendar_version() {
        assert_eq!(ActivityCalendar::VERSION, 0);
    }

    #[test]
    fn test_activity__calendar_new() {
        let obis = ObisCode::from_str("0.0.20.0.0.255").unwrap();
        let obj = ActivityCalendar::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_activity__calendar_get_logical_name() {
        let obis = ObisCode::from_str("0.0.20.0.0.255").unwrap();
        let obj = ActivityCalendar::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_activity__calendar_get_calendar_name_active() {
        let obj = ActivityCalendar::new(ObisCode::from_str("0.0.20.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_activity__calendar_get_calendar_name_passive() {
        let obj = ActivityCalendar::new(ObisCode::from_str("0.0.20.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_activity__calendar_get_season_list_active() {
        let obj = ActivityCalendar::new(ObisCode::from_str("0.0.20.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `ActivityCalendar` 结构体
- [ ] 实现所有 9 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 20`
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

*文件名: IC20_ActivityCalendar.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
