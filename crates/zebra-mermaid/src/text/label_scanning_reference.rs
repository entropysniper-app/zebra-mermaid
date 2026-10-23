//! Independent labels and combined callback ordering, shared by public/internal paths.
use super::*;
use crate::text::render_label::{LabelRead, LabelView};
use crate::text::{Icons, MetricsMeasurer, TextMeasurer};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::sync::Arc;

thread_local! { static TRACE: RefCell<Vec<Value>> = const { RefCell::new(Vec::new()) }; }
fn record(event: Value) {
    TRACE.with(|trace| trace.borrow_mut().push(event));
}
fn bits(value: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(value, 16).unwrap())
}
fn frozen(value: f64) -> Value {
    json!({"$bits":format!("{:016x}",value.to_bits())})
}
fn freeze_label(label: LabelView<'_>) -> Value {
    let lines: Vec<Vec<Value>> = label
        .lines
        .iter()
        .map(|line| {
            line.iter()
                .map(|span| {
                    let mut value = json!({"text":span.text,"style":frozen(span.style as f64),"width":frozen(span.width)});
                    if let Some(icon) = &span.icon {
                        value["icon"] = json!(icon);
                    }
                    value
                })
                .collect()
        })
        .collect();
    json!({"lines":lines,"widths":label.widths.iter().map(|&n| frozen(n)).collect::<Vec<_>>(),
        "width":frozen(label.width),"height":frozen(label.height),"size":frozen(label.size),"lineHeight":frozen(label.line_height)})
}

#[test]
fn labels_and_combined_callbacks_match_pinned_reference() {
    let data: Value = serde_json::from_reader(flate2::read::GzDecoder::new(
        &include_bytes!("../../../../tests/fixtures/pele-1.9.0/unit/label-scanning-reference.json.gz")[..],
    ))
    .unwrap();
    assert_eq!(data["upstreamCommit"], crate::PELE_COMMIT);
    assert_eq!(data["seed"], "0x1abe17");
    assert_eq!(data["generatedSources"], 64);
    assert_eq!(data["sourceCases"], 88);
    let cases = data["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 8448);
    for (index, case) in cases.iter().enumerate() {
        for compact in [false, true] {
            TRACE.with(|trace| trace.borrow_mut().clear());
            let icons = Icons::new(case["icons"].as_bool().unwrap().then(|| {
                Arc::new(|name: &str| {
                    record(json!(["icon", name]));
                    (name == "fa:fa-car").then(|| "<path/>".into())
                }) as Arc<dyn crate::text::IconResolver>
            }));
            let measurer = |text: &str, size: f64, style: u8| {
                record(json!(["width", text, format!("{:016x}", size.to_bits()), style]));
                let w = MetricsMeasurer.width(text, size, style);
                if case["mode"] == "custom" {
                    w - if text.contains("AV") { size * 0.75 } else { 0.0 }
                } else {
                    w
                }
            };
            let m = Measure::new(&measurer, &icons);
            let source = case["source"].as_str();
            let markdown = case["markdown"].as_bool().unwrap();
            let size = bits(case["size"].as_str().unwrap());
            let max = bits(case["max"].as_str().unwrap());
            let base = case["base"].as_u64().unwrap() as u8;
            let actual = if compact {
                freeze_label(layout_render_label(source, markdown, &m, size, max, base).view())
            } else {
                freeze_label(layout_label(source, markdown, &m, size, max, base).view())
            };
            assert_eq!(actual, case["expected"], "label {index}, compact={compact}");
            TRACE.with(|trace| assert_eq!(json!(*trace.borrow()), case["trace"], "callback order {index}, compact={compact}"));
        }
    }
}
