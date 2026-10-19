//! Parse and render options. Ported from Pele `src/types.ts`.

use std::fmt;
use std::sync::Arc;

use crate::config::Config;
use crate::text::{IconResolver, TextMeasurer};
use crate::theme::{CanvasBackground, ColorMode, MermaidPalette, MermaidThemeMode};

/// Mermaid's default `maxTextSize`, counted in UTF-16 code units as JavaScript counts length.
pub const DEFAULT_LIMIT: usize = 50_000;
/// Mermaid has no limit on its output. A source within the limit can still describe a drawing
/// of many megabytes, which would stall the page it is put into. Counted in UTF-16 code units.
pub const DEFAULT_OUTPUT_LIMIT: usize = 4_000_000;
/// Mermaid stops at 500. `A & B --> C & D` multiplies: without a limit, a few kilobytes of
/// source ask for millions of edges.
pub const DEFAULT_MAX_EDGES: usize = 5000;

/// Options read when the text is parsed. Rendering a [`crate::ParsedDiagram`] does not read
/// them again.
#[derive(Clone, Default)]
pub struct ParseOptions {
    /// The longest source accepted, in UTF-16 code units. Defaults to [`DEFAULT_LIMIT`].
    pub limit: Option<usize>,
    /// The most edges a flowchart or agentflow may have. Defaults to [`DEFAULT_MAX_EDGES`].
    pub max_edges: Option<usize>,
    /// Configuration from the host. The diagram's own front matter and directives override it.
    pub config: Option<Config>,
}

/// Options for one render. Every field left `None` takes Pele's default.
#[derive(Clone, Default)]
pub struct RenderOptions {
    /// Measures label widths. Defaults to [`crate::MetricsMeasurer`], Pele's Arial estimate.
    pub measurer: Option<Arc<dyn TextMeasurer + Send + Sync>>,
    /// The fonts labels are measured in, and drawn in where the page sets no `--pele-font`.
    pub font_family: Option<String>,
    pub font_family_mono: Option<String>,
    /// Defaults to 16.
    pub font_size: Option<f64>,
    /// Prefix of the ids in the SVG. Defaults to `pele`.
    pub id_prefix: Option<String>,
    /// Whether the SVG scales down to its container. Defaults to true unless the diagram's
    /// configuration sets `useMaxWidth: false`.
    pub responsive: Option<bool>,
    /// The width the host has for the diagram. A type that can draw itself narrower does.
    pub max_width: Option<f64>,
    /// Whether a diagram that runs across may be drawn running down when it does not fit.
    pub auto_direction: Option<bool>,
    /// The width under which `auto_direction` applies. Defaults to 640.
    pub direction_breakpoint: Option<f64>,
    /// Defaults to 8.
    pub padding: Option<f64>,
    /// See [`ParseOptions::limit`].
    pub limit: Option<usize>,
    /// The longest SVG returned, in UTF-16 code units. Defaults to [`DEFAULT_OUTPUT_LIMIT`].
    pub output_limit: Option<usize>,
    /// See [`ParseOptions::max_edges`].
    pub max_edges: Option<usize>,
    /// Schemes a link may use besides a relative address. Defaults to http, https, mailto, tel.
    pub link_schemes: Option<Vec<String>>,
    /// Schemes an image may use besides a relative address. Defaults to http and https.
    pub image_schemes: Option<Vec<String>>,
    /// Whether links are written. Defaults to true.
    pub links: Option<bool>,
    /// Whether images are written. Defaults to true.
    pub images: Option<bool>,
    /// Link `rel` attribute. Defaults to `noopener`; an empty string omits it.
    pub link_rel: Option<String>,
    /// The host's icons. Without a resolver, icon names take no room and draw nothing.
    pub icons: Option<Arc<dyn IconResolver>>,
    /// See [`ParseOptions::config`].
    pub config: Option<Config>,
    /// The time Gantt charts treat as now, in milliseconds since the Unix epoch. Defaults to
    /// the current time.
    pub now: Option<f64>,
    /// The offset of local time from UTC in minutes, for the date arithmetic of Gantt charts,
    /// as JavaScript's `Date` would apply it. Defaults to 0 (UTC).
    pub timezone_offset_minutes: Option<i32>,
    /// How colors are written: Pele's `--pele-*` custom properties (the default), or concrete
    /// colors for every role.
    pub color_mode: ColorMode,
    /// Light or dark: picks the default palette when `palette` is `None`.
    pub theme: MermaidThemeMode,
    /// Zebra's semantic palette. With a palette, every role takes its colors from it.
    pub palette: Option<MermaidPalette>,
    /// Whether the SVG paints its own canvas. Defaults to transparent in CSS mode (as Pele
    /// draws no canvas) and to opaque in resolved mode.
    pub canvas_background: Option<CanvasBackground>,
    /// The text the `info` diagram shows. Defaults to `Pele v<PELE_VERSION>` for
    /// compatibility with the pinned upstream. Hosts can supply their own branding.
    pub info_text: Option<String>,
}

