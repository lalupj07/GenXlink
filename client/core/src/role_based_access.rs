use crate::permission_profiles::{Permission, PermissionCategory};
use crate::ClientError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Role-based access control system
pub struct RoleBasedAccessControl {
    roles: HashMap<String, Role>,
    user_roles: HashMap<String, Vec<String>>, // user_id -> role_ids
    role_hierarchy: HashMap<String, Vec<String>>, // role_id -> parent_role_ids
    config: RBACConfig,
}

/// RBAC configuration
#[derive(Debug, Clone)]
pub struct RBACConfig {
    /// Enable role inheritance
    pub enable_inheritance: bool,
    /// Maximum role depth for inheritance
    pub max_role_depth: usize,
    /// Enable time-based role assignments
    pub enable_time_based_roles: bool,
    /// Enable dynamic role evaluation
    pub enable_dynamic_roles: bool,
}

impl Default for RBACConfig {
    fn default() -> Self {
        Self {
            enable_inheritance: true,
            max_role_depth: 5,
            enable_time_based_roles: true,
            enable_dynamic_roles: true,
        }
    }
}

/// Role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permissions: HashMap<Permission, PermissionScope>,
    pub metadata: RoleMetadata,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// Permission scope within a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionScope {
    pub allowed: bool,
    pub conditions: Vec<ScopeCondition>,
    pub limitations: Vec<ScopeLimitation>,
}

/// Scope condition for permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScopeCondition {
    TimeRange { start: String, end: String },
    DeviceGroup { group: String },
    Location { allowed_locations: Vec<String> },
    NetworkType { allowed_types: Vec<String> },
}

/// Scope limitation for permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScopeLimitation {
    MaxDuration { minutes: u64 },
    MaxFileSize { mb: u64 },
    RateLimit { requests_per_minute: u32 },
    RequireApproval { approver_roles: Vec<String> },
}

/// Role metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleMetadata {
    pub category: RoleCategory,
    pub priority: u8,
    pub is_system_role: bool,
    pub is_temporary: bool,
    pub expires_at: Option<SystemTime>,
    pub created_by: String,
}

/// Role categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoleCategory {
    System,
    Administrator,
    Operator,
    User,
    Guest,
    Custom,
}

/// User role assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleAssignment {
    pub user_id: String,
    pub role_id: String,
    pub assigned_at: SystemTime,
    pub assigned_by: String,
    pub expires_at: Option<SystemTime>,
    pub conditions: Vec<AssignmentCondition>,
}

/// Assignment condition for user roles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssignmentCondition {
    TimeBased { start: SystemTime, end: SystemTime },
    DeviceSpecific { device_ids: Vec<String> },
    ContextBased { context: HashMap<String, String> },
}

/// Role evaluation context
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub user_id: String,
    pub device_id: String,
    pub session_id: String,
    pub timestamp: SystemTime,
    pub location: Option<String>,
    pub network_type: String,
    pub additional_context: HashMap<String, String>,
}

/// Role evaluation result
#[derive(Debug, Clone)]
pub struct RoleEvaluationResult {
    pub allowed: bool,
    pub roles: Vec<String>,
    pub effective_permissions: Vec<EffectivePermission>,
    pub requires_approval: bool,
    pub limitations: Vec<ScopeLimitation>,
    pub reason: String,
}

/// Effective permission with scope
#[derive(Debug, Clone)]
pub struct EffectivePermission {
    pub permission: Permission,
    pub source_role: String,
    pub scope: PermissionScope,
    pub inherited: bool,
}

impl RoleBasedAccessControl {
    /// Create a new RBAC system
    pub fn new(config: RBACConfig) -> Self {
        let mut rbac = Self {
            roles: HashMap::new(),
            user_roles: HashMap::new(),
            role_hierarchy: HashMap::new(),
            config,
        };

        // Initialize default roles
        rbac.initialize_default_roles();
        rbac
    }

