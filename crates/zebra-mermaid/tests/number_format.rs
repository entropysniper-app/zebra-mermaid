//! Node String(number) reference around the integer fast path and notation boundaries.
#[test]
fn safe_integer_format_matches_javascript_reference() {
    for line in include_str!("../../../tests/fixtures/unit/js-safe-integers.jsonl").lines() {
        let row: serde_json::Value = serde_json::from_str(line).unwrap();
        let bits = u64::from_str_radix(row["bits"].as_str().unwrap(), 16).unwrap();
        assert_eq!(zebra_mermaid::util::js::to_string(f64::from_bits(bits)), row["expected"].as_str().unwrap(), "bits {bits:016x}");
    }
}

fn decimal_reference() -> serde_json::Value {
    let data: serde_json::Value = serde_json::from_reader(flate2::read::GzDecoder::new(
        &include_bytes!("../../../tests/fixtures/unit/js-decimal-regression.json.gz")[..],
    ))
    .unwrap();
    assert_eq!(data["seed"], "0x6d0d3e");
    assert_eq!(data["randomCases"], 4096);
    assert_eq!(data["cases"].as_array().unwrap().len(), 6905);
    data
}

#[test]
fn decimal_strings_match_javascript_across_exponents_and_random_bits() {
    let data = decimal_reference();
    for row in data["cases"].as_array().unwrap() {
        let bits = u64::from_str_radix(row["bits"].as_str().unwrap(), 16).unwrap();
        assert_eq!(zebra_mermaid::util::js::to_string(f64::from_bits(bits)), row["string"].as_str().unwrap(), "bits {bits:016x}");
    }
}

#[test]
fn significant_digits_match_javascript_through_one_hundred_digits() {
    let data = decimal_reference();
    for row in data["cases"].as_array().unwrap() {
        let bits = u64::from_str_radix(row["bits"].as_str().unwrap(), 16).unwrap();
        let value = f64::from_bits(bits);
        for reference in row["precision"].as_array().unwrap() {
            let precision = reference["precision"].as_u64().unwrap() as u32;
            assert_eq!(
                zebra_mermaid::util::js::to_precision(value, precision),
                reference["expected"].as_str().unwrap(),
                "bits {bits:016x}, precision {precision}"
            );
        }
    }
}

#[test]
fn fixed_digits_match_javascript_through_one_hundred_places() {
    let data = decimal_reference();
    for row in data["cases"].as_array().unwrap() {
        let bits = u64::from_str_radix(row["bits"].as_str().unwrap(), 16).unwrap();
        let value = f64::from_bits(bits);
        for reference in row["fixed"].as_array().unwrap() {
            let fraction = reference["fraction"].as_u64().unwrap() as u32;
            assert_eq!(
                zebra_mermaid::util::js::to_fixed(value, fraction),
                reference["expected"].as_str().unwrap(),
                "bits {bits:016x}, fraction {fraction}"
            );
        }
    }
}
