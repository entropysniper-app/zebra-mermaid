//! Four-engine comparison's native library driver; inputs and SVG checks are outside timing.
use serde_json::{json, Value};
use std::{fs, hint::black_box, time::Instant};

struct Options {
    zebra: zebra_mermaid::RenderOptions,
    mmdr: mermaid_rs_renderer::RenderOptions,
}
fn render(engine: &str, source: &str, options: &Options) -> Result<String, String> {
    match engine {
        "zebra" => zebra_mermaid::render(source, &options.zebra).map(|r| r.svg).map_err(|e| e.to_string()),
        "mmdr" => {
            // Keep its normal native font measurement; no fastText shortcut.
            // This API consumes options, so cloning the caller's configuration is included.
            mermaid_rs_renderer::render_with_options(source, options.mmdr.clone()).map_err(|e| e.to_string())
        }
        _ => panic!("unknown engine"),
    }
}

fn measure(engine: &str, inputs: &[Value], options: &Options) -> Value {
    let run = || {
        let mut bytes = 0;
        for input in inputs {
            bytes += black_box(render(engine, black_box(input["source"].as_str().unwrap()), options)).expect("verified input failed").len();
        }
        black_box(bytes);
    };
    for _ in 0..5 {
        run();
    }
    let mut repeats = 1;
    loop {
        let start = Instant::now();
        for _ in 0..repeats {
            run();
        }
        if start.elapsed().as_secs_f64() * 1000.0 >= 10.0 || repeats >= 131072 {
            break;
        }
        repeats *= 2;
    }
    let mut elapsed = 0.0;
    let mut samples = vec![];
    while elapsed < 300.0 || samples.len() < 9 {
        let start = Instant::now();
        for _ in 0..repeats {
            run();
        }
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        elapsed += ms;
        samples.push(ms / repeats as f64);
    }
    samples.sort_by(f64::total_cmp);
    json!({"ms":samples[(samples.len()-1)/2],"samples":samples,"repeats":repeats,"inputs":inputs.len()})
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let (engine, mode, input, output) = (&args[1], &args[2], &args[3], &args[4]);
    let cases: Vec<Value> = serde_json::from_slice(&fs::read(input).unwrap()).unwrap();
    let mut results = vec![];
    let mut theme = mermaid_rs_renderer::Theme::dark();
    theme.font_family = "Arial".into();
    theme.font_size = 16.0;
    let options = Options {
        zebra: zebra_mermaid::RenderOptions {
            now: Some(1792368000000.0),
            font_size: Some(16.0),
            font_family: Some("Arial".into()),
            ..Default::default()
        },
        mmdr: mermaid_rs_renderer::RenderOptions { theme, layout: Default::default() },
    };
    std::panic::set_hook(Box::new(|_| {}));
    for case in cases {
        let inputs = case["inputs"].as_array().unwrap();
        let result = if mode == "verify" {
            let rows: Vec<_> = inputs
                .iter()
                .map(|input| {
                    let source = input["source"].as_str().unwrap();
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| render(engine, source, &options))) {
                        Ok(Ok(svg)) => {
                            let actual_type = if engine == "mmdr" {
                                format!("{:?}", mermaid_rs_renderer::parse_mermaid_strict(source).unwrap().graph.kind)
                            } else {
                                zebra_mermaid::detect_type(source).unwrap().unwrap().as_str().to_owned()
                            };
                            json!({"id":input["id"],"ok":true,"svg":svg,"actualType":actual_type})
                        }
                        Ok(Err(error)) => json!({"id":input["id"],"ok":false,"error":error}),
                        Err(_) => json!({"id":input["id"],"ok":false,"error":"Renderer panicked"}),
                    }
                })
                .collect();
            json!({"name":case["name"],"results":rows})
        } else {
            json!({"name":case["name"],"timing":measure(engine,inputs,&options)})
        };
        results.push(result);
        println!("{engine} {mode} {}", case["name"]);
    }
    fs::write(output, serde_json::to_vec(&json!({"engine":engine,"cases":results})).unwrap()).unwrap();
}