    /// Initialize default system roles
    fn initialize_default_roles(&mut self) {
        let now = SystemTime::now();

        // System Administrator Role
        let mut admin_permissions = HashMap::new();
        for permission in self.get_all_permissions() {
            admin_permissions.insert(permission, PermissionScope {
                allowed: true,
                conditions: Vec::new(),
                limitations: Vec::new(),
            });
        }

        let admin_role = Role {
            id: "system_admin".to_string(),
            name: "System Administrator".to_string(),
            description: "Full system access with all permissions".to_string(),
            permissions: admin_permissions,
            metadata: RoleMetadata {
                category: RoleCategory::System,
                priority: 100,
                is_system_role: true,
                is_temporary: false,
                expires_at: None,
                created_by: "system".to_string(),
            },
            created_at: now,
            updated_at: now,
        };

        // Operator Role
        let operator_permissions = self.get_operator_permissions();
        let operator_role = Role {
            id: "operator".to_string(),
            name: "Operator".to_string(),
            description: "Standard operator with control and file access".to_string(),
            permissions: operator_permissions,
            metadata: RoleMetadata {
                category: RoleCategory::Operator,
                priority: 50,
                is_system_role: true,
                is_temporary: false,
                expires_at: None,
                created_by: "system".to_string(),
            },
            created_at: now,
            updated_at: now,
        };

        // Viewer Role
        let viewer_permissions = self.get_viewer_permissions();
        let viewer_role = Role {
            id: "viewer".to_string(),
            name: "Viewer".to_string(),
            description: "View-only access for monitoring".to_string(),
            permissions: viewer_permissions,
            metadata: RoleMetadata {
                category: RoleCategory::User,
                priority: 25,
                is_system_role: true,
                is_temporary: false,
                expires_at: None,
                created_by: "system".to_string(),
            },
            created_at: now,
            updated_at: now,
        };

        // Guest Role
        let guest_permissions = self.get_guest_permissions();
        let guest_role = Role {
            id: "guest".to_string(),
            name: "Guest".to_string(),
            description: "Limited guest access".to_string(),
            permissions: guest_permissions,
            metadata: RoleMetadata {
                category: RoleCategory::Guest,
                priority: 10,
                is_system_role: true,
                is_temporary: false,
                expires_at: None,
                created_by: "system".to_string(),
            },
            created_at: now,
            updated_at: now,
        };

        // Add roles
        self.roles.insert(admin_role.id.clone(), admin_role);
        self.roles.insert(operator_role.id.clone(), operator_role);
        self.roles.insert(viewer_role.id.clone(), viewer_role);
        self.roles.insert(guest_role.id.clone(), guest_role);

        // Set up role hierarchy (Operator inherits from Viewer, Admin inherits from Operator)
        self.role_hierarchy.insert("operator".to_string(), vec!["viewer".to_string()]);
        self.role_hierarchy.insert("system_admin".to_string(), vec!["operator".to_string()]);
    }

    /// Create a new custom role
    pub fn create_role(&mut self, role: Role) -> Result<(), ClientError> {
        if self.roles.contains_key(&role.id) {
            return Err(ClientError::IoError("Role already exists".to_string()));
        }

        let role_id = role.id.clone();
        self.roles.insert(role_id.clone(), role);
        tracing::info!("Created role: {}", role_id);
        Ok(())
    }

    /// Assign role to user
    pub fn assign_role(&mut self, user_id: String, role_id: String, _assigned_by: String) -> Result<(), ClientError> {
        if !self.roles.contains_key(&role_id) {
            return Err(ClientError::IoError("Role not found".to_string()));
        }

        let user_roles = self.user_roles.entry(user_id.clone()).or_insert_with(Vec::new);
        let role_id_clone = role_id.clone();
        if !user_roles.contains(&role_id) {
            user_roles.push(role_id);
        }

        tracing::info!("Assigned role {} to user {}", role_id_clone, user_id);
        Ok(())
    }

    /// Remove role from user
    pub fn remove_role(&mut self, user_id: &str, role_id: &str) -> Result<(), ClientError> {
        if let Some(user_roles) = self.user_roles.get_mut(user_id) {
            user_roles.retain(|r| r != role_id);
            tracing::info!("Removed role {} from user {}", role_id, user_id);
        }
        Ok(())
    }

    /// Evaluate user permissions based on roles
    pub fn evaluate_permissions(&self, context: &EvaluationContext) -> Result<RoleEvaluationResult, ClientError> {
        let user_roles = self.user_roles.get(&context.user_id)
            .ok_or_else(|| ClientError::IoError("User has no roles assigned".to_string()))?;

        let mut effective_permissions = Vec::new();
        let mut all_roles = Vec::new();
        let mut requires_approval = false;
        let mut limitations = Vec::new();

        // Get all roles including inherited ones
        for role_id in user_roles {
            let mut visited = std::collections::HashSet::new();
            self.collect_roles_recursive(role_id, &mut all_roles, &mut visited, 0)?;
        }

        // Collect permissions from all roles
        for role_id in &all_roles {
            if let Some(role) = self.roles.get(role_id) {
                for (permission, scope) in &role.permissions {
                    if scope.allowed {
                        // Check if permission is already granted by a higher priority role
                        if !effective_permissions.iter().any(|ep: &EffectivePermission| ep.permission == *permission) {
                            effective_permissions.push(EffectivePermission {
                                permission: permission.clone(),
                                source_role: role_id.clone(),
                                scope: scope.clone(),
                                inherited: role_id != user_roles.iter().find(|&r| r == role_id).unwrap_or(&String::new()),
                            });
                        }

                        // Check for approval requirements
                        for limitation in &scope.limitations {
                            if let ScopeLimitation::RequireApproval { .. } = limitation {
                                requires_approval = true;
                            }
                            limitations.push(limitation.clone());
                        }
                    }
                }
            }
        }

        Ok(RoleEvaluationResult {
            allowed: !effective_permissions.is_empty(),
            roles: all_roles,
            effective_permissions,
            requires_approval,
            limitations,
            reason: "Permissions evaluated from assigned roles".to_string(),
        })
    }

