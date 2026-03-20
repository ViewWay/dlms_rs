# IEC HDLC Setup 接口类完整实现规范

**Class ID: 23 | Version: 1 | 优先级: 🟢 低**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

IEC HDLC Setup 配置 HDLC 协议参数。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | comm_speed | enum | static | x + 0x08 | 通信速度 |
| 3 | window_size_transmit | unsigned | static | x + 0x10 | 发送窗口大小 |
| 4 | window_size_receive | unsigned | static | x + 0x18 | 接收窗口大小 |
| 5 | max_info_field_length_transmit | long_unsigned | static | x + 0x20 | 最大发送信息字段长度 |
| 6 | max_info_field_length_receive | long_unsigned | static | x + 0x28 | 最大接收信息字段长度 |
| 7 | inter_octet_time_out | long_unsigned | static | x + 0x30 | 字节间超时 |
| 8 | inter_character_time_out | long_unsigned | static | x + 0x38 | 字符间超时 |
| 9 | inactivity_time_out | long_unsigned | static | x + 0x40 | 不活动超时 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: comm_speed

- **类型**: `enum`
- **访问**: static
- **Short Name**: `x + 0x08`

通信速度

#### 属性 3: window_size_transmit

- **类型**: `unsigned`
- **访问**: static
- **Short Name**: `x + 0x10`

发送窗口大小

#### 属性 4: window_size_receive

- **类型**: `unsigned`
- **访问**: static
- **Short Name**: `x + 0x18`

接收窗口大小

#### 属性 5: max_info_field_length_transmit

- **类型**: `long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x20`

最大发送信息字段长度

#### 属性 6: max_info_field_length_receive

- **类型**: `long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x28`

最大接收信息字段长度

#### 属性 7: inter_octet_time_out

- **类型**: `long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x30`

字节间超时

#### 属性 8: inter_character_time_out

- **类型**: `long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x38`

字符间超时

#### 属性 9: inactivity_time_out

- **类型**: `long_unsigned`
- **访问**: static
- **Short Name**: `x + 0x40`

不活动超时

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// IEC HDLC Setup 接口类 (Class ID: 23, Version: 1)
/// 
/// IEC HDLC Setup 配置 HDLC 协议参数。
#[derive(Debug, Clone)]
pub struct IecHdlcSetup {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 通信速度
    pub comm_speed: u8,
    /// 发送窗口大小
    pub window_size_transmit: u8,
    /// 接收窗口大小
    pub window_size_receive: u8,
    /// 最大发送信息字段长度
    pub max_info_field_length_transmit: u16,
    /// 最大接收信息字段长度
    pub max_info_field_length_receive: u16,
    /// 字节间超时
    pub inter_octet_time_out: u16,
    /// 字符间超时
    pub inter_character_time_out: u16,
    /// 不活动超时
    pub inactivity_time_out: u16,
}

impl IecHdlcSetup {
    /// 创建新的 IEC HDLC Setup 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            comm_speed: 0,
            window_size_transmit: 0,
            window_size_receive: 0,
            max_info_field_length_transmit: 0,
            max_info_field_length_receive: 0,
            inter_octet_time_out: 0,
            inter_character_time_out: 0,
            inactivity_time_out: 0,
        }
    }
}

impl CosemObject for IecHdlcSetup {
    const CLASS_ID: u16 = 23;
    const VERSION: u8 = 1;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.comm_speed.clone().into()),
            3 => Ok(self.window_size_transmit.clone().into()),
            4 => Ok(self.window_size_receive.clone().into()),
            5 => Ok(self.max_info_field_length_transmit.clone().into()),
            6 => Ok(self.max_info_field_length_receive.clone().into()),
            7 => Ok(self.inter_octet_time_out.clone().into()),
            8 => Ok(self.inter_character_time_out.clone().into()),
            9 => Ok(self.inactivity_time_out.clone().into()),
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
    fn test_iec_hdlc__setup_class_id() {
        assert_eq!(IecHdlcSetup::CLASS_ID, 23);
    }

    #[test]
    fn test_iec_hdlc__setup_version() {
        assert_eq!(IecHdlcSetup::VERSION, 1);
    }

    #[test]
    fn test_iec_hdlc__setup_new() {
        let obis = ObisCode::from_str("0.0.23.0.0.255").unwrap();
        let obj = IecHdlcSetup::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_iec_hdlc__setup_get_logical_name() {
        let obis = ObisCode::from_str("0.0.23.0.0.255").unwrap();
        let obj = IecHdlcSetup::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_iec_hdlc__setup_get_comm_speed() {
        let obj = IecHdlcSetup::new(ObisCode::from_str("0.0.23.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_iec_hdlc__setup_get_window_size_transmit() {
        let obj = IecHdlcSetup::new(ObisCode::from_str("0.0.23.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_iec_hdlc__setup_get_window_size_receive() {
        let obj = IecHdlcSetup::new(ObisCode::from_str("0.0.23.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `IecHdlcSetup` 结构体
- [ ] 实现所有 9 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 23`
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

*文件名: IC23_IecHdlcSetup.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
