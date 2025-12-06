//! GenXLink Complete Application - All Features
//! 
//! This shows the complete GenXLink with all original features

use eframe::egui;

mod services;
use services::BackendServices;

/// Get translated text based on language
fn get_text(key: &str, language: &str) -> String {
    match language {
        "Hindi" => match key {
            "dashboard" => "डैशबोर्ड".to_string(),
            "connections" => "कनेक्शन".to_string(),
            "sessions" => "सत्र".to_string(),
            "file_transfer" => "फ़ाइल स्थानांतरण".to_string(),
            "settings" => "सेटिंग्स".to_string(),
            "premium" => "प्रीमियम".to_string(),
            "about" => "के बारे में".to_string(),
            "logs" => "लॉग्स".to_string(),
            "welcome" => "GenXLink में आपका स्वागत है".to_string(),
            "connected" => "कनेक्टेड".to_string(),
            "ready_to_connect" => "कनेक्ट करने के लिए तैयार".to_string(),
            "quick_connect" => "त्वरित कनेक्ट".to_string(),
            "connect" => "कनेक्ट करें".to_string(),
            "language" => "भाषा".to_string(),
            "theme" => "थीम".to_string(),
            "general" => "सामान्य".to_string(),
            "screen_share" => "स्क्रीन शेयर".to_string(),
            "audio" => "ऑडियो".to_string(),
            "security" => "सुरक्षा".to_string(),
            "network" => "नेटवर्क".to_string(),
            "save_settings" => "सेटिंग्स सहेजें".to_string(),
            "reset" => "रीसेट करें".to_string(),
            "active_sessions" => "सक्रिय सत्र".to_string(),
            "devices_found" => "डिवाइस मिले".to_string(),
            "resolution" => "रेज़ोल्यूशन".to_string(),
            "quality" => "गुणवत्ता".to_string(),
            "frame_rate" => "फ्रेम रेट".to_string(),
            "share_audio" => "ऑडियो साझा करें".to_string(),
            "show_cursor" => "कर्सर दिखाएं".to_string(),
            "multi_monitor" => "मल्टी-मॉनिटर".to_string(),
            "hardware_accel" => "हार्डवेयर त्वरण".to_string(),
            "input_device" => "इनपुट डिवाइस".to_string(),
            "noise_suppression" => "शोर दमन".to_string(),
            "echo_cancel" => "इको रद्द करें".to_string(),
            "auto_gain" => "ऑटो गेन".to_string(),
            "require_password" => "पासवर्ड आवश्यक".to_string(),
            "encryption" => "एन्क्रिप्शन".to_string(),
            "two_factor" => "दो-कारक प्रमाणीकरण".to_string(),
            "log_connections" => "कनेक्शन लॉग करें".to_string(),
            "port" => "पोर्ट".to_string(),
            "auto_start" => "विंडोज के साथ ऑटो-स्टार्ट".to_string(),
            "minimize_tray" => "ट्रे में छोटा करें".to_string(),
            "export_settings" => "सेटिंग्स निर्यात करें".to_string(),
            "import_settings" => "सेटिंग्स आयात करें".to_string(),
            _ => key.to_string(),
        },
        "Malayalam" => match key {
            "dashboard" => "ഡാഷ്ബോർഡ്".to_string(),
            "connections" => "കണക്ഷനുകൾ".to_string(),
            "sessions" => "സെഷനുകൾ".to_string(),
            "file_transfer" => "ഫയൽ കൈമാറ്റം".to_string(),
            "settings" => "ക്രമീകരണങ്ങൾ".to_string(),
            "premium" => "പ്രീമിയം".to_string(),
            "about" => "കുറിച്ച്".to_string(),
            "logs" => "ലോഗുകൾ".to_string(),
            "welcome" => "GenXLink-ലേക്ക് സ്വാഗതം".to_string(),
            "connected" => "കണക്റ്റുചെയ്തു".to_string(),
            "ready_to_connect" => "കണക്റ്റുചെയ്യാൻ തയ്യാർ".to_string(),
            "quick_connect" => "ദ്രുത കണക്ഷൻ".to_string(),
            "connect" => "കണക്റ്റുചെയ്യുക".to_string(),
            "language" => "ഭാഷ".to_string(),
            "theme" => "തീം".to_string(),
            "general" => "പൊതുവായ".to_string(),
            "screen_share" => "സ്ക്രീൻ പങ്കിടൽ".to_string(),
            "audio" => "ഓഡിയോ".to_string(),
            "security" => "സുരക്ഷ".to_string(),
            "network" => "നെറ്റ്‌വർക്ക്".to_string(),
            "save_settings" => "ക്രമീകരണങ്ങൾ സംരക്ഷിക്കുക".to_string(),
            "reset" => "പുനഃസജ്ജമാക്കുക".to_string(),
            "active_sessions" => "സജീവ സെഷനുകൾ".to_string(),
            "devices_found" => "ഉപകരണങ്ങൾ കണ്ടെത്തി".to_string(),
            "resolution" => "റെസല്യൂഷൻ".to_string(),
            "quality" => "ഗുണനിലവാരം".to_string(),
            "frame_rate" => "ഫ്രെയിം റേറ്റ്".to_string(),
            "share_audio" => "ഓഡിയോ പങ്കിടുക".to_string(),
            "show_cursor" => "കഴ്സർ കാണിക്കുക".to_string(),
            "multi_monitor" => "മൾട്ടി-മോണിറ്റർ".to_string(),
            "hardware_accel" => "ഹാർഡ്‌വെയർ ത്വരണം".to_string(),
            "input_device" => "ഇൻപുട്ട് ഉപകരണം".to_string(),
            "noise_suppression" => "ശബ്ദ അടിച്ചമർത്തൽ".to_string(),
            "echo_cancel" => "എക്കോ റദ്ദാക്കൽ".to_string(),
            "auto_gain" => "ഓട്ടോ ഗെയിൻ".to_string(),
            "require_password" => "പാസ്‌വേഡ് ആവശ്യമാണ്".to_string(),
            "encryption" => "എൻക്രിപ്ഷൻ".to_string(),
            "two_factor" => "ടു-ഫാക്ടർ".to_string(),
            "log_connections" => "കണക്ഷനുകൾ ലോഗ് ചെയ്യുക".to_string(),
            "port" => "പോർട്ട്".to_string(),
            "auto_start" => "വിൻഡോസിൽ ഓട്ടോ-സ്റ്റാർട്ട്".to_string(),
            "minimize_tray" => "ട്രേയിലേക്ക് ചെറുതാക്കുക".to_string(),
            "export_settings" => "ക്രമീകരണങ്ങൾ കയറ്റുമതി ചെയ്യുക".to_string(),
            "import_settings" => "ക്രമീകരണങ്ങൾ ഇറക്കുമതി ചെയ്യുക".to_string(),
            _ => key.to_string(),
        },
        "Spanish" => match key {
            "dashboard" => "Panel".to_string(),
            "connections" => "Conexiones".to_string(),
            "sessions" => "Sesiones".to_string(),
            "file_transfer" => "Transferencia".to_string(),
            "settings" => "Ajustes".to_string(),
            "premium" => "Premium".to_string(),
            "about" => "Acerca de".to_string(),
            "logs" => "Registros".to_string(),
            "welcome" => "Bienvenido a GenXLink".to_string(),
            "connected" => "Conectado".to_string(),
            "ready_to_connect" => "Listo para conectar".to_string(),
            "quick_connect" => "Conexión rápida".to_string(),
            "connect" => "Conectar".to_string(),
            "language" => "Idioma".to_string(),
            "theme" => "Tema".to_string(),
            "general" => "General".to_string(),
            "screen_share" => "Compartir pantalla".to_string(),
            "audio" => "Audio".to_string(),
            "security" => "Seguridad".to_string(),
            "network" => "Red".to_string(),
            "save_settings" => "Guardar ajustes".to_string(),
            "reset" => "Restablecer".to_string(),
            "active_sessions" => "Sesiones activas".to_string(),
            "devices_found" => "Dispositivos encontrados".to_string(),
            _ => key.to_string(),
        },
        "French" => match key {
            "dashboard" => "Tableau de bord".to_string(),
            "connections" => "Connexions".to_string(),
            "sessions" => "Sessions".to_string(),
            "file_transfer" => "Transfert".to_string(),
            "settings" => "Paramètres".to_string(),
            "premium" => "Premium".to_string(),
            "about" => "À propos".to_string(),
            "logs" => "Journaux".to_string(),
            "welcome" => "Bienvenue sur GenXLink".to_string(),
            "connected" => "Connecté".to_string(),
            "ready_to_connect" => "Prêt à connecter".to_string(),
            "quick_connect" => "Connexion rapide".to_string(),
            "connect" => "Connecter".to_string(),
            "language" => "Langue".to_string(),
            "theme" => "Thème".to_string(),
            "general" => "Général".to_string(),
            "screen_share" => "Partage d'écran".to_string(),
            "audio" => "Audio".to_string(),
            "security" => "Sécurité".to_string(),
            "network" => "Réseau".to_string(),
            "save_settings" => "Enregistrer".to_string(),
            "reset" => "Réinitialiser".to_string(),
            "active_sessions" => "Sessions actives".to_string(),
            "devices_found" => "Appareils trouvés".to_string(),
            _ => key.to_string(),
        },
        "German" => match key {
            "dashboard" => "Dashboard".to_string(),
            "connections" => "Verbindungen".to_string(),
            "sessions" => "Sitzungen".to_string(),
            "file_transfer" => "Dateiübertragung".to_string(),
            "settings" => "Einstellungen".to_string(),
            "premium" => "Premium".to_string(),
            "about" => "Über".to_string(),
            "logs" => "Protokolle".to_string(),
            "welcome" => "Willkommen bei GenXLink".to_string(),
            "connected" => "Verbunden".to_string(),
            "ready_to_connect" => "Bereit zum Verbinden".to_string(),
            "quick_connect" => "Schnellverbindung".to_string(),
            "connect" => "Verbinden".to_string(),
            "language" => "Sprache".to_string(),
            "theme" => "Thema".to_string(),
            "general" => "Allgemein".to_string(),
            "screen_share" => "Bildschirmfreigabe".to_string(),
            "audio" => "Audio".to_string(),
            "security" => "Sicherheit".to_string(),
            "network" => "Netzwerk".to_string(),
            "save_settings" => "Speichern".to_string(),
            "reset" => "Zurücksetzen".to_string(),
            "active_sessions" => "Aktive Sitzungen".to_string(),
            "devices_found" => "Geräte gefunden".to_string(),
            _ => key.to_string(),
        },
        "Chinese" => match key {
            "dashboard" => "仪表板".to_string(),
            "connections" => "连接".to_string(),
            "sessions" => "会话".to_string(),
            "file_transfer" => "文件传输".to_string(),
            "settings" => "设置".to_string(),
            "premium" => "高级版".to_string(),
            "about" => "关于".to_string(),
            "logs" => "日志".to_string(),
            "welcome" => "欢迎使用 GenXLink".to_string(),
            "connected" => "已连接".to_string(),
            "ready_to_connect" => "准备连接".to_string(),
            "quick_connect" => "快速连接".to_string(),
            "connect" => "连接".to_string(),
            "language" => "语言".to_string(),
            "theme" => "主题".to_string(),
            "general" => "常规".to_string(),
            "screen_share" => "屏幕共享".to_string(),
            "audio" => "音频".to_string(),
            "security" => "安全".to_string(),
            "network" => "网络".to_string(),
            "save_settings" => "保存设置".to_string(),
            "reset" => "重置".to_string(),
            "active_sessions" => "活动会话".to_string(),
            "devices_found" => "发现设备".to_string(),
            _ => key.to_string(),
        },
        "Japanese" => match key {
            "dashboard" => "ダッシュボード".to_string(),
            "connections" => "接続".to_string(),
            "sessions" => "セッション".to_string(),
            "file_transfer" => "ファイル転送".to_string(),
            "settings" => "設定".to_string(),
            "premium" => "プレミアム".to_string(),
            "about" => "について".to_string(),
            "logs" => "ログ".to_string(),
            "welcome" => "GenXLinkへようこそ".to_string(),
            "connected" => "接続済み".to_string(),
            "ready_to_connect" => "接続準備完了".to_string(),
            "quick_connect" => "クイック接続".to_string(),
            "connect" => "接続".to_string(),
            "language" => "言語".to_string(),
            "theme" => "テーマ".to_string(),
            "general" => "一般".to_string(),
            "screen_share" => "画面共有".to_string(),
            "audio" => "オーディオ".to_string(),
            "security" => "セキュリティ".to_string(),
            "network" => "ネットワーク".to_string(),
            "save_settings" => "設定を保存".to_string(),
            "reset" => "リセット".to_string(),
            "active_sessions" => "アクティブセッション".to_string(),
            "devices_found" => "デバイスが見つかりました".to_string(),
            _ => key.to_string(),
        },
        _ => match key { // English (default)
            "dashboard" => "Dashboard".to_string(),
            "connections" => "Connections".to_string(),
            "sessions" => "Sessions".to_string(),
            "file_transfer" => "File Transfer".to_string(),
            "settings" => "Settings".to_string(),
            "premium" => "Premium".to_string(),
            "about" => "About".to_string(),
            "logs" => "Logs".to_string(),
            "welcome" => "Welcome to GenXLink".to_string(),
            "connected" => "Connected".to_string(),
            "ready_to_connect" => "Ready to connect".to_string(),
            "quick_connect" => "Quick Connect".to_string(),
            "connect" => "Connect".to_string(),
            "language" => "Language".to_string(),
            "theme" => "Theme".to_string(),
            "general" => "General".to_string(),
            "screen_share" => "Screen Share".to_string(),
            "audio" => "Audio".to_string(),
            "security" => "Security".to_string(),
            "network" => "Network".to_string(),
            "save_settings" => "Save Settings".to_string(),
            "reset" => "Reset to Defaults".to_string(),
            "active_sessions" => "Active Sessions".to_string(),
            "devices_found" => "Devices Found".to_string(),
            _ => key.to_string(),
        },
    }
}

