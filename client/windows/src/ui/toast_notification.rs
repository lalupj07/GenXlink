// Copyright (c) 2025 GenXis Innovations
// Licensed under the Apache License, Version 2.0

use eframe::egui::{self, Context, Ui, Vec2, Pos2, Color32, RichText};
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Toast notification type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

impl ToastType {
    pub fn color(&self) -> Color32 {
        match self {
            ToastType::Success => Color32::from_rgb(34, 197, 94),
            ToastType::Error => Color32::from_rgb(239, 68, 68),
            ToastType::Warning => Color32::from_rgb(245, 158, 11),
            ToastType::Info => Color32::from_rgb(99, 102, 241),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ToastType::Success => "✅",
            ToastType::Error => "❌",
            ToastType::Warning => "⚠️",
            ToastType::Info => "ℹ️",
        }
    }
}

/// Toast notification
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Toast {
    pub fn new(message: impl Into<String>, toast_type: ToastType) -> Self {
        Self {
            message: message.into(),
            toast_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(4),
        }
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.duration
    }

    pub fn opacity(&self) -> f32 {
        let elapsed = self.created_at.elapsed().as_secs_f32();
        let total = self.duration.as_secs_f32();
        
        if elapsed < 0.3 {
            // Fade in
            elapsed / 0.3
        } else if elapsed > total - 0.5 {
            // Fade out
            (total - elapsed) / 0.5
        } else {
            1.0
        }
    }
}

/// Toast notification manager
pub struct ToastManager {
    toasts: VecDeque<Toast>,
    max_toasts: usize,
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            toasts: VecDeque::new(),
            max_toasts: 5,
        }
    }

    /// Add a success toast
    pub fn success(&mut self, message: impl Into<String>) {
        self.add_toast(Toast::new(message, ToastType::Success));
    }

    /// Add an error toast
    pub fn error(&mut self, message: impl Into<String>) {
        self.add_toast(Toast::new(message, ToastType::Error));
    }

    /// Add a warning toast
    pub fn warning(&mut self, message: impl Into<String>) {
        self.add_toast(Toast::new(message, ToastType::Warning));
    }

    /// Add an info toast
    pub fn info(&mut self, message: impl Into<String>) {
        self.add_toast(Toast::new(message, ToastType::Info));
    }

    /// Add a custom toast
    pub fn add_toast(&mut self, toast: Toast) {
        if self.toasts.len() >= self.max_toasts {
            self.toasts.pop_front();
        }
        self.toasts.push_back(toast);
    }

    /// Update and render toasts
    pub fn show(&mut self, ctx: &Context) {
        // Remove expired toasts
        self.toasts.retain(|toast| !toast.is_expired());

        if self.toasts.is_empty() {
            return;
        }

        let screen_rect = ctx.screen_rect();
        let toast_width = 400.0;
        let toast_height = 60.0;
        let spacing = 10.0;
        let margin = 20.0;

        egui::Area::new("toast_container")
            .fixed_pos(Pos2::new(
                screen_rect.max.x - toast_width - margin,
                margin,
            ))
            .show(ctx, |ui| {
                ui.set_width(toast_width);

                for (i, toast) in self.toasts.iter().enumerate() {
                    let y_offset = i as f32 * (toast_height + spacing);
                    
                    ui.allocate_ui_at_rect(
                        egui::Rect::from_min_size(
                            Pos2::new(0.0, y_offset),
                            Vec2::new(toast_width, toast_height),
                        ),
                        |ui| {
                            self.render_toast(ui, toast);
                        },
                    );
                }
            });

        // Request repaint for animations
        ctx.request_repaint();
    }

    fn render_toast(&self, ui: &mut Ui, toast: &Toast) {
        let opacity = toast.opacity();
        let color = toast.toast_type.color();
        let bg_color = Color32::from_rgba_premultiplied(
            30, 30, 30,
            (opacity * 240.0) as u8,
        );

        egui::Frame::none()
            .fill(bg_color)
            .rounding(8.0)
            .stroke(egui::Stroke::new(1.0, color.linear_multiply(opacity)))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Icon
                    ui.label(
                        RichText::new(toast.toast_type.icon())
                            .size(20.0)
                            .color(color.linear_multiply(opacity))
                    );

                    ui.add_space(8.0);

                    // Message
                    ui.label(
                        RichText::new(&toast.message)
                            .size(14.0)
                            .color(Color32::WHITE.linear_multiply(opacity))
                    );
                });
            });
    }
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}
