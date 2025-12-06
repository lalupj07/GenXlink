use crate::permission_profiles::{Permission, PermissionProfile, PermissionProfileType};
use crate::ClientError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, Instant};
use uuid::Uuid;

/// Enhanced access control system with granular permissions
pub struct AccessControlManager {
    sessions: HashMap<String, AccessSession>,
    policies: HashMap<String, AccessPolicy>,
    audit_log: Vec<AuditEvent>,
    config: AccessControlConfig,
}

/// Access control configuration
#[derive(Debug, Clone)]
pub struct AccessControlConfig {
    /// Maximum session duration
    pub max_session_duration: Duration,
    /// Enable audit logging
    pub enable_audit_log: bool,
    /// Require confirmation for sensitive actions
    pub require_confirmation: bool,
    /// Enable temporary permissions
    pub enable_temporary_permissions: bool,
    /// Maximum temporary permission duration
    pub max_temporary_duration: Duration,
    /// Enable role-based access control
    pub enable_rbac: bool,
}

impl Default for AccessControlConfig {
    fn default() -> Self {
        Self {
            max_session_duration: Duration::from_secs(8 * 60 * 60), // 8 hours
            enable_audit_log: true,
            require_confirmation: true,
            enable_temporary_permissions: true,
            max_temporary_duration: Duration::from_secs(60 * 60), // 1 hour
            enable_rbac: true,
        }
    }
}

/// Access session for a remote connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessSession {
    pub id: String,
    pub remote_device_id: String,
    pub profile: PermissionProfile,
    pub temporary_permissions: HashMap<Permission, TemporaryPermission>,
    pub created_at: SystemTime,
    pub expires_at: Option<SystemTime>,
    pub last_activity: SystemTime,
    pub status: SessionStatus,
    pub metadata: SessionMetadata,
}

/// Temporary permission with expiration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryPermission {
    pub permission: Permission,
    pub granted_at: SystemTime,
    pub expires_at: SystemTime,
    pub granted_by: String,
    pub reason: String,
}

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Paused,
    Suspended,
    Terminated,
    Expired,
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub remote_ip: Option<String>,
    pub device_name: String,
    pub os_version: Option<String>,
    pub connection_type: ConnectionType,
    pub encryption_enabled: bool,
}

/// Connection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    Local,
    Lan,
    Internet,
    Vpn,
}

/// Access policy for automated permission management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub conditions: Vec<PolicyCondition>,
    pub actions: Vec<PolicyAction>,
    pub enabled: bool,
    pub priority: u8,
}

/// Policy condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyCondition {
    TimeRange { start: String, end: String },
    DeviceGroup { group: String },
    UserRole { role: String },
    ConnectionType { connection_type: ConnectionType },
    PermissionRequested { permission: Permission },
    RiskLevel { level: RiskLevel },
}

/// Policy action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyAction {
    GrantPermission { permission: Permission },
    DenyPermission { permission: Permission },
    RequireConfirmation,
    NotifyAdmin,
    LogEvent { level: AuditLevel },
    LimitDuration { minutes: u64 },
}

/// Risk level for access decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Audit event for security logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: SystemTime,
    pub session_id: String,
    pub event_type: AuditEventType,
    pub level: AuditLevel,
    pub description: String,
    pub details: HashMap<String, String>,
}

/// Audit event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    SessionCreated,
    SessionTerminated,
    PermissionGranted,
    PermissionDenied,
    PolicyApplied,
    SecurityViolation,
    ConfigurationChanged,
}

/// Audit severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Permission request with context
#[derive(Debug, Clone)]
pub struct PermissionRequest {
    pub session_id: String,
    pub permission: Permission,
    pub requested_by: String,
    pub reason: Option<String>,
    pub temporary: bool,
    pub duration: Option<Duration>,
}

/// Permission check result
#[derive(Debug, Clone)]
pub struct PermissionCheckResult {
    pub allowed: bool,
    pub reason: String,
    pub requires_confirmation: bool,
    pub temporary: bool,
    pub expires_at: Option<SystemTime>,
}

impl Default for AccessControlManager {
    fn default() -> Self {
        Self::new(AccessControlConfig::default())
    }
}

