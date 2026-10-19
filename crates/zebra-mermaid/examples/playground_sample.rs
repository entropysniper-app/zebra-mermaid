//! Browser-hosted sample comparison. Only text measurements and trusted site icons cross
//! the host boundary; parsing, layout and SVG construction execute in the Rust renderer.
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashMap,
    io::Read,
    sync::{Arc, Mutex},
};
use zebra_mermaid::{render, IconTable, MetricsMeasurer, RenderOptions, TextMeasurer};

#[derive(Clone, Deserialize)]
struct Measurement {
    text: String,
    size: f64,
    style: u8,
    width: f64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    source: String,
    id_prefix: String,
    font_family: String,
    font_family_mono: String,
    max_width: f64,
    now: f64,
    measurements: Vec<Measurement>,
    icons: HashMap<String, Option<String>>,
}
fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let request: Request = serde_json::from_str(&input).unwrap();
    let widths: HashMap<(String, u64, u8), f64> =
        request.measurements.into_iter().map(|m| ((m.text, m.size.to_bits(), m.style), m.width)).collect();
    let missing = Arc::new(Mutex::new(HashMap::new()));
    let missing_measurements = missing.clone();
    let measurer = move |text: &str, size: f64, style: u8| {
        widths.get(&(text.to_owned(), size.to_bits(), style)).copied().unwrap_or_else(|| {
            missing_measurements
                .lock()
                .unwrap()
                .insert((text.to_owned(), size.to_bits(), style), json!({"text":text,"size":size,"style":style}));
            // The caller must collect these widths and rerun; this provisional output is
            // never accepted or published while the missing list is nonempty.
            MetricsMeasurer.width(text, size, style)
        })
    };
    let mut icons = IconTable::new();
    for (name, markup) in request.icons {
        if let Some(markup) = markup {
            icons.insert(name, markup);
        }
    }
    let options = RenderOptions {
        font_family: Some(request.font_family),
        font_family_mono: Some(request.font_family_mono),
        max_width: Some(request.max_width),
        now: Some(request.now),
        id_prefix: Some(request.id_prefix),
        icons: Some(Arc::new(icons)),
        measurer: Some(Arc::new(measurer)),
        ..Default::default()
    };
    let result = render(&request.source, &options);
    let mut missing: Vec<_> = missing.lock().unwrap().values().cloned().collect();
    missing.sort_by_key(|m| m.to_string());
    let output = match result {
        Ok(r) => {
            json!({"ok":true,"type":r.diagram_type.as_str(),"width":r.width,"height":r.height,"svg":r.svg,"missingMeasurements":missing})
        }
        Err(e) => {
            json!({"ok":false,"error":{"code":e.code.as_str(),"message":e.message,"line":e.line,"column":e.column},"missingMeasurements":missing})
        }
    };
    println!("{output}");
}
