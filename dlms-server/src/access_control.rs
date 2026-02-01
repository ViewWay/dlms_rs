//! Access Control List (ACL) for DLMS/COSEM server
//!
//! This module provides fine-grained access control for COSEM objects.
//! It implements:
//! - Permission management (GET, SET, ACTION)
//! - Role-based access control
//! - Object-level and attribute-level permissions
//! - Method invocation permissions

use dlms_core::{DlmsError, DlmsResult, ObisCode};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Access permission for a single operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessPermission {
    /// Access denied
    Denied,
    /// Read-only access (GET)
    ReadOnly,
    /// Write access (SET)
    Write,
    /// Full access (GET, SET, ACTION)
    Full,
}

impl AccessPermission {
    /// Check if read access is granted
    pub fn can_read(&self) -> bool {
        matches!(self, Self::ReadOnly | Self::Write | Self::Full)
    }

    /// Check if write access is granted
    pub fn can_write(&self) -> bool {
        matches!(self, Self::Write | Self::Full)
    }

    /// Check if action invocation is granted
    pub fn can_execute(&self) -> bool {
        matches!(self, Self::Full)
    }
}

/// Access rule for a specific object/attribute
#[derive(Debug, Clone)]
pub struct AccessRule {
    /// Permission for this rule
    pub permission: AccessPermission,
    /// Optional description
    pub description: Option<String>,
}

impl AccessRule {
    /// Create a new access rule
    pub fn new(permission: AccessPermission) -> Self {
        Self {
            permission,
            description: None,
        }
    }

    /// Create a new access rule with description
    pub fn with_description(permission: AccessPermission, description: String) -> Self {
        Self {
            permission,
            description: Some(description),
        }
    }

    /// Create a denied rule
    pub fn denied() -> Self {
        Self::new(AccessPermission::Denied)
    }

    /// Create a read-only rule
    pub fn read_only() -> Self {
        Self::new(AccessPermission::ReadOnly)
    }

    /// Create a write rule
    pub fn write() -> Self {
        Self::new(AccessPermission::Write)
    }

    /// Create a full access rule
    pub fn full() -> Self {
        Self::new(AccessPermission::Full)
    }
}

/// Access control entry key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AclKey {
    /// Object-level permission (applies to all attributes)
    Object(ObisCode),
    /// Attribute-level permission (applies to specific attribute)
    Attribute { obis: ObisCode, attribute_id: u8 },
    /// Method-level permission (applies to specific method)
    Method { obis: ObisCode, method_id: u8 },
}

/// Access Control List for a client
///
/// Manages permissions for a specific client (identified by client SAP).
#[derive(Debug, Clone)]
pub struct AccessControlList {
    /// Client SAP address
    client_sap: u16,
    /// Access rules indexed by key
    rules: HashMap<AclKey, AccessRule>,
    /// Default permission when no rule matches
    default_permission: AccessPermission,
}

impl AccessControlList {
    /// Create a new ACL for a client
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    /// * `default_permission` - Default permission when no rule matches
    pub fn new(client_sap: u16, default_permission: AccessPermission) -> Self {
        Self {
            client_sap,
            rules: HashMap::new(),
            default_permission,
        }
    }

    /// Create an ACL with deny-all default
    pub fn deny_all(client_sap: u16) -> Self {
        Self::new(client_sap, AccessPermission::Denied)
    }

    /// Create an ACL with read-only default
    pub fn read_only_default(client_sap: u16) -> Self {
        Self::new(client_sap, AccessPermission::ReadOnly)
    }

    /// Create an ACL with full-access default
    pub fn full_access_default(client_sap: u16) -> Self {
        Self::new(client_sap, AccessPermission::Full)
    }

    /// Get the client SAP
    pub fn client_sap(&self) -> u16 {
        self.client_sap
    }

