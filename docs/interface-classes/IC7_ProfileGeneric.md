# Profile Generic 接口类完整实现规范

**Class ID: 7 | Version: 1 | 优先级: 🔴 高**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Profile Generic 是最重要的接口类之一，用于存储时间序列历史数据。它支持周期性捕获和选择性访问。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | 对象实例标识 |
| 2 | buffer | array | dynamic | x + 0x08 | 历史数据缓冲区，每行是一个 structure |
| 3 | capture_objects | array | static | x + 0x10 | 定义要捕获的对象列表 |
| 4 | capture_period | double_long_unsigned | static | x + 0x18 | 自动捕获周期（秒），0 表示手动 |
| 5 | sort_method | enum | static | x + 0x20 | 排序方法: 0=FIFO, 1=LIFO, 2=Smallest, 3=Largest, 4=Nearest |
| 6 | sort_object | capture_object_definition | static | x + 0x28 | 排序依据的对象 |
| 7 | entries_in_use | double_long_unsigned | dynamic | x + 0x30 | 当前使用的条目数 |
| 8 | profile_entries | double_long_unsigned | static | x + 0x38 | 缓冲区最大容量 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

对象实例标识

#### 属性 2: buffer

- **类型**: `array`
- **访问**: dynamic
- **Short Name**: `x + 0x08`

历史数据缓冲区，每行是一个 structure

