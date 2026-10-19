//! Command line parsing. Hand-written: the commands are few and the library stays free of a
//! parser dependency.

use std::ffi::OsString;
use std::path::PathBuf;

use zebra_mermaid::{CanvasBackground, ColorMode, MermaidThemeMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Output {
    Svg,
    Json,
}

/// Options shared by every command that renders.
#[derive(Debug, Clone)]
pub struct RenderArgs {
    pub theme: MermaidThemeMode,
    pub color_mode: ColorMode,
    pub palette: Option<PathBuf>,
    pub canvas_background: Option<CanvasBackground>,
    pub font_family: Option<String>,
    pub font_family_mono: Option<String>,
    pub font_size: Option<f64>,
    pub max_width: Option<f64>,
    pub auto_direction: Option<bool>,
    pub direction_breakpoint: Option<f64>,
    pub padding: Option<f64>,
    pub id_prefix: Option<String>,
    pub now: Option<f64>,
    pub timezone_offset_minutes: Option<i32>,
    pub limit: Option<usize>,
    pub output_limit: Option<usize>,
    pub max_edges: Option<usize>,
    pub links: Option<bool>,
    pub images: Option<bool>,
    pub link_schemes: Option<Vec<String>>,
    pub image_schemes: Option<Vec<String>>,
    pub link_rel: Option<String>,
    pub responsive: Option<bool>,
    pub config: Option<PathBuf>,
    pub zebra: bool,
    pub max_input_bytes: usize,
}

impl Default for RenderArgs {
    fn default() -> Self {
        RenderArgs {
            theme: MermaidThemeMode::Light,
            // Standalone SVG files are read by many programs without CSS custom properties.
            color_mode: ColorMode::Resolved,
            palette: None,
            canvas_background: None,
            font_family: None,
            font_family_mono: None,
            font_size: None,
            max_width: None,
            auto_direction: None,
            direction_breakpoint: None,
            padding: None,
            id_prefix: None,
            now: None,
            timezone_offset_minutes: None,
            limit: None,
            output_limit: None,
            max_edges: None,
            links: None,
            images: None,
            link_schemes: None,
            image_schemes: None,
            link_rel: None,
            responsive: None,
            config: None,
            zebra: false,
            max_input_bytes: 4 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    Render { input: String, output: Option<PathBuf>, force: bool, format: Output, render: RenderArgs },
    Check { inputs: Vec<String>, format: Format, render: RenderArgs },
    Batch { inputs: Vec<PathBuf>, out_dir: PathBuf, jobs: usize, force: bool, format: Format, render: RenderArgs },
    Detect { input: String, format: Format, max_input_bytes: usize },
    Help,
    Version,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageError(pub String);

pub const USAGE: &str = "\
zebra-mermaid: render Mermaid diagrams to SVG

USAGE:
    zebra-mermaid render <FILE|-> [-o <OUT>] [--force] [--format svg|json] [OPTIONS]
    zebra-mermaid check <FILE|->... [--format text|json] [OPTIONS]
    zebra-mermaid batch <DIR|FILE>... --out-dir <DIR> [--jobs <N>] [--force] [--format text|json] [OPTIONS]
    zebra-mermaid detect <FILE|-> [--format text|json]
    zebra-mermaid --help | --version

Input is UTF-8 text; '-' reads standard input. SVG goes to standard output unless -o is
given; diagnostics go to standard error. An existing output file is not replaced unless
--force is given; output is written to a temporary file next to it and moved into place.

COLOR OPTIONS:
    --theme light|dark                 Default palette when no --palette is given [light]
    --color-mode resolved|css          Concrete colors, or Pele's --pele-* CSS variables [resolved]
    --palette <FILE>                   Zebra palette JSON (camelCase MermaidPalette)
    --canvas-background opaque|transparent
                                       Paint the canvas or not [opaque for resolved, transparent for css]
    --zebra                            Zebra's native profile: resolved colors, no links or images,
                                       geometry independent of width

LAYOUT AND TEXT:
    --font-family <LIST>  --font-family-mono <LIST>  --font-size <PX>
    --max-width <PX>  --no-auto-direction  --direction-breakpoint <PX>  --padding <PX>
    --responsive | --fixed-size        Scale with the container, or keep the drawn size
    --id-prefix <ID>

DATA AND LIMITS:
    --config <FILE>                    Host configuration (JSON object); the diagram's own wins
    --now <RFC3339|MS>                 The time Gantt charts treat as now
    --timezone UTC|+HH:MM|-HH:MM       Fixed offset for Gantt dates [UTC]
    --limit <N>  --output-limit <N>  --max-edges <N>  --max-input-bytes <N>
    --no-links  --no-images  --link-schemes <a,b>  --image-schemes <a,b>  --link-rel <REL>

BATCH:
    --out-dir <DIR>  --jobs <N>        Inputs: .mmd and .mermaid files, directories searched recursively;
                                       relative paths are kept under --out-dir

EXIT STATUS:
    0 success   2 usage or invalid option   3 unsupported, syntax or semantic error
    4 resource limit   5 input/output error   1 internal error
";

fn parse_number(flag: &str, value: &str) -> Result<f64, UsageError> {
    let v: f64 = value.parse().map_err(|_| UsageError(format!("{flag} expects a number, got {value:?}")))?;
    if !v.is_finite() {
        return Err(UsageError(format!("{flag} expects a finite number")));
    }
    Ok(v)
}

fn parse_count(flag: &str, value: &str) -> Result<usize, UsageError> {
    value.parse().map_err(|_| UsageError(format!("{flag} expects a non-negative integer, got {value:?}")))
}

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// An offset such as `Z`, `UTC`, `+08:00`, `-0530`, in minutes.
pub fn parse_offset(text: &str) -> Option<i32> {
    if text.eq_ignore_ascii_case("utc") || text == "Z" || text == "z" {
        return Some(0);
    }
    let (sign, rest) = match text.as_bytes().first()? {
        b'+' => (1, &text[1..]),
        b'-' => (-1, &text[1..]),
        _ => return None,
    };
    let digits: String = rest.chars().filter(|c| *c != ':').collect();
    if digits.len() != 4 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let h: i32 = digits[..2].parse().ok()?;
    let m: i32 = digits[2..].parse().ok()?;
    if h > 23 || m > 59 {
        return None;
    }
    Some(sign * (h * 60 + m))
}

/// `2026-10-19T00:00:00Z`, with optional fraction and offset, or milliseconds since the epoch.
pub fn parse_time(text: &str) -> Option<f64> {
    if let Ok(ms) = text.parse::<f64>() {
        return ms.is_finite().then_some(ms);
    }
    let b = text.as_bytes();
    if b.len() < 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let num = |s: &str| -> Option<i64> { s.bytes().all(|c| c.is_ascii_digit()).then(|| s.parse().ok())? };
    let y = num(&text[0..4])?;
    let mo = num(&text[5..7])?;
    let d = num(&text[8..10])?;
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) {
        return None;
    }
    let mut ms = days_from_civil(y, mo, d) * 86_400_000;
    let mut rest = &text[10..];
    if rest.is_empty() {
        return Some(ms as f64);
    }
    if !(rest.starts_with('T') || rest.starts_with('t') || rest.starts_with(' ')) || rest.len() < 9 {
        return None;
    }
    let t = &rest[1..];
    let h = num(&t[0..2])?;
    let mi = num(&t[3..5])?;
    let s = num(&t[6..8])?;
    if t.as_bytes()[2] != b':' || t.as_bytes()[5] != b':' || h > 23 || mi > 59 || s > 60 {
        return None;
    }
    ms += (h * 3600 + mi * 60 + s) * 1000;
    rest = &t[8..];
    if let Some(frac) = rest.strip_prefix('.') {
        let end = frac.find(|c: char| !c.is_ascii_digit()).unwrap_or(frac.len());
        if end == 0 {
            return None;
        }
        let digits = &frac[..end.min(3)];
        let scale = [100, 10, 1];
        for (i, c) in digits.bytes().enumerate() {
            ms += (c - b'0') as i64 * scale[i];
        }
        rest = &frac[end..];
    }
    if rest.is_empty() {
        return Some(ms as f64);
    }
    let offset = parse_offset(rest)?;
    Some((ms - offset as i64 * 60_000) as f64)
}

fn list(value: &str) -> Vec<String> {
    value.split(',').map(|s| s.trim().to_ascii_lowercase()).filter(|s| !s.is_empty()).collect()
}

pub fn parse(args: Vec<OsString>) -> Result<Command, UsageError> {
    let mut args: Vec<String> = args
        .into_iter()
        .map(|a| a.into_string().map_err(|a| UsageError(format!("argument is not valid UTF-8: {a:?}"))))
        .collect::<Result<_, _>>()?;
    if args.is_empty() {
        return Ok(Command::Help);
    }
    let command = args.remove(0);
    match command.as_str() {
        "-h" | "--help" | "help" => return Ok(Command::Help),
        "-V" | "--version" | "version" => return Ok(Command::Version),
        "render" | "check" | "batch" | "detect" => {}
        other => return Err(UsageError(format!("unknown command {other:?}; try --help"))),
    }
    let mut render = RenderArgs::default();
    let mut positional: Vec<String> = Vec::new();
    let mut output: Option<PathBuf> = None;
    let mut out_dir: Option<PathBuf> = None;
    let mut force = false;
    let mut format: Option<String> = None;
    let mut jobs: usize = std::thread::available_parallelism().map_or(1, |n| n.get());
    let mut explicit_color_mode = false;
    let mut i = 0;
    let mut only_positional = false;
    while i < args.len() {
        let arg = args[i].clone();
        i += 1;
        if only_positional || arg == "-" || !arg.starts_with('-') {
            positional.push(arg);
            continue;
        }
        if arg == "--" {
            only_positional = true;
            continue;
        }
        let (flag, inline) = match arg.split_once('=') {
            Some((f, v)) if f.starts_with("--") => (f.to_string(), Some(v.to_string())),
            _ => (arg.clone(), None),
        };
        let mut value = |name: &str| -> Result<String, UsageError> {
            if let Some(v) = &inline {
                return Ok(v.clone());
            }
            if i < args.len() {
                i += 1;
                Ok(args[i - 1].clone())
            } else {
                Err(UsageError(format!("{name} expects a value")))
            }
        };
        match flag.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
            "-o" | "--output" => output = Some(PathBuf::from(value(&flag)?)),
            "--out-dir" => out_dir = Some(PathBuf::from(value(&flag)?)),
            "--force" | "-f" => force = true,
            "--format" => format = Some(value(&flag)?),
            "--jobs" | "-j" => {
                jobs = parse_count(&flag, &value(&flag)?)?;
                if jobs == 0 {
                    return Err(UsageError("--jobs must be at least 1".into()));
                }
            }
            "--theme" => {
                render.theme = match value(&flag)?.as_str() {
                    "light" => MermaidThemeMode::Light,
                    "dark" => MermaidThemeMode::Dark,
                    // The plan once used `--theme resolved`; the color mode is a separate option now.
                    "resolved" | "css" => {
                        return Err(UsageError("--theme takes light or dark; use --color-mode resolved|css for the color output".into()))
                    }
                    other => return Err(UsageError(format!("--theme takes light or dark, got {other:?}"))),
                }
            }
            "--color-mode" => {
                explicit_color_mode = true;
                render.color_mode = match value(&flag)?.as_str() {
                    "resolved" => ColorMode::Resolved,
                    "css" => ColorMode::Css,
                    other => return Err(UsageError(format!("--color-mode takes resolved or css, got {other:?}"))),
                }
            }
            "--palette" => render.palette = Some(PathBuf::from(value(&flag)?)),
            "--canvas-background" => {
                render.canvas_background = Some(match value(&flag)?.as_str() {
                    "opaque" => CanvasBackground::Opaque,
                    "transparent" => CanvasBackground::Transparent,
                    other => return Err(UsageError(format!("--canvas-background takes opaque or transparent, got {other:?}"))),
                })
            }
            "--zebra" => render.zebra = true,
            "--font-family" => render.font_family = Some(value(&flag)?),
            "--font-family-mono" => render.font_family_mono = Some(value(&flag)?),
            "--font-size" => {
                let v = parse_number(&flag, &value(&flag)?)?;
                if v <= 0.0 {
                    return Err(UsageError("--font-size must be positive".into()));
                }
                render.font_size = Some(v)
            }
            "--max-width" => render.max_width = Some(parse_number(&flag, &value(&flag)?)?),
            "--no-auto-direction" => render.auto_direction = Some(false),
            "--direction-breakpoint" => render.direction_breakpoint = Some(parse_number(&flag, &value(&flag)?)?),
            "--padding" => render.padding = Some(parse_number(&flag, &value(&flag)?)?),
            "--id-prefix" => render.id_prefix = Some(value(&flag)?),
            "--responsive" => render.responsive = Some(true),
            "--fixed-size" => render.responsive = Some(false),
            "--config" => render.config = Some(PathBuf::from(value(&flag)?)),
            "--now" => {
                let v = value(&flag)?;
                render.now =
                    Some(parse_time(&v).ok_or_else(|| UsageError(format!("--now expects RFC 3339 time or milliseconds, got {v:?}")))?);
            }
            "--timezone" => {
                let v = value(&flag)?;
                render.timezone_offset_minutes = Some(
                    parse_offset(&v)
                        .ok_or_else(|| UsageError(format!("--timezone expects UTC or a fixed offset such as +08:00, got {v:?}")))?,
                );
            }
            "--limit" => render.limit = Some(parse_count(&flag, &value(&flag)?)?),
            "--output-limit" => render.output_limit = Some(parse_count(&flag, &value(&flag)?)?),
            "--max-edges" => render.max_edges = Some(parse_count(&flag, &value(&flag)?)?),
            "--max-input-bytes" => render.max_input_bytes = parse_count(&flag, &value(&flag)?)?,
            "--no-links" => render.links = Some(false),
            "--no-images" => render.images = Some(false),
            "--link-schemes" => render.link_schemes = Some(list(&value(&flag)?)),
            "--image-schemes" => render.image_schemes = Some(list(&value(&flag)?)),
            "--link-rel" => render.link_rel = Some(value(&flag)?),
            other => return Err(UsageError(format!("unknown option {other:?}; try --help"))),
        }
    }
    if render.zebra && explicit_color_mode && render.color_mode == ColorMode::Css {
        return Err(UsageError("--zebra always writes resolved colors; drop --color-mode css".into()));
    }
    let text_format = |f: &Option<String>| -> Result<Format, UsageError> {
        match f.as_deref() {
            None | Some("text") => Ok(Format::Text),
            Some("json") => Ok(Format::Json),
            Some(other) => Err(UsageError(format!("--format takes text or json, got {other:?}"))),
        }
    };
    match command.as_str() {
        "render" => {
            if positional.len() != 1 {
                return Err(UsageError("render takes one input (a file, or - for standard input)".into()));
            }
            let format = match format.as_deref() {
                None | Some("svg") => Output::Svg,
                Some("json") => Output::Json,
                Some(other) => return Err(UsageError(format!("--format takes svg or json for render, got {other:?}"))),
            };
            Ok(Command::Render { input: positional.remove(0), output, force, format, render })
        }
        "check" => {
            if positional.is_empty() {
                return Err(UsageError("check takes at least one input".into()));
            }
            Ok(Command::Check { inputs: positional, format: text_format(&format)?, render })
        }
        "batch" => {
            if positional.is_empty() {
                return Err(UsageError("batch takes at least one input directory or file".into()));
            }
            if positional.iter().any(|p| p == "-") {
                return Err(UsageError("batch does not read standard input".into()));
            }
            let out_dir = out_dir.ok_or_else(|| UsageError("batch needs --out-dir".into()))?;
            Ok(Command::Batch {
                inputs: positional.into_iter().map(PathBuf::from).collect(),
                out_dir,
                jobs,
                force,
                format: text_format(&format)?,
                render,
            })
        }
        "detect" => {
            if positional.len() != 1 {
                return Err(UsageError("detect takes one input".into()));
            }
            Ok(Command::Detect { input: positional.remove(0), format: text_format(&format)?, max_input_bytes: render.max_input_bytes })
        }
        _ => unreachable!("checked above"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(args: &[&str]) -> Result<Command, UsageError> {
        parse(args.iter().map(OsString::from).collect())
    }

    #[test]
    fn times_and_offsets() {
        assert_eq!(parse_time("2026-10-19T00:00:00Z"), Some(1_792_368_000_000.0));
        assert_eq!(parse_time("2026-10-19T08:00:00+08:00"), Some(1_792_368_000_000.0));
        assert_eq!(parse_time("2026-10-19"), Some(1_792_368_000_000.0));
        assert_eq!(parse_time("1792368000000"), Some(1_792_368_000_000.0));
        assert_eq!(parse_time("2026-10-19T00:00:00.5Z"), Some(1_792_368_000_500.0));
        assert_eq!(parse_time("2026-13-01"), None);
        assert_eq!(parse_offset("-05:30"), Some(-330));
        assert_eq!(parse_offset("UTC"), Some(0));
        assert_eq!(parse_offset("+2400"), None);
    }

    #[test]
    fn commands() {
        assert!(matches!(p(&["render", "a.mmd", "-o", "a.svg", "--theme", "dark"]), Ok(Command::Render { .. })));
        assert!(p(&["render", "--theme", "resolved", "a.mmd"]).unwrap_err().0.contains("--color-mode"));
        assert!(p(&["render"]).is_err());
        assert!(p(&["batch", "dir"]).unwrap_err().0.contains("--out-dir"));
        assert!(matches!(p(&["check", "-", "--format=json"]), Ok(Command::Check { format: Format::Json, .. })));
        assert!(p(&["frobnicate"]).is_err());
        assert!(p(&["render", "a", "--font-size", "x"]).is_err());
        assert!(p(&["render", "a", "--zebra", "--color-mode", "css"]).is_err());
    }
}
