use eframe::egui;
use genxlink_client_core::auth_service::{AuthService, LoginRequest, RegisterRequest, UpdateProfileRequest};
use genxlink_client_core::database::{UserAccount, UserPreferences, SubscriptionType};
use std::pin::Pin;
use futures::Future;

/// Authentication panel UI
#[derive(Default)]
pub struct AuthPanel {
    state: AuthPanelState,
    login_form: LoginForm,
    register_form: RegisterForm,
    profile_form: ProfileForm,
    reset_password_form: ResetPasswordForm,
    error_message: String,
    success_message: String,
    loading: bool,
    pending_login: Option<LoginOperation>,
    pending_register: Option<RegisterOperation>,
}

/// Authentication panel state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthPanelState {
    Login,
    Register,
    Profile,
    ResetPassword,
    Loading,
}

impl Default for AuthPanelState {
    fn default() -> Self {
        Self::Login
    }
}

/// Login form data
#[derive(Default, Clone)]
pub struct LoginForm {
    email: String,
    password: String,
    remember_me: bool,
}

/// Register form data
#[derive(Default, Clone)]
pub struct RegisterForm {
    email: String,
    password: String,
    confirm_password: String,
    username: String,
    display_name: String,
    agree_terms: bool,
}

/// Profile form data
#[derive(Default, Clone)]
pub struct ProfileForm {
    display_name: String,
    avatar_url: String,
    theme: String,
    language: String,
    notifications_enabled: bool,
    auto_accept_connections: bool,
    require_confirmation: bool,
    max_concurrent_sessions: u32,
}

/// Reset password form data
#[derive(Default, Clone)]
pub struct ResetPasswordForm {
    email: String,
}

impl AuthPanel {
    /// Create new authentication panel
    pub fn new() -> Self {
        Self::default()
    }

