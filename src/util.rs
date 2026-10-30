use bevy::prelude::*;

pub const LEFT_PANEL_W: f32 = 280.0;
pub const RIGHT_PANEL_W: f32 = 336.0;
pub const TOP_BAR_H: f32 = 64.0;
pub const BOTTOM_DOCK_H: f32 = 60.0;
pub const NODE_RADIUS: f32 = 0.115;
pub const NODE_LIFT: f32 = 0.10;
pub const DEFAULT_GLOBE_RADIUS: f32 = 2.8;
pub const MIN_THOUGHT_DIST: f32 = 4.0 * NODE_RADIUS;
pub const GROUP_ANGULAR_RADIUS: f32 = 0.16;
pub const INK: Color = Color::srgb(0.91, 0.94, 0.98);
pub const MUTED: Color = Color::srgb(0.58, 0.64, 0.73);
pub const FAINT: Color = Color::srgb(0.36, 0.43, 0.53);
pub const ACCENT: Color = Color::srgb(0.35, 0.85, 0.91);
pub const MINT: Color = Color::srgb(0.43, 0.91, 0.72);
pub const AMBER: Color = Color::srgb(1.0, 0.71, 0.33);
pub const CORAL: Color = Color::srgb(1.0, 0.48, 0.47);
pub const PANEL: Color = Color::srgba(0.027, 0.043, 0.075, 0.97);
pub const LINE: Color = Color::srgba(0.10, 0.15, 0.23, 1.0);

pub fn color_from_hex(hex: &str) -> Color {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return MUTED;
    }
    let Ok(r) = u8::from_str_radix(&hex[0..2], 16) else {
        return MUTED;
    };
    let Ok(g) = u8::from_str_radix(&hex[2..4], 16) else {
        return MUTED;
    };
    let Ok(b) = u8::from_str_radix(&hex[4..6], 16) else {
        return MUTED;
    };
    Color::srgb_u8(r, g, b)
}

pub fn alpha(color: Color, a: f32) -> Color {
    let c = color.to_srgba();
    Color::srgba(c.red, c.green, c.blue, a)
}
