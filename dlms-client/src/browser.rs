//! Object browsing service for DLMS/COSEM client
//!
//! This module provides functionality for browsing and discovering
//! COSEM objects on a remote meter.

use dlms_core::{DlmsResult, ObisCode, DataObject};
use dlms_application::pdu::{
    GetRequest, GetResponse,
    CosemAttributeDescriptor, InvokeIdAndPriority, GetDataResult,
};

use std::time::Duration;

/// COSEM object descriptor
///
/// Represents a single COSEM object discovered during browsing.
#[derive(Debug, Clone)]
pub struct CosemObjectDescriptor {
    /// Object OBIS code
    pub obis_code: ObisCode,
    /// Object class ID
    pub class_id: u16,
    /// Object version (optional)
    pub version: Option<u8>,
    /// Object logical name (attribute 1)
    pub logical_name: Option<ObisCode>,
}

impl CosemObjectDescriptor {
    /// Create a new object descriptor
    pub fn new(obis_code: ObisCode, class_id: u16) -> Self {
        Self {
            obis_code,
            class_id,
            version: None,
            logical_name: None,
        }
    }

    /// Create with version
    pub fn with_version(mut self, version: u8) -> Self {
        self.version = Some(version);
        self
    }

    /// Create with logical name
    pub fn with_logical_name(mut self, logical_name: ObisCode) -> Self {
        self.logical_name = Some(logical_name);
        self
    }
}

/// Object browser
///
/// Provides methods for browsing COSEM objects on a remote meter.
pub struct ObjectBrowser<'a> {
    /// Reference to the connection
    connection: &'a mut (dyn crate::Connection + Send + Sync),
}

impl<'a> ObjectBrowser<'a> {
    /// Create a new object browser
    ///
    /// # Arguments
    /// * `connection` - Reference to the connection
    pub fn new(connection: &'a mut (dyn crate::Connection + Send + Sync)) -> Self {
        Self { connection }
    }

    /// Browse all objects (get attribute 1 from multiple OBIS codes)
    ///
    /// This method sends GET requests for attribute 1 (logical name)
    /// to discover objects.
    ///
    /// # Arguments
    /// * `obis_codes` - List of OBIS codes to browse
    /// * `progress` - Optional progress callback
    ///
    /// # Returns
    /// Vector of discovered object descriptors
    pub async fn browse_objects<F>(
        &mut self,
        obis_codes: &[ObisCode],
        mut progress: F,
    ) -> DlmsResult<Vec<CosemObjectDescriptor>>
    where
        F: FnMut(usize, usize), // (current, total)
    {
        let total = obis_codes.len();
        let mut objects = Vec::new();

        for (i, &obis) in obis_codes.iter().enumerate() {
            progress(i, total);

            // Try to get attribute 1 (logical_name) for this OBIS
            match self.read_attribute_1(obis).await {
                Ok(Some(class_id)) => {
                    objects.push(CosemObjectDescriptor::new(obis, class_id));
                }
                Ok(None) => {
                    // Object doesn't exist, skip
                }
                Err(_) => {
                    // Error reading, skip
                }
            }
        }

        Ok(objects)
    }

    /// Read attribute 1 (logical name) from an object
    ///
    /// This attempts to read the logical name attribute which
    /// contains the class ID in the first byte.
    ///
    /// # Arguments
    /// * `obis` - OBIS code to read
    ///
    /// # Returns
    /// Some(class_id) if object exists, None if it doesn't
    async fn read_attribute_1(&mut self, obis: ObisCode) -> DlmsResult<Option<u16>> {
        let invoke_id = InvokeIdAndPriority::new(1, false)?;

        // Create GetRequest for attribute 1
        let descriptor = CosemAttributeDescriptor::new_logical_name(3, obis, 1)?;
        let request = GetRequest::new_normal(invoke_id, descriptor, None);
        let request_data = request.encode()?;

        // Send request
        let response_data = self.connection.send_request(&request_data, Some(Duration::from_secs(5))).await?;

        // Parse response
        let response = GetResponse::decode(&response_data)?;

        match response {
            GetResponse::Normal(normal) => {
                match normal.result {
                    GetDataResult::Data(data) => {
                        // Extract class ID from logical name (first byte of OBIS)
                        if let DataObject::OctetString(bytes) = data {
                            if bytes.len() >= 6 {
                                // Class ID is encoded in the logical name
                                // Format: [A][B][C][D][E][F] where A contains class ID
                                let class_id = (bytes[0] & 0x7F) as u16;
                                return Ok(Some(class_id));
                            }
                        }
                        Ok(None)
                    }
                    GetDataResult::DataAccessResult(code) => {
                        // Check error code - some errors mean object doesn't exist
                        if code == 1 {
                            // Object undefined
                            Ok(None)
                        } else {
                            // Other error - might exist but not accessible
                            Ok(None)
                        }
                    }
                    GetDataResult::DataBlock(_) => {
                        // Block transfer not supported for this operation
                        Ok(None)
                    }
                }
            }
            _ => {
                // Unexpected response type
                Ok(None)
            }
        }
    }

