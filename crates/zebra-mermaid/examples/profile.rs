//! Uninstrumented workload loop for an external CPU sampler and process memory measurement.
#[path = "support/profile.rs"]
mod support;
use serde_json::json;
use std::time::{Duration, Instant};
fn main() {
    let (name, inputs, options) = support::workload();
    let seconds: f64 = std::env::args().nth(2).unwrap_or_else(|| "2".into()).parse().unwrap();
    let expected = support::batch(&inputs, &options);
    for _ in 0..4 {
        assert_eq!(support::batch(&inputs, &options), expected);
    }
    let started = Instant::now();
    let mut batches = 0;
    while started.elapsed() < Duration::from_secs_f64(seconds) {
        assert_eq!(support::batch(&inputs, &options), expected);
        batches += 1;
    }
    println!(
        "{}",
        json!({"engine":"rust", "case":name, "batches":batches,
        "elapsedMs":started.elapsed().as_secs_f64()*1000.0,
        "renderedPerBatch":expected.0, "rejectedPerBatch":expected.1, "svgBytesPerBatch":expected.2})
    );
}
