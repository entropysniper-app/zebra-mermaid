//! Parsing and rendering from text to SVG. Ported from Pele `src/core.ts`.

use crate::config::{Config, ConfigMap};
use crate::detect::{detect, DiagramType};
use crate::diagrams::{self, DiagramModel, ParseInput, RenderCtx};
use crate::error::{Error, ErrorCode};
use crate::options::{ParseLimits, ParseOptions, RenderOptions, DEFAULT_LIMIT, DEFAULT_MAX_EDGES, DEFAULT_OUTPUT_LIMIT};
use crate::preprocess::{encode_entities, preprocess};
use crate::text::entities::decode_entities;
use crate::text::{Icons, MetricsMeasurer, TextMeasurer};
use crate::theme::Theming;
use crate::util::js;
use crate::util::url::LinkInfo;

/// A parsed diagram: its type, the configuration that applies to it, and its model. It can be
/// rendered any number of times, at different widths, without being parsed again.
#[derive(Debug, Clone)]
pub struct ParsedDiagram {
    diagram_type: DiagramType,
    config: Config,
    title: Option<String>,
    model: DiagramModel,
}

impl ParsedDiagram {
    pub fn diagram_type(&self) -> DiagramType {
        self.diagram_type
    }

    /// The configuration from the host, front matter and directives, merged.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// The title from front matter, if any. A diagram's own title statement is in its model.
    pub fn front_matter_title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn model(&self) -> &DiagramModel {
        &self.model
    }
}

/// The result of a render.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderResult {
    pub svg: String,
    pub width: f64,
    pub height: f64,
    pub diagram_type: DiagramType,
    /// Links drawn in the SVG, with the id of what each is on.
    pub links: Vec<LinkInfo>,
}

/// The type of a diagram, or `None` when no type is recognized. Fails only when the front
/// matter is not valid YAML.
pub fn detect_type(source: &str) -> Result<Option<DiagramType>, Error> {
    Ok(detect(&preprocess(source)?.text))
}

/// Whether the text names a diagram type that this build renders. This does not check that
/// the rest of the text is valid.
pub fn supports(source: &str) -> bool {
    matches!(detect_type(source), Ok(Some(t)) if diagrams::enabled(t))
}

/// Parses diagram text.
pub fn parse(source: &str, options: &ParseOptions) -> Result<ParsedDiagram, Error> {
    let limit = options.limit.unwrap_or(DEFAULT_LIMIT);
    if js::len16(source) > limit {
        return Err(Error::limit(format!("Diagram source is longer than the limit of {} characters.", limit)));
    }
    let pre = preprocess(source)?;
    let Some(diagram_type) = detect(&pre.text) else {
        return Err(Error::new("No diagram type detected for the given text.", ErrorCode::UnsupportedDiagram));
    };
    if !diagrams::enabled(diagram_type) {
        return Err(diagrams::unsupported(diagram_type));
    }
    // `{ ...extra, ...pre.config }`: a shallow merge in which the diagram's own keys win.
    let config = match &options.config {
        Some(extra) => {
            let mut merged: ConfigMap = extra.clone();
            for (k, v) in pre.config.iter() {
                merged.insert(k, v.clone());
            }
            merged
        }
        None => pre.config.clone(),
    };
    let (text, mut map) = if diagram_type.keeps_comments() {
        (pre.with_comments.as_str(), pre.with_comments_map.clone())
    } else {
        (pre.text.as_str(), pre.text_map.clone())
    };
    let (encoded, entity_map) = encode_entities(text);
    map.extend(entity_map);
    let mut source_text = encoded;
    source_text.push('\n');
    let input = ParseInput {
        source: &source_text,
        config: &config,
        title: pre.title.as_deref(),
        line_offset: pre.line_offset,
        limits: ParseLimits { max_edges: options.max_edges.unwrap_or(DEFAULT_MAX_EDGES) },
    };
    let model = diagrams::parse(diagram_type, &input).map_err(|mut e| {
        if let Some(offset) = e.offset {
            let at = map.to_source(offset).min(source.len());
            e.span = Some(at..at);
        }
        e
    })?;
    Ok(ParsedDiagram { diagram_type, config, title: pre.title, model })
}

/// Parses and renders diagram text.
pub fn render(source: &str, options: &RenderOptions) -> Result<RenderResult, Error> {
    let parsed = parse(source, &options.parse_options())?;
    render_parsed(&parsed, options)
}

/// Renders a parsed diagram. The parse-time options (`limit`, `max_edges`, `config`) are not
/// read again.
pub fn render_parsed(parsed: &ParsedDiagram, options: &RenderOptions) -> Result<RenderResult, Error> {
    let diagram_type = parsed.diagram_type;
    if let Some(palette) = &options.palette {
        palette.validate().map_err(|e| Error::new(format!("Invalid Mermaid palette: {e}."), ErrorCode::InvalidOptions))?;
    }
    let theming = Theming::new(options.color_mode, options.theme, options.palette.as_ref(), options.canvas());
    let fixed =
        parsed.config.section(diagram_type.config_section()).and_then(|s| s.get("useMaxWidth")).is_some_and(|v| v.as_bool() == Some(false));
    let responsive = options.responsive.unwrap_or(!fixed);
    let icons = Icons::new(options.icons.clone());
    let default_measurer = MetricsMeasurer;
    let measurer: &dyn TextMeasurer = match &options.measurer {
        Some(m) => m.as_ref(),
        None => &default_measurer,
    };
    let ctx = RenderCtx::new(options, responsive, measurer, &icons, &theming);
    let mut rendered = diagrams::render(&parsed.model, &parsed.config, &ctx)?;
    let output_limit = options.output_limit.unwrap_or(DEFAULT_OUTPUT_LIMIT);
    if js::len16(&rendered.svg) > output_limit {
        return Err(
            Error::limit(format!("Diagram output is longer than the limit of {} characters.", output_limit)).with_type(diagram_type)
        );
    }
    for link in &mut rendered.links {
        link.id = decode_entities(&link.id);
    }
    Ok(RenderResult { svg: rendered.svg, width: rendered.width, height: rendered.height, diagram_type, links: rendered.links })
}