    /// Add a rule for an object
    ///
    /// # Arguments
    /// * `obis` - OBIS code of the object
    /// * `rule` - Access rule
    pub fn add_object_rule(&mut self, obis: ObisCode, rule: AccessRule) {
        self.rules.insert(AclKey::Object(obis), rule);
    }

    /// Add a rule for an attribute
    ///
    /// # Arguments
    /// * `obis` - OBIS code of the object
    /// * `attribute_id` - Attribute ID
    /// * `rule` - Access rule
    pub fn add_attribute_rule(&mut self, obis: ObisCode, attribute_id: u8, rule: AccessRule) {
        self.rules.insert(AclKey::Attribute { obis, attribute_id }, rule);
    }

    /// Add a rule for a method
    ///
    /// # Arguments
    /// * `obis` - OBIS code of the object
    /// * `method_id` - Method ID
    /// * `rule` - Access rule
    pub fn add_method_rule(&mut self, obis: ObisCode, method_id: u8, rule: AccessRule) {
        self.rules.insert(AclKey::Method { obis, method_id }, rule);
    }

    /// Set the default permission
    pub fn set_default_permission(&mut self, permission: AccessPermission) {
        self.default_permission = permission;
    }

    /// Check read access for an attribute
    ///
    /// # Arguments
    /// * `obis` - OBIS code of the object
    /// * `attribute_id` - Attribute ID
    ///
    /// # Returns
    /// true if read access is granted, false otherwise
    pub fn can_read(&self, obis: &ObisCode, attribute_id: u8) -> bool {
        self.check_permission(obis, attribute_id, |p| p.can_read())
    }

    /// Check write access for an attribute
    ///
    /// # Arguments
    /// * `obis` - OBIS code of the object
    /// * `attribute_id` - Attribute ID
    ///
    /// # Returns
    /// true if write access is granted, false otherwise
    pub fn can_write(&self, obis: &ObisCode, attribute_id: u8) -> bool {
        self.check_permission(obis, attribute_id, |p| p.can_write())
    }

    /// Check execution access for a method
    ///
    /// # Arguments
    /// * `obis` - OBIS code of the object
    /// * `method_id` - Method ID
    ///
    /// # Returns
    /// true if execution access is granted, false otherwise
    pub fn can_execute(&self, obis: &ObisCode, method_id: u8) -> bool {
        // Check method-specific rule first
        if let Some(rule) = self.rules.get(&AclKey::Method { obis: *obis, method_id }) {
            return rule.permission.can_execute();
        }

        // Check object-level rule
        if let Some(rule) = self.rules.get(&AclKey::Object(*obis)) {
            return rule.permission.can_execute();
        }

        // Fall back to default
        self.default_permission.can_execute()
    }

    /// Get the permission for an attribute
    ///
    /// Returns the effective permission considering attribute-level and object-level rules.
    pub fn get_attribute_permission(&self, obis: &ObisCode, attribute_id: u8) -> AccessPermission {
        // Check attribute-specific rule first
        if let Some(rule) = self.rules.get(&AclKey::Attribute { obis: *obis, attribute_id }) {
            return rule.permission;
        }

        // Check object-level rule
        if let Some(rule) = self.rules.get(&AclKey::Object(*obis)) {
            return rule.permission;
        }

        // Fall back to default
        self.default_permission
    }

    /// Check permission using a predicate
    fn check_permission<F>(&self, obis: &ObisCode, attribute_id: u8, predicate: F) -> bool
    where
        F: Fn(AccessPermission) -> bool,
    {
        let permission = self.get_attribute_permission(obis, attribute_id);
        predicate(permission)
    }

    /// Remove a rule
    ///
    /// # Arguments
    /// * `key` - ACL key to remove
    ///
    /// # Returns
    /// true if a rule was removed, false otherwise
    pub fn remove_rule(&mut self, key: &AclKey) -> bool {
        self.rules.remove(key).is_some()
    }

