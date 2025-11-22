use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Permission profile types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionProfileType {
    Default,
    ScreenSharing,
    FullAccess,
    UnattendedAccess,
}

impl PermissionProfileType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::ScreenSharing => "Screen Sharing",
            Self::FullAccess => "Full Access",
            Self::UnattendedAccess => "Unattended Access",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            Self::Default => "Basic screen viewing with limited control",
            Self::ScreenSharing => "View screen only, no control",
            Self::FullAccess => "Full control with all permissions",
            Self::UnattendedAccess => "Access device without user present",
        }
    }
}

/// Individual permissions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    // Audio & Sound
    HearDeviceSound,
    
    // Control
    ControlDevice,
    RestartDevice,
    SendCtrlAltDel,
    BlockInputDevices,
    LockDevice,
    SignOutUser,
    
    // Privacy
    EnablePrivacyMode,
    ShowColoredCursor,
    
    // Clipboard & Files
    AccessClipboard,
    AccessClipboardForFileTransfer,
    UseFileManager,
    
    // System
    SeeSystemInformation,
    DrawOnScreen,
    CreateTcpTunnels,
    
    // Recording
    RecordSession,
    
    // Windows
    InteractWithRestrictedWindows,
}

impl Permission {
    pub fn name(&self) -> &'static str {
        match self {
            Self::HearDeviceSound => "Hear my device's sound",
            Self::ControlDevice => "Control my device",
            Self::RestartDevice => "Restart my device",
            Self::SendCtrlAltDel => "Send Ctrl + Alt + Del",
            Self::BlockInputDevices => "Block my input devices",
            Self::LockDevice => "Lock my device",
            Self::SignOutUser => "Sign out user",
            Self::EnablePrivacyMode => "Enable privacy mode",
            Self::ShowColoredCursor => "Show a colored cursor when input is disabled",
            Self::AccessClipboard => "Access my device's clipboard",
            Self::AccessClipboardForFileTransfer => "Access my device's clipboard to transfer files",
            Self::UseFileManager => "Use File Manager",
            Self::SeeSystemInformation => "See my system information",
            Self::DrawOnScreen => "Draw on my device's screen",
            Self::CreateTcpTunnels => "Create TCP tunnels",
            Self::RecordSession => "Record the session",
            Self::InteractWithRestrictedWindows => "Interact with windows that have restricted access",
        }
    }
    
    pub fn category(&self) -> PermissionCategory {
        match self {
            Self::HearDeviceSound => PermissionCategory::Audio,
            Self::ControlDevice | Self::RestartDevice | Self::SendCtrlAltDel | 
            Self::BlockInputDevices | Self::LockDevice | Self::SignOutUser => PermissionCategory::Control,
            Self::EnablePrivacyMode | Self::ShowColoredCursor => PermissionCategory::Privacy,
            Self::AccessClipboard | Self::AccessClipboardForFileTransfer | 
            Self::UseFileManager => PermissionCategory::Files,
            Self::SeeSystemInformation | Self::DrawOnScreen | Self::CreateTcpTunnels => PermissionCategory::System,
            Self::RecordSession => PermissionCategory::Recording,
            Self::InteractWithRestrictedWindows => PermissionCategory::Advanced,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionCategory {
    Audio,
    Control,
    Privacy,
    Files,
    System,
    Recording,
    Advanced,
}

/// Permission profile with enabled permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionProfile {
    pub profile_type: PermissionProfileType,
    pub enabled: bool,
    pub permissions: HashMap<Permission, bool>,
}

impl PermissionProfile {
    /// Create a new permission profile with default permissions
    pub fn new(profile_type: PermissionProfileType) -> Self {
        let mut profile = Self {
            profile_type,
            enabled: true,
            permissions: HashMap::new(),
        };
        
        // Set default permissions based on profile type
        profile.set_default_permissions();
        profile
    }
    
    /// Set default permissions based on profile type
    fn set_default_permissions(&mut self) {
        match self.profile_type {
            PermissionProfileType::Default => {
                self.set_permission(Permission::HearDeviceSound, true);
                self.set_permission(Permission::ControlDevice, true);
                self.set_permission(Permission::RestartDevice, true);
                self.set_permission(Permission::SendCtrlAltDel, true);
                self.set_permission(Permission::BlockInputDevices, true);
                self.set_permission(Permission::LockDevice, true);
                self.set_permission(Permission::ShowColoredCursor, true);
                self.set_permission(Permission::AccessClipboard, true);
                self.set_permission(Permission::AccessClipboardForFileTransfer, true);
                self.set_permission(Permission::UseFileManager, true);
                self.set_permission(Permission::SeeSystemInformation, true);
                self.set_permission(Permission::DrawOnScreen, true);
                self.set_permission(Permission::RecordSession, true);
            }
            PermissionProfileType::ScreenSharing => {
                // View only - no control permissions
                self.set_permission(Permission::ShowColoredCursor, true);
            }
            PermissionProfileType::FullAccess => {
                // All permissions enabled
                self.set_permission(Permission::HearDeviceSound, true);
                self.set_permission(Permission::ControlDevice, true);
                self.set_permission(Permission::RestartDevice, true);
                self.set_permission(Permission::SendCtrlAltDel, true);
                self.set_permission(Permission::BlockInputDevices, true);
                self.set_permission(Permission::LockDevice, true);
                self.set_permission(Permission::SignOutUser, true);
                self.set_permission(Permission::ShowColoredCursor, true);
                self.set_permission(Permission::AccessClipboard, true);
                self.set_permission(Permission::AccessClipboardForFileTransfer, true);
                self.set_permission(Permission::UseFileManager, true);
                self.set_permission(Permission::SeeSystemInformation, true);
                self.set_permission(Permission::DrawOnScreen, true);
                self.set_permission(Permission::CreateTcpTunnels, true);
                self.set_permission(Permission::RecordSession, true);
            }
            PermissionProfileType::UnattendedAccess => {
                // Full access plus unattended features
                self.set_permission(Permission::HearDeviceSound, true);
                self.set_permission(Permission::ControlDevice, true);
                self.set_permission(Permission::RestartDevice, true);
                self.set_permission(Permission::SendCtrlAltDel, true);
                self.set_permission(Permission::BlockInputDevices, true);
                self.set_permission(Permission::LockDevice, true);
                self.set_permission(Permission::ShowColoredCursor, true);
                self.set_permission(Permission::AccessClipboard, true);
                self.set_permission(Permission::AccessClipboardForFileTransfer, true);
                self.set_permission(Permission::UseFileManager, true);
                self.set_permission(Permission::SeeSystemInformation, true);
                self.set_permission(Permission::DrawOnScreen, true);
                self.set_permission(Permission::CreateTcpTunnels, true);
                self.set_permission(Permission::RecordSession, true);
            }
        }
    }
    
    /// Set a permission
    pub fn set_permission(&mut self, permission: Permission, enabled: bool) {
        self.permissions.insert(permission, enabled);
    }
    
    /// Check if a permission is enabled
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.get(permission).copied().unwrap_or(false)
    }
    
    /// Get all enabled permissions
    pub fn enabled_permissions(&self) -> Vec<Permission> {
        self.permissions
            .iter()
            .filter(|(_, &enabled)| enabled)
            .map(|(perm, _)| perm.clone())
            .collect()
    }
}

