# Phase 5: UI & User Experience - Task List

## Overview

Phase 5 focuses on creating an intuitive and modern graphical user interface for GenXLink, making it easy for users to connect to and control remote devices.

**Timeline:** 2-3 weeks  
**Status:** 🚀 Starting now  
**Prerequisites:** Phase 1-4 ✅

---

## 🎯 Goals

- Create modern, native-looking UI
- Implement device discovery and connection
- Add system tray integration
- Create settings and configuration UI
- Implement connection status indicators
- Add keyboard shortcuts
- Ensure responsive and intuitive UX

---

## 📋 UI Framework Selection

### Options Considered

1. **egui** (Immediate Mode)
   - ✅ Pure Rust
   - ✅ Cross-platform
   - ✅ Easy to use
   - ✅ Good performance
   - ❌ Less native look

2. **iced** (Elm-inspired)
   - ✅ Pure Rust
   - ✅ Modern architecture
   - ✅ Cross-platform
   - ⚠️ Still maturing

3. **Tauri** (Web-based)
   - ✅ Modern web tech
   - ✅ Great flexibility
   - ✅ Beautiful UIs
   - ❌ Larger bundle size

4. **slint** (Declarative)
   - ✅ Pure Rust
   - ✅ Native performance
   - ✅ Good tooling
   - ⚠️ Newer framework

### **Decision: egui**

**Rationale:**
- Pure Rust integration
- Proven stability
- Good documentation
- Active community
- Fast development cycle
- Cross-platform support

---

## 📦 Tasks

### 1. Setup egui Framework (HIGH PRIORITY)

**Estimated Time:** 1 day

#### Dependencies

```toml
[dependencies]
eframe = "0.25"  # egui framework
egui = "0.25"
egui_extras = "0.25"
image = "0.24"  # For icons
```

#### Tasks
- [ ] Add egui dependencies
- [ ] Create main window structure
- [ ] Setup application state
- [ ] Configure window properties
- [ ] Add application icon

---

### 2. Main Window Layout (HIGH PRIORITY)

**File:** `client/windows/src/ui/mod.rs`  
**Estimated Time:** 2-3 days

#### Layout Structure

```
┌─────────────────────────────────────────────────────┐
│  GenXLink                                    [_][□][X]│
├─────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐             │
│  │ Devices │  │ History │  │ Settings│             │
│  └─────────┘  └─────────┘  └─────────┘             │
├─────────────────────────────────────────────────────┤
│                                                      │
│  📱 Available Devices                               │
│  ┌──────────────────────────────────────────────┐  │
│  │  🖥️  Desktop-PC                              │  │
│  │      192.168.1.100 • Online                  │  │
│  │      [Connect]                               │  │
│  ├──────────────────────────────────────────────┤  │
│  │  💻  Laptop-Work                             │  │
│  │      192.168.1.101 • Online                  │  │
│  │      [Connect]                               │  │
│  ├──────────────────────────────────────────────┤  │
│  │  📱  Phone-Android                           │  │
│  │      192.168.1.102 • Offline                 │  │
│  │      [Unavailable]                           │  │
│  └──────────────────────────────────────────────┘  │
│                                                      │
│  Status: Ready • Device ID: abc123                  │
└─────────────────────────────────────────────────────┘
```

#### Components

**2.1 Top Menu Bar**
```rust
pub struct MenuBar {
    // File, Edit, View, Help menus
}
```

**2.2 Tab Navigation**
```rust
pub enum Tab {
    Devices,
    History,
    Settings,
}
```

**2.3 Device List**
```rust
pub struct DeviceList {
    devices: Vec<DeviceInfo>,
    selected: Option<DeviceId>,
}
```

**2.4 Status Bar**
```rust
pub struct StatusBar {
    connection_status: ConnectionStatus,
    device_id: DeviceId,
    network_status: NetworkStatus,
}
```

#### Implementation Tasks
- [ ] Create main window struct
- [ ] Implement tab navigation
- [ ] Add device list view
- [ ] Create status bar
- [ ] Add menu bar
- [ ] Implement responsive layout

---

### 3. Device List View (HIGH PRIORITY)

**File:** `client/windows/src/ui/devices.rs`  
**Estimated Time:** 2 days

#### Features

**Device Card:**
```rust
pub struct DeviceCard {
    device_id: DeviceId,
    name: String,
    device_type: DeviceType,
    ip_address: String,
    status: DeviceStatus,
    last_seen: Option<DateTime<Utc>>,
}
```

**Device Actions:**
- Connect button
- Quick actions menu (settings, remove, etc.)
- Status indicator (online/offline/connecting)
- Last seen timestamp

#### Implementation
```rust
impl DeviceList {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for device in &self.devices {
                self.show_device_card(ui, device);
            }
        });
    }
    
    fn show_device_card(&mut self, ui: &mut egui::Ui, device: &DeviceInfo) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                // Device icon
                ui.label(device.icon());
                
                ui.vertical(|ui| {
                    // Device name
                    ui.heading(&device.name);
                    
                    // IP and status
                    ui.label(format!("{} • {}", device.ip, device.status));
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if device.is_online() {
                        if ui.button("Connect").clicked() {
                            self.connect_to_device(device.id.clone());
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("Unavailable"));
                    }
                });
            });
        });
    }
}
```

