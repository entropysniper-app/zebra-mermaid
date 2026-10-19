//! The way Zebra calls its Mermaid renderer today, on top of this library.
//!
//! Zebra's native bridge (C ABI version 2, `zebra_mermaid_render_svg`) takes the source, a
//! dark-theme flag, an optional camelCase palette JSON, the font size, a target width and a
//! canvas policy, and returns an SVG with its width and height. This module keeps that request
//! shape and its error kinds so that an ABI adapter can sit on it without unsafe code here.
//!
//! The output follows Zebra's constraints: concrete colors (no CSS custom properties), no
//! links or images (Zebra's SVG validator refuses `<a>` and `<image>`), and geometry that does
//! not depend on the target width. Zebra's native service declares
//! `renderingDependsOnTargetWidth = false` and caches SVGs without the width in their identity;
//! the width is validated here and otherwise ignored, so that contract holds.

use std::fmt;

use crate::options::RenderOptions;
use crate::theme::{CanvasBackground, ColorMode, MermaidPalette, MermaidThemeMode};
use crate::{render, ErrorCode};

/// Zebra's default UI font stack.
pub const ZEBRA_FONT_FAMILY: &str = "-apple-system, BlinkMacSystemFont, \"SF Pro Text\", sans-serif";
/// The ABI's limits.
pub const ABI_VERSION: u32 = 2;
pub const MAX_SOURCE_UTF8_BYTES: usize = 512 * 1024;
pub const MAX_PALETTE_UTF8_BYTES: usize = 16 * 1024;
pub const MAX_OUTPUT_UTF8_BYTES: usize = 4 * 1024 * 1024;
/// Zebra's host refuses longer sources before they reach the renderer
/// (`NativeMermaidRendererService.Configuration.maximumSourceUTF16Length`). The adapter uses it
/// as the source limit, instead of Pele's 50,000, so that diagrams Zebra renders today are not
/// refused by the new engine.
pub const MAX_SOURCE_UTF16: usize = 262_144;

#[derive(Debug, Clone, PartialEq)]
pub struct ZebraRenderOptions {
    pub theme_mode: MermaidThemeMode,
    pub palette: Option<MermaidPalette>,
    pub font_family: String,
    pub font_size: f64,
    pub canvas_background: CanvasBackground,
    pub maximum_output_utf8_bytes: usize,
}