impl AccessControlManager {
    /// Create a new access control manager
    pub fn new(config: AccessControlConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            policies: HashMap::new(),
            audit_log: Vec::new(),
            config,
        }
    }

    /// Create a new access session
    pub fn create_session(
        &mut self,
        remote_device_id: String,
        profile: PermissionProfile,
        metadata: SessionMetadata,
    ) -> Result<String, ClientError> {
        let session_id = Uuid::new_v4().to_string();
        let expires_at = SystemTime::now() + self.config.max_session_duration;

        let session = AccessSession {
            id: session_id.clone(),
            remote_device_id,
            profile,
            temporary_permissions: HashMap::new(),
            created_at: SystemTime::now(),
            expires_at: Some(expires_at),
            last_activity: SystemTime::now(),
            status: SessionStatus::Active,
            metadata,
        };

        self.sessions.insert(session_id.clone(), session);

        if self.config.enable_audit_log {
            self.log_audit_event(AuditEvent {
                id: Uuid::new_v4().to_string(),
                timestamp: SystemTime::now(),
                session_id: session_id.clone(),
                event_type: AuditEventType::SessionCreated,
                level: AuditLevel::Info,
                description: "New access session created".to_string(),
                details: HashMap::new(),
            });
        }

        tracing::info!("Created access session: {}", session_id);
        Ok(session_id)
    }

    /// Check if a permission is allowed for a session
    pub fn check_permission(&mut self, request: PermissionRequest) -> Result<PermissionCheckResult, ClientError> {
        let session_id = request.session_id.clone();
        let session = self.sessions.get_mut(&session_id)
            .ok_or_else(|| ClientError::IoError("Session not found".to_string()))?;

        // Update last activity
        session.last_activity = SystemTime::now();

        // Check if session is active
        if session.status != SessionStatus::Active {
            return Ok(PermissionCheckResult {
                allowed: false,
                reason: "Session is not active".to_string(),
                requires_confirmation: false,
                temporary: false,
                expires_at: None,
            });
        }

        // Check session expiration
        if let Some(expires_at) = session.expires_at {
            if SystemTime::now() > expires_at {
                session.status = SessionStatus::Expired;
                return Ok(PermissionCheckResult {
                    allowed: false,
                    reason: "Session has expired".to_string(),
                    requires_confirmation: false,
                    temporary: false,
                    expires_at: None,
                });
            }
        }

        // Check base profile permissions
        let base_allowed = session.profile.has_permission(&request.permission);

        // Check temporary permissions
        let temp_allowed = session.temporary_permissions.get(&request.permission)
            .map(|temp_perm| {
                SystemTime::now() < temp_perm.expires_at
            })
            .unwrap_or(false);

        let allowed = base_allowed || temp_allowed;

        // Get expires_at for temporary permissions before borrowing self again
        let expires_at = session.temporary_permissions.get(&request.permission)
            .map(|temp| temp.expires_at);

        // Apply policies
        let policy_result = self.apply_policies(&request)?;
        let final_allowed = allowed && policy_result.allowed;

        // Log the permission check
        if self.config.enable_audit_log {
            self.log_audit_event(AuditEvent {
                id: Uuid::new_v4().to_string(),
                timestamp: SystemTime::now(),
                session_id: request.session_id.clone(),
                event_type: if final_allowed {
                    AuditEventType::PermissionGranted
                } else {
                    AuditEventType::PermissionDenied
                },
                level: if request.permission == Permission::RestartDevice {
                    AuditLevel::Warning
                } else {
                    AuditLevel::Info
                },
                description: format!(
                    "Permission {} {} for session {}",
                    request.permission.name(),
                    if final_allowed { "granted" } else { "denied" },
                    request.session_id
                ),
                details: {
                    let mut details = HashMap::new();
                    details.insert("permission".to_string(), request.permission.name().to_string());
                    details.insert("requested_by".to_string(), request.requested_by);
                    if let Some(reason) = &request.reason {
                        details.insert("reason".to_string(), reason.clone());
                    }
                    details
                },
            });

            Ok(PermissionCheckResult {
                allowed: final_allowed,
                reason: policy_result.reason,
                requires_confirmation: policy_result.requires_confirmation,
                temporary: temp_allowed,
                expires_at,
            })
        } else {
            Ok(PermissionCheckResult {
                allowed: final_allowed,
                reason: policy_result.reason,
                requires_confirmation: policy_result.requires_confirmation,
                temporary: temp_allowed,
                expires_at,
            })
        }
    }

    /// Grant temporary permission to a session
    pub fn grant_temporary_permission(
        &mut self,
        session_id: &str,
        permission: Permission,
        duration: Duration,
        granted_by: String,
        reason: String,
    ) -> Result<(), ClientError> {
        if !self.config.enable_temporary_permissions {
            return Err(ClientError::IoError("Temporary permissions are disabled".to_string()));
        }

        if duration > self.config.max_temporary_duration {
            return Err(ClientError::IoError("Duration exceeds maximum allowed".to_string()));
        }

        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| ClientError::IoError("Session not found".to_string()))?;

        let temp_permission = TemporaryPermission {
            permission: permission.clone(),
            granted_at: SystemTime::now(),
            expires_at: SystemTime::now() + duration,
            granted_by,
            reason,
        };

        session.temporary_permissions.insert(permission.clone(), temp_permission);

        if self.config.enable_audit_log {
            self.log_audit_event(AuditEvent {
                id: Uuid::new_v4().to_string(),
                timestamp: SystemTime::now(),
                session_id: session_id.to_string(),
                event_type: AuditEventType::PermissionGranted,
                level: AuditLevel::Info,
                description: format!("Temporary permission {} granted", permission.name()),
                details: {
                    let mut details = HashMap::new();
                    details.insert("permission".to_string(), permission.name().to_string());
                    details.insert("duration".to_string(), format!("{:?}", duration));
                    details
                },
            });
        }

        tracing::info!("Granted temporary permission {} to session {}", permission.name(), session_id);
        Ok(())
    }

    /// Apply access policies to a permission request
    fn apply_policies(&self, request: &PermissionRequest) -> Result<PolicyResult, ClientError> {
        let mut result = PolicyResult {
            allowed: true,
            reason: "No policies applied".to_string(),
            requires_confirmation: false,
        };

        // Sort policies by priority (higher first)
        let mut sorted_policies: Vec<_> = self.policies.values().collect();
        sorted_policies.sort_by(|a, b| b.priority.cmp(&a.priority));

        for policy in sorted_policies {
            if !policy.enabled {
                continue;
            }

            if self.evaluate_policy_conditions(&policy.conditions, request) {
                for action in &policy.actions {
                    match action {
                        PolicyAction::DenyPermission { permission } if *permission == request.permission => {
                            result.allowed = false;
                            result.reason = format!("Denied by policy: {}", policy.name);
                        }
                        PolicyAction::RequireConfirmation => {
                            result.requires_confirmation = true;
                        }
                        PolicyAction::LimitDuration { minutes } => {
                            result.reason = format!("Duration limited to {} minutes by policy: {}", minutes, policy.name);
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(result)
    }

    /// Evaluate policy conditions
    fn evaluate_policy_conditions(&self, conditions: &[PolicyCondition], request: &PermissionRequest) -> bool {
        for condition in conditions {
            let matches = match condition {
                PolicyCondition::PermissionRequested { permission } => *permission == request.permission,
                PolicyCondition::RiskLevel { level } => self.assess_risk_level(request) == *level,
                _ => true, // Placeholder for other condition types
            };
            if !matches {
                return false;
            }
        }
        true
    }

    /// Assess risk level for a permission request
    fn assess_risk_level(&self, request: &PermissionRequest) -> RiskLevel {
        match request.permission {
            Permission::RestartDevice | Permission::SignOutUser => RiskLevel::High,
            Permission::ControlDevice | Permission::AccessClipboard => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }

    /// Terminate a session
    pub fn terminate_session(&mut self, session_id: &str) -> Result<(), ClientError> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| ClientError::IoError("Session not found".to_string()))?;

        session.status = SessionStatus::Terminated;

        if self.config.enable_audit_log {
            self.log_audit_event(AuditEvent {
                id: Uuid::new_v4().to_string(),
                timestamp: SystemTime::now(),
                session_id: session_id.to_string(),
                event_type: AuditEventType::SessionTerminated,
                level: AuditLevel::Info,
                description: "Session terminated".to_string(),
                details: HashMap::new(),
            });
        }

        tracing::info!("Terminated session: {}", session_id);
        Ok(())
    }

    /// Add an access policy
    pub fn add_policy(&mut self, policy: AccessPolicy) {
        self.policies.insert(policy.id.clone(), policy);
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Option<&AccessSession> {
        self.sessions.get(session_id)
    }

    /// Get all active sessions
    pub fn get_active_sessions(&self) -> Vec<&AccessSession> {
        self.sessions.values()
            .filter(|session| session.status == SessionStatus::Active)
            .collect()
    }

    /// Get audit log
    pub fn get_audit_log(&self) -> &[AuditEvent] {
        &self.audit_log
    }

    /// Log an audit event
    fn log_audit_event(&mut self, event: AuditEvent) {
        self.audit_log.push(event);
        
        // Keep only last 1000 events
        if self.audit_log.len() > 1000 {
            self.audit_log.remove(0);
        }
    }

    /// Cleanup expired sessions and temporary permissions
    pub fn cleanup(&mut self) {
        let now = SystemTime::now();
        
        // Remove expired sessions
        self.sessions.retain(|_, session| {
            if let Some(expires_at) = session.expires_at {
                now < expires_at && session.status != SessionStatus::Terminated
            } else {
                session.status != SessionStatus::Terminated
            }
        });

        // Remove expired temporary permissions
        for session in self.sessions.values_mut() {
            session.temporary_permissions.retain(|_, temp_perm| now < temp_perm.expires_at);
        }
    }
}

/// Policy evaluation result
#[derive(Debug, Clone)]
struct PolicyResult {
    allowed: bool,
    reason: String,
    requires_confirmation: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission_profiles::{PermissionProfileManager, PermissionProfileType};

    #[test]
    fn test_access_control_creation() {
        let config = AccessControlConfig::default();
        let manager = AccessControlManager::new(config);
        assert_eq!(manager.sessions.len(), 0);
        assert_eq!(manager.policies.len(), 0);
    }

    #[test]
    fn test_session_creation() {
        let config = AccessControlConfig::default();
        let mut manager = AccessControlManager::new(config);
        
        let profile = PermissionProfile::new(PermissionProfileType::Default);
        let metadata = SessionMetadata {
            remote_ip: Some("192.168.1.100".to_string()),
            device_name: "Test Device".to_string(),
            os_version: Some("Windows 10".to_string()),
            connection_type: ConnectionType::Lan,
            encryption_enabled: true,
        };

        let session_id = manager.create_session(
            "device-123".to_string(),
            profile,
            metadata,
        ).unwrap();

        assert!(!session_id.is_empty());
        assert_eq!(manager.sessions.len(), 1);
        
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.remote_device_id, "device-123");
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn test_permission_check() {
        let config = AccessControlConfig::default();
        let mut manager = AccessControlManager::new(config);
        
        let profile = PermissionProfile::new(PermissionProfileType::ScreenSharing);
        let metadata = SessionMetadata {
            remote_ip: None,
            device_name: "Test Device".to_string(),
            os_version: None,
            connection_type: ConnectionType::Local,
            encryption_enabled: true,
        };

        let session_id = manager.create_session(
            "device-123".to_string(),
            profile,
            metadata,
        ).unwrap();

        let request = PermissionRequest {
            session_id: session_id.clone(),
            permission: Permission::ControlDevice,
            requested_by: "user".to_string(),
            reason: None,
            temporary: false,
            duration: None,
        };

        let result = manager.check_permission(request).unwrap();
        assert!(!result.allowed); // Screen sharing profile doesn't allow control
    }

    #[test]
    fn test_temporary_permission() {
        let mut config = AccessControlConfig::default();
        config.enable_temporary_permissions = true;
        let mut manager = AccessControlManager::new(config);
        
        let profile = PermissionProfile::new(PermissionProfileType::ScreenSharing);
        let metadata = SessionMetadata {
            remote_ip: None,
            device_name: "Test Device".to_string(),
            os_version: None,
            connection_type: ConnectionType::Local,
            encryption_enabled: true,
        };

        let session_id = manager.create_session(
            "device-123".to_string(),
            profile,
            metadata,
        ).unwrap();

        // Grant temporary permission
        manager.grant_temporary_permission(
            &session_id,
            Permission::ControlDevice,
            Duration::from_secs(60),
            "admin".to_string(),
            "Testing".to_string(),
        ).unwrap();

        let request = PermissionRequest {
            session_id: session_id.clone(),
            permission: Permission::ControlDevice,
            requested_by: "user".to_string(),
            reason: None,
            temporary: false,
            duration: None,
        };

        let result = manager.check_permission(request).unwrap();
        assert!(result.allowed);
        assert!(result.temporary);
    }
}
