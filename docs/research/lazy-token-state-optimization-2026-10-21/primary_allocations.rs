#[path = "../../tools/profiler/src/allocator.rs"]
mod allocator;
use allocator::*;
use serde_json::{json, Value};
use std::sync::atomic::Ordering::Relaxed;
#[global_allocator]
static ALLOCATOR: Counted = Counted;
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let cases: Vec<Value> = serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    let case = cases.iter().find(|c| c["name"] == args[2]).unwrap();
    let inputs = case["inputs"].as_array().unwrap();
    let options = zebra_mermaid::RenderOptions {
        now: Some(1792368000000.0), font_size: Some(16.0), font_family: Some("Arial".into()),
        ..Default::default()
    };
    let batch = || {
        let mut bytes = 0;
        for input in inputs {
            let result = zebra_mermaid::render(std::hint::black_box(input["source"].as_str().unwrap()), &options).unwrap();
            bytes += std::hint::black_box(result).svg.len();
        }
        bytes
    };
    let expected = batch();
    for _ in 0..4 { assert_eq!(batch(), expected); }
    let baseline = LIVE.load(Relaxed);
    PEAK.store(baseline, Relaxed);
    TRACK.store(true, Relaxed);
    let actual = batch();
    TRACK.store(false, Relaxed);
    assert_eq!(actual, expected);
    // Snapshot before constructing the JSON report, which itself allocates memory.
    let allocations = ALLOCATIONS.load(Relaxed);
    let reallocations = REALLOCATIONS.load(Relaxed);
    let requested = BYTES.load(Relaxed);
    let peak = PEAK.load(Relaxed).saturating_sub(baseline);
    let retained = LIVE.load(Relaxed) as i128 - baseline as i128;
    println!("{}", json!({"case": case["name"], "inputsPerBatch": inputs.len(), "svgBytesPerBatch": actual,
        "allocations": allocations, "reallocations": reallocations, "requestedBytes": requested,
        "peakAdditionalLiveBytes": peak, "retainedBytesAfterBatch": retained}));
}
