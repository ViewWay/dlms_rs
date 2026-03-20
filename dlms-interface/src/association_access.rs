//! Association visibility and per-user access rights (IC15-style object list + user grants).
//!
//! - [`AssociationObjectListEntry`]: one visible object instance and baseline attribute/method modes.
//! - [`UserAccessEntry`] / [`UserObjectAccessGrant`]: per-`user_id` overrides for a specific object.
//! - [`AssociationAccessResolver`]: effective read/write/method checks.
//! - [`CosemInvocationContext`]: passed into [`crate::CosemObject`] on each GET/SET/ACTION.

use crate::descriptor::{AccessMode, CosemObjectDescriptor};
use dlms_core::{DlmsError, DlmsResult, ObisCode};
use std::sync::Arc;
use tokio::sync::RwLock;

/// One object visible in the association, with baseline access for attributes and methods.
///
/// If `attribute_access` is empty, every attribute ID is treated as [`AccessMode::ReadWrite`]
/// for that object. If non-empty, only listed attribute IDs are permitted; others are
/// [`AccessMode::NoAccess`].
///
/// The same rule applies to `method_access` for method invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociationObjectListEntry {
    pub class_id: u16,
    pub logical_name: ObisCode,
    pub version: u8,
    pub attribute_access: Vec<(u8, AccessMode)>,
    pub method_access: Vec<(u8, AccessMode)>,
}

impl AssociationObjectListEntry {
    pub fn new(class_id: u16, logical_name: ObisCode, version: u8) -> Self {
        Self {
            class_id,
            logical_name,
            version,
            attribute_access: Vec::new(),
            method_access: Vec::new(),
        }
    }

    pub fn from_descriptor(desc: CosemObjectDescriptor) -> Self {
        Self {
            class_id: desc.class_id,
            logical_name: desc.logical_name,
            version: desc.version,
            attribute_access: Vec::new(),
            method_access: Vec::new(),
        }
    }
}

impl From<CosemObjectDescriptor> for AssociationObjectListEntry {
    fn from(desc: CosemObjectDescriptor) -> Self {
        Self::from_descriptor(desc)
    }
}

/// Per-user access overlay for one `(class_id, logical_name)` instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserObjectAccessGrant {
    pub class_id: u16,
    pub logical_name: ObisCode,
    /// `None` matches any object version; `Some(v)` requires exact version match.
    pub version: Option<u8>,
    pub attribute_access: Vec<(u8, AccessMode)>,
    pub method_access: Vec<(u8, AccessMode)>,
}

impl UserObjectAccessGrant {
    pub fn new(class_id: u16, logical_name: ObisCode) -> Self {
        Self {
            class_id,
            logical_name,
            version: None,
            attribute_access: Vec::new(),
            method_access: Vec::new(),
        }
    }

    pub fn with_version(mut self, version: u8) -> Self {
        self.version = Some(version);
        self
    }

    pub fn add_attribute_right(&mut self, attribute_id: u8, mode: AccessMode) {
        self.attribute_access.retain(|(id, _)| *id != attribute_id);
        self.attribute_access.push((attribute_id, mode));
    }

    pub fn add_method_right(&mut self, method_id: u8, mode: AccessMode) {
        self.method_access.retain(|(id, _)| *id != method_id);
        self.method_access.push((method_id, mode));
    }
}

/// Access rights for one user (`user_id`), scoped per object instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserAccessEntry {
    pub user_id: u8,
    pub object_grants: Vec<UserObjectAccessGrant>,
}

impl UserAccessEntry {
    pub fn new(user_id: u8) -> Self {
        Self {
            user_id,
            object_grants: Vec::new(),
        }
    }

    pub fn add_object_grant(&mut self, grant: UserObjectAccessGrant) {
        self.object_grants.retain(|g| {
            !(g.class_id == grant.class_id
                && g.logical_name == grant.logical_name
                && g.version == grant.version)
        });
        self.object_grants.push(grant);
    }
}

/// Resolves IC15-style visibility and effective modes (object list + optional user overlay).
#[derive(Debug, Clone)]
pub struct AssociationAccessResolver {
    object_list: Arc<RwLock<Vec<AssociationObjectListEntry>>>,
    user_access_list: Arc<RwLock<Vec<UserAccessEntry>>>,
}

