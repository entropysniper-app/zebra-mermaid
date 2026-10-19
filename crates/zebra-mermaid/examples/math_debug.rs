use std::collections::BTreeMap;
use zebra_mermaid::util::math;

fn main() {
    let text = std::fs::read_to_string("tests/fixtures/unit/math.jsonl").unwrap();
    let mut bad: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for line in text.lines() {
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        let name = v["fn"].as_str().unwrap().to_string();
        let args: Vec<f64> = v["args"].as_array().unwrap().iter().map(|a| a.as_f64().unwrap_or(f64::NAN)).collect();
        let r = match name.as_str() {
            "sin" => math::sin(args[0]),
            "cos" => math::cos(args[0]),
            "tan" => math::tan(args[0]),
            "atan" => math::atan(args[0]),
            "asin" => math::asin(args[0]),
            "acos" => math::acos(args[0]),
            "exp" => math::exp(args[0]),
            "log" => math::log(args[0]),
            "log10" => math::log10(args[0]),
            "log2" => math::log2(args[0]),
            "atan2" => math::atan2(args[0], args[1]),
            "pow" => math::pow(args[0], args[1]),
            "hypot" => math::hypot(args[0], args[1]),
            _ => continue,
        };
        let e = &mut bad.entry(name.clone()).or_default();
        e.1 += 1;
        if format!("{:016x}", r.to_bits()) != v["expect"].as_str().unwrap() && !r.is_nan() {
            e.0 += 1;
            if std::env::var("SHOW").is_ok_and(|s| s == name) {
                println!("{name}{args:?} rust={:016x} js={}", r.to_bits(), v["expect"]);
            }
        }
    }
    for (k, (b, n)) in bad {
        println!("{k}: {b}/{n}");
    }
    // std comparisons
    let x = 1155.2908457815647f64;
    println!("std sin {:016x} libm {:016x}", x.sin().to_bits(), libm_sin(x));
}
fn libm_sin(x: f64) -> u64 {
    math::sin(x).to_bits()
}
