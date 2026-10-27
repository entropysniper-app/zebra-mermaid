//! Complete independent ports of Pele 1.12.0 release callbacks (MIT, kepano 2026).
//! Each callback retains its own function and mapping in tests/upstream/ports.json.
#![allow(dead_code)]
use regex::Regex;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use zebra_mermaid::{parse, render, DiagramModel, MetricsMeasurer, ParseOptions, RenderOptions, RenderResult, TextMeasurer};
const ON_LIGHT: &str = "var(--pele-fg-on-light,#1f1f1f)";
const ON_DARK: &str = "var(--pele-fg-on-dark,#fff)";
fn re(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap()
}
fn captures(svg: &str, pattern: &str) -> Vec<String> {
    re(pattern)
        .captures(svg)
        .unwrap_or_else(|| panic!("No match for {pattern}"))
        .iter()
        .map(|m| m.map_or(String::new(), |s| s.as_str().to_owned()))
        .collect()
}
fn matches(svg: &str, pattern: &str) -> Vec<Vec<String>> {
    re(pattern).captures_iter(svg).map(|c| c.iter().map(|m| m.map_or(String::new(), |s| s.as_str().to_owned())).collect()).collect()
}
fn has(svg: &str, pattern: &str) -> bool {
    re(pattern).is_match(svg)
}
fn number(s: &str) -> f64 {
    s.parse().unwrap()
}
fn close(a: f64, b: f64, precision: i32) {
    assert!((a - b).abs() < 0.5 * 10f64.powi(-precision), "{a} != {b}, precision {precision}");
}
fn options(extra: Value) -> RenderOptions {
    crate::common::render_options(&extra, &Value::Null)
}
fn draw(source: &str) -> RenderResult {
    render(source, &RenderOptions::default()).unwrap()
}
fn draw_with(source: &str, extra: Value) -> RenderResult {
    render(source, &options(extra)).unwrap()
}
fn svg(source: &str) -> String {
    draw(source).svg
}
fn svg_with(source: &str, extra: Value) -> String {
    draw_with(source, extra).svg
}
fn model(source: &str) -> zebra_mermaid::ParsedDiagram {
    parse(source, &ParseOptions::default()).unwrap()
}
#[derive(Debug, Clone)]
struct Element {
    name: String,
    attrs: BTreeMap<String, String>,
}
impl Element {
    fn attr(&self, name: &str) -> &str {
        self.attrs.get(name).map(String::as_str).unwrap_or("")
    }
    fn n(&self, name: &str) -> f64 {
        if self.attr(name).is_empty() {
            0.0
        } else {
            number(self.attr(name))
        }
    }
}
fn elements(svg: &str) -> Vec<Element> {
    let mut reader = quick_xml::Reader::from_str(svg);
    let mut out = Vec::new();
    loop {
        match reader.read_event().unwrap() {
            quick_xml::events::Event::Start(e) | quick_xml::events::Event::Empty(e) => {
                let attrs = e
                    .attributes()
                    .map(|a| {
                        let a = a.unwrap();
                        (a.key.as_ref().to_owned(), a.normalized_value(quick_xml::XmlVersion::Implicit1_0).unwrap().to_string())
                    })
                    .collect();
                out.push(Element { name: e.name().as_ref().to_owned(), attrs });
            }
            quick_xml::events::Event::Eof => break,
            _ => {}
        }
    }
    out
}
fn node(svg: &str, id: &str) -> (String, String) {
    let c = captures(svg, &format!(r#"<g class="([^"]*pele-node[^"]*)" data-id="{}"[^>]*>(.*?)</g>"#, regex::escape(id)));
    (c[1].clone(), c[2].clone())
}
fn at(svg: &str, id: &str) -> [f64; 2] {
    let c = captures(svg, &format!(r#"data-id="{}" transform="translate\(([-\d.]+),([-\d.]+)"#, regex::escape(id)));
    [number(&c[1]), number(&c[2])]
}

#[cfg(feature = "block")]
#[path = "release_1_12_block.rs"]
mod block;
#[path = "release_1_12_color.rs"]
mod color;
#[cfg(feature = "flowchart")]
#[path = "release_1_12_flow.rs"]
mod flow;
#[path = "release_1_12_mono.rs"]
mod mono;
#[path = "release_1_12_parse.rs"]
mod parsing;
#[cfg(feature = "sequence")]
#[path = "release_1_12_sequence.rs"]
mod sequence;
#[cfg(feature = "treemap")]
#[path = "release_1_12_treemap.rs"]
mod treemap;
