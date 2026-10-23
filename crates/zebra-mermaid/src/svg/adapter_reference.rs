//! Supplementary checks for the direct-part adapters against existing independent Node data.
use super::*;
use crate::text::{layout_render_label, MetricsMeasurer, Span, TextMeasurer};
use serde_json::{json, Value};
use std::sync::Arc;

pub(crate) fn bits(value: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(value, 16).unwrap())
}

pub(crate) fn label(value: &Value) -> Label {
    let number = |n: &Value| bits(n["$bits"].as_str().unwrap());
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
                        math: None,
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
        heights: value["heights"].as_array().map(|h| h.iter().map(number).collect()),
    }
}

#[test]
fn direct_label_parts_match_independent_node_svg() {
    let data: Value = serde_json::from_reader(flate2::read::GzDecoder::new(
        &include_bytes!("../../../../tests/fixtures/pele-1.9.0/unit/render-label-reference.json.gz")[..],
    ))
    .unwrap();
    assert_eq!(data["upstreamCommit"], crate::PELE_COMMIT);
    assert_eq!(data["seed"], "0x18c0de");
    let cases = data["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 6048);
    for (index, case) in cases.iter().enumerate() {
        let measurer = |text: &str, size: f64, style: u8| {
            let width = MetricsMeasurer.width(text, size, style);
            if case["mode"] == "custom" {
                width - if text.contains("AV") { size * 0.75 } else { 0.0 }
            } else {
                width
            }
        };
        let icons = Icons::new(Some(Arc::new(|name: &str| (name == "fa:fa-car").then(|| "<path d=\"M1 2\"/>".to_owned()))));
        let measure = crate::text::Measure::new(&measurer, &icons);
        let label = layout_render_label(
            case["source"].as_str(),
            case["markdown"].as_bool().unwrap(),
            &measure,
            bits(case["size"].as_str().unwrap()),
            bits(case["max"].as_str().unwrap()),
            case["base"].as_u64().unwrap() as u8,
        );
        let view = label.view();
        for (resolver, key) in [(None, "svg"), (Some(&icons), "svgWithIcons")] {
            let actual = crate::svg::concat::svg_format!(render_label_svg(&label, -12.345, 0.1, " class=\"test\"", resolver));
            assert_eq!(actual, case[key].as_str().unwrap(), "label part {index} {key}");
            let actual = crate::svg::concat::svg_format!(view_label_svg(&view, -12.345, 0.1, " class=\"test\"", resolver));
            assert_eq!(actual, case[key].as_str().unwrap(), "view part {index} {key}");
        }
    }
    // Diagnostics only; target-specific layout is not an API assertion or a speed claim.
    println!(
        "{}",
        json!({"labelViewBytes":std::mem::size_of::<LabelView>(),
        "labelReferenceBytes":std::mem::size_of::<&RenderLabel>(),
        "renderLabelAdapterBytes":std::mem::size_of::<RenderLabelSvg>(),
        "viewLabelAdapterBytes":std::mem::size_of::<ViewLabelSvg>(),
        "publicEdgeAdapterBytes":std::mem::size_of::<crate::svg::edges::EdgeLabelSvg>(),
        "renderEdgeAdapterBytes":std::mem::size_of::<crate::svg::edges::RenderEdgeLabelSvg>(),
        "leftLabelAdapterBytes":std::mem::size_of::<crate::diagrams::common::literal_label::LeftLabelSvg<RenderLabel>>() })
    );
}
