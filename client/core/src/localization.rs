use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    English,
    Hindi,
    Spanish,
    French,
    German,
    Chinese,
    Japanese,
    Korean,
    Portuguese,
    Russian,
    Arabic,
    Italian,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Hindi => "hi",
            Self::Spanish => "es",
            Self::French => "fr",
            Self::German => "de",
            Self::Chinese => "zh",
            Self::Japanese => "ja",
            Self::Korean => "ko",
            Self::Portuguese => "pt",
            Self::Russian => "ru",
            Self::Arabic => "ar",
            Self::Italian => "it",
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Hindi => "हिन्दी (Hindi)",
            Self::Spanish => "Español",
            Self::French => "Français",
            Self::German => "Deutsch",
            Self::Chinese => "中文 (Chinese)",
            Self::Japanese => "日本語 (Japanese)",
            Self::Korean => "한국어 (Korean)",
            Self::Portuguese => "Português",
            Self::Russian => "Русский",
            Self::Arabic => "العربية (Arabic)",
            Self::Italian => "Italiano",
        }
    }
    
    pub fn flag(&self) -> &'static str {
        match self {
            Self::English => "🇬🇧",
            Self::Hindi => "🇮🇳",
            Self::Spanish => "🇪🇸",
            Self::French => "🇫🇷",
            Self::German => "🇩🇪",
            Self::Chinese => "🇨🇳",
            Self::Japanese => "🇯🇵",
            Self::Korean => "🇰🇷",
            Self::Portuguese => "🇵🇹",
            Self::Russian => "🇷🇺",
            Self::Arabic => "🇸🇦",
            Self::Italian => "🇮🇹",
        }
    }
    
    pub fn all() -> Vec<Language> {
        vec![
            Self::English,
            Self::Hindi,
            Self::Spanish,
            Self::French,
            Self::German,
            Self::Chinese,
            Self::Japanese,
            Self::Korean,
            Self::Portuguese,
            Self::Russian,
            Self::Arabic,
            Self::Italian,
        ]
    }
}

impl Default for Language {
    fn default() -> Self {
        Self::English
    }
}

/// Translation key
pub type TranslationKey = &'static str;

/// Localization manager
pub struct LocalizationManager {
    current_language: Language,
    translations: HashMap<Language, HashMap<String, String>>,
}

impl Default for LocalizationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalizationManager {
    pub fn new() -> Self {
        let mut manager = Self {
            current_language: Language::English,
            translations: HashMap::new(),
        };
        
        // Load default translations
        manager.load_translations();
        manager
    }
    
    /// Load translations for all languages
    fn load_translations(&mut self) {
        // English translations
        let mut en = HashMap::new();
        en.insert("app_name".to_string(), "GenXLink".to_string());
        en.insert("connect".to_string(), "Connect".to_string());
        en.insert("disconnect".to_string(), "Disconnect".to_string());
        en.insert("settings".to_string(), "Settings".to_string());
        en.insert("devices".to_string(), "Devices".to_string());
        en.insert("history".to_string(), "History".to_string());
        en.insert("premium".to_string(), "Premium".to_string());
        en.insert("permissions".to_string(), "Permissions".to_string());
        en.insert("audio".to_string(), "Audio".to_string());
        en.insert("language".to_string(), "Language".to_string());
        en.insert("theme".to_string(), "Theme".to_string());
        en.insert("device_id".to_string(), "Device ID".to_string());
        en.insert("connecting".to_string(), "Connecting...".to_string());
        en.insert("connected".to_string(), "Connected".to_string());
        en.insert("disconnected".to_string(), "Disconnected".to_string());
        self.translations.insert(Language::English, en);
        
        // Hindi translations
        let mut hi = HashMap::new();
        hi.insert("app_name".to_string(), "GenXLink".to_string());
        hi.insert("connect".to_string(), "कनेक्ट करें".to_string());
        hi.insert("disconnect".to_string(), "डिस्कनेक्ट करें".to_string());
        hi.insert("settings".to_string(), "सेटिंग्स".to_string());
        hi.insert("devices".to_string(), "डिवाइस".to_string());
        hi.insert("history".to_string(), "इतिहास".to_string());
        hi.insert("premium".to_string(), "प्रीमियम".to_string());
        hi.insert("permissions".to_string(), "अनुमतियाँ".to_string());
        hi.insert("audio".to_string(), "ऑडियो".to_string());
        hi.insert("language".to_string(), "भाषा".to_string());
        hi.insert("theme".to_string(), "थीम".to_string());
        hi.insert("device_id".to_string(), "डिवाइस आईडी".to_string());
        hi.insert("connecting".to_string(), "कनेक्ट हो रहा है...".to_string());
        hi.insert("connected".to_string(), "कनेक्ट हो गया".to_string());
        hi.insert("disconnected".to_string(), "डिस्कनेक्ट हो गया".to_string());
        self.translations.insert(Language::Hindi, hi);
        
        // Spanish translations
        let mut es = HashMap::new();
        es.insert("app_name".to_string(), "GenXLink".to_string());
        es.insert("connect".to_string(), "Conectar".to_string());
        es.insert("disconnect".to_string(), "Desconectar".to_string());
        es.insert("settings".to_string(), "Configuración".to_string());
        es.insert("devices".to_string(), "Dispositivos".to_string());
        es.insert("history".to_string(), "Historial".to_string());
        es.insert("premium".to_string(), "Premium".to_string());
        es.insert("permissions".to_string(), "Permisos".to_string());
        es.insert("audio".to_string(), "Audio".to_string());
        es.insert("language".to_string(), "Idioma".to_string());
        es.insert("theme".to_string(), "Tema".to_string());
        es.insert("device_id".to_string(), "ID del dispositivo".to_string());
        es.insert("connecting".to_string(), "Conectando...".to_string());
        es.insert("connected".to_string(), "Conectado".to_string());
        es.insert("disconnected".to_string(), "Desconectado".to_string());
        self.translations.insert(Language::Spanish, es);
        
        // Add more languages as needed...
    }
    
    /// Get current language
    pub fn current_language(&self) -> Language {
        self.current_language
    }
    
    /// Set language
    pub fn set_language(&mut self, language: Language) {
        self.current_language = language;
    }
    
    /// Translate a key
    pub fn translate(&self, key: &str) -> String {
        self.translations
            .get(&self.current_language)
            .and_then(|lang_map| lang_map.get(key))
            .cloned()
            .unwrap_or_else(|| {
                // Fallback to English
                self.translations
                    .get(&Language::English)
                    .and_then(|lang_map| lang_map.get(key))
                    .cloned()
                    .unwrap_or_else(|| key.to_string())
            })
    }
    
    /// Short alias for translate
    pub fn t(&self, key: &str) -> String {
        self.translate(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_language_codes() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::Hindi.code(), "hi");
        assert_eq!(Language::Spanish.code(), "es");
    }
    
    #[test]
    fn test_translations() {
        let mut manager = LocalizationManager::new();
        
        // English
        assert_eq!(manager.translate("connect"), "Connect");
        
        // Hindi
        manager.set_language(Language::Hindi);
        assert_eq!(manager.translate("connect"), "कनेक्ट करें");
        
        // Spanish
        manager.set_language(Language::Spanish);
        assert_eq!(manager.translate("connect"), "Conectar");
    }
    
    #[test]
    fn test_fallback() {
        let manager = LocalizationManager::new();
        // Non-existent key should return the key itself
        assert_eq!(manager.translate("non_existent_key"), "non_existent_key");
    }
}
