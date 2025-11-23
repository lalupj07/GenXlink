use eframe::egui;

/// Premium features panel showing upgrade options
#[derive(Default)]
pub struct PremiumFeaturesPanel {
    show_details: bool,
    show_annual: bool,
    selected_tier: PricingTier,
    selected_theme: AppTheme,
    selected_language: AppLanguage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLanguage {
    English,
    Hindi,
    Tamil,
    Telugu,
    Bengali,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::System
    }
}

impl Default for AppLanguage {
    fn default() -> Self {
        Self::English
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PricingTier {
    Free,
    Solo,
    Team,
}

impl Default for PricingTier {
    fn default() -> Self {
        Self::Free
    }
}

impl PremiumFeaturesPanel {
    pub fn new() -> Self {
        Self {
            show_details: false,
            show_annual: true,
            selected_tier: PricingTier::Free,
            selected_theme: AppTheme::System,
            selected_language: AppLanguage::English,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> PremiumAction {
        let mut action = PremiumAction::None;

        // Header
        ui.vertical_centered(|ui| {
            ui.heading("🌐 GenXLink Pricing");
            ui.add_space(5.0);
            ui.label("Fast • Secure • Ultra-Low Latency Remote Desktop Access");
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(15.0);

        // Current plan badge
        ui.horizontal(|ui| {
            ui.label("Current Plan:");
            ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "🟢 Free Tier");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Billing toggle
                ui.label(if self.show_annual { "Annual" } else { "Monthly" });
                if ui.button("⇄").clicked() {
                    self.show_annual = !self.show_annual;
                }
            });
        });

        ui.add_space(15.0);

        // Pricing cards
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                // Free Tier
                self.show_pricing_card(
                    ui,
                    "🟢 Free Tier",
                    "₹0",
                    "month",
                    None,
                    "Perfect for personal & occasional use",
                    &[
                        "✔ All core remote-access features",
                        "✔ GPU Acceleration",
                        "✔ Ultra-Low Latency",
                        "✔ Adaptive Bitrate",
                        "✔ Smooth streaming & control",
                    ],
                    &[
                        "1 registered device",
                        "Login from 1 device",
                        "No unattended access",
                        "No recording",
                    ],
                    "Best for: Students, casual users, home use",
                    egui::Color32::from_rgb(100, 200, 100),
                    true,
                    &mut action,
                );

                ui.add_space(10.0);

                // Solo Plan
                let solo_price = if self.show_annual { "₹670" } else { "₹840" };
                let solo_usd = if self.show_annual { "$7.99" } else { "$9.99" };
                let solo_save = if self.show_annual { Some("Save 20%") } else { None };
                
                self.show_pricing_card(
                    ui,
                    "🔵 Solo Plan",
                    solo_price,
                    "month",
                    solo_save,
                    "Ideal for creators, professionals & freelancers",
                    &[
                        "✔ Everything in Free +",
                        "✔ Audio streaming",
                        "✔ AI-powered enhancements",
                        "✔ Unattended access",
                        "✔ Session recording",
                        "✔ Multi-user sessions",
                    ],
                    &[
                        "1 registered device",
                        "Login from up to 5 devices",
                        "2 concurrent sessions",
                    ],
                    &format!("Best for: Creators, freelancers • {}", solo_usd),
                    egui::Color32::from_rgb(100, 150, 255),
                    false,
                    &mut action,
                );

                ui.add_space(10.0);

                // Team Plan
                let team_price = if self.show_annual { "₹1,090" } else { "₹1,260" };
                let team_usd = if self.show_annual { "$12.99" } else { "$14.99" };
                let team_save = if self.show_annual { Some("Save 27% ⭐") } else { Some("⭐ Most Popular") };
                
                self.show_pricing_card(
                    ui,
                    "🟣 Team Plan",
                    team_price,
                    "month",
                    team_save,
                    "Built for support teams, studios & IT admins",
                    &[
                        "✔ Everything in Solo +",
                        "✔ Team Dashboard",
                        "✔ Role-based access control",
                        "✔ Technician switching",
                        "✔ Shared device groups",
                        "✔ Advanced reports & logs",
                        "✔ Priority routing",
                    ],
                    &[
                        "Login from up to 10 devices",
                        "5 concurrent sessions",
                        "Multiple registered devices",
                    ],
                    &format!("Best for: IT teams, remote support • {}", team_usd),
                    egui::Color32::from_rgb(200, 100, 255),
                    false,
                    &mut action,
                );
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(15.0);

            // Theme & Language Settings
            ui.heading("🎨 Appearance & Language");
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label("🎨 Theme:");
                ui.add_space(10.0);
                
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.selected_theme))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_theme, AppTheme::Light, "☀️ Light");
                        ui.selectable_value(&mut self.selected_theme, AppTheme::Dark, "🌙 Dark");
                        ui.selectable_value(&mut self.selected_theme, AppTheme::System, "💻 System");
                    });
                
                ui.add_space(20.0);
                
                ui.label("🌍 Language:");
                ui.add_space(10.0);
                
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.selected_language))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_language, AppLanguage::English, "🇬🇧 English");
                        ui.selectable_value(&mut self.selected_language, AppLanguage::Hindi, "🇮🇳 हिंदी");
                        ui.selectable_value(&mut self.selected_language, AppLanguage::Tamil, "🇮🇳 தமிழ்");
                        ui.selectable_value(&mut self.selected_language, AppLanguage::Telugu, "🇮🇳 తెలుగు");
                        ui.selectable_value(&mut self.selected_language, AppLanguage::Bengali, "🇮🇳 বাংলা");
                    });
            });
            
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(150, 150, 150), 
                    "💡 Theme changes apply immediately • Language support coming soon");
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(15.0);

            // Feature comparison table
            ui.heading("🔍 Feature Comparison");
            ui.add_space(10.0);

            self.show_comparison_table(ui);

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(15.0);

            // Trust & Security
            ui.heading("🛡️ Trust & Security");
            ui.add_space(10.0);
            
            ui.horizontal_wrapped(|ui| {
                ui.label("🔒 End-to-end encrypted sessions");
                ui.label("•");
                ui.label("⚡ India-optimized low-latency routing");
                ui.label("•");
                ui.label("🛠️ Enterprise-grade infrastructure");
            });

            ui.add_space(15.0);

            // Trial notice
            ui.horizontal(|ui| {
                ui.label("🎧");
                ui.colored_label(
                    egui::Color32::from_rgb(100, 200, 255),
                    "14-day Premium Trial included with Solo/Team plans!",
                );
            });

            ui.add_space(15.0);

            // Enterprise option
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("🏢 Need More Users?");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("📧 Contact Sales").clicked() {
                            action = PremiumAction::ContactSales;
                        }
                    });
                });
                ui.label("Custom Enterprise Plans Available");
                ui.label("For 20+ technicians, bulk devices, or white-label access");
            });

            ui.add_space(15.0);

            // Billing notes
            ui.horizontal(|ui| {
                ui.label("ℹ️");
                ui.colored_label(
                    egui::Color32::from_rgb(150, 150, 150),
                    "All prices exclude GST • INR pricing is primary • No hidden fees",
                );
            });
        });

        action
    }

    fn show_pricing_card(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        price: &str,
        period: &str,
        badge: Option<&str>,
        description: &str,
        features: &[&str],
        limits: &[&str],
        best_for: &str,
        color: egui::Color32,
        is_current: bool,
        action: &mut PremiumAction,
    ) {
        let card_width = 280.0;
        
        ui.group(|ui| {
            ui.set_min_width(card_width);
            ui.set_max_width(card_width);
            
            ui.vertical(|ui| {
                // Title with badge
                ui.horizontal(|ui| {
                    ui.colored_label(color, title);
                    if let Some(badge_text) = badge {
                        ui.label(badge_text);
                    }
                });
                
                ui.add_space(10.0);
                
                // Price
                ui.horizontal_wrapped(|ui| {
                    ui.heading(price);
                    ui.label(format!("/ {}", period));
                });
                
                ui.add_space(5.0);
                ui.label(description);
                ui.add_space(10.0);
                
                // Features
                for feature in features {
                    ui.label(*feature);
                }
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);
                
                // Limits
                ui.label("Limits:");
                for limit in limits {
                    ui.label(format!("  • {}", limit));
                }
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);
                
                // Best for
                ui.colored_label(egui::Color32::from_rgb(150, 150, 150), best_for);
                
                ui.add_space(10.0);
                
                // Action button
                if is_current {
                    ui.colored_label(egui::Color32::DARK_GREEN, "✓ Current Plan");
                } else {
                    let button_text = if title.contains("Solo") {
                        "🔵 Upgrade to Solo"
                    } else {
                        "🟣 Upgrade to Team"
                    };
                    
                    if ui.button(button_text).clicked() {
                        *action = if title.contains("Solo") {
                            PremiumAction::UpgradeToSolo
                        } else {
                            PremiumAction::UpgradeToTeam
                        };
                    }
                }
            });
        });
    }

    fn show_comparison_table(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            // Header with better alignment
            ui.horizontal(|ui| {
                // Feature column
                ui.add_space(5.0);
                ui.label(egui::RichText::new("Feature").strong().size(14.0));
                ui.add_space(120.0);
                
                // Plan columns with centered alignment
                ui.add_space(20.0);
                ui.label(egui::RichText::new("🟢 Free").strong().size(14.0).color(egui::Color32::from_rgb(100, 200, 100)));
                ui.add_space(40.0);
                ui.label(egui::RichText::new("🔵 Solo").strong().size(14.0).color(egui::Color32::from_rgb(100, 150, 255)));
                ui.add_space(40.0);
                ui.label(egui::RichText::new("🟣 Team").strong().size(14.0).color(egui::Color32::from_rgb(200, 100, 255)));
            });
            
            ui.separator();
            
            // Features with better spacing and alignment
            let features = [
                ("💰 Price/month", "₹0", "₹840", "₹1,260"),
                ("💎 Annual price", "₹0", "₹670/mo", "₹1,090/mo"),
                ("🚀 GPU Acceleration", "✅", "✅", "✅"),
                ("⚡ Ultra-Low Latency", "✅", "✅", "✅"),
                ("📊 Adaptive Bitrate", "✅", "✅", "✅"),
                ("🎵 Audio Streaming", "❌", "✅", "✅"),
                ("🤖 AI Features", "❌", "✅", "✅"),
                ("📹 Recording", "❌", "✅", "✅"),
                ("🔓 Unattended Access", "❌", "✅", "✅"),
                ("👥 Multi-user Sessions", "❌", "✅", "✅"),
                ("📊 Team Dashboard", "❌", "❌", "✅"),
                ("🔐 Role-based Access", "❌", "❌", "✅"),
                ("📱 Device Logins", "1", "5", "10"),
                ("🔄 Concurrent Sessions", "0", "2", "5"),
            ];
            
            for (feature, free, solo, team) in features {
                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                    ui.label(feature);
                    ui.add_space(80.0);
                    
                    // Center the plan values
                    ui.add_space(15.0);
                    ui.label(free);
                    ui.add_space(45.0);
                    ui.label(solo);
                    ui.add_space(45.0);
                    ui.label(team);
                });
            }
        });
    }
}

/// Actions that can be triggered from the premium panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PremiumAction {
    None,
    UpgradeToSolo,
    UpgradeToTeam,
    ContactSales,
}
