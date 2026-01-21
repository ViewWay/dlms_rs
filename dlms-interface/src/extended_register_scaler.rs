//! Extended Register (with scaler) interface class enhancement (Class ID: 4)
//!
//! This is a basic extended register with additional scaler/unit support.

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DlmsError, DlmsResult, ObisCode, DataObject};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::CosemObject;

/// Extended Register with Scaler
#[derive(Debug, Clone)]
pub struct ExtendedRegisterScaler {
    logical_name: ObisCode,
    value: Arc<RwLock<i64>>,
    scaler: Arc<RwLock<i32>>,
    unit: Arc<RwLock<String>>,
}

impl ExtendedRegisterScaler {
    pub const CLASS_ID: u16 = 4;

    pub fn default_obis() -> ObisCode {
        ObisCode::new(1, 1, 4, 0, 0, 255)
    }

    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            value: Arc::new(RwLock::new(0)),
            scaler: Arc::new(RwLock::new(0)),
            unit: Arc::new(RwLock::new(String::new())),
        }
    }

    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    pub async fn value(&self) -> i64 {
        *self.value.read().await
    }

    pub async fn set_value(&self, value: i64) {
        *self.value.write().await = value;
    }

    pub async fn scaler(&self) -> i32 {
        *self.scaler.read().await
    }

    pub async fn set_scaler(&self, scaler: i32) {
        *self.scaler.write().await = scaler;
    }

    pub async fn unit(&self) -> String {
        self.unit.read().await.clone()
    }

    pub async fn set_unit(&self, unit: String) {
        *self.unit.write().await = unit;
    }

    pub async fn scaled_value(&self) -> f64 {
        let value = self.value().await as f64;
        let scaler = self.scaler().await as f64;
        if scaler != 0.0 {
            value / (10.0_f64.powi(scaler as i32))
        } else {
            value
        }
    }
}

#[async_trait]
impl CosemObject for ExtendedRegisterScaler {
    fn class_id(&self) -> u16 {
        Self::CLASS_ID
    }

    fn obis_code(&self) -> ObisCode {
        self.logical_name
    }

    async fn get_attribute(
        &self,
        attribute_id: u8,
        _selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<DataObject> {
        match attribute_id {
            1 => Ok(DataObject::OctetString(self.logical_name.to_bytes().to_vec())),
            2 => Ok(DataObject::Integer64(self.value().await)),
            3 => Ok(DataObject::Integer32(self.scaler().await)),
            4 => Ok(DataObject::OctetString(self.unit().await.into_bytes())),
            _ => Err(DlmsError::InvalidData(format!(
                "ExtendedRegisterScaler has no attribute {}",
                attribute_id
            ))),
        }
    }

    async fn set_attribute(
        &self,
        attribute_id: u8,
        value: DataObject,
        _selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<()> {
        match attribute_id {
            1 => Err(DlmsError::AccessDenied("Attribute 1 is read-only".to_string())),
            2 => match value {
                DataObject::Integer64(v) => {
                    self.set_value(v).await;
                    Ok(())
                }
                _ => Err(DlmsError::InvalidData("Expected Integer64".to_string())),
            },
            3 => match value {
                DataObject::Integer32(s) => {
                    self.set_scaler(s).await;
                    Ok(())
                }
                _ => Err(DlmsError::InvalidData("Expected Integer32".to_string())),
            },
            4 => match value {
                DataObject::OctetString(bytes) => {
                    self.set_unit(String::from_utf8_lossy(&bytes).to_string()).await;
                    Ok(())
                }
                _ => Err(DlmsError::InvalidData("Expected OctetString".to_string())),
            },
            _ => Err(DlmsError::InvalidData(format!(
                "ExtendedRegisterScaler has no attribute {}",
                attribute_id
            ))),
        }
    }

    async fn invoke_method(
        &self,
        method_id: u8,
        _parameters: Option<DataObject>,
        _selective_access: Option<&SelectiveAccessDescriptor>,
    ) -> DlmsResult<Option<DataObject>> {
        Err(DlmsError::InvalidData(format!(
            "ExtendedRegisterScaler has no method {}",
            method_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extended_register_scaler() {
        let reg = ExtendedRegisterScaler::with_default_obis();
        assert_eq!(reg.class_id(), 4);

        reg.set_value(12345).await;
        assert_eq!(reg.value().await, 12345);

        reg.set_scaler(2).await;
        assert_eq!(reg.scaler().await, 2);

        let scaled = reg.scaled_value().await;
        assert!((scaled - 123.45).abs() < 0.01);
    }
}
