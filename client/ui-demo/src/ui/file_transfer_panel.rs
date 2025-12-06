use eframe::egui;

/// File transfer panel for managing file transfers
#[derive(Default)]
pub struct FileTransferPanel {
    transfers: Vec<FileTransferItem>,
    show_completed: bool,
}

/// File transfer item for UI display
#[derive(Debug, Clone)]
pub struct FileTransferItem {
    pub id: String,
    pub file_name: String,
    pub file_size: u64,
    pub bytes_transferred: u64,
    pub direction: TransferDirection,
    pub status: TransferStatus,
    pub speed: f64, // bytes per second
    pub eta: Option<f64>, // seconds
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferDirection {
    Sending,
    Receiving,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

impl FileTransferPanel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new transfer
    pub fn add_transfer(&mut self, transfer: FileTransferItem) {
        self.transfers.push(transfer);
    }

    /// Update transfer progress
    pub fn update_transfer(&mut self, id: &str, bytes_transferred: u64, speed: f64, eta: Option<f64>) {
        if let Some(transfer) = self.transfers.iter_mut().find(|t| t.id == id) {
            transfer.bytes_transferred = bytes_transferred;
            transfer.speed = speed;
            transfer.eta = eta;
            
            if bytes_transferred >= transfer.file_size {
                transfer.status = TransferStatus::Completed;
            } else if transfer.status == TransferStatus::Pending {
                transfer.status = TransferStatus::InProgress;
            }
        }
    }

    /// Cancel a transfer
    pub fn cancel_transfer(&mut self, id: &str) {
        if let Some(transfer) = self.transfers.iter_mut().find(|t| t.id == id) {
            transfer.status = TransferStatus::Cancelled;
        }
    }

    /// Clear completed transfers
    pub fn clear_completed(&mut self) {
        self.transfers.retain(|t| t.status != TransferStatus::Completed);
    }

    /// Render the file transfer panel
    pub fn ui(&mut self, ui: &mut egui::Ui) -> FileTransferAction {
        let mut action = FileTransferAction::None;

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("📁 File Transfers");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🗑 Clear Completed").clicked() {
                        action = FileTransferAction::ClearCompleted;
                    }
                    ui.checkbox(&mut self.show_completed, "Show completed");
                });
            });

            ui.separator();

            // Display transfers
            let visible_transfers: Vec<_> = self.transfers.iter()
                .filter(|t| self.show_completed || !matches!(t.status, TransferStatus::Completed))
                .collect();

            if visible_transfers.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label("No active transfers");
                    ui.label("Drop files here to send");
                    ui.add_space(20.0);
                });
            } else {
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for transfer in visible_transfers {
                            if let Some(cancel_id) = self.show_transfer_item(ui, transfer) {
                                action = FileTransferAction::Cancel(cancel_id);
                            }
                            ui.add_space(5.0);
                        }
                    });
            }
        });

        action
    }

    /// Show a single transfer item
    fn show_transfer_item(&self, ui: &mut egui::Ui, transfer: &FileTransferItem) -> Option<String> {
        let mut cancel_id = None;

        egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(8.0)
            .rounding(4.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Direction icon
                    let icon = match transfer.direction {
                        TransferDirection::Sending => "📤",
                        TransferDirection::Receiving => "📥",
                    };
                    ui.label(egui::RichText::new(icon).size(24.0));

                    ui.add_space(8.0);

                    // File info
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&transfer.file_name).strong());
                        
                        ui.horizontal(|ui| {
                            // Status
                            let (status_text, status_color) = match transfer.status {
                                TransferStatus::Pending => ("Pending", egui::Color32::GRAY),
                                TransferStatus::InProgress => ("In Progress", egui::Color32::LIGHT_BLUE),
                                TransferStatus::Completed => ("Completed", egui::Color32::GREEN),
                                TransferStatus::Failed => ("Failed", egui::Color32::RED),
                                TransferStatus::Cancelled => ("Cancelled", egui::Color32::YELLOW),
                            };
                            ui.colored_label(status_color, status_text);

                            ui.label("•");
                            ui.label(Self::format_size(transfer.file_size));

                            if transfer.status == TransferStatus::InProgress {
                                ui.label("•");
                                ui.label(Self::format_speed(transfer.speed));
                                
                                if let Some(eta) = transfer.eta {
                                    ui.label("•");
                                    ui.label(format!("ETA: {}", Self::format_eta(eta)));
                                }
                            }
                        });

                        // Progress bar
                        if matches!(transfer.status, TransferStatus::InProgress | TransferStatus::Completed) {
                            let progress = transfer.bytes_transferred as f32 / transfer.file_size.max(1) as f32;
                            let progress_bar = egui::ProgressBar::new(progress)
                                .text(format!("{:.1}%", progress * 100.0));
                            ui.add(progress_bar);
                        }
                    });

                    // Cancel button
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if transfer.status == TransferStatus::InProgress {
                            if ui.button("❌").clicked() {
                                cancel_id = Some(transfer.id.clone());
                            }
                        }
                    });
                });
            });

        cancel_id
    }

    /// Format file size
    fn format_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }

    /// Format transfer speed
    fn format_speed(bytes_per_sec: f64) -> String {
        format!("{}/s", Self::format_size(bytes_per_sec as u64))
    }

    /// Format ETA
    fn format_eta(seconds: f64) -> String {
        let secs = seconds as u64;
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        }
    }
}

/// Actions that can be triggered from the file transfer panel
#[derive(Debug, Clone, PartialEq)]
pub enum FileTransferAction {
    None,
    Cancel(String),
    ClearCompleted,
}
