#![allow(unused_parens)]
// Generated from the fixed original stress callbacks and input-building AST.
// Input construction and all iterations remain inside the original timed block.
use crate::upstream_stress as stress;
// Original Vitest cases are sequential within each source file.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn case_000() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xce04cedf8af75129, || { stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"A->>B: "#.to_owned()), stress::repeat_text(r#"word "#.to_owned(), (50000.0_f64 / 5.0_f64))) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_001() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xcf5f0b69afc7f7c3, || { stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"A->>B: wrap: "#.to_owned()), stress::repeat_text(r#"word "#.to_owned(), (50000.0_f64 / 5.0_f64))) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_002() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x75d287d3b8f02d06, || { stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"Note over A: wrap: "#.to_owned()), stress::repeat_text(r#"x"#.to_owned(), 50000.0_f64)) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_003() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x605429e11d1eb86a, || { stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"A->>B: "#.to_owned()), stress::repeat_text(r#"#35;<br/>"#.to_owned(), (50000.0_f64 / 9.0_f64))) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_004() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xaf10d3f9525bc8da, || { stress::concat(stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"participant "#.to_owned()), stress::repeat_text(r#"n"#.to_owned(), 50000.0_f64)), r#"
A->>B: x"#.to_owned()) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_005() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0xac744a003f91ec3b, || { stress::concat(stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"participant A"#.to_owned()), stress::repeat_text(r#" "#.to_owned(), 50000.0_f64)), r#"x"#.to_owned()) }, |source| { std::hint::black_box(super::lexer::tokenize(&source.encode_utf16().collect::<Vec<_>>())); Ok(()) });
    });
}

#[test]
fn case_006() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0xba93088246c18763, || { stress::concat(stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"participant a"#.to_owned()), stress::repeat_text(r#" #b"#.to_owned(), (50000.0_f64 / 3.0_f64))), r#":"#.to_owned()) }, |source| { std::hint::black_box(super::lexer::tokenize(&source.encode_utf16().collect::<Vec<_>>())); Ok(()) });
    });
}

#[test]
fn case_007() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0xaac0338013bf98d7, || { stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_text(r#"participant a b c d e f:
"#.to_owned(), (50000.0_f64 / 24.0_f64))) }, |source| { std::hint::black_box(super::lexer::tokenize(&source.encode_utf16().collect::<Vec<_>>())); Ok(()) });
    });
}

#[test]
fn case_008() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0xa8dbab03b8a33225, || { stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"participant a"#.to_owned()), stress::repeat_text(r#" as"#.to_owned(), (50000.0_f64 / 3.0_f64))) }, |source| { std::hint::black_box(super::lexer::tokenize(&source.encode_utf16().collect::<Vec<_>>())); Ok(()) });
    });
}

#[test]
fn case_009() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xfc25cc33684bc758, || { stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"participant A@{ "#.to_owned()), stress::repeat_text(r#""a": 1, "#.to_owned(), (50000.0_f64 / 8.0_f64))) }, |source| crate::parse(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_010() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xce2bf2575d8492d7, || { stress::concat(stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"participant A@{ "a": "#.to_owned()), stress::repeat_text(r#"["#.to_owned(), 50000.0_f64)), r#" }
A->>A: x"#.to_owned()) }, |source| crate::parse(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_011() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0xf1025cddbf9070fd, || { stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"participant A"#.to_owned()), stress::repeat_text(r#"@{}"#.to_owned(), (50000.0_f64 / 3.0_f64))) }, |source| { std::hint::black_box(super::lexer::tokenize(&source.encode_utf16().collect::<Vec<_>>())); Ok(()) });
    });
}

#[test]
fn case_012() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0x13261935f2f2d6a4, || { stress::concat(stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"A"#.to_owned()), stress::repeat_text(r#"-"#.to_owned(), 50000.0_f64)), r#">>B: x"#.to_owned()) }, |source| { std::hint::black_box(super::lexer::tokenize(&source.encode_utf16().collect::<Vec<_>>())); Ok(()) });
    });
}

#[test]
fn case_013() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0x7a57a35f0671f9d3, || { stress::concat(stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"A"#.to_owned()), stress::repeat_text(r#"-b"#.to_owned(), (50000.0_f64 / 2.0_f64))), r#"->>B: x"#.to_owned()) }, |source| { std::hint::black_box(super::lexer::tokenize(&source.encode_utf16().collect::<Vec<_>>())); Ok(()) });
    });
}

#[test]
fn case_014() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0xea87cdfcb1a39e03, || { stress::concat(stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"A"#.to_owned()), stress::repeat_text(r#"/(|)\"#.to_owned(), (50000.0_f64 / 5.0_f64))), r#"->>B: x"#.to_owned()) }, |source| { std::hint::black_box(super::lexer::tokenize(&source.encode_utf16().collect::<Vec<_>>())); Ok(()) });
    });
}

#[test]
fn case_015() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0xd53b870798418ca7, || { stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_text(r#"1"#.to_owned(), 50000.0_f64)), r#"x"#.to_owned()) }, |source| { std::hint::black_box(super::lexer::tokenize(&source.encode_utf16().collect::<Vec<_>>())); Ok(()) });
    });
}