    /// Get all rules
    pub fn rules(&self) -> &HashMap<AclKey, AccessRule> {
        &self.rules
    }
}

/// Access Control Manager
///
/// Manages ACLs for all clients and performs access control checks.
pub struct AccessControlManager {
    /// ACLs indexed by client SAP
    acls: Arc<RwLock<HashMap<u16, AccessControlList>>>,
    /// Whether access control is enabled
    enabled: Arc<RwLock<bool>>,
    /// Default ACL for clients without explicit ACL
    default_acl: Arc<RwLock<Option<AccessControlList>>>,
}

impl AccessControlManager {
    /// Create a new access control manager
    pub fn new() -> Self {
        Self {
            acls: Arc::new(RwLock::new(HashMap::new())),
            enabled: Arc::new(RwLock::new(true)),
            default_acl: Arc::new(RwLock::new(None)),
        }
    }

    /// Check if access control is enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }

    /// Enable or disable access control
    pub async fn set_enabled(&self, enabled: bool) {
        *self.enabled.write().await = enabled;
    }

    /// Set the default ACL for clients without explicit ACL
    pub async fn set_default_acl(&self, acl: AccessControlList) {
        *self.default_acl.write().await = Some(acl);
    }

    /// Clear the default ACL
    pub async fn clear_default_acl(&self) {
        *self.default_acl.write().await = None;
    }

    /// Register an ACL for a client
    ///
    /// # Arguments
    /// * `acl` - Access control list
    pub async fn register_acl(&self, acl: AccessControlList) {
        let mut acls = self.acls.write().await;
        acls.insert(acl.client_sap(), acl);
    }

    /// Unregister an ACL for a client
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    pub async fn unregister_acl(&self, client_sap: u16) {
        let mut acls = self.acls.write().await;
        acls.remove(&client_sap);
    }

    /// Get ACL for a client
    ///
    /// Returns the client's ACL or the default ACL if none exists.
    async fn get_acl(&self, client_sap: u16) -> Option<AccessControlList> {
        let acls = self.acls.read().await;
        if let Some(acl) = acls.get(&client_sap) {
            Some(acl.clone())
        } else {
            let default = self.default_acl.read().await;
            default.clone()
        }
    }

    /// Check read access
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    /// * `obis` - OBIS code of the object
    /// * `attribute_id` - Attribute ID
    ///
    /// # Returns
    /// Ok(true) if access is granted, Ok(false) if denied
    /// Err if access control is enabled but no ACL exists
    pub async fn check_read_access(
        &self,
        client_sap: u16,
        obis: &ObisCode,
        attribute_id: u8,
    ) -> DlmsResult<bool> {
        if !self.is_enabled().await {
            return Ok(true); // Access control disabled, allow all
        }

        let acl = self.get_acl(client_sap).await.ok_or_else(|| {
            DlmsError::AccessDenied(format!(
                "No ACL found for client {} and no default ACL configured",
                client_sap
            ))
        })?;

        Ok(acl.can_read(obis, attribute_id))
    }

    /// Check write access
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    /// * `obis` - OBIS code of the object
    /// * `attribute_id` - Attribute ID
    ///
    /// # Returns
    /// Ok(true) if access is granted, Ok(false) if denied
    /// Err if access control is enabled but no ACL exists
    pub async fn check_write_access(
        &self,
        client_sap: u16,
        obis: &ObisCode,
        attribute_id: u8,
    ) -> DlmsResult<bool> {
        if !self.is_enabled().await {
            return Ok(true); // Access control disabled, allow all
        }

        let acl = self.get_acl(client_sap).await.ok_or_else(|| {
            DlmsError::AccessDenied(format!(
                "No ACL found for client {} and no default ACL configured",
                client_sap
            ))
        })?;

        Ok(acl.can_write(obis, attribute_id))
    }