impl Default for ZebraRenderOptions {
    fn default() -> Self {
        ZebraRenderOptions {
            theme_mode: MermaidThemeMode::Light,
            palette: None,
            font_family: ZEBRA_FONT_FAMILY.into(),
            font_size: 16.0,
            canvas_background: CanvasBackground::Opaque,
            maximum_output_utf8_bytes: MAX_OUTPUT_UTF8_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZebraRendered {
    pub svg: String,
    pub width: f64,
    pub height: f64,
}

/// The error kinds of Zebra's renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZebraRenderErrorKind {
    InvalidOptions,
    InvalidPalette,
    ParseFailure,
    OutputTooLarge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZebraRenderError {
    pub kind: ZebraRenderErrorKind,
    pub message: String,
}

impl fmt::Display for ZebraRenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ZebraRenderError {}

fn error(kind: ZebraRenderErrorKind, message: impl Into<String>) -> ZebraRenderError {
    ZebraRenderError { kind, message: message.into() }
}

/// The library options that a Zebra request maps to. Nothing in them depends on a width.
pub fn render_options(options: &ZebraRenderOptions) -> RenderOptions {
    RenderOptions {
        color_mode: ColorMode::Resolved,
        theme: options.theme_mode,
        palette: options.palette.clone(),
        canvas_background: Some(options.canvas_background),
        font_family: Some(options.font_family.clone()),
        font_size: Some(options.font_size),
        links: Some(false),
        images: Some(false),
        limit: Some(MAX_SOURCE_UTF16),
        // The byte limit below is the one Zebra applies.
        output_limit: Some(usize::MAX),
        max_width: None,
        ..Default::default()
    }
}

/// Renders a diagram the way Zebra's renderer is called.
pub fn render_for_zebra(source: &str, options: &ZebraRenderOptions) -> Result<ZebraRendered, ZebraRenderError> {
    if !options.font_size.is_finite()
        || !(1.0..=256.0).contains(&options.font_size)
        || options.font_family.is_empty()
        || options.maximum_output_utf8_bytes == 0
    {
        return Err(error(ZebraRenderErrorKind::InvalidOptions, "Invalid Mermaid render options."));
    }
    if let Some(palette) = &options.palette {
        if !palette.is_valid() {
            return Err(error(ZebraRenderErrorKind::InvalidPalette, "Invalid Mermaid palette."));
        }
    }
    let rendered = render(source, &render_options(options)).map_err(|e| match e.code {
        ErrorCode::InvalidOptions => error(ZebraRenderErrorKind::InvalidOptions, e.message),
        _ => error(ZebraRenderErrorKind::ParseFailure, e.message),
    })?;
    if rendered.svg.len() > options.maximum_output_utf8_bytes {
        return Err(error(ZebraRenderErrorKind::OutputTooLarge, "Rendered diagram exceeds the output limit."));
    }
    Ok(ZebraRendered { svg: rendered.svg, width: rendered.width, height: rendered.height })
}

/// The statuses of the C ABI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ZebraStatus {
    Ok = 0,
    InvalidArgument = 1,
    InvalidUtf8 = 2,
    ParseFailure = 3,
    OutputTooLarge = 4,
    InternalPanic = 255,
}

/// `zebra_mermaid_render_svg` without the pointers: the same checks, in the same order, with
/// the same messages. `palette_json` of `None` (or empty) means no palette. `target_width` is
/// checked and then not used: geometry does not depend on it.
pub fn render_abi_v2(
    source: &[u8],
    dark_theme: bool,
    palette_json: Option<&[u8]>,
    font_size: f64,
    target_width: f64,
    canvas_background: u32,
) -> Result<ZebraRendered, (ZebraStatus, String)> {
    if !font_size.is_finite()
        || !(1.0..=256.0).contains(&font_size)
        || !target_width.is_finite()
        || !(1.0..=100_000.0).contains(&target_width)
    {
        return Err((ZebraStatus::InvalidArgument, "Invalid Mermaid layout metrics.".into()));
    }
    if source.is_empty() || source.len() > MAX_SOURCE_UTF8_BYTES {
        return Err((ZebraStatus::InvalidArgument, "Invalid Mermaid source.".into()));
    }
    let source = std::str::from_utf8(source).map_err(|_| (ZebraStatus::InvalidUtf8, "Invalid Mermaid source.".to_string()))?;
    let palette = match palette_json.filter(|p| !p.is_empty()) {
        None => None,
        Some(bytes) => {
            if bytes.len() > MAX_PALETTE_UTF8_BYTES {
                return Err((ZebraStatus::InvalidArgument, "Invalid Mermaid palette.".into()));
            }
            let text = std::str::from_utf8(bytes).map_err(|_| (ZebraStatus::InvalidUtf8, "Invalid Mermaid palette.".to_string()))?;
            Some(MermaidPalette::decode_json(text).map_err(|_| (ZebraStatus::InvalidArgument, "Invalid Mermaid palette.".to_string()))?)
        }
    };
    let canvas_background = match canvas_background {
        0 => CanvasBackground::Opaque,
        1 => CanvasBackground::Transparent,
        _ => return Err((ZebraStatus::InvalidArgument, "Invalid Mermaid canvas background policy.".into())),
    };
    let options = ZebraRenderOptions {
        theme_mode: if dark_theme { MermaidThemeMode::Dark } else { MermaidThemeMode::Light },
        palette,
        font_size,
        canvas_background,
        ..Default::default()
    };
    render_for_zebra(source, &options).map_err(|e| {
        let status = match e.kind {
            ZebraRenderErrorKind::ParseFailure => ZebraStatus::ParseFailure,
            ZebraRenderErrorKind::OutputTooLarge => ZebraStatus::OutputTooLarge,
            ZebraRenderErrorKind::InvalidOptions | ZebraRenderErrorKind::InvalidPalette => ZebraStatus::InvalidArgument,
        };
        // The ABI copies at most 512 bytes of a message.
        let mut message = e.message;
        if message.len() > 512 {
            let mut cut = 512;
            while !message.is_char_boundary(cut) {
                cut -= 1;
            }
            message.truncate(cut);
        }
        (status, message)
    })
}
