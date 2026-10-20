use std::{env, fs, hint::black_box, time::{Duration, Instant}};
use serde_json::{json, Value};
fn main() {
    let args: Vec<_> = env::args().collect();
    let cases: Vec<Value> = serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let case = cases.iter().find(|c| c["name"] == args[2]).unwrap();
    let inputs = case["inputs"].as_array().unwrap();
    let options = zebra_mermaid::RenderOptions {
        now: Some(1792368000000.0), font_size: Some(16.0), font_family: Some("Arial".into()),
        ..Default::default()
    };
    let batch = || {
        let mut bytes = 0usize;
        for input in inputs {
            let svg = zebra_mermaid::render(black_box(input["source"].as_str().unwrap()), &options).unwrap().svg;
            bytes += black_box(svg).len();
        }
        black_box(bytes)
    };
    for _ in 0..5 { batch(); }
    let bytes = batch();
    let deadline = Instant::now() + Duration::from_secs(args[3].parse().unwrap());
    let mut batches = 0;
    while Instant::now() < deadline { assert_eq!(batch(), bytes); batches += 1; }
    println!("{}", json!({"case":case["name"],"batches":batches,"inputsPerBatch":inputs.len(),"svgBytesPerBatch":bytes,"scope":"CPU sampling only; no latency statistics"}));
}