    /// Check method execution access
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    /// * `obis` - OBIS code of the object
    /// * `method_id` - Method ID
    ///
    /// # Returns
    /// Ok(true) if access is granted, Ok(false) if denied
    /// Err if access control is enabled but no ACL exists
    pub async fn check_execute_access(
        &self,
        client_sap: u16,
        obis: &ObisCode,
        method_id: u8,
    ) -> DlmsResult<bool> {
        if !self.is_enabled().await {
            return Ok(true); // Access control disabled, allow all
        }

        let acl = self.get_acl(client_sap).await.ok_or_else(|| {
            DlmsError::AccessDenied(format!(
                "No ACL found for client {} and no default ACL configured",
                client_sap
            ))
        })?;

        Ok(acl.can_execute(obis, method_id))
    }

    /// Require read access (returns error if denied)
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    /// * `obis` - OBIS code of the object
    /// * `attribute_id` - Attribute ID
    ///
    /// # Errors
    /// Returns DlmsError::AccessDenied if access is denied
    pub async fn require_read_access(
        &self,
        client_sap: u16,
        obis: &ObisCode,
        attribute_id: u8,
    ) -> DlmsResult<()> {
        if !self.check_read_access(client_sap, obis, attribute_id).await? {
            return Err(DlmsError::AccessDenied(format!(
                "Read access denied for client {} on {} attribute {}",
                client_sap, obis, attribute_id
            )));
        }
        Ok(())
    }

    /// Require write access (returns error if denied)
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    /// * `obis` - OBIS code of the object
    /// * `attribute_id` - Attribute ID
    ///
    /// # Errors
    /// Returns DlmsError::AccessDenied if access is denied
    pub async fn require_write_access(
        &self,
        client_sap: u16,
        obis: &ObisCode,
        attribute_id: u8,
    ) -> DlmsResult<()> {
        if !self.check_write_access(client_sap, obis, attribute_id).await? {
            return Err(DlmsError::AccessDenied(format!(
                "Write access denied for client {} on {} attribute {}",
                client_sap, obis, attribute_id
            )));
        }
        Ok(())
    }

    /// Require execution access (returns error if denied)
    ///
    /// # Arguments
    /// * `client_sap` - Client SAP address
    /// * `obis` - OBIS code of the object
    /// * `method_id` - Method ID
    ///
    /// # Errors
    /// Returns DlmsError::AccessDenied if access is denied
    pub async fn require_execute_access(
        &self,
        client_sap: u16,
        obis: &ObisCode,
        method_id: u8,
    ) -> DlmsResult<()> {
        if !self.check_execute_access(client_sap, obis, method_id).await? {
            return Err(DlmsError::AccessDenied(format!(
                "Execution access denied for client {} on {} method {}",
                client_sap, obis, method_id
            )));
        }
        Ok(())
    }

    /// Get all registered client SAPs
    pub async fn get_registered_clients(&self) -> Vec<u16> {
        let acls = self.acls.read().await;
        acls.keys().copied().collect()
    }

    /// Get ACL count
    pub async fn acl_count(&self) -> usize {
        let acls = self.acls.read().await;
        acls.len()
    }
}

