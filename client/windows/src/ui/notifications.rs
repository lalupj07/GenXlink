use std::time::{Duration, Instant};

/// Notification type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

/// A notification message
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: usize,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Notification {
    pub fn new(title: String, message: String, notification_type: NotificationType) -> Self {
        Self {
            id: 0, // Will be set by manager
            title,
            message,
            notification_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(5),
        }
    }
    
    pub fn info(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title.into(), message.into(), NotificationType::Info)
    }
    
    pub fn success(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title.into(), message.into(), NotificationType::Success)
    }
    
    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title.into(), message.into(), NotificationType::Warning)
    }
    
    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title.into(), message.into(), NotificationType::Error)
    }
    
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.duration
    }
    
    pub fn icon(&self) -> &'static str {
        match self.notification_type {
            NotificationType::Info => "ℹ️",
            NotificationType::Success => "✅",
            NotificationType::Warning => "⚠️",
            NotificationType::Error => "❌",
        }
    }
    
    pub fn color(&self) -> egui::Color32 {
        match self.notification_type {
            NotificationType::Info => egui::Color32::from_rgb(59, 130, 246),
            NotificationType::Success => egui::Color32::from_rgb(34, 197, 94),
            NotificationType::Warning => egui::Color32::from_rgb(251, 191, 36),
            NotificationType::Error => egui::Color32::from_rgb(239, 68, 68),
        }
    }
}

/// Manages notifications
pub struct NotificationManager {
    notifications: Vec<Notification>,
    next_id: usize,
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
            next_id: 0,
        }
    }
    
    pub fn add(&mut self, mut notification: Notification) {
        notification.id = self.next_id;
        self.next_id += 1;
        self.notifications.push(notification);
    }
    
    pub fn info(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.add(Notification::info(title, message));
    }
    
    pub fn success(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.add(Notification::success(title, message));
    }
    
    pub fn warning(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.add(Notification::warning(title, message));
    }
    
    pub fn error(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.add(Notification::error(title, message));
    }
    
    pub fn show(&mut self, ctx: &egui::Context) {
        // Remove expired notifications
        self.notifications.retain(|n| !n.is_expired());
        
        // Show active notifications
        for (i, notif) in self.notifications.iter().enumerate() {
            let y_offset = 10.0 + (i as f32 * 80.0);
            
            egui::Window::new(format!("notification_{}", notif.id))
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::RIGHT_TOP, [-10.0, y_offset])
                .fixed_size([300.0, 70.0])
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(31, 41, 55))
                        .stroke(egui::Stroke::new(1.0, notif.color()))
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Icon
                                ui.label(
                                    egui::RichText::new(notif.icon())
                                        .size(24.0)
                                        .color(notif.color())
                                );
                                
                                ui.add_space(10.0);
                                
                                // Content
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(&notif.title)
                                            .strong()
                                            .color(egui::Color32::WHITE)
                                    );
                                    ui.label(
                                        egui::RichText::new(&notif.message)
                                            .small()
                                            .color(egui::Color32::from_rgb(156, 163, 175))
                                    );
                                });
                            });
                        });
                });
        }
    }
}