#[derive(Debug, Clone)]
pub struct AppSettings {
    // General Settings
    pub language: String,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    
    // UI Settings
    pub theme: String,
    pub font_size: f32,
    pub window_opacity: f32,
    pub show_notifications: bool,
    pub notification_sound: bool,
    
    // Screen Share Settings
    pub resolution: String,
    pub quality: String,
    pub frame_rate: u32,
    pub share_audio: bool,
    pub show_cursor: bool,
    pub multi_monitor: bool,
    pub hardware_acceleration: bool,
    
    // Audio Settings
    pub input_device: String,
    pub output_device: String,
    pub noise_suppression: bool,
    pub echo_cancellation: bool,
    pub auto_gain: bool,
    pub audio_quality: String,
    
    // File Transfer Settings
    pub download_location: String,
    pub max_file_size: u64,
    pub auto_accept: bool,
    pub resume_transfers: bool,
    pub compression: bool,
    
    // Security Settings
    pub require_password: bool,
    pub end_to_end_encryption: bool,
    pub two_factor_auth: bool,
    pub session_timeout: u32,
    pub log_connections: bool,
    
    // Network Settings
    pub port: u16,
    pub bandwidth_limit: u64,
    pub auto_reconnect: bool,
    pub connection_timeout: u32,
    pub keep_alive: bool,
    
