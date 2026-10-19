//! Paired benchmark driver; validation is outside the timed region.
use serde_json::{json, Value};
use std::{hint::black_box, time::Instant};
use zebra_mermaid::{parse, render, type_enabled, DiagramType, ParseOptions, RenderOptions};
fn hash(s: &str) -> String {
    let h = s.as_bytes().iter().fold(0xcbf29ce484222325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x100000001b3));
    format!("{h:016x}")
}
fn time(f: impl Fn(&str) -> bool, inputs: &[String]) -> Value {
    let run = || inputs.iter().filter(|s| f(s)).count();
    let done = run();
    for _ in 0..4 {
        black_box(run());
    }
    let mut repeats = 1usize;
    loop {
        let start = Instant::now();
        for _ in 0..repeats {
            black_box(run());
        }
        if start.elapsed().as_secs_f64() * 1000.0 >= 10.0 || repeats >= 131072 {
            break;
        }
        repeats *= 2;
    }
    let budget = std::env::var("BENCH_MS").ok().and_then(|s| s.parse::<f64>().ok()).unwrap_or(300.0);
    let count = std::env::var("BENCH_SAMPLES").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(9);
    let mut elapsed = 0.0;
    let mut samples = vec![];
    while elapsed < budget || samples.len() < count {
        let start = Instant::now();
        for _ in 0..repeats {
            assert_eq!(black_box(run()), done, "success count changed between samples");
        }
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        elapsed += ms;
        samples.push(ms / repeats as f64);
    }
    samples.sort_by(f64::total_cmp);
    let q = |p: f64| samples[((samples.len() - 1) as f64 * p).floor() as usize];
    json!({"ms":q(0.5),"p10":q(0.1),"p90":q(0.9),"done":done,"samples":samples,"repeats":repeats})
}
fn main() {
    let cases: Vec<Value> =
        serde_json::from_str(&std::fs::read_to_string("target/bench/cases.json").expect("generate cases first")).unwrap();
    let options = RenderOptions { now: Some(1792368000000.0), ..Default::default() };
    let mut out = vec![];
    for c in cases {
        let inputs: Vec<_> = c["inputs"].as_array().unwrap().iter().map(|s| s.as_str().unwrap().to_owned()).collect();
        let verification: Vec<_> = inputs
            .iter()
            .map(|s| match render(s, &options) {
                Ok(r) => json!({"ok":true,"svgHash":hash(&r.svg),"width":r.width,"height":r.height}),
                Err(e) => json!({"ok":false,"code":e.code.as_str(),"message":e.message,"line":e.line,"column":e.column}),
            })
            .collect();
        let valid: Vec<_> = inputs.iter().zip(&verification).filter(|(_, v)| v["ok"] == true).map(|(s, _)| s.clone()).collect();
        let invalid: Vec<_> = inputs.iter().zip(&verification).filter(|(_, v)| v["ok"] == false).map(|(s, _)| s.clone()).collect();
        let p =
            if valid.is_empty() { Value::Null } else { time(|s| black_box(parse(black_box(s), &ParseOptions::default())).is_ok(), &valid) };
        let r = if valid.is_empty() { Value::Null } else { time(|s| black_box(render(black_box(s), &options)).is_ok(), &valid) };
        let rejection =
            if invalid.is_empty() { Value::Null } else { time(|s| black_box(render(black_box(s), &options)).is_err(), &invalid) };
        out.push(json!({"name":c["name"],"diagrams":inputs.len(),"verification":verification,"renderInputs":valid.len(),"rejectInputs":invalid.len(),"render":r,"parse":p,"rejection":rejection}));
    }
    let enabled: Vec<_> = DiagramType::ALL.into_iter().filter(|t| type_enabled(*t)).map(|t| t.as_str()).collect();
    let report = json!({"engine":"zebra-mermaid","version":env!("CARGO_PKG_VERSION"),"enabledTypes":enabled,"cases":out});
    let output = std::env::args().nth(1).unwrap_or_else(|| "target/bench/rust.json".into());
    std::fs::write(output, serde_json::to_string(&report).unwrap()).unwrap();
}
