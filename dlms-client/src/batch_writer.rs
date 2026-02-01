//! Batch data writing service for DLMS/COSEM client
//!
//! This module provides functionality for writing multiple attributes
//! in a single request using SetRequest-With-List.

use dlms_core::{DlmsResult, ObisCode, DataObject};
use dlms_application::pdu::{
    SetRequest, SetResponse,
    CosemAttributeDescriptor, InvokeIdAndPriority, SetDataResult,
};
use std::time::Duration;

/// Batch write result
///
/// Represents the result of a batch write operation.
#[derive(Debug, Clone)]
pub struct BatchWriteResult {
    /// Successful writes
    pub successful: Vec<AttributeWriteResult>,
    /// Failed writes
    pub failed: Vec<AttributeWriteError>,
}

/// Successful attribute write result
#[derive(Debug, Clone)]
pub struct AttributeWriteResult {
    /// OBIS code of the object
    pub obis_code: ObisCode,
    /// Class ID of the object
    pub class_id: u16,
    /// Attribute ID
    pub attribute_id: u8,
}

/// Failed attribute write result
#[derive(Debug, Clone)]
pub struct AttributeWriteError {
    /// OBIS code of the object
    pub obis_code: ObisCode,
    /// Class ID of the object (if known)
    pub class_id: Option<u16>,
    /// Attribute ID
    pub attribute_id: u8,
    /// Error message
    pub error: String,
}

/// Attribute value reference for batch write operations
#[derive(Debug, Clone)]
pub struct AttributeValue {
    /// OBIS code of the object
    pub obis_code: ObisCode,
    /// Class ID of the object
    pub class_id: u16,
    /// Attribute ID to write
    pub attribute_id: u8,
    /// Value to write
    pub value: DataObject,
}

impl AttributeValue {
    /// Create a new attribute value reference
    pub fn new(obis_code: ObisCode, class_id: u16, attribute_id: u8, value: DataObject) -> Self {
        Self {
            obis_code,
            class_id,
            attribute_id,
            value,
        }
    }

    /// Convert to CosemAttributeDescriptor
    pub fn to_descriptor(&self) -> CosemAttributeDescriptor {
        CosemAttributeDescriptor::new_logical_name(
            self.class_id,
            self.obis_code,
            self.attribute_id,
        ).unwrap_or_else(|_| {
            // Fallback to a basic descriptor if creation fails
            CosemAttributeDescriptor::new_logical_name(
                0,
                ObisCode::new(0, 0, 0, 0, 0, 1),
                1,
            ).unwrap()
        })
    }
}

/// Batch writer
///
/// Provides methods for writing multiple attributes efficiently.
pub struct BatchWriter<'a> {
    /// Reference to the connection
    connection: &'a mut (dyn crate::Connection + Send + Sync),
    /// Maximum number of attributes per request
    max_per_request: usize,
    /// Stop on first error flag
    stop_on_error: bool,
}

