//! Exercise the host theme contract against every enabled diagram's documentation inputs.
mod common;
use common::theme::sentinel;
use quick_xml::{events::Event, Reader};
use std::collections::BTreeMap;
use zebra_mermaid::{render, type_enabled, CanvasBackground, ColorMode, DiagramType, RenderOptions};

fn concrete_attributes(svg: &str) {
    let mut reader = Reader::from_str(svg);
    loop {
        match reader.read_event().expect("well-formed SVG") {
            Event::Start(e) | Event::Empty(e) => {
                for attr in e.attributes() {
                    let attr = attr.expect("valid XML attribute");
                    let value = attr.value.as_ref();
                    for forbidden in ["var(", "color-mix(", "light-dark("] {
                        assert!(!value.contains(forbidden), "unresolved SVG attribute: {value}");
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
}

#[test]
fn every_enabled_family_accepts_the_host_palette_without_changing_geometry() {
    let mut counts = BTreeMap::<String, usize>::new();
    let mut files: Vec<_> = std::fs::read_dir(common::fixtures().join("corpus")).unwrap().map(|e| e.unwrap().path()).collect();
    files.sort();
    for path in files {
        let name = path.file_stem().unwrap().to_str().unwrap();
        let Ok(kind) = name.parse::<DiagramType>() else {
            continue;
        };
        if !type_enabled(kind) {
            continue;
        }
        for row in common::read_jsonl_path(&path) {
            if row["expect"]["ok"] != true {
                continue;
            }
            let source = row["source"].as_str().unwrap();
            let mut p = sentinel();
            let base = RenderOptions {
                color_mode: ColorMode::Resolved,
                palette: Some(p.clone()),
                now: Some(1_792_368_000_000.0),
                canvas_background: Some(CanvasBackground::Transparent),
                ..Default::default()
            };
            let a = render(source, &base).unwrap_or_else(|e| panic!("{name} {}: {e}", row["id"]));
            p.background = "#E7E8E9".into();
            p.text = "#141516".into();
            p.accent = "#334455".into();
            p.series.as_mut().unwrap()[0] = "#223344".into();
            let b = render(source, &RenderOptions { palette: Some(p), ..base }).unwrap();
            assert_eq!((a.width, a.height), (b.width, b.height), "palette changed {name} geometry, case {}", row["id"]);
            assert!(a.width.is_finite() && a.height.is_finite());
            concrete_attributes(&a.svg);
            concrete_attributes(&b.svg);
            *counts.entry(name.into()).or_default() += 1;
        }
    }
    assert!(!counts.is_empty(), "enable at least one diagram for the theme matrix");
    for (name, count) in counts {
        eprintln!("{name}: {count} documentation inputs passed concrete-attribute and geometry checks");
    }
}
