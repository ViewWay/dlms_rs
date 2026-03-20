# Disconnect Control 接口类完整实现规范

**Class ID: 70 | Version: 1 | 优先级: 🟡 中**

> 基于 Blue Book Edition 16 Part 2

---

## 1. 概述

Disconnect Control 控制负载开关。

---

## 2. 属性定义

| 属性 ID | 名称 | 类型 | 访问 | Short Name | 说明 |
|---------|------|------|------|------------|------|
| 1 | logical_name | octet-string | static | x | - |
| 2 | output_state | boolean | dynamic | x + 0x08 | 输出状态: true=连接, false=断开 |
| 3 | control_state | enum | dynamic | x + 0x10 | 控制状态: 0=disconnected, 1=connected, 2=ready_for_reconnection |
| 4 | control_mode | enum | static | x + 0x18 | 控制模式 |


### 2.1 属性详细说明

#### 属性 1: logical_name

- **类型**: `octet-string`
- **访问**: static
- **Short Name**: `x`

待补充

#### 属性 2: output_state

- **类型**: `boolean`
- **访问**: dynamic
- **Short Name**: `x + 0x08`

输出状态: true=连接, false=断开

#### 属性 3: control_state

- **类型**: `enum`
- **访问**: dynamic
- **Short Name**: `x + 0x10`

控制状态: 0=disconnected, 1=connected, 2=ready_for_reconnection

#### 属性 4: control_mode

- **类型**: `enum`
- **访问**: static
- **Short Name**: `x + 0x18`

控制模式

---

## 3. 方法定义

| 方法 ID | 名称 | 必选/可选 | Short Name | 参数类型 | 说明 |
|---------|------|----------|------------|----------|------|
| 1 | remote_disconnect | 必选 | x + 0x60 | integer (0) | 远程断开 |
| 2 | remote_reconnect | 必选 | x + 0x68 | integer (0) | 远程重连 |


### 3.1 方法详细说明

#### 方法 1: remote_disconnect

- **必选/可选**: 必选
- **Short Name**: `x + 0x60`
- **参数类型**: `integer (0)`

远程断开

#### 方法 2: remote_reconnect

- **必选/可选**: 必选
- **Short Name**: `x + 0x68`
- **参数类型**: `integer (0)`

远程重连

---

## 4. Rust 完整实现

