//! Batch data reading service for DLMS/COSEM client
//!
//! This module provides functionality for reading multiple attributes
//! in a single request using GetRequest-With-List.

use dlms_core::{DlmsResult, ObisCode, DataObject};
use dlms_application::pdu::{
    GetRequest, GetResponse,
    CosemAttributeDescriptor, InvokeIdAndPriority, GetDataResult,
};
use std::time::Duration;

/// Batch read result
///
/// Represents the result of a batch read operation.
#[derive(Debug, Clone)]
pub struct BatchReadResult {
    /// Successful reads
    pub successful: Vec<AttributeReadResult>,
    /// Failed reads
    pub failed: Vec<AttributeReadError>,
}

/// Successful attribute read result
#[derive(Debug, Clone)]
pub struct AttributeReadResult {
    /// OBIS code of the object
    pub obis_code: ObisCode,
    /// Class ID of the object
    pub class_id: u16,
    /// Attribute ID
    pub attribute_id: u8,
    /// Attribute value
    pub value: DataObject,
}

/// Failed attribute read result
#[derive(Debug, Clone)]
pub struct AttributeReadError {
    /// OBIS code of the object
    pub obis_code: ObisCode,
    /// Class ID of the object (if known)
    pub class_id: Option<u16>,
    /// Attribute ID
    pub attribute_id: u8,
    /// Error message
    pub error: String,
}

/// Attribute reference for batch operations
#[derive(Debug, Clone)]
pub struct AttributeReference {
    /// OBIS code of the object
    pub obis_code: ObisCode,
    /// Class ID of the object
    pub class_id: u16,
    /// Attribute ID to read
    pub attribute_id: u8,
}

impl AttributeReference {
    /// Create a new attribute reference
    pub fn new(obis_code: ObisCode, class_id: u16, attribute_id: u8) -> Self {
        Self {
            obis_code,
            class_id,
            attribute_id,
        }
    }

    /// Convert to CosemAttributeDescriptor
    pub fn to_descriptor(&self) -> CosemAttributeDescriptor {
        // Note: This creates a new descriptor each time
        // In production code, you might want to cache this or handle errors
        CosemAttributeDescriptor::new_logical_name(
            self.class_id,
            self.obis_code,
            self.attribute_id,
        ).unwrap_or_else(|_| {
            // Fallback to a basic descriptor if creation fails
            // This should not happen with valid inputs
            CosemAttributeDescriptor::new_logical_name(
                0,
                ObisCode::new(0, 0, 0, 0, 0, 1),
                1,
            ).unwrap()
        })
    }
}

/// Batch reader
///
/// Provides methods for reading multiple attributes efficiently.
pub struct BatchReader<'a> {
    /// Reference to the connection
    connection: &'a mut (dyn crate::Connection + Send + Sync),
    /// Maximum number of attributes per request
    max_per_request: usize,
}

impl<'a> BatchReader<'a> {
    /// Create a new batch reader
    ///
    /// # Arguments
    /// * `connection` - Reference to the connection
    pub fn new(connection: &'a mut (dyn crate::Connection + Send + Sync)) -> Self {
        Self {
            connection,
            max_per_request: 10, // Default limit
        }
    }

    /// Set the maximum number of attributes per request
    ///
    /// # Arguments
    /// * `max` - Maximum per request
    pub fn with_max_per_request(mut self, max: usize) -> Self {
        self.max_per_request = max;
        self
    }

