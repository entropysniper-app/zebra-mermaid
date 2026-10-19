//! A pure Rust Mermaid to SVG renderer, ported from [Pele](https://github.com/obsidianmd/pele).
//!
//! The library parses Mermaid text, lays it out and writes SVG without a JavaScript engine,
//! browser or external layout program. Output follows Pele 1.6.0 (commit `44da152b`), which
//! targets Mermaid 12.1.0 syntax.
//!
//! ```
//! use zebra_mermaid::{render, RenderOptions};
//!
//! let result = render("pie\n  \"Dogs\" : 386\n  \"Cats\" : 85\n", &RenderOptions::default()).unwrap();
//! assert!(result.svg.starts_with("<svg"));
//! assert_eq!(result.diagram_type.as_str(), "pie");
//! ```
//!
//! Text widths are estimated with [`MetricsMeasurer`], Pele's table of Arial advances, unless
//! the host gives a [`TextMeasurer`] of its own.

pub mod cache;
pub mod config;
pub mod detect;
pub mod diagrams;
mod engine;
pub mod error;
pub mod layout;
pub mod options;
pub mod preprocess;
pub mod svg;
pub mod text;
pub mod theme;
pub mod util;
pub mod zebra;

#[cfg(all(test, feature = "er"))]
#[path = "../tests/upstream/support.rs"]
mod upstream_support;

pub use cache::{CacheStats, RenderCache};
pub use config::{Config, ConfigMap, ConfigValue};
pub use detect::DiagramType;
pub use diagrams::{enabled as type_enabled, DiagramModel};
pub use engine::{detect_type, parse, render, render_parsed, supports, ParsedDiagram, RenderResult};
pub use error::{Error, ErrorCode, SourcePosition};
pub use options::{ParseOptions, RenderOptions, DEFAULT_LIMIT, DEFAULT_MAX_EDGES, DEFAULT_OUTPUT_LIMIT};
pub use text::{IconResolver, IconTable, MetricsMeasurer, TextMeasurer};
pub use theme::{CanvasBackground, ColorMode, MermaidPalette, MermaidThemeMode, PaletteError, ResolvedTheme, Role};
pub use util::url::LinkInfo;

/// The upstream this library follows.
pub const PELE_VERSION: &str = "1.6.0";
pub const PELE_COMMIT: &str = "44da152bbb79b91f03e68016f42499bb25bf6e82";
pub const MERMAID_SYNTAX_VERSION: &str = "12.1.0";
