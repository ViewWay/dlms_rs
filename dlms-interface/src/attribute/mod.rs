//! COSEM interface class attributes
//!
//! This module provides attribute handling functionality for COSEM interface classes.
//!
//! # Features
//!
//! - **Attribute Accessor**: Trait for accessing attributes
//! - **Access Rights**: Read/write access control
//! - **Value Validation**: Attribute value validation
//! - **Metadata**: Attribute metadata storage

use dlms_core::{DlmsResult, DlmsError, DataObject};
use std::collections::HashMap;
use std::sync::Arc;

/// Attribute access rights
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttributeAccess {
    /// No access
    #[default]
    NoAccess,
    /// Read-only access
    ReadOnly,
    /// Write-only access
    WriteOnly,
    /// Read and write access
    ReadWrite,
}

impl AttributeAccess {
    /// Check if read access is allowed
    pub fn can_read(&self) -> bool {
        matches!(self, Self::ReadOnly | Self::ReadWrite)
    }

    /// Check if write access is allowed
    pub fn can_write(&self) -> bool {
        matches!(self, Self::WriteOnly | Self::ReadWrite)
    }

    /// Create read-only access
    pub fn read_only() -> Self {
        Self::ReadOnly
    }

    /// Create write-only access
    pub fn write_only() -> Self {
        Self::WriteOnly
    }

    /// Create read-write access
    pub fn read_write() -> Self {
        Self::ReadWrite
    }
}

/// Attribute metadata
#[derive(Debug, Clone)]
pub struct AttributeMetadata {
    /// Attribute ID (1-255)
    pub id: u8,
    /// Attribute name
    pub name: String,
    /// Access rights
    pub access: AttributeAccess,
    /// Whether this attribute is mandatory
    pub mandatory: bool,
    /// Whether this attribute can be selected with selective access
    pub selectable: bool,
    /// Minimum value (for numeric types)
    pub min_value: Option<DataObject>,
    /// Maximum value (for numeric types)
    pub max_value: Option<DataObject>,
    /// Allowed values (for enum types)
    pub allowed_values: Option<Vec<DataObject>>,
}

impl AttributeMetadata {
    /// Create new attribute metadata
    pub fn new(id: u8, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            access: AttributeAccess::default(),
            mandatory: false,
            selectable: false,
            min_value: None,
            max_value: None,
            allowed_values: None,
        }
    }

    /// Set access rights
    pub fn with_access(mut self, access: AttributeAccess) -> Self {
        self.access = access;
        self
    }

    /// Set as mandatory
    pub fn with_mandatory(mut self, mandatory: bool) -> Self {
        self.mandatory = mandatory;
        self
    }

    /// Set as selectable
    pub fn with_selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Set minimum value
    pub fn with_min_value(mut self, value: DataObject) -> Self {
        self.min_value = Some(value);
        self
    }

    /// Set maximum value
    pub fn with_max_value(mut self, value: DataObject) -> Self {
        self.max_value = Some(value);
        self
    }

    /// Set allowed values
    pub fn with_allowed_values(mut self, values: Vec<DataObject>) -> Self {
        self.allowed_values = Some(values);
        self
    }

    /// Build read-only attribute metadata
    pub fn read_only(id: u8, name: impl Into<String>) -> Self {
        Self::new(id, name).with_access(AttributeAccess::ReadOnly)
    }

    /// Build read-write attribute metadata
    pub fn read_write(id: u8, name: impl Into<String>) -> Self {
        Self::new(id, name).with_access(AttributeAccess::ReadWrite)
    }

    /// Build write-only attribute metadata
    pub fn write_only(id: u8, name: impl Into<String>) -> Self {
        Self::new(id, name).with_access(AttributeAccess::WriteOnly)
    }
}

/// Attribute value validator
pub trait AttributeValidator: Send + Sync {
    /// Validate an attribute value
    ///
    /// # Arguments
    /// * `attribute_id` - Attribute ID being validated
    /// * `value` - Value to validate
    ///
    /// # Returns
    /// Ok(()) if valid, Err with description if invalid
    fn validate(&self, attribute_id: u8, value: &DataObject) -> DlmsResult<()>;
}

/// Validator based on attribute metadata
#[derive(Debug, Clone)]
pub struct MetadataValidator {
    /// Attribute metadata map
    metadata: HashMap<u8, AttributeMetadata>,
}

impl MetadataValidator {
    /// Create a new validator from metadata
    pub fn new(metadata: Vec<AttributeMetadata>) -> Self {
        let mut map = HashMap::new();
        for m in metadata {
            map.insert(m.id, m);
        }
        Self { metadata: map }
    }

    /// Add metadata for an attribute
    pub fn add_metadata(&mut self, metadata: AttributeMetadata) {
        self.metadata.insert(metadata.id, metadata);
    }