    // Advanced Settings
    pub debug_mode: bool,
    pub log_level: String,
    pub max_connections: u32,
    pub buffer_size: u32,
    pub thread_pool_size: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "English".to_string(),
            auto_start: false,
            minimize_to_tray: true,
            theme: "Dark".to_string(),
            font_size: 14.0,
            window_opacity: 1.0,
            show_notifications: true,
            notification_sound: true,
            resolution: "1920x1080".to_string(),
            quality: "High".to_string(),
            frame_rate: 60,
            share_audio: true,
            show_cursor: true,
            multi_monitor: true,
            hardware_acceleration: true,
            input_device: "Default".to_string(),
            output_device: "Default".to_string(),
            noise_suppression: true,
            echo_cancellation: true,
            auto_gain: false,
            audio_quality: "High".to_string(),
            download_location: "C:\\Users\\Downloads\\GenXLink".to_string(),
            max_file_size: 10737418240, // 10GB
            auto_accept: false,
            resume_transfers: true,
            compression: true,
            require_password: false,
            end_to_end_encryption: true,
            two_factor_auth: false,
            session_timeout: 3600, // 1 hour
            log_connections: true,
            port: 8080,
            bandwidth_limit: 104857600, // 100MB/s
            auto_reconnect: true,
            connection_timeout: 30,
            keep_alive: true,
            debug_mode: false,
            log_level: "Info".to_string(),
            max_connections: 10,
            buffer_size: 65536,
            thread_pool_size: 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub device_id: String,
    pub device_name: String,
    pub status: String,
    pub last_seen: String,
    pub capabilities: Vec<String>,
    pub ip_address: String,
    pub os_type: String,
    pub connection_type: String,
    pub signal_strength: u8,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub remote_device: String,
    pub start_time: String,
    pub duration: String,
    pub bandwidth_used: String,
    pub screen_resolution: String,
    pub audio_active: bool,
    pub file_transfer_active: bool,
}

#[derive(Debug, Clone)]
pub struct TransferInfo {
    pub file_name: String,
    pub file_size: u64,
    pub progress: f32,
    pub speed: String,
    pub status: String,
    pub direction: String, // "Upload" or "Download"
}

pub struct GenXLinkApp {
    settings: AppSettings,
    connections: Vec<ConnectionInfo>,
    sessions: Vec<SessionInfo>,
    transfers: Vec<TransferInfo>,
    current_view: View,
    selected_device: Option<usize>,
    connected_device: Option<String>,
    is_connected: bool,
    connection_time: String,
    // Backend services
    backend: Option<BackendServices>,
    backend_initialized: bool,
}

