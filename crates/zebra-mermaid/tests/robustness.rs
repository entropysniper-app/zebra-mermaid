//! Rust-side invariants (docs/TEST-PLAN.md §1, §7): no panic on mutated input, well-formed XML,
//! deterministic output, independent renders across threads, and resource limits that hold.
//! Every enabled diagram type is covered through the corpus seeds.

mod common;

use std::collections::BTreeMap;

use proptest::prelude::*;
use quick_xml::events::Event;
use quick_xml::Reader;
use zebra_mermaid::config::yaml::parse_yaml;
use zebra_mermaid::preprocess::preprocess;
use zebra_mermaid::text::{layout_label, Icons, Measure, MetricsMeasurer};
use zebra_mermaid::{render, type_enabled, ColorMode, DiagramType, ErrorCode, RenderOptions};

/// A small deterministic generator (xorshift64*).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545F4914F6CDD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

const PIECES: &[&str] = &[
    "\n",
    " ",
    "  ",
    "\t",
    ":",
    ";",
    "\"",
    "'",
    "[",
    "]",
    "(",
    ")",
    "{",
    "}",
    "<",
    ">",
    "-->",
    "---",
    "%%",
    "%%{init: {}}%%",
    "#",
    "&amp;",
    "#quot;",
    "<br/>",
    "**",
    "_",
    "\\n",
    "中文",
    "😀",
    "e\u{301}",
    "\u{feff}",
    "\u{2028}",
    "\u{0}",
    "0",
    "-1",
    "1e9",
    "NaN",
    "100%",
    "end",
    "subgraph",
    "title",
    "accTitle: x",
    "accDescr {",
    "click A \"javascript:alert(1)\"",
    "style A fill:url(#x)",
    "@{ shape: rect }",
];

fn mutate(seed: &str, rng: &mut Rng) -> String {
    let mut chars: Vec<char> = seed.chars().collect();
    for _ in 0..1 + rng.below(6) {
        let at = rng.below(chars.len() + 1);
        match rng.below(4) {
            0 if !chars.is_empty() && at < chars.len() => {
                let n = 1 + rng.below(8).min(chars.len() - at);
                chars.drain(at..(at + n).min(chars.len()));
            }
            1 => {
                let piece = PIECES[rng.below(PIECES.len())];
                chars.splice(at..at, piece.chars());
            }
            2 if !chars.is_empty() => {
                // Duplicate a stretch, which grows nesting and repetition.
                let from = rng.below(chars.len());
                let to = (from + 1 + rng.below(40)).min(chars.len());
                let copy: Vec<char> = chars[from..to].to_vec();
                chars.splice(at..at, copy);
            }
            _ => {
                let c = char::from_u32(rng.below(0x3000) as u32).unwrap_or('x');
                chars.insert(at, c);
            }
        }
    }
    chars.into_iter().collect()
}

fn well_formed(svg: &str) -> Result<(), String> {
    let mut reader = Reader::from_str(svg);
    let mut depth = 0i64;
    loop {
        match reader.read_event() {
            Ok(Event::Start(_)) => depth += 1,
            Ok(Event::End(_)) => depth -= 1,
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => return Err(format!("{e} at {}", reader.buffer_position())),
        }
    }
    if depth != 0 {
        return Err("unbalanced".into());
    }
    Ok(())
}

fn seeds() -> Vec<(String, String)> {
    let dir = common::fixtures().join("corpus");
    let mut out = Vec::new();
    let mut files: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    files.sort();
    for path in files {
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let enabled = name.parse::<DiagramType>().is_ok_and(type_enabled);
        if !enabled {
            continue;
        }
        for row in common::read_jsonl_path(&path) {
            out.push((name.clone(), row["source"].as_str().unwrap().to_string()));
        }
    }
    out
}