    /// Check if user has specific permission
    pub fn has_permission(&self, _user_id: &str, permission: &Permission, context: &EvaluationContext) -> Result<bool, ClientError> {
        let evaluation = self.evaluate_permissions(context)?;
        Ok(evaluation.effective_permissions.iter().any(|ep| &ep.permission == permission))
    }

    /// Collect roles recursively with inheritance
    fn collect_roles_recursive(
        &self,
        role_id: &str,
        collected: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
        depth: usize,
    ) -> Result<(), ClientError> {
        if depth > self.config.max_role_depth {
            return Err(ClientError::IoError("Maximum role depth exceeded".to_string()));
        }

        if visited.contains(role_id) {
            return Err(ClientError::IoError("Circular role dependency detected".to_string()));
        }

        visited.insert(role_id.to_string());
        
        if !collected.contains(&role_id.to_string()) {
            collected.push(role_id.to_string());
        }

        if self.config.enable_inheritance {
            if let Some(parent_roles) = self.role_hierarchy.get(role_id) {
                for parent_role in parent_roles {
                    self.collect_roles_recursive(parent_role, collected, visited, depth + 1)?;
                }
            }
        }

        visited.remove(role_id);
        Ok(())
    }

    /// Get all available permissions
    fn get_all_permissions(&self) -> Vec<Permission> {
        vec![
            Permission::HearDeviceSound,
            Permission::ControlDevice,
            Permission::RestartDevice,
            Permission::SendCtrlAltDel,
            Permission::BlockInputDevices,
            Permission::LockDevice,
            Permission::SignOutUser,
            Permission::EnablePrivacyMode,
            Permission::ShowColoredCursor,
            Permission::AccessClipboard,
            Permission::AccessClipboardForFileTransfer,
            Permission::UseFileManager,
            Permission::SeeSystemInformation,
            Permission::DrawOnScreen,
            Permission::CreateTcpTunnels,
            Permission::RecordSession,
            Permission::InteractWithRestrictedWindows,
        ]
    }

    /// Get operator permissions
    fn get_operator_permissions(&self) -> HashMap<Permission, PermissionScope> {
        let mut permissions = HashMap::new();
        
        let operator_perms = vec![
            Permission::HearDeviceSound,
            Permission::ControlDevice,
            Permission::ShowColoredCursor,
            Permission::AccessClipboard,
            Permission::AccessClipboardForFileTransfer,
            Permission::UseFileManager,
            Permission::SeeSystemInformation,
            Permission::DrawOnScreen,
        ];

        for permission in operator_perms {
            permissions.insert(permission, PermissionScope {
                allowed: true,
                conditions: vec![
                    ScopeCondition::NetworkType { 
                        allowed_types: vec!["local".to_string(), "lan".to_string()] 
                    }
                ],
                limitations: vec![
                    ScopeLimitation::RequireApproval { 
                        approver_roles: vec!["system_admin".to_string()] 
                    }
                ],
            });
        }

        permissions
    }

    /// Get viewer permissions
    fn get_viewer_permissions(&self) -> HashMap<Permission, PermissionScope> {
        let mut permissions = HashMap::new();
        
        let viewer_perms = vec![
            Permission::HearDeviceSound,
            Permission::ShowColoredCursor,
            Permission::SeeSystemInformation,
        ];

        for permission in viewer_perms {
            permissions.insert(permission, PermissionScope {
                allowed: true,
                conditions: Vec::new(),
                limitations: Vec::new(),
            });
        }

        permissions
    }

    /// Get guest permissions
    fn get_guest_permissions(&self) -> HashMap<Permission, PermissionScope> {
        let mut permissions = HashMap::new();
        
        permissions.insert(Permission::ShowColoredCursor, PermissionScope {
            allowed: true,
            conditions: vec![ScopeCondition::TimeRange { 
                start: "09:00".to_string(), 
                end: "17:00".to_string() 
            }],
            limitations: vec![ScopeLimitation::MaxDuration { minutes: 30 }],
        });

        permissions
    }

    /// Get role by ID
    pub fn get_role(&self, role_id: &str) -> Option<&Role> {
        self.roles.get(role_id)
    }