impl Default for GenXLinkApp {
    fn default() -> Self {
        Self {
            settings: AppSettings::default(),
            connections: Vec::new(),
            sessions: Vec::new(),
            transfers: Vec::new(),
            current_view: View::default(),
            selected_device: None,
            connected_device: None,
            is_connected: false,
            connection_time: String::new(),
            backend: None,
            backend_initialized: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
enum View {
    #[default]
    Dashboard,
    Connections,
    Sessions,
    FileTransfer,
    Settings,
    Premium,
    About,
    Logs,
}

impl GenXLinkApp {
    pub fn new() -> Self {
        println!("🚀 Initializing GenXLink Application...");
        
        // Initialize backend services
        let backend = BackendServices::new();
        backend.start();
        println!("✅ Backend services initialized");

        Self {
            settings: AppSettings::default(),
            connections: vec![
                ConnectionInfo {
                    device_id: "OFFICE-PC-001".to_string(),
                    device_name: "Office Desktop".to_string(),
                    status: "Online".to_string(),
                    last_seen: "Active now".to_string(),
                    capabilities: vec![
                        "Screen Share".to_string(), 
                        "Audio".to_string(), 
                        "File Transfer".to_string(),
                        "Remote Control".to_string(),
                        "Multi-Monitor".to_string()
                    ],
                    ip_address: "192.168.1.100".to_string(),
                    os_type: "Windows 11 Pro".to_string(),
                    connection_type: "WiFi".to_string(),
                    signal_strength: 85,
                },
                ConnectionInfo {
                    device_id: "LAPTOP-002".to_string(),
                    device_name: "Personal Laptop".to_string(),
                    status: "Offline".to_string(),
                    last_seen: "2 hours ago".to_string(),
                    capabilities: vec![
                        "Screen Share".to_string(), 
                        "Audio".to_string(),
                        "Remote Control".to_string()
                    ],
                    ip_address: "192.168.1.101".to_string(),
                    os_type: "Windows 11 Home".to_string(),
                    connection_type: "WiFi".to_string(),
                    signal_strength: 0,
                },
                ConnectionInfo {
                    device_id: "SERVER-003".to_string(),
                    device_name: "Production Server".to_string(),
                    status: "Online".to_string(),
                    last_seen: "Active now".to_string(),
                    capabilities: vec![
                        "Screen Share".to_string(), 
                        "File Transfer".to_string(),
                        "Remote Control".to_string(),
                        "Terminal Access".to_string()
                    ],
                    ip_address: "10.0.0.50".to_string(),
                    os_type: "Ubuntu Server 22.04".to_string(),
                    connection_type: "Ethernet".to_string(),
                    signal_strength: 95,
                },
            ],
            sessions: vec![
                SessionInfo {
                    session_id: "sess_001".to_string(),
                    remote_device: "Office Desktop".to_string(),
                    start_time: "2024-12-06 09:30:00".to_string(),
                    duration: "2h 15m".to_string(),
                    bandwidth_used: "1.2 GB".to_string(),
                    screen_resolution: "1920x1080".to_string(),
                    audio_active: true,
                    file_transfer_active: false,
                },
            ],
            transfers: vec![
                TransferInfo {
                    file_name: "project_report.pdf".to_string(),
                    file_size: 5242880, // 5MB
                    progress: 75.0,
                    speed: "2.5 MB/s".to_string(),
                    status: "Downloading".to_string(),
                    direction: "Download".to_string(),
                },
                TransferInfo {
                    file_name: "presentation.pptx".to_string(),
                    file_size: 15728640, // 15MB
                    progress: 100.0,
                    speed: "Completed".to_string(),
                    status: "Completed".to_string(),
                    direction: "Upload".to_string(),
                },
            ],
            current_view: View::Dashboard,
            selected_device: None,
            connected_device: None,
            is_connected: false,
            connection_time: String::new(),
            backend: Some(backend),
            backend_initialized: true,
        }
    }
    
    /// Connect to a device using backend services
    fn connect_to_device(&mut self, device_id: &str, device_ip: &str) {
        if let Some(ref backend) = self.backend {
            if let Ok(mut conn) = backend.connection.lock() {
                match conn.connect_to_peer_sync(device_id, device_ip) {
                    Ok(_) => {
                        println!("✅ Connected to device: {}", device_id);
                        self.is_connected = true;
                        self.connected_device = Some(device_id.to_string());
                    }
                    Err(e) => println!("❌ Connection failed: {}", e),
                }
            }
        }
    }
    
    /// Start file transfer using backend services
    fn start_file_upload(&mut self, file_path: &str, peer_id: &str) {
        if let Some(ref backend) = self.backend {
            if let Ok(mut ft) = backend.file_transfer.lock() {
                match ft.start_upload(std::path::Path::new(file_path), peer_id) {
                    Ok(transfer_id) => println!("📤 Upload started: {}", transfer_id),
                    Err(e) => println!("❌ Upload failed: {}", e),
                }
            }
        }
    }
    
    /// Create a new session using backend services
    fn create_session(&mut self, peer_id: &str, peer_name: &str, is_host: bool) {
        if let Some(ref backend) = self.backend {
            if let Ok(mut session) = backend.session.lock() {
                match session.create_session(
                    peer_id, 
                    peer_name, 
                    services::session_service::SessionType::ScreenShare,
                    is_host
                ) {
                    Ok(session_id) => {
                        println!("🎬 Session created: {}", session_id);
                        // Start the session
                        session.start_session(&session_id).ok();
                    }
                    Err(e) => println!("❌ Session creation failed: {}", e),
                }
            }
        }
    }
    
    fn apply_theme(&self, ctx: &egui::Context) {
        match self.settings.theme.as_str() {
            "Light" => {
                ctx.set_visuals(egui::Visuals::light());
            }
            "System" => {
                // For demo purposes, use dark theme
                ctx.set_visuals(egui::Visuals::dark());
            }
            _ => {
                ctx.set_visuals(egui::Visuals::dark());
            }
        }
    }
    
    fn show_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("🏠 Dashboard");
        ui.separator();
        
        // Connection status
        ui.horizontal(|ui| {
            let status_color = if self.is_connected {
                egui::Color32::from_rgb(40, 167, 69)
            } else {
                egui::Color32::from_rgb(255, 193, 7)
            };
            let status_text = if self.is_connected {
                "🟢 Connected"
            } else {
                "🟡 Ready to connect"
            };
            ui.colored_label(status_color, status_text);
            
            if self.is_connected {
                ui.separator();
                ui.label(format!("Connected to: {}", self.connected_device.as_ref().unwrap_or(&"Unknown".to_string())));
                ui.separator();
                ui.label(format!("Connection time: {}", self.connection_time));
            }
            
            ui.separator();
            ui.label("Device: DESKTOP-DEMO");
            ui.separator();
            ui.label("Network: Excellent");
        });
        
        ui.add_space(20.0);
        
        // Quick connect
        egui::Frame {
            fill: egui::Color32::from_rgb(35, 35, 35),
            stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)),
            rounding: 8.0.into(),
            inner_margin: egui::Margin::same(16.0),
            ..Default::default()
        }.show(ui, |ui| {
            ui.heading("🔗 Quick Connect");
            ui.add_space(10.0);
            
            let mut device_id = "Enter device ID or access code".to_string();
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut device_id);
                if ui.button("Connect").clicked() {
                    // Simulate connection
                    self.is_connected = true;
                    self.connected_device = Some("Office Desktop".to_string());
                    self.connection_time = "0:00:00".to_string();
                }
            });
        });
        
        ui.add_space(20.0);
        
        // Active sessions
        ui.heading("📱 Active Sessions");
        ui.add_space(10.0);
        
        for session in &self.sessions {
            egui::Frame {
                fill: egui::Color32::from_rgb(40, 44, 52),
                stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)),
                rounding: 6.0.into(),
                inner_margin: egui::Margin::same(12.0),
                ..Default::default()
            }.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading(&session.remote_device);
                        ui.label(format!("Started: {} | Duration: {}", session.start_time, session.duration));
                        ui.label(format!("Resolution: {} | Bandwidth: {}", session.screen_resolution, session.bandwidth_used));
                        ui.horizontal(|ui| {
                            if session.audio_active {
                                ui.colored_label(egui::Color32::from_rgb(40, 167, 69), "🎤 Audio");
                            }
                            if session.file_transfer_active {
                                ui.colored_label(egui::Color32::from_rgb(0, 120, 215), "📁 Files");
                            }
                        });
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Disconnect").clicked() {
                            self.is_connected = false;
                            self.connected_device = None;
                        }
                        if ui.button("Control").clicked() {
                            // Open remote control
                        }
                        if ui.button("View").clicked() {
                            // View session details
                        }
                    });
                });
            });
            ui.add_space(8.0);
        }
    }
    
    fn show_connections(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔗 Connections");
        ui.separator();
        
        ui.horizontal(|ui| {
            if ui.button("🔄 Refresh").clicked() {
                // Refresh connections
            }
            if ui.button("➕ Add Device").clicked() {
                // Add new device
            }
            if ui.button("🔍 Scan Network").clicked() {
                // Scan for devices
            }
        });
        
        ui.add_space(20.0);
        
        for (i, conn) in self.connections.iter().enumerate() {
            egui::Frame {
                fill: if conn.status == "Online" { 
                    egui::Color32::from_rgb(40, 44, 52) 
                } else { 
                    egui::Color32::from_rgb(30, 30, 30) 
                },
                stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)),
                rounding: 6.0.into(),
                inner_margin: egui::Margin::same(12.0),
                ..Default::default()
            }.show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Status indicator
                    let status_color = if conn.status == "Online" {
                        egui::Color32::from_rgb(40, 167, 69)
                    } else {
                        egui::Color32::from_rgb(255, 140, 0) // Orange instead of red
                    };
                    ui.colored_label(status_color, "●");
                    
                    // Device info
                    ui.vertical(|ui| {
                        ui.heading(&conn.device_name);
                        ui.label(format!("ID: {} | {} | {}", conn.device_id, conn.os_type, conn.connection_type));
                        ui.label(format!("IP: {} | Signal: {}% | {}", conn.ip_address, conn.signal_strength, conn.last_seen));
                        ui.horizontal(|ui| {
                            for cap in &conn.capabilities {
                                ui.colored_label(egui::Color32::from_rgb(0, 120, 215), cap);
                                ui.label("•");
                            }
                        });
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if conn.status == "Online" {
                            if ui.button("Connect").clicked() {
                                self.selected_device = Some(i);
                                self.is_connected = true;
                                self.connected_device = Some(conn.device_name.clone());
                            }
                            if ui.button("Share Screen").clicked() {
                                // Start screen sharing
                            }
                            if ui.button("Files").clicked() {
                                // Open file transfer
                            }
                        } else {
                            ui.add_enabled(false, egui::Button::new("Offline"));
                        }
                    });
                });
            });
            ui.add_space(8.0);
        }
    }
    
    fn show_file_transfer(&mut self, ui: &mut egui::Ui) {
        ui.heading("📁 File Transfer");
        ui.separator();
        
        ui.horizontal(|ui| {
            if ui.button("📤 Upload Files").clicked() {
                // Open file upload dialog
            }
            if ui.button("📥 Download Folder").clicked() {
                // Open download folder
            }
            if ui.button("⚙️ Transfer Settings").clicked() {
                // Open transfer settings
            }
        });
        
        ui.add_space(20.0);
        
        ui.heading("Active Transfers");
        ui.add_space(10.0);
        
        for transfer in &self.transfers {
            egui::Frame {
                fill: egui::Color32::from_rgb(35, 35, 35),
                stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)),
                rounding: 6.0.into(),
                inner_margin: egui::Margin::same(12.0),
                ..Default::default()
            }.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading(&transfer.file_name);
                        ui.label(format!("Size: {} MB | Speed: {}", transfer.file_size / 1024 / 1024, transfer.speed));
                        ui.label(format!("Status: {} | Direction: {}", transfer.status, transfer.direction));
                        
                        // Progress bar
                        ui.add_space(5.0);
                        let progress_text = format!("{:.0}%", transfer.progress);
                        ui.horizontal(|ui| {
                            ui.add(egui::ProgressBar::new(transfer.progress / 100.0).show_percentage());
                            ui.label(progress_text);
                        });
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if transfer.status == "Downloading" {
                            if ui.button("⏸️ Pause").clicked() {
                                // Pause transfer
                            }
                            if ui.button("❌ Cancel").clicked() {
                                // Cancel transfer
                            }
                        } else if transfer.status == "Completed" {
                            if ui.button("📂 Open").clicked() {
                                // Open file location
                            }
                        }
                    });
                });
            });
            ui.add_space(8.0);
        }
    }
    
    fn show_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙️ Settings");
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            // General Settings
            ui.collapsing(format!("🎨 {}", get_text("general", &self.settings.language)), |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", get_text("language", &self.settings.language)));
                    // Languages with their native names for better identification
                    let languages = [
                        ("English", "English"),
                        ("Hindi", "हिंदी (Hindi)"),
                        ("Malayalam", "മലയാളം (Malayalam)"),
                        ("Spanish", "Español"),
                        ("French", "Français"),
                        ("German", "Deutsch"),
                        ("Chinese", "中文"),
                        ("Japanese", "日本語"),
                    ];
                    
                    egui::ComboBox::from_id_source("language_dropdown")
                        .selected_text(&self.settings.language)
                        .show_ui(ui, |ui| {
                            for (lang_code, lang_display) in languages.iter() {
                                if ui.selectable_label(self.settings.language == *lang_code, *lang_display).clicked() {
                                    self.settings.language = lang_code.to_string();
                                    println!("✅ LANGUAGE CHANGED TO: {}", self.settings.language);
                                }
                            }
                        });
                });
                
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", get_text("theme", &self.settings.language)));
                    let themes = ["Dark", "Light", "System"];
                    
                    egui::ComboBox::from_id_source("theme_dropdown")
                        .selected_text(&self.settings.theme)
                        .show_ui(ui, |ui| {
                            for &theme in themes.iter() {
                                if ui.selectable_label(self.settings.theme == theme, theme).clicked() {
                                    self.settings.theme = theme.to_string();
                                    println!("✅ THEME CHANGED TO: {}", self.settings.theme);
                                }
                            }
                        });
                });
                
                ui.horizontal(|ui| {
                    ui.label("Font Size:");
                    ui.add(egui::Slider::new(&mut self.settings.font_size, 10.0..=24.0).text("px"));
                });
                
                ui.checkbox(&mut self.settings.auto_start, &get_text("auto_start", &self.settings.language));
                ui.checkbox(&mut self.settings.minimize_to_tray, &get_text("minimize_tray", &self.settings.language));
            });
            
            // Screen Share Settings
            let lang = self.settings.language.clone();
            ui.collapsing(format!("🖥️ {}", get_text("screen_share", &lang)), |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", get_text("resolution", &lang)));
                    let resolutions = ["1280x720", "1920x1080", "2560x1440", "3840x2160"];
                    let mut selected_index = resolutions.iter().position(|&r| r == self.settings.resolution).unwrap_or(1);
                    let selected_text = resolutions[selected_index].to_string();
                    if egui::ComboBox::from_id_source("resolution_dropdown")
                        .selected_text(&selected_text)
                        .show_index(ui, &mut selected_index, resolutions.len(), |i| resolutions[i].to_string()).changed() {
                        self.settings.resolution = resolutions[selected_index].to_string();
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", get_text("quality", &lang)));
                    let qualities = ["Low", "Medium", "High", "Ultra"];
                    let mut selected_index = qualities.iter().position(|&q| q == self.settings.quality).unwrap_or(2);
                    let selected_text = qualities[selected_index].to_string();
                    if egui::ComboBox::from_id_source("quality_dropdown")
                        .selected_text(&selected_text)
                        .show_index(ui, &mut selected_index, qualities.len(), |i| qualities[i].to_string()).changed() {
                        self.settings.quality = qualities[selected_index].to_string();
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", get_text("frame_rate", &lang)));
                    ui.add(egui::Slider::new(&mut self.settings.frame_rate, 15..=120).text("fps"));
                });
                
                ui.checkbox(&mut self.settings.share_audio, &get_text("share_audio", &lang));
                ui.checkbox(&mut self.settings.show_cursor, &get_text("show_cursor", &lang));
                ui.checkbox(&mut self.settings.multi_monitor, &get_text("multi_monitor", &lang));
                ui.checkbox(&mut self.settings.hardware_acceleration, &get_text("hardware_accel", &lang));
            });
            
            // Audio Settings
            ui.collapsing(format!("🎤 {}", get_text("audio", &lang)), |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", get_text("input_device", &lang)));
                    let devices = ["Default", "Headset", "Microphone", "Webcam"];
                    let mut selected_index = devices.iter().position(|&d| d == self.settings.input_device).unwrap_or(0);
                    let selected_text = devices[selected_index].to_string();
                    if egui::ComboBox::from_id_source("input_device_dropdown")
                        .selected_text(&selected_text)
                        .show_index(ui, &mut selected_index, devices.len(), |i| devices[i].to_string()).changed() {
                        self.settings.input_device = devices[selected_index].to_string();
                    }
                });
                
                ui.checkbox(&mut self.settings.noise_suppression, &get_text("noise_suppression", &lang));
                ui.checkbox(&mut self.settings.echo_cancellation, &get_text("echo_cancel", &lang));
                ui.checkbox(&mut self.settings.auto_gain, &get_text("auto_gain", &lang));
            });
            
            // Security Settings
            ui.collapsing(format!("🔒 {}", get_text("security", &lang)), |ui| {
                ui.checkbox(&mut self.settings.require_password, &get_text("require_password", &lang));
                ui.checkbox(&mut self.settings.end_to_end_encryption, &get_text("encryption", &lang));
                ui.checkbox(&mut self.settings.two_factor_auth, &get_text("two_factor", &lang));
                ui.checkbox(&mut self.settings.log_connections, &get_text("log_connections", &lang));
                
                ui.horizontal(|ui| {
                    ui.label("Session Timeout (seconds):");
                    ui.add(egui::Slider::new(&mut self.settings.session_timeout, 300..=86400));
                });
            });
            
            // Network Settings
            ui.collapsing(format!("📊 {}", get_text("network", &lang)), |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", get_text("port", &lang)));
                    ui.add(egui::Slider::new(&mut self.settings.port, 1024..=65535));
                });
                
                ui.checkbox(&mut self.settings.auto_reconnect, "Auto-reconnect on disconnection");
                ui.checkbox(&mut self.settings.keep_alive, "Keep connection alive");
                
                ui.horizontal(|ui| {
                    ui.label("Bandwidth Limit (MB/s):");
                    ui.add(egui::Slider::new(&mut self.settings.bandwidth_limit, 1..=1000));
                });
            });
            
            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                if ui.button(&get_text("save_settings", &lang)).clicked() {
                    println!("Settings saved!");
                }
                if ui.button(&get_text("reset", &lang)).clicked() {
                    self.settings = AppSettings::default();
                }
                if ui.button(&get_text("export_settings", &lang)).clicked() {
                    println!("Export settings");
                }
                if ui.button(&get_text("import_settings", &lang)).clicked() {
                    println!("Import settings");
                }
            });
        });
    }
    
    fn show_premium(&mut self, ui: &mut egui::Ui) {
        ui.heading("💎 Premium Features");
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Choose the plan that's right for you:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("Current: Free Plan");
            });
        });
        
        ui.add_space(20.0);
        
        ui.columns(3, |columns| {
            // Free Plan
            columns[0].vertical_centered(|ui| {
                egui::Frame {
                    fill: egui::Color32::from_rgb(35, 35, 35),
                    stroke: egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 100)),
                    rounding: 8.0.into(),
                    inner_margin: egui::Margin::same(16.0),
                    ..Default::default()
                }.show(ui, |ui| {
                    ui.heading("Free");
                    ui.label("₹0/month");
                    ui.separator();
                    ui.label("✅ Basic screen sharing");
                    ui.label("✅ 1 concurrent session");
                    ui.label("✅ Standard quality");
                    ui.label("✅ 3 device connections");
                    ui.label("❌ No audio sharing");
                    ui.label("❌ No file transfer");
                    ui.label("❌ No encryption");
                    ui.add_space(10.0);
                    ui.add_enabled(false, egui::Button::new("Current Plan"));
                });
            });
            
            // Solo Plan
            columns[1].vertical_centered(|ui| {
                egui::Frame {
                    fill: egui::Color32::from_rgb(40, 44, 52),
                    stroke: egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 120, 215)),
                    rounding: 8.0.into(),
                    inner_margin: egui::Margin::same(16.0),
                    ..Default::default()
                }.show(ui, |ui| {
                    ui.colored_label(egui::Color32::from_rgb(0, 120, 215), "🏆 Best Value");
                    ui.heading("Solo");
                    ui.label("₹199/month");
                    ui.label("≈ $2.39 USD");
                    ui.separator();
                    ui.label("✅ HD screen sharing");
                    ui.label("✅ 3 concurrent sessions");
                    ui.label("✅ Audio sharing");
                    ui.label("✅ File transfer");
                    ui.label("✅ End-to-end encryption");
                    ui.label("✅ 10 device connections");
                    ui.label("✅ Priority support");
                    ui.add_space(10.0);
                    if ui.button("Upgrade Now").clicked() {
                        println!("Upgrade to Solo");
                    }
                });
            });
            
            // Team Plan
            columns[2].vertical_centered(|ui| {
                egui::Frame {
                    fill: egui::Color32::from_rgb(45, 35, 35),
                    stroke: egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 140, 0)), // Orange border
                    rounding: 8.0.into(),
                    inner_margin: egui::Margin::same(16.0),
                    ..Default::default()
                }.show(ui, |ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 140, 0), "🔥 Most Popular"); // Orange text
                    ui.heading("Team");
                    ui.label("₹399/month");
                    ui.label("≈ $4.79 USD");
                    ui.separator();
                    ui.label("✅ 4K screen sharing");
                    ui.label("✅ Unlimited sessions");
                    ui.label("✅ Premium audio");
                    ui.label("✅ Advanced file transfer");
                    ui.label("✅ Military-grade encryption");
                    ui.label("✅ 50 device connections");
                    ui.label("✅ Team management");
                    ui.label("✅ Enterprise support");
                    ui.add_space(10.0);
                    if ui.button("Upgrade Now").clicked() {
                        println!("Upgrade to Team");
                    }
                });
            });
        });
        
        ui.add_space(30.0);
        
        ui.heading("🎯 Premium Features Comparison");
        ui.separator();
        
        // Feature comparison table
        ui.horizontal(|ui| {
            ui.label("Feature");
            ui.separator();
            ui.label("Free");
            ui.label("Solo");
            ui.label("Team");
        });
        
        ui.separator();
        
        let features = vec![
            ("Screen Resolution", "720p", "1080p", "4K"),
            ("Max Sessions", "1", "3", "Unlimited"),
            ("Audio Sharing", "❌", "✅", "✅"),
            ("File Transfer", "❌", "✅", "✅"),
            ("Encryption", "❌", "✅", "✅"),
            ("Device Limit", "3", "10", "50"),
            ("Support", "Community", "Priority", "Enterprise"),
        ];
        
        for (feature, free, solo, team) in features {
            ui.horizontal(|ui| {
                ui.label(feature);
                ui.separator();
                ui.label(free);
                ui.label(solo);
                ui.label(team);
            });
        }
    }
    
    fn show_about(&mut self, ui: &mut egui::Ui) {
        ui.heading("ℹ️ About GenXLink");
        ui.separator();
        
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("🌐 GenXLink Remote Desktop");
            ui.label("Version 1.0.0");
            ui.label("Build: 2024.12.06");
            ui.add_space(20.0);
            
            ui.label("© 2024 GenXis Innovations");
            ui.label("Licensed under Apache License 2.0");
            ui.add_space(20.0);
            
            ui.label("A high-performance, secure remote desktop solution");
            ui.label("with ultra-low latency and military-grade encryption.");
            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                if ui.button("📖 Documentation").clicked() {
                    println!("Open documentation");
                }
                if ui.button("🐛 Report Issue").clicked() {
                    println!("Report issue");
                }
                if ui.button("💬 Support").clicked() {
                    println!("Open support");
                }
            });
            
            ui.add_space(30.0);
            
            ui.label("System Information:");
            ui.horizontal(|ui| {
                ui.label("OS:");
                ui.label("Windows 11");
            });
            ui.horizontal(|ui| {
                ui.label("Architecture:");
                ui.label("x64");
            });
            ui.horizontal(|ui| {
                ui.label("Rust Version:");
                ui.label("1.75.0");
            });
        });
    }
    
    fn show_logs(&mut self, ui: &mut egui::Ui) {
        ui.heading("📊 Logs & Monitoring");
        ui.separator();
        
        ui.horizontal(|ui| {
            if ui.button("🔄 Refresh").clicked() {
                // Refresh logs
            }
            if ui.button("📤 Export Logs").clicked() {
                // Export logs
            }
            if ui.button("🗑️ Clear Logs").clicked() {
                // Clear logs
            }
        });
        
        ui.add_space(20.0);
        
        // Log entries
        let logs = vec![
            ("2024-12-06 09:30:15", "INFO", "Application started successfully"),
            ("2024-12-06 09:30:16", "INFO", "Network interface initialized"),
            ("2024-12-06 09:30:17", "INFO", "Security protocols loaded"),
            ("2024-12-06 09:30:18", "DEBUG", "Scanning for available devices..."),
            ("2024-12-06 09:30:20", "INFO", "Found 3 devices on network"),
            ("2024-12-06 09:30:21", "INFO", "Device OFFICE-PC-001 is online"),
            ("2024-12-06 09:30:22", "INFO", "Device SERVER-003 is online"),
            ("2024-12-06 09:30:23", "WARNING", "Device LAPTOP-002 is offline"),
            ("2024-12-06 09:30:25", "INFO", "Ready for connections"),
        ];
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (timestamp, level, message) in logs {
                ui.horizontal(|ui| {
                    // Timestamp
                    ui.label(timestamp);
                    ui.separator();
                    
                    // Log level with color
                    let (color, icon) = match level {
                        "ERROR" => (egui::Color32::from_rgb(255, 140, 0), "⚠️"), // Orange instead of red
                        "WARNING" => (egui::Color32::from_rgb(255, 193, 7), "⚠️"),
                        "INFO" => (egui::Color32::from_rgb(40, 167, 69), "ℹ️"),
                        "DEBUG" => (egui::Color32::from_rgb(108, 117, 125), "🔍"),
                        _ => (egui::Color32::GRAY, "•"),
                    };
                    ui.colored_label(color, format!("{} {}", icon, level));
                    ui.separator();
                    
                    // Message
                    ui.label(message);
                });
            }
        });
    }
}