#[test]
fn mutated_inputs_never_panic_and_stay_well_formed() {
    let mut rng = Rng(0x9E3779B97F4A7C15);
    let options = [
        RenderOptions { id_prefix: Some("t".into()), now: Some(1.79e12), ..Default::default() },
        RenderOptions { color_mode: ColorMode::Resolved, max_width: Some(200.0), now: Some(1.79e12), ..Default::default() },
    ];
    let mut stats: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let seeds = seeds();
    assert!(!seeds.is_empty());
    for (kind, seed) in &seeds {
        for _ in 0..60 {
            let input = mutate(seed, &mut rng);
            for o in &options {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| render(&input, o)));
                let result = result.unwrap_or_else(|_| panic!("panic on {kind} input {input:?}"));
                let e = stats.entry(kind.clone()).or_default();
                match result {
                    Ok(r) => {
                        e.0 += 1;
                        well_formed(&r.svg).unwrap_or_else(|err| panic!("{kind}: malformed SVG ({err}) for {input:?}"));
                        assert!(r.width.is_finite() && r.height.is_finite() && r.width >= 0.0 && r.height >= 0.0, "{kind}: size");
                    }
                    Err(_) => e.1 += 1,
                }
            }
        }
    }
    for (kind, (ok, err)) in stats {
        eprintln!("{kind:>14}: {ok} rendered, {err} refused");
    }
}

#[test]
fn renders_are_deterministic_and_thread_independent() {
    let seeds = seeds();
    let options = RenderOptions { now: Some(1.79e12), ..Default::default() };
    let sequential: Vec<String> = seeds.iter().map(|(_, s)| render(s, &options).map(|r| r.svg).unwrap_or_else(|e| e.message)).collect();
    let again: Vec<String> = seeds.iter().map(|(_, s)| render(s, &options).map(|r| r.svg).unwrap_or_else(|e| e.message)).collect();
    assert_eq!(sequential, again);
    let parallel: Vec<String> = std::thread::scope(|scope| {
        let handles: Vec<_> = seeds
            .iter()
            .map(|(_, s)| {
                let options = options.clone();
                scope.spawn(move || render(s, &options).map(|r| r.svg).unwrap_or_else(|e| e.message))
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    assert_eq!(sequential, parallel);
}

#[test]
fn limits_hold() {
    let big = format!("pie\n{}", "  \"a\" : 1\n".repeat(6000));
    let e = render(&big, &RenderOptions::default()).unwrap_err();
    assert_eq!(e.code, ErrorCode::Limit);
    assert_eq!(e.message, "Diagram source is longer than the limit of 50000 characters.");
    // Counted in UTF-16 units: 25,001 astral characters are 50,002 units.
    let astral = format!("pie title {}", "😀".repeat(25_000));
    assert_eq!(render(&astral, &RenderOptions::default()).unwrap_err().code, ErrorCode::Limit);
    let small = RenderOptions { output_limit: Some(100), ..Default::default() };
    let e = render("pie\n \"a\" : 1", &small).unwrap_err();
    assert_eq!(e.message, "Diagram output is longer than the limit of 100 characters.");
    // Deep YAML and JSON in front matter and directives are refused, not overflowed.
    let deep_yaml = format!("---\nconfig: {}{}\n---\npie", "[".repeat(100), "]".repeat(100));
    assert_eq!(render(&deep_yaml, &RenderOptions::default()).unwrap_err().code, ErrorCode::Syntax);
    let deep_json = format!("%%{{init: {}{}}}%%\npie", "{\"a\":".repeat(300), "1".to_string() + &"}".repeat(300));
    assert_eq!(render(&deep_json, &RenderOptions::default()).unwrap_err().code, ErrorCode::Limit);
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 512, failure_persistence: None, ..ProptestConfig::default() })]

    #[test]
    fn core_functions_take_any_text(text in "\\PC{0,80}") {
        let _ = preprocess(&text);
        let _ = parse_yaml(&text);
        let icons = Icons::none();
        let m = Measure::new(&MetricsMeasurer, &icons);
        for markdown in [false, true] {
            let l = layout_label(Some(&text), markdown, &m, 16.0, 60.0, 0);
            prop_assert!(l.width.is_finite());
        }
        let _ = zebra_mermaid::util::url::sanitize_url(&text);
        let _ = render(&text, &RenderOptions::default());
    }

    #[test]
    fn source_positions_point_into_the_source(prefix in "[a-z \\n%#:\"中😀]{0,30}", body in "[a-z \\n:\"0-9]{0,30}") {
        let src = format!("{prefix}\npie\n{body}");
        if let Err(e) = render(&src, &RenderOptions::default()) {
            if let Some(span) = &e.span {
                prop_assert!(span.start <= src.len());
                prop_assert!(src.is_char_boundary(span.start));
            }
        }
    }
}
