//! The `zebra-mermaid` command line tool: renders Mermaid text to SVG with the zebra-mermaid
//! library. Standard output carries only the requested result; diagnostics go to standard
//! error.

// Failures carry a JSON detail; they are rare and never on a hot path.
#![allow(clippy::result_large_err)]

pub mod args;
mod files;

use std::ffi::OsString;
use std::io::{Read, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use zebra_mermaid::zebra::{self, ZebraRenderOptions};
use zebra_mermaid::{ConfigValue, Error, ErrorCode, MermaidPalette, RenderOptions};

use args::{Command, Format, Output, RenderArgs, USAGE};

pub const EXIT_OK: i32 = 0;
pub const EXIT_INTERNAL: i32 = 1;
pub const EXIT_USAGE: i32 = 2;
pub const EXIT_DIAGRAM: i32 = 3;
pub const EXIT_LIMIT: i32 = 4;
pub const EXIT_IO: i32 = 5;

/// Each render runs on a thread with this much stack, so that deep input meets the library's
/// explicit nesting limits long before the stack runs out.
const STACK: usize = 64 * 1024 * 1024;

/// A failure of one input: what kind, and what to say.
#[derive(Debug, Clone)]
pub struct Failure {
    pub exit: i32,
    pub code: String,
    pub message: String,
    pub detail: Value,
}

impl Failure {
    fn io(code: &str, message: impl Into<String>) -> Self {
        Failure { exit: EXIT_IO, code: code.into(), message: message.into(), detail: Value::Null }
    }

    fn usage(message: impl Into<String>) -> Self {
        Failure { exit: EXIT_USAGE, code: "invalid-options".into(), message: message.into(), detail: Value::Null }
    }

    fn from_error(e: &Error, source: &str) -> Self {
        let exit = match e.code {
            ErrorCode::Limit => EXIT_LIMIT,
            ErrorCode::InvalidOptions => EXIT_USAGE,
            _ => EXIT_DIAGRAM,
        };
        let position = e.source_position(source);
        let detail = json!({
            "diagramType": e.diagram_type.map(|t| t.as_str()),
            "mermaidType": e.mermaid_type.map(|t| t.as_str()),
            "line": e.line,
            "column": e.column,
            "snippet": e.snippet,
            "byteSpan": e.span.as_ref().map(|s| [s.start, s.end]),
            "source": position.map(|p| json!({ "line": p.line, "column": p.column, "columnUtf16": p.column_utf16, "byte": p.byte })),
        });
        Failure { exit, code: e.code.as_str().into(), message: e.message.clone(), detail }
    }

    fn to_json(&self, file: Option<&str>) -> Value {
        let mut error = json!({ "code": self.code, "message": self.message });
        if let (Some(o), Value::Object(d)) = (error.as_object_mut(), &self.detail) {
            for (k, v) in d {
                o.insert(k.clone(), v.clone());
            }
        }
        json!({ "ok": false, "file": file, "error": error })
    }
}

/// What a successful render produced.
#[derive(Debug, Clone)]
pub struct Rendered {
    pub svg: String,
    pub width: f64,
    pub height: f64,
    pub diagram_type: String,
    pub mermaid_type: String,
    pub links: Vec<(String, String, bool)>,
}

/// Reads an input as UTF-8 within the byte limit. `-` is standard input.
pub fn read_input(input: &str, max_bytes: usize) -> Result<String, Failure> {
    let mut bytes = Vec::new();
    let label = if input == "-" { "standard input".to_string() } else { input.to_string() };
    let read = if input == "-" {
        std::io::stdin().lock().take(max_bytes as u64 + 1).read_to_end(&mut bytes)
    } else {
        match std::fs::File::open(input) {
            Ok(f) => f.take(max_bytes as u64 + 1).read_to_end(&mut bytes),
            Err(e) => return Err(Failure::io("io", format!("cannot read {label}: {e}"))),
        }
    };
    if let Err(e) = read {
        return Err(Failure::io("io", format!("cannot read {label}: {e}")));
    }
    if bytes.len() > max_bytes {
        return Err(Failure {
            exit: EXIT_LIMIT,
            code: "limit".into(),
            message: format!("{label} is larger than {max_bytes} bytes (see --max-input-bytes)"),
            detail: Value::Null,
        });
    }
    String::from_utf8(bytes).map_err(|e| {
        let at = e.utf8_error().valid_up_to();
        Failure {
            exit: EXIT_IO,
            code: "invalid-utf8".into(),
            message: format!("{label} is not valid UTF-8 at byte {at}"),
            detail: json!({ "byte": at }),
        }
    })
}

fn json_to_config(v: &Value) -> ConfigValue {
    match v {
        Value::Null => ConfigValue::Null,
        Value::Bool(b) => ConfigValue::Bool(*b),
        Value::Number(n) => ConfigValue::Number(n.as_f64().unwrap_or(f64::NAN)),
        Value::String(s) => ConfigValue::String(s.clone()),
        Value::Array(a) => ConfigValue::Array(a.iter().map(json_to_config).collect()),
        Value::Object(o) => {
            let mut m = zebra_mermaid::ConfigMap::new();
            for (k, v) in o {
                m.insert(k.clone(), json_to_config(v));
            }
            ConfigValue::Object(m)
        }
    }
}

/// How to render, resolved from the arguments once for all inputs.
#[derive(Clone)]
pub enum Plan {
    Library(Box<RenderOptions>),
    Zebra(Box<ZebraRenderOptions>),
}

pub fn plan(r: &RenderArgs) -> Result<Plan, Failure> {
    let palette = match &r.palette {
        None => None,
        Some(path) => {
            let text =
                std::fs::read_to_string(path).map_err(|e| Failure::io("io", format!("cannot read palette {}: {e}", path.display())))?;
            Some(MermaidPalette::from_json(&text).map_err(|e| Failure::usage(format!("invalid palette {}: {e}", path.display())))?)
        }
    };
    if r.zebra {
        let options = ZebraRenderOptions {
            theme_mode: r.theme,
            palette,
            font_family: r.font_family.clone().unwrap_or_else(|| zebra::ZEBRA_FONT_FAMILY.to_string()),
            font_size: r.font_size.unwrap_or(16.0),
            canvas_background: r.canvas_background.unwrap_or_default(),
            ..Default::default()
        };
        return Ok(Plan::Zebra(Box::new(options)));
    }
    let config = match &r.config {
        None => None,
        Some(path) => {
            let text =
                std::fs::read_to_string(path).map_err(|e| Failure::io("io", format!("cannot read config {}: {e}", path.display())))?;
            let v: Value =
                serde_json::from_str(&text).map_err(|e| Failure::usage(format!("config {} is not JSON: {e}", path.display())))?;
            match json_to_config(&v) {
                ConfigValue::Object(m) => Some(m),
                _ => return Err(Failure::usage(format!("config {} must be a JSON object", path.display()))),
            }
        }
    };
    Ok(Plan::Library(Box::new(RenderOptions {
        font_family: r.font_family.clone(),
        font_family_mono: r.font_family_mono.clone(),
        font_size: r.font_size,
        id_prefix: r.id_prefix.clone(),
        responsive: r.responsive,
        max_width: r.max_width,
        auto_direction: r.auto_direction,
        direction_breakpoint: r.direction_breakpoint,
        padding: r.padding,
        limit: r.limit,
        output_limit: r.output_limit,
        max_edges: r.max_edges,
        link_schemes: r.link_schemes.clone(),
        image_schemes: r.image_schemes.clone(),
        links: r.links,
        images: r.images,
        link_rel: r.link_rel.clone(),
        config,
        now: r.now,
        timezone_offset_minutes: r.timezone_offset_minutes,
        color_mode: r.color_mode,
        theme: r.theme,
        palette,
        canvas_background: r.canvas_background,
        ..Default::default()
    })))
}

/// Renders on a thread with a large stack, turning a panic into an internal error.
pub fn render_source(source: String, plan: &Plan) -> Result<Rendered, Failure> {
    let plan = plan.clone();
    let worker = std::thread::Builder::new().stack_size(STACK).spawn(move || {
        catch_unwind(AssertUnwindSafe(|| match &plan {
            Plan::Library(options) => zebra_mermaid::render(&source, options)
                .map(|r| Rendered {
                    svg: r.svg,
                    width: r.width,
                    height: r.height,
                    diagram_type: r.diagram_type.as_str().into(),
                    mermaid_type: r.mermaid_type.as_str().into(),
                    links: r.links.into_iter().map(|l| (l.id, l.href, l.internal)).collect(),
                })
                .map_err(|e| Failure::from_error(&e, &source)),
            Plan::Zebra(options) => zebra::render_for_zebra(&source, options)
                .map(|r| {
                    let found =
                        zebra_mermaid::preprocess::preprocess(&source).ok().and_then(|p| zebra_mermaid::detect::detect_diagram(&p.text));
                    Rendered {
                        svg: r.svg,
                        width: r.width,
                        height: r.height,
                        diagram_type: found.map_or("unknown".into(), |t| t.diagram_type.as_str().into()),
                        mermaid_type: found.map_or("unknown".into(), |t| t.mermaid_type.as_str().into()),
                        links: Vec::new(),
                    }
                })
                .map_err(|e| {
                    let (exit, code) = match e.kind {
                        zebra::ZebraRenderErrorKind::InvalidOptions | zebra::ZebraRenderErrorKind::InvalidPalette => {
                            (EXIT_USAGE, "invalid-options")
                        }
                        zebra::ZebraRenderErrorKind::OutputTooLarge => (EXIT_LIMIT, "limit"),
                        zebra::ZebraRenderErrorKind::ParseFailure => (EXIT_DIAGRAM, "parse-failure"),
                    };
                    Failure { exit, code: code.into(), message: e.message, detail: Value::Null }
                }),
        }))
    });
    match worker {
        Err(e) => Err(Failure {
            exit: EXIT_INTERNAL,
            code: "internal".into(),
            message: format!("cannot start a render thread: {e}"),
            detail: Value::Null,
        }),
        Ok(handle) => match handle.join() {
            Ok(Ok(result)) => result,
            _ => Err(Failure {
                exit: EXIT_INTERNAL,
                code: "internal".into(),
                message: "the renderer failed unexpectedly (internal error)".into(),
                detail: Value::Null,
            }),
        },
    }
}

fn result_json(r: &Rendered, file: Option<&str>) -> Value {
    json!({
        "ok": true,
        "file": file,
        "type": r.diagram_type,
        "mermaidType": r.mermaid_type,
        "width": r.width,
        "height": r.height,
        "links": r.links.iter().map(|(id, href, internal)| json!({ "id": id, "href": href, "internal": internal })).collect::<Vec<_>>(),
        "svg": r.svg,
    })
}

fn report(failure: &Failure, file: Option<&str>, format: Format) {
    match format {
        Format::Json => println!("{}", failure.to_json(file)),
        Format::Text => {
            let name = file.map(|f| format!("{f}: ")).unwrap_or_default();
            let at = failure
                .detail
                .get("source")
                .and_then(|s| s.as_object())
                .map(|s| format!(" (line {}, column {})", s["line"], s["column"]))
                .unwrap_or_default();
            eprintln!("zebra-mermaid: {name}{} error{at}: {}", failure.code, failure.message);
        }
    }
}

fn run_render(input: &str, output: Option<&Path>, force: bool, format: Output, r: &RenderArgs) -> i32 {
    let file = (input != "-").then_some(input);
    let result = read_input(input, r.max_input_bytes).and_then(|source| {
        let plan = plan(r)?;
        render_source(source, &plan)
    });
    let rendered = match result {
        Ok(r) => r,
        Err(f) => {
            report(&f, file, if format == Output::Json { Format::Json } else { Format::Text });
            return f.exit;
        }
    };
    let body = match format {
        Output::Svg => rendered.svg.clone(),
        Output::Json => result_json(&rendered, file).to_string() + "\n",
    };
    match output {
        None => {
            let mut out = std::io::stdout().lock();
            if out.write_all(body.as_bytes()).and_then(|_| out.flush()).is_err() {
                eprintln!("zebra-mermaid: io error: cannot write standard output");
                return EXIT_IO;
            }
            EXIT_OK
        }
        Some(path) => match files::write_atomic(path, body.as_bytes(), force) {
            Ok(()) => EXIT_OK,
            Err(f) => {
                report(&f, file, Format::Text);
                f.exit
            }
        },
    }
}

fn run_check(inputs: &[String], format: Format, r: &RenderArgs) -> i32 {
    let plan = match plan(r) {
        Ok(p) => p,
        Err(f) => {
            report(&f, None, format);
            return f.exit;
        }
    };
    let mut worst = EXIT_OK;
    for input in inputs {
        let file = (input != "-").then_some(input.as_str());
        match read_input(input, r.max_input_bytes).and_then(|source| render_source(source, &plan)) {
            Ok(rendered) => match format {
                Format::Json => println!(
                    "{}",
                    json!({ "ok": true, "file": file, "type": rendered.diagram_type, "mermaidType": rendered.mermaid_type, "width": rendered.width, "height": rendered.height })
                ),
                Format::Text => eprintln!("zebra-mermaid: {}: ok ({})", file.unwrap_or("standard input"), rendered.diagram_type),
            },
            Err(f) => {
                report(&f, file, format);
                worst = worse(worst, f.exit);
            }
        }
    }
    worst
}

/// The more serious of two exit codes: I/O over limits over diagram errors.
fn worse(a: i32, b: i32) -> i32 {
    let rank = |c: i32| match c {
        EXIT_OK => 0,
        EXIT_DIAGRAM => 1,
        EXIT_LIMIT => 2,
        EXIT_USAGE => 3,
        EXIT_IO => 4,
        _ => 5,
    };
    if rank(b) > rank(a) {
        b
    } else {
        a
    }
}

fn run_batch(inputs: &[PathBuf], out_dir: &Path, jobs: usize, force: bool, format: Format, r: &RenderArgs) -> i32 {
    let jobs_list = match files::plan_batch(inputs, out_dir) {
        Ok(list) => list,
        Err(f) => {
            report(&f, None, format);
            return f.exit;
        }
    };
    let plan = match plan(r) {
        Ok(p) => p,
        Err(f) => {
            report(&f, None, format);
            return f.exit;
        }
    };
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<Option<Result<(), Failure>>>> = Mutex::new(vec![None; jobs_list.len()]);
    let max_bytes = r.max_input_bytes;
    let jobs_list = Arc::new(jobs_list);
    std::thread::scope(|scope| {
        for _ in 0..jobs.min(jobs_list.len().max(1)) {
            scope.spawn(|| loop {
                let k = next.fetch_add(1, Ordering::SeqCst);
                let Some(job) = jobs_list.get(k) else { break };
                let outcome = read_input(&job.input.to_string_lossy(), max_bytes).and_then(|source| render_source(source, &plan)).and_then(
                    |rendered| {
                        if let Some(parent) = job.output.parent() {
                            std::fs::create_dir_all(parent)
                                .map_err(|e| Failure::io("io", format!("cannot create {}: {e}", parent.display())))?;
                        }
                        files::write_atomic(&job.output, rendered.svg.as_bytes(), force)
                    },
                );
                results.lock().expect("results")[k] = Some(outcome);
            });
        }
    });
    let results = results.into_inner().expect("results");
    let mut worst = EXIT_OK;
    let mut ok = 0;
    for (job, outcome) in jobs_list.iter().zip(results) {
        let file = job.input.to_string_lossy().to_string();
        match outcome.expect("every job runs") {
            Ok(()) => {
                ok += 1;
                if format == Format::Json {
                    println!("{}", json!({ "ok": true, "file": file, "output": job.output.to_string_lossy() }));
                }
            }
            Err(f) => {
                report(&f, Some(&file), format);
                worst = worse(worst, f.exit);
            }
        }
    }
    let failed = jobs_list.len() - ok;
    match format {
        Format::Json => println!("{}", json!({ "summary": { "total": jobs_list.len(), "ok": ok, "failed": failed } })),
        Format::Text => eprintln!("zebra-mermaid: {} rendered, {} failed, {} total", ok, failed, jobs_list.len()),
    }
    worst
}

fn run_detect(input: &str, format: Format, max_bytes: usize) -> i32 {
    let file = (input != "-").then_some(input);
    let source = match read_input(input, max_bytes) {
        Ok(s) => s,
        Err(f) => {
            report(&f, file, format);
            return f.exit;
        }
    };
    match zebra_mermaid::detect_type(&source) {
        Ok(t) => {
            let name = t.map(|t| t.as_str());
            let supported = t.is_some_and(zebra_mermaid::type_enabled);
            match format {
                Format::Json => println!("{}", json!({ "ok": true, "file": file, "type": name, "supported": supported })),
                Format::Text => println!("{}", name.unwrap_or("none")),
            }
            if name.is_some() {
                EXIT_OK
            } else {
                EXIT_DIAGRAM
            }
        }
        Err(e) => {
            let f = Failure::from_error(&e, &source);
            report(&f, file, format);
            f.exit
        }
    }
}

/// Runs the tool with its arguments (without the program name) and returns the exit status.
pub fn run(args: Vec<OsString>) -> i32 {
    let command = match args::parse(args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("zebra-mermaid: {}", e.0);
            return EXIT_USAGE;
        }
    };
    match command {
        Command::Help => {
            print!("{USAGE}");
            EXIT_OK
        }
        Command::Version => {
            println!(
                "zebra-mermaid {} (Pele {} at {}, Mermaid syntax {})",
                env!("CARGO_PKG_VERSION"),
                zebra_mermaid::PELE_VERSION,
                &zebra_mermaid::PELE_COMMIT[..8],
                zebra_mermaid::MERMAID_SYNTAX_VERSION
            );
            EXIT_OK
        }
        Command::Render { input, output, force, format, render } => run_render(&input, output.as_deref(), force, format, &render),
        Command::Check { inputs, format, render } => run_check(&inputs, format, &render),
        Command::Batch { inputs, out_dir, jobs, force, format, render } => run_batch(&inputs, &out_dir, jobs, force, format, &render),
        Command::Detect { input, format, max_input_bytes } => run_detect(&input, format, max_input_bytes),
    }
}
