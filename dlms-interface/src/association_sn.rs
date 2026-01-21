//! COSEM Association SN interface class (Class ID: 12)
//!
//! This interface class manages short name addressing associations.
//! It provides access control, user management, and security configuration
//! using short names (SAPs) instead of logical names (OBIS codes).

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DataObject, DlmsError, DlmsResult, ObisCode};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    CosemObject, AccessRight, AccessMode, UserInfo,
};

/// Short Name (SN) type for SN addressing
///
/// Short names are 16-bit values that identify COSEM objects
/// in short name addressing mode.
pub type ShortName = u16;

/// Association SN interface class (Class ID: 12)
///
/// Default OBIS: 0-0:42.0.0.255
///
/// This class represents the association between a client and a server
/// using short name addressing. It contains:
/// - The list of objects accessible through this association (identified by short names)
/// - Access rights for different users
/// - Security setup reference
/// - User list with authentication information
/// - SAP (Service Access Point) configuration
#[derive(Debug, Clone)]
pub struct AssociationSn {
    /// Logical name (OBIS code) - even though this is SN addressing,
    /// the association object itself still has an OBIS code
    logical_name: ObisCode,

    /// List of short names accessible through this association
    object_list: Arc<RwLock<Vec<ShortName>>>,

    /// Access rights list (user_id -> rights mapping)
    access_rights_list: Arc<RwLock<Vec<AccessRight>>>,

    /// Reference to the Security Setup object
    security_setup_reference: ObisCode,

    /// List of users for this association
    user_list: Arc<RwLock<Vec<UserInfo>>>,

    /// Client SAP (Service Access Point)
    client_sap: u16,

    /// Server SAP (Service Access Point)
    server_sap: u16,

    /// Application context name
    application_context_name: Arc<RwLock<Vec<u8>>>,
}

impl AssociationSn {
    /// Class ID for Association SN
    pub const CLASS_ID: u16 = 12;

    /// Default OBIS code for Association SN (0-0:42.0.0.255)
    /// Note: Use `default_obis()` method to get the default OBIS code
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 42, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_OBJECT_LIST: u8 = 2;
    pub const ATTR_ACCESS_RIGHTS_LIST: u8 = 3;
    pub const ATTR_SECURITY_SETUP_REFERENCE: u8 = 4;
    pub const ATTR_USER_LIST: u8 = 5;
    pub const ATTR_CLIENT_SAP: u8 = 6;
    pub const ATTR_SERVER_SAP: u8 = 7;
    pub const ATTR_APPLICATION_CONTEXT_NAME: u8 = 8;