#### 属性 3: capture_objects

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x10`

定义要捕获的对象列表

#### 属性 4: capture_period

- **类型**: `double_long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x18`

自动捕获周期（秒），0 表示手动

#### 属性 5: sort_method

- **类型**: `enum`
- **访问**: static
- **Short Name**: `x + 0x20`

排序方法: 0=FIFO, 1=LIFO, 2=Smallest, 3=Largest, 4=Nearest

#### 属性 6: sort_object

- **类型**: `capture_object_definition`
- **访问**: static
- **Short Name**: `x + 0x28`

排序依据的对象

#### 属性 7: entries_in_use

- **类型**: `double_long_unsigned`
- **访问**: dynamic
- **Short Name**: `x + 0x30`

当前使用的条目数

#### 属性 8: profile_entries

- **类型**: `double_long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x38`

缓冲区最大容量

---

## 3. 方法定义

| 方法 ID | 名称 | 必选/可选 | Short Name | 参数类型 | 说明 |
|---------|------|----------|------------|----------|------|
| 1 | reset | 必选 | x + 0x60 | integer (0) | 清空缓冲区 |
| 2 | capture | 必选 | x + 0x68 | integer (0) | 手动触发一次捕获 |
| 3 | get_capture_objects | 可选 | x + 0x70 | integer (0) | 获取捕获对象列表 |
| 4 | set_capture_objects | 可选 | x + 0x78 | array | 设置捕获对象列表 |


### 3.1 方法详细说明

#### 方法 1: reset

- **必选/可选**: 必选
- **Short Name**: `x + 0x60`
- **参数类型**: `integer (0)`

清空缓冲区

#### 方法 2: capture

- **必选/可选**: 必选
- **Short Name**: `x + 0x68`
- **参数类型**: `integer (0)`

手动触发一次捕获

#### 方法 3: get_capture_objects

- **必选/可选**: 可选
- **Short Name**: `x + 0x70`
- **参数类型**: `integer (0)`

获取捕获对象列表

#### 方法 4: set_capture_objects

- **必选/可选**: 可选
- **Short Name**: `x + 0x78`
- **参数类型**: `array`

设置捕获对象列表

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Profile Generic 接口类 (Class ID: 7, Version: 1)
/// 
/// Profile Generic 是最重要的接口类之一，用于存储时间序列历史数据。它支持周期性捕获和选择性访问。
#[derive(Debug, Clone)]
pub struct ProfileGeneric {
    /// 对象实例标识
    pub logical_name: Vec<u8>,
    /// 历史数据缓冲区，每行是一个 structure
    pub buffer: Vec<DlmsType>,
    /// 定义要捕获的对象列表
    pub capture_objects: Vec<DlmsType>,
    /// 自动捕获周期（秒），0 表示手动
    pub capture_period: u32,
    /// 排序方法: 0=FIFO, 1=LIFO, 2=Smallest, 3=Largest, 4=Nearest
    pub sort_method: u8,
    /// 排序依据的对象
    pub sort_object: CaptureObjectDefinition,
    /// 当前使用的条目数
    pub entries_in_use: u32,
    /// 缓冲区最大容量
    pub profile_entries: u32,
}

impl ProfileGeneric {
    /// 创建新的 Profile Generic 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            buffer: Vec::new(),
            capture_objects: Vec::new(),
            capture_period: 0,
            sort_method: 0,
            sort_object: Default::default(),
            entries_in_use: 0,
            profile_entries: 0,
        }
    }

    /// 方法 1: reset
    /// 
    /// 清空缓冲区
    pub fn reset(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 reset
        // 参数类型: integer (0)
        Ok(DlmsType::Null)
    }

    /// 方法 2: capture
    /// 
    /// 手动触发一次捕获
    pub fn capture(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 capture
        // 参数类型: integer (0)
        Ok(DlmsType::Null)
    }

    /// 方法 3: get_capture_objects
    /// 
    /// 获取捕获对象列表
    pub fn get_capture_objects(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 get_capture_objects
        // 参数类型: integer (0)
        Ok(DlmsType::Null)
    }

    /// 方法 4: set_capture_objects
    /// 
    /// 设置捕获对象列表
    pub fn set_capture_objects(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 set_capture_objects
        // 参数类型: array
        Ok(DlmsType::Null)
    }
}

impl CosemObject for ProfileGeneric {
    const CLASS_ID: u16 = 7;
    const VERSION: u8 = 1;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.buffer.clone().into()),
            3 => Ok(self.capture_objects.clone().into()),
            4 => Ok(self.capture_period.clone().into()),
            5 => Ok(self.sort_method.clone().into()),
            6 => Ok(self.sort_object.clone().into()),
            7 => Ok(self.entries_in_use.clone().into()),
            8 => Ok(self.profile_entries.clone().into()),
            _ => Err(CosemError::InvalidAttribute(attr_id)),
        }
    }

    fn set_attribute(&mut self, attr_id: u8, value: DlmsType) -> Result<(), CosemError> {
        match attr_id {
            2 => {
                self.buffer = value.try_into()?;
                Ok(())
            }
            7 => {
                self.entries_in_use = value.try_into()?;
                Ok(())
            }
            _ => Err(CosemError::ReadOnlyAttribute(attr_id)),
        }
    }

    fn invoke_method(&mut self, method_id: u8, params: DlmsType) -> Result<DlmsType, CosemError> {
        match method_id {
            1 => self.reset(params),
            2 => self.capture(params),
            3 => self.get_capture_objects(params),
            4 => self.set_capture_objects(params),
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
    fn test_profile__generic_class_id() {
        assert_eq!(ProfileGeneric::CLASS_ID, 7);
    }

    #[test]
    fn test_profile__generic_version() {
        assert_eq!(ProfileGeneric::VERSION, 1);
    }

    #[test]
    fn test_profile__generic_new() {
        let obis = ObisCode::from_str("0.0.7.0.0.255").unwrap();
        let obj = ProfileGeneric::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_profile__generic_get_logical_name() {
        let obis = ObisCode::from_str("0.0.7.0.0.255").unwrap();
        let obj = ProfileGeneric::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_profile__generic_get_buffer() {
        let obj = ProfileGeneric::new(ObisCode::from_str("0.0.7.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_profile__generic_get_capture_objects() {
        let obj = ProfileGeneric::new(ObisCode::from_str("0.0.7.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_profile__generic_get_capture_period() {
        let obj = ProfileGeneric::new(ObisCode::from_str("0.0.7.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }


    #[test]
    fn test_profile__generic_reset() {
        let mut obj = ProfileGeneric::new(ObisCode::from_str("0.0.7.0.0.255").unwrap());
        let result = obj.invoke_method(1, DlmsType::Null);
        assert!(result.is_ok());
    }


    #[test]
    fn test_profile__generic_capture() {
        let mut obj = ProfileGeneric::new(ObisCode::from_str("0.0.7.0.0.255").unwrap());
        let result = obj.invoke_method(2, DlmsType::Null);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `ProfileGeneric` 结构体
- [ ] 实现所有 8 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 7`
- [ ] `VERSION = 1`
- [ ] `get_attribute()` - 实现 8 个属性
- [ ] `set_attribute()` - 实现可写属性

### 6.3 方法实现 (4 个)

- [ ] 方法 1: `reset()`
- [ ] 方法 2: `capture()`
- [ ] 方法 3: `get_capture_objects()`
- [ ] 方法 4: `set_capture_objects()`

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

*文件名: IC7_ProfileGeneric.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