#[test]
fn case_016() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0x873597cfa5048555, || { stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_text(r#"1.2."#.to_owned(), (50000.0_f64 / 4.0_f64))) }, |source| { std::hint::black_box(super::lexer::tokenize(&source.encode_utf16().collect::<Vec<_>>())); Ok(()) });
    });
}

#[test]
fn case_017() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0xf9d9b61f33f58c5, || { stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_text(stress::concat(stress::concat(r#"links"#.to_owned(), stress::repeat_text(r#" "#.to_owned(), 40.0_f64)), r#"
"#.to_owned()), (50000.0_f64 / 46.0_f64))) }, |source| { std::hint::black_box(super::lexer::tokenize(&source.encode_utf16().collect::<Vec<_>>())); Ok(()) });
    });
}

#[test]
fn case_018() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0xb170cd08f6a2655, || { stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_text(r#"title"#.to_owned(), (50000.0_f64 / 5.0_f64))) }, |source| { std::hint::black_box(super::lexer::tokenize(&source.encode_utf16().collect::<Vec<_>>())); Ok(()) });
    });
}

#[test]
fn case_019() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0x2b0370f6a4645a6e, || { stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"accDescr {"#.to_owned()), stress::repeat_text(r#" x"#.to_owned(), (50000.0_f64 / 2.0_f64))) }, |source| { std::hint::black_box(super::lexer::tokenize(&source.encode_utf16().collect::<Vec<_>>())); Ok(()) });
    });
}

#[test]
fn case_020() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x3dfdd35092d08fe5, || { stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_text(r#"accDescr{}"#.to_owned(), (50000.0_f64 / 10.0_f64))) }, |source| crate::parse(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_021() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0x6ca5340b585c5033, || { stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_text(r#"a%%"#.to_owned(), (50000.0_f64 / 3.0_f64))) }, |source| { std::hint::black_box(super::lexer::tokenize(&source.encode_utf16().collect::<Vec<_>>())); Ok(()) });
    });
}

#[test]
fn case_022() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0xaba7af79128fcd15, || { stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_text(r#" 
	"#.to_owned(), (50000.0_f64 / 3.0_f64))), r#"A->>B: x"#.to_owned()) }, |source| { std::hint::black_box(super::lexer::tokenize(&source.encode_utf16().collect::<Vec<_>>())); Ok(()) });
    });
}