#### Tasks
- [ ] Create device card component
- [ ] Add device icons
- [ ] Implement status indicators
- [ ] Add connect button
- [ ] Show device details
- [ ] Add search/filter

---

### 4. Connection Dialog (HIGH PRIORITY)

**File:** `client/windows/src/ui/connection.rs`  
**Estimated Time:** 2 days

#### Dialog Layout

```
┌─────────────────────────────────────┐
│  Connecting to Desktop-PC           │
├─────────────────────────────────────┤
│                                      │
│  🔄 Establishing connection...      │
│                                      │
│  ▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░  50%         │
│                                      │
│  Status: Exchanging encryption keys │
│                                      │
│  [Cancel]                           │
└─────────────────────────────────────┘
```

#### Connection States
1. Initializing
2. Connecting to signaling server
3. Exchanging offer/answer
4. Gathering ICE candidates
5. Establishing P2P connection
6. Connected

#### Implementation
```rust
pub struct ConnectionDialog {
    target_device: DeviceInfo,
    state: ConnectionState,
    progress: f32,
    status_message: String,
}

impl ConnectionDialog {
    pub fn show(&mut self, ctx: &egui::Context) -> DialogResult {
        egui::Window::new("Connecting")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(format!("Connecting to {}", self.target_device.name));
                    ui.add_space(10.0);
                    
                    // Progress bar
                    ui.add(egui::ProgressBar::new(self.progress)
                        .text(format!("{}%", (self.progress * 100.0) as u32)));
                    
                    ui.add_space(10.0);
                    ui.label(&self.status_message);
                    
                    ui.add_space(20.0);
                    if ui.button("Cancel").clicked() {
                        return DialogResult::Cancel;
                    }
                });
                
                DialogResult::Continue
            })
    }
}
```

#### Tasks
- [ ] Create connection dialog
- [ ] Add progress indicator
- [ ] Show connection steps
- [ ] Implement cancel button
- [ ] Handle connection errors
- [ ] Add timeout handling

---

### 5. System Tray Integration (MEDIUM PRIORITY)

**File:** `client/windows/src/ui/tray.rs`  
**Estimated Time:** 1-2 days

#### Features
- Minimize to tray
- Tray icon with status indicator
- Context menu
- Notifications

#### Tray Menu
```
┌─────────────────────────┐
│ GenXLink                │
├─────────────────────────┤
│ ✓ Online                │
│                         │
│ 📱 2 devices available  │
│                         │
│ Show Window             │
│ Quick Connect →         │
│   Desktop-PC            │
│   Laptop-Work           │
│ Settings                │
│ ───────────────────     │
│ Exit                    │
└─────────────────────────┘
```

#### Implementation
```rust
use tray_icon::{TrayIcon, TrayIconBuilder, menu::Menu};

pub struct SystemTray {
    tray_icon: TrayIcon,
    menu: Menu,
}

impl SystemTray {
    pub fn new() -> Result<Self> {
        let menu = Menu::new();
        menu.append_item("Show Window");
        menu.append_separator();
        menu.append_item("Exit");
        
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("GenXLink")
            .with_icon(load_icon())
            .build()?;
        
        Ok(Self { tray_icon, menu })
    }
}
```

#### Tasks
- [ ] Add tray icon dependency
- [ ] Create tray icon
- [ ] Implement context menu
- [ ] Add minimize to tray
- [ ] Show notifications
- [ ] Handle tray events

---

### 6. Settings Panel (MEDIUM PRIORITY)

**File:** `client/windows/src/ui/settings.rs`  
**Estimated Time:** 2-3 days

#### Settings Categories

**General**
- Device name
- Auto-start on boot
- Minimize to tray
- Show notifications

**Connection**
- STUN/TURN servers
- Port configuration
- Network interface
- Connection timeout

**Display**
- Video quality
- Frame rate
- Compression level
- Color depth

**Input**
- Keyboard shortcuts
- Mouse sensitivity
- Clipboard sync
- File transfer

**Security**
- Encryption settings
- Allowed devices
- Connection password
- Auto-lock

#### Implementation
```rust
pub struct SettingsPanel {
    config: AppConfig,
    modified: bool,
}

impl SettingsPanel {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Settings");
            
            ui.collapsing("General", |ui| {
                self.show_general_settings(ui);
            });
            
            ui.collapsing("Connection", |ui| {
                self.show_connection_settings(ui);
            });
            
            ui.collapsing("Display", |ui| {
                self.show_display_settings(ui);
            });
            
            // Save/Cancel buttons
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    self.save_settings();
                }
                if ui.button("Cancel").clicked() {
                    self.revert_settings();
                }
            });
        });
    }
}
```

#### Tasks
- [ ] Create settings structure
- [ ] Implement general settings
- [ ] Add connection settings
- [ ] Create display settings
- [ ] Add input settings
- [ ] Implement security settings
- [ ] Add save/load functionality

