//! COSEM Association LN interface class (Class ID: 15)
//!
//! This interface class manages logical name addressing associations.
//! It provides access control, user management, and security configuration.

use async_trait::async_trait;
use dlms_application::pdu::SelectiveAccessDescriptor;
use dlms_core::{DataObject, DlmsError, DlmsResult, ObisCode};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    CosemObject, AccessRight, AccessMode, CosemObjectDescriptor, UserInfo, ObisCodeExt,
};

/// Association LN interface class (Class ID: 15)
///
/// Default OBIS: 0-0:40.0.0.255
///
/// This class represents the association between a client and a server
/// using logical name addressing. It contains:
/// - The list of COSEM objects visible in this association
/// - Access rights for different users
/// - Security setup reference
/// - User list with authentication information
#[derive(Debug, Clone)]
pub struct AssociationLn {
    /// Logical name (OBIS code)
    logical_name: ObisCode,

    /// List of COSEM objects accessible through this association
    object_list: Arc<RwLock<Vec<CosemObjectDescriptor>>>,

    /// Access rights list (user_id -> rights mapping)
    access_rights_list: Arc<RwLock<Vec<AccessRight>>>,

    /// Reference to the Security Setup object
    security_setup_reference: ObisCode,

    /// List of users for this association
    user_list: Arc<RwLock<Vec<UserInfo>>>,

    /// xDLMS context information
    xdlms_context_info: Arc<RwLock<Vec<u8>>>,

    /// Authentication mechanism name (OID)
    authentication_mechanism_name: Arc<RwLock<Option<Vec<u8>>>>,

    /// Secret (password/key for authentication)
    secret: Arc<RwLock<Option<Vec<u8>>>>,
}

impl AssociationLn {
    /// Class ID for Association LN
    pub const CLASS_ID: u16 = 15;

    /// Default OBIS code for Association LN (0-0:40.0.0.255)
    /// Note: Use `default_obis()` method to get the default OBIS code
    pub fn default_obis() -> ObisCode {
        ObisCode::new(0, 0, 40, 0, 0, 255)
    }

    /// Attribute IDs
    pub const ATTR_LOGICAL_NAME: u8 = 1;
    pub const ATTR_OBJECT_LIST: u8 = 2;
    pub const ATTR_ACCESS_RIGHTS_LIST: u8 = 3;
    pub const ATTR_SECURITY_SETUP_REFERENCE: u8 = 4;
    pub const ATTR_USER_LIST: u8 = 5;
    pub const ATTR_XDLMS_CONTEXT_INFO: u8 = 6;
    pub const ATTR_AUTHENTICATION_MECHANISM_NAME: u8 = 7;
    pub const ATTR_SECRET: u8 = 8;

