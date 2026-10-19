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
stress::timed(3000, 0xa814f5a90317af18, || { stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_text(r#"a - contains -> b "#.to_owned(), (50000.0_f64 / 18.0_f64))), r#"direction XX"#.to_owned()) }, |source| { std::hint::black_box(super::lexer::tokenize(source)); Ok(()) });
    });
}

#[test]
fn case_001() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0x68aa1f0c250e417c, || { stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_text(r#": "#.to_owned(), (50000.0_f64 / 2.0_f64))), r#"direction LR"#.to_owned()) }, |source| { std::hint::black_box(super::lexer::tokenize(source)); Ok(()) });
    });
}

#[test]
fn case_002() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0x55e8c341bdb55e7b, || { stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_text(stress::concat(r#"direction"#.to_owned(), stress::repeat_text(r#"
"#.to_owned(), 40.0_f64)), (50000.0_f64 / 50.0_f64))), r#"TB"#.to_owned()) }, |source| { std::hint::black_box(super::lexer::tokenize(source)); Ok(()) });
    });
}

#[test]
fn case_003() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0x580cd53abe3ba813, || { stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_text(r#"direction "#.to_owned(), (50000.0_f64 / 10.0_f64))), r#"TB"#.to_owned()) }, |source| { std::hint::black_box(super::lexer::tokenize(source)); Ok(()) });
    });
}

#[test]
fn case_004() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0xdfc2a0403f0c4077, || { stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_text(r#"direction TB "#.to_owned(), (50000.0_f64 / 13.0_f64))) }, |source| { std::hint::black_box(super::lexer::tokenize(source)); Ok(()) });
    });
}

#[test]
fn case_005() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0xfe56430dda49fbeb, || { stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_text(stress::concat(stress::concat(r#"accTitle"#.to_owned(), stress::repeat_text(r#" "#.to_owned(), 100.0_f64)), r#"
"#.to_owned()), (50000.0_f64 / 110.0_f64))) }, |source| { std::hint::black_box(super::lexer::tokenize(source)); Ok(()) });
    });
}

#[test]
fn case_006() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0x4f62e2e4c35d9ffb, || { stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"accDescr"#.to_owned()), stress::repeat_text(r#" 
"#.to_owned(), (50000.0_f64 / 2.0_f64))) }, |source| { std::hint::black_box(super::lexer::tokenize(source)); Ok(()) });
    });
}

#[test]
fn case_007() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0x30b68cdeaac68dfc, || { stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"accDescr {"#.to_owned()), stress::repeat_text(r#" a
"#.to_owned(), (50000.0_f64 / 3.0_f64))) }, |source| { std::hint::black_box(super::lexer::tokenize(source)); Ok(()) });
    });
}

#[test]
fn case_008() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0x39533ac48b8c8539, || { stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_text(r#"title
"#.to_owned(), (50000.0_f64 / 6.0_f64))) }, |source| { std::hint::black_box(super::lexer::tokenize(source)); Ok(()) });
    });
}

#[test]
fn case_009() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0x48a3a6bdb7b22c6f, || { stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_text(r#"title #"#.to_owned(), (50000.0_f64 / 7.0_f64))) }, |source| { std::hint::black_box(super::lexer::tokenize(source)); Ok(()) });
    });
}

#[test]
fn case_010() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0xccdd63b1d57a099a, || { stress::concat(stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"a"#.to_owned()), stress::repeat_text(r#" "#.to_owned(), 50000.0_f64)), r#"b"#.to_owned()) }, |source| { std::hint::black_box(super::lexer::tokenize(source)); Ok(()) });
    });
}

#[test]
fn case_011() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0xad22020bd399c145, || { stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"element ""#.to_owned()), stress::repeat_text(r#"x"#.to_owned(), 50000.0_f64)) }, |source| { std::hint::black_box(super::lexer::tokenize(source)); Ok(()) });
    });
}

#[test]
fn case_012() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0x764c57ec489cad95, || { stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_text(r#"""#.to_owned(), 50000.0_f64)) }, |source| { std::hint::black_box(super::lexer::tokenize(source)); Ok(()) });
    });
}

