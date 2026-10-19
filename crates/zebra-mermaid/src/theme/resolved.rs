//! Semantic roles: what each colored part of a diagram is, and its color in each output mode.
//!
//! [`ResolvedTheme`] derives every role from a [`MermaidPalette`] exactly as Zebra's native
//! renderer does (`MermaidVisualStyle::resolve` and `apply_theme` in its `theme.rs`). Diagram
//! code names the role of each colored part together with the expression Pele writes there;
//! [`Theming::paint`] returns Pele's expression in CSS mode, which keeps that output identical
//! to Pele, and the role's concrete color in resolved mode.

use std::sync::{Arc, OnceLock};

use super::color::{contrasting_text_on_fill, mix_hex};
use super::palette::{CanvasBackground, ColorMode, MermaidPalette, MermaidThemeMode};

// Zebra's visual style mixes (MermaidVisualStyle::default).
const TEXT_MUTED_MIX: f32 = 0.40;
const TEXT_FAINT_MIX: f32 = 0.25;
const LINE_MIX: f32 = 0.50;
const INNER_STROKE_MIX: f32 = 0.12;

/// Every color role a diagram can use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// The canvas, label patches over lines, and fills that should read as empty.
    Background,
    /// Primary text.
    Text,
    /// Secondary text: captions, axis and edge labels, Pele's "muted" color.
    SecondaryText,
    /// Text mixed 40% toward the background (Zebra `muted_text`).
    MutedText,
    /// Text mixed 25% toward the background (Zebra `faint_text`).
    FaintText,
    /// Node and shape fill (`surface`).
    NodeFill,
    /// Node and shape outline (`border`).
    NodeStroke,
    /// Group (subgraph, cluster, boundary) fill.
    GroupFill,
    /// Group title band and secondary surfaces (`surfaceSecondary`).
    GroupHeader,
    /// Group outline (`groupBorder`, else `border`).
    GroupStroke,
    /// Dividers inside a shape and label patch outlines.
    InnerStroke,
    /// Connectors (`edge`).
    Line,
    /// Arrowheads (`accent`, as in Zebra's renderer).
    Arrow,
    Accent,
    Selection,
    WarningBackground,
    WarningText,
    DangerBackground,
    DangerText,
    SuccessBackground,
    SuccessText,
}

/// All roles resolved to `#RRGGBB`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolvedTheme {
    pub background: String,
    pub node_fill: String,
    pub node_stroke: String,
    pub group_fill: String,
    pub group_header: String,
    pub group_stroke: String,
    pub inner_stroke: String,
    pub text: String,
    pub secondary_text: String,
    pub muted_text: String,
    pub faint_text: String,
    pub line: String,
    pub arrow: String,
    pub accent: String,
    pub selection: String,
    pub warning_background: String,
    pub warning_text: String,
    pub danger_background: String,
    pub danger_text: String,
    pub success_background: String,
    pub success_text: String,
    /// The caller's six series colors, when given.
    pub series: Option<[String; 6]>,
    /// Twelve chart fills (pie slices, chart series), Zebra's `chart_palette`.
    pub chart: [String; 12],
    /// Eight series colors for lines and branches, Zebra's `chart_series`.
    pub lines: [String; 8],
}

impl ResolvedTheme {
    /// Resolves a palette with the native renderer's rules for missing optional fields.
    pub fn from_palette(p: &MermaidPalette) -> Self {
        // Groups (flowchart subgraphs and the like) take `groupBackground`, else the canvas, as
        // Zebra's `apply_layout` styles subgraphs.
        let group_fill = p.group_background.clone().unwrap_or_else(|| p.background.clone());
        let group_header = p.surface_secondary.clone();
        let mut t = ResolvedTheme {
            background: p.background.clone(),
            node_fill: p.surface.clone(),
            node_stroke: p.border.clone(),
            group_fill,
            group_header,
            group_stroke: p.group_border.clone().unwrap_or_else(|| p.border.clone()),
            inner_stroke: p.group_border.clone().unwrap_or_else(|| mix_hex(&p.text, &p.background, INNER_STROKE_MIX)),
            text: p.text.clone(),
            secondary_text: p.secondary_text.clone(),
            muted_text: mix_hex(&p.text, &p.background, TEXT_MUTED_MIX),
            faint_text: mix_hex(&p.text, &p.background, TEXT_FAINT_MIX),
            line: p.edge.clone().unwrap_or_else(|| mix_hex(&p.text, &p.background, LINE_MIX)),
            arrow: p.accent.clone(),
            accent: p.accent.clone(),
            selection: p.selection.clone(),
            warning_background: p.warning_background.clone(),
            warning_text: p.warning_text.clone(),
            danger_background: p.danger_background.clone(),
            danger_text: p.danger_text.clone(),
            success_background: p.success_background.clone(),
            success_text: p.success_text.clone(),
            series: p.series.clone(),
            chart: Default::default(),
            lines: Default::default(),
        };
        t.chart = chart_palette(&t);
        t.lines = chart_series(&t);
        t
    }

