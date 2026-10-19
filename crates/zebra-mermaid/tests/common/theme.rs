//! Helpers for Zebra theme tests.

use zebra_mermaid::{ColorMode, MermaidPalette, RenderOptions};

/// Seventeen distinct, easily recognized colors and six series colors. Series 1 is dark and
/// series 3 light, so that labels on them must take opposite colors.
pub fn sentinel() -> MermaidPalette {
    MermaidPalette {
        background: "#F0F0E0".into(),
        surface: "#A10001".into(),
        surface_secondary: "#A20002".into(),
        text: "#101020".into(),
        secondary_text: "#A40004".into(),
        accent: "#A50005".into(),
        border: "#A60006".into(),
        selection: "#A70007".into(),
        warning_background: "#A80008".into(),
        warning_text: "#A90009".into(),
        danger_background: "#AA000A".into(),
        danger_text: "#AB000B".into(),
        success_background: "#AC000C".into(),
        success_text: "#AD000D".into(),
        edge: Some("#AE000E".into()),
        group_background: Some("#AF000F".into()),
        group_border: Some("#B00010".into()),
        series: Some(["#C10001", "#202020", "#C30003", "#E0E0E0", "#C50005", "#C60006"].map(String::from)),
    }
}

/// Every (JSON field name, color) of a palette, series as `series[i]`.
pub fn roles(p: &MermaidPalette) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = p.required().iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    for (k, v) in [("edge", &p.edge), ("groupBackground", &p.group_background), ("groupBorder", &p.group_border)] {
        if let Some(v) = v {
            out.push((k.to_string(), v.clone()));
        }
    }
    if let Some(series) = &p.series {
        for (i, v) in series.iter().enumerate() {
            out.push((format!("series[{i}]"), v.clone()));
        }
    }
    out
}

pub fn resolved_options(palette: Option<MermaidPalette>) -> RenderOptions {
    RenderOptions { color_mode: ColorMode::Resolved, palette, ..Default::default() }
}

/// Resolved output must not need CSS at run time.
pub fn assert_no_css_runtime(svg: &str) {
    for needle in ["var(", "color-mix(", "light-dark(", "@media", "@import", "--_", "--pele-"] {
        assert!(!svg.contains(needle), "resolved SVG contains {needle}: {svg}");
    }
}

/// Replaces every `#RRGGBB` with a placeholder, to compare geometry alone.
pub fn without_colors(svg: &str) -> String {
    let b = svg.as_bytes();
    let mut out = String::with_capacity(svg.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'#' && i + 7 <= b.len() && b[i + 1..i + 7].iter().all(|c| c.is_ascii_hexdigit()) {
            out.push_str("#COLOR");
            i += 7;
        } else {
            let ch = svg[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}
