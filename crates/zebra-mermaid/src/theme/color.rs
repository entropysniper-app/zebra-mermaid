//! Color arithmetic of Zebra's existing native renderer, kept exact so that derived colors
//! (mixed roles, contrasting labels) are the same values Zebra produces today.
//!
//! Ported from Zebra `Vendor/ZebraMermaidBridge/renderer/src/theme.rs` (`Rgb`, `mix_hex`) and
//! `core/src/theme.rs` (`contrasting_text_on_fill`, `relative_luminance`). Both compute in
//! `f32`, as the originals do.

/// An opaque sRGB color written as canonical `#RRGGBB` (digits and upper-case A–F).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb {
    /// Parses a canonical `#RRGGBB` color. Lower-case digits, short forms, alpha and CSS
    /// expressions are refused, as Zebra's palette boundary refuses them.
    pub fn parse(value: &str) -> Option<Self> {
        if value.len() != 7 || !value.starts_with('#') {
            return None;
        }
        let channels = &value.as_bytes()[1..];
        if !channels.iter().all(|b| b.is_ascii_digit() || (b'A'..=b'F').contains(b)) {
            return None;
        }
        Some(Rgb {
            red: u8::from_str_radix(&value[1..3], 16).ok()?,
            green: u8::from_str_radix(&value[3..5], 16).ok()?,
            blue: u8::from_str_radix(&value[5..7], 16).ok()?,
        })
    }

    /// Mixes component-wise in sRGB: `weight` of `self` and the rest of `background`.
    pub fn mix(self, background: Rgb, weight: f32) -> Rgb {
        fn channel(foreground: u8, background: u8, weight: f32) -> u8 {
            (f32::from(foreground) * weight + f32::from(background) * (1.0 - weight)).round() as u8
        }
        Rgb {
            red: channel(self.red, background.red, weight),
            green: channel(self.green, background.green, weight),
            blue: channel(self.blue, background.blue, weight),
        }
    }

    pub fn hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }
}

/// `foreground_weight` of `foreground` over `background`, as `#RRGGBB`. A color that does not
/// parse is passed through, as in Zebra.
pub fn mix_hex(foreground: &str, background: &str, foreground_weight: f32) -> String {
    let Some(f) = Rgb::parse(foreground) else {
        return foreground.to_string();
    };
    let Some(b) = Rgb::parse(background) else {
        return background.to_string();
    };
    f.mix(b, foreground_weight.clamp(0.0, 1.0)).hex()
}

/// Zebra core's lenient hex reader: `#RGB`, `#RRGGBB` or `#RRGGBBAA` (alpha ignored).
fn parse_hex(color: &str) -> Option<(f32, f32, f32)> {
    let hex = color.trim().strip_prefix('#')?;
    let digits = match hex.len() {
        3 => {
            let mut expanded = String::with_capacity(6);
            for ch in hex.chars() {
                expanded.push(ch);
                expanded.push(ch);
            }
            expanded
        }
        6 => hex.to_string(),
        8 => hex.get(..6)?.to_string(),
        _ => return None,
    };
    let r = u8::from_str_radix(digits.get(0..2)?, 16).ok()?;
    let g = u8::from_str_radix(digits.get(2..4)?, 16).ok()?;
    let b = u8::from_str_radix(digits.get(4..6)?, 16).ok()?;
    Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
}

fn relative_luminance(rgb: (f32, f32, f32)) -> f32 {
    let linear = |value: f32| {
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(rgb.0) + 0.7152 * linear(rgb.1) + 0.0722 * linear(rgb.2)
}

/// Picks a readable label color for text drawn on `fill` (at `opacity` over `background`).
/// The theme's own text and background colors are preferred; black or white is the fallback
/// that guarantees 4.5:1 contrast.
pub fn contrasting_text_on_fill(fill: &str, text: &str, background: &str, opacity: f32) -> String {
    let Some(fill_rgb) = parse_hex(fill) else {
        return text.to_string();
    };
    let Some(background_rgb) = parse_hex(background) else {
        return text.to_string();
    };
    let Some(text_rgb) = parse_hex(text) else {
        return text.to_string();
    };
    let alpha = opacity.clamp(0.0, 1.0);
    let composite = (
        fill_rgb.0 * alpha + background_rgb.0 * (1.0 - alpha),
        fill_rgb.1 * alpha + background_rgb.1 * (1.0 - alpha),
        fill_rgb.2 * alpha + background_rgb.2 * (1.0 - alpha),
    );
    let fill_luminance = relative_luminance(composite);
    let contrast = |candidate: (f32, f32, f32)| {
        let candidate_luminance = relative_luminance(candidate);
        (candidate_luminance.max(fill_luminance) + 0.05) / (candidate_luminance.min(fill_luminance) + 0.05)
    };
    let (themed, themed_rgb) = if contrast(background_rgb) > contrast(text_rgb) { (background, background_rgb) } else { (text, text_rgb) };
    if contrast(themed_rgb) >= 4.5 {
        return themed.to_string();
    }
    if contrast((0.0, 0.0, 0.0)) >= contrast((1.0, 1.0, 1.0)) {
        "#000000".to_string()
    } else {
        "#FFFFFF".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixes_like_zebra() {
        // The expectations of Zebra's own tests.
        assert_eq!(mix_hex("#000000", "#FFFFFF", 0.20), "#CCCCCC");
        assert_eq!(mix_hex("#FFFFFF", "#000000", 0.03), "#080808");
        assert_eq!(mix_hex("#abcdef", "#FFFFFF", 0.5), "#abcdef");
    }

    #[test]
    fn canonical_colors_only() {
        assert!(Rgb::parse("#2563EB").is_some());
        for bad in ["#2563eb", "#FFF", "#FFFFFFFF", "rgba(0,0,0,.2)", "2563EB", "#GGGGGG", ""] {
            assert!(Rgb::parse(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn contrast_follows_zebra() {
        // Zebra's semantic pie test: a bright slice takes the canvas color, a dark one the text.
        assert_eq!(contrasting_text_on_fill("#EFEFEF", "#F0F0F0", "#101010", 0.92), "#101010");
        assert_eq!(contrasting_text_on_fill("#252525", "#F0F0F0", "#101010", 0.92), "#F0F0F0");
        // The medium-lightness fallback: neither theme color reaches 4.5:1.
        assert_eq!(contrasting_text_on_fill("#5E7E51", "#322A20", "#FAE7C9", 0.92), "#000000");
    }
}
