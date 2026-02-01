//! COSEM interface class methods
//!
//! This module provides method handling functionality for COSEM interface classes.
//!
//! # Features
//!
//! - **Method Invoker**: Trait for invoking methods
//! - **Method Metadata**: Information about methods
//! - **Parameter Validation**: Method parameter validation
//! - **Error Handling**: Method error handling

use dlms_core::{DlmsResult, DlmsError, DataObject};
use std::collections::HashMap;
use std::sync::Arc;

/// Method execution result
pub type MethodResult = DlmsResult<Option<DataObject>>;

/// Method parameter validator
pub trait MethodParameterValidator: Send + Sync {
    /// Validate method parameters
    ///
    /// # Arguments
    /// * `method_id` - Method ID being validated
    /// * `parameters` - Parameters to validate
    ///
    /// # Returns
    /// Ok(()) if valid, Err with description if invalid
    fn validate_parameters(&self, method_id: u8, parameters: &Option<DataObject>) -> DlmsResult<()>;
}

/// Method metadata
#[derive(Debug, Clone)]
pub struct MethodMetadata {
    /// Method ID (1-255)
    pub id: u8,
    /// Method name
    pub name: String,
    /// Whether this method is mandatory
    pub mandatory: bool,
    /// Expected parameter type (None = no parameters)
    pub parameter_type: Option<DataObjectType>,
    /// Return type (None = no return value)
    pub return_type: Option<DataObjectType>,
}

impl MethodMetadata {
    /// Create new method metadata
    pub fn new(id: u8, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            mandatory: false,
            parameter_type: None,
            return_type: None,
        }
    }

    /// Set as mandatory
    pub fn with_mandatory(mut self, mandatory: bool) -> Self {
        self.mandatory = mandatory;
        self
    }

    /// Set parameter type
    pub fn with_parameter_type(mut self, ty: DataObjectType) -> Self {
        self.parameter_type = Some(ty);
        self
    }

    /// Set return type
    pub fn with_return_type(mut self, ty: DataObjectType) -> Self {
        self.return_type = Some(ty);
        self
    }
}

/// Data object type (for parameter/return type specification)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataObjectType {
    /// Null data (no value)
    NullData,
    /// Boolean
    Boolean,
    /// Integer (8-bit signed)
    Integer,
    /// Long integer (16-bit signed)
    LongInteger,
    /// Double long (32-bit signed)
    DoubleLong,
    /// Long64 (64-bit signed)
    Long64,
    /// Unsigned (8-bit)
    Unsigned,
    /// Long unsigned (16-bit)
    LongUnsigned,
    /// Double long unsigned (32-bit)
    DoubleLongUnsigned,
    /// Long64 unsigned (64-bit)
    Long64Unsigned,
    /// Float32
    Float32,
    /// Float64
    Float64,
    /// Enumerate
    Enumerate,
    /// Octet string
    OctetString,
    /// Visible string
    VisibleString,
    /// UTF-8 string
    Utf8String,
    /// BCD
    Bcd,
    /// Binary coded decimal
    BitString,
    /// Array
    Array,
    /// Structure
    Structure,
    /// Any type
    Any,
}

impl DataObjectType {
    /// Check if a DataObject matches this type
    pub fn matches(&self, value: &DataObject) -> bool {
        use DataObjectType::*;
        match (self, value) {
            (NullData, DataObject::Null) => true,
            (Boolean, DataObject::Boolean(_)) => true,
            (Integer, DataObject::Integer8(_)) => true,
            (LongInteger, DataObject::Integer16(_)) => true,
            (DoubleLong, DataObject::Integer32(_)) => true,
            (Long64, DataObject::Integer64(_)) => true,
            (Unsigned, DataObject::Unsigned8(_)) => true,
            (LongUnsigned, DataObject::Unsigned16(_)) => true,
            (DoubleLongUnsigned, DataObject::Unsigned32(_)) => true,
            (Long64Unsigned, DataObject::Unsigned64(_)) => true,
            (Float32, DataObject::Float32(_)) => true,
            (Float64, DataObject::Float64(_)) => true,
            (Enumerate, DataObject::Enumerate(_)) => true,
            (OctetString, DataObject::OctetString(_)) => true,
            (VisibleString, DataObject::VisibleString(_)) => true,
            (Utf8String, DataObject::Utf8String(_)) => true,
            (Bcd, DataObject::Bcd(_)) => true,
            (Array, DataObject::Array(_)) => true,
            (Structure, DataObject::Structure(_)) => true,
            (Any, _) => true,
            _ => false,
        }
    }
}

