pub mod app;
pub mod devices;
pub mod settings;
pub mod notifications;
pub mod connection_dialog;
pub mod remote_control_panel;
pub mod file_transfer_panel;
pub mod premium_features;
pub mod permission_panel;
pub mod screen_preview;
pub mod streaming_panel;

pub use app::GenXLinkApp;
pub use notifications::{Notification, NotificationManager, NotificationType};
pub use connection_dialog::{ConnectionDialog, ConnectionDialogState, ConnectionStep, DialogResult};
