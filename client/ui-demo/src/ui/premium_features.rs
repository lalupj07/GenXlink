use eframe::egui;

/// Premium features panel showing upgrade options
#[derive(Default)]
pub struct PremiumFeaturesPanel {
    show_details: bool,
    show_annual: bool,
    selected_tier: PricingTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PricingTier {
    Free,
    Solo,
    Team,
    Enterprise,
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
                let solo_price = if self.show_annual { "₹199" } else { "₹199" };
                let solo_usd = if self.show_annual { "$2.39" } else { "$2.39" };
                let solo_save = if self.show_annual { None } else { Some("🔥 Best Value") };
                
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
                let team_price = if self.show_annual { "₹399" } else { "₹399" };
                let team_usd = if self.show_annual { "$4.79" } else { "$4.79" };
                let team_save = if self.show_annual { Some("⭐ Most Popular") } else { Some("⭐ Most Popular") };
                
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

                ui.add_space(10.0);

                // Enterprise Plan
                let enterprise_price = if self.show_annual { "₹699" } else { "₹699" };
                let enterprise_usd = if self.show_annual { "$8.39" } else { "$8.39" };
                let enterprise_save = if self.show_annual { Some("🏢 Enterprise") } else { Some("🏢 Enterprise") };
                
                self.show_pricing_card(
                    ui,
                    "🏢 Enterprise Plan",
                    enterprise_price,
                    "month",
                    enterprise_save,
                    "Designed for large organizations & unlimited scale",
                    &[
                        "✔ Everything in Team +",
                        "✔ Unlimited users",
                        "✔ Unlimited concurrent sessions",
                        "✔ Advanced security & compliance",
                        "✔ Custom integrations & API",
                        "✔ Dedicated support team",
                        "✔ On-premise deployment option",
                        "✔ SLA guarantees",
                    ],
                    &[
                        "Unlimited devices",
                        "Unlimited technicians",
                        "Custom branding",
                        "Priority 24/7 support",
                    ],
                    &format!("Best for: Enterprise, large organizations • {}", enterprise_usd),
                    egui::Color32::from_rgb(255, 100, 100),
                    false,
                    &mut action,
                );
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
                    } else if title.contains("Team") {
                        "🟣 Upgrade to Team"
                    } else {
                        "🏢 Upgrade to Enterprise"
                    };
                    
                    if ui.button(button_text).clicked() {
                        *action = if title.contains("Solo") {
                            PremiumAction::UpgradeToSolo
                        } else if title.contains("Team") {
                            PremiumAction::UpgradeToTeam
                        } else {
                            PremiumAction::UpgradeToEnterprise
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
                ui.add_space(40.0);
                ui.label(egui::RichText::new("🏢 Enterprise").strong().size(14.0).color(egui::Color32::from_rgb(255, 100, 100)));
            });
            
            ui.separator();
            
            // Features with better spacing and alignment
            let features = [
                ("💰 Price/month", "₹0", "₹199", "₹399", "₹699"),
                ("💎 Annual price", "₹0", "₹199/mo", "₹399/mo", "₹699/mo"),
                ("🚀 GPU Acceleration", "✅", "✅", "✅", "✅"),
                ("⚡ Ultra-Low Latency", "✅", "✅", "✅", "✅"),
                ("📊 Adaptive Bitrate", "✅", "✅", "✅", "✅"),
                ("🎵 Audio Streaming", "❌", "✅", "✅", "✅"),
                ("🤖 AI Features", "❌", "✅", "✅", "✅"),
                ("📹 Recording", "❌", "✅", "✅", "✅"),
                ("🔓 Unattended Access", "❌", "✅", "✅", "✅"),
                ("👥 Multi-user Sessions", "❌", "✅", "✅", "✅"),
                ("📊 Team Dashboard", "❌", "❌", "✅", "✅"),
                ("🔐 Role-based Access", "❌", "❌", "✅", "✅"),
                ("🔒 Advanced Security", "❌", "❌", "❌", "✅"),
                ("📱 Device Logins", "1", "5", "10", "Unlimited"),
                ("🔄 Concurrent Sessions", "0", "2", "5", "Unlimited"),
            ];
            
            for (feature, free, solo, team, enterprise) in features {
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
                    ui.add_space(45.0);
                    ui.label(enterprise);
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
    UpgradeToEnterprise,
    ContactSales,
}