---

### 7. Connection View (HIGH PRIORITY)

**File:** `client/windows/src/ui/viewer.rs`  
**Estimated Time:** 3-4 days

#### Features
- Remote screen display
- Input capture
- Connection stats overlay
- Toolbar with actions

#### Layout
```
┌─────────────────────────────────────────────────────┐
│  Desktop-PC - GenXLink                       [_][□][X]│
├─────────────────────────────────────────────────────┤
│  [⟲] [⚙] [📋] [📁] [Ctrl+Alt+Del] [🔊] [Disconnect]│
├─────────────────────────────────────────────────────┤
│                                                      │
│                                                      │
│              Remote Screen Display                  │
│                                                      │
│                                                      │
├─────────────────────────────────────────────────────┤
│  📊 FPS: 60 | Latency: 45ms | Quality: High        │
└─────────────────────────────────────────────────────┘
```

#### Implementation
```rust
pub struct ConnectionView {
    device: DeviceInfo,
    frame_buffer: Option<egui::ColorImage>,
    stats: ConnectionStats,
    toolbar_visible: bool,
}

impl ConnectionView {
    pub fn show(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Toolbar
            if self.toolbar_visible {
                self.show_toolbar(ui);
            }
            
            // Remote screen
            if let Some(frame) = &self.frame_buffer {
                let texture = ui.ctx().load_texture(
                    "remote_screen",
                    frame.clone(),
                    Default::default()
                );
                ui.image(&texture);
            }
            
            // Stats overlay
            self.show_stats_overlay(ui);
        });
    }
}
```

#### Tasks
- [ ] Create viewer window
- [ ] Implement frame display
- [ ] Add toolbar
- [ ] Show connection stats
- [ ] Handle input capture
- [ ] Add fullscreen mode
- [ ] Implement scaling options

---

### 8. Keyboard Shortcuts (LOW PRIORITY)

**Estimated Time:** 1 day

#### Shortcuts
- `Ctrl+N` - New connection
- `Ctrl+D` - Disconnect
- `Ctrl+S` - Settings
- `Ctrl+Q` - Quit
- `F11` - Fullscreen
- `Ctrl+Alt+Del` - Send to remote
- `Ctrl+C` - Copy (clipboard sync)
- `Ctrl+V` - Paste (clipboard sync)

#### Implementation
```rust
pub fn handle_shortcuts(&mut self, ctx: &egui::Context) {
    if ctx.input(|i| i.key_pressed(egui::Key::N) && i.modifiers.ctrl) {
        self.show_new_connection_dialog();
    }
    
    if ctx.input(|i| i.key_pressed(egui::Key::D) && i.modifiers.ctrl) {
        self.disconnect();
    }
    
    // ... more shortcuts
}
```

#### Tasks
- [ ] Define shortcut mappings
- [ ] Implement shortcut handler
- [ ] Add shortcut hints in UI
- [ ] Make shortcuts configurable
- [ ] Test all shortcuts

---

## 🎨 Design Guidelines

### Color Scheme
- **Primary:** #2563eb (Blue)
- **Secondary:** #10b981 (Green)
- **Accent:** #f59e0b (Orange)
- **Background:** #1f2937 (Dark Gray)
- **Surface:** #374151 (Gray)
- **Text:** #f3f4f6 (Light Gray)
- **Error:** #ef4444 (Red)

### Typography
- **Headings:** 18-24px, Bold
- **Body:** 14px, Regular
- **Small:** 12px, Regular
- **Code:** Monospace, 13px

### Spacing
- **Small:** 4px
- **Medium:** 8px
- **Large:** 16px
- **XLarge:** 24px

### Icons
- Use emoji or icon font
- Consistent size (16x16 or 24x24)
- Clear and recognizable

---

## 🧪 Testing Strategy

### Manual Testing
- [ ] Window resizing
- [ ] Tab navigation
- [ ] Device list interaction
- [ ] Connection flow
- [ ] Settings persistence
- [ ] System tray functionality
- [ ] Keyboard shortcuts
- [ ] Error handling

### Usability Testing
- [ ] First-time user experience
- [ ] Connection workflow
- [ ] Settings discoverability
- [ ] Error message clarity
- [ ] Performance on low-end hardware

---

## 📊 Success Criteria

- [ ] Modern, intuitive UI
- [ ] Smooth 60 FPS performance
- [ ] Responsive layout
- [ ] Clear status indicators
- [ ] Easy device connection
- [ ] Functional system tray
- [ ] Persistent settings
- [ ] Keyboard shortcuts working
- [ ] Clean error handling
- [ ] Professional appearance

---

## 🚀 Deliverables

1. **Main Window** - Device list and navigation
2. **Connection Dialog** - Connection progress
3. **Viewer Window** - Remote screen display
4. **Settings Panel** - Configuration UI
5. **System Tray** - Background operation
6. **Keyboard Shortcuts** - Quick actions
7. **Documentation** - User guide
8. **Icons & Assets** - Professional branding

---

**Next:** Phase 6 - Testing & Polish  
**Estimated Start:** After Phase 5 completion
