# Association LN 接口类完整实现规范

**Class ID: 15 | Version: 3 | 优先级: 🔴 高**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Association LN 是最重要的关联类，用于逻辑名称（Logical Name）寻址的应用关联。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | object_list | object_list_type | static | x + 0x08 | 可见对象列表及访问权限 |
| 3 | associated_partners_id | structure | static | x + 0x10 | (client_SAP, server_SAP) |
| 4 | application_context_name | context_name_type | static | x + 0x18 | 应用上下文名称 (OID) |
| 5 | xdlms_context_info | structure | static | x + 0x20 | xDLMS 上下文信息 |
| 6 | authentication_mechanism_name | mechanism_name_type | static | x + 0x28 | 认证机制名称 |
| 7 | secret | octet-string | static | x + 0x30 | 认证密钥/密码 |
| 8 | association_status | enum | dynamic | x + 0x38 | 0=NonAssociated, 1=InProgress, 2=Associated, 3=Terminated |
| 9 | security_setup_reference | octet-string | static | x + 0x40 | Security Setup 对象引用 |
| 10 | user_list | array | static | x + 0x48 | 用户列表 |
| 11 | current_user | structure | dynamic | x + 0x50 | 当前用户信息 |


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

可见对象列表及访问权限

#### 属性 3: associated_partners_id

- **类型**: `structure`
- **访问**: static
- **Short Name**: `x + 0x10`

(client_SAP, server_SAP)

#### 属性 4: application_context_name

- **类型**: `context_name_type`
- **访问**: static
- **Short Name**: `x + 0x18`

应用上下文名称 (OID)

#### 属性 5: xdlms_context_info

- **类型**: `structure`
- **访问**: static
- **Short Name**: `x + 0x20`

xDLMS 上下文信息

#### 属性 6: authentication_mechanism_name

- **类型**: `mechanism_name_type`
- **访问**: static
- **Short Name**: `x + 0x28`

认证机制名称

#### 属性 7: secret

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x30`

认证密钥/密码

#### 属性 8: association_status

- **类型**: `enum`
- **访问**: dynamic
- **Short Name**: `x + 0x38`

0=NonAssociated, 1=InProgress, 2=Associated, 3=Terminated

#### 属性 9: security_setup_reference

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x40`

Security Setup 对象引用

#### 属性 10: user_list

- **类型**: `array`
- **访问**: static
- **Short Name**: `x + 0x48`

用户列表

#### 属性 11: current_user

- **类型**: `structure`
- **访问**: dynamic
- **Short Name**: `x + 0x50`

当前用户信息

---

## 3. 方法定义

| 方法 ID | 名称 | 必选/可选 | Short Name | 参数类型 | 说明 |
|---------|------|----------|------------|----------|------|
| 1 | reply_to_HLS_authentication | 可选 | x + 0x60 | octet-string | HLS 认证响应 |
| 2 | change_HLS_secret | 可选 | x + 0x68 | structure | 更改 HLS 密钥 |
| 3 | add_object | 可选 | x + 0x70 | object_list_element | 添加对象到关联 |
| 4 | remove_object | 可选 | x + 0x78 | object_id | 从关联移除对象 |
| 5 | add_user | 可选 | x + 0x80 | user_list_entry | 添加用户 |
| 6 | remove_user | 可选 | x + 0x88 | unsigned | 移除用户 |


### 3.1 方法详细说明

#### 方法 1: reply_to_HLS_authentication

- **必选/可选**: 可选
- **Short Name**: `x + 0x60`
- **参数类型**: `octet-string`

HLS 认证响应

#### 方法 2: change_HLS_secret

- **必选/可选**: 可选
- **Short Name**: `x + 0x68`
- **参数类型**: `structure`

更改 HLS 密钥

#### 方法 3: add_object

- **必选/可选**: 可选
- **Short Name**: `x + 0x70`
- **参数类型**: `object_list_element`

添加对象到关联

#### 方法 4: remove_object

- **必选/可选**: 可选
- **Short Name**: `x + 0x78`
- **参数类型**: `object_id`

从关联移除对象

#### 方法 5: add_user

- **必选/可选**: 可选
- **Short Name**: `x + 0x80`
- **参数类型**: `user_list_entry`

添加用户

#### 方法 6: remove_user

- **必选/可选**: 可选
- **Short Name**: `x + 0x88`
- **参数类型**: `unsigned`

移除用户

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Association LN 接口类 (Class ID: 15, Version: 3)
/// 
/// Association LN 是最重要的关联类，用于逻辑名称（Logical Name）寻址的应用关联。
#[derive(Debug, Clone)]
pub struct AssociationLn {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 可见对象列表及访问权限
    pub object_list: Vec<ObjectListElement>,
    /// (client_SAP, server_SAP)
    pub associated_partners_id: Vec<DlmsType>,
    /// 应用上下文名称 (OID)
    pub application_context_name: ApplicationContextName,
    /// xDLMS 上下文信息
    pub xdlms_context_info: Vec<DlmsType>,
    /// 认证机制名称
    pub authentication_mechanism_name: AuthenticationMechanismName,
    /// 认证密钥/密码
    pub secret: Vec<u8>,
    /// 0=NonAssociated, 1=InProgress, 2=Associated, 3=Terminated
    pub association_status: u8,
    /// Security Setup 对象引用
    pub security_setup_reference: Vec<u8>,
    /// 用户列表
    pub user_list: Vec<DlmsType>,
    /// 当前用户信息
    pub current_user: Vec<DlmsType>,
}

