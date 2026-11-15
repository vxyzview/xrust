//! Colors and built-in terminal color schemes.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Color {
    /// Terminal default foreground/background (scheme-defined).
    #[default]
    Default,
    /// Indexed palette entry (0-255). Palette may be overridden at runtime.
    Indexed(u8),
    /// Direct RGB, `0xRRGGBB`.
    Rgb(u32),
}

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color::Rgb(((r as u32) << 16) | ((g as u32) << 8) | b as u32)
    }

    /// Parse `#RRGGBB` or `RRGGBB` (used for OSC 4/10/11 and settings import).
    pub fn parse_hex(s: &str) -> Option<Self> {
        let t = s.strip_prefix('#').unwrap_or(s);
        if t.len() != 6 || !t.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let v = u32::from_str_radix(t, 16).ok()?;
        Some(Color::Rgb(v))
    }

    pub fn to_hex(self) -> String {
        match self {
            Color::Rgb(v) => format!("#{v:06X}"),
            _ => String::new(),
        }
    }

    /// Relative luminance (0..1), ITU-R BT.709 weights.
    pub fn luminance(self) -> f64 {
        let v = match self {
            Color::Rgb(v) => v,
            _ => return 0.0,
        };
        let c = |sh: u32| -> f64 {
            let x = ((v >> sh) & 0xFF) as f64 / 255.0;
            if x <= 0.03928 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * c(16) + 0.7152 * c(8) + 0.0722 * c(0)
    }

    /// WCAG contrast ratio between two colors; both must be `Rgb`.
    pub fn contrast(a: Color, b: Color) -> f64 {
        let (la, lb) = (a.luminance(), b.luminance());
        let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
        (hi + 0.05) / (lo + 0.05)
    }
}

/// One full terminal scheme. `palette` holds ANSI 0-15; indices 16-255 are
/// derived from the xterm-256 standard table unless overridden.
#[derive(Debug, Clone, PartialEq)]
pub struct Scheme {
    pub name: String,
    pub is_dark: bool,
    pub foreground: u32,
    pub background: u32,
    pub cursor: u32,
    pub selection: u32,
    pub palette: [u32; 16],
}

impl Scheme {
    /// Standard xterm 256-color palette for indices >= 16.
    pub fn extended_index(i: u8) -> u32 {
        match i {
            0..=15 => 0,
            16..=231 => {
                let i = i as u32 - 16;
                let r = i / 36;
                let g = (i % 36) / 6;
                let b = i % 6;
                let f = |v: u32| if v == 0 { 0 } else { 55 + v * 40 };
                (f(r) << 16) | (f(g) << 8) | f(b)
            }
            232..=255 => {
                let g = 8 + (i as u32 - 232) * 10;
                (g << 16) | (g << 8) | g
            }
        }
    }

    pub const fn alpine_dark() -> Self {
        Self {
            name: String::new(), // replaced by `named` below (const fn limitation)
            is_dark: true,
            foreground: 0xE8E8E8,
            background: 0x0E1218,
            cursor: 0x4FC3F7,
            selection: 0x27476E,
            palette: [
                0x2E3440, 0xD9555F, 0x88C98B, 0xD9B36C, 0x6E9FD4, 0xB57FC6, 0x63C6C1, 0xC8CDD4,
                0x5C6672, 0xEC7179, 0xA5D9A8, 0xE6C98C, 0x8FBCE8, 0xC9A0D8, 0x86DBD6, 0xECEFF3,
            ],
        }
    }

