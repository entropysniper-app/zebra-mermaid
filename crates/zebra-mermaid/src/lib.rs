//! A pure Rust Mermaid to SVG renderer, ported from [Pele](https://github.com/obsidianmd/pele).
//!
//! The library parses Mermaid text, lays it out and writes SVG without a JavaScript engine,
//! browser or external layout program. Output follows Pele 1.7.0 (commit `8603bf3`), which
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

#[cfg(all(test, any(feature = "er", feature = "class", feature = "flowchart", feature = "sequence")))]
#[path = "../tests/upstream/support.rs"]
mod upstream_support;

#[cfg(all(test, any(feature = "class", feature = "sequence", feature = "agentflow")))]
#[path = "../tests/upstream/stress.rs"]
mod upstream_stress;

pub use cache::{CacheStats, RenderCache};
pub use config::{Config, ConfigMap, ConfigValue};
pub use detect::{DiagramType, MermaidType};
pub use diagrams::{enabled as type_enabled, DiagramModel};
pub use engine::{detect_type, parse, render, render_parsed, supports, ParsedDiagram, RenderResult};
/// Pele 1.7's parsed model contract, represented by an immutable Rust model envelope.
pub type ParsedDiagramModel = ParsedDiagram;
pub use error::{Error, ErrorCode, SourcePosition};
pub use options::{ParseOptions, RenderOptions, DEFAULT_LIMIT, DEFAULT_MAX_EDGES, DEFAULT_OUTPUT_LIMIT};
pub use text::{IconResolver, IconTable, MetricsMeasurer, TextMeasurer};
pub use theme::{CanvasBackground, ColorMode, MermaidPalette, MermaidThemeMode, PaletteError, ResolvedTheme, Role};
pub use util::url::LinkInfo;

/// The upstream this library follows.
pub const PELE_VERSION: &str = "1.7.0";
pub const PELE_COMMIT: &str = "8603bf3313add9143cd417015d4a6bd6c522dada";
pub const MERMAID_SYNTAX_VERSION: &str = "12.1.0";
