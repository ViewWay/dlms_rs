# Security Setup 接口类完整实现规范

**Class ID: 64 | Version: 1 | 优先级: 🔴 高**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Security Setup 配置安全参数，是安全通信的核心。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | security_policy | bit_string | static | x + 0x08 | 安全策略位图 |
| 3 | security_suite | enum | static | x + 0x10 | 安全套件: 0=无, 1=AES-GCM-128 |
| 4 | server_system_title | octet-string | static | x + 0x18 | 服务器系统标题 (8字节) |
| 5 | certificates | array | static | x + 0x20 | 证书列表 |
| 6 | global_unicast_encryption_key | octet-string | static | x + 0x28 | 全局单播加密密钥 (16字节) |
| 7 | global_broadcast_encryption_key | octet-string | static | x + 0x30 | 全局广播加密密钥 (16字节) |
| 8 | authentication_key | octet-string | static | x + 0x38 | 认证密钥 (16字节) |
| 9 | security_activate | enum | static | x + 0x40 | 安全激活状态 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: security_policy

- **类型**: `bit_string`
- **访问**: static
- **Short Name**: `x + 0x08`

安全策略位图

#### 属性 3: security_suite

- **类型**: `enum`
- **访问**: static
- **Short Name**: `x + 0x10`

安全套件: 0=无, 1=AES-GCM-128

#### 属性 4: server_system_title

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x18`

服务器系统标题 (8字节)

#### 属性 5: certificates

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x20`

证书列表

#### 属性 6: global_unicast_encryption_key

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x28`

全局单播加密密钥 (16字节)

#### 属性 7: global_broadcast_encryption_key

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x30`

全局广播加密密钥 (16字节)

#### 属性 8: authentication_key

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x38`

认证密钥 (16字节)

#### 属性 9: security_activate

- **类型**: `enum`
- **访问**: static
- **Short Name**: `x + 0x40`

安全激活状态

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Security Setup 接口类 (Class ID: 64, Version: 1)
/// 
/// Security Setup 配置安全参数，是安全通信的核心。
#[derive(Debug, Clone)]
pub struct SecuritySetup {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 安全策略位图
    pub security_policy: Vec<u8>,
    /// 安全套件: 0=无, 1=AES-GCM-128
    pub security_suite: u8,
    /// 服务器系统标题 (8字节)
    pub server_system_title: Vec<u8>,
    /// 证书列表
    pub certificates: Vec<DlmsType>,
    /// 全局单播加密密钥 (16字节)
    pub global_unicast_encryption_key: Vec<u8>,
    /// 全局广播加密密钥 (16字节)
    pub global_broadcast_encryption_key: Vec<u8>,
    /// 认证密钥 (16字节)
    pub authentication_key: Vec<u8>,
    /// 安全激活状态
    pub security_activate: u8,
}

impl SecuritySetup {
    /// 创建新的 Security Setup 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            security_policy: Vec::new(),
            security_suite: 0,
            server_system_title: Vec::new(),
            certificates: Vec::new(),
            global_unicast_encryption_key: Vec::new(),
            global_broadcast_encryption_key: Vec::new(),
            authentication_key: Vec::new(),
            security_activate: 0,
        }
    }
}

impl CosemObject for SecuritySetup {
    const CLASS_ID: u16 = 64;
    const VERSION: u8 = 1;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.security_policy.clone().into()),
            3 => Ok(self.security_suite.clone().into()),
            4 => Ok(self.server_system_title.clone().into()),
            5 => Ok(self.certificates.clone().into()),
            6 => Ok(self.global_unicast_encryption_key.clone().into()),
            7 => Ok(self.global_broadcast_encryption_key.clone().into()),
            8 => Ok(self.authentication_key.clone().into()),
            9 => Ok(self.security_activate.clone().into()),
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
    fn test_security__setup_class_id() {
        assert_eq!(SecuritySetup::CLASS_ID, 64);
    }

    #[test]
    fn test_security__setup_version() {
        assert_eq!(SecuritySetup::VERSION, 1);
    }

    #[test]
    fn test_security__setup_new() {
        let obis = ObisCode::from_str("0.0.64.0.0.255").unwrap();
        let obj = SecuritySetup::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_security__setup_get_logical_name() {
        let obis = ObisCode::from_str("0.0.64.0.0.255").unwrap();
        let obj = SecuritySetup::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_security__setup_get_security_policy() {
        let obj = SecuritySetup::new(ObisCode::from_str("0.0.64.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_security__setup_get_security_suite() {
        let obj = SecuritySetup::new(ObisCode::from_str("0.0.64.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_security__setup_get_server_system_title() {
        let obj = SecuritySetup::new(ObisCode::from_str("0.0.64.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `SecuritySetup` 结构体
- [ ] 实现所有 9 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 64`
- [ ] `VERSION = 1`
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

*文件名: IC64_SecuritySetup.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
