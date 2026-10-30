use crate::scene::Color;

/// Colours of a drawing. Colour is a signal only: `alert` means too late, `warn` means look.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub page: Color,
    pub ink: Color,
    pub bright: Color,
    pub dim: Color,
    pub muted: Color,
    pub raised: Color,
    pub rule: Color,
    pub accent: Color,
    pub warn: Color,
    pub alert: Color,
    pub alert_ground: Color,
}

impl Palette {
    pub const fn labra() -> Self {
        Palette {
            page: Color(0x04, 0x07, 0x04),
            ink: Color(0xd3, 0xd7, 0xb5),
            bright: Color(0xde, 0xe1, 0xc8),
            dim: Color(0x9e, 0xa1, 0x88),
            muted: Color(0x58, 0x5d, 0x58),
            raised: Color(0x1d, 0x20, 0x1d),
            rule: Color(0x2a, 0x2e, 0x2a),
            accent: Color(0xe3, 0x3d, 0x84),
            warn: Color(0xd3, 0xdd, 0x2c),
            alert: Color(0xee, 0x58, 0x5d),
            alert_ground: Color(0x3a, 0x14, 0x16),
        }
    }

    pub fn colors(&self) -> [Color; 11] {
        [self.page, self.ink, self.bright, self.dim, self.muted, self.raised, self.rule, self.accent, self.warn, self.alert, self.alert_ground]
    }
}

pub const FONT: &str = "JetBrains Mono";
pub const REGULAR: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");
pub const BOLD: &[u8] = include_bytes!("../assets/JetBrainsMono-Bold.ttf");
/// Advance of one character in JetBrains Mono, in font sizes.
pub const ADVANCE: f32 = 0.6;