    /// Create a new Association LN object
    pub fn new(logical_name: ObisCode) -> Self {
        Self {
            logical_name,
            object_list: Arc::new(RwLock::new(Vec::new())),
            access_rights_list: Arc::new(RwLock::new(Vec::new())),
            security_setup_reference: ObisCode::new(0, 0, 64, 0, 0, 255),
            user_list: Arc::new(RwLock::new(Vec::new())),
            xdlms_context_info: Arc::new(RwLock::new(Vec::new())),
            authentication_mechanism_name: Arc::new(RwLock::new(None)),
            secret: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with default OBIS code
    pub fn with_default_obis() -> Self {
        Self::new(Self::default_obis())
    }

    /// Add a COSEM object to the object list
    pub async fn add_object(&self, descriptor: CosemObjectDescriptor) {
        let mut list = self.object_list.write().await;
        // Avoid duplicates
        if !list.iter().any(|d| {
            d.class_id == descriptor.class_id && d.logical_name == descriptor.logical_name
        }) {
            list.push(descriptor);
        }
    }

    /// Remove a COSEM object from the object list
    pub async fn remove_object(&self, class_id: u16, logical_name: ObisCode) -> bool {
        let mut list = self.object_list.write().await;
        let pos = list.iter().position(|d| {
            d.class_id == class_id && d.logical_name == logical_name
        });
        if let Some(idx) = pos {
            list.remove(idx);
            true
        } else {
            false
        }
    }

    /// Get the object list
    pub async fn get_objects(&self) -> Vec<CosemObjectDescriptor> {
        self.object_list.read().await.clone()
    }

    /// Check if an object exists in the list
    pub async fn has_object(&self, class_id: u16, logical_name: ObisCode) -> bool {
        let list = self.object_list.read().await;
        list.iter().any(|d| d.class_id == class_id && d.logical_name == logical_name)
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
        _class_id: u16,
        _logical_name: ObisCode,
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

    /// Set the xDLMS context info
    pub async fn set_xdlms_context_info(&self, info: Vec<u8>) {
        let mut ctx = self.xdlms_context_info.write().await;
        *ctx = info;
    }

    /// Set the authentication mechanism name (OID)
    pub async fn set_authentication_mechanism_name(&self, name: Option<Vec<u8>>) {
        let mut auth_name = self.authentication_mechanism_name.write().await;
        *auth_name = name;
    }

    /// Set the secret (password/key)
    pub async fn set_secret(&self, secret: Option<Vec<u8>>) {
        let mut sec = self.secret.write().await;
        *sec = secret;
    }

    /// Get the secret
    pub async fn get_secret(&self) -> Option<Vec<u8>> {
        self.secret.read().await.clone()
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

    /// Encode the object list as a DataObject (array of structures)
    async fn encode_object_list(&self) -> DataObject {
        let list = self.object_list.read().await;
        let mut objects = Vec::new();

        for desc in list.iter() {
            // Each object is a structure: [class_id, logical_name, version]
            let mut object_fields = Vec::new();
            object_fields.push(DataObject::Unsigned16(desc.class_id));
            object_fields.push(DataObject::OctetString(desc.logical_name.to_bytes().to_vec()));
            object_fields.push(DataObject::Unsigned8(desc.version));
            objects.push(DataObject::Structure(object_fields));
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
impl CosemObject for AssociationLn {
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
            Self::ATTR_XDLMS_CONTEXT_INFO => {
                let ctx = self.xdlms_context_info.read().await;
                Ok(DataObject::OctetString(ctx.clone()))
            }
            Self::ATTR_AUTHENTICATION_MECHANISM_NAME => {
                let auth_name = self.authentication_mechanism_name.read().await;
                if let Some(name) = auth_name.as_ref() {
                    Ok(DataObject::OctetString(name.clone()))
                } else {
                    Ok(DataObject::OctetString(Vec::new()))
                }
            }
            Self::ATTR_SECRET => {
                let secret = self.secret.read().await;
                if let Some(sec) = secret.as_ref() {
                    Ok(DataObject::OctetString(sec.clone()))
                } else {
                    Ok(DataObject::OctetString(Vec::new()))
                }
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
                if let DataObject::OctetString(bytes) = value {
                    if bytes.len() == 6 {
                        let _obis = ObisCode::from_bytes(&bytes)?;
                        // This is a mutable reference issue - we need a different approach
                        // For now, return an error indicating this needs to be set differently
                        return Err(DlmsError::InvalidData(
                            "Security setup reference must be set via method".to_string(),
                        ));
                    }
                }
                Err(DlmsError::InvalidData(
                    "Invalid OBIS code format".to_string(),
                ))
            }
            Self::ATTR_USER_LIST => {
                // User list is managed through specific methods
                Err(DlmsError::AccessDenied(format!(
                    "Attribute {} is read-only",
                    attribute_id
                )))
            }
            Self::ATTR_XDLMS_CONTEXT_INFO => {
                if let DataObject::OctetString(bytes) = value {
                    let mut ctx = self.xdlms_context_info.write().await;
                    *ctx = bytes;
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected octet string for xDLMS context info".to_string(),
                    ))
                }
            }
            Self::ATTR_AUTHENTICATION_MECHANISM_NAME => {
                if let DataObject::OctetString(bytes) = value {
                    let mut auth_name = self.authentication_mechanism_name.write().await;
                    if bytes.is_empty() {
                        *auth_name = None;
                    } else {
                        *auth_name = Some(bytes);
                    }
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected octet string for authentication mechanism name".to_string(),
                    ))
                }
            }
            Self::ATTR_SECRET => {
                if let DataObject::OctetString(bytes) = value {
                    let mut secret = self.secret.write().await;
                    if bytes.is_empty() {
                        *secret = None;
                    } else {
                        *secret = Some(bytes);
                    }
                    Ok(())
                } else {
                    Err(DlmsError::InvalidData(
                        "Expected octet string for secret".to_string(),
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
            // Association LN typically doesn't have methods in the standard
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
    async fn test_association_ln_class_id() {
        let assoc = AssociationLn::with_default_obis();
        assert_eq!(assoc.class_id(), 15);
    }

    #[tokio::test]
    async fn test_association_ln_obis_code() {
        let assoc = AssociationLn::with_default_obis();
        assert_eq!(assoc.obis_code(), AssociationLn::default_obis());
    }

    #[tokio::test]
    async fn test_association_ln_add_object() {
        let assoc = AssociationLn::with_default_obis();
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let desc = CosemObjectDescriptor::new(3, obis, 0);

        assoc.add_object(desc).await;

        assert!(assoc.has_object(3, obis).await);
    }

    #[tokio::test]
    async fn test_association_ln_remove_object() {
        let assoc = AssociationLn::with_default_obis();
        let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
        let desc = CosemObjectDescriptor::new(3, obis, 0);

        assoc.add_object(desc).await;
        assert!(assoc.remove_object(3, obis).await);
        assert!(!assoc.has_object(3, obis).await);
    }

    #[tokio::test]
    async fn test_association_ln_add_user() {
        let assoc = AssociationLn::with_default_obis();
        let user = UserInfo::new(1).with_name("Admin".to_string());

        assoc.add_user(user).await.unwrap();
        let retrieved = assoc.get_user(1).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_name, Some("Admin".to_string()));
    }

    #[tokio::test]
    async fn test_association_ln_invalid_user() {
        let assoc = AssociationLn::with_default_obis();
        let user = UserInfo::new(0); // Invalid user ID

        assert!(assoc.add_user(user).await.is_err());
    }

    #[tokio::test]
    async fn test_association_ln_access_rights() {
        let assoc = AssociationLn::with_default_obis();
        let mut rights = AccessRight::new(1);
        rights.add_attribute_right(2, AccessMode::ReadWrite);

        assoc.add_access_right(rights).await;

        let retrieved = assoc.get_access_rights(1).await;
        assert!(retrieved.is_some());
        assert!(retrieved.unwrap().can_access_attribute(2, AccessMode::ReadWrite));
    }

    #[tokio::test]
    async fn test_association_ln_get_logical_name() {
        let assoc = AssociationLn::with_default_obis();
        let result = assoc.get_attribute(1, None).await.unwrap();

        match result {
            DataObject::OctetString(bytes) => {
                assert_eq!(bytes.len(), 6);
                assert_eq!(bytes, AssociationLn::default_obis().to_bytes());
            }
            _ => panic!("Expected OctetString"),
        }
    }

    #[tokio::test]
    async fn test_association_ln_set_secret() {
        let assoc = AssociationLn::with_default_obis();
        let secret = vec![0x31, 0x32, 0x33, 0x34];

        assoc.set_secret(Some(secret.clone())).await;

        let retrieved = assoc.get_secret().await;
        assert_eq!(retrieved, Some(secret));
    }

    #[tokio::test]
    async fn test_association_ln_verify_user() {
        let assoc = AssociationLn::with_default_obis();
        let password = vec![0x31, 0x32, 0x33, 0x34];
        let user = UserInfo::new(1)
            .with_name("Admin".to_string())
            .with_password(password.clone());

        assoc.add_user(user).await.unwrap();

        assert!(assoc.verify_user(1, &password).await);
        assert!(!assoc.verify_user(1, &[0x00, 0x00, 0x00, 0x00]).await);
    }
}