/// Permission profile manager
pub struct PermissionProfileManager {
    profiles: HashMap<PermissionProfileType, PermissionProfile>,
    active_profile: PermissionProfileType,
}

impl Default for PermissionProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionProfileManager {
    pub fn new() -> Self {
        let mut profiles = HashMap::new();
        
        // Create all default profiles
        profiles.insert(
            PermissionProfileType::Default,
            PermissionProfile::new(PermissionProfileType::Default),
        );
        profiles.insert(
            PermissionProfileType::ScreenSharing,
            PermissionProfile::new(PermissionProfileType::ScreenSharing),
        );
        profiles.insert(
            PermissionProfileType::FullAccess,
            PermissionProfile::new(PermissionProfileType::FullAccess),
        );
        profiles.insert(
            PermissionProfileType::UnattendedAccess,
            PermissionProfile::new(PermissionProfileType::UnattendedAccess),
        );
        
        Self {
            profiles,
            active_profile: PermissionProfileType::Default,
        }
    }
    
    /// Get a profile
    pub fn get_profile(&self, profile_type: PermissionProfileType) -> Option<&PermissionProfile> {
        self.profiles.get(&profile_type)
    }
    
    /// Get a mutable profile
    pub fn get_profile_mut(&mut self, profile_type: PermissionProfileType) -> Option<&mut PermissionProfile> {
        self.profiles.get_mut(&profile_type)
    }
    
    /// Get the active profile
    pub fn get_active_profile(&self) -> &PermissionProfile {
        self.profiles.get(&self.active_profile).unwrap()
    }
    
    /// Set the active profile
    pub fn set_active_profile(&mut self, profile_type: PermissionProfileType) {
        self.active_profile = profile_type;
    }
    
    /// Get active profile type
    pub fn active_profile_type(&self) -> PermissionProfileType {
        self.active_profile
    }
    
    /// Check if a permission is enabled in the active profile
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.get_active_profile().has_permission(permission)
    }
    
    /// Get all profile types
    pub fn all_profile_types() -> Vec<PermissionProfileType> {
        vec![
            PermissionProfileType::Default,
            PermissionProfileType::ScreenSharing,
            PermissionProfileType::FullAccess,
            PermissionProfileType::UnattendedAccess,
        ]
    }
    
    /// Get all permissions
    pub fn all_permissions() -> Vec<Permission> {
        vec![
            Permission::HearDeviceSound,
            Permission::ControlDevice,
            Permission::RestartDevice,
            Permission::EnablePrivacyMode,
            Permission::SendCtrlAltDel,
            Permission::BlockInputDevices,
            Permission::LockDevice,
            Permission::SignOutUser,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_profile() {
        let profile = PermissionProfile::new(PermissionProfileType::Default);
        assert!(profile.has_permission(&Permission::ControlDevice));
        assert!(profile.has_permission(&Permission::HearDeviceSound));
    }
    
    #[test]
    fn test_screen_sharing_profile() {
        let profile = PermissionProfile::new(PermissionProfileType::ScreenSharing);
        assert!(!profile.has_permission(&Permission::ControlDevice));
        assert!(!profile.has_permission(&Permission::HearDeviceSound));
    }
    
    #[test]
    fn test_full_access_profile() {
        let profile = PermissionProfile::new(PermissionProfileType::FullAccess);
        assert!(profile.has_permission(&Permission::ControlDevice));
        assert!(profile.has_permission(&Permission::SignOutUser));
    }
    
    #[test]
    fn test_permission_manager() {
        let mut manager = PermissionProfileManager::new();
        assert_eq!(manager.active_profile_type(), PermissionProfileType::Default);
        
        manager.set_active_profile(PermissionProfileType::FullAccess);
        assert_eq!(manager.active_profile_type(), PermissionProfileType::FullAccess);
        assert!(manager.has_permission(&Permission::ControlDevice));
    }
}