impl<'a> BatchWriter<'a> {
    /// Create a new batch writer
    ///
    /// # Arguments
    /// * `connection` - Reference to the connection
    pub fn new(connection: &'a mut (dyn crate::Connection + Send + Sync)) -> Self {
        Self {
            connection,
            max_per_request: 10, // Default limit
            stop_on_error: false,
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

    /// Set whether to stop on first error
    ///
    /// # Arguments
    /// * `stop` - If true, stop processing on first error
    pub fn with_stop_on_error(mut self, stop: bool) -> Self {
        self.stop_on_error = stop;
        self
    }

    /// Write multiple attributes in a single request
    ///
    /// # Arguments
    /// * `attributes` - List of attribute values to write
    ///
    /// # Returns
    /// Batch write result with successes and failures
    pub async fn write_attributes(
        &mut self,
        attributes: Vec<AttributeValue>,
    ) -> DlmsResult<BatchWriteResult> {
        let mut successful = Vec::new();
        let mut failed = Vec::new();

        // Split into chunks if exceeding max_per_request
        for chunk in attributes.chunks(self.max_per_request) {
            let invoke_id = InvokeIdAndPriority::new(1, false)?;
            let descriptors: Vec<_> = chunk
                .iter()
                .map(|a| a.to_descriptor())
                .collect();
            let values: Vec<_> = chunk
                .iter()
                .map(|a| a.value.clone())
                .collect();
            let access_selections = vec![None; chunk.len()];

            // Create SetRequest-WithList
            let request = SetRequest::new_with_list(
                invoke_id,
                descriptors,
                access_selections,
                values,
            )?;
            let request_data = request.encode()?;

            // Send request
            let response_data = self.connection.send_request(&request_data, Some(Duration::from_secs(10))).await?;

            // Parse response
            match SetResponse::decode(&response_data)? {
                SetResponse::WithList(with_list) => {
                    for (i, result) in with_list.result_list.iter().enumerate() {
                        let attr = &chunk[i];

                        match result {
                            SetDataResult::Success => {
                                successful.push(AttributeWriteResult {
                                    obis_code: attr.obis_code,
                                    class_id: attr.class_id,
                                    attribute_id: attr.attribute_id,
                                });
                            }
                            SetDataResult::DataAccessResult(code) => {
                                let error = AttributeWriteError {
                                    obis_code: attr.obis_code,
                                    class_id: Some(attr.class_id),
                                    attribute_id: attr.attribute_id,
                                    error: format!("Access error: code {}", code),
                                };
                                failed.push(error);
                                if self.stop_on_error {
                                    return Ok(BatchWriteResult {
                                        successful,
                                        failed,
                                    });
                                }
                            }
                        }
                    }
                }
                SetResponse::Normal(normal) => {
                    // Single response - this might indicate the server
                    // doesn't support WithList or there was a global error
                    match normal.result {
                        SetDataResult::DataAccessResult(code) => {
                            for attr in chunk {
                                failed.push(AttributeWriteError {
                                    obis_code: attr.obis_code,
                                    class_id: Some(attr.class_id),
                                    attribute_id: attr.attribute_id,
                                    error: format!("Access error: code {}", code),
                                });
                            }
                        }
                        SetDataResult::Success => {
                            // Assume first succeeded if we got success
                            if let Some(attr) = chunk.first() {
                                successful.push(AttributeWriteResult {
                                    obis_code: attr.obis_code,
                                    class_id: attr.class_id,
                                    attribute_id: attr.attribute_id,
                                });
                            }
                        }
                    }
                }
                SetResponse::WithDataBlock { .. } => {
                    // Block transfer response - not expected for WithList
                    for attr in chunk {
                        failed.push(AttributeWriteError {
                            obis_code: attr.obis_code,
                            class_id: Some(attr.class_id),
                            attribute_id: attr.attribute_id,
                            error: "Unexpected block transfer response".to_string(),
                        });
                    }
                }
            }
        }

        // Fallback: write remaining attributes individually
        // This handles cases where the server doesn't support WithList
        for attr in attributes {
            let was_written = successful.iter().any(|r| {
                r.obis_code == attr.obis_code && r.attribute_id == attr.attribute_id
            });
            let had_error = failed.iter().any(|e| {
                e.obis_code == attr.obis_code && e.attribute_id == attr.attribute_id
            });

            if !was_written && !had_error {
                match self
                    .connection
                    .set_attribute(attr.obis_code, attr.class_id, attr.attribute_id, attr.value)
                    .await
                {
                    Ok(_) => {
                        successful.push(AttributeWriteResult {
                            obis_code: attr.obis_code,
                            class_id: attr.class_id,
                            attribute_id: attr.attribute_id,
                        });
                    }
                    Err(e) => {
                        failed.push(AttributeWriteError {
                            obis_code: attr.obis_code,
                            class_id: Some(attr.class_id),
                            attribute_id: attr.attribute_id,
                            error: e.to_string(),
                        });
                        if self.stop_on_error {
                            return Ok(BatchWriteResult {
                                successful,
                                failed,
                            });
                        }
                    }
                }
            }
        }

        Ok(BatchWriteResult {
            successful,
            failed,
        })
    }

    /// Write all specified attributes and return success/failure counts
    ///
    /// This is a convenience method that returns only the counts.
    ///
    /// # Arguments
    /// * `attributes` - List of attribute values to write
    ///
    /// # Returns
    /// Tuple of (success_count, failure_count)
    pub async fn write_attributes_count(
        &mut self,
        attributes: Vec<AttributeValue>,
    ) -> DlmsResult<(usize, usize)> {
        let result = self.write_attributes(attributes).await?;
        Ok((result.successful.len(), result.failed.len()))
    }

    /// Write multiple attributes on the same object
    ///
    /// # Arguments
    /// * `obis` - OBIS code of the object
    /// * `class_id` - Class ID of the object
    /// * `attributes` - List of (attribute_id, value) tuples to write
    ///
    /// # Returns
    /// Batch write result
    pub async fn write_object_attributes(
        &mut self,
        obis: ObisCode,
        class_id: u16,
        attributes: Vec<(u8, DataObject)>,
    ) -> DlmsResult<BatchWriteResult> {
        let attr_values: Vec<_> = attributes
            .into_iter()
            .map(|(id, value)| AttributeValue::new(obis, class_id, id, value))
            .collect();

        self.write_attributes(attr_values).await
    }

    /// Write the same attribute on multiple objects
    ///
    /// # Arguments
    /// * `objects` - List of (obis_code, class_id) tuples
    /// * `attribute_id` - Attribute ID to write on each object
    /// * `value` - Value to write
    ///
    /// # Returns
    /// Batch write result
    pub async fn write_same_attribute_on_objects(
        &mut self,
        objects: Vec<(ObisCode, u16)>,
        attribute_id: u8,
        value: DataObject,
    ) -> DlmsResult<BatchWriteResult> {
        let attr_values: Vec<_> = objects
            .into_iter()
            .map(|(obis, class_id)| AttributeValue::new(obis, class_id, attribute_id, value.clone()))
            .collect();

        self.write_attributes(attr_values).await
    }

    /// Write attributes and verify all succeeded
    ///
    /// # Arguments
    /// * `attributes` - List of attribute values to write
    ///
    /// # Returns
    /// Ok(()) if all writes succeeded, Err with details of any failures
    pub async fn write_attributes_all(
        &mut self,
        attributes: Vec<AttributeValue>,
    ) -> DlmsResult<()> {
        let result = self.write_attributes(attributes).await?;

        if result.failed.is_empty() {
            Ok(())
        } else {
            let error_msg = result.failed
                .iter()
                .map(|e| format!("{}:{}: {}", e.obis_code, e.attribute_id, e.error))
                .collect::<Vec<_>>()
                .join("; ");

            Err(dlms_core::DlmsError::InvalidData(format!(
                "Batch write failed: {}",
                error_msg
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_value() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let value = DataObject::Unsigned8(42);
        let attr = AttributeValue::new(obis, 3, 2, value);

        assert_eq!(attr.obis_code, obis);
        assert_eq!(attr.class_id, 3);
        assert_eq!(attr.attribute_id, 2);
    }

    #[test]
    fn test_batch_write_result() {
        let result = BatchWriteResult {
            successful: vec![],
            failed: vec![],
        };

        assert!(result.successful.is_empty());
        assert!(result.failed.is_empty());
    }

    #[test]
    fn test_attribute_write_result() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let result = AttributeWriteResult {
            obis_code: obis,
            class_id: 3,
            attribute_id: 2,
        };

        assert_eq!(result.obis_code, obis);
        assert_eq!(result.class_id, 3);
        assert_eq!(result.attribute_id, 2);
    }

    #[test]
    fn test_attribute_write_error() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let error = AttributeWriteError {
            obis_code: obis,
            class_id: Some(3),
            attribute_id: 2,
            error: "Test error".to_string(),
        };

        assert_eq!(error.obis_code, obis);
        assert_eq!(error.class_id, Some(3));
        assert_eq!(error.attribute_id, 2);
        assert_eq!(error.error, "Test error");
    }
}
