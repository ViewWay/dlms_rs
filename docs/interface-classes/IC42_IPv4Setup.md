# IPv4 Setup 接口类完整实现规范

**Class ID: 42 | Version: 0 | 优先级: 🟢 低**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

IPv4 Setup 配置 IPv4 网络参数。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | ip_address | octet-string | static | x + 0x08 | IPv4 地址 (4字节) |
| 3 | subnet_mask | octet-string | static | x + 0x10 | 子网掩码 |
| 4 | gateway_ip_address | octet-string | static | x + 0x18 | 网关地址 |
| 5 | primary_dns_address | octet-string | static | x + 0x20 | 主 DNS |
| 6 | secondary_dns_address | octet-string | static | x + 0x28 | 备用 DNS |
| 7 | dhcp_enabled | boolean | static | x + 0x30 | 是否启用 DHCP |
| 8 | dhcp_server_ip_address | octet-string | dynamic | x + 0x38 | DHCP 服务器地址 |
| 9 | dhcp_lease_time | long_unsigned | dynamic | x + 0x40 | DHCP 租约时间 |
| 10 | dhcp_lease_time_remaining | long_unsigned | dynamic | x + 0x48 | 剩余租约时间 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: ip_address

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x08`

IPv4 地址 (4字节)

#### 属性 3: subnet_mask

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x10`

子网掩码

#### 属性 4: gateway_ip_address

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x18`

网关地址

#### 属性 5: primary_dns_address

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x20`

主 DNS

#### 属性 6: secondary_dns_address

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x28`

备用 DNS

#### 属性 7: dhcp_enabled

- **类型**: `boolean`
- **访问**: static
- **Short Name**: `x + 0x30`

是否启用 DHCP

#### 属性 8: dhcp_server_ip_address

- **类型**: `octet-string`
- **访问**: dynamic
- **Short Name**: `x + 0x38`

DHCP 服务器地址

#### 属性 9: dhcp_lease_time

- **类型**: `long_unsigned`
- **访问**: dynamic
- **Short Name**: `x + 0x40`

DHCP 租约时间

#### 属性 10: dhcp_lease_time_remaining

- **类型**: `long_unsigned`
- **访问**: dynamic
- **Short Name**: `x + 0x48`

剩余租约时间

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// IPv4 Setup 接口类 (Class ID: 42, Version: 0)
/// 
/// IPv4 Setup 配置 IPv4 网络参数。
#[derive(Debug, Clone)]
pub struct Ipv4Setup {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// IPv4 地址 (4字节)
    pub ip_address: Vec<u8>,
    /// 子网掩码
    pub subnet_mask: Vec<u8>,
    /// 网关地址
    pub gateway_ip_address: Vec<u8>,
    /// 主 DNS
    pub primary_dns_address: Vec<u8>,
    /// 备用 DNS
    pub secondary_dns_address: Vec<u8>,
    /// 是否启用 DHCP
    pub dhcp_enabled: bool,
    /// DHCP 服务器地址
    pub dhcp_server_ip_address: Vec<u8>,
    /// DHCP 租约时间
    pub dhcp_lease_time: u16,
    /// 剩余租约时间
    pub dhcp_lease_time_remaining: u16,
}

impl Ipv4Setup {
    /// 创建新的 IPv4 Setup 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            ip_address: Vec::new(),
            subnet_mask: Vec::new(),
            gateway_ip_address: Vec::new(),
            primary_dns_address: Vec::new(),
            secondary_dns_address: Vec::new(),
            dhcp_enabled: false,
            dhcp_server_ip_address: Vec::new(),
            dhcp_lease_time: 0,
            dhcp_lease_time_remaining: 0,
        }
    }
}

impl CosemObject for Ipv4Setup {
    const CLASS_ID: u16 = 42;
    const VERSION: u8 = 0;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.ip_address.clone().into()),
            3 => Ok(self.subnet_mask.clone().into()),
            4 => Ok(self.gateway_ip_address.clone().into()),
            5 => Ok(self.primary_dns_address.clone().into()),
            6 => Ok(self.secondary_dns_address.clone().into()),
            7 => Ok(self.dhcp_enabled.clone().into()),
            8 => Ok(self.dhcp_server_ip_address.clone().into()),
            9 => Ok(self.dhcp_lease_time.clone().into()),
            10 => Ok(self.dhcp_lease_time_remaining.clone().into()),
            _ => Err(CosemError::InvalidAttribute(attr_id)),
        }
    }

    fn set_attribute(&mut self, attr_id: u8, value: DlmsType) -> Result<(), CosemError> {
        match attr_id {
            8 => {
                self.dhcp_server_ip_address = value.try_into()?;
                Ok(())
            }
            9 => {
                self.dhcp_lease_time = value.try_into()?;
                Ok(())
            }
            10 => {
                self.dhcp_lease_time_remaining = value.try_into()?;
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
    fn test_i_pv4__setup_class_id() {
        assert_eq!(Ipv4Setup::CLASS_ID, 42);
    }

    #[test]
    fn test_i_pv4__setup_version() {
        assert_eq!(Ipv4Setup::VERSION, 0);
    }

    #[test]
    fn test_i_pv4__setup_new() {
        let obis = ObisCode::from_str("0.0.42.0.0.255").unwrap();
        let obj = Ipv4Setup::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_i_pv4__setup_get_logical_name() {
        let obis = ObisCode::from_str("0.0.42.0.0.255").unwrap();
        let obj = Ipv4Setup::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_i_pv4__setup_get_ip_address() {
        let obj = Ipv4Setup::new(ObisCode::from_str("0.0.42.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_i_pv4__setup_get_subnet_mask() {
        let obj = Ipv4Setup::new(ObisCode::from_str("0.0.42.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_i_pv4__setup_get_gateway_ip_address() {
        let obj = Ipv4Setup::new(ObisCode::from_str("0.0.42.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `Ipv4Setup` 结构体
- [ ] 实现所有 10 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 42`
- [ ] `VERSION = 0`
- [ ] `get_attribute()` - 实现 10 个属性
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

*文件名: IC42_Ipv4Setup.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
