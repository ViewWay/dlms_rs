# IPv6 Setup 接口类完整实现规范

**Class ID: 48 | Version: 0 | 优先级: 🟢 低**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

IPv6 Setup 配置 IPv6 网络参数。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | address_configuration_mode | enum | static | x + 0x08 | 地址配置模式 |
| 3 | link_local_address | octet-string | static | x + 0x10 | 链路本地地址 (16字节) |
| 4 | address_list | array | dynamic | x + 0x18 | IPv6 地址列表 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: address_configuration_mode

- **类型**: `enum`
- **访问**: static
- **Short Name**: `x + 0x08`

地址配置模式

#### 属性 3: link_local_address

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x10`

链路本地地址 (16字节)

#### 属性 4: address_list

- **类型**: `array`
- **访问**: dynamic
- **Short Name**: `x + 0x18`

IPv6 地址列表

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// IPv6 Setup 接口类 (Class ID: 48, Version: 0)
/// 
/// IPv6 Setup 配置 IPv6 网络参数。
#[derive(Debug, Clone)]
pub struct Ipv6Setup {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 地址配置模式
    pub address_configuration_mode: u8,
    /// 链路本地地址 (16字节)
    pub link_local_address: Vec<u8>,
    /// IPv6 地址列表
    pub address_list: Vec<DlmsType>,
}

impl Ipv6Setup {
    /// 创建新的 IPv6 Setup 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            address_configuration_mode: 0,
            link_local_address: Vec::new(),
            address_list: Vec::new(),
        }
    }
}

impl CosemObject for Ipv6Setup {
    const CLASS_ID: u16 = 48;
    const VERSION: u8 = 0;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.address_configuration_mode.clone().into()),
            3 => Ok(self.link_local_address.clone().into()),
            4 => Ok(self.address_list.clone().into()),
            _ => Err(CosemError::InvalidAttribute(attr_id)),
        }
    }

    fn set_attribute(&mut self, attr_id: u8, value: DlmsType) -> Result<(), CosemError> {
        match attr_id {
            4 => {
                self.address_list = value.try_into()?;
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
    fn test_i_pv6__setup_class_id() {
        assert_eq!(Ipv6Setup::CLASS_ID, 48);
    }

    #[test]
    fn test_i_pv6__setup_version() {
        assert_eq!(Ipv6Setup::VERSION, 0);
    }

    #[test]
    fn test_i_pv6__setup_new() {
        let obis = ObisCode::from_str("0.0.48.0.0.255").unwrap();
        let obj = Ipv6Setup::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_i_pv6__setup_get_logical_name() {
        let obis = ObisCode::from_str("0.0.48.0.0.255").unwrap();
        let obj = Ipv6Setup::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_i_pv6__setup_get_address_configuration_mode() {
        let obj = Ipv6Setup::new(ObisCode::from_str("0.0.48.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_i_pv6__setup_get_link_local_address() {
        let obj = Ipv6Setup::new(ObisCode::from_str("0.0.48.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_i_pv6__setup_get_address_list() {
        let obj = Ipv6Setup::new(ObisCode::from_str("0.0.48.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `Ipv6Setup` 结构体
- [ ] 实现所有 4 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 48`
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

*文件名: IC48_Ipv6Setup.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
