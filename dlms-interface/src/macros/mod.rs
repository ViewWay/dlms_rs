//! Macros for defining COSEM interface classes
//!
//! This module provides procedural and declarative macros to simplify the definition
//! of COSEM interface classes, reducing boilerplate code.

/// Macro to define a COSEM interface class with standard attributes
///
/// This macro generates a struct with common COSEM object fields and implements
/// the Debug, Clone, and PartialEq traits.
///
/// # Syntax
///
/// ```ignore
/// cosem_class! {
///     /// Optional documentation
///     pub struct MyObject {
///         // Standard fields (automatically included):
///         // - logical_name: ObisCode
///         // Optional custom fields:
///         custom_field: Type,
///     }
/// }
/// ```
///
/// # Example
///
/// ```ignore
/// cosem_class! {
///     /// My custom COSEM object
///     pub struct MyObject {
///         value: Arc<RwLock<DataObject>>,
///         status: Arc<RwLock<Option<u8>>>,
///     }
/// }
/// ```
///
/// This expands to:
/// ```ignore
/// #[derive(Clone)]
/// pub struct MyObject {
///     pub logical_name: ObisCode,
///     value: Arc<RwLock<DataObject>>,
///     status: Arc<RwLock<Option<u8>>>,
/// }
/// ```
#[macro_export]
macro_rules! cosem_class {
    (
        $(#[$attr:meta])*
        pub struct $name:ident {
            $(
                $(#[$field_attr:meta])*
                $field_name:ident : $field_ty:ty
            ),* $(,)?
        }
    ) => {
        $(#[$attr])*
        #[derive(Clone)]
        pub struct $name {
            /// Logical name (OBIS code) of this object
            pub logical_name: $crate::ObisCode,
            $(
                $(#[$field_attr])*
                $field_name : $field_ty,
            )*
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("logical_name", &self.logical_name)
                    $(
                        .field(stringify!($field_name), &self.$field_name)
                    )*
                    .finish()
            }
        }
    };
}

/// Macro to implement the CosemObject trait for a struct
///
/// This macro implements the required methods from the CosemObject trait
/// based on provided attribute and method definitions.
///
/// # Syntax
///
/// ```ignore
/// impl_cosem_object_for!(MyStruct, class_id = 3, {
///     attributes {
///         // attribute_id, getter_type, setter_type
///         1: logical_name, none,
///         2: value, data,
///         3: scaler_unit, none,
///     }
///     methods {
///         // method_id, handler
///     }
/// });
/// ```
///
/// # Example
///
/// ```ignore
/// impl_cosem_object_for!(Register, class_id = 3, {
///     attributes {
///         1: logical_name, none,
///         2: value, data,
///         3: scaler_unit, none,
///         4: status, optional_data,
///     }
/// });
/// ```
#[macro_export]
macro_rules! impl_cosem_object_for {
    (
        $struct_name:ident, class_id = $class_id:expr, {
            attributes {
                $(
                    $attr_id:tt : $attr_getter:ident $(, $attr_setter:ident)?
                ),* $(,)?
            }
            $( methods {
                $(
                    $method_id:tt : $method_handler:ident
                ),* $(,)?
            } )?
        }
    ) => {
        #[async_trait::async_trait]
        impl $crate::CosemObject for $struct_name {
            fn obis_code(&self) -> $crate::ObisCode {
                self.logical_name
            }

            fn class_id(&self) -> u16 {
                $class_id
            }

            async fn get_attribute(
                &self,
                attribute_id: u8,
                _access_selector: Option<&$crate::dlms_application::pdu::SelectiveAccessDescriptor>,
                ctx: Option<&$crate::association_access::CosemInvocationContext>,
            ) -> $crate::DlmsResult<$crate::DataObject> {
                $crate::enforce_attribute_read(
                    ctx,
                    self.class_id(),
                    self.obis_code(),
                    attribute_id,
                )
                .await?;
                match attribute_id {
                    $(
                        $attr_id => self.$attr_getter().await,
                    )*
                    _ => Err($crate::DlmsError::InvalidData(format!(
                        "Unknown attribute ID: {}", attribute_id
                    ))),
                }
            }

            async fn set_attribute(
                &self,
                attribute_id: u8,
                value: $crate::DataObject,
                _access_selector: Option<&$crate::dlms_application::pdu::SelectiveAccessDescriptor>,
                ctx: Option<&$crate::association_access::CosemInvocationContext>,
            ) -> $crate::DlmsResult<()> {
                $crate::enforce_attribute_write(
                    ctx,
                    self.class_id(),
                    self.obis_code(),
                    attribute_id,
                )
                .await?;
                match attribute_id {
                    $(
                        $attr_id => {
                            $(
                                self.$attr_setter(value).await
                            )?
                        }
                    )*
                    _ => Err($crate::DlmsError::InvalidData(format!(
                        "Cannot set attribute ID: {}", attribute_id
                    ))),
                }
            }

            async fn invoke_method(
                &self,
                method_id: u8,
                _parameters: Option<$crate::DataObject>,
                _selective_access: Option<&$crate::dlms_application::pdu::SelectiveAccessDescriptor>,
                ctx: Option<&$crate::association_access::CosemInvocationContext>,
            ) -> $crate::DlmsResult<Option<$crate::DataObject>> {
                $crate::enforce_method_execute(
                    ctx,
                    self.class_id(),
                    self.obis_code(),
                    method_id,
                )
                .await?;
                match method_id {
                    $(
                        $(
                            $method_id => self.$method_handler().await,
                        )*
                    )*
                    _ => Err($crate::DlmsError::InvalidData(format!(
                        "Unknown method ID: {}", method_id
                    ))),
                }
            }
        }
    };
}

/// Macro to generate attribute accessor methods
///
/// This macro generates getter (and optionally setter) methods for attributes.
///
/// # Syntax
///
/// ```ignore
/// accessors! {
///     /// Attribute 1: value (read-write)
///     value[2]: DataObject,
///     /// Attribute 2: status (read-only)
///     status[3]: Option<u8> as DataObject,
/// }
/// ```
///
/// # Example
///
/// ```ignore
/// accessors! {
///     /// Current value
///     value: DataObject,
///     /// Status byte
///     status: Option<u8>,
/// }
/// ```
#[macro_export]
macro_rules! accessors {
    (
        $(
            $(#[$attr_doc:meta])*
            $field_name:ident : $inner_ty:ty $(as $data_wrapper:expr)?
        ),* $(,)?
    ) => {
        $(
            $(#[$attr_doc])*
            pub async fn $field_name(&self) -> $crate::DlmsResult<$crate::DataObject> {
                let value = self.$field_name.read().await;
                Ok($crate::DataObject::clone(&*value))
            }

            $(
                pub async fn set_$field_name(&self, value: $crate::DataObject) -> $crate::DlmsResult<()> {
                    let mut field = self.$field_name.write().await;
                    *field = value;
                    Ok(())
                }
            )?
        )*
    };
}

/// Standard attribute IDs for COSEM objects
pub struct CosemAttribute;

impl CosemAttribute {
    /// Attribute 1: logical_name
    pub const LOGICAL_NAME: u8 = 1;
}

impl CosemAttribute {
    /// Helper to get attribute ID by name
    pub fn from_name(name: &str) -> Option<u8> {
        match name {
            "logical_name" => Some(Self::LOGICAL_NAME),
            _ => None,
        }
    }
}

/// Macro to generate a simple attribute getter
///
/// # Syntax
///
/// ```ignore
/// getter!(my_field, DataObject);
/// ```
#[macro_export]
macro_rules! getter {
    ($field:ident, $ty:ty) => {
        pub async fn $field(&self) -> $crate::DlmsResult<$crate::DataObject> {
            let value = self.$field.read().await;
            Ok($crate::DataObject::clone(&*value))
        }
    };
}

/// Macro to generate a simple attribute setter
///
/// # Syntax
///
/// ```ignore
/// setter!(my_field, set_my_field);
/// ```
#[macro_export]
macro_rules! setter {
    ($field:ident, $setter_name:ident) => {
        pub async fn $setter_name(&self, value: $crate::DataObject) -> $crate::DlmsResult<()> {
            let mut field = self.$field.write().await;
            *field = value;
            Ok(())
        }
    };
}

/// Macro to generate both getter and setter for an attribute
///
/// # Syntax
///
/// ```ignore
/// accessor!(value, DataObject, set_value);
/// ```
#[macro_export]
macro_rules! accessor {
    ($field:ident, $getter_name:ident, $setter_name:ident) => {
        getter!($field, $crate::DataObject);
        setter!($field, $setter_name);
    };
}

/// Macro to implement Default for a COSEM object with given OBIS code
///
/// # Syntax
///
/// ```ignore
/// impl_default_for!(MyObject, [1, 1, 1, 8, 0, 255]);
/// ```
#[macro_export]
macro_rules! impl_default_for {
    ($struct_name:ident, [$($obis:expr),* $(,)?]) => {
        impl Default for $struct_name {
            fn default() -> Self {
                Self::new($crate::ObisCode::new($($obis),*))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use dlms_core::{ObisCode, DataObject};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Test the cosem_class macro
    cosem_class! {
        /// Test object for macro testing
        pub struct TestObject {
            value: Arc<RwLock<DataObject>>,
            counter: Arc<RwLock<u32>>,
        }
    }

    impl TestObject {
        pub fn new(logical_name: ObisCode) -> Self {
            Self {
                logical_name,
                value: Arc::new(RwLock::new(DataObject::new_null())),
                counter: Arc::new(RwLock::new(0)),
            }
        }

        pub async fn value(&self) -> dlms_core::DlmsResult<DataObject> {
            let value = self.value.read().await;
            Ok(DataObject::clone(&*value))
        }

        pub async fn set_value(&self, value: DataObject) -> dlms_core::DlmsResult<()> {
            let mut v = self.value.write().await;
            *v = value;
            Ok(())
        }

        pub async fn counter(&self) -> dlms_core::DlmsResult<DataObject> {
            let c = self.counter.read().await;
            Ok(DataObject::new_unsigned32(*c))
        }
    }

    #[test]
    fn test_cosem_class_macro() {
        let obj = TestObject::new(ObisCode::new(1, 2, 3, 4, 5, 6));
        assert_eq!(obj.logical_name, ObisCode::new(1, 2, 3, 4, 5, 6));
    }

    #[test]
    fn test_attribute_ids() {
        assert_eq!(CosemAttribute::LOGICAL_NAME, 1);
    }

    #[tokio::test]
    async fn test_accessors() {
        let obj = TestObject::new(ObisCode::new(1, 1, 1, 8, 0, 255));

        // Test getter
        let value = obj.value().await.unwrap();
        assert!(value.is_null());

        // Test setter
        obj.set_value(DataObject::new_unsigned32(42)).await.unwrap();

        let value = obj.value().await.unwrap();
        assert_eq!(value.as_unsigned32().unwrap(), 42);
    }
}
