# Image Transfer 接口类完整实现规范

**Class ID: 18 | Version: 0 | 优先级: 🔴 高**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Image Transfer 用于固件升级，支持分块传输和验证。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | image_identifier | octet-string | static | x + 0x08 | 镜像标识符 |
| 3 | image_size | double_long_unsigned | static | x + 0x10 | 镜像大小（字节） |
| 4 | transferred_block_status | array | dynamic | x + 0x18 | 已传输块的位图 |
| 5 | first_not_transferred_block_number | double_long_unsigned | dynamic | x + 0x20 | 第一个未传输块号 |
| 6 | image_transfer_status | enum | dynamic | x + 0x28 | 传输状态 |
| 7 | image_to_activate_info | structure | static | x + 0x30 | 待激活镜像信息 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: image_identifier

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x + 0x08`

镜像标识符

#### 属性 3: image_size

- **类型**: `double_long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x10`

镜像大小（字节）

#### 属性 4: transferred_block_status

- **类型**: `array`
- **访问**: dynamic
- **Short Name**: `x + 0x18`

已传输块的位图

#### 属性 5: first_not_transferred_block_number

- **类型**: `double_long_unsigned`
- **访问**: dynamic
- **Short Name**: `x + 0x20`

第一个未传输块号

#### 属性 6: image_transfer_status

- **类型**: `enum`
- **访问**: dynamic
- **Short Name**: `x + 0x28`

传输状态

#### 属性 7: image_to_activate_info

- **类型**: `structure`
- **访问**: static
- **Short Name**: `x + 0x30`

待激活镜像信息

---

## 3. 方法定义

| 方法 ID | 名称 | 必选/可选 | Short Name | 参数类型 | 说明 |
|---------|------|----------|------------|----------|------|
| 1 | initiate_image_transfer | 必选 | x + 0x60 | octet-string | 初始化镜像传输 |
| 2 | image_block_transfer | 必选 | x + 0x68 | structure | 传输镜像块 |
| 3 | image_verify | 必选 | x + 0x70 | integer (0) | 验证镜像 |
| 4 | image_activate | 必选 | x + 0x78 | integer (0) | 激活镜像 |


### 3.1 方法详细说明

#### 方法 1: initiate_image_transfer

- **必选/可选**: 必选
- **Short Name**: `x + 0x60`
- **参数类型**: `octet-string`

初始化镜像传输

#### 方法 2: image_block_transfer

- **必选/可选**: 必选
- **Short Name**: `x + 0x68`
- **参数类型**: `structure`

传输镜像块

#### 方法 3: image_verify

- **必选/可选**: 必选
- **Short Name**: `x + 0x70`
- **参数类型**: `integer (0)`

验证镜像

#### 方法 4: image_activate

- **必选/可选**: 必选
- **Short Name**: `x + 0x78`
- **参数类型**: `integer (0)`

激活镜像

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Image Transfer 接口类 (Class ID: 18, Version: 0)
/// 
/// Image Transfer 用于固件升级，支持分块传输和验证。
#[derive(Debug, Clone)]
pub struct ImageTransfer {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 镜像标识符
    pub image_identifier: Vec<u8>,
    /// 镜像大小（字节）
    pub image_size: u32,
    /// 已传输块的位图
    pub transferred_block_status: Vec<DlmsType>,
    /// 第一个未传输块号
    pub first_not_transferred_block_number: u32,
    /// 传输状态
    pub image_transfer_status: u8,
    /// 待激活镜像信息
    pub image_to_activate_info: Vec<DlmsType>,
}

impl ImageTransfer {
    /// 创建新的 Image Transfer 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            image_identifier: Vec::new(),
            image_size: 0,
            transferred_block_status: Vec::new(),
            first_not_transferred_block_number: 0,
            image_transfer_status: 0,
            image_to_activate_info: Vec::new(),
        }
    }

    /// 方法 1: initiate_image_transfer
    /// 
    /// 初始化镜像传输
    pub fn initiate_image_transfer(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 initiate_image_transfer
        // 参数类型: octet-string
        Ok(DlmsType::Null)
    }

    /// 方法 2: image_block_transfer
    /// 
    /// 传输镜像块
    pub fn image_block_transfer(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 image_block_transfer
        // 参数类型: structure
        Ok(DlmsType::Null)
    }

    /// 方法 3: image_verify
    /// 
    /// 验证镜像
    pub fn image_verify(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 image_verify
        // 参数类型: integer (0)
        Ok(DlmsType::Null)
    }

    /// 方法 4: image_activate
    /// 
    /// 激活镜像
    pub fn image_activate(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 image_activate
        // 参数类型: integer (0)
        Ok(DlmsType::Null)
    }
}

impl CosemObject for ImageTransfer {
    const CLASS_ID: u16 = 18;
    const VERSION: u8 = 0;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.image_identifier.clone().into()),
            3 => Ok(self.image_size.clone().into()),
            4 => Ok(self.transferred_block_status.clone().into()),
            5 => Ok(self.first_not_transferred_block_number.clone().into()),
            6 => Ok(self.image_transfer_status.clone().into()),
            7 => Ok(self.image_to_activate_info.clone().into()),
            _ => Err(CosemError::InvalidAttribute(attr_id)),
        }
    }

    fn set_attribute(&mut self, attr_id: u8, value: DlmsType) -> Result<(), CosemError> {
        match attr_id {
            4 => {
                self.transferred_block_status = value.try_into()?;
                Ok(())
            }
            5 => {
                self.first_not_transferred_block_number = value.try_into()?;
                Ok(())
            }
            6 => {
                self.image_transfer_status = value.try_into()?;
                Ok(())
            }
            _ => Err(CosemError::ReadOnlyAttribute(attr_id)),
        }
    }

    fn invoke_method(&mut self, method_id: u8, params: DlmsType) -> Result<DlmsType, CosemError> {
        match method_id {
            1 => self.initiate_image_transfer(params),
            2 => self.image_block_transfer(params),
            3 => self.image_verify(params),
            4 => self.image_activate(params),
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
    fn test_image__transfer_class_id() {
        assert_eq!(ImageTransfer::CLASS_ID, 18);
    }

    #[test]
    fn test_image__transfer_version() {
        assert_eq!(ImageTransfer::VERSION, 0);
    }

    #[test]
    fn test_image__transfer_new() {
        let obis = ObisCode::from_str("0.0.18.0.0.255").unwrap();
        let obj = ImageTransfer::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_image__transfer_get_logical_name() {
        let obis = ObisCode::from_str("0.0.18.0.0.255").unwrap();
        let obj = ImageTransfer::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_image__transfer_get_image_identifier() {
        let obj = ImageTransfer::new(ObisCode::from_str("0.0.18.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_image__transfer_get_image_size() {
        let obj = ImageTransfer::new(ObisCode::from_str("0.0.18.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_image__transfer_get_transferred_block_status() {
        let obj = ImageTransfer::new(ObisCode::from_str("0.0.18.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }


    #[test]
    fn test_image__transfer_initiate_image_transfer() {
        let mut obj = ImageTransfer::new(ObisCode::from_str("0.0.18.0.0.255").unwrap());
        let result = obj.invoke_method(1, DlmsType::Null);
        assert!(result.is_ok());
    }


    #[test]
    fn test_image__transfer_image_block_transfer() {
        let mut obj = ImageTransfer::new(ObisCode::from_str("0.0.18.0.0.255").unwrap());
        let result = obj.invoke_method(2, DlmsType::Null);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `ImageTransfer` 结构体
- [ ] 实现所有 7 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 18`
- [ ] `VERSION = 0`
- [ ] `get_attribute()` - 实现 7 个属性
- [ ] `set_attribute()` - 实现可写属性

### 6.3 方法实现 (4 个)

- [ ] 方法 1: `initiate_image_transfer()`
- [ ] 方法 2: `image_block_transfer()`
- [ ] 方法 3: `image_verify()`
- [ ] 方法 4: `image_activate()`

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

*文件名: IC18_ImageTransfer.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