impl Default for AccessControlManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_obis() -> ObisCode {
        ObisCode::new(1, 1, 1, 8, 0, 255)
    }

    #[test]
    fn test_access_permission() {
        assert!(!AccessPermission::Denied.can_read());
        assert!(!AccessPermission::Denied.can_write());
        assert!(!AccessPermission::Denied.can_execute());

        assert!(AccessPermission::ReadOnly.can_read());
        assert!(!AccessPermission::ReadOnly.can_write());
        assert!(!AccessPermission::ReadOnly.can_execute());

        assert!(AccessPermission::Write.can_read());
        assert!(AccessPermission::Write.can_write());
        assert!(!AccessPermission::Write.can_execute());

        assert!(AccessPermission::Full.can_read());
        assert!(AccessPermission::Full.can_write());
        assert!(AccessPermission::Full.can_execute());
    }

    #[test]
    fn test_acl_deny_all() {
        let acl = AccessControlList::deny_all(1);
        let obis = create_test_obis();

        assert!(!acl.can_read(&obis, 2));
        assert!(!acl.can_write(&obis, 2));
        assert!(!acl.can_execute(&obis, 1));
    }

    #[test]
    fn test_acl_read_only_default() {
        let acl = AccessControlList::read_only_default(1);
        let obis = create_test_obis();

        assert!(acl.can_read(&obis, 2));
        assert!(!acl.can_write(&obis, 2));
        assert!(!acl.can_execute(&obis, 1));
    }

    #[test]
    fn test_acl_full_access_default() {
        let acl = AccessControlList::full_access_default(1);
        let obis = create_test_obis();

        assert!(acl.can_read(&obis, 2));
        assert!(acl.can_write(&obis, 2));
        assert!(acl.can_execute(&obis, 1));
    }

    #[test]
    fn test_acl_object_rule() {
        let mut acl = AccessControlList::deny_all(1);
        let obis = create_test_obis();

        // Initially denied
        assert!(!acl.can_read(&obis, 2));

        // Add object-level read-only rule
        acl.add_object_rule(obis, AccessRule::read_only());

        // Now should be readable but not writable
        assert!(acl.can_read(&obis, 2));
        assert!(!acl.can_write(&obis, 2));
    }

    #[test]
    fn test_acl_attribute_override() {
        let mut acl = AccessControlList::read_only_default(1);
        let obis = create_test_obis();

        // Default: read-only
        assert!(acl.can_read(&obis, 2));
        assert!(!acl.can_write(&obis, 2));

        // Attribute 2 gets write access
        acl.add_attribute_rule(obis, 2, AccessRule::write());

        // Attribute 2 should be writable
        assert!(acl.can_write(&obis, 2));

        // Attribute 3 still read-only
        assert!(!acl.can_write(&obis, 3));
    }

    #[test]
    fn test_acl_method_rule() {
        let mut acl = AccessControlList::read_only_default(1);
        let obis = create_test_obis();

        // Read-only default can't execute methods
        assert!(!acl.can_execute(&obis, 1));

        // Add method rule
        acl.add_method_rule(obis, 1, AccessRule::full());

        // Now method 1 can be executed
        assert!(acl.can_execute(&obis, 1));

        // But method 2 still can't
        assert!(!acl.can_execute(&obis, 2));
    }

    #[tokio::test]
    async fn test_access_control_manager() {
        let manager = AccessControlManager::new();
        let obis = create_test_obis();

        // Disabled = allow all
        manager.set_enabled(false).await;
        assert!(manager.check_read_access(1, &obis, 2).await.unwrap());

        // Enable without default ACL = deny unknown clients
        manager.set_enabled(true).await;
        assert!(manager.check_read_access(1, &obis, 2).await.is_err());

        // Set default ACL
        let default_acl = AccessControlList::read_only_default(0);
        manager.set_default_acl(default_acl).await;
        assert!(manager.check_read_access(1, &obis, 2).await.unwrap());
        assert!(!manager.check_write_access(1, &obis, 2).await.unwrap());

        // Register specific client ACL
        let client_acl = AccessControlList::full_access_default(1);
        manager.register_acl(client_acl).await;
        assert!(manager.check_write_access(1, &obis, 2).await.unwrap());
    }

    #[tokio::test]
    async fn test_require_access() {
        let manager = AccessControlManager::new();
        let obis = create_test_obis();

        // Register read-only ACL
        let acl = AccessControlList::read_only_default(1);
        manager.register_acl(acl).await;

        // Read should succeed
        assert!(manager.require_read_access(1, &obis, 2).await.is_ok());

        // Write should fail
        assert!(manager.require_write_access(1, &obis, 2).await.is_err());
    }
}