    pub fn color(&self, role: Role) -> &str {
        match role {
            Role::Background => &self.background,
            Role::Text => &self.text,
            Role::SecondaryText => &self.secondary_text,
            Role::MutedText => &self.muted_text,
            Role::FaintText => &self.faint_text,
            Role::NodeFill => &self.node_fill,
            Role::NodeStroke => &self.node_stroke,
            Role::GroupFill => &self.group_fill,
            Role::GroupHeader => &self.group_header,
            Role::GroupStroke => &self.group_stroke,
            Role::InnerStroke => &self.inner_stroke,
            Role::Line => &self.line,
            Role::Arrow => &self.arrow,
            Role::Accent => &self.accent,
            Role::Selection => &self.selection,
            Role::WarningBackground => &self.warning_background,
            Role::WarningText => &self.warning_text,
            Role::DangerBackground => &self.danger_background,
            Role::DangerText => &self.danger_text,
            Role::SuccessBackground => &self.success_background,
            Role::SuccessText => &self.success_text,
        }
    }

    /// The fill of chart series `index`, cycling through twelve.
    pub fn chart_color(&self, index: i64) -> &str {
        &self.chart[index.rem_euclid(12) as usize]
    }

    /// The line color of series `index`, cycling through eight.
    pub fn line_color(&self, index: i64) -> &str {
        &self.lines[index.rem_euclid(8) as usize]
    }

    /// A readable text color on `fill`, Zebra's contrast rule at full opacity.
    pub fn text_on(&self, fill: &str) -> String {
        contrasting_text_on_fill(fill, &self.text, &self.background, 1.0)
    }
}

fn chart_series(t: &ResolvedTheme) -> [String; 8] {
    if let Some(series) = &t.series {
        return std::array::from_fn(|i| if i < 6 { series[i].clone() } else { mix_hex(&series[i - 6], &t.text, 0.70) });
    }
    [
        t.arrow.clone(),
        t.success_text.clone(),
        t.warning_text.clone(),
        t.danger_text.clone(),
        mix_hex(&t.accent, &t.text, 0.55),
        mix_hex(&t.success_text, &t.text, 0.55),
        mix_hex(&t.warning_text, &t.text, 0.55),
        mix_hex(&t.danger_text, &t.text, 0.55),
    ]
}

fn chart_palette(t: &ResolvedTheme) -> [String; 12] {
    if let Some(series) = &t.series {
        return std::array::from_fn(|i| if i < 6 { series[i].clone() } else { mix_hex(&series[i - 6], &t.background, 0.70) });
    }
    [
        t.arrow.clone(),
        t.success_text.clone(),
        t.warning_text.clone(),
        t.danger_text.clone(),
        mix_hex(&t.accent, &t.background, 0.78),
        mix_hex(&t.success_text, &t.background, 0.78),
        mix_hex(&t.warning_text, &t.background, 0.78),
        mix_hex(&t.danger_text, &t.background, 0.78),
        t.selection.clone(),
        t.success_background.clone(),
        t.warning_background.clone(),
        t.danger_background.clone(),
    ]
}

fn default_theme(mode: MermaidThemeMode) -> Arc<ResolvedTheme> {
    static LIGHT: OnceLock<Arc<ResolvedTheme>> = OnceLock::new();
    static DARK: OnceLock<Arc<ResolvedTheme>> = OnceLock::new();
    let cell = match mode {
        MermaidThemeMode::Light => &LIGHT,
        MermaidThemeMode::Dark => &DARK,
    };
    cell.get_or_init(|| Arc::new(ResolvedTheme::from_palette(&MermaidPalette::default_for(mode)))).clone()
}

/// The color choices of one render.
#[derive(Debug, Clone)]
pub struct Theming {
    pub mode: ColorMode,
    pub theme_mode: MermaidThemeMode,
    pub canvas: CanvasBackground,
    /// Every role resolved, from the caller's palette or the default for the theme mode.
    /// Shared: the defaults for light and dark are resolved once per process.
    pub resolved: Arc<ResolvedTheme>,
    /// Whether CSS output keeps Pele's own fallback colors (light theme, no palette).
    pub pele_defaults: bool,
}

impl Theming {
    pub fn new(mode: ColorMode, theme_mode: MermaidThemeMode, palette: Option<&MermaidPalette>, canvas: CanvasBackground) -> Self {
        let pele_defaults = palette.is_none() && theme_mode == MermaidThemeMode::Light;
        let resolved = match palette {
            Some(p) => Arc::new(ResolvedTheme::from_palette(p)),
            None => default_theme(theme_mode),
        };
        Theming { mode, theme_mode, canvas, resolved, pele_defaults }
    }

    /// Pele's output: CSS mode, light, no palette, transparent canvas.
    pub fn pele() -> Self {
        Theming::new(ColorMode::Css, MermaidThemeMode::Light, None, CanvasBackground::Transparent)
    }

    #[inline]
    pub fn resolved_mode(&self) -> bool {
        self.mode == ColorMode::Resolved
    }

