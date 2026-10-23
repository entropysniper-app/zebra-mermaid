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
        stress::timed(
            3000,
            0xc9fe6b88dadecff2,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(r#"A --> B "#.to_owned(), stress::repeat_text(r#"A "#.to_owned(), (50000.0_f64 / 2.0_f64))),
                        r#"direction XX"#.to_owned(),
                    ),
                )
            },
            |source| {
                std::hint::black_box(super::lexer::tokenize(source));
                Ok(())
            },
        );
    });
}

#[test]
fn case_001() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0xbb5ed5f3ef116402,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::repeat_text(
                        r#"direction
"#
                        .to_owned(),
                        (50000.0_f64 / 10.0_f64),
                    ),
                )
            },
            |source| {
                std::hint::black_box(super::lexer::tokenize(source));
                Ok(())
            },
        );
    });
}

#[test]
fn case_002() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0xcf52cd0c5ea1e3f6,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(
                            r#"direction"#.to_owned(),
                            stress::repeat_text(
                                r#"
 "#
                                .to_owned(),
                                (50000.0_f64 / 2.0_f64),
                            ),
                        ),
                        r#"XX
class A"#
                            .to_owned(),
                    ),
                )
            },
            |source| {
                std::hint::black_box(super::lexer::tokenize(source));
                Ok(())
            },
        );
    });
}

#[test]
fn case_003() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0x9462ac3009a7fede,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::repeat_text(r#"direction X "#.to_owned(), (50000.0_f64 / 12.0_f64)),
                )
            },
            |source| {
                std::hint::black_box(super::lexer::tokenize(source));
                Ok(())
            },
        );
    });
}

#[test]
fn case_004() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0x27e525ca1c49a2b2,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::repeat_text(r#"a "#.to_owned(), (50000.0_f64 / 2.0_f64)),
                )
            },
            |source| {
                std::hint::black_box(super::lexer::tokenize(source));
                Ok(())
            },
        );
    });
}

#[test]
fn case_005() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0xd28162ff0f03e6a7,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(r#"class A"#.to_owned(), stress::repeat_text(r#" "#.to_owned(), 50000.0_f64)),
                        r#"x"#.to_owned(),
                    ),
                )
            },
            |source| {
                std::hint::black_box(super::lexer::tokenize(source));
                Ok(())
            },
        );
    });
}

#[test]
fn case_006() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xdf9ff809340cb266,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::repeat_lines(2000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                stress::concat(
                                    stress::concat(r#"class C"#.to_owned(), crate::util::js::to_string(_index)),
                                    r#""#.to_owned(),
                                ),
                                stress::repeat_text(
                                    r#"
 	"#
                                    .to_owned(),
                                    8.0_f64,
                                ),
                            ),
                            r#"
"#
                            .to_owned(),
                        )
                    }),
                )
            },
            |source| {
                crate::parse(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_007() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0x39e78439501a05e3,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::repeat_text(r#"accTitle "#.to_owned(), (50000.0_f64 / 9.0_f64)),
                )
            },
            |source| {
                std::hint::black_box(super::lexer::tokenize(source));
                Ok(())
            },
        );
    });
}

#[test]
fn case_008() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0xa38c13cd6a26dc0,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(r#"accDescr"#.to_owned(), stress::repeat_text(r#" "#.to_owned(), 50000.0_f64)),
                )
            },
            |source| {
                std::hint::black_box(super::lexer::tokenize(source));
                Ok(())
            },
        );
    });
}

#[test]
fn case_009() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x9cd37ca421d8b261,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        r#"accDescr {"#.to_owned(),
                        stress::repeat_text(
                            r#"x
"#
                            .to_owned(),
                            (50000.0_f64 / 2.0_f64),
                        ),
                    ),
                )
            },
            |source| {
                crate::parse(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_010() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0x60cd1bc13d1c5f40,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(r#"click A call"#.to_owned(), stress::repeat_text(r#" "#.to_owned(), 50000.0_f64)),
                        r#"f()"#.to_owned(),
                    ),
                )
            },
            |source| {
                std::hint::black_box(super::lexer::tokenize(source));
                Ok(())
            },
        );
    });
}

#[test]
fn case_011() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xce21aca3450cf05b,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(r#"click A call f("#.to_owned(), stress::repeat_text(r#"a,"#.to_owned(), (50000.0_f64 / 2.0_f64))),
                )
            },
            |source| {
                crate::parse(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_012() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0xc37a4daac6f330f0,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(r#"click A call f("#.to_owned(), stress::repeat_text(r#" "#.to_owned(), 50000.0_f64)),
                        r#"x)"#.to_owned(),
                    ),
                )
            },
            |source| {
                std::hint::black_box(super::lexer::tokenize(source));
                Ok(())
            },
        );
    });
}

