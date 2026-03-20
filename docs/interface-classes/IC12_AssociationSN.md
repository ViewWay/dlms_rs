# Association SN 接口类完整实现规范

**Class ID: 12 | Version: 4 | 优先级: 🟢 低**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Association SN 用于短名称（Short Name）寻址的应用关联。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | object_list | object_list_type | static | x + 0x08 | 可见对象列表 |
| 3 | access_rights_list | array | static | x + 0x10 | 访问权限列表 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: object_list

- **类型**: `object_list_type`
- **访问**: static
- **Short Name**: `x + 0x08`

可见对象列表

#### 属性 3: access_rights_list

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x10`

访问权限列表

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Association SN 接口类 (Class ID: 12, Version: 4)
/// 
/// Association SN 用于短名称（Short Name）寻址的应用关联。
#[derive(Debug, Clone)]
pub struct AssociationSn {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 可见对象列表
    pub object_list: Vec<ObjectListElement>,
    /// 访问权限列表
    pub access_rights_list: Vec<DlmsType>,
}

impl AssociationSn {
    /// 创建新的 Association SN 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            object_list: Default::default(),
            access_rights_list: Vec::new(),
        }
    }
}

impl CosemObject for AssociationSn {
    const CLASS_ID: u16 = 12;
    const VERSION: u8 = 4;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.object_list.clone().into()),
            3 => Ok(self.access_rights_list.clone().into()),
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
    fn test_association_sn_class_id() {
        assert_eq!(AssociationSn::CLASS_ID, 12);
    }

    #[test]
    fn test_association_sn_version() {
        assert_eq!(AssociationSn::VERSION, 4);
    }

    #[test]
    fn test_association_sn_new() {
        let obis = ObisCode::from_str("0.0.12.0.0.255").unwrap();
        let obj = AssociationSn::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_association_sn_get_logical_name() {
        let obis = ObisCode::from_str("0.0.12.0.0.255").unwrap();
        let obj = AssociationSn::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_association_sn_get_object_list() {
        let obj = AssociationSn::new(ObisCode::from_str("0.0.12.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_association_sn_get_access_rights_list() {
        let obj = AssociationSn::new(ObisCode::from_str("0.0.12.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `AssociationSn` 结构体
- [ ] 实现所有 3 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 12`
- [ ] `VERSION = 4`
- [ ] `get_attribute()` - 实现 3 个属性
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

*文件名: IC12_AssociationSn.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
