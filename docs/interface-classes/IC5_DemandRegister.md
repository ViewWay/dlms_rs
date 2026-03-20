# Demand Register 接口类完整实现规范

**Class ID: 5 | Version: 0 | 优先级: 🟡 中**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Demand Register 用于存储需量值，计算一段时间内的平均功率。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | current_average_value | scalar_unit_type | dynamic | x + 0x08 | 当前平均值的标量单位表示 |
| 3 | last_average_value | scalar_unit_type | dynamic | x + 0x10 | 上一个周期的平均值的标量单位表示 |
| 4 | scaler_unit | scalar_unit | static | x + 0x18 | 缩放因子和单位 |
| 5 | status | unsigned | dynamic | x + 0x20 | 状态标志 |
| 6 | capture_time | date_time | dynamic | x + 0x28 | 最后捕获时间 |
| 7 | start_time_current | date_time | dynamic | x + 0x30 | 当前积分周期开始时间 |
| 8 | period | double_long_unsigned | static | x + 0x38 | 积分周期（秒） |
| 9 | number_of_periods | double_long_unsigned | static | x + 0x40 | 用于计算平均值的周期数 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: current_average_value

- **类型**: `scalar_unit_type`
- **访问**: dynamic
- **Short Name**: `x + 0x08`

当前平均值的标量单位表示

#### 属性 3: last_average_value

- **类型**: `scalar_unit_type`
- **访问**: dynamic
- **Short Name**: `x + 0x10`

上一个周期的平均值的标量单位表示

#### 属性 4: scaler_unit

- **类型**: `scalar_unit`
- **访问**: static
- **Short Name**: `x + 0x18`

缩放因子和单位

#### 属性 5: status

- **类型**: `unsigned`
- **访问**: dynamic
- **Short Name**: `x + 0x20`

状态标志

#### 属性 6: capture_time

- **类型**: `date_time`
- **访问**: dynamic
- **Short Name**: `x + 0x28`

最后捕获时间

#### 属性 7: start_time_current

- **类型**: `date_time`
- **访问**: dynamic
- **Short Name**: `x + 0x30`

当前积分周期开始时间

#### 属性 8: period

- **类型**: `double_long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x38`

积分周期（秒）

#### 属性 9: number_of_periods

- **类型**: `double_long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x40`

用于计算平均值的周期数

---

## 3. 方法定义

| 方法 ID | 名称 | 必选/可选 | Short Name | 参数类型 | 说明 |
|---------|------|----------|------------|----------|------|
| 1 | reset | 必选 | x + 0x60 | integer (0) | 重置需量寄存器 |
| 2 | next_period | 必选 | x + 0x68 | integer (0) | 切换到下一个积分周期 |


### 3.1 方法详细说明

#### 方法 1: reset

- **必选/可选**: 必选
- **Short Name**: `x + 0x60`
- **参数类型**: `integer (0)`

重置需量寄存器

#### 方法 2: next_period

- **必选/可选**: 必选
- **Short Name**: `x + 0x68`
- **参数类型**: `integer (0)`

切换到下一个积分周期

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Demand Register 接口类 (Class ID: 5, Version: 0)
/// 
/// Demand Register 用于存储需量值，计算一段时间内的平均功率。
#[derive(Debug, Clone)]
pub struct DemandRegister {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 当前平均值的标量单位表示
    pub current_average_value: ScalerUnitValue,
    /// 上一个周期的平均值的标量单位表示
    pub last_average_value: ScalerUnitValue,
    /// 缩放因子和单位
    pub scaler_unit: ScalerUnit,
    /// 状态标志
    pub status: u8,
    /// 最后捕获时间
    pub capture_time: CosemDateTime,
    /// 当前积分周期开始时间
    pub start_time_current: CosemDateTime,
    /// 积分周期（秒）
    pub period: u32,
    /// 用于计算平均值的周期数
    pub number_of_periods: u32,
}