#[test]
fn case_013() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xee0871d86dbd0408,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(r#"note ""#.to_owned(), stress::repeat_text(r#"x"#.to_owned(), 50000.0_f64)),
                )
            },
            |source| {
                crate::parse(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_014() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x1c6fd689ba539801,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(r#"class A~"#.to_owned(), stress::repeat_text(r#"T"#.to_owned(), 50000.0_f64)),
                )
            },
            |source| {
                crate::parse(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_015() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xcba9538f6e633822,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(r#"class `"#.to_owned(), stress::repeat_text(r#"a"#.to_owned(), 50000.0_f64)),
                )
            },
            |source| {
                crate::parse(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_016() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0x115679277d99ed6b,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(r#"A "#.to_owned(), stress::repeat_text(r#":"#.to_owned(), 50000.0_f64)),
                )
            },
            |source| {
                std::hint::black_box(super::lexer::tokenize(source));
                Ok(())
            },
        );
    });
}

#[test]
fn case_017() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0xded8d41deeb211bb,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(r#"A "#.to_owned(), stress::repeat_text(r#":;"#.to_owned(), (50000.0_f64 / 2.0_f64))),
                )
            },
            |source| {
                std::hint::black_box(super::lexer::tokenize(source));
                Ok(())
            },
        );
    });
}

#[test]
fn case_018() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0x1a67c5a531036c01,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(
                            stress::concat(r#"A "#.to_owned(), stress::repeat_text(r#"-"#.to_owned(), (50000.0_f64 / 2.0_f64))),
                            stress::repeat_text(r#"."#.to_owned(), (50000.0_f64 / 2.0_f64)),
                        ),
                        r#" B"#.to_owned(),
                    ),
                )
            },
            |source| {
                std::hint::black_box(super::lexer::tokenize(source));
                Ok(())
            },
        );
    });
}

#[test]
fn case_019() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0xe74d70e797bbe69b,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(r#"A "#.to_owned(), stress::repeat_text(r#"<|"#.to_owned(), (50000.0_f64 / 4.0_f64))),
                        stress::repeat_text(r#"()"#.to_owned(), (50000.0_f64 / 4.0_f64)),
                    ),
                )
            },
            |source| {
                std::hint::black_box(super::lexer::tokenize(source));
                Ok(())
            },
        );
    });
}

#[test]
fn case_020() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xb1247d5c0dc72d5c,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(r#"class "#.to_owned(), stress::repeat_text(r#"A."#.to_owned(), (50000.0_f64 / 2.0_f64))),
                        r#"B"#.to_owned(),
                    ),
                )
            },
            |source| {
                crate::parse(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_021() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x7e37af703edbcba4,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(stress::repeat_text(r#"A "#.to_owned(), (50000.0_f64 / 2.0_f64)), r#"--> B"#.to_owned()),
                )
            },
            |source| {
                crate::parse(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_022() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xa9106d2de70f670d,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(r#"namespace "#.to_owned(), stress::repeat_text(r#"a."#.to_owned(), 12000.0_f64)),
                        r#"b {
class X
}"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::parse(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_023() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xd71d8d985f06141d,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(r#"namespace "#.to_owned(), stress::repeat_text(r#"a."#.to_owned(), 3000.0_f64)),
                        r#"b {
class X
}"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_024() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xfd4250f61c79d1b0,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(
                            stress::repeat_text(
                                r#"namespace n {
"#
                                .to_owned(),
                                4000.0_f64,
                            ),
                            r#"class X
"#
                            .to_owned(),
                        ),
                        stress::repeat_text(
                            r#"}
"#
                            .to_owned(),
                            4000.0_f64,
                        ),
                    ),
                )
            },
            |source| {
                crate::parse(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_025() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xab2dcfcf27138746,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(
                            stress::repeat_lines(2000.0_f64, |_index| {
                                stress::concat(
                                    stress::concat(r#"namespace n"#.to_owned(), crate::util::js::to_string(_index)),
                                    r#" {
"#
                                    .to_owned(),
                                )
                            }),
                            r#"class X
"#
                            .to_owned(),
                        ),
                        stress::repeat_text(
                            r#"}
"#
                            .to_owned(),
                            2000.0_f64,
                        ),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_026() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x45cffbf217a7ebf0,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::repeat_text(
                            r#"namespace n {
"#
                            .to_owned(),
                            4000.0_f64,
                        ),
                        r#"class X
"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::parse(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_027() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xce9409e6137483e6,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        r#"namespace n {
class X
"#
                        .to_owned(),
                        stress::repeat_text(r#"}"#.to_owned(), 50000.0_f64),
                    ),
                )
            },
            |source| {
                crate::parse(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_028() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xf8efa2dd61dbe598,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::repeat_text(r#"namespace "#.to_owned(), (50000.0_f64 / 10.0_f64)),
                        r#"x {
class X
}"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::parse(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_029() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xe837bebb7cee1d4,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                stress::concat(
                                    stress::concat(r#"namespace n"#.to_owned(), crate::util::js::to_string(_index)),
                                    r#" {
class C"#
                                        .to_owned(),
                                ),
                                crate::util::js::to_string(_index),
                            ),
                            r#"
}
"#
                            .to_owned(),
                        )
                    }),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_030() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x47da7253fd6aed8d,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(
                            r#"class A {
"#
                            .to_owned(),
                            stress::repeat_lines(10000.0_f64, |_index| {
                                stress::concat(
                                    stress::concat(r#"+m"#.to_owned(), crate::util::js::to_string(_index)),
                                    r#"()
"#
                                    .to_owned(),
                                )
                            }),
                        ),
                        r#"}
"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_031() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x17dba7d4b97a267e,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(
                            r#"class A {
+a)"#
                                .to_owned(),
                            stress::repeat_text(r#"("#.to_owned(), 50000.0_f64),
                        ),
                        r#"
}
"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_032() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xcfdd93be0de64217,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(
                            r#"class A {