#[test]
fn case_013() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0xd682ad519062aafd, || { stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_text(stress::concat(stress::concat(r#"a"#.to_owned(), stress::repeat_text(r#" 
	"#.to_owned(), 30.0_f64)), r#":"#.to_owned()), (50000.0_f64 / 100.0_f64))) }, |source| { std::hint::black_box(super::lexer::tokenize(source)); Ok(()) });
    });
}

#[test]
fn case_014() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
stress::timed(3000, 0xadeeb82e50e3bf79, || { stress::concat(stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"style a"#.to_owned()), stress::repeat_text(r#" "#.to_owned(), 50000.0_f64)), r#"fill:red
"#.to_owned()) }, |source| { std::hint::black_box(super::lexer::tokenize(source)); Ok(()) });
    });
}

#[test]
fn case_015() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x531adff1f4acbc43, || { stress::concat(stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"style a "#.to_owned()), stress::repeat_text(r#"f:#-;"#.to_owned(), (50000.0_f64 / 5.0_f64))), r#"
"#.to_owned()) }, |source| crate::parse(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_016() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x5ef73e659c94d4cf, || { stress::concat(stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"requirement a {
}
class a "#.to_owned()), stress::repeat_text(r#"c,"#.to_owned(), (50000.0_f64 / 2.0_f64))), r#"c
"#.to_owned()) }, |source| crate::parse(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_017() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xd5a5646aa541d39b, || { stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"requirement a {
}
"#.to_owned()), stress::repeat_text(r#"classDef c fill:red
class a c
"#.to_owned(), 1600.0_f64)) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_018() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x89f973b56e5ef70c, || { stress::concat(stress::concat(stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"requirement a {
}
classDef c "#.to_owned()), stress::repeat_text(r#"a:b,"#.to_owned(), 6000.0_f64)), r#"a:b
"#.to_owned()), stress::repeat_text(r#"class a c
"#.to_owned(), 2500.0_f64)) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_019() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xd658e5a4c959fbcb, || { stress::concat(stress::concat(stress::concat(stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_lines(1000.0_f64, |_index| { stress::concat(stress::concat(r#"element e"#.to_owned(), crate::util::js::to_string(_index)), r#" {
}
"#.to_owned()) })), r#"classDef c "#.to_owned()), stress::repeat_text(r#"a:b,"#.to_owned(), 3000.0_f64)), r#"a:b
"#.to_owned()), stress::repeat_text(stress::concat(stress::concat(r#"class "#.to_owned(), stress::join_lines(1000.0_f64, |array_index| { stress::concat(stress::concat(r#"e"#.to_owned(), crate::util::js::to_string(array_index)), r#""#.to_owned()) }, &r#","#.to_owned())), r#" c
"#.to_owned()), 3.0_f64)) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_020() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x3e9b64969a1befc3, || { stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_lines(2000.0_f64, |_index| { stress::concat(stress::concat(r#"element e"#.to_owned(), crate::util::js::to_string(_index)), r#":::c {
}
"#.to_owned()) })), stress::repeat_text(r#"classDef c a:b
"#.to_owned(), 600.0_f64)) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_021() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x35960cc558dc7d9b, || { stress::concat(stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"requirement a {
"#.to_owned()), stress::repeat_text(r#"id: 1
"#.to_owned(), 10000.0_f64)), r#"}
"#.to_owned()) }, |source| crate::parse(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_022() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x5a4751ae1c3498dc, || { stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"requirement a {
"#.to_owned()), stress::repeat_text(r#"text: x
"#.to_owned(), 6000.0_f64)) }, |source| crate::parse(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_023() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x6f84cbb908c4b983, || { stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_text(r#"
 
"#.to_owned(), (50000.0_f64 / 3.0_f64))), r#"requirement a {
}
"#.to_owned()) }, |source| crate::parse(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_024() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xfa79404a402f5db5, || { stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_text(r#"# c
% d
"#.to_owned(), (50000.0_f64 / 8.0_f64))) }, |source| crate::parse(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_025() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x55eeee87ca4e7b5b, || { stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_lines(2000.0_f64, |_index| { stress::concat(stress::concat(stress::concat(stress::concat(r#"requirement r"#.to_owned(), crate::util::js::to_string(_index)), r#" {
id: "#.to_owned()), crate::util::js::to_string(_index)), r#"
risk: low
}
"#.to_owned()) })) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_026() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xfb589c1fd7c3a668, || { stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_lines(1500.0_f64, |_index| { stress::concat(stress::concat(r#"element e"#.to_owned(), crate::util::js::to_string(_index)), r#" {
}
"#.to_owned()) })), stress::repeat_lines(1499.0_f64, |_index| { stress::concat(stress::concat(stress::concat(stress::concat(r#"e"#.to_owned(), crate::util::js::to_string(_index)), r#" - traces -> e"#.to_owned()), crate::util::js::to_string((_index + 1.0_f64))), r#"
"#.to_owned()) })) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_027() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x9c1c506e96b5c6e4, || { stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_lines(1500.0_f64, |_index| { stress::concat(stress::concat(r#"element e"#.to_owned(), crate::util::js::to_string(_index)), r#" {
}
"#.to_owned()) })), stress::repeat_lines(1499.0_f64, |_index| { stress::concat(stress::concat(r#"e0 - contains -> e"#.to_owned(), crate::util::js::to_string((_index + 1.0_f64))), r#"
"#.to_owned()) })) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_028() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x820dccf4adb1af1a, || { stress::concat(stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"element a {
}
element b {
}
"#.to_owned()), stress::repeat_text(r#"a - copies -> b
"#.to_owned(), 2500.0_f64)), stress::repeat_text(r#"a - contains -> a
"#.to_owned(), 500.0_f64)) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_029() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xdf3711cec718201c, || { stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"element a {
}
"#.to_owned()), stress::repeat_lines(2500.0_f64, |_index| { stress::concat(stress::concat(r#"a - refines -> x"#.to_owned(), crate::util::js::to_string(_index)), r#"
"#.to_owned()) })) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_030() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x580dc6cd438f82df, || { let mut s = stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_lines(40.0_f64, |_index| { stress::concat(stress::concat(r#"element e"#.to_owned(), crate::util::js::to_string(_index)), r#" {
}
"#.to_owned()) }));
let mut i = 0.0_f64;
while (i < 40.0_f64) {
let mut j = (i + 1.0_f64);
while (j < 40.0_f64) {
s.push_str(&stress::concat(stress::concat(stress::concat(stress::concat(r#"e"#.to_owned(), crate::util::js::to_string(i)), r#" - derives -> e"#.to_owned()), crate::util::js::to_string(j)), r#"
"#.to_owned()));
j += 1.0;
}
i += 1.0;
}
s }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_031() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0x43e5bbefa48b0dd2, || { stress::concat(stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"requirement a {
text: ""#.to_owned()), stress::repeat_text(r#"word "#.to_owned(), (50000.0_f64 / 5.0_f64))), r#""
}
"#.to_owned()) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_032() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xabd66088292ce472, || { stress::concat(stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"requirement a {
text: ""#.to_owned()), stress::repeat_text(r#"*a _b **c "#.to_owned(), (50000.0_f64 / 10.0_f64))), r#""
}
"#.to_owned()) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_033() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xd6b47abc10378b82, || { stress::concat(stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"requirement "#.to_owned()), stress::repeat_text(r#"x"#.to_owned(), 50000.0_f64)), r#" {
}
"#.to_owned()) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_034() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
stress::timed(3000, 0xc50317418b5a8d94, || { stress::concat(stress::concat(stress::concat(r#"requirementDiagram
"#.to_owned(), r#"requirement a {
text: ""#.to_owned()), stress::repeat_text(r#"#35;&amp;<br>"#.to_owned(), (50000.0_f64 / 13.0_f64))), r#""
}
"#.to_owned()) }, |source| crate::render(source, &options).map(|result| { std::hint::black_box(result); }));
    });
}

#[test]
fn case_035() { let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner()); stress::bounded(60000, || {
{ let source = stress::concat(r#"requirementDiagram
"#.to_owned(), stress::repeat_text(r#"element a {
}
"#.to_owned(), 5000.0_f64)); stress::input_hash(&source, 0xe18ed0c8927c4b45);
let error = crate::render(&source, &crate::RenderOptions::default()).unwrap_err();
assert!(error.message.contains("limit of 50000"), "{}", error.message);
}
}); }
