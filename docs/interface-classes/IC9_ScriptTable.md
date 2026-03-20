# Script Table 接口类完整实现规范

**Class ID: 9 | Version: 0 | 优先级: 🟡 中**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Script Table 存储脚本定义，每个脚本包含一系列动作。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | scripts | array | static | x + 0x08 | 脚本列表，每个脚本是 (script_id, actions) 的结构 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: scripts

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x08`

脚本列表，每个脚本是 (script_id, actions) 的结构

---

## 3. 方法定义

| 方法 ID | 名称 | 必选/可选 | Short Name | 参数类型 | 说明 |
|---------|------|----------|------------|----------|------|
| 1 | execute | 必选 | x + 0x60 | long_unsigned | 执行指定 ID 的脚本 |


### 3.1 方法详细说明

#### 方法 1: execute

- **必选/可选**: 必选
- **Short Name**: `x + 0x60`
- **参数类型**: `long_unsigned`

执行指定 ID 的脚本

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Script Table 接口类 (Class ID: 9, Version: 0)
/// 
/// Script Table 存储脚本定义，每个脚本包含一系列动作。
#[derive(Debug, Clone)]
pub struct ScriptTable {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 脚本列表，每个脚本是 (script_id, actions) 的结构
    pub scripts: Vec<DlmsType>,
}

impl ScriptTable {
    /// 创建新的 Script Table 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            scripts: Vec::new(),
        }
    }

    /// 方法 1: execute
    /// 
    /// 执行指定 ID 的脚本
    pub fn execute(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 execute
        // 参数类型: long_unsigned
        Ok(DlmsType::Null)
    }
}

impl CosemObject for ScriptTable {
    const CLASS_ID: u16 = 9;
    const VERSION: u8 = 0;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.scripts.clone().into()),
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
            1 => self.execute(params),
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
    fn test_script__table_class_id() {
        assert_eq!(ScriptTable::CLASS_ID, 9);
    }

    #[test]
    fn test_script__table_version() {
        assert_eq!(ScriptTable::VERSION, 0);
    }

    #[test]
    fn test_script__table_new() {
        let obis = ObisCode::from_str("0.0.9.0.0.255").unwrap();
        let obj = ScriptTable::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_script__table_get_logical_name() {
        let obis = ObisCode::from_str("0.0.9.0.0.255").unwrap();
        let obj = ScriptTable::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_script__table_get_scripts() {
        let obj = ScriptTable::new(ObisCode::from_str("0.0.9.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_script__table_execute() {
        let mut obj = ScriptTable::new(ObisCode::from_str("0.0.9.0.0.255").unwrap());
        let result = obj.invoke_method(1, DlmsType::Null);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `ScriptTable` 结构体
- [ ] 实现所有 2 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 9`
- [ ] `VERSION = 0`
- [ ] `get_attribute()` - 实现 2 个属性
- [ ] `set_attribute()` - 实现可写属性

### 6.3 方法实现 (1 个)

- [ ] 方法 1: `execute()`

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

*文件名: IC9_ScriptTable.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