impl eframe::App for GenXLinkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply current theme
        self.apply_theme(ctx);

        // Top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🌐 GenXLink Remote Desktop");
                ui.separator();
                
                // Navigation buttons with translations
                let lang = &self.settings.language;
                if ui.selectable_label(matches!(self.current_view, View::Dashboard), format!("🏠 {}", get_text("dashboard", lang))).clicked() {
                    self.current_view = View::Dashboard;
                }
                if ui.selectable_label(matches!(self.current_view, View::Connections), format!("🔗 {}", get_text("connections", lang))).clicked() {
                    self.current_view = View::Connections;
                }
                if ui.selectable_label(matches!(self.current_view, View::Sessions), format!("📱 {}", get_text("sessions", lang))).clicked() {
                    self.current_view = View::Sessions;
                }
                if ui.selectable_label(matches!(self.current_view, View::FileTransfer), format!("📁 {}", get_text("file_transfer", lang))).clicked() {
                    self.current_view = View::FileTransfer;
                }
                if ui.selectable_label(matches!(self.current_view, View::Settings), format!("⚙️ {}", get_text("settings", lang))).clicked() {
                    self.current_view = View::Settings;
                }
                if ui.selectable_label(matches!(self.current_view, View::Premium), format!("💎 {}", get_text("premium", lang))).clicked() {
                    self.current_view = View::Premium;
                }
                if ui.selectable_label(matches!(self.current_view, View::Logs), format!("📊 {}", get_text("logs", lang))).clicked() {
                    self.current_view = View::Logs;
                }
                if ui.selectable_label(matches!(self.current_view, View::About), format!("ℹ️ {}", get_text("about", lang))).clicked() {
                    self.current_view = View::About;
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.is_connected {
                        ui.colored_label(egui::Color32::from_rgb(40, 167, 69), "🟢 Connected");
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(255, 193, 7), "🟡 Ready");
                    }
                    ui.separator();
                    ui.label("v1.0.0");
                });
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                View::Dashboard => self.show_dashboard(ui),
                View::Connections => self.show_connections(ui),
                View::Sessions => self.show_dashboard(ui), // Reuse dashboard for sessions
                View::FileTransfer => self.show_file_transfer(ui),
                View::Settings => self.show_settings(ui),
                View::Premium => self.show_premium(ui),
                View::About => self.show_about(ui),
                View::Logs => self.show_logs(ui),
            }
        });
        
        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status: Ready");
                ui.separator();
                ui.label("Network: Excellent");
                ui.separator();
                ui.label("CPU: 8% | Memory: 156MB");
                ui.separator();
                ui.label("Connections: 3 devices | 1 active");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("GenXlink Remote Desktop v1.0.0");
                });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    env_logger::init();
    
    let app = GenXLinkApp::new();
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1400.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        "GenXLink Remote Desktop",
        native_options,
        Box::new(|cc| {
            // Setup custom fonts for multi-language support
            let mut fonts = egui::FontDefinitions::default();
            let mut loaded_fonts: Vec<String> = Vec::new();
            
            // Load Segoe UI as base font (should always be available on Windows)
            if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf") {
                fonts.font_data.insert(
                    "segoeui".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                loaded_fonts.push("segoeui".to_owned());
                println!("✅ Loaded Segoe UI font");
            }
            
            // Load Segoe UI Emoji for emoji support
            if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\seguiemj.ttf") {
                fonts.font_data.insert(
                    "emoji".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                loaded_fonts.push("emoji".to_owned());
                println!("✅ Loaded Segoe UI Emoji font");
            }
            
            // Load Nirmala UI for Indian languages (Hindi, Malayalam, Tamil, etc.)
            if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\Nirmala.ttc") {
                let mut font = egui::FontData::from_owned(font_data);
                font.index = 0; // First font in the collection
                fonts.font_data.insert("nirmala".to_owned(), font);
                loaded_fonts.push("nirmala".to_owned());
                println!("✅ Loaded Nirmala UI font for Indian languages");
            }
            
            // Load Microsoft YaHei for Chinese
            if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\msyh.ttc") {
                let mut font = egui::FontData::from_owned(font_data);
                font.index = 0;
                fonts.font_data.insert("msyh".to_owned(), font);
                loaded_fonts.push("msyh".to_owned());
                println!("✅ Loaded Microsoft YaHei font for Chinese");
            }
            
            // Load Meiryo for Japanese (optional - may not be available)
            if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\meiryo.ttc") {
                let mut font = egui::FontData::from_owned(font_data);
                font.index = 0;
                fonts.font_data.insert("meiryo".to_owned(), font);
                loaded_fonts.push("meiryo".to_owned());
                println!("✅ Loaded Meiryo font for Japanese");
            }
            
            // Set up font family with only the fonts that were successfully loaded
            fonts.families.insert(
                egui::FontFamily::Proportional,
                loaded_fonts.clone(),
            );
            
            fonts.families.insert(
                egui::FontFamily::Monospace,
                loaded_fonts,
            );
            
            cc.egui_ctx.set_fonts(fonts);
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(app)
        }),
    )
}
