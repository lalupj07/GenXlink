use eframe::egui;
use std::path::PathBuf;
use genxlink_client_core::file_transfer::{TransferDirection, TransferStatus};
use genxlink_client_core::file_transfer_enhanced::{
    EnhancedFileTransfer, FileType, TransferPriority, TransferStatistics
};

/// Enhanced file transfer panel with drag & drop support
#[derive(Default)]
pub struct EnhancedFileTransferPanel {
    transfers: Vec<EnhancedFileTransferItem>,
    show_completed: bool,
    drag_active: bool,
    hovered_files: Vec<String>,
    statistics: TransferStatistics,
    selected_filter: TransferFilter,
    sort_by: SortOption,
    sort_ascending: bool,
}

/// Enhanced file transfer item for UI display
#[derive(Debug, Clone)]
pub struct EnhancedFileTransferItem {
    pub id: String,
    pub file_name: String,
    pub file_size: u64,
    pub bytes_transferred: u64,
    pub direction: TransferDirection,
    pub status: TransferStatus,
    pub speed: f64, // bytes per second
    pub eta: Option<f64>, // seconds
    pub file_type: FileType,
    pub priority: TransferPriority,
    pub compression_enabled: bool,
    pub parallel_transfer: bool,
    pub checksum: Option<String>,
    pub created_at: std::time::SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferFilter {
    All,
    Sending,
    Receiving,
    Documents,
    Images,
    Videos,
    Audio,
    Archives,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOption {
    Name,
    Size,
    Progress,
    Speed,
    CreatedAt,
    Priority,
}

impl Default for TransferFilter {
    fn default() -> Self {
        Self::All
    }
}

impl Default for SortOption {
    fn default() -> Self {
        Self::CreatedAt
    }
}

impl EnhancedFileTransferPanel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new enhanced transfer
    pub fn add_transfer(&mut self, transfer: EnhancedFileTransfer) {
        let item = EnhancedFileTransferItem {
            id: transfer.base_transfer.id,
            file_name: transfer.base_transfer.file_name,
            file_size: transfer.base_transfer.file_size,
            bytes_transferred: transfer.base_transfer.bytes_transferred,
            direction: transfer.base_transfer.direction,
            status: transfer.base_transfer.status,
            speed: 0.0, // Will be calculated
            eta: None,
            file_type: transfer.file_type,
            priority: transfer.priority,
            compression_enabled: transfer.compression_enabled,
            parallel_transfer: transfer.parallel_transfer,
            checksum: transfer.checksum,
            created_at: transfer.created_at,
        };
        
        self.transfers.push(item);
        self.apply_sorting();
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
        self.apply_sorting();
    }

    /// Update statistics
    pub fn update_statistics(&mut self, stats: TransferStatistics) {
        self.statistics = stats;
    }

    /// Handle drag & drop files
    pub fn handle_drag_drop(&mut self, _paths: &[String]) -> Vec<PathBuf> {
        // For now, return empty - drag_drop not available in current egui version
        vec![]
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

    /// Apply current filtering and sorting
    fn get_filtered_transfers(&self) -> Vec<&EnhancedFileTransferItem> {
        let mut filtered: Vec<_> = self.transfers.iter()
            .filter(|t| self.matches_filter(t))
            .filter(|t| self.show_completed || !matches!(t.status, TransferStatus::Completed))
            .collect();

        // Apply sorting
        filtered.sort_by(|a, b| {
            let ordering = match self.sort_by {
                SortOption::Name => a.file_name.cmp(&b.file_name),
                SortOption::Size => a.file_size.cmp(&b.file_size),
                SortOption::Progress => {
                    let progress_a = a.bytes_transferred as f64 / a.file_size.max(1) as f64;
                    let progress_b = b.bytes_transferred as f64 / b.file_size.max(1) as f64;
                    progress_a.partial_cmp(&progress_b).unwrap_or(std::cmp::Ordering::Equal)
                },
                SortOption::Speed => a.speed.partial_cmp(&b.speed).unwrap_or(std::cmp::Ordering::Equal),
                SortOption::CreatedAt => a.created_at.cmp(&b.created_at),
                SortOption::Priority => {
                    let priority_a = self.priority_value(a.priority);
                    let priority_b = self.priority_value(b.priority);
                    priority_a.cmp(&priority_b)
                },
            };

            if self.sort_ascending { ordering } else { ordering.reverse() }
        });

        filtered
    }

    /// Check if transfer matches current filter
    fn matches_filter(&self, transfer: &EnhancedFileTransferItem) -> bool {
        match self.selected_filter {
            TransferFilter::All => true,
            TransferFilter::Sending => transfer.direction == TransferDirection::Sending,
            TransferFilter::Receiving => transfer.direction == TransferDirection::Receiving,
            TransferFilter::Documents => transfer.file_type == FileType::Document,
            TransferFilter::Images => transfer.file_type == FileType::Image,
            TransferFilter::Videos => transfer.file_type == FileType::Video,
            TransferFilter::Audio => transfer.file_type == FileType::Audio,
            TransferFilter::Archives => transfer.file_type == FileType::Archive,
        }
    }

    /// Get numeric value for priority sorting
    fn priority_value(&self, priority: TransferPriority) -> u8 {
        match priority {
            TransferPriority::Low => 0,
            TransferPriority::Normal => 1,
            TransferPriority::High => 2,
            TransferPriority::Critical => 3,
        }
    }

    /// Apply current sorting to transfers
    fn apply_sorting(&mut self) {
        // Sorting is applied in get_filtered_transfers
    }

    /// Render the enhanced file transfer panel
    pub fn ui(&mut self, ui: &mut egui::Ui) -> EnhancedFileTransferAction {
        let mut action = EnhancedFileTransferAction::None;

        // Handle drag & drop
        let response = ui.allocate_response(
            egui::Vec2::new(ui.available_width(), ui.available_height()),
            egui::Sense::click_and_drag()
        );

        if let Some(_pointer_pos) = response.hover_pos() {
            if ui.input(|i| i.pointer.any_pressed()) {
                self.drag_active = true;
            }
        }

        // Main panel
        egui::Frame::none()
            .fill(if self.drag_active { 
                egui::Color32::from_rgb(240, 248, 255) // Light blue when dragging
            } else { 
                ui.visuals().panel_fill 
            })
            .stroke(if self.drag_active {
                egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE)
            } else {
                egui::Stroke::NONE
            })
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                self.render_header(ui, &mut action);
                ui.add_space(8.0);
                
                if self.drag_active {
                    self.render_drag_overlay(ui);
                } else {
                    self.render_controls(ui, &mut action);
                    ui.add_space(8.0);
                    self.render_transfer_list(ui, &mut action);
                }
            });