/// Method invoker trait
///
/// This trait provides methods for invoking COSEM interface methods.
pub trait MethodInvoker: Send + Sync {
    /// Invoke a method
    ///
    /// # Arguments
    /// * `method_id` - Method ID to invoke (1-255)
    /// * `parameters` - Method parameters (optional)
    ///
    /// # Returns
    /// The method return value, or None if the method has no return value
    fn invoke_method(&self, method_id: u8, parameters: Option<DataObject>) -> MethodResult;

    /// Get method metadata
    fn get_method_metadata(&self, method_id: u8) -> Option<MethodMetadata>;

    /// Get all method metadata
    fn get_all_method_metadata(&self) -> Vec<MethodMetadata>;

    /// Check if a method exists
    fn has_method(&self, method_id: u8) -> bool {
        self.get_method_metadata(method_id).is_some()
    }

    /// Validate method parameters
    fn validate_method_parameters(&self, method_id: u8, parameters: &Option<DataObject>) -> DlmsResult<()> {
        // Default implementation - always valid
        let _ = method_id;
        let _ = parameters;
        Ok(())
    }
}

/// Method registry for managing multiple methods
#[derive(Clone)]
pub struct MethodRegistry {
    /// Metadata for all methods
    metadata: HashMap<u8, MethodMetadata>,
    /// Optional validator
    validator: Option<Arc<dyn MethodParameterValidator>>,
}

impl Default for MethodRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MethodRegistry {
    /// Create a new method registry
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
            validator: None,
        }
    }

    /// Add method metadata
    pub fn register(&mut self, metadata: MethodMetadata) {
        self.metadata.insert(metadata.id, metadata);
    }

    /// Register multiple methods
    pub fn register_all(&mut self, metadata: Vec<MethodMetadata>) {
        for m in metadata {
            self.register(m);
        }
    }

    /// Set a validator
    pub fn set_validator(&mut self, validator: Arc<dyn MethodParameterValidator>) {
        self.validator = Some(validator);
    }

    /// Get method metadata
    pub fn get(&self, method_id: u8) -> Option<&MethodMetadata> {
        self.metadata.get(&method_id)
    }

    /// Get all metadata
    pub fn get_all(&self) -> Vec<MethodMetadata> {
        self.metadata.values().cloned().collect()
    }

    /// Validate method parameters
    pub fn validate(&self, method_id: u8, parameters: &Option<DataObject>) -> DlmsResult<()> {
        // Check if method exists
        let meta = self.metadata.get(&method_id)
            .ok_or_else(|| DlmsError::AccessDenied(format!("Unknown method: {}", method_id)))?;

        // Check parameter type if specified
        if let Some(expected_type) = &meta.parameter_type {
            match parameters {
                Some(params) if !expected_type.matches(params) => {
                    return Err(DlmsError::InvalidData(format!(
                        "Parameter type mismatch for method {}: expected {:?}, got {:?}",
                        method_id, expected_type, params
                    )));
                }
                None => {
                    return Err(DlmsError::InvalidData(format!(
                        "Method {} requires parameters, but none provided",
                        method_id
                    )));
                }
                _ => {}
            }
        }

        // Use custom validator if set
        if let Some(validator) = &self.validator {
            validator.validate_parameters(method_id, parameters)?;
        }

        Ok(())
    }

    /// Get all mandatory methods
    pub fn get_mandatory(&self) -> Vec<&MethodMetadata> {
        self.metadata.values()
            .filter(|m| m.mandatory)
            .collect()
    }
}