    /// Get metadata for an attribute
    pub fn get_metadata(&self, attribute_id: u8) -> Option<&AttributeMetadata> {
        self.metadata.get(&attribute_id)
    }
}

impl AttributeValidator for MetadataValidator {
    fn validate(&self, attribute_id: u8, value: &DataObject) -> DlmsResult<()> {
        let meta = self.metadata.get(&attribute_id)
            .ok_or_else(|| DlmsError::AccessDenied(format!("Unknown attribute: {}", attribute_id)))?;

        // Check allowed values if specified
        if let Some(allowed) = &meta.allowed_values {
            if !allowed.contains(value) {
                return Err(DlmsError::InvalidData(format!(
                    "Value {:?} not in allowed list for attribute {}",
                    value, attribute_id
                )));
            }
        }

        // Check min value if specified
        if let Some(min) = &meta.min_value {
            if let Err(e) = compare_values(value, min, true) {
                return Err(DlmsError::InvalidData(format!(
                    "Value {:?} below minimum {:?} for attribute {}: {}",
                    value, min, attribute_id, e
                )));
            }
        }

        // Check max value if specified
        if let Some(max) = &meta.max_value {
            if let Err(e) = compare_values(value, max, false) {
                return Err(DlmsError::InvalidData(format!(
                    "Value {:?} above maximum {:?} for attribute {}: {}",
                    value, max, attribute_id, e
                )));
            }
        }

        Ok(())
    }
}

/// Compare two DataObject values
///
/// If `check_min` is true, checks that value >= limit
/// If `check_min` is false, checks that value <= limit
fn compare_values(value: &DataObject, limit: &DataObject, check_min: bool) -> Result<(), String> {
    use DataObject::*;

    // Same type comparison
    match (value, limit) {
        // Unsigned comparisons
        (Unsigned8(v), Unsigned8(l)) => {
            if check_min && *v < *l { return Err(format!("{} < {}", v, l)); }
            if !check_min && *v > *l { return Err(format!("{} > {}", v, l)); }
        }
        (Unsigned16(v), Unsigned16(l)) => {
            if check_min && *v < *l { return Err(format!("{} < {}", v, l)); }
            if !check_min && *v > *l { return Err(format!("{} > {}", v, l)); }
        }
        (Unsigned32(v), Unsigned32(l)) => {
            if check_min && *v < *l { return Err(format!("{} < {}", v, l)); }
            if !check_min && *v > *l { return Err(format!("{} > {}", v, l)); }
        }
        (Unsigned64(v), Unsigned64(l)) => {
            if check_min && *v < *l { return Err(format!("{} < {}", v, l)); }
            if !check_min && *v > *l { return Err(format!("{} > {}", v, l)); }
        }
        // Signed comparisons
        (Integer8(v), Integer8(l)) => {
            if check_min && *v < *l { return Err(format!("{} < {}", v, l)); }
            if !check_min && *v > *l { return Err(format!("{} > {}", v, l)); }
        }
        (Integer16(v), Integer16(l)) => {
            if check_min && *v < *l { return Err(format!("{} < {}", v, l)); }
            if !check_min && *v > *l { return Err(format!("{} > {}", v, l)); }
        }
        (Integer32(v), Integer32(l)) => {
            if check_min && *v < *l { return Err(format!("{} < {}", v, l)); }
            if !check_min && *v > *l { return Err(format!("{} > {}", v, l)); }
        }
        (Integer64(v), Integer64(l)) => {
            if check_min && *v < *l { return Err(format!("{} < {}", v, l)); }
            if !check_min && *v > *l { return Err(format!("{} > {}", v, l)); }
        }
        // Float comparisons
        (Float32(v), Float32(l)) => {
            if check_min && *v < *l { return Err(format!("{} < {}", v, l)); }
            if !check_min && *v > *l { return Err(format!("{} > {}", v, l)); }
        }
        (Float64(v), Float64(l)) => {
            if check_min && *v < *l { return Err(format!("{} < {}", v, l)); }
            if !check_min && *v > *l { return Err(format!("{} > {}", v, l)); }
        }
        // Type mismatch - can't compare
        _ => {
            // For different types, we skip validation to avoid breaking existing code
            // In a production system, you might want to return an error here
        }
    }
    Ok(())
}

/// Attribute accessor trait
///
/// This trait provides methods for accessing attributes with metadata
/// and validation support.
pub trait AttributeAccessor: Send + Sync {
    /// Get attribute metadata
    fn get_attribute_metadata(&self, attribute_id: u8) -> Option<AttributeMetadata>;

    /// Get all attribute metadata
    fn get_all_metadata(&self) -> Vec<AttributeMetadata>;

