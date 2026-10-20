//! Render-only label storage. A single span lives inline; rich labels retain their owned
//! vectors. Public Label ownership and the shared layout algorithm remain unchanged.
use super::{Label, Span};
use std::ops::Index;

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct RenderLabel {
    pub width: f64,
    pub height: f64,
    pub size: f64,
    pub line_height: f64,
    content: Content,
}

#[derive(Debug, Clone, PartialEq, Default)]
enum Content {
    #[default]
    Empty,
    Single {
        span: Span,
        line_width: f64,
    },
    Rich {
        lines: Vec<Vec<Span>>,
        widths: Vec<f64>,
    },
}

#[derive(Clone, Copy)]
pub(crate) enum Lines<'a> {
    Empty,
    Single(&'a Span),
    Rich(&'a [Vec<Span>]),
}

impl<'a> Lines<'a> {
    pub fn len(self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Single(_) => 1,
            Self::Rich(lines) => lines.len(),
        }
    }
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
    pub fn get(self, index: usize) -> Option<&'a [Span]> {
        match self {
            Self::Empty => None,
            Self::Single(span) => (index == 0).then(|| std::slice::from_ref(span)),
            Self::Rich(lines) => lines.get(index).map(Vec::as_slice),
        }
    }
    pub fn iter(self) -> LineIter<'a> {
        LineIter { view: self, from: 0, to: self.len() }
    }
}

pub(crate) struct LineIter<'a> {
    view: Lines<'a>,
    from: usize,
    to: usize,
}
impl<'a> Iterator for LineIter<'a> {
    type Item = &'a [Span];
    fn next(&mut self) -> Option<Self::Item> {
        if self.from == self.to {
            None
        } else {
            let i = self.from;
            self.from += 1;
            self.view.get(i)
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.to - self.from;
        (n, Some(n))
    }
}
impl DoubleEndedIterator for LineIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.from == self.to {
            None
        } else {
            self.to -= 1;
            self.view.get(self.to)
        }
    }
}
impl ExactSizeIterator for LineIter<'_> {}
impl<'a> IntoIterator for Lines<'a> {
    type Item = &'a [Span];
    type IntoIter = LineIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a> IntoIterator for &Lines<'a> {
    type Item = &'a [Span];
    type IntoIter = LineIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Index<usize> for Lines<'_> {
    type Output = [Span];
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("label line index")
    }
}

#[derive(Clone, Copy)]
pub(crate) struct LabelView<'a> {
    pub lines: Lines<'a>,
    pub widths: &'a [f64],
    pub width: f64,
    pub height: f64,
    pub size: f64,
    pub line_height: f64,
}

pub(crate) trait LabelRead {
    fn view(&self) -> LabelView<'_>;
}

impl LabelRead for Label {
    fn view(&self) -> LabelView<'_> {
        LabelView {
            lines: Lines::Rich(&self.lines),
            widths: &self.widths,
            width: self.width,
            height: self.height,
            size: self.size,
            line_height: self.line_height,
        }
    }
}

impl LabelRead for RenderLabel {
    fn view(&self) -> LabelView<'_> {
        LabelView {
            lines: self.lines(),
            widths: self.widths(),
            width: self.width,
            height: self.height,
            size: self.size,
            line_height: self.line_height,
        }
    }
}