    /// Read multiple attributes in a single request
    ///
    /// # Arguments
    /// * `attributes` - List of attribute references to read
    ///
    /// # Returns
    /// Batch read result with successes and failures
    pub async fn read_attributes(
        &mut self,
        attributes: Vec<AttributeReference>,
    ) -> DlmsResult<BatchReadResult> {
        let mut successful = Vec::new();
        let mut failed = Vec::new();

        // Split into chunks if exceeding max_per_request
        for chunk in attributes.chunks(self.max_per_request) {
            let invoke_id = InvokeIdAndPriority::new(1, false)?;
            let descriptors: Vec<_> = chunk
                .iter()
                .map(|a| a.to_descriptor())
                .collect();

            // Create GetRequest-With-List
            let request = GetRequest::WithList {
                invoke_id_and_priority: invoke_id,
                attribute_descriptor_list: descriptors,
                access_selection_list: None,
            };
            let request_data = request.encode()?;

            // Send request
            let response_data = self.connection.send_request(&request_data, Some(Duration::from_secs(10))).await?;

            // Parse response
            match GetResponse::decode(&response_data)? {
                GetResponse::WithList { result_list, .. } => {
                    for (i, result) in result_list.iter().enumerate() {
                        let attr = &chunk[i];

                        match result {
                            GetDataResult::Data(value) => {
                                successful.push(AttributeReadResult {
                                    obis_code: attr.obis_code,
                                    class_id: attr.class_id,
                                    attribute_id: attr.attribute_id,
                                    value: value.clone(),
                                });
                            }
                            GetDataResult::DataAccessResult(code) => {
                                failed.push(AttributeReadError {
                                    obis_code: attr.obis_code,
                                    class_id: Some(attr.class_id),
                                    attribute_id: attr.attribute_id,
                                    error: format!("Access error: code {}", code),
                                });
                            }
                            GetDataResult::DataBlock(_) => {
                                // Block transfer not supported in this context
                                failed.push(AttributeReadError {
                                    obis_code: attr.obis_code,
                                    class_id: Some(attr.class_id),
                                    attribute_id: attr.attribute_id,
                                    error: "Block transfer not supported".to_string(),
                                });
                            }
                        }
                    }
                }
                GetResponse::Normal(normal) => {
                    // Single response - treat all as failed except possibly first
                    match normal.result {
                        GetDataResult::DataAccessResult(code) => {
                            for attr in chunk {
                                failed.push(AttributeReadError {
                                    obis_code: attr.obis_code,
                                    class_id: Some(attr.class_id),
                                    attribute_id: attr.attribute_id,
                                    error: format!("Access error: code {}", code),
                                });
                            }
                        }
                        _ => {
                            for attr in chunk {
                                failed.push(AttributeReadError {
                                    obis_code: attr.obis_code,
                                    class_id: Some(attr.class_id),
                                    attribute_id: attr.attribute_id,
                                    error: "Unexpected response format".to_string(),
                                });
                            }
                        }
                    }
                }
                _ => {
                    // Unexpected response format
                    for attr in chunk {
                        failed.push(AttributeReadError {
                            obis_code: attr.obis_code,
                            class_id: Some(attr.class_id),
                            attribute_id: attr.attribute_id,
                            error: "Unexpected response format".to_string(),
                        });
                    }
                }
            }
        }

        // Fallback: read remaining attributes individually
        // This handles cases where the server doesn't support With-List
        for attr in attributes {
            let was_read = successful.iter().any(|r| {
                r.obis_code == attr.obis_code && r.attribute_id == attr.attribute_id
            });
            let had_error = failed.iter().any(|e| {
                e.obis_code == attr.obis_code && e.attribute_id == attr.attribute_id
            });

            if !was_read && !had_error {
                match self
                    .connection
                    .get_attribute(attr.obis_code, attr.class_id, attr.attribute_id)
                    .await
                {
                    Ok(value) => {
                        successful.push(AttributeReadResult {
                            obis_code: attr.obis_code,
                            class_id: attr.class_id,
                            attribute_id: attr.attribute_id,
                            value,
                        });
                    }
                    Err(e) => {
                        failed.push(AttributeReadError {
                            obis_code: attr.obis_code,
                            class_id: Some(attr.class_id),
                            attribute_id: attr.attribute_id,
                            error: e.to_string(),
                        });
                    }
                }
            }
        }

        Ok(BatchReadResult {
            successful,
            failed,
        })
    }

    /// Read all specified attributes, returning a hashmap
    ///
    /// This is a convenience method that returns the results as a map.
    ///
    /// # Arguments
    /// * `attributes` - List of attribute references to read
    ///
    /// # Returns
    /// HashMap of (obis_code, attribute_id) -> value
    pub async fn read_attributes_map(
        &mut self,
        attributes: Vec<AttributeReference>,
    ) -> DlmsResult<std::collections::HashMap<(ObisCode, u8), DataObject>> {
        let result = self.read_attributes(attributes).await?;

        let mut map = std::collections::HashMap::new();
        for success in result.successful {
            map.insert((success.obis_code, success.attribute_id), success.value);
        }

        Ok(map)
    }

    /// Read multiple attributes from the same object
    ///
    /// # Arguments
    /// * `obis` - OBIS code of the object
    /// * `class_id` - Class ID of the object
    /// * `attribute_ids` - List of attribute IDs to read
    ///
    /// # Returns
    /// Batch read result
    pub async fn read_object_attributes(
        &mut self,
        obis: ObisCode,
        class_id: u16,
        attribute_ids: &[u8],
    ) -> DlmsResult<BatchReadResult> {
        let attributes: Vec<_> = attribute_ids
            .iter()
            .map(|&id| AttributeReference::new(obis, class_id, id))
            .collect();

        self.read_attributes(attributes).await
    }

    /// Read specific attributes from multiple objects
    ///
    /// # Arguments
    /// * `objects` - List of (obis_code, class_id) tuples
    /// * `attribute_id` - Attribute ID to read from each object
    ///
    /// # Returns
    /// Batch read result
    pub async fn read_same_attribute_from_objects(
        &mut self,
        objects: Vec<(ObisCode, u16)>,
        attribute_id: u8,
    ) -> DlmsResult<BatchReadResult> {
        let attributes: Vec<_> = objects
            .into_iter()
            .map(|(obis, class_id)| AttributeReference::new(obis, class_id, attribute_id))
            .collect();

        self.read_attributes(attributes).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_reference() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let attr = AttributeReference::new(obis, 3, 2);

        assert_eq!(attr.obis_code, obis);
        assert_eq!(attr.class_id, 3);
        assert_eq!(attr.attribute_id, 2);
    }

    #[test]
    fn test_batch_read_result() {
        let result = BatchReadResult {
            successful: vec![],
            failed: vec![],
        };

        assert!(result.successful.is_empty());
        assert!(result.failed.is_empty());
    }
}