    /// Check if read access is allowed
    fn can_read_attribute(&self, attribute_id: u8) -> bool {
        self.get_attribute_metadata(attribute_id)
            .map(|m| m.access.can_read())
            .unwrap_or(false)
    }

    /// Check if write access is allowed
    fn can_write_attribute(&self, attribute_id: u8) -> bool {
        self.get_attribute_metadata(attribute_id)
            .map(|m| m.access.can_write())
            .unwrap_or(false)
    }

    /// Validate an attribute value
    fn validate_attribute_value(&self, attribute_id: u8, value: &DataObject) -> DlmsResult<()> {
        // Default implementation allows all values
        let _ = attribute_id;
        let _ = value;
        Ok(())
    }
}

/// Attribute registry for managing multiple attributes
#[derive(Clone)]
pub struct AttributeRegistry {
    /// Metadata for all attributes
    metadata: HashMap<u8, AttributeMetadata>,
    /// Optional validator
    validator: Option<Arc<dyn AttributeValidator>>,
}

impl Default for AttributeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributeRegistry {
    /// Create a new attribute registry
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
            validator: None,
        }
    }

    /// Add attribute metadata
    pub fn register(&mut self, metadata: AttributeMetadata) {
        self.metadata.insert(metadata.id, metadata);
    }

    /// Register multiple attributes
    pub fn register_all(&mut self, metadata: Vec<AttributeMetadata>) {
        for m in metadata {
            self.register(m);
        }
    }

    /// Set a validator
    pub fn set_validator(&mut self, validator: Arc<dyn AttributeValidator>) {
        self.validator = Some(validator);
    }

    /// Get attribute metadata
    pub fn get(&self, attribute_id: u8) -> Option<&AttributeMetadata> {
        self.metadata.get(&attribute_id)
    }

    /// Get all metadata
    pub fn get_all(&self) -> Vec<AttributeMetadata> {
        self.metadata.values().cloned().collect()
    }

    /// Check if read access is allowed
    pub fn can_read(&self, attribute_id: u8) -> bool {
        self.get(attribute_id)
            .map(|m| m.access.can_read())
            .unwrap_or(false)
    }

    /// Check if write access is allowed
    pub fn can_write(&self, attribute_id: u8) -> bool {
        self.get(attribute_id)
            .map(|m| m.access.can_write())
            .unwrap_or(false)
    }

    /// Validate an attribute value
    pub fn validate(&self, attribute_id: u8, value: &DataObject) -> DlmsResult<()> {
        if let Some(validator) = &self.validator {
            validator.validate(attribute_id, value)?;
        }
        Ok(())
    }

    /// Get all mandatory attributes
    pub fn get_mandatory(&self) -> Vec<&AttributeMetadata> {
        self.metadata.values()
            .filter(|m| m.mandatory)
            .collect()
    }

    /// Get all selectable attributes
    pub fn get_selectable(&self) -> Vec<&AttributeMetadata> {
        self.metadata.values()
            .filter(|m| m.selectable)
            .collect()
    }
}

impl AttributeAccessor for AttributeRegistry {
    fn get_attribute_metadata(&self, attribute_id: u8) -> Option<AttributeMetadata> {
        self.get(attribute_id).cloned()
    }

    fn get_all_metadata(&self) -> Vec<AttributeMetadata> {
        self.get_all()
    }

    fn validate_attribute_value(&self, attribute_id: u8, value: &DataObject) -> DlmsResult<()> {
        self.validate(attribute_id, value)
    }
}

/// Helper trait for objects that have an attribute registry
pub trait WithAttributes: Send + Sync {
    /// Get the attribute registry
    fn attributes(&self) -> &AttributeRegistry;

    /// Check if read access is allowed
    fn can_read_attribute(&self, attribute_id: u8) -> bool {
        self.attributes().can_read(attribute_id)
    }

    /// Check if write access is allowed
    fn can_write_attribute(&self, attribute_id: u8) -> bool {
        self.attributes().can_write(attribute_id)
    }

    /// Validate an attribute value
    fn validate_attribute(&self, attribute_id: u8, value: &DataObject) -> DlmsResult<()> {
        self.attributes().validate(attribute_id, value)
    }

