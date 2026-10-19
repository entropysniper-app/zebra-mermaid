//! Color themes: Zebra's semantic palette, light and dark defaults, and the roles diagrams
//! paint with. See `docs/ZEBRA-THEME-COMPATIBILITY.md`.

pub mod color;
pub mod palette;
pub mod resolved;

pub use palette::{CanvasBackground, ColorMode, MermaidPalette, MermaidThemeMode, PaletteError};
pub use resolved::{ResolvedTheme, Role, Theming};
