/// GenXLink application icon data
/// 
/// This module contains the embedded icon for the GenXLink application.
/// The icon features a stylized "X" with cyan-to-pink gradient and particles,
/// representing cross-platform connectivity.

use egui::IconData;

/// Embedded icon data (256x256 PNG converted to RGBA)
/// This is the GenXLink logo with the stylized X
static ICON_DATA: &[u8] = include_bytes!("../../../assets/icons/genxlink.ico");

/// Load the GenXLink application icon
pub fn load_icon() -> IconData {
    // Try to load from embedded ICO file
    if let Some(icon) = load_icon_from_ico() {
        return icon;
    }
    // Fallback to programmatic icon
    create_genxlink_icon()
}

/// Load icon from embedded ICO data
fn load_icon_from_ico() -> Option<IconData> {
    // ICO files can contain multiple images, we want the largest one
    // For simplicity, we'll use the programmatic fallback for now
    // In production, use the `ico` crate to parse the ICO file
    None
}

/// Create a programmatic version of the GenXLink icon
/// This creates a 256x256 icon with the GenXLink "X" design
fn create_genxlink_icon() -> IconData {
    let size = 256;
    let mut rgba = vec![0u8; size * size * 4];
    
    // Background: Dark navy blue (#0a1628)
    let bg_r = 10;
    let bg_g = 22;
    let bg_b = 40;
    
    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            
            // Calculate distance from center for rounded corners
            let center_x = size as f32 / 2.0;
            let center_y = size as f32 / 2.0;
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let _dist_from_center = (dx * dx + dy * dy).sqrt();
            let corner_radius = size as f32 * 0.45; // Rounded corners
            
            // Check if we're within the rounded rectangle
            let in_bounds = if x < size / 2 && y < size / 2 {
                // Top-left corner
                let corner_dist = ((x as f32 - corner_radius).powi(2) + (y as f32 - corner_radius).powi(2)).sqrt();
                corner_dist <= corner_radius || x as f32 >= corner_radius || y as f32 >= corner_radius
            } else if x >= size / 2 && y < size / 2 {
                // Top-right corner
                let corner_dist = ((x as f32 - (size as f32 - corner_radius)).powi(2) + (y as f32 - corner_radius).powi(2)).sqrt();
                corner_dist <= corner_radius || x as f32 <= size as f32 - corner_radius || y as f32 >= corner_radius
            } else if x < size / 2 && y >= size / 2 {
                // Bottom-left corner
                let corner_dist = ((x as f32 - corner_radius).powi(2) + (y as f32 - (size as f32 - corner_radius)).powi(2)).sqrt();
                corner_dist <= corner_radius || x as f32 >= corner_radius || y as f32 <= size as f32 - corner_radius
            } else {
                // Bottom-right corner
                let corner_dist = ((x as f32 - (size as f32 - corner_radius)).powi(2) + (y as f32 - (size as f32 - corner_radius)).powi(2)).sqrt();
                corner_dist <= corner_radius || x as f32 <= size as f32 - corner_radius || y as f32 <= size as f32 - corner_radius
            };
            
            if !in_bounds {
                // Transparent outside rounded corners
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 0;
                continue;
            }
            
            // Background color
            rgba[idx] = bg_r;
            rgba[idx + 1] = bg_g;
            rgba[idx + 2] = bg_b;
            rgba[idx + 3] = 255;
            
            // Draw the "X" shape with gradient
            // The X is centered and spans from about 25% to 75% of the icon
            let x_start = size as f32 * 0.25;
            let x_end = size as f32 * 0.75;
            let y_start = size as f32 * 0.20;
            let y_end = size as f32 * 0.70;
            
            let stroke_width = size as f32 * 0.08;
            
            // Diagonal from top-left to bottom-right (cyan to purple)
            let dist_to_diag1 = ((x as f32 - x_start) * (y_end - y_start) - (y as f32 - y_start) * (x_end - x_start)).abs() 
                / ((x_end - x_start).powi(2) + (y_end - y_start).powi(2)).sqrt();
            
            if dist_to_diag1 < stroke_width && x as f32 >= x_start && x as f32 <= x_end && y as f32 >= y_start && y as f32 <= y_end {
                // Gradient from cyan (#00d9ff) to purple (#8b5cf6)
                let t = (x as f32 - x_start) / (x_end - x_start);
                let r = (0.0 * (1.0 - t) + 139.0 * t) as u8;
                let g = (217.0 * (1.0 - t) + 92.0 * t) as u8;
                let b = (255.0 * (1.0 - t) + 246.0 * t) as u8;
                
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = 255;
            }
            
            // Diagonal from top-right to bottom-left (pink to cyan)
            let dist_to_diag2 = ((x as f32 - x_end) * (y_end - y_start) + (y as f32 - y_start) * (x_end - x_start)).abs() 
                / ((x_end - x_start).powi(2) + (y_end - y_start).powi(2)).sqrt();
            
            if dist_to_diag2 < stroke_width && x as f32 >= x_start && x as f32 <= x_end && y as f32 >= y_start && y as f32 <= y_end {
                // Gradient from pink (#ff6ec7) to cyan (#00d9ff)
                let t = (x as f32 - x_start) / (x_end - x_start);
                let r = (255.0 * (1.0 - t) + 0.0 * t) as u8;
                let g = (110.0 * (1.0 - t) + 217.0 * t) as u8;
                let b = (199.0 * (1.0 - t) + 255.0 * t) as u8;
                
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = 255;
            }
            
            // Add particle effects around the X
            // Small dots scattered around the X
            let particles = [
                (0.35, 0.25, 0.02), // Top-left area
                (0.42, 0.28, 0.015),
                (0.65, 0.22, 0.02), // Top-right area
                (0.70, 0.25, 0.015),
                (0.72, 0.30, 0.018),
                (0.30, 0.55, 0.02), // Bottom-left area
                (0.35, 0.60, 0.015),
                (0.68, 0.50, 0.02), // Right area
                (0.72, 0.55, 0.015),
            ];
            
            for (px, py, psize) in particles.iter() {
                let particle_x = size as f32 * px;
                let particle_y = size as f32 * py;
                let particle_radius = size as f32 * psize;
                
                let dist_to_particle = ((x as f32 - particle_x).powi(2) + (y as f32 - particle_y).powi(2)).sqrt();
                
                if dist_to_particle < particle_radius {
                    // Gradient color based on position (cyan to pink)
                    let t = *px;
                    let r = (0.0 * (1.0 - t) + 255.0 * t) as u8;
                    let g = (217.0 * (1.0 - t) + 110.0 * t) as u8;
                    let b = 255;
                    
                    rgba[idx] = r;
                    rgba[idx + 1] = g;
                    rgba[idx + 2] = b;
                    rgba[idx + 3] = 255;
                }
            }
        }
    }
    
    IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}

/// Create a smaller 32x32 version for system tray
pub fn load_tray_icon() -> IconData {
    let size = 32;
    let mut rgba = vec![0u8; size * size * 4];
    
    // Simplified version for small size
    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            
            // Background
            rgba[idx] = 10;
            rgba[idx + 1] = 22;
            rgba[idx + 2] = 40;
            rgba[idx + 3] = 255;
            
            // Simple X shape
            let center = size as f32 / 2.0;
            let dist_to_diag1 = ((x as f32 - center).abs() - (y as f32 - center).abs()).abs();
            let dist_to_diag2 = ((x as f32 - center) + (y as f32 - center) - center).abs();
            
            let stroke = 3.0;
            
            if dist_to_diag1 < stroke || dist_to_diag2 < stroke {
                // Gradient color
                let t = x as f32 / size as f32;
                let r = (0.0 * (1.0 - t) + 255.0 * t) as u8;
                let g = (217.0 * (1.0 - t) + 110.0 * t) as u8;
                let b = 255;
                
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = 255;
            }
        }
    }
    
    IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}
