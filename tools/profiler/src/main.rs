//! Allocation instrumentation is isolated from normal timings and the production allocator.
#[path = "../../../crates/zebra-mermaid/examples/support/profile.rs"]
mod support;
use serde_json::json;
mod allocator;
use allocator::*;
use std::sync::atomic::Ordering::Relaxed;
#[global_allocator]
static ALLOCATOR: Counted = Counted;
fn main() {
    let (name, inputs, options) = support::workload();
    let expected = support::batch(&inputs, &options);
    for _ in 0..4 {
        assert_eq!(support::batch(&inputs, &options), expected);
    }
    let baseline = LIVE.load(Relaxed);
    PEAK.store(baseline, Relaxed);
    TRACK.store(true, Relaxed);
    let actual = support::batch(&inputs, &options);
    TRACK.store(false, Relaxed);
    let allocations = ALLOCATIONS.load(Relaxed);
    let reallocations = REALLOCATIONS.load(Relaxed);
    let bytes = BYTES.load(Relaxed);
    let peak = PEAK.load(Relaxed).saturating_sub(baseline);
    let retained = LIVE.load(Relaxed) as i128 - baseline as i128;
    assert_eq!(actual, expected);
    println!(
        "{}",
        json!({"engine":"rust", "case":name, "allocations":allocations,
        "reallocations":reallocations,"requestedBytes":bytes,"peakAdditionalLiveBytes":peak,
        "retainedBytesAfterBatch":retained,"renderedPerBatch":actual.0,"rejectedPerBatch":actual.1,
        "scope":"one batch after five warm-ups; inputs and options excluded; results dropped each iteration; requested allocator bytes, not RSS; realloc counts full new request"})
    );
}
