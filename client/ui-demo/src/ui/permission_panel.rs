use eframe::egui;
use genxlink_client_core::permission_profiles::*;

/// Permission panel for managing access profiles
pub struct PermissionPanel {
    manager: PermissionProfileManager,
    selected_profile: PermissionProfileType,
}

impl Default for PermissionPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionPanel {
    pub fn new() -> Self {
        Self {
            manager: PermissionProfileManager::new(),
            selected_profile: PermissionProfileType::Default,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) -> PermissionAction {
        let mut action = PermissionAction::None;
        
        ui.heading("Permission Profiles");
        ui.add_space(10.0);
        
        ui.label("When other users connect to me, they can:");
        ui.add_space(15.0);
        
        // Profile selector tabs
        ui.horizontal(|ui| {
            for profile_type in PermissionProfileManager::all_profile_types() {
                let is_selected = self.selected_profile == profile_type;
                
                if ui.selectable_label(is_selected, profile_type.name()).clicked() {
                    self.selected_profile = profile_type;
                    action = PermissionAction::ProfileChanged(profile_type);
                }
            }
        });
        
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(15.0);
        
        // Show selected profile
        let selected_profile = self.selected_profile;
        if let Some(profile) = self.manager.get_profile_mut(selected_profile) {
            Self::show_profile_editor(ui, profile);
        }
        
        action
    }
    
    fn show_profile_editor(ui: &mut egui::Ui, profile: &mut PermissionProfile) {
        // Profile header
        ui.horizontal(|ui| {
            ui.heading(profile.profile_type.name());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut profile.enabled, "Enable permission profile");
            });
        });
        
        ui.add_space(5.0);
        ui.label(
            egui::RichText::new(profile.profile_type.description())
                .small()
                .color(egui::Color32::from_rgb(156, 163, 175))
        );
        ui.add_space(15.0);
        
        // Permissions list
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                // Group permissions by category
                Self::show_permission_category(ui, profile, PermissionCategory::Audio, "🔊 Audio & Sound");
                ui.add_space(10.0);
                
                Self::show_permission_category(ui, profile, PermissionCategory::Control, "🎮 Control");
                ui.add_space(10.0);
                
                Self::show_permission_category(ui, profile, PermissionCategory::Privacy, "🔒 Privacy");
                ui.add_space(10.0);
                
                Self::show_permission_category(ui, profile, PermissionCategory::Files, "📁 Clipboard & Files");
                ui.add_space(10.0);
                
                Self::show_permission_category(ui, profile, PermissionCategory::System, "⚙️ System");
                ui.add_space(10.0);
                
                Self::show_permission_category(ui, profile, PermissionCategory::Recording, "🎥 Recording");
                ui.add_space(10.0);
                
                Self::show_permission_category(ui, profile, PermissionCategory::Advanced, "🔧 Advanced");
            });
    }
    
    fn show_permission_category(
        ui: &mut egui::Ui,
        profile: &mut PermissionProfile,
        category: PermissionCategory,
        title: &str,
    ) {
        ui.label(egui::RichText::new(title).strong());
        ui.add_space(5.0);
        
        for permission in PermissionProfileManager::all_permissions() {
            if permission.category() == category {
                let mut enabled = profile.has_permission(&permission);
                
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut enabled, "").changed() {
                        profile.set_permission(permission.clone(), enabled);
                    }
                    ui.label(permission.name());
                });
            }
        }
    }
    
    /// Get the permission manager
    pub fn manager(&self) -> &PermissionProfileManager {
        &self.manager
    }
    
    /// Get the permission manager mutably
    pub fn manager_mut(&mut self) -> &mut PermissionProfileManager {
        &mut self.manager
    }
}

/// Actions that can be triggered from the permission panel
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PermissionAction {
    None,
    ProfileChanged(PermissionProfileType),
}
