//! Exact sampling protocol of Pele scripts/bench.mjs: one warm-up, 300 ms, >=5,
//! at most 2000 batches, upper median. Each sample renders one entire input batch.
use serde_json::{json, Value};
use std::{hint::black_box, time::Instant};
use zebra_mermaid::{render, type_enabled, DiagramType, RenderOptions};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let cases: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(&args[1]).unwrap()).unwrap();
    let options = RenderOptions::default();
    let mut out = vec![];
    for c in &cases {
        let inputs: Vec<_> = c["inputs"].as_array().unwrap().iter().map(|s| s.as_str().unwrap()).collect();
        let run = || inputs.iter().filter(|s| black_box(render(black_box(s), &options)).is_ok()).count();
        let done = run();
        let mut budget = 300.0;
        let mut samples = vec![];
        while budget > 0.0 || samples.len() < 5 {
            let started = Instant::now();
            black_box(run());
            let ms = started.elapsed().as_secs_f64() * 1000.0;
            samples.push(ms);
            budget -= ms;
            if samples.len() >= 2000 {
                break;
            }
        }
        samples.sort_by(f64::total_cmp);
        out.push(json!({"name":c["name"],"diagrams":inputs.len(),"ms":samples[samples.len()>>1],"done":done,"samples":samples}));
    }
    // Correctness check AFTER all timed workloads, avoiding extra pre-timing warm-ups.
    let verification: Vec<_> = cases
        .iter()
        .map(|c| {
            c["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| match render(s.as_str().unwrap(), &options) {
                    Ok(r) => json!({"svg":r.svg,"width":r.width,"height":r.height}),
                    Err(e) => json!({"error":e.message}),
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let enabled: Vec<_> = DiagramType::ALL.into_iter().filter(|t| type_enabled(*t)).map(|t| t.as_str()).collect();
    std::fs::write(&args[2], serde_json::to_string(&json!({"cases":out,"verification":verification,"enabledTypes":enabled})).unwrap())
        .unwrap();
}
