//! Independent pinned-reference measurements, compared by binary64 bits.
use zebra_mermaid::text::{MetricsMeasurer, TextMeasurer};

#[test]
fn default_metrics_match_pinned_reference_bits() {
    let data: serde_json::Value = serde_json::from_reader(flate2::read::GzDecoder::new(
        &include_bytes!("../../../tests/fixtures/unit/metrics-regression.json.gz")[..],
    ))
    .unwrap();
    assert_eq!(data["upstreamCommit"], zebra_mermaid::PELE_COMMIT);
    assert_eq!(data["seed"], "0x51a710");
    assert_eq!(data["randomScalars"], 4096);
    assert_eq!(data["randomStrings"], 128);
    let sizes: Vec<_> = data["sizes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| f64::from_bits(u64::from_str_radix(value.as_str().unwrap(), 16).unwrap()))
        .collect();
    let styles: Vec<_> = data["styles"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u8).collect();
    assert_eq!(sizes.len(), 7);
    assert_eq!(styles, [0, 1, 2, 3, 4, 5, 6, 7, 128, 255]);
    let sources = data["sources"].as_array().unwrap();
    let widths = data["widths"].as_array().unwrap();
    assert_eq!(sources.len(), widths.len());
    assert_eq!(sources.len(), 4427);
    for (source, expected) in sources.iter().zip(widths) {
        let source = source.as_str().unwrap();
        let expected = expected.as_array().unwrap();
        assert_eq!(expected.len(), styles.len() * sizes.len());
        for (style_index, &style) in styles.iter().enumerate() {
            for (size_index, &size) in sizes.iter().enumerate() {
                let bits = u64::from_str_radix(expected[style_index * sizes.len() + size_index].as_str().unwrap(), 16).unwrap();
                assert_eq!(MetricsMeasurer.width(source, size, style).to_bits(), bits, "{source:?}, size {size}, style {style}");
            }
        }
    }
}