    /// Get attribute metadata
    fn get_attribute_info(&self, attribute_id: u8) -> Option<AttributeMetadata> {
        self.attributes().get(attribute_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dlms_core::DataObject;

    #[test]
    fn test_attribute_access() {
        let read_only = AttributeAccess::ReadOnly;
        let write_only = AttributeAccess::WriteOnly;
        let read_write = AttributeAccess::ReadWrite;
        let no_access = AttributeAccess::NoAccess;

        assert!(read_only.can_read());
        assert!(!read_only.can_write());

        assert!(!write_only.can_read());
        assert!(write_only.can_write());

        assert!(read_write.can_read());
        assert!(read_write.can_write());

        assert!(!no_access.can_read());
        assert!(!no_access.can_write());
    }

    #[test]
    fn test_attribute_metadata() {
        let meta = AttributeMetadata::read_only(2, "value")
            .with_mandatory(true)
            .with_selectable(false);

        assert_eq!(meta.id, 2);
        assert_eq!(meta.name, "value");
        assert_eq!(meta.access, AttributeAccess::ReadOnly);
        assert!(meta.mandatory);
        assert!(!meta.selectable);
    }

    #[test]
    fn test_attribute_registry() {
        let mut registry = AttributeRegistry::new();

        registry.register(AttributeMetadata::read_only(1, "logical_name"));
        registry.register(AttributeMetadata::read_write(2, "value"));
        registry.register(AttributeMetadata::write_only(3, "secret"));

        assert!(registry.can_read(1));
        assert!(!registry.can_write(1));

        assert!(registry.can_read(2));
        assert!(registry.can_write(2));

        assert!(!registry.can_read(3));
        assert!(registry.can_write(3));
    }

    #[test]
    fn test_metadata_validator_allowed_values() {
        let validator = MetadataValidator::new(vec![
            AttributeMetadata::new(2, "status")
                .with_allowed_values(vec![
                    DataObject::Unsigned8(0),
                    DataObject::Unsigned8(1),
                ])
        ]);

        // Valid value
        assert!(validator.validate(2, &DataObject::Unsigned8(0)).is_ok());
        assert!(validator.validate(2, &DataObject::Unsigned8(1)).is_ok());

        // Invalid value
        assert!(validator.validate(2, &DataObject::Unsigned8(2)).is_err());
    }

    #[test]
    fn test_metadata_validator_min_max() {
        let validator = MetadataValidator::new(vec![
            AttributeMetadata::new(2, "value")
                .with_min_value(DataObject::Unsigned16(0))
                .with_max_value(DataObject::Unsigned16(100))
        ]);

        // Valid values
        assert!(validator.validate(2, &DataObject::Unsigned16(0)).is_ok());
        assert!(validator.validate(2, &DataObject::Unsigned16(50)).is_ok());
        assert!(validator.validate(2, &DataObject::Unsigned16(100)).is_ok());

        // Invalid values
        assert!(validator.validate(2, &DataObject::Unsigned16(101)).is_err());
    }

    #[test]
    fn test_metadata_validator_min_max_signed() {
        let validator = MetadataValidator::new(vec![
            AttributeMetadata::new(2, "value")
                .with_min_value(DataObject::Integer16(-10))
                .with_max_value(DataObject::Integer16(100))
        ]);

        // Valid values
        assert!(validator.validate(2, &DataObject::Integer16(-10)).is_ok());
        assert!(validator.validate(2, &DataObject::Integer16(0)).is_ok());
        assert!(validator.validate(2, &DataObject::Integer16(50)).is_ok());
        assert!(validator.validate(2, &DataObject::Integer16(100)).is_ok());

        // Invalid values
        assert!(validator.validate(2, &DataObject::Integer16(-11)).is_err());
        assert!(validator.validate(2, &DataObject::Integer16(101)).is_err());
    }

    #[test]
    fn test_registry_mandatory() {
        let mut registry = AttributeRegistry::new();

        registry.register(AttributeMetadata::read_only(1, "logical_name")
            .with_mandatory(true));
        registry.register(AttributeMetadata::read_only(2, "value")
            .with_mandatory(false));

        let mandatory = registry.get_mandatory();
        assert_eq!(mandatory.len(), 1);
        assert_eq!(mandatory[0].id, 1);
    }

    #[test]
    fn test_registry_selectable() {
        let mut registry = AttributeRegistry::new();

        registry.register(AttributeMetadata::read_only(1, "logical_name"));
        registry.register(AttributeMetadata::read_only(2, "value")
            .with_selectable(true));
        registry.register(AttributeMetadata::read_only(3, "status")
            .with_selectable(true));

        let selectable = registry.get_selectable();
        assert_eq!(selectable.len(), 2);
        assert_eq!(selectable[0].id, 2);
        assert_eq!(selectable[1].id, 3);
    }

    #[test]
    fn test_registry_with_validator() {
        let mut registry = AttributeRegistry::new();

        registry.register(AttributeMetadata::new(2, "status")
            .with_allowed_values(vec![
                DataObject::Unsigned8(0),
                DataObject::Unsigned8(1),
            ]));

        registry.set_validator(Arc::new(MetadataValidator::new(registry.get_all())));

        // Valid value
        assert!(registry.validate(2, &DataObject::Unsigned8(0)).is_ok());

        // Invalid value
        assert!(registry.validate(2, &DataObject::Unsigned8(99)).is_err());
    }
}