/// Helper trait for objects that have a method registry
pub trait WithMethods: Send + Sync {
    /// Get the method registry
    fn methods(&self) -> &MethodRegistry;

    /// Check if a method exists
    fn has_method(&self, method_id: u8) -> bool {
        self.methods().get(method_id).is_some()
    }

    /// Validate method parameters
    fn validate_method_parameters(&self, method_id: u8, parameters: &Option<DataObject>) -> DlmsResult<()> {
        self.methods().validate(method_id, parameters)
    }

    /// Get method metadata
    fn get_method_info(&self, method_id: u8) -> Option<MethodMetadata> {
        self.methods().get(method_id).cloned()
    }
}

/// Simple method handler
///
/// A function-based method handler that can be used to implement methods.
pub type MethodHandler = Arc<dyn Fn(Option<DataObject>) -> MethodResult + Send + Sync>;

/// Method registry with handlers
#[derive(Clone)]
pub struct MethodHandlerRegistry {
    /// Metadata for all methods
    metadata: HashMap<u8, MethodMetadata>,
    /// Method handlers
    handlers: HashMap<u8, MethodHandler>,
}

impl Default for MethodHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MethodHandlerRegistry {
    /// Create a new method handler registry
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
            handlers: HashMap::new(),
        }
    }

    /// Add a method with handler
    pub fn register(&mut self, metadata: MethodMetadata, handler: MethodHandler) {
        self.handlers.insert(metadata.id, handler);
        self.metadata.insert(metadata.id, metadata);
    }

    /// Get method metadata
    pub fn get_metadata(&self, method_id: u8) -> Option<&MethodMetadata> {
        self.metadata.get(&method_id)
    }

    /// Get all metadata
    pub fn get_all_metadata(&self) -> Vec<MethodMetadata> {
        self.metadata.values().cloned().collect()
    }

    /// Invoke a method
    pub fn invoke(&self, method_id: u8, parameters: Option<DataObject>) -> MethodResult {
        let handler = self.handlers.get(&method_id)
            .ok_or_else(|| DlmsError::AccessDenied(format!("Unknown method: {}", method_id)))?;

        // Validate parameter types
        if let Some(meta) = self.metadata.get(&method_id) {
            if let Some(expected_type) = &meta.parameter_type {
                match &parameters {
                    Some(params) if !expected_type.matches(params) => {
                        return Err(DlmsError::InvalidData(format!(
                            "Parameter type mismatch for method {}: expected {:?}",
                            method_id, expected_type
                        )));
                    }
                    None if meta.parameter_type.is_some() => {
                        return Err(DlmsError::InvalidData(format!(
                            "Method {} requires parameters",
                            method_id
                        )));
                    }
                    _ => {}
                }
            }
        }

        handler(parameters)
    }

    /// Check if a method exists
    pub fn has_method(&self, method_id: u8) -> bool {
        self.handlers.contains_key(&method_id)
    }
}

/// Validator based on method metadata
#[derive(Debug, Clone)]
pub struct MetadataMethodValidator {
    /// Method metadata map
    metadata: HashMap<u8, MethodMetadata>,
}

impl MetadataMethodValidator {
    /// Create a new validator from metadata
    pub fn new(metadata: Vec<MethodMetadata>) -> Self {
        let mut map = HashMap::new();
        for m in metadata {
            map.insert(m.id, m);
        }
        Self { metadata: map }
    }
}