"#
                            .to_owned(),
                            stress::repeat_text(r#"(a)"#.to_owned(), (50000.0_f64 / 3.0_f64)),
                        ),
                        r#"
}
"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_033() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xfd89294bb4899630,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(
                            r#"class A {
+x "#
                                .to_owned(),
                            stress::repeat_text(r#"~"#.to_owned(), 50000.0_f64),
                        ),
                        r#"
}
"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_034() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xdc9aa9618e513da9,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(
                            r#"class A {
+x("#
                                .to_owned(),
                            stress::repeat_text(r#"~a,"#.to_owned(), (50000.0_f64 / 3.0_f64)),
                        ),
                        r#")
}
"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_035() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0x263c28fdc8d9be19,
            || {
                stress::concat(
                    stress::repeat_text(stress::concat(r#"("#.to_owned(), "\u{d}".to_owned()), (50000.0_f64 / 2.0_f64)),
                    r#"a(b)"#.to_owned(),
                )
            },
            |source| {
                std::hint::black_box(super::Member::new(source, true));
                Ok(())
            },
        );
    });
}

#[test]
fn case_036() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0xdb45c8b9f1e9d0ed,
            || stress::concat(stress::concat(r#"~"#.to_owned(), stress::repeat_text(r#","#.to_owned(), 50000.0_f64)), r#"~"#.to_owned()),
            |source| {
                std::hint::black_box(crate::diagrams::common::generics::parse_generic_types(source));
                Ok(())
            },
        );
    });
}

#[test]
fn case_037() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x53cb7649e61f6a70,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(
                            r#"class A {
+"#
                            .to_owned(),
                            stress::repeat_text(r#"word "#.to_owned(), (50000.0_f64 / 5.0_f64)),
                        ),
                        r#"
}
"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_038() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x1ced40902abe13c,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(r#"note ""#.to_owned(), stress::repeat_text(r#"word "#.to_owned(), (50000.0_f64 / 5.0_f64))),
                        r#""
"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_039() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xab612eb971e5aafc,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(r#"note ""#.to_owned(), stress::repeat_text(r#"*a _b <br>"#.to_owned(), (50000.0_f64 / 10.0_f64))),
                        r#""