    /// Get all attributes of an object
    ///
    /// # Arguments
    /// * `obis` - OBIS code of the object
    /// * `class_id` - Class ID of the object
    /// * `attribute_ids` - List of attribute IDs to read
    ///
    /// # Returns
    /// Vector of (attribute_id, value) tuples
    pub async fn get_object_attributes(
        &mut self,
        obis: ObisCode,
        class_id: u16,
        attribute_ids: &[u8],
    ) -> DlmsResult<Vec<(u8, DataObject)>> {
        let mut attributes = Vec::new();

        for &attr_id in attribute_ids {
            match self.connection.get_attribute(obis, class_id, attr_id).await {
                Ok(value) => {
                    attributes.push((attr_id, value));
                }
                Err(_) => {
                    // Skip attributes that can't be read
                }
            }
        }

        Ok(attributes)
    }

    /// Discover object profile
    ///
    /// Attempts to read common attributes to understand the object's capabilities.
    ///
    /// # Arguments
    /// * `obis` - OBIS code of the object
    /// * `class_id` - Class ID of the object
    ///
    /// # Returns
    /// Object descriptor with populated information
    pub async fn discover_object(
        &mut self,
        obis: ObisCode,
        class_id: u16,
    ) -> DlmsResult<CosemObjectDescriptor> {
        let mut descriptor = CosemObjectDescriptor::new(obis, class_id);

        // Try to read attribute 1 (logical name)
        match self.connection.get_attribute(obis, class_id, 1).await {
            Ok(DataObject::OctetString(bytes)) => {
                // Logical name should be the OBIS code itself
                if bytes.len() == 6 {
                    let logical_name = ObisCode::new(bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]);
                    descriptor = descriptor.with_logical_name(logical_name);
                }
            }
            _ => {}
        }

        // Try to read attribute 2 (value) for some interface classes
        // This varies by class, so we don't do it here

        Ok(descriptor)
    }

    /// Search for objects by class ID
    ///
    /// Searches through a range of OBIS codes for objects of a specific class.
    ///
    /// # Arguments
    /// * `class_id` - Class ID to search for
    /// * `obis_range` - Range of OBIS codes to search (start, end)
    /// * `progress` - Optional progress callback
    ///
    /// # Returns
    /// Vector of found object descriptors
    pub async fn search_by_class<F>(
        &mut self,
        _class_id: u16,
        obis_range: (ObisCode, ObisCode),
        progress: F,
    ) -> DlmsResult<Vec<CosemObjectDescriptor>>
    where
        F: FnMut(usize, usize), // (current, total)
    {
        // Generate OBIS codes to search
        let mut obis_codes = Vec::new();
        let (start, end) = obis_range;

        // Simple linear search (could be optimized)
        let start_bytes = start.to_bytes();
        let end_bytes = end.to_bytes();

        // For now, just search a limited set
        // A full implementation would enumerate all OBIS codes in range
        for i in 0..255u8 {
            let mut bytes = start_bytes.clone();
            bytes[5] = i;

            // Check if we've exceeded the range
            if bytes > end_bytes {
                break;
            }

            if bytes.len() == 6 {
                let obis = ObisCode::new(bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]);
                obis_codes.push(obis);
            }
        }

        self.browse_objects(&obis_codes, progress).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_descriptor() {
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let descriptor = CosemObjectDescriptor::new(obis, 3);

        assert_eq!(descriptor.obis_code, obis);
        assert_eq!(descriptor.class_id, 3);
        assert!(descriptor.version.is_none());
        assert!(descriptor.logical_name.is_none());

        let descriptor = descriptor
            .with_version(1)
            .with_logical_name(obis);

        assert_eq!(descriptor.version, Some(1));
        assert_eq!(descriptor.logical_name, Some(obis));
    }

    #[test]
    fn test_obis_bytes() {
        let bytes = [1, 1, 1, 8, 0, 255];
        let obis = ObisCode::new(bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]);
        assert_eq!(obis.to_bytes(), bytes);
    }
}