    /// The color of `role`: Pele's expression `css` in CSS mode, the concrete color otherwise.
    #[inline]
    pub fn paint<'a>(&'a self, role: Role, css: &'a str) -> &'a str {
        match self.mode {
            ColorMode::Css => css,
            ColorMode::Resolved => self.resolved.color(role),
        }
    }

    /// The fill of data series `index`: Pele's `seriesColor` in CSS mode, the chart palette in
    /// resolved mode.
    pub fn series(&self, index: i64) -> String {
        match self.mode {
            ColorMode::Css => {
                let k = index.rem_euclid(8) as usize;
                if self.pele_defaults {
                    crate::svg::theme::series_color(index)
                } else {
                    format!("var(--pele-series-{},{})", k + 1, self.resolved.chart_color(index))
                }
            }
            ColorMode::Resolved => self.resolved.chart_color(index).to_string(),
        }
    }

    /// The line color of series `index` (branches, curves): Pele's `seriesColor` in CSS mode.
    pub fn series_line(&self, index: i64) -> String {
        match self.mode {
            ColorMode::Css => self.series(index),
            ColorMode::Resolved => self.resolved.line_color(index).to_string(),
        }
    }

    /// Text drawn on a fill: Pele's expression in CSS mode, a contrasting color otherwise.
    pub fn text_on<'a>(&'a self, fill: &str, css: &'a str) -> std::borrow::Cow<'a, str> {
        match self.mode {
            ColorMode::Css => std::borrow::Cow::Borrowed(css),
            ColorMode::Resolved => std::borrow::Cow::Owned(self.resolved.text_on(fill)),
        }
    }

    /// The fallbacks written into the root's custom properties in CSS mode.
    pub(crate) fn css_root_style(&self) -> String {
        if self.pele_defaults {
            return crate::svg::theme::ROOT_STYLE.to_string();
        }
        let t = &self.resolved;
        format!(
            "--_bg:var(--pele-bg,{});--_fg:var(--pele-fg,{});--_m:var(--pele-muted,{});--_l:var(--pele-line,{});--_s:var(--pele-surface,{});--_a:var(--pele-surface-alt,{});--_b:var(--pele-border,{});--_c:var(--pele-accent,{});color:var(--_fg)",
            t.background, t.text, t.secondary_text, t.line, t.node_fill, t.group_header, t.node_stroke, t.accent
        )
    }

    /// The fallback of Pele's `--pele-critical` in CSS mode.
    pub fn critical(&self) -> String {
        match self.mode {
            ColorMode::Css if self.pele_defaults => crate::svg::theme::CRITICAL.to_string(),
            ColorMode::Css => format!("var(--pele-critical,{})", self.resolved.danger_text),
            ColorMode::Resolved => self.resolved.danger_text.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_roles_are_authoritative() {
        // Zebra's `semantic_edges_groups_and_series_are_authoritative`.
        let mut p = MermaidPalette::dark();
        p.edge = Some("#7AB3CD".into());
        p.group_background = Some("#192A3B".into());
        p.group_border = Some("#6987A5".into());
        let series = ["#F0E0D0", "#CC4466", "#4488AA", "#66AA88", "#BBAA44", "#9966AA"].map(String::from);
        p.series = Some(series.clone());
        let t = ResolvedTheme::from_palette(&p);
        assert_eq!(t.line, "#7AB3CD");
        assert_eq!(t.group_fill, "#192A3B");
        assert_eq!(t.group_stroke, "#6987A5");
        assert_eq!(&t.chart[..6], &series);
        assert_eq!(t.arrow, p.accent);
        assert_ne!(t.group_stroke, t.node_stroke);
    }

    #[test]
    fn native_fallbacks_for_missing_fields() {
        let p = MermaidPalette::light();
        let t = ResolvedTheme::from_palette(&p);
        assert_eq!(t.line, mix_hex(&p.text, &p.background, 0.5));
        assert_eq!(t.inner_stroke, mix_hex(&p.text, &p.background, 0.12));
        assert_eq!(t.group_stroke, p.border);
        // Without series, charts derive their colors.
        assert_eq!(t.group_fill, p.background);
        assert_eq!(t.chart[0], p.accent);
        assert_eq!(t.chart[8], p.selection);
        // The Swift-completed palette resolves differently.
        let s = ResolvedTheme::from_palette(&p.swift_completed());
        assert_eq!(s.line, p.secondary_text);
        assert_ne!(s.line, t.line);
        assert_eq!(s.chart[4], p.text);
        assert_ne!(s.chart[4], t.chart[4]);
    }

    #[test]
    fn paint_keeps_pele_css() {
        let css = Theming::pele();
        assert_eq!(css.paint(Role::GroupStroke, "var(--_b)"), "var(--_b)");
        assert_eq!(css.series(9), "var(--pele-series-2,#f58518)");
        let resolved = Theming::new(ColorMode::Resolved, MermaidThemeMode::Light, None, CanvasBackground::Opaque);
        assert_eq!(resolved.paint(Role::Background, "var(--_bg)"), "#FFFFFF");
    }
}
