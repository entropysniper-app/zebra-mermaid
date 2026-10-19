//! Native execution of original timed stress callbacks. No renderer output is a golden value.
use crate::Error;
use std::time::{Duration, Instant};

pub fn concat(mut left: String, right: String) -> String {
    left.push_str(&right);
    left
}
pub fn repeat_text(text: String, count: f64) -> String {
    assert!(count.is_finite() && count >= 0.0, "unsupported String.repeat count");
    text.repeat(count.trunc() as usize)
}
pub fn repeat_lines(count: f64, line: impl FnMut(f64) -> String) -> String {
    assert!(count.is_finite() && count >= 0.0, "unsupported Array.from length");
    // Array.from({length}, callback).join(''): retain every callback in order and join.
    (0..count.trunc() as usize).map(|i| i as f64).map(line).collect::<Vec<_>>().concat()
}
pub fn join_lines(count: f64, line: impl FnMut(f64) -> String, separator: &str) -> String {
    assert!(count.is_finite() && count >= 0.0, "unsupported Array.from length");
    (0..count.trunc() as usize).map(|index| index as f64).map(line).collect::<Vec<_>>().join(separator)
}
pub fn input_hash(source: &str, expected: u64) {
    let actual = source.bytes().fold(0xcbf29ce484222325_u64, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3));
    assert_eq!(actual, expected, "native input construction differs from the original JavaScript callback");
}
pub fn timed(limit_ms: u64, expected_input: u64, build: impl FnOnce() -> String, run: impl FnOnce(&str) -> Result<(), Error>) {
    let started = Instant::now();
    let source = build();
    // An Error value is the counterpart of instanceof PeleError. Panics are not caught.
    let _: Result<(), Error> = std::hint::black_box(run(&source));
    let elapsed = started.elapsed();
    eprintln!("stress_elapsed_ms={:.6}", elapsed.as_secs_f64() * 1000.0);
    // This extra construction-fidelity check is outside the original measured region.
    input_hash(&source, expected_input);
    assert!(elapsed < Duration::from_millis(limit_ms), "original <{limit_ms} ms threshold exceeded: {elapsed:?}");
}
pub fn bounded(timeout_ms: u64, run: impl FnOnce() + Send + 'static) {
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(run));
        let _ = send.send(result);
    });
    match receive.recv_timeout(Duration::from_millis(timeout_ms)) {
        Ok(Ok(())) => (),
        Ok(Err(error)) => std::panic::resume_unwind(error),
        Err(error) => panic!("Original upstream timeout {timeout_ms} ms: {error}"),
    }
}
