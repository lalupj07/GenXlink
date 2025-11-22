use serde::{Deserialize, Serialize};

/// Application theme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
            Self::System => "System",
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Light => "☀️",
            Self::Dark => "🌙",
            Self::System => "💻",
        }
    }
    
    pub fn all() -> Vec<Theme> {
        vec![Self::Light, Self::Dark, Self::System]
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::System
    }
}

/// Color scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    // Background colors
    pub background: [u8; 3],
    pub surface: [u8; 3],
    pub panel: [u8; 3],
    
    // Text colors
    pub text_primary: [u8; 3],
    pub text_secondary: [u8; 3],
    pub text_disabled: [u8; 3],
    
    // Accent colors
    pub primary: [u8; 3],
    pub secondary: [u8; 3],
    pub accent: [u8; 3],
    
    // Status colors
    pub success: [u8; 3],
    pub warning: [u8; 3],
    pub error: [u8; 3],
    pub info: [u8; 3],
    
    // Border colors
    pub border: [u8; 3],
    pub divider: [u8; 3],
}

impl ColorScheme {
    /// Light theme colors
    pub fn light() -> Self {
        Self {
            background: [255, 255, 255],
            surface: [250, 250, 250],
            panel: [245, 245, 245],
            
            text_primary: [0, 0, 0],
            text_secondary: [100, 100, 100],
            text_disabled: [180, 180, 180],
            
            primary: [59, 130, 246],    // Blue
            secondary: [139, 92, 246],   // Purple
            accent: [236, 72, 153],      // Pink
            
            success: [34, 197, 94],      // Green
            warning: [251, 191, 36],     // Yellow
            error: [239, 68, 68],        // Red
            info: [59, 130, 246],        // Blue
            
            border: [229, 229, 229],
            divider: [240, 240, 240],
        }
    }
    
    /// Dark theme colors
    pub fn dark() -> Self {
        Self {
            background: [18, 18, 18],
            surface: [30, 30, 30],
            panel: [40, 40, 40],
            
            text_primary: [255, 255, 255],
            text_secondary: [156, 163, 175],
            text_disabled: [100, 100, 100],
            
            primary: [96, 165, 250],     // Light Blue
            secondary: [167, 139, 250],  // Light Purple
            accent: [244, 114, 182],     // Light Pink
            
            success: [74, 222, 128],     // Light Green
            warning: [253, 224, 71],     // Light Yellow
            error: [248, 113, 113],      // Light Red
            info: [96, 165, 250],        // Light Blue
            
            border: [55, 65, 81],
            divider: [45, 55, 72],
        }
    }
}

/// Theme manager
pub struct ThemeManager {
    current_theme: Theme,
    light_scheme: ColorScheme,
    dark_scheme: ColorScheme,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            current_theme: Theme::default(),
            light_scheme: ColorScheme::light(),
            dark_scheme: ColorScheme::dark(),
        }
    }
    
    /// Get current theme
    pub fn current_theme(&self) -> Theme {
        self.current_theme
    }
    
    /// Set theme
    pub fn set_theme(&mut self, theme: Theme) {
        self.current_theme = theme;
    }
    
    /// Get active color scheme
    pub fn get_color_scheme(&self) -> &ColorScheme {
        match self.current_theme {
            Theme::Light => &self.light_scheme,
            Theme::Dark => &self.dark_scheme,
            Theme::System => {
                // TODO: Detect system theme
                // For now, default to dark
                &self.dark_scheme
            }
        }
    }
    
    /// Check if dark mode is active
    pub fn is_dark_mode(&self) -> bool {
        matches!(self.current_theme, Theme::Dark) ||
        (matches!(self.current_theme, Theme::System) && self.is_system_dark())
    }
    
    /// Detect system dark mode
    fn is_system_dark(&self) -> bool {
        // TODO: Implement system theme detection
        // Windows: Check registry
        // Linux: Check GTK/Qt theme
        // macOS: Check NSAppearance
        true // Default to dark for now
    }
    
    /// Get light scheme
    pub fn light_scheme(&self) -> &ColorScheme {
        &self.light_scheme
    }
    
    /// Get dark scheme
    pub fn dark_scheme(&self) -> &ColorScheme {
        &self.dark_scheme
    }
    
    /// Set custom light scheme
    pub fn set_light_scheme(&mut self, scheme: ColorScheme) {
        self.light_scheme = scheme;
    }
    
    /// Set custom dark scheme
    pub fn set_dark_scheme(&mut self, scheme: ColorScheme) {
        self.dark_scheme = scheme;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_theme_names() {
        assert_eq!(Theme::Light.name(), "Light");
        assert_eq!(Theme::Dark.name(), "Dark");
        assert_eq!(Theme::System.name(), "System");
    }
    
    #[test]
    fn test_color_schemes() {
        let light = ColorScheme::light();
        let dark = ColorScheme::dark();
        
        assert_eq!(light.background, [255, 255, 255]);
        assert_eq!(dark.background, [18, 18, 18]);
    }
    
    #[test]
    fn test_theme_manager() {
        let mut manager = ThemeManager::new();
        assert_eq!(manager.current_theme(), Theme::System);
        
        manager.set_theme(Theme::Dark);
        assert_eq!(manager.current_theme(), Theme::Dark);
        assert!(manager.is_dark_mode());
    }
}
