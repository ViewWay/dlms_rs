# Clock 接口类完整实现规范

**Class ID: 8 | Version: 0 | 优先级: 🔴 高**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Clock 接口类管理设备的日期时间，包括时区和夏令时。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | time | date_time | dynamic | x + 0x08 | 当前本地日期时间（12字节） |
| 3 | time_zone | long | static | x + 0x10 | 时区偏差（分钟），UTC+8 = 480 |
| 4 | status | unsigned | dynamic | x + 0x18 | 时钟状态标志 |
| 5 | daylight_savings_begin | date_time | static | x + 0x20 | 夏令时开始时间 |
| 6 | daylight_savings_end | date_time | static | x + 0x28 | 夏令时结束时间 |
| 7 | daylight_savings_deviation | integer | static | x + 0x30 | 夏令时偏差（分钟） |
| 8 | daylight_savings_enabled | boolean | static | x + 0x38 | 是否启用夏令时 |
| 9 | clock_base | enum | static | x + 0x40 | 时钟源: 0=未定义, 1=晶振, 2=50Hz, 3=60Hz, 4=GPS, 5=无线电 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: time

- **类型**: `date_time`
- **访问**: dynamic
- **Short Name**: `x + 0x08`

当前本地日期时间（12字节）

#### 属性 3: time_zone

- **类型**: `long`
- **访问**: static
- **Short Name**: `x + 0x10`
- **范围**: -840 ~ +720

时区偏差（分钟），UTC+8 = 480

#### 属性 4: status

- **类型**: `unsigned`
- **访问**: dynamic
- **Short Name**: `x + 0x18`

时钟状态标志

#### 属性 5: daylight_savings_begin

- **类型**: `date_time`
- **访问**: static
- **Short Name**: `x + 0x20`

夏令时开始时间

#### 属性 6: daylight_savings_end

- **类型**: `date_time`
- **访问**: static
- **Short Name**: `x + 0x28`

夏令时结束时间

#### 属性 7: daylight_savings_deviation

- **类型**: `integer`
- **访问**: static
- **Short Name**: `x + 0x30`
- **范围**: -120 ~ +120

夏令时偏差（分钟）

#### 属性 8: daylight_savings_enabled

- **类型**: `boolean`
- **访问**: static
- **Short Name**: `x + 0x38`

是否启用夏令时

#### 属性 9: clock_base

- **类型**: `enum`
- **访问**: static
- **Short Name**: `x + 0x40`

时钟源: 0=未定义, 1=晶振, 2=50Hz, 3=60Hz, 4=GPS, 5=无线电

---

## 3. 方法定义

| 方法 ID | 名称 | 必选/可选 | Short Name | 参数类型 | 说明 |
|---------|------|----------|------------|----------|------|
| 1 | adjust_to_quarter | 可选 | x + 0x60 | integer (0) | 调整到最近的15分钟整点 |
| 2 | adjust_to_measuring_period | 可选 | x + 0x68 | integer (0) | 调整到最近的计量周期起点 |
| 3 | adjust_to_minute | 可选 | x + 0x70 | integer (0) | 调整到最近的分钟整点 |
| 4 | adjust_to_preset_time | 可选 | x + 0x78 | integer (0) | 调整到预设时间 |
| 5 | preset_adjusting_time | 可选 | x + 0x80 | structure | 设置预设时间和有效期 |
| 6 | shift_time | 可选 | x + 0x88 | long | 时间偏移（秒），范围 -900 ~ +900 |


### 3.1 方法详细说明

#### 方法 1: adjust_to_quarter

- **必选/可选**: 可选
- **Short Name**: `x + 0x60`
- **参数类型**: `integer (0)`

调整到最近的15分钟整点

#### 方法 2: adjust_to_measuring_period

- **必选/可选**: 可选
- **Short Name**: `x + 0x68`
- **参数类型**: `integer (0)`

调整到最近的计量周期起点

#### 方法 3: adjust_to_minute

- **必选/可选**: 可选
- **Short Name**: `x + 0x70`
- **参数类型**: `integer (0)`

调整到最近的分钟整点

#### 方法 4: adjust_to_preset_time

- **必选/可选**: 可选
- **Short Name**: `x + 0x78`
- **参数类型**: `integer (0)`

调整到预设时间

#### 方法 5: preset_adjusting_time

- **必选/可选**: 可选
- **Short Name**: `x + 0x80`
- **参数类型**: `structure`

设置预设时间和有效期

#### 方法 6: shift_time

- **必选/可选**: 可选
- **Short Name**: `x + 0x88`
- **参数类型**: `long`

时间偏移（秒），范围 -900 ~ +900

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Clock 接口类 (Class ID: 8, Version: 0)
/// 
/// Clock 接口类管理设备的日期时间，包括时区和夏令时。
#[derive(Debug, Clone)]
pub struct Clock {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 当前本地日期时间（12字节）
    pub time: CosemDateTime,
    /// 时区偏差（分钟），UTC+8 = 480
    pub time_zone: i16,
    /// 时钟状态标志
    pub status: u8,
    /// 夏令时开始时间
    pub daylight_savings_begin: CosemDateTime,
    /// 夏令时结束时间
    pub daylight_savings_end: CosemDateTime,
    /// 夏令时偏差（分钟）
    pub daylight_savings_deviation: i8,
    /// 是否启用夏令时
    pub daylight_savings_enabled: bool,
    /// 时钟源: 0=未定义, 1=晶振, 2=50Hz, 3=60Hz, 4=GPS, 5=无线电
    pub clock_base: u8,
}