    /// Render the authentication panel
    pub fn ui(&mut self, ui: &mut egui::Ui, auth_service: &mut AuthService) -> AuthAction {
        let mut action = AuthAction::None;
        
        // Clear messages after a delay
        if !self.error_message.is_empty() || !self.success_message.is_empty() {
            ui.horizontal(|ui| {
                if !self.error_message.is_empty() {
                    ui.colored_label(egui::Color32::RED, &self.error_message);
                }
                if !self.success_message.is_empty() {
                    ui.colored_label(egui::Color32::GREEN, &self.success_message);
                }
            });
            ui.add_space(8.0);
        }

        // Show loading spinner if loading
        if self.loading {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.spinner();
                ui.label("Loading...");
                ui.add_space(20.0);
            });
            return action;
        }

        match self.state {
            AuthPanelState::Login => self.render_login_panel(ui, auth_service, &mut action),
            AuthPanelState::Register => self.render_register_panel(ui, auth_service, &mut action),
            AuthPanelState::Profile => self.render_profile_panel(ui, auth_service, &mut action),
            AuthPanelState::ResetPassword => self.render_reset_password_panel(ui, auth_service, &mut action),
            AuthPanelState::Loading => {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.spinner();
                    ui.add_space(10.0);
                    ui.label("Processing...");
                });
            }
        }
        
        action
    }

    /// Render login panel
    fn render_login_panel(&mut self, ui: &mut egui::Ui, auth_service: &mut AuthService, action: &mut AuthAction) {
        ui.vertical_centered(|ui| {
            ui.heading("🔐 Login to GenXLink");
            ui.add_space(16.0);
        });

        egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .rounding(8.0)
            .inner_margin(20.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Email:");
                    ui.text_edit_singleline(&mut self.login_form.email);
                });
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.login_form.password).password(true));
                });
                
                ui.add_space(8.0);
                
                ui.checkbox(&mut self.login_form.remember_me, "Remember me");
                
                ui.add_space(16.0);
                
                ui.horizontal(|ui| {
                    if ui.button("🔑 Login").clicked() {
                        self.handle_login(auth_service);
                    }
                    
                    if ui.button("❌ Cancel").clicked() {
                        self.clear_forms();
                    }
                });
                
                ui.add_space(16.0);
                
                ui.separator();
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("Don't have an account?");
                    if ui.link("Register").clicked() {
                        self.state = AuthPanelState::Register;
                        self.clear_messages();
                    }
                    
                    ui.add_space(16.0);
                    
                    if ui.link("Forgot Password?").clicked() {
                        self.state = AuthPanelState::ResetPassword;
                        self.clear_messages();
                    }
                });
            });
    }

    /// Render register panel
    fn render_register_panel(&mut self, ui: &mut egui::Ui, auth_service: &mut AuthService, action: &mut AuthAction) {
        ui.vertical_centered(|ui| {
            ui.heading("📝 Register for GenXLink");
            ui.add_space(16.0);
        });

        egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .rounding(8.0)
            .inner_margin(20.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Email:");
                    ui.text_edit_singleline(&mut self.register_form.email);
                });
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("Username:");
                    ui.text_edit_singleline(&mut self.register_form.username);
                });
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("Display Name:");
                    ui.text_edit_singleline(&mut self.register_form.display_name);
                });
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.register_form.password).password(true));
                });
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("Confirm Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.register_form.confirm_password).password(true));
                });
                
                ui.add_space(8.0);
                
                ui.checkbox(&mut self.register_form.agree_terms, "I agree to the Terms of Service and Privacy Policy");
                
                ui.add_space(16.0);
                
                ui.horizontal(|ui| {
                    if ui.button("📝 Register").clicked() {
                        self.handle_register(auth_service);
                    }
                    
                    if ui.button("❌ Cancel").clicked() {
                        self.state = AuthPanelState::Login;
                        self.clear_forms();
                    }
                });
                
                ui.add_space(16.0);
                
                ui.separator();
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("Already have an account?");
                    if ui.link("Login").clicked() {
                        self.state = AuthPanelState::Login;
                        self.clear_messages();
                    }
                });
            });
    }

    /// Render profile panel
    fn render_profile_panel(&mut self, ui: &mut egui::Ui, auth_service: &mut AuthService, action: &mut AuthAction) {
        if let Some(user) = auth_service.get_current_user() {
            let user_clone = user.clone(); // Clone to avoid borrowing issues
            
            ui.vertical_centered(|ui| {
                ui.heading("👤 User Profile");
                ui.add_space(16.0);
            });

            egui::Frame::none()
                .fill(ui.visuals().faint_bg_color)
                .rounding(8.0)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Email:");
                        ui.label(&user_clone.email);
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Username:");
                        ui.label(&user_clone.username);
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Subscription:");
                        ui.colored_label(
                            match user_clone.subscription_type {
                                SubscriptionType::Free => egui::Color32::GRAY,
                                SubscriptionType::Premium => egui::Color32::GOLD,
                                SubscriptionType::Enterprise => egui::Color32::from_rgb(128, 0, 128),
                            },
                            format!("{:?}", user_clone.subscription_type)
                        );
                    });
                    
                    ui.add_space(16.0);
                    
                    ui.separator();
                    
                    ui.add_space(16.0);
                    
                    ui.heading("Edit Profile");
                    
                    ui.add_space(8.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Display Name:");
                        ui.text_edit_singleline(&mut self.profile_form.display_name);
                    });
                    
                    ui.add_space(8.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Avatar URL:");
                        ui.text_edit_singleline(&mut self.profile_form.avatar_url);
                    });
                    
                    ui.add_space(16.0);
                    
                    ui.heading("Preferences");
                    
                    ui.add_space(8.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Theme:");
                        egui::ComboBox::from_label("")
                            .selected_text(&self.profile_form.theme)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.profile_form.theme, "dark".to_string(), "Dark");
                                ui.selectable_value(&mut self.profile_form.theme, "light".to_string(), "Light");
                                ui.selectable_value(&mut self.profile_form.theme, "auto".to_string(), "Auto");
                            });
                    });
                    
                    ui.add_space(8.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Language:");
                        egui::ComboBox::from_label("")
                            .selected_text(&self.profile_form.language)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.profile_form.language, "en".to_string(), "English");
                                ui.selectable_value(&mut self.profile_form.language, "es".to_string(), "Spanish");
                                ui.selectable_value(&mut self.profile_form.language, "fr".to_string(), "French");
                                ui.selectable_value(&mut self.profile_form.language, "de".to_string(), "German");
                                ui.selectable_value(&mut self.profile_form.language, "zh".to_string(), "Chinese");
                            });
                    });
                    
                    ui.add_space(8.0);
                    
                    ui.checkbox(&mut self.profile_form.notifications_enabled, "Enable notifications");
                    ui.checkbox(&mut self.profile_form.auto_accept_connections, "Auto-accept connections");
                    ui.checkbox(&mut self.profile_form.require_confirmation, "Require confirmation for sensitive actions");
                    
                    ui.add_space(8.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Max concurrent sessions:");
                        ui.add(egui::Slider::new(&mut self.profile_form.max_concurrent_sessions, 1..=10));
                    });
                    
                    ui.add_space(16.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("💾 Save Changes").clicked() {
                            self.handle_profile_update(auth_service);
                        }
                        
                        if ui.button("🔄 Refresh").clicked() {
                            self.load_profile_data(&user_clone);
                        }
                        
                        if ui.button("🚪 Logout").clicked() {
                            self.handle_logout(auth_service);
                        }
                    });
                });
        } else {
            ui.vertical_centered(|ui| {
                ui.label("No user logged in");
                if ui.button("Go to Login").clicked() {
                    self.state = AuthPanelState::Login;
                }
            });
        }
    }

    /// Render reset password panel
    fn render_reset_password_panel(&mut self, ui: &mut egui::Ui, auth_service: &mut AuthService, action: &mut AuthAction) {
        ui.vertical_centered(|ui| {
            ui.heading("🔑 Reset Password");
            ui.add_space(16.0);
        });

        egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .rounding(8.0)
            .inner_margin(20.0)
            .show(ui, |ui| {
                ui.label("Enter your email address and we'll send you a link to reset your password.");
                ui.add_space(16.0);
                
                ui.horizontal(|ui| {
                    ui.label("Email:");
                    ui.text_edit_singleline(&mut self.reset_password_form.email);
                });
                
                ui.add_space(16.0);
                
                ui.horizontal(|ui| {
                    if ui.button("📧 Send Reset Link").clicked() {
                        self.handle_password_reset(auth_service);
                    }
                    
                    if ui.button("❌ Cancel").clicked() {
                        self.state = AuthPanelState::Login;
                        self.clear_forms();
                    }
                });
                
                ui.add_space(16.0);
                
                ui.separator();
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("Remember your password?");
                    if ui.link("Back to Login").clicked() {
                        self.state = AuthPanelState::Login;
                        self.clear_messages();
                    }
                });
            });
    }

    /// Handle login
    fn handle_login(&mut self, auth_service: &mut AuthService) {
        if self.login_form.email.is_empty() || self.login_form.password.is_empty() {
            self.error_message = "Please fill in all fields".to_string();
            return;
        }

        self.loading = true;
        self.state = AuthPanelState::Loading;

        let request = LoginRequest {
            email: self.login_form.email.clone(),
            password: self.login_form.password.clone(),
        };

        // Create a future to handle the async login
        let auth_future = async move {
            // This would be implemented with actual backend call
            // For now, we'll simulate the async behavior
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            Ok(())
        };

        // Store the future for execution in the main app loop
        self.pending_login = Some(LoginOperation {
            request,
            future: Box::pin(auth_future),
        });
    }

    /// Handle registration
    fn handle_register(&mut self, _auth_service: &mut AuthService) {
        if self.register_form.email.is_empty() 
            || self.register_form.password.is_empty()
            || self.register_form.username.is_empty()
            || self.register_form.display_name.is_empty() {
            self.error_message = "Please fill in all fields".to_string();
            return;
        }

        if self.register_form.password != self.register_form.confirm_password {
            self.error_message = "Passwords do not match".to_string();
            return;
        }

        if !self.register_form.agree_terms {
            self.error_message = "Please agree to the Terms of Service".to_string();
            return;
        }

        self.loading = true;
        self.state = AuthPanelState::Loading;

        let request = RegisterRequest {
            email: self.register_form.email.clone(),
            password: self.register_form.password.clone(),
            username: self.register_form.username.clone(),
            display_name: self.register_form.display_name.clone(),
        };

        // Create a future to handle the async registration
        let register_future = async move {
            // This would be implemented with actual backend call
            // For now, we'll simulate the async behavior
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
            Ok(())
        };

        // Store the future for execution in the main app loop
        self.pending_register = Some(RegisterOperation {
            request,
            future: Box::pin(register_future),
        });
    }

    /// Handle profile update
    fn handle_profile_update(&mut self, _auth_service: &mut AuthService) {
        let preferences = UserPreferences {
            theme: self.profile_form.theme.clone(),
            language: self.profile_form.language.clone(),
            notifications_enabled: self.profile_form.notifications_enabled,
            auto_accept_connections: self.profile_form.auto_accept_connections,
            require_confirmation: self.profile_form.require_confirmation,
            max_concurrent_sessions: self.profile_form.max_concurrent_sessions,
            default_permissions: vec!["screen_sharing".to_string(), "remote_control".to_string()],
        };

        let _request = UpdateProfileRequest {
            display_name: if self.profile_form.display_name.is_empty() { None } else { Some(self.profile_form.display_name.clone()) },
            avatar_url: if self.profile_form.avatar_url.is_empty() { None } else { Some(self.profile_form.avatar_url.clone()) },
            preferences: Some(preferences),
        };

        // Simulate profile update
        self.success_message = "Profile updated successfully!".to_string();
    }

    /// Handle password reset
    fn handle_password_reset(&mut self, _auth_service: &AuthService) {
        if self.reset_password_form.email.is_empty() {
            self.error_message = "Please enter your email address".to_string();
            return;
        }

        // Simulate password reset
        self.state = AuthPanelState::Login;
        self.clear_messages();
        self.success_message = "Password reset link sent! Please check your email.".to_string();
    }

    /// Handle logout
    fn handle_logout(&mut self, _auth_service: &mut AuthService) {
        // Simulate logout
        self.state = AuthPanelState::Login;
        self.clear_forms();
        self.clear_messages();
    }

    /// Load profile data
    fn load_profile_data(&mut self, user: &UserAccount) {
        self.profile_form.display_name = user.display_name.clone();
        self.profile_form.avatar_url = user.avatar_url.clone().unwrap_or_default();
        self.profile_form.theme = user.preferences.theme.clone();
        self.profile_form.language = user.preferences.language.clone();
        self.profile_form.notifications_enabled = user.preferences.notifications_enabled;
        self.profile_form.auto_accept_connections = user.preferences.auto_accept_connections;
        self.profile_form.require_confirmation = user.preferences.require_confirmation;
        self.profile_form.max_concurrent_sessions = user.preferences.max_concurrent_sessions;
    }

    /// Clear all forms
    fn clear_forms(&mut self) {
        self.login_form = LoginForm::default();
        self.register_form = RegisterForm::default();
        self.profile_form = ProfileForm::default();
        self.reset_password_form = ResetPasswordForm::default();
    }

    /// Clear messages
    fn clear_messages(&mut self) {
        self.error_message.clear();
        self.success_message.clear();
    }

    /// Set current user and switch to profile view
    pub fn set_user(&mut self, user: &UserAccount) {
        self.load_profile_data(user);
        self.state = AuthPanelState::Profile;
        self.clear_messages();
    }

    /// Get current panel state
    pub fn get_state(&self) -> &AuthPanelState {
        &self.state
    }

    /// Update async operations (call this in the main UI loop)
    pub fn update_async_operations(&mut self, auth_service: &mut AuthService) {
        // Check pending login
        if let Some(mut login_op) = self.pending_login.take() {
            if let std::task::Poll::Ready(result) = 
                std::future::Future::poll(Pin::new(&mut login_op.future), &mut std::task::Context::from_waker(futures::task::noop_waker_ref())) {
                
                match result {
                    Ok(()) => {
                        // Simulate successful login
                        self.state = AuthPanelState::Profile;
                        self.clear_messages();
                        self.success_message = "Login successful!".to_string();
                    }
                    Err(e) => {
                        self.error_message = format!("Login failed: {}", e);
                        self.state = AuthPanelState::Login;
                    }
                }
                self.loading = false;
            } else {
                self.pending_login = Some(login_op);
            }
        }

        // Check pending registration
        if let Some(mut register_op) = self.pending_register.take() {
            if let std::task::Poll::Ready(result) = 
                std::future::Future::poll(Pin::new(&mut register_op.future), &mut std::task::Context::from_waker(futures::task::noop_waker_ref())) {
                
                match result {
                    Ok(()) => {
                        // Simulate successful registration
                        self.state = AuthPanelState::Login;
                        self.clear_messages();
                        self.success_message = "Registration successful! Please check your email to verify your account.".to_string();
                    }
                    Err(e) => {
                        self.error_message = format!("Registration failed: {}", e);
                        self.state = AuthPanelState::Register;
                    }
                }
                self.loading = false;
            } else {
                self.pending_register = Some(register_op);
            }
        }
    }
}

/// Async login operation
pub struct LoginOperation {
    pub request: LoginRequest,
    pub future: Pin<Box<dyn Future<Output = Result<(), String>> + Send>>,
}

/// Async registration operation
pub struct RegisterOperation {
    pub request: RegisterRequest,
    pub future: Pin<Box<dyn Future<Output = Result<(), String>> + Send>>,
}

/// Authentication actions
#[derive(Debug, Clone, PartialEq)]
pub enum AuthAction {
    None,
    LoginSuccess,
    RegisterSuccess,
    Logout,
}

impl AuthPanel {
    /// Set panel state
    pub fn set_state(&mut self, state: AuthPanelState) {
        self.state = state;
        self.clear_messages();
    }
}