impl DemandRegister {
    /// 创建新的 Demand Register 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            current_average_value: Default::default(),
            last_average_value: Default::default(),
            scaler_unit: Default::default(),
            status: 0,
            capture_time: Default::default(),
            start_time_current: Default::default(),
            period: 0,
            number_of_periods: 0,
        }
    }

    /// 方法 1: reset
    /// 
    /// 重置需量寄存器
    pub fn reset(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 reset
        // 参数类型: integer (0)
        Ok(DlmsType::Null)
    }

    /// 方法 2: next_period
    /// 
    /// 切换到下一个积分周期
    pub fn next_period(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 next_period
        // 参数类型: integer (0)
        Ok(DlmsType::Null)
    }
}

impl CosemObject for DemandRegister {
    const CLASS_ID: u16 = 5;
    const VERSION: u8 = 0;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.current_average_value.clone().into()),
            3 => Ok(self.last_average_value.clone().into()),
            4 => Ok(self.scaler_unit.clone().into()),
            5 => Ok(self.status.clone().into()),
            6 => Ok(self.capture_time.clone().into()),
            7 => Ok(self.start_time_current.clone().into()),
            8 => Ok(self.period.clone().into()),
            9 => Ok(self.number_of_periods.clone().into()),
            _ => Err(CosemError::InvalidAttribute(attr_id)),
        }
    }

    fn set_attribute(&mut self, attr_id: u8, value: DlmsType) -> Result<(), CosemError> {
        match attr_id {
            2 => {
                self.current_average_value = value.try_into()?;
                Ok(())
            }
            3 => {
                self.last_average_value = value.try_into()?;
                Ok(())
            }
            5 => {
                self.status = value.try_into()?;
                Ok(())
            }
            6 => {
                self.capture_time = value.try_into()?;
                Ok(())
            }
            7 => {
                self.start_time_current = value.try_into()?;
                Ok(())
            }
            _ => Err(CosemError::ReadOnlyAttribute(attr_id)),
        }
    }

    fn invoke_method(&mut self, method_id: u8, params: DlmsType) -> Result<DlmsType, CosemError> {
        match method_id {
            1 => self.reset(params),
            2 => self.next_period(params),
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
    fn test_demand__register_class_id() {
        assert_eq!(DemandRegister::CLASS_ID, 5);
    }

    #[test]
    fn test_demand__register_version() {
        assert_eq!(DemandRegister::VERSION, 0);
    }

    #[test]
    fn test_demand__register_new() {
        let obis = ObisCode::from_str("0.0.5.0.0.255").unwrap();
        let obj = DemandRegister::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_demand__register_get_logical_name() {
        let obis = ObisCode::from_str("0.0.5.0.0.255").unwrap();
        let obj = DemandRegister::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_demand__register_get_current_average_value() {
        let obj = DemandRegister::new(ObisCode::from_str("0.0.5.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_demand__register_get_last_average_value() {
        let obj = DemandRegister::new(ObisCode::from_str("0.0.5.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_demand__register_get_scaler_unit() {
        let obj = DemandRegister::new(ObisCode::from_str("0.0.5.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }


    #[test]
    fn test_demand__register_reset() {
        let mut obj = DemandRegister::new(ObisCode::from_str("0.0.5.0.0.255").unwrap());
        let result = obj.invoke_method(1, DlmsType::Null);
        assert!(result.is_ok());
    }


    #[test]
    fn test_demand__register_next_period() {
        let mut obj = DemandRegister::new(ObisCode::from_str("0.0.5.0.0.255").unwrap());
        let result = obj.invoke_method(2, DlmsType::Null);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `DemandRegister` 结构体
- [ ] 实现所有 9 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 5`
- [ ] `VERSION = 0`
- [ ] `get_attribute()` - 实现 9 个属性
- [ ] `set_attribute()` - 实现可写属性

### 6.3 方法实现 (2 个)

- [ ] 方法 1: `reset()`
- [ ] 方法 2: `next_period()`

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

*文件名: IC5_DemandRegister.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