```rust
use dlms_core::{ObisCode, DlmsType, CosemDateTime};
use dlms_interface::CosemObject;
use crate::error::CosemError;

/// Disconnect Control 接口类 (Class ID: 70, Version: 1)
/// 
/// Disconnect Control 控制负载开关。
#[derive(Debug, Clone)]
pub struct DisconnectControl {
    /// logical_name
    pub logical_name: Vec<u8>,
    /// 输出状态: true=连接, false=断开
    pub output_state: bool,
    /// 控制状态: 0=disconnected, 1=connected, 2=ready_for_reconnection
    pub control_state: u8,
    /// 控制模式
    pub control_mode: u8,
}

impl DisconnectControl {
    /// 创建新的 Disconnect Control 实例
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            output_state: false,
            control_state: 0,
            control_mode: 0,
        }
    }

    /// 方法 1: remote_disconnect
    /// 
    /// 远程断开
    pub fn remote_disconnect(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 remote_disconnect
        // 参数类型: integer (0)
        Ok(DlmsType::Null)
    }

    /// 方法 2: remote_reconnect
    /// 
    /// 远程重连
    pub fn remote_reconnect(&mut self, params: DlmsType) -> Result<DlmsType, CosemError> {
        // TODO: 实现 remote_reconnect
        // 参数类型: integer (0)
        Ok(DlmsType::Null)
    }
}

impl CosemObject for DisconnectControl {
    const CLASS_ID: u16 = 70;
    const VERSION: u8 = 1;

    fn get_attribute(&self, attr_id: u8) -> Result<DlmsType, CosemError> {
        match attr_id {
            1 => Ok(self.logical_name.clone().into()),
            2 => Ok(self.output_state.clone().into()),
            3 => Ok(self.control_state.clone().into()),
            4 => Ok(self.control_mode.clone().into()),
            _ => Err(CosemError::InvalidAttribute(attr_id)),
        }
    }

    fn set_attribute(&mut self, attr_id: u8, value: DlmsType) -> Result<(), CosemError> {
        match attr_id {
            2 => {
                self.output_state = value.try_into()?;
                Ok(())
            }
            3 => {
                self.control_state = value.try_into()?;
                Ok(())
            }
            _ => Err(CosemError::ReadOnlyAttribute(attr_id)),
        }
    }

    fn invoke_method(&mut self, method_id: u8, params: DlmsType) -> Result<DlmsType, CosemError> {
        match method_id {
            1 => self.remote_disconnect(params),
            2 => self.remote_reconnect(params),
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
    fn test_disconnect__control_class_id() {
        assert_eq!(DisconnectControl::CLASS_ID, 70);
    }

    #[test]
    fn test_disconnect__control_version() {
        assert_eq!(DisconnectControl::VERSION, 1);
    }

    #[test]
    fn test_disconnect__control_new() {
        let obis = ObisCode::from_str("0.0.70.0.0.255").unwrap();
        let obj = DisconnectControl::new(obis.clone());
        assert_eq!(obj.logical_name, obis);
    }

    #[test]
    fn test_disconnect__control_get_logical_name() {
        let obis = ObisCode::from_str("0.0.70.0.0.255").unwrap();
        let obj = DisconnectControl::new(obis.clone());
        
        let result = obj.get_attribute(1).unwrap();
        let bytes: Vec<u8> = result.try_into().unwrap();
        assert_eq!(bytes.as_slice(), obis.to_bytes());
    }


    #[test]
    fn test_disconnect__control_get_output_state() {
        let obj = DisconnectControl::new(ObisCode::from_str("0.0.70.0.0.255").unwrap());
        let result = obj.get_attribute(2);
        assert!(result.is_ok());
    }


    #[test]
    fn test_disconnect__control_get_control_state() {
        let obj = DisconnectControl::new(ObisCode::from_str("0.0.70.0.0.255").unwrap());
        let result = obj.get_attribute(3);
        assert!(result.is_ok());
    }


    #[test]
    fn test_disconnect__control_get_control_mode() {
        let obj = DisconnectControl::new(ObisCode::from_str("0.0.70.0.0.255").unwrap());
        let result = obj.get_attribute(4);
        assert!(result.is_ok());
    }


    #[test]
    fn test_disconnect__control_remote_disconnect() {
        let mut obj = DisconnectControl::new(ObisCode::from_str("0.0.70.0.0.255").unwrap());
        let result = obj.invoke_method(1, DlmsType::Null);
        assert!(result.is_ok());
    }


    #[test]
    fn test_disconnect__control_remote_reconnect() {
        let mut obj = DisconnectControl::new(ObisCode::from_str("0.0.70.0.0.255").unwrap());
        let result = obj.invoke_method(2, DlmsType::Null);
        assert!(result.is_ok());
    }

}
```

---

## 6. 实现检查清单

### 6.1 数据结构

- [ ] 定义 `DisconnectControl` 结构体
- [ ] 实现所有 4 个属性字段
- [ ] 实现关联的数据类型

### 6.2 CosemObject trait

- [ ] `CLASS_ID = 70`
- [ ] `VERSION = 1`
- [ ] `get_attribute()` - 实现 4 个属性
- [ ] `set_attribute()` - 实现可写属性

### 6.3 方法实现 (2 个)

- [ ] 方法 1: `remote_disconnect()`
- [ ] 方法 2: `remote_reconnect()`

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

*文件名: IC70_DisconnectControl.md*
*生成时间: 2026-03-20*
*来源: Blue Book Edition 16 Part 2*