impl AssociationLn {
    /// 创建新的 Association LN 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            object_list: Default::default(),
            associated_partners_id: Vec::new(),
            application_context_name: Default::default(),
            xdlms_context_info: Vec::new(),
            authentication_mechanism_name: Default::default(),
            secret: Vec::new(),
            association_status: 0,
            security_setup_reference: Vec::new(),
            user_list: Vec::new(),
            current_user: Vec::new(),
        }
    }

    /// 方法 1: reply_to_HLS_authentication
    /// 
    /// HLS 认证响应
    pub fn reply_to_hls_authentication(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 reply_to_HLS_authentication
        // 参数类型: octet-string
        Ok(DlmsType::Null)
    }

    /// 方法 2: change_HLS_secret
    /// 
    /// 更改 HLS 密钥
    pub fn change_hls_secret(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 change_HLS_secret
        // 参数类型: structure
        Ok(DlmsType::Null)
    }

    /// 方法 3: add_object
    /// 
    /// 添加对象到关联
    pub fn add_object(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 add_object
        // 参数类型: object_list_element
        Ok(DlmsType::Null)
    }

    /// 方法 4: remove_object
    /// 
    /// 从关联移除对象
    pub fn remove_object(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 remove_object
        // 参数类型: object_id
        Ok(DlmsType::Null)
    }

    /// 方法 5: add_user
    /// 
    /// 添加用户
    pub fn add_user(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 add_user
        // 参数类型: user_list_entry
        Ok(DlmsType::Null)
    }

    /// 方法 6: remove_user
    /// 
    /// 移除用户
    pub fn remove_user(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 remove_user
        // 参数类型: unsigned
        Ok(DlmsType::Null)
    }
}

impl CosemObject for AssociationLn {
    const CLASS_ID: u16 = 15;
    const VERSION: u8 = 3;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.object_list.clone().into()),
            3 => Ok(self.associated_partners_id.clone().into()),
            4 => Ok(self.application_context_name.clone().into()),
            5 => Ok(self.xdlms_context_info.clone().into()),
            6 => Ok(self.authentication_mechanism_name.clone().into()),
            7 => Ok(self.secret.clone().into()),
            8 => Ok(self.association_status.clone().into()),
            9 => Ok(self.security_setup_reference.clone().into()),
            10 => Ok(self.user_list.clone().into()),
            11 => Ok(self.current_user.clone().into()),
            _ => Err(CosemError::InvalidAttribute(attr_id)),
        }
    }

    fn set_attribute(&mut self, attr_id: u8, value: DlmsType) -> Result<(), CosemError> {
        match attr_id {
            7 => {
                self.secret = value.try_into()?;
                Ok(())
            }
            8 => {
                self.association_status = value.try_into()?;
                Ok(())
            }
            11 => {
                self.current_user = value.try_into()?;
                Ok(())
            }
            _ => Err(CosemError::ReadOnlyAttribute(attr_id)),
        }
    }

    fn invoke_method(&mut self, method_id: u8, params: DlmsType) -> Result<DlmsType, CosemError> {
        match method_id {
            1 => self.reply_to_hls_authentication(params),
            2 => self.change_hls_secret(params),
            3 => self.add_object(params),
            4 => self.remove_object(params),
            5 => self.add_user(params),
            6 => self.remove_user(params),
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
    fn test_association_ln_class_id() {
        assert_eq!(AssociationLn::CLASS_ID, 15);
    }

    #[test]
    fn test_association_ln_version() {
        assert_eq!(AssociationLn::VERSION, 3);
    }

    #[test]
    fn test_association_ln_new() {
        let obis = ObisCode::from_str("0.0.15.0.0.255").unwrap();
        let obj = AssociationLn::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_association_ln_get_logical_name() {
        let obis = ObisCode::from_str("0.0.15.0.0.255").unwrap();
        let obj = AssociationLn::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_association_ln_get_object_list() {
        let obj = AssociationLn::new(ObisCode::from_str("0.0.15.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_association_ln_get_associated_partners_id() {
        let obj = AssociationLn::new(ObisCode::from_str("0.0.15.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_association_ln_get_application_context_name() {
        let obj = AssociationLn::new(ObisCode::from_str("0.0.15.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }


    #[test]
    fn test_association_ln_reply_to_hls_authentication() {
        let mut obj = AssociationLn::new(ObisCode::from_str("0.0.15.0.0.255").unwrap());
        let result = obj.invoke_method(1, DlmsType::Null);
        assert!(result.is_ok());
    }


    #[test]
    fn test_association_ln_change_hls_secret() {
        let mut obj = AssociationLn::new(ObisCode::from_str("0.0.15.0.0.255").unwrap());
        let result = obj.invoke_method(2, DlmsType::Null);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `AssociationLn` 结构体
- [ ] 实现所有 11 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 15`
- [ ] `VERSION = 3`
- [ ] `get_attribute()` - 实现 11 个属性
- [ ] `set_attribute()` - 实现可写属性

### 6.3 方法实现 (6 个)

- [ ] 方法 1: `reply_to_HLS_authentication()`
- [ ] 方法 2: `change_HLS_secret()`
- [ ] 方法 3: `add_object()`
- [ ] 方法 4: `remove_object()`
- [ ] 方法 5: `add_user()`
- [ ] 方法 6: `remove_user()`

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

*文件名: IC15_AssociationLn.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
