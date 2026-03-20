# TCP UDP Setup 接口类完整实现规范

**Class ID: 41 | Version: 0 | 优先级: 🟢 低**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

TCP-UDP Setup 配置 TCP/UDP 传输参数。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | ip_reference | octet-string | static | x + 0x08 | IP 配置引用 |
| 3 | port | long_unsigned | static | x + 0x10 | 端口号 |
| 4 | ip_config_type | enum | static | x + 0x18 | IP 配置类型 |
| 5 | ip_address | octet-string | static | x + 0x20 | IP 地址 |
| 6 | subnet_mask | octet-string | static | x + 0x28 | 子网掩码 |
| 7 | gateway_ip_address | octet-string | static | x + 0x30 | 网关地址 |
| 8 | dns_ip_address | octet-string | static | x + 0x38 | DNS 地址 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: ip_reference

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x08`

IP 配置引用

#### 属性 3: port

- **类型**: `long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x10`

端口号

#### 属性 4: ip_config_type

- **类型**: `enum`
- **访问**: static
- **Short Name**: `x + 0x18`

IP 配置类型

#### 属性 5: ip_address

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x20`

IP 地址

#### 属性 6: subnet_mask

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x28`

子网掩码

#### 属性 7: gateway_ip_address

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x30`

网关地址

#### 属性 8: dns_ip_address

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x38`

DNS 地址

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// TCP UDP Setup 接口类 (Class ID: 41, Version: 0)
/// 
/// TCP-UDP Setup 配置 TCP/UDP 传输参数。
#[derive(Debug, Clone)]
pub struct TcpUdpSetup {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// IP 配置引用
    pub ip_reference: Vec<u8>,
    /// 端口号
    pub port: u16,
    /// IP 配置类型
    pub ip_config_type: u8,
    /// IP 地址
    pub ip_address: Vec<u8>,
    /// 子网掩码
    pub subnet_mask: Vec<u8>,
    /// 网关地址
    pub gateway_ip_address: Vec<u8>,
    /// DNS 地址
    pub dns_ip_address: Vec<u8>,
}

impl TcpUdpSetup {
    /// 创建新的 TCP UDP Setup 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            ip_reference: Vec::new(),
            port: 0,
            ip_config_type: 0,
            ip_address: Vec::new(),
            subnet_mask: Vec::new(),
            gateway_ip_address: Vec::new(),
            dns_ip_address: Vec::new(),
        }
    }
}

impl CosemObject for TcpUdpSetup {
    const CLASS_ID: u16 = 41;
    const VERSION: u8 = 0;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.ip_reference.clone().into()),
            3 => Ok(self.port.clone().into()),
            4 => Ok(self.ip_config_type.clone().into()),
            5 => Ok(self.ip_address.clone().into()),
            6 => Ok(self.subnet_mask.clone().into()),
            7 => Ok(self.gateway_ip_address.clone().into()),
            8 => Ok(self.dns_ip_address.clone().into()),
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
    fn test_tcp_udp__setup_class_id() {
        assert_eq!(TcpUdpSetup::CLASS_ID, 41);
    }

    #[test]
    fn test_tcp_udp__setup_version() {
        assert_eq!(TcpUdpSetup::VERSION, 0);
    }

    #[test]
    fn test_tcp_udp__setup_new() {
        let obis = ObisCode::from_str("0.0.41.0.0.255").unwrap();
        let obj = TcpUdpSetup::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_tcp_udp__setup_get_logical_name() {
        let obis = ObisCode::from_str("0.0.41.0.0.255").unwrap();
        let obj = TcpUdpSetup::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_tcp_udp__setup_get_ip_reference() {
        let obj = TcpUdpSetup::new(ObisCode::from_str("0.0.41.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_tcp_udp__setup_get_port() {
        let obj = TcpUdpSetup::new(ObisCode::from_str("0.0.41.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_tcp_udp__setup_get_ip_config_type() {
        let obj = TcpUdpSetup::new(ObisCode::from_str("0.0.41.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `TcpUdpSetup` 结构体
- [ ] 实现所有 8 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 41`
- [ ] `VERSION = 0`
- [ ] `get_attribute()` - 实现 8 个属性
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

*文件名: IC41_TcpUdpSetup.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