impl AssociationAccessResolver {
    pub fn new(
        object_list: Arc<RwLock<Vec<AssociationObjectListEntry>>>,
        user_access_list: Arc<RwLock<Vec<UserAccessEntry>>>,
    ) -> Self {
        Self {
            object_list,
            user_access_list,
        }
    }

    fn baseline_attribute_mode(entry: &AssociationObjectListEntry, attribute_id: u8) -> AccessMode {
        if entry.attribute_access.is_empty() {
            AccessMode::ReadWrite
        } else {
            entry
                .attribute_access
                .iter()
                .find(|(id, _)| *id == attribute_id)
                .map(|(_, m)| *m)
                .unwrap_or(AccessMode::NoAccess)
        }
    }

    fn baseline_method_mode(entry: &AssociationObjectListEntry, method_id: u8) -> AccessMode {
        if entry.method_access.is_empty() {
            AccessMode::ReadWrite
        } else {
            entry
                .method_access
                .iter()
                .find(|(id, _)| *id == method_id)
                .map(|(_, m)| *m)
                .unwrap_or(AccessMode::NoAccess)
        }
    }

    fn find_user_grant<'a>(
        user_access: &'a [UserAccessEntry],
        user_id: u8,
        class_id: u16,
        logical_name: ObisCode,
        version: u8,
    ) -> Option<&'a UserObjectAccessGrant> {
        let user = user_access.iter().find(|u| u.user_id == user_id)?;
        user.object_grants.iter().find(|g| {
            g.class_id == class_id
                && g.logical_name == logical_name
                && (g.version.is_none() || g.version == Some(version))
        })
    }

    fn overlay_attribute_mode(
        grant: &UserObjectAccessGrant,
        attribute_id: u8,
        baseline: AccessMode,
    ) -> AccessMode {
        if grant.attribute_access.is_empty() {
            baseline
        } else if let Some((_, m)) = grant
            .attribute_access
            .iter()
            .find(|(id, _)| *id == attribute_id)
        {
            *m
        } else {
            baseline
        }
    }

    fn overlay_method_mode(
        grant: &UserObjectAccessGrant,
        method_id: u8,
        baseline: AccessMode,
    ) -> AccessMode {
        if grant.method_access.is_empty() {
            baseline
        } else if let Some((_, m)) = grant.method_access.iter().find(|(id, _)| *id == method_id) {
            *m
        } else {
            baseline
        }
    }

    pub async fn effective_attribute_mode_async(
        &self,
        user_id: Option<u8>,
        class_id: u16,
        logical_name: ObisCode,
        attribute_id: u8,
    ) -> DlmsResult<AccessMode> {
        let objects = self.object_list.read().await;
        if objects.is_empty() {
            return Ok(AccessMode::ReadWrite);
        }
        let entry = objects
            .iter()
            .find(|e| e.class_id == class_id && e.logical_name == logical_name)
            .ok_or_else(|| {
                DlmsError::AccessDenied(format!(
                    "Object not visible in association: class {} {}",
                    class_id, logical_name
                ))
            })?;

        let mut mode = Self::baseline_attribute_mode(entry, attribute_id);
        if let Some(uid) = user_id {
            let users = self.user_access_list.read().await;
            if let Some(grant) =
                Self::find_user_grant(&users, uid, class_id, logical_name, entry.version)
            {
                mode = Self::overlay_attribute_mode(grant, attribute_id, mode);
            }
        }
        Ok(mode)
    }

    pub async fn effective_method_mode_async(
        &self,
        user_id: Option<u8>,
        class_id: u16,
        logical_name: ObisCode,
        method_id: u8,
    ) -> DlmsResult<AccessMode> {
        let objects = self.object_list.read().await;
        if objects.is_empty() {
            return Ok(AccessMode::ReadWrite);
        }
        let entry = objects
            .iter()
            .find(|e| e.class_id == class_id && e.logical_name == logical_name)
            .ok_or_else(|| {
                DlmsError::AccessDenied(format!(
                    "Object not visible in association: class {} {}",
                    class_id, logical_name
                ))
            })?;

        let mut mode = Self::baseline_method_mode(entry, method_id);
        if let Some(uid) = user_id {
            let users = self.user_access_list.read().await;
            if let Some(grant) =
                Self::find_user_grant(&users, uid, class_id, logical_name, entry.version)
            {
                mode = Self::overlay_method_mode(grant, method_id, mode);
            }
        }
        Ok(mode)
    }

    pub async fn require_attribute_read(
        &self,
        user_id: Option<u8>,
        authenticated: bool,
        class_id: u16,
        logical_name: ObisCode,
        attribute_id: u8,
    ) -> DlmsResult<()> {
        let mode = self
            .effective_attribute_mode_async(user_id, class_id, logical_name, attribute_id)
            .await?;
        if !mode.can_read() {
            return Err(DlmsError::AccessDenied(format!(
                "Read denied for attribute {} on {} (class {})",
                attribute_id, logical_name, class_id
            )));
        }
        if mode.requires_auth() && !authenticated {
            return Err(DlmsError::AccessDenied(
                "Authenticated read required".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn require_attribute_write(
        &self,
        user_id: Option<u8>,
        authenticated: bool,
        class_id: u16,
        logical_name: ObisCode,
        attribute_id: u8,
    ) -> DlmsResult<()> {
        let mode = self
            .effective_attribute_mode_async(user_id, class_id, logical_name, attribute_id)
            .await?;
        if !mode.can_write() {
            return Err(DlmsError::AccessDenied(format!(
                "Write denied for attribute {} on {} (class {})",
                attribute_id, logical_name, class_id
            )));
        }
        if mode.requires_auth() && !authenticated {
            return Err(DlmsError::AccessDenied(
                "Authenticated write required".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn require_method_execute(
        &self,
        user_id: Option<u8>,
        authenticated: bool,
        class_id: u16,
        logical_name: ObisCode,
        method_id: u8,
    ) -> DlmsResult<()> {
        let mode = self
            .effective_method_mode_async(user_id, class_id, logical_name, method_id)
            .await?;
        if mode == AccessMode::NoAccess {
            return Err(DlmsError::AccessDenied(format!(
                "Method {} execution denied on {} (class {})",
                method_id, logical_name, class_id
            )));
        }
        if mode.requires_auth() && !authenticated {
            return Err(DlmsError::AccessDenied(
                "Authenticated method invocation required".to_string(),
            ));
        }
        Ok(())
    }
}

/// Carries the active user and association resolver for [`crate::CosemObject`] calls.
#[derive(Clone)]
pub struct CosemInvocationContext {
    pub user_id: Option<u8>,
    pub authenticated: bool,
    pub resolver: Arc<AssociationAccessResolver>,
}

impl CosemInvocationContext {
    pub fn new(user_id: Option<u8>, authenticated: bool, resolver: Arc<AssociationAccessResolver>) -> Self {
        Self {
            user_id,
            authenticated,
            resolver,
        }
    }
}

pub async fn enforce_attribute_read(
    ctx: Option<&CosemInvocationContext>,
    class_id: u16,
    logical_name: ObisCode,
    attribute_id: u8,
) -> DlmsResult<()> {
    let Some(ctx) = ctx else {
        return Ok(());
    };
    ctx.resolver
        .require_attribute_read(
            ctx.user_id,
            ctx.authenticated,
            class_id,
            logical_name,
            attribute_id,
        )
        .await
}

pub async fn enforce_attribute_write(
    ctx: Option<&CosemInvocationContext>,
    class_id: u16,
    logical_name: ObisCode,
    attribute_id: u8,
) -> DlmsResult<()> {
    let Some(ctx) = ctx else {
        return Ok(());
    };
    ctx.resolver
        .require_attribute_write(
            ctx.user_id,
            ctx.authenticated,
            class_id,
            logical_name,
            attribute_id,
        )
        .await
}

pub async fn enforce_method_execute(
    ctx: Option<&CosemInvocationContext>,
    class_id: u16,
    logical_name: ObisCode,
    method_id: u8,
) -> DlmsResult<()> {
    let Some(ctx) = ctx else {
        return Ok(());
    };
    ctx.resolver
        .require_method_execute(
            ctx.user_id,
            ctx.authenticated,
            class_id,
            logical_name,
            method_id,
        )
        .await
}