#[test]
fn case_023() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x605396fadbba6193, || { stress::concat(stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"box rgb("#.to_owned()), stress::repeat_text(r#"1, "#.to_owned(), (50000.0_f64 / 3.0_f64))), r#") T
participant A
end"#.to_owned()) }, |source| crate::parse(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_024() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x171e12c3f5c1aef3, || { stress::concat(stress::concat(stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"box rgb"#.to_owned()), stress::repeat_text(r#" "#.to_owned(), (50000.0_f64 / 2.0_f64))), stress::repeat_text(r#"("#.to_owned(), (50000.0_f64 / 2.0_f64))), r#")
participant A
end"#.to_owned()) }, |source| crate::parse(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_025() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xa622769f13b6f23e, || { stress::concat(stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"participant A
links A: {"#.to_owned()), stress::repeat_lines(4000.0_f64, |_index| { stress::concat(stress::concat(r#""k"#.to_owned(), crate::util::js::to_string(_index)), r#"": "v", "#.to_owned()) })), r#""z": 1}"#.to_owned()) }, |source| crate::parse(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_026() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x70f86c47dbc86567, || { stress::concat(stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"participant A
links A: "#.to_owned()), stress::repeat_text(r#"["#.to_owned(), (50000.0_f64 / 2.0_f64))), stress::repeat_text(r#"]"#.to_owned(), (50000.0_f64 / 2.0_f64))) }, |source| crate::parse(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_027() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xeb755f8fc6c15cf0, || { stress::concat(stress::concat(stress::concat(stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"autonumber "#.to_owned()), stress::repeat_text(r#"9"#.to_owned(), (50000.0_f64 / 2.0_f64))), r#" "#.to_owned()), stress::repeat_text(r#"9"#.to_owned(), (50000.0_f64 / 2.0_f64))), r#"
A->>B: x
B->>A: y"#.to_owned()) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_028() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x200ef240e3644ad5, || { stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_text(r#"A->>B: x
"#.to_owned(), 8000.0_f64)) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_029() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x7844bb59b6e54c99, || { stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_lines(4000.0_f64, |_index| { stress::concat(stress::concat(r#"participant p"#.to_owned(), crate::util::js::to_string(_index)), r#"
"#.to_owned()) })) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_030() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xfc2c5ece806d482b, || { stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_lines(2500.0_f64, |_index| { stress::concat(stress::concat(stress::concat(stress::concat(r#"p"#.to_owned(), crate::util::js::to_string(_index)), r#"->>p"#.to_owned()), crate::util::js::to_string((2499.0_f64 - _index))), r#": m
"#.to_owned()) })) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_031() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x1eb3da8ecb96bb7f, || { stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_lines(1500.0_f64, |_index| { stress::concat(stress::concat(stress::concat(stress::concat(stress::concat(stress::concat(r#"p"#.to_owned(), crate::util::js::to_string(_index)), r#"->>p"#.to_owned()), crate::util::js::to_string(((_index * 7.0_f64) % 1500.0_f64))), r#": a long message "#.to_owned()), crate::util::js::to_string(_index)), r#"
"#.to_owned()) })) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_032() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xa649720582ff08ab, || { stress::concat(stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_text(r#"loop
"#.to_owned(), 10000.0_f64)), r#"A->>B: x
"#.to_owned()), stress::repeat_text(r#"end
"#.to_owned(), 10000.0_f64)) }, |source| crate::parse(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_033() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x92555983c8eebb0b, || { stress::concat(stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_lines(3000.0_f64, |_index| { (if ((_index % 2.0_f64) != 0.0) { r#"alt a
"#.to_owned() } else { r#"rect
"#.to_owned() }) })), r#"A->>B: x
"#.to_owned()), stress::repeat_text(r#"end
"#.to_owned(), 3000.0_f64)) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_034() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xae44ea0707e97292, || { stress::concat(stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"alt a
"#.to_owned()), stress::repeat_text(r#"else b
"#.to_owned(), 10000.0_f64)), r#"end
"#.to_owned()) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_035() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x6cf4c6d18296194, || { stress::concat(stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"par a
A->>B: x
"#.to_owned()), stress::repeat_text(r#"and b
B->>A: y
"#.to_owned(), 5000.0_f64)), r#"end
"#.to_owned()) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_036() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xd0a5cff2e16be58d, || { stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_text(r#"A->>+B: x
"#.to_owned(), 5000.0_f64)), stress::repeat_text(r#"B-->>-A: y
"#.to_owned(), 5000.0_f64)) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_037() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x76b4b49a3445fdad, || { stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_text(r#"activate A
"#.to_owned(), 4500.0_f64)) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_038() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xe2091615dd7e66d0, || { stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"participant A
participant B
participant C
"#.to_owned()), stress::repeat_text(r#"Note over A,C: n
"#.to_owned(), 3000.0_f64)) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_039() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xc13bc95c5eb2508d, || { stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_text(r#"A->>A: x
"#.to_owned(), 5000.0_f64)) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_040() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xd0b68b9ac2adb2d3, || { stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_lines(2000.0_f64, |_index| { stress::concat(stress::concat(r#"create participant c"#.to_owned(), crate::util::js::to_string(_index)), r#"
"#.to_owned()) })), stress::repeat_text(r#"Note over c0: n
"#.to_owned(), 500.0_f64)) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_041() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xbaa2127110d6f0b5, || { stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_lines(1000.0_f64, |_index| { stress::concat(stress::concat(stress::concat(stress::concat(stress::concat(stress::concat(stress::concat(stress::concat(r#"create participant c"#.to_owned(), crate::util::js::to_string(_index)), r#"
A->>c"#.to_owned()), crate::util::js::to_string(_index)), r#": x
destroy c"#.to_owned()), crate::util::js::to_string(_index)), r#"
A-xc"#.to_owned()), crate::util::js::to_string(_index)), r#": y
"#.to_owned()) })) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_042() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xda950e68309d855f, || { stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_lines(1500.0_f64, |_index| { stress::concat(stress::concat(stress::concat(stress::concat(r#"box Aqua Group "#.to_owned(), crate::util::js::to_string(_index)), r#"
participant b"#.to_owned()), crate::util::js::to_string(_index)), r#"
end
"#.to_owned()) })) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_043() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x580eaae430b0b49b, || { stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"autonumber
"#.to_owned()), stress::repeat_text(r#"A()->>()B: x
"#.to_owned(), 3000.0_f64)) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_044() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xd714feec5b5e2393, || { stress::concat(stress::concat(r#"sequenceDiagram
"#.to_owned(), r#"A->>B: x
"#.to_owned()), stress::repeat_text(r#"deactivate A
"#.to_owned(), 3000.0_f64)) }, |source| crate::parse(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_045() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x7ae434db55351595, || { stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_text(r#">"#.to_owned(), 50000.0_f64)) }, |source| crate::parse(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_046() { let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner()); stress::bounded(60000, || {
{ let source = stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_text(r#"A->>B: hello
"#.to_owned(), 5000.0_f64)); stress::input_hash(&source, 0xdc9617c6ae4e6915);
let error = crate::render(&source, &crate::RenderOptions::default()).unwrap_err();
assert!(error.message.contains("limit of 50000"), "{}", error.message);
}
{ let source = stress::concat(r#"sequenceDiagram
"#.to_owned(), stress::repeat_text(r#"A->>B: hello
"#.to_owned(), 5000.0_f64)); stress::input_hash(&source, 0xdc9617c6ae4e6915);
let result = crate::render(&source, &crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() }).unwrap();
assert!(result.svg.contains("<svg"));
}
}); }
