//! Node String(number) reference around the integer fast path and notation boundaries.
#[test]
fn safe_integer_format_matches_javascript_reference() {
    for line in include_str!("../../../tests/fixtures/unit/js-safe-integers.jsonl").lines() {
        let row: serde_json::Value = serde_json::from_str(line).unwrap();
        let bits = u64::from_str_radix(row["bits"].as_str().unwrap(), 16).unwrap();
        assert_eq!(zebra_mermaid::util::js::to_string(f64::from_bits(bits)), row["expected"].as_str().unwrap(), "bits {bits:016x}");
    }
}