impl RenderLabel {
    pub fn empty() -> Self {
        Self::default()
    }
    pub fn single(span: Span, size: f64, line_height: f64) -> Self {
        let width = span.width;
        Self { width, height: line_height, size, line_height, content: Content::Single { span, line_width: width } }
    }
    pub fn from_owned(label: Label) -> Self {
        Self {
            width: label.width,
            height: label.height,
            size: label.size,
            line_height: label.line_height,
            content: Content::Rich { lines: label.lines, widths: label.widths },
        }
    }
    pub fn from_lines(lines: Vec<Vec<Span>>, widths: Vec<f64>, width: f64, height: f64, size: f64, line_height: f64) -> Self {
        Self { width, height, size, line_height, content: Content::Rich { lines, widths } }
    }
    pub fn keep_first_line(&mut self) {
        if let Content::Rich { lines, widths } = &mut self.content {
            if lines.len() > 1 {
                lines.truncate(1);
                widths.truncate(1);
                self.width = widths[0];
                self.height = self.line_height;
            }
        }
    }
    pub fn restyle(&mut self, bits: u8, m: &super::Measure) {
        if bits == 0 || self.lines().is_empty() {
            return;
        }
        match &mut self.content {
            Content::Empty => {}
            Content::Single { span, line_width } => {
                span.style |= bits;
                if span.icon.is_none() {
                    span.width = m.width(&span.text, self.size, span.style);
                }
                *line_width = 0.0 + span.width;
                self.width = crate::util::js::max(0.0, *line_width);
            }
            Content::Rich { lines, widths } => {
                let mut width = 0.0;
                let mut changed = Vec::new();
                for line in lines {
                    let mut total = 0.0;
                    for span in line {
                        span.style |= bits;
                        if span.icon.is_none() {
                            span.width = m.width(&span.text, self.size, span.style);
                        }
                        total += span.width;
                    }
                    changed.push(total);
                    width = crate::util::js::max(width, total);
                }
                *widths = changed;
                self.width = width;
            }
        }
    }
    pub fn lines(&self) -> Lines<'_> {
        match &self.content {
            Content::Empty => Lines::Empty,
            Content::Single { span, .. } => Lines::Single(span),
            Content::Rich { lines, .. } => Lines::Rich(lines),
        }
    }
    pub fn widths(&self) -> &[f64] {
        match &self.content {
            Content::Empty => &[],
            Content::Single { line_width, .. } => std::slice::from_ref(line_width),
            Content::Rich { widths, .. } => widths,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::{layout_label, layout_render_label, Icons, Measure, MetricsMeasurer, TextMeasurer};
    use serde_json::{json, Value};
    use std::cell::RefCell;
    use std::sync::Arc;

    fn fixture() -> Value {
        let data: Value = serde_json::from_reader(flate2::read::GzDecoder::new(
            &include_bytes!("../../../../tests/fixtures/unit/render-label-reference.json.gz")[..],
        ))
        .unwrap();
        assert_eq!(data["upstreamCommit"], crate::PELE_COMMIT);
        assert_eq!(data["seed"], "0x18c0de");
        assert_eq!(data["absentSource"], "JavaScript undefined corresponds to Rust None");
        data
    }
    fn bits(value: &str) -> f64 {
        f64::from_bits(u64::from_str_radix(value, 16).unwrap())
    }
    fn frozen(value: f64) -> Value {
        json!({"$bits":format!("{:016x}",value.to_bits())})
    }
    fn freeze_label(label: LabelView<'_>) -> Value {
        let lines: Vec<Value> = label
            .lines
            .iter()
            .map(|line| {
                let spans: Vec<Value> = line
                    .iter()
                    .map(|span| {
                        let mut value = json!({"text":span.text,"style":frozen(span.style as f64),"width":frozen(span.width)});
                        if let Some(icon) = &span.icon {
                            value["icon"] = json!(icon);
                        }
                        value
                    })
                    .collect();
                json!(spans)
            })
            .collect();
        let widths: Vec<Value> = label.widths.iter().map(|&value| frozen(value)).collect();
        json!({"lines":lines,"widths":widths,"width":frozen(label.width),"height":frozen(label.height),"size":frozen(label.size),"lineHeight":frozen(label.line_height)})
    }
    fn icons() -> Icons {
        Icons::new(Some(Arc::new(|name: &str| (name == "fa:fa-car").then(|| "<path d=\"M1 2\"/>".to_owned()))))
    }

    #[test]
    fn render_and_public_labels_match_independent_node_values_traces_and_svg() {
        let data = fixture();
        let cases = data["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 6048);
        for (index, case) in cases.iter().enumerate() {
            for compact in [false, true] {
                let calls = RefCell::new(Vec::new());
                let measurer = |text: &str, size: f64, style: u8| {
                    calls.borrow_mut().push((text.to_owned(), format!("{:016x}", size.to_bits()), style));
                    let width = MetricsMeasurer.width(text, size, style);
                    if case["mode"] == "custom" {
                        width - if text.contains("AV") { size * 0.75 } else { 0.0 }
                    } else {
                        width
                    }
                };
                let icons = icons();
                let m = Measure::new(&measurer, &icons);
                let source = case["source"].as_str();
                let markdown = case["markdown"].as_bool().unwrap();
                let size = bits(case["size"].as_str().unwrap());
                let max = bits(case["max"].as_str().unwrap());
                let base = case["base"].as_u64().unwrap() as u8;
                let (value, svg) = if compact {
                    let label = layout_render_label(source, markdown, &m, size, max, base);
                    assert_eq!(
                        crate::svg::builder::render_label_svg(&label, -12.345, 0.1, " class=\"test\"", Some(&icons)).to_string(),
                        case["svgWithIcons"].as_str().unwrap(),
                        "icon SVG {index}"
                    );
                    (
                        freeze_label(label.view()),
                        crate::svg::builder::render_label_svg(&label, -12.345, 0.1, " class=\"test\"", None).to_string(),
                    )
                } else {
                    let label = layout_label(source, markdown, &m, size, max, base);
                    assert_eq!(
                        crate::svg::builder::label_svg(&label, -12.345, 0.1, " class=\"test\"", Some(&icons)).to_string(),
                        case["svgWithIcons"].as_str().unwrap(),
                        "icon SVG {index}"
                    );
                    (freeze_label(label.view()), crate::svg::builder::label_svg(&label, -12.345, 0.1, " class=\"test\"", None).to_string())
                };
                assert_eq!(value, case["expect"], "case {index}, compact {compact}");
                assert_eq!(serde_json::to_value(calls.into_inner()).unwrap(), case["trace"], "trace {index}, compact {compact}");
                assert_eq!(svg, case["svg"].as_str().unwrap(), "SVG {index}, compact {compact}");
            }
        }
    }

    #[test]
    fn rounding_matches_independent_node_bits() {
        let data = fixture();
        let cases = data["round"].as_array().unwrap();
        assert_eq!(cases.len(), 4108);
        for row in cases {
            let input = bits(row[0].as_str().unwrap());
            let expected = bits(row[1].as_str().unwrap());
            assert_eq!(crate::util::js::round(input).to_bits(), expected.to_bits(), "round bits {}", row[0]);
        }
    }
    #[test]
    fn restyling_matches_the_pinned_internal_node_function() {
        let data = fixture();
        let cases = data["cases"].as_array().unwrap();
        assert_eq!(data["restyleSourceSha256"], "dd6dfbb4ab327b00ee0e5a99d3000548fc3f85a29df52d6b83d03b3ea9a53918");
        let mut count = 0;
        for (index, case) in cases.iter().enumerate() {
            for styled in case["restyled"].as_array().unwrap() {
                let calls = RefCell::new(Vec::new());
                let measurer = |text: &str, size: f64, style: u8| {
                    calls.borrow_mut().push((text.to_owned(), format!("{:016x}", size.to_bits()), style));
                    let width = MetricsMeasurer.width(text, size, style);
                    if case["mode"] == "custom" {
                        width - if text.contains("AV") { size * 0.75 } else { 0.0 }
                    } else {
                        width
                    }
                };
                let icons = icons();
                let m = Measure::new(&measurer, &icons);
                let mut label = layout_render_label(
                    case["source"].as_str(),
                    case["markdown"].as_bool().unwrap(),
                    &m,
                    bits(case["size"].as_str().unwrap()),
                    bits(case["max"].as_str().unwrap()),
                    case["base"].as_u64().unwrap() as u8,
                );
                calls.borrow_mut().clear();
                label.restyle(styled["bits"].as_u64().unwrap() as u8, &m);
                assert_eq!(freeze_label(label.view()), styled["expect"], "restyle {index}, bits {}", styled["bits"]);
                assert_eq!(serde_json::to_value(calls.into_inner()).unwrap(), styled["trace"], "restyle trace {index}");
                assert_eq!(
                    crate::svg::builder::render_label_svg(&label, -12.345, 0.1, " class=\"test\"", None).to_string(),
                    styled["svg"].as_str().unwrap(),
                    "restyle SVG {index}"
                );
                assert_eq!(
                    crate::svg::builder::render_label_svg(&label, -12.345, 0.1, " class=\"test\"", Some(&icons)).to_string(),
                    styled["svgWithIcons"].as_str().unwrap(),
                    "restyle icon SVG {index}"
                );
                count += 1;
            }
        }
        assert_eq!(count, 18144);
    }
}