    pub fn named_schemes() -> Vec<Scheme> {
        let mut v = vec![
            Self::alpine_dark(),
            scheme(
                "Alpine Light",
                false,
                0x1B2430,
                0xF4F6F9,
                0x0B57D0,
                0xC7DBF7,
                [
                    0x3A4252, 0xC23B4E, 0x3E7A46, 0x9A6B16, 0x2E5F94, 0x7C4D8F, 0x2E7E78, 0x6B7280,
                    0x525B66, 0xD95A6C, 0x3F8A48, 0xA8760F, 0x3E76C4, 0x955F9E, 0x27918A, 0x37414D,
                ],
            ),
            scheme(
                "AMOLED Pure Black",
                true,
                0xE4E4E4,
                0x000000,
                0xFFFFFF,
                0x1E6F50,
                [
                    0x121212, 0xFF6E6E, 0x7BE28A, 0xE7C878, 0x82AAFF, 0xC792EA, 0x89DDFF, 0xBDBDBD,
                    0x4A4A4A, 0xFF8B8B, 0xA5E8B0, 0xF0DCA0, 0xA6C8FF, 0xDCAFF2, 0xA7E8EE, 0xEAEAEA,
                ],
            ),
            scheme(
                "Rose Ash",
                true,
                0xEDE6E4,
                0x191417,
                0xEB8CAC,
                0x4A2C3A,
                [
                    0x26232B, 0xE8697D, 0x83C99E, 0xD9A66C, 0x8CA6E0, 0xC69ACF, 0x7FC9C4, 0xD8D3D1,
                    0x5D5760, 0xF08596, 0xA4DDB9, 0xE7C18E, 0xA9BFEC, 0xD9B4E5, 0x9CDAD6, 0xEFEAEE,
                ],
            ),
            scheme(
                "Dracula",
                true,
                0xF8F8F2,
                0x282A36,
                0xF8F8F0,
                0x44475A,
                [
                    0x21222C, 0xFF5555, 0x50FA7B, 0xF1FA8C, 0xBD93F9, 0xFF79C6, 0x8BE9FD, 0xF8F8F2,
                    0x6272A4, 0xFF6E6E, 0x69FF94, 0xFFFFA5, 0xD6ACFF, 0xFF92DF, 0xA4FFFF, 0xFFFFFF,
                ],
            ),
            scheme(
                "Nord",
                true,
                0xD8DEE9,
                0x2E3440,
                0x88C0D0,
                0x434C5E,
                [
                    0x3B4252, 0xBF616A, 0xA3BE8C, 0xEBCB8B, 0x81A1C1, 0xB48EAD, 0x88C0D0, 0xE5E9F0,
                    0x4C566A, 0xD08770, 0xB48EAD, 0xEBCB8B, 0x81A1C1, 0xB48EAD, 0x8FBCBB, 0xECEFF4,
                ],
            ),
            scheme(
                "Gruvbox Dark",
                true,
                0xEBDBB2,
                0x282828,
                0xFABD2F,
                0x504945,
                [
                    0x282828, 0xCC241D, 0x98971A, 0xD79921, 0x458588, 0xB16286, 0x689D6A, 0xA89984,
                    0x928374, 0xFB4934, 0xB8BB26, 0xFABD2F, 0x83A598, 0xD3869B, 0x8EC07C, 0xEBDBB2,
                ],
            ),
            scheme(
                "Gruvbox Light",
                false,
                0x3C3836,
                0xFBF1C7,
                0xB57614,
                0xDDD6C0,
                [
                    0xFBF1C7, 0x9D0006, 0x79740E, 0xB57614, 0x076678, 0x8F3F71, 0x427B58, 0x7C6F64,
                    0x928374, 0xAF3A03, 0x879100, 0xD65D0E, 0x458588, 0xB0548D, 0x4899A0, 0x3C3836,
                ],
            ),
            scheme(
                "Solarized Dark",
                true,
                0x93A1A1,
                0x002B36,
                0x268BD2,
                0x073642,
                [
                    0x073642, 0xDC322F, 0x859900, 0xB58900, 0x268BD2, 0xD33682, 0x2AA198, 0xEEE8D5,
                    0x586E75, 0xCB4B16, 0x859900, 0xB58900, 0x268BD2, 0xD33682, 0x2AA198, 0xFDF6E3,
                ],
            ),
            scheme(
                "Solarized Light",
                false,
                0x52676F,
                0xFDF6E3,
                0x268BD2,
                0xEEE8D5,
                [
                    0xFDF6E3, 0xDC322F, 0x859900, 0xB58900, 0x268BD2, 0xD33682, 0x2AA198, 0x93A1A1,
                    0x839496, 0xCB4B16, 0x859900, 0xB58900, 0x268BD2, 0xD33682, 0x2AA198, 0x002B36,
                ],
            ),
            scheme(
                "One Dark",
                true,
                0xDCDFE4,
                0x282C34,
                0x61AFEF,
                0x3E4451,
                [
                    0x282C34, 0xE06C75, 0x98C379, 0xE5C07B, 0x61AFEF, 0xC678DD, 0x56B6C2, 0xDCDFE4,
                    0x5C6370, 0xE06C75, 0x98C379, 0xE5C07B, 0x61AFEF, 0xC678DD, 0x56B6C2, 0xFFFFFF,
                ],
            ),
            scheme(
                "Tokyo Night",
                true,
                0xC0CAF5,
                0x1A1B26,
                0x7AA2F7,
                0x33467C,
                [
                    0x1D202F, 0xF7768E, 0x9ECE6A, 0xE0AF68, 0x7AA2F7, 0xBB9AF7, 0x7DCFFF, 0xA9B1D6,
                    0x414868, 0xFF7A93, 0xB9F27C, 0xFF9E64, 0x7DA6FF, 0xBB9AF7, 0x0DB9D7, 0xC0CAF5,
                ],
            ),
        ];
        // Fill names produced by the const constructor.
        v[0].name = "Alpine Dark".into();
        v
    }
}