    /// Create a new Association SN object
    pub fn new(logical_name: ObisCode, client_sap: u16, server_sap: u16) -> Self {
        Self {
            logical_name,
            object_list: Arc::new(RwLock::new(Vec::new())),
            access_rights_list: Arc::new(RwLock::new(Vec::new())),
            security_setup_reference: ObisCode::new(0, 0, 64, 0, 0, 255),
            user_list: Arc::new(RwLock::new(Vec::new())),
            client_sap,
            server_sap,
            application_context_name: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis(client_sap: u16, server_sap: u16) -> Self {
        Self::new(Self::default_obis(), client_sap, server_sap)
    }

    /// Add a short name to the object list
    pub async fn add_object(&self, short_name: ShortName) {
        let mut list = self.object_list.write().await;
        // Avoid duplicates
        if !list.contains(&short_name) {
            list.push(short_name);
        }
    }

    /// Remove a short name from the object list
    pub async fn remove_object(&self, short_name: ShortName) -> bool {
        let mut list = self.object_list.write().await;
        if let Some(pos) = list.iter().position(|&sn| sn == short_name) {
            list.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get the object list
    pub async fn get_objects(&self) -> Vec<ShortName> {
        self.object_list.read().await.clone()
    }

    /// Check if a short name exists in the list
    pub async fn has_object(&self, short_name: ShortName) -> bool {
        let list = self.object_list.read().await;
        list.contains(&short_name)
    }

    /// Add an access right entry
    pub async fn add_access_right(&self, access_right: AccessRight) {
        let mut list = self.access_rights_list.write().await;
        // Remove existing entry for the same user
        list.retain(|ar| ar.user_id != access_right.user_id);
        list.push(access_right);
    }

    /// Get access rights for a specific user
    pub async fn get_access_rights(&self, user_id: u8) -> Option<AccessRight> {
        let list = self.access_rights_list.read().await;
        list.iter().find(|ar| ar.user_id == user_id).cloned()
    }

    /// Check if a user has access to an attribute
    pub async fn can_access_attribute(
        &self,
        user_id: u8,
        attribute_id: u8,
        mode: AccessMode,
    ) -> bool {
        if let Some(rights) = self.get_access_rights(user_id).await {
            rights.can_access_attribute(attribute_id, mode)
        } else {
            false
        }
    }

    /// Add a user to the user list
    pub async fn add_user(&self, user: UserInfo) -> DlmsResult<()> {
        user.validate_user_id()?;
        let mut list = self.user_list.write().await;
        // Remove existing entry for the same user
        list.retain(|u| u.user_id != user.user_id);
        list.push(user);
        Ok(())
    }

    /// Get user information by ID
    pub async fn get_user(&self, user_id: u8) -> Option<UserInfo> {
        let list = self.user_list.read().await;
        list.iter().find(|u| u.user_id == user_id).cloned()
    }

    /// Set the security setup reference
    pub fn set_security_setup_reference(&mut self, reference: ObisCode) {
        self.security_setup_reference = reference;
    }

    /// Get the client SAP
    pub fn client_sap(&self) -> u16 {
        self.client_sap
    }

    /// Get the server SAP
    pub fn server_sap(&self) -> u16 {
        self.server_sap
    }

    /// Set the SAP values
    pub fn set_sap(&mut self, client_sap: u16, server_sap: u16) {
        self.client_sap = client_sap;
        self.server_sap = server_sap;
    }

    /// Set the application context name
    pub async fn set_application_context_name(&self, context: Vec<u8>) {
        let mut ctx = self.application_context_name.write().await;
        *ctx = context;
    }

    /// Get the application context name
    pub async fn get_application_context_name(&self) -> Vec<u8> {
        self.application_context_name.read().await.clone()
    }

    /// Verify user credentials
    pub async fn verify_user(&self, user_id: u8, password: &[u8]) -> bool {
        if let Some(user) = self.get_user(user_id).await {
            if let Some(stored_password) = &user.password {
                return stored_password == password;
            }
        }
        false
    }

    /// Encode the object list as a DataObject (array of unsigned 16-bit integers)
    async fn encode_object_list(&self) -> DataObject {
        let list = self.object_list.read().await;
        let mut objects = Vec::new();

        for &sn in list.iter() {
            objects.push(DataObject::Unsigned16(sn));
        }

        DataObject::Array(objects)
    }

    /// Encode the access rights list as a DataObject (array of structures)
    async fn encode_access_rights_list(&self) -> DataObject {
        let list = self.access_rights_list.read().await;
        let mut rights = Vec::new();

        for ar in list.iter() {
            let mut ar_fields = Vec::new();
            ar_fields.push(DataObject::Unsigned8(ar.user_id));

            // Attribute rights: array of [attribute_id, access_mode]
            let mut attr_rights = Vec::new();
            for (attr_id, mode) in &ar.attribute_rights {
                let mut pair = Vec::new();
                pair.push(DataObject::Unsigned8(*attr_id));
                pair.push(DataObject::Unsigned8(mode.value()));
                attr_rights.push(DataObject::Structure(pair));
            }
            ar_fields.push(DataObject::Array(attr_rights));

            // Method rights: array of [method_id, access_mode]
            let mut method_rights = Vec::new();
            for (method_id, mode) in &ar.method_rights {
                let mut pair = Vec::new();
                pair.push(DataObject::Unsigned8(*method_id));
                pair.push(DataObject::Unsigned8(mode.value()));
                method_rights.push(DataObject::Structure(pair));
            }
            ar_fields.push(DataObject::Array(method_rights));

            rights.push(DataObject::Structure(ar_fields));
        }

        DataObject::Array(rights)
    }

    /// Encode the user list as a DataObject (array of structures)
    async fn encode_user_list(&self) -> DataObject {
        let list = self.user_list.read().await;
        let mut users = Vec::new();

        for user in list.iter() {
            let mut user_fields = Vec::new();
            user_fields.push(DataObject::Unsigned8(user.user_id));

            // User name (optional)
            if let Some(name) = &user.user_name {
                user_fields.push(DataObject::Utf8String(name.as_bytes().to_vec()));
            } else {
                user_fields.push(DataObject::Utf8String(Vec::new()));
            }

            users.push(DataObject::Structure(user_fields));
        }

        DataObject::Array(users)
    }
}

#[async_trait]
impl CosemObject for AssociationSn {
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
            Self::ATTR_LOGICAL_NAME => {
                Ok(DataObject::OctetString(self.logical_name.to_bytes().to_vec()))
            }
            Self::ATTR_OBJECT_LIST => {
                Ok(self.encode_object_list().await)
            }
            Self::ATTR_ACCESS_RIGHTS_LIST => {
                Ok(self.encode_access_rights_list().await)
            }
            Self::ATTR_SECURITY_SETUP_REFERENCE => {
                Ok(DataObject::OctetString(self.security_setup_reference.to_bytes().to_vec()))
            }
            Self::ATTR_USER_LIST => {
                Ok(self.encode_user_list().await)
            }
            Self::ATTR_CLIENT_SAP => {
                Ok(DataObject::Unsigned16(self.client_sap))
            }
            Self::ATTR_SERVER_SAP => {
                Ok(DataObject::Unsigned16(self.server_sap))
            }
            Self::ATTR_APPLICATION_CONTEXT_NAME => {
                let ctx = self.application_context_name.read().await;
                Ok(DataObject::OctetString(ctx.clone()))
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Attribute not supported: {}",
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
            Self::ATTR_LOGICAL_NAME => {
                // Logical name is typically read-only
                Err(DlmsError::AccessDenied(format!(
                    "Attribute {} is read-only",
                    attribute_id
                )))
            }
            Self::ATTR_OBJECT_LIST => {
                // Object list is typically managed by the system
                Err(DlmsError::AccessDenied(format!(
                    "Attribute {} is read-only",
                    attribute_id
                )))
            }
            Self::ATTR_ACCESS_RIGHTS_LIST => {
                // Access rights are managed through specific methods
                Err(DlmsError::AccessDenied(format!(
                    "Attribute {} is read-only",
                    attribute_id
                )))
            }
            Self::ATTR_SECURITY_SETUP_REFERENCE => {
                // Security setup reference is read-only
                Err(DlmsError::AccessDenied(format!(
                    "Attribute {} is read-only",
                    attribute_id
                )))
            }
            Self::ATTR_USER_LIST => {
                // User list is managed through specific methods
                Err(DlmsError::AccessDenied(format!(
                    "Attribute {} is read-only",
                    attribute_id
                )))
            }
            Self::ATTR_CLIENT_SAP => {
                if let DataObject::Unsigned16(_sap) = value {
                    // Note: This would require mutable self, so we return an error for now
                    // In a real implementation, this might need interior mutability
                    Err(DlmsError::InvalidData(
                        "Client SAP must be set during construction".to_string(),
                    ))
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected unsigned 16-bit integer for client SAP".to_string(),
                    ))
                }
            }
            Self::ATTR_SERVER_SAP => {
                if let DataObject::Unsigned16(_sap) = value {
                    // Note: This would require mutable self, so we return an error for now
                    Err(DlmsError::InvalidData(
                        "Server SAP must be set during construction".to_string(),
                    ))
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected unsigned 16-bit integer for server SAP".to_string(),
                    ))
                }
            }
            Self::ATTR_APPLICATION_CONTEXT_NAME => {
                if let DataObject::OctetString(bytes) = value {
                    let mut ctx = self.application_context_name.write().await;
                    *ctx = bytes;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected octet string for application context name".to_string(),
                    ))
                }
            }
            _ => Err(DlmsError::InvalidData(format!(
                "Attribute not supported: {}",
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
        match method_id {
            // Association SN typically doesn't have methods in the standard
            _ => Err(DlmsError::InvalidData(format!(
                "Method not supported: {}",
                method_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_association_sn_class_id() {
        let assoc = AssociationSn::with_default_obis(1, 16);
        assert_eq!(assoc.class_id(), 12);
    }

    #[tokio::test]
    async fn test_association_sn_obis_code() {
        let assoc = AssociationSn::with_default_obis(1, 16);
        assert_eq!(assoc.obis_code(), AssociationSn::default_obis());
    }

    #[tokio::test]
    async fn test_association_sn_sap() {
        let assoc = AssociationSn::with_default_obis(1, 16);
        assert_eq!(assoc.client_sap(), 1);
        assert_eq!(assoc.server_sap(), 16);
    }

    #[tokio::test]
    async fn test_association_sn_add_object() {
        let assoc = AssociationSn::with_default_obis(1, 16);
        let short_name = 0x1000;

        assoc.add_object(short_name).await;

        assert!(assoc.has_object(short_name).await);
    }

    #[tokio::test]
    async fn test_association_sn_remove_object() {
        let assoc = AssociationSn::with_default_obis(1, 16);
        let short_name = 0x1000;

        assoc.add_object(short_name).await;
        assert!(assoc.remove_object(short_name).await);
        assert!(!assoc.has_object(short_name).await);
    }

    #[tokio::test]
    async fn test_association_sn_add_user() {
        let assoc = AssociationSn::with_default_obis(1, 16);
        let user = UserInfo::new(1).with_name("Admin".to_string());

        assoc.add_user(user).await.unwrap();
        let retrieved = assoc.get_user(1).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_name, Some("Admin".to_string()));
    }

    #[tokio::test]
    async fn test_association_sn_invalid_user() {
        let assoc = AssociationSn::with_default_obis(1, 16);
        let user = UserInfo::new(0); // Invalid user ID

        assert!(assoc.add_user(user).await.is_err());
    }

    #[tokio::test]
    async fn test_association_sn_access_rights() {
        let assoc = AssociationSn::with_default_obis(1, 16);
        let mut rights = AccessRight::new(1);
        rights.add_attribute_right(2, AccessMode::ReadWrite);

        assoc.add_access_right(rights).await;

        let retrieved = assoc.get_access_rights(1).await;
        assert!(retrieved.is_some());
        assert!(retrieved.unwrap().can_access_attribute(2, AccessMode::ReadWrite));
    }

    #[tokio::test]
    async fn test_association_sn_get_logical_name() {
        let assoc = AssociationSn::with_default_obis(1, 16);
        let result = assoc.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
                assert_eq!(bytes, AssociationSn::default_obis().to_bytes());
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_association_sn_get_sap() {
        let assoc = AssociationSn::with_default_obis(100, 1);

        let client_result = assoc.get_attribute(6, None).await.unwrap();
        let server_result = assoc.get_attribute(7, None).await.unwrap();

        match client_result {
            DataObject::Unsigned16(sap) => assert_eq!(sap, 100),
            _ => panic!("Expected Unsigned16"),
        }

        match server_result {
            DataObject::Unsigned16(sap) => assert_eq!(sap, 1),
            _ => panic!("Expected Unsigned16"),
        }
    }

    #[tokio::test]
    async fn test_association_sn_verify_user() {
        let assoc = AssociationSn::with_default_obis(1, 16);
        let password = vec![0x31, 0x32, 0x33, 0x34];
        let user = UserInfo::new(1)
            .with_name("Admin".to_string())
            .with_password(password.clone());

        assoc.add_user(user).await.unwrap();

        assert!(assoc.verify_user(1, &password).await);
        assert!(!assoc.verify_user(1, &[0x00, 0x00, 0x00, 0x00]).await);
    }

    #[tokio::test]
    async fn test_association_sn_application_context() {
        let assoc = AssociationSn::with_default_obis(1, 16);
        let context = vec![0x01, 0x02, 0x03];

        assoc.set_application_context_name(context.clone()).await;

        let retrieved = assoc.get_application_context_name().await;
        assert_eq!(retrieved, context);
    }
}