        action
    }

    /// Render panel header
    fn render_header(&mut self, ui: &mut egui::Ui, action: &mut EnhancedFileTransferAction) {
        ui.horizontal(|ui| {
            ui.heading("📁 Enhanced File Transfers");
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Statistics
                ui.label(egui::RichText::new(format!(
                    "Active: {} | Total: {}",
                    self.statistics.active_transfers(),
                    self.transfers.len()
                )).size(12.0).color(egui::Color32::GRAY));
                
                ui.add_space(16.0);
                
                if ui.button("🗑 Clear Completed").clicked() {
                    *action = EnhancedFileTransferAction::ClearCompleted;
                }
                
                ui.checkbox(&mut self.show_completed, "Show completed");
            });
        });
    }

    /// Render drag & drop overlay
    fn render_drag_overlay(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(egui::RichText::new("📤 Drop files here to transfer").size(24.0).color(egui::Color32::LIGHT_BLUE));
            ui.add_space(16.0);
            
            if !self.hovered_files.is_empty() {
                ui.label(format!("{} file(s) ready to transfer:", self.hovered_files.len()));
                for file_name in &self.hovered_files {
                    ui.label(egui::RichText::new(format!("  • {}", file_name)).size(14.0));
                }
            }
            
            ui.add_space(20.0);
            ui.label(egui::RichText::new("Release to start transfer").size(14.0).color(egui::Color32::GRAY));
            ui.add_space(40.0);
        });
    }

    /// Render control buttons and filters
    fn render_controls(&mut self, ui: &mut egui::Ui, _action: &mut EnhancedFileTransferAction) {
        ui.horizontal(|ui| {
            // Filter dropdown
            egui::ComboBox::from_label("Filter:")
                .selected_text(format!("{:?}", self.selected_filter))
                .show_ui(ui, |ui| {
                    for filter in [
                        TransferFilter::All,
                        TransferFilter::Sending,
                        TransferFilter::Receiving,
                        TransferFilter::Documents,
                        TransferFilter::Images,
                        TransferFilter::Videos,
                        TransferFilter::Audio,
                        TransferFilter::Archives,
                    ] {
                        if ui.selectable_label(self.selected_filter == filter, format!("{:?}", filter)).clicked() {
                            self.selected_filter = filter;
                        }
                    }
                });

            ui.add_space(8.0);

            // Sort dropdown
            egui::ComboBox::from_label("Sort by:")
                .selected_text(format!("{:?}", self.sort_by))
                .show_ui(ui, |ui| {
                    for sort_option in [
                        SortOption::Name,
                        SortOption::Size,
                        SortOption::Progress,
                        SortOption::Speed,
                        SortOption::CreatedAt,
                        SortOption::Priority,
                    ] {
                        if ui.selectable_label(self.sort_by == sort_option, format!("{:?}", sort_option)).clicked() {
                            self.sort_by = sort_option;
                        }
                    }
                });

            // Sort direction toggle
            if ui.button(if self.sort_ascending { "↑" } else { "↓" }).clicked() {
                self.sort_ascending = !self.sort_ascending;
            }
        });
    }

    /// Render the transfer list
    fn render_transfer_list(&mut self, ui: &mut egui::Ui, action: &mut EnhancedFileTransferAction) {
        let visible_transfers = self.get_filtered_transfers();

        if visible_transfers.is_empty() {
            self.render_empty_state(ui);
        } else {
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for transfer in visible_transfers {
                        if let Some(cancel_id) = self.show_enhanced_transfer_item(ui, transfer) {
                            *action = EnhancedFileTransferAction::Cancel(cancel_id);
                        }
                        ui.add_space(4.0);
                    }
                });
        }
    }

    /// Render empty state
    fn render_empty_state(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(egui::RichText::new("📂 No active transfers").size(18.0).color(egui::Color32::GRAY));
            ui.add_space(8.0);
            ui.label("Drag and drop files here to start transferring");
            ui.add_space(40.0);
        });
    }

    /// Show a single enhanced transfer item
    fn show_enhanced_transfer_item(&self, ui: &mut egui::Ui, transfer: &EnhancedFileTransferItem) -> Option<String> {
        let mut cancel_id = None;

        egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(12.0)
            .rounding(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // File type icon and direction
                    let icon = self.get_file_icon(transfer);
                    let direction_icon = match transfer.direction {
                        TransferDirection::Sending => "📤",
                        TransferDirection::Receiving => "📥",
                    };
                    
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(format!("{} {}", icon, direction_icon)).size(20.0));
                        
                        // Priority indicator
                        if transfer.priority != TransferPriority::Normal {
                            let priority_text = match transfer.priority {
                                TransferPriority::Low => "🔽",
                                TransferPriority::Normal => "",
                                TransferPriority::High => "🔼",
                                TransferPriority::Critical => "⚡",
                            };
                            ui.label(egui::RichText::new(priority_text).size(12.0));
                        }
                    });

                    ui.add_space(12.0);

                    // File information
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&transfer.file_name).strong());
                            
                            // Status badges
                            let (status_text, status_color) = match transfer.status {
                                TransferStatus::Pending => ("Pending", egui::Color32::GRAY),
                                TransferStatus::InProgress => ("In Progress", egui::Color32::LIGHT_BLUE),
                                TransferStatus::Completed => ("Completed", egui::Color32::GREEN),
                                TransferStatus::Failed => ("Failed", egui::Color32::RED),
                                TransferStatus::Cancelled => ("Cancelled", egui::Color32::YELLOW),
                            };
                            ui.colored_label(status_color, status_text);
                        });

                        ui.horizontal(|ui| {
                            ui.label(format!("Size: {}", Self::format_size(transfer.file_size)));
                            
                            if transfer.status == TransferStatus::InProgress {
                                ui.label("•");
                                ui.label(format!("Speed: {}", Self::format_speed(transfer.speed)));
                                
                                if let Some(eta) = transfer.eta {
                                    ui.label("•");
                                    ui.label(format!("ETA: {}", Self::format_eta(eta)));
                                }
                            }
                        });

                        // Feature indicators
                        ui.horizontal(|ui| {
                            if transfer.compression_enabled {
                                ui.label(egui::RichText::new("🗜️ Compressed").size(10.0).color(egui::Color32::GRAY));
                            }
                            if transfer.parallel_transfer {
                                ui.label(egui::RichText::new("⚡ Parallel").size(10.0).color(egui::Color32::GRAY));
                            }
                            if transfer.checksum.is_some() {
                                ui.label(egui::RichText::new("🔐 Verified").size(10.0).color(egui::Color32::GRAY));
                            }
                        });

                        // Progress bar
                        if matches!(transfer.status, TransferStatus::InProgress | TransferStatus::Completed) {
                            let progress = transfer.bytes_transferred as f32 / transfer.file_size.max(1) as f32;
                            let progress_bar = egui::ProgressBar::new(progress)
                                .text(format!("{:.1}%", progress * 100.0))
                                .show_percentage();
                            ui.add(progress_bar);
                        }
                    });

                    // Action buttons
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if transfer.status == TransferStatus::InProgress {
                            if ui.button("❌").on_hover_text("Cancel transfer").clicked() {
                                cancel_id = Some(transfer.id.clone());
                            }
                        }
                        
                        if transfer.status == TransferStatus::Completed {
                            if ui.button("📁").on_hover_text("Open file location").clicked() {
                                // TODO: Open file location
                            }
                        }
                    });
                });
            });

        cancel_id
    }

    /// Get appropriate icon for file type
    fn get_file_icon(&self, transfer: &EnhancedFileTransferItem) -> &'static str {
        match transfer.file_type {
            FileType::Document => "📄",
            FileType::Image => "🖼️",
            FileType::Video => "🎬",
            FileType::Audio => "🎵",
            FileType::Archive => "📦",
            FileType::Text => "📝",
            FileType::Binary => "⚙️",
            FileType::Unknown => "📎",
        }
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

/// Enhanced actions that can be triggered from the file transfer panel
#[derive(Debug, Clone, PartialEq)]
pub enum EnhancedFileTransferAction {
    None,
    Cancel(String),
    ClearCompleted,
    FilesDropped(Vec<PathBuf>),
    OpenFileLocation(String),
}