impl fmt::Debug for RenderOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderOptions")
            .field("measurer", &self.measurer.as_ref().map(|_| "custom"))
            .field("font_family", &self.font_family)
            .field("font_family_mono", &self.font_family_mono)
            .field("font_size", &self.font_size)
            .field("id_prefix", &self.id_prefix)
            .field("responsive", &self.responsive)
            .field("max_width", &self.max_width)
            .field("auto_direction", &self.auto_direction)
            .field("direction_breakpoint", &self.direction_breakpoint)
            .field("padding", &self.padding)
            .field("limit", &self.limit)
            .field("output_limit", &self.output_limit)
            .field("max_edges", &self.max_edges)
            .field("link_schemes", &self.link_schemes)
            .field("image_schemes", &self.image_schemes)
            .field("links", &self.links)
            .field("images", &self.images)
            .field("link_rel", &self.link_rel)
            .field("icons", &self.icons.as_ref().map(|_| "custom"))
            .field("config", &self.config)
            .field("now", &self.now)
            .field("timezone_offset_minutes", &self.timezone_offset_minutes)
            .field("color_mode", &self.color_mode)
            .field("theme", &self.theme)
            .field("palette", &self.palette)
            .field("canvas_background", &self.canvas_background)
            .field("info_text", &self.info_text)
            .finish()
    }
}

impl fmt::Debug for ParseOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseOptions")
            .field("limit", &self.limit)
            .field("max_edges", &self.max_edges)
            .field("config", &self.config)
            .finish()
    }
}

impl RenderOptions {
    /// The options that are read at parse time.
    pub fn parse_options(&self) -> ParseOptions {
        ParseOptions { limit: self.limit, max_edges: self.max_edges, config: self.config.clone() }
    }

    pub fn font_size_or_default(&self) -> f64 {
        self.font_size.unwrap_or(16.0)
    }

    pub fn padding_or_default(&self) -> f64 {
        self.padding.unwrap_or(8.0)
    }

    /// A canonical text of every option that can change the output, for hosts that cache
    /// rendered SVGs. Two option sets with the same identity render the same SVG for the same
    /// source. The full palette is part of it, so any change of any color changes it.
    ///
    /// A custom measurer or icon resolver cannot be described here: it appears only as
    /// `custom`, and a host that swaps one must add its own identity of it.
    pub fn render_identity(&self) -> String {
        let f = |v: Option<f64>| v.map_or("-".to_string(), crate::util::js::to_string);
        let b = |v: Option<bool>| v.map_or("-", |b| if b { "1" } else { "0" });
        let s = |v: &Option<String>| v.as_deref().map_or("-".to_string(), |s| format!("{:?}", s));
        let u = |v: Option<usize>| v.map_or("-".to_string(), |n| n.to_string());
        let list = |v: &Option<Vec<String>>| v.as_ref().map_or("-".to_string(), |l| format!("{:?}", l));
        format!(
            "v1|color={}|theme={}|palette={}|canvas={}|font={}|mono={}|size={}|id={}|responsive={}|maxw={}|auto={}|break={}|pad={}|limit={}|out={}|edges={}|links={}:{}|images={}:{}|rel={}|config={}|now={}|tz={}|info={}|measurer={}|icons={}",
            self.color_mode.as_str(),
            self.theme.as_str(),
            self.palette.as_ref().map_or("-".to_string(), MermaidPalette::to_json),
            self.canvas().as_str(),
            s(&self.font_family),
            s(&self.font_family_mono),
            f(self.font_size),
            s(&self.id_prefix),
            b(self.responsive),
            f(self.max_width),
            b(self.auto_direction),
            f(self.direction_breakpoint),
            f(self.padding),
            u(self.limit),
            u(self.output_limit),
            u(self.max_edges),
            b(self.links),
            list(&self.link_schemes),
            b(self.images),
            list(&self.image_schemes),
            s(&self.link_rel),
            self.config.as_ref().map_or("-".to_string(), |c| crate::config::ConfigValue::Object(c.clone()).to_json_string()),
            f(self.now),
            self.timezone_offset_minutes.map_or("-".to_string(), |m| m.to_string()),
            s(&self.info_text),
            if self.measurer.is_some() { "custom" } else { "metrics" },
            if self.icons.is_some() { "custom" } else { "-" },
        )
    }

    /// The canvas policy that applies: the explicit one, or the default of the color mode.
    pub fn canvas(&self) -> CanvasBackground {
        self.canvas_background.unwrap_or(match self.color_mode {
            ColorMode::Css => CanvasBackground::Transparent,
            ColorMode::Resolved => CanvasBackground::Opaque,
        })
    }
}

/// Limits set by the host. A diagram's own configuration cannot change them.
#[derive(Debug, Clone, Copy)]
pub struct ParseLimits {
    pub max_edges: usize,
}