impl MethodParameterValidator for MetadataMethodValidator {
    fn validate_parameters(&self, method_id: u8, parameters: &Option<DataObject>) -> DlmsResult<()> {
        let meta = self.metadata.get(&method_id)
            .ok_or_else(|| DlmsError::AccessDenied(format!("Unknown method: {}", method_id)))?;

        // Check parameter type if specified
        if let Some(expected_type) = &meta.parameter_type {
            match parameters {
                Some(params) if !expected_type.matches(params) => {
                    return Err(DlmsError::InvalidData(format!(
                        "Parameter type mismatch for method {}: expected {:?}",
                        method_id, expected_type
                    )));
                }
                None => {
                    return Err(DlmsError::InvalidData(format!(
                        "Method {} requires parameters, but none provided",
                        method_id
                    )));
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_metadata() {
        let meta = MethodMetadata::new(1, "reset")
            .with_mandatory(true)
            .with_return_type(DataObjectType::NullData);

        assert_eq!(meta.id, 1);
        assert_eq!(meta.name, "reset");
        assert!(meta.mandatory);
        assert_eq!(meta.return_type, Some(DataObjectType::NullData));
    }

    #[test]
    fn test_method_registry() {
        let mut registry = MethodRegistry::new();

        registry.register(MethodMetadata::new(1, "reset"));
        registry.register(MethodMetadata::new(2, "get_status")
            .with_return_type(DataObjectType::Unsigned));

        assert!(registry.get(1).is_some());
        assert!(registry.get(2).is_some());
        assert!(registry.get(3).is_none());
    }

    #[test]
    fn test_method_registry_validate() {
        let mut registry = MethodRegistry::new();

        registry.register(MethodMetadata::new(1, "set_value")
            .with_parameter_type(DataObjectType::Unsigned));

        // Valid parameters
        assert!(registry.validate(1, &Some(DataObject::Unsigned8(42))).is_ok());

        // Invalid parameters
        assert!(registry.validate(1, &Some(DataObject::Boolean(true))).is_err());

        // Missing parameters
        assert!(registry.validate(1, &None).is_err());
    }

    #[test]
    fn test_method_handler_registry() {
        let mut registry = MethodHandlerRegistry::new();

        let handler: MethodHandler = Arc::new(|params| match params {
            Some(DataObject::Unsigned8(n)) => Ok(Some(DataObject::Unsigned8(n + 1))),
            _ => Err(DlmsError::InvalidData("Expected Unsigned8 parameter".into())),
        });

        registry.register(
            MethodMetadata::new(1, "increment")
                .with_parameter_type(DataObjectType::Unsigned)
                .with_return_type(DataObjectType::Unsigned),
            handler,
        );

        // Valid invocation
        let result = registry.invoke(1, Some(DataObject::Unsigned8(5))).unwrap();
        assert_eq!(result, Some(DataObject::Unsigned8(6)));

        // Invalid parameter type
        assert!(registry.invoke(1, Some(DataObject::Boolean(true))).is_err());

        // Unknown method
        assert!(registry.invoke(99, None).is_err());
    }

    #[test]
    fn test_data_object_type_matches() {
        use DataObject::*;

        assert!(DataObjectType::Boolean.matches(&Boolean(true)));
        assert!(!DataObjectType::Boolean.matches(&Unsigned8(1)));

        assert!(DataObjectType::Unsigned.matches(&Unsigned8(42)));
        assert!(!DataObjectType::Unsigned.matches(&Unsigned16(42)));

        assert!(DataObjectType::OctetString.matches(&OctetString(vec![])));
        assert!(DataObjectType::Any.matches(&Boolean(true)));
        assert!(DataObjectType::Any.matches(&Structure(vec![])));
    }

    #[test]
    fn test_metadata_method_validator() {
        let validator = MetadataMethodValidator::new(vec![
            MethodMetadata::new(1, "set_value")
                .with_parameter_type(DataObjectType::LongUnsigned),
        ]);

        // Valid parameters
        assert!(validator.validate_parameters(1, &Some(DataObject::Unsigned16(100))).is_ok());

        // Invalid parameters
        assert!(validator.validate_parameters(1, &Some(DataObject::Unsigned8(100))).is_err());

        // Missing parameters
        assert!(validator.validate_parameters(1, &None).is_err());
    }
}