#[allow(clippy::too_many_arguments)]
fn scheme(
    name: &str,
    is_dark: bool,
    fg: u32,
    bg: u32,
    cursor: u32,
    selection: u32,
    palette: [u32; 16],
) -> Scheme {
    Scheme {
        name: name.to_string(),
        is_dark,
        foreground: fg,
        background: bg,
        cursor,
        selection,
        palette,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_variants() {
        assert_eq!(Color::parse_hex("#FF8800"), Some(Color::Rgb(0xFF8800)));
        assert_eq!(Color::parse_hex("00ff00"), Some(Color::Rgb(0x00FF00)));
        assert_eq!(Color::parse_hex("#12G456"), None);
        assert_eq!(Color::parse_hex("#123"), None);
        assert_eq!(Color::parse_hex(""), None);
    }

    #[test]
    fn hex_roundtrip() {
        let c = Color::Rgb(0xAB12CD);
        assert_eq!(c.to_hex(), "#AB12CD");
        assert_eq!(Color::parse_hex(&c.to_hex()), Some(c));
    }

    #[test]
    fn luminance_and_contrast() {
        assert!(Color::Rgb(0xFFFFFF).luminance() > 0.99);
        assert!(Color::Rgb(0x000000).luminance() < 0.01);
        let ratio = Color::contrast(Color::Rgb(0xFFFFFF), Color::Rgb(0x000000));
        assert!((ratio - 21.0).abs() < 0.1);
        assert!(Color::contrast(Color::Rgb(0x777777), Color::Rgb(0x888888)) < 2.0);
    }

    #[test]
    fn extended_palette_256() {
        assert_eq!(Scheme::extended_index(16), 0x000000);
        assert_eq!(Scheme::extended_index(231), 0xFFFFFF);
        assert_eq!(Scheme::extended_index(232), 0x080808);
        assert_eq!(Scheme::extended_index(255), 0xEEEEEE);
        assert_eq!(Scheme::extended_index(196), 0xFF0000);
    }

    #[test]
    fn builtin_schemes_valid() {
        let schemes = Scheme::named_schemes();
        assert!(schemes.len() >= 12, "need at least 12 built-in schemes");
        for s in &schemes {
            assert!(!s.name.is_empty());
            assert_ne!(s.foreground, s.background, "{}", s.name);
            assert!(
                Color::contrast(Color::Rgb(s.foreground), Color::Rgb(s.background)) > 4.5,
                "low contrast in {}",
                s.name
            );
            assert!(s.palette.iter().all(|&p| p <= 0xFFFFFF));
        }
        let amoled = schemes
            .iter()
            .find(|s| s.name == "AMOLED Pure Black")
            .unwrap();
        assert_eq!(amoled.background, 0x000000, "AMOLED must be pure black");
        let rose = schemes.iter().find(|s| s.name == "Rose Ash").unwrap();
        assert!(rose.is_dark);
    }
}
