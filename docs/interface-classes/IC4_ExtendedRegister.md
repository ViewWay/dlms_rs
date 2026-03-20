# Extended Register 接口类完整实现规范

**Class ID: 4 | Version: 0 | 优先级: 🟡 中**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Extended Register 扩展了基本 Register，增加了状态和时间戳信息。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | 对象实例标识 |
| 2 | value | scalar_unit_type | dynamic | x + 0x08 | 寄存器值 |
| 3 | scaler_unit | scalar_unit | static | x + 0x10 | 缩放因子和单位 |
| 4 | status | unsigned | dynamic | x + 0x18 | 寄存器状态标志 |
| 5 | capture_time | date_time | dynamic | x + 0x20 | 最后一次捕获的时间 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

对象实例标识

#### 属性 2: value

- **类型**: `scalar_unit_type`
- **访问**: dynamic
- **Short Name**: `x + 0x08`

寄存器值

#### 属性 3: scaler_unit

- **类型**: `scalar_unit`
- **访问**: static
- **Short Name**: `x + 0x10`

缩放因子和单位

#### 属性 4: status

- **类型**: `unsigned`
- **访问**: dynamic
- **Short Name**: `x + 0x18`

寄存器状态标志

#### 属性 5: capture_time

- **类型**: `date_time`
- **访问**: dynamic
- **Short Name**: `x + 0x20`

最后一次捕获的时间

---

## 3. 方法定义

| 方法 ID | 名称 | 必选/可选 | Short Name | 参数类型 | 说明 |
|---------|------|----------|------------|----------|------|
| 1 | reset | 必选 | x + 0x60 | integer (0) | 重置寄存器 |


### 3.1 方法详细说明

#### 方法 1: reset

- **必选/可选**: 必选
- **Short Name**: `x + 0x60`
- **参数类型**: `integer (0)`

重置寄存器

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Extended Register 接口类 (Class ID: 4, Version: 0)
/// 
/// Extended Register 扩展了基本 Register，增加了状态和时间戳信息。
#[derive(Debug, Clone)]
pub struct ExtendedRegister {
    /// 对象实例标识
    pub logical_name: Vec<u8>,
    /// 寄存器值
    pub value: ScalerUnitValue,
    /// 缩放因子和单位
    pub scaler_unit: ScalerUnit,
    /// 寄存器状态标志
    pub status: u8,
    /// 最后一次捕获的时间
    pub capture_time: CosemDateTime,
}

impl ExtendedRegister {
    /// 创建新的 Extended Register 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            value: Default::default(),
            scaler_unit: Default::default(),
            status: 0,
            capture_time: Default::default(),
        }
    }

    /// 方法 1: reset
    /// 
    /// 重置寄存器
    pub fn reset(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 reset
        // 参数类型: integer (0)
        Ok(DlmsType::Null)
    }
}

impl CosemObject for ExtendedRegister {
    const CLASS_ID: u16 = 4;
    const VERSION: u8 = 0;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.value.clone().into()),
            3 => Ok(self.scaler_unit.clone().into()),
            4 => Ok(self.status.clone().into()),
            5 => Ok(self.capture_time.clone().into()),
            _ => Err(CosemError::InvalidAttribute(attr_id)),
        }
    }

    fn set_attribute(&mut self, attr_id: u8, value: DlmsType) -> Result<(), CosemError> {
        match attr_id {
            2 => {
                self.value = value.try_into()?;
                Ok(())
            }
            4 => {
                self.status = value.try_into()?;
                Ok(())
            }
            5 => {
                self.capture_time = value.try_into()?;
                Ok(())
            }
            _ => Err(CosemError::ReadOnlyAttribute(attr_id)),
        }
    }

    fn invoke_method(&mut self, method_id: u8, params: DlmsType) -> Result<DlmsType, CosemError> {
        match method_id {
            1 => self.reset(params),
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
    fn test_extended__register_class_id() {
        assert_eq!(ExtendedRegister::CLASS_ID, 4);
    }

    #[test]
    fn test_extended__register_version() {
        assert_eq!(ExtendedRegister::VERSION, 0);
    }

    #[test]
    fn test_extended__register_new() {
        let obis = ObisCode::from_str("0.0.4.0.0.255").unwrap();
        let obj = ExtendedRegister::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_extended__register_get_logical_name() {
        let obis = ObisCode::from_str("0.0.4.0.0.255").unwrap();
        let obj = ExtendedRegister::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_extended__register_get_value() {
        let obj = ExtendedRegister::new(ObisCode::from_str("0.0.4.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_extended__register_get_scaler_unit() {
        let obj = ExtendedRegister::new(ObisCode::from_str("0.0.4.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_extended__register_get_status() {
        let obj = ExtendedRegister::new(ObisCode::from_str("0.0.4.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }


    #[test]
    fn test_extended__register_reset() {
        let mut obj = ExtendedRegister::new(ObisCode::from_str("0.0.4.0.0.255").unwrap());
        let result = obj.invoke_method(1, DlmsType::Null);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `ExtendedRegister` 结构体
- [ ] 实现所有 5 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 4`
- [ ] `VERSION = 0`
- [ ] `get_attribute()` - 实现 5 个属性
- [ ] `set_attribute()` - 实现可写属性

### 6.3 方法实现 (1 个)

- [ ] 方法 1: `reset()`

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

*文件名: IC4_ExtendedRegister.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