impl Clock {
    /// 创建新的 Clock 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            time: Default::default(),
            time_zone: 0,
            status: 0,
            daylight_savings_begin: Default::default(),
            daylight_savings_end: Default::default(),
            daylight_savings_deviation: 0,
            daylight_savings_enabled: false,
            clock_base: 0,
        }
    }

    /// 方法 1: adjust_to_quarter
    /// 
    /// 调整到最近的15分钟整点
    pub fn adjust_to_quarter(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 adjust_to_quarter
        // 参数类型: integer (0)
        Ok(DlmsType::Null)
    }

    /// 方法 2: adjust_to_measuring_period
    /// 
    /// 调整到最近的计量周期起点
    pub fn adjust_to_measuring_period(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 adjust_to_measuring_period
        // 参数类型: integer (0)
        Ok(DlmsType::Null)
    }

    /// 方法 3: adjust_to_minute
    /// 
    /// 调整到最近的分钟整点
    pub fn adjust_to_minute(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 adjust_to_minute
        // 参数类型: integer (0)
        Ok(DlmsType::Null)
    }

    /// 方法 4: adjust_to_preset_time
    /// 
    /// 调整到预设时间
    pub fn adjust_to_preset_time(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 adjust_to_preset_time
        // 参数类型: integer (0)
        Ok(DlmsType::Null)
    }

    /// 方法 5: preset_adjusting_time
    /// 
    /// 设置预设时间和有效期
    pub fn preset_adjusting_time(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 preset_adjusting_time
        // 参数类型: structure
        Ok(DlmsType::Null)
    }

    /// 方法 6: shift_time
    /// 
    /// 时间偏移（秒），范围 -900 ~ +900
    pub fn shift_time(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 shift_time
        // 参数类型: long
        Ok(DlmsType::Null)
    }
}

impl CosemObject for Clock {
    const CLASS_ID: u16 = 8;
    const VERSION: u8 = 0;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.time.clone().into()),
            3 => Ok(self.time_zone.clone().into()),
            4 => Ok(self.status.clone().into()),
            5 => Ok(self.daylight_savings_begin.clone().into()),
            6 => Ok(self.daylight_savings_end.clone().into()),
            7 => Ok(self.daylight_savings_deviation.clone().into()),
            8 => Ok(self.daylight_savings_enabled.clone().into()),
            9 => Ok(self.clock_base.clone().into()),
            _ => Err(CosemError::InvalidAttribute(attr_id)),
        }
    }

    fn set_attribute(&mut self, attr_id: u8, value: DlmsType) -> Result<(), CosemError> {
        match attr_id {
            2 => {
                self.time = value.try_into()?;
                Ok(())
            }
            4 => {
                self.status = value.try_into()?;
                Ok(())
            }
            _ => Err(CosemError::ReadOnlyAttribute(attr_id)),
        }
    }

    fn invoke_method(&mut self, method_id: u8, params: DlmsType) -> Result<DlmsType, CosemError> {
        match method_id {
            1 => self.adjust_to_quarter(params),
            2 => self.adjust_to_measuring_period(params),
            3 => self.adjust_to_minute(params),
            4 => self.adjust_to_preset_time(params),
            5 => self.preset_adjusting_time(params),
            6 => self.shift_time(params),
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
    fn test_clock_class_id() {
        assert_eq!(Clock::CLASS_ID, 8);
    }

    #[test]
    fn test_clock_version() {
        assert_eq!(Clock::VERSION, 0);
    }

    #[test]
    fn test_clock_new() {
        let obis = ObisCode::from_str("0.0.8.0.0.255").unwrap();
        let obj = Clock::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_clock_get_logical_name() {
        let obis = ObisCode::from_str("0.0.8.0.0.255").unwrap();
        let obj = Clock::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_clock_get_time() {
        let obj = Clock::new(ObisCode::from_str("0.0.8.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_clock_get_time_zone() {
        let obj = Clock::new(ObisCode::from_str("0.0.8.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_clock_get_status() {
        let obj = Clock::new(ObisCode::from_str("0.0.8.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }


    #[test]
    fn test_clock_adjust_to_quarter() {
        let mut obj = Clock::new(ObisCode::from_str("0.0.8.0.0.255").unwrap());
        let result = obj.invoke_method(1, DlmsType::Null);
        assert!(result.is_ok());
    }


    #[test]
    fn test_clock_adjust_to_measuring_period() {
        let mut obj = Clock::new(ObisCode::from_str("0.0.8.0.0.255").unwrap());
        let result = obj.invoke_method(2, DlmsType::Null);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `Clock` 结构体
- [ ] 实现所有 9 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 8`
- [ ] `VERSION = 0`
- [ ] `get_attribute()` - 实现 9 个属性
- [ ] `set_attribute()` - 实现可写属性

### 6.3 方法实现 (6 个)

- [ ] 方法 1: `adjust_to_quarter()`
- [ ] 方法 2: `adjust_to_measuring_period()`
- [ ] 方法 3: `adjust_to_minute()`
- [ ] 方法 4: `adjust_to_preset_time()`
- [ ] 方法 5: `preset_adjusting_time()`
- [ ] 方法 6: `shift_time()`

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

*文件名: IC8_Clock.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
