//! Supplementary independent Node references; original-test mappings are unchanged.
use std::cell::RefCell;

use serde_json::Value;
use zebra_mermaid::diagrams::common::literal_label::{break_label, left_label_svg, literal_label};
use zebra_mermaid::text::{Icons, Label, Measure, MetricsMeasurer, Span, TextMeasurer};

fn fixture() -> Value {
    let value: Value = serde_json::from_reader(flate2::read::GzDecoder::new(
        &include_bytes!("../../../tests/fixtures/unit/literal-label-regression.json.gz")[..],
    ))
    .unwrap();
    assert_eq!(value["upstreamCommit"], zebra_mermaid::PELE_COMMIT);
    assert_eq!(value["seed"], "0x17e2b6");
    value
}

fn bits(value: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(value, 16).unwrap())
}

fn number(value: &Value) -> f64 {
    bits(value["$bits"].as_str().unwrap())
}

fn label(value: &Value) -> Label {
    Label {
        lines: value["lines"]
            .as_array()
            .unwrap()
            .iter()
            .map(|line| {
                line.as_array()
                    .unwrap()
                    .iter()
                    .map(|span| Span {
                        text: span["text"].as_str().unwrap().to_owned(),
                        style: number(&span["style"]) as u8,
                        icon: span["icon"].as_str().map(str::to_owned),
                        width: number(&span["width"]),
                    })
                    .collect()
            })
            .collect(),
        widths: value["widths"].as_array().unwrap().iter().map(number).collect(),
        width: number(&value["width"]),
        height: number(&value["height"]),
        size: number(&value["size"]),
        line_height: number(&value["lineHeight"]),
    }
}

fn assert_label(actual: &Label, expected: &Value, index: usize) {
    let expected = label(expected);
    assert_eq!(actual, &expected, "case {index}");
    for (a, b) in [
        (actual.width, expected.width),
        (actual.height, expected.height),
        (actual.size, expected.size),
        (actual.line_height, expected.line_height),
    ] {
        assert_eq!(a.to_bits(), b.to_bits(), "case {index}");
    }
    for (a, b) in actual.widths.iter().zip(&expected.widths) {
        assert_eq!(a.to_bits(), b.to_bits(), "case {index}");
    }
    for (a, b) in actual.lines.iter().flatten().zip(expected.lines.iter().flatten()) {
        assert_eq!(a.width.to_bits(), b.width.to_bits(), "case {index}");
    }
}

#[test]
fn literal_layout_preserves_reference_results_and_callback_order() {
    let data = fixture();
    let cases = data["literal"].as_array().unwrap();
    assert_eq!(cases.len(), 1008);
    for (index, case) in cases.iter().enumerate() {
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
        let icons = Icons::none();
        let measure = Measure::new(&measurer, &icons);
        let actual = literal_label(
            case["text"].as_str().unwrap(),
            &measure,
            bits(case["size"].as_str().unwrap()),
            bits(case["room"].as_str().unwrap()),
            case["style"].as_u64().unwrap() as u8,
        );
        assert_label(&actual, &case["expected"], index);
        assert_eq!(serde_json::to_value(calls.into_inner()).unwrap(), case["trace"], "case {index}");
    }
}

#[test]
fn character_wrapping_preserves_reference_results_and_callback_order() {
    let data = fixture();
    let cases = data["broken"].as_array().unwrap();
    assert_eq!(cases.len(), 1184);
    for (index, case) in cases.iter().enumerate() {
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
        let icons = Icons::none();
        let measure = Measure::new(&measurer, &icons);
        let actual = break_label(&label(&case["label"]), &measure, bits(case["room"].as_str().unwrap()));
        assert_label(&actual, &case["expected"], index);
        assert_eq!(serde_json::to_value(calls.into_inner()).unwrap(), case["trace"], "case {index}");
    }
}

#[test]
fn left_aligned_svg_preserves_reference_bytes() {
    let data = fixture();
    let cases = data["svg"].as_array().unwrap();
    assert_eq!(cases.len(), 444);
    for (index, case) in cases.iter().enumerate() {
        let actual = left_label_svg(
            &label(&case["label"]),
            bits(case["x"].as_str().unwrap()),
            bits(case["top"].as_str().unwrap()),
            case["attrs"].as_str().unwrap(),
        );
        assert_eq!(actual, case["expected"].as_str().unwrap(), "case {index}");
    }
}