    /// Get all roles
    pub fn get_all_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }

    /// Get user roles
    pub fn get_user_roles(&self, user_id: &str) -> Vec<&Role> {
        if let Some(role_ids) = self.user_roles.get(user_id) {
            role_ids.iter()
                .filter_map(|role_id| self.roles.get(role_id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Update role
    pub fn update_role(&mut self, role: Role) -> Result<(), ClientError> {
        if !self.roles.contains_key(&role.id) {
            return Err(ClientError::IoError("Role not found".to_string()));
        }

        let role_id = role.id.clone();
        self.roles.insert(role_id.clone(), role);
        tracing::info!("Updated role: {}", role_id);
        Ok(())
    }

    /// Delete role
    pub fn delete_role(&mut self, role_id: &str) -> Result<(), ClientError> {
        let role = self.roles.get(role_id)
            .ok_or_else(|| ClientError::IoError("Role not found".to_string()))?;

        if role.metadata.is_system_role {
            return Err(ClientError::IoError("Cannot delete system role".to_string()));
        }

        self.roles.remove(role_id);
        
        // Remove role from all users
        for user_roles in self.user_roles.values_mut() {
            user_roles.retain(|r| r != role_id);
        }

        // Remove from hierarchy
        self.role_hierarchy.remove(role_id);

        tracing::info!("Deleted role: {}", role_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rbac_creation() {
        let config = RBACConfig::default();
        let rbac = RoleBasedAccessControl::new(config);
        
        // Should have default roles
        assert_eq!(rbac.roles.len(), 4);
        assert!(rbac.roles.contains_key("system_admin"));
        assert!(rbac.roles.contains_key("operator"));
        assert!(rbac.roles.contains_key("viewer"));
        assert!(rbac.roles.contains_key("guest"));
    }

    #[test]
    fn test_role_assignment() {
        let config = RBACConfig::default();
        let mut rbac = RoleBasedAccessControl::new(config);
        
        rbac.assign_role("user123".to_string(), "viewer".to_string(), "admin".to_string()).unwrap();
        
        let user_roles = rbac.get_user_roles("user123");
        assert_eq!(user_roles.len(), 1);
        assert_eq!(user_roles[0].id, "viewer");
    }

    #[test]
    fn test_permission_evaluation() {
        let config = RBACConfig::default();
        let mut rbac = RoleBasedAccessControl::new(config);
        
        rbac.assign_role("user123".to_string(), "viewer".to_string(), "admin".to_string()).unwrap();
        
        let context = EvaluationContext {
            user_id: "user123".to_string(),
            device_id: "device1".to_string(),
            session_id: "session1".to_string(),
            timestamp: SystemTime::now(),
            location: None,
            network_type: "lan".to_string(),
            additional_context: HashMap::new(),
        };
        
        let result = rbac.evaluate_permissions(&context).unwrap();
        assert!(result.allowed);
        assert!(result.roles.contains(&"viewer".to_string()));
    }

    #[test]
    fn test_role_inheritance() {
        let config = RBACConfig::default();
        let mut rbac = RoleBasedAccessControl::new(config);
        
        rbac.assign_role("user123".to_string(), "operator".to_string(), "admin".to_string()).unwrap();
        
        let context = EvaluationContext {
            user_id: "user123".to_string(),
            device_id: "device1".to_string(),
            session_id: "session1".to_string(),
            timestamp: SystemTime::now(),
            location: None,
            network_type: "lan".to_string(),
            additional_context: HashMap::new(),
        };
        
        let result = rbac.evaluate_permissions(&context).unwrap();
        
        // Should have operator role and inherit from viewer
        assert!(result.roles.contains(&"operator".to_string()));
        assert!(result.roles.contains(&"viewer".to_string()));
    }

    #[test]
    fn test_custom_role_creation() {
        let config = RBACConfig::default();
        let mut rbac = RoleBasedAccessControl::new(config);
        
        let mut permissions = HashMap::new();
        permissions.insert(Permission::SeeSystemInformation, PermissionScope {
            allowed: true,
            conditions: Vec::new(),
            limitations: Vec::new(),
        });
        
        let custom_role = Role {
            id: "custom_role".to_string(),
            name: "Custom Role".to_string(),
            description: "A custom role".to_string(),
            permissions,
            metadata: RoleMetadata {
                category: RoleCategory::Custom,
                priority: 30,
                is_system_role: false,
                is_temporary: false,
                expires_at: None,
                created_by: "admin".to_string(),
            },
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };
        
        rbac.create_role(custom_role).unwrap();
        assert_eq!(rbac.roles.len(), 5);
        assert!(rbac.roles.contains_key("custom_role"));
    }
}