"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_040() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x8bbda2d82b1db626,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(r#"class A[""#.to_owned(), stress::repeat_text(r#"#35;&amp;"#.to_owned(), (50000.0_f64 / 9.0_f64))),
                        r#""]
"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_041() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xbe34dc3182a472ca,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(r#"class C"#.to_owned(), crate::util::js::to_string(_index)),
                            r#"
"#
                            .to_owned(),
                        )
                    }),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_042() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xe62131bf5d5e5fd0,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::repeat_lines(4000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                stress::concat(
                                    stress::concat(r#"C"#.to_owned(), crate::util::js::to_string(_index)),
                                    r#" <|-- C"#.to_owned(),
                                ),
                                crate::util::js::to_string((_index + 1.0_f64)),
                            ),
                            r#"
"#
                            .to_owned(),
                        )
                    }),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_043() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x345a57dd28981614,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::repeat_lines(4000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                stress::concat(
                                    stress::concat(r#"Hub "1" --> ""#.to_owned(), crate::util::js::to_string(_index)),
                                    r#"" C"#.to_owned(),
                                ),
                                crate::util::js::to_string(_index),
                            ),
                            r#"
"#
                            .to_owned(),
                        )
                    }),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_044() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xfa854dae0c9d0012,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::repeat_text(
                            r#"A "1" --> "2" B : x
"#
                            .to_owned(),
                            2000.0_f64,
                        ),
                        stress::repeat_text(
                            r#"A "1" --> "2" A : y
"#
                            .to_owned(),
                            2000.0_f64,
                        ),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_045() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x12db5e48a083a2a3,
            || {
                let mut s = r#""#.to_owned();
                let mut i = 0.0_f64;
                while (i < 50.0_f64) {
                    let mut j = (i + 1.0_f64);
                    while (j < 50.0_f64) {
                        s.push_str(&stress::concat(
                            stress::concat(
                                stress::concat(stress::concat(r#"C"#.to_owned(), crate::util::js::to_string(i)), r#" --> C"#.to_owned()),
                                crate::util::js::to_string(j),
                            ),
                            r#"
"#
                            .to_owned(),
                        ));
                        j += 1.0;
                    }
                    i += 1.0;
                }
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    s,
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_046() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x620827432fbb5ff6,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::repeat_lines(3000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(r#"i"#.to_owned(), crate::util::js::to_string(_index)),
                            r#" ()-- A
"#
                            .to_owned(),
                        )
                    }),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_047() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xa8893b6400767e7b,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        r#"class A
"#
                        .to_owned(),
                        stress::repeat_lines(3000.0_f64, |_index| {
                            stress::concat(
                                stress::concat(r#"note for A "n"#.to_owned(), crate::util::js::to_string(_index)),
                                r#""
"#
                                .to_owned(),
                            )
                        }),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_048() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x3a003e1273630b35,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(
                            r#"class A
style A fill:#fff"#
                                .to_owned(),
                            stress::repeat_text(r#",stroke:#000"#.to_owned(), 4000.0_f64),
                        ),
                        r#"
"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_049() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xe48041577ea47dd6,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::repeat_lines(100.0_f64, |_index| {
                            stress::concat(
                                stress::concat(r#"class C"#.to_owned(), crate::util::js::to_string(_index)),
                                r#":::k
"#
                                .to_owned(),
                            )
                        }),
                        stress::repeat_text(
                            r#"classDef k fill:#fff
"#
                            .to_owned(),
                            3000.0_f64,
                        ),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_050() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xe34457a480b39ca0,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::repeat_lines(1600.0_f64, |_index| {
                            stress::concat(
                                stress::concat(r#"class C"#.to_owned(), crate::util::js::to_string(_index)),
                                r#":::k
"#
                                .to_owned(),
                            )
                        }),
                        stress::repeat_text(
                            r#"classDef k fill:#fff
"#
                            .to_owned(),
                            1600.0_f64,
                        ),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_051() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xd6d4573afc61f71d,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(
                            stress::concat(
                                stress::repeat_lines(800.0_f64, |_index| {
                                    stress::concat(
                                        stress::concat(
                                            stress::concat(
                                                stress::concat(
                                                    stress::concat(
                                                        stress::concat(r#"class C"#.to_owned(), crate::util::js::to_string(_index)),
                                                        r#":::k
style C"#
                                                            .to_owned(),
                                                    ),
                                                    crate::util::js::to_string(_index),
                                                ),
                                                r#" x:"#.to_owned(),
                                            ),
                                            crate::util::js::to_string(_index),
                                        ),
                                        r#"
"#
                                        .to_owned(),
                                    )
                                }),
                                r#"classDef k x:y"#.to_owned(),
                            ),
                            stress::repeat_lines(5000.0_f64, |_index| {
                                stress::concat(stress::concat(r#",a"#.to_owned(), crate::util::js::to_string(_index)), r#":b"#.to_owned())
                            }),
                        ),
                        r#"
"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_052() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xc6616b4b7fe42ba2,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(
                            r#"class A
cssClass ""#
                                .to_owned(),
                            stress::repeat_text(r#"A,"#.to_owned(), 12000.0_f64),
                        ),
                        r#"A" k
"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_053() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x31fffaf40bc37c7a,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::concat(
                        stress::concat(
                            r#"class A
link A "&"#
                                .to_owned(),
                            stress::repeat_text(r#"#38;"#.to_owned(), (50000.0_f64 / 4.0_f64)),
                        ),
                        r#"x"
"#
                        .to_owned(),
                    ),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_054() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x18aa777cf8b3a252,
            || {
                stress::concat(
                    r#"classDiagram
"#
                    .to_owned(),
                    stress::repeat_lines(4000.0_f64, |_index| {
                        r#"<<a>> A
"#
                        .to_owned()
                    }),
                )
            },
            |source| {
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_055() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let source = stress::concat(
            r#"classDiagram
"#
            .to_owned(),
            stress::repeat_text(
                r#"class A
"#
                .to_owned(),
                10000.0_f64,
            ),
        );
        stress::input_hash(&source, 0xee88cc7755d19552);
        let error = crate::render(&source, &crate::RenderOptions::default()).unwrap_err();
        assert!(error.message.contains("limit of 50000"), "{}", error.message);
    });
}
