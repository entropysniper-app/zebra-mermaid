use serde_json::Value;
use zebra_mermaid::{render, RenderOptions};

pub fn workload() -> (String, Vec<String>, RenderOptions) {
    let name = std::env::args().nth(1).expect("workload name argument");
    let path = std::env::var_os("ZM_PROFILE_CASES").unwrap_or_else(|| "target/bench/cases.json".into());
    let cases: Vec<Value> = serde_json::from_slice(&std::fs::read(path).expect("generate benchmark cases first")).unwrap();
    let case = cases.iter().find(|case| case["name"] == name).expect("unknown workload");
    let options =
        RenderOptions { limit: (case["unlimitedInput"] == true).then_some(usize::MAX), now: Some(1792368000000.0), ..Default::default() };
    let inputs = case["inputs"].as_array().unwrap().iter().map(|s| s.as_str().unwrap().to_owned()).collect();
    (name, inputs, options)
}

pub fn batch(inputs: &[String], options: &RenderOptions) -> (usize, usize, usize) {
    let (mut rendered, mut rejected, mut bytes) = (0, 0, 0);
    for source in inputs {
        match std::hint::black_box(render(std::hint::black_box(source), options)) {
            Ok(result) => {
                rendered += 1;
                bytes += result.svg.len();
            }
            Err(_) => rejected += 1,
        }
    }
    (rendered, rejected, bytes)
}
