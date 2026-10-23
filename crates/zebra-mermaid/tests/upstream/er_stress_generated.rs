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
            0x1e064bc1aab876d0,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
"#
                        .to_owned(),
                        stress::repeat_text(r#"A "#.to_owned(), (50000.0_f64 / 2.0_f64)),
                    ),
                    r#"direction XX"#.to_owned(),
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
            0xdd78e333beb4a767,
            || {
                stress::concat(
                    r#"erDiagram
"#
                    .to_owned(),
                    stress::repeat_text(r#"direction "#.to_owned(), (50000.0_f64 / 10.0_f64)),
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
            0x59eaf5ddc6b212d4,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
"#
                        .to_owned(),
                        stress::repeat_text(
                            stress::concat(
                                r#"direction"#.to_owned(),
                                stress::repeat_text(
                                    r#"
"#
                                    .to_owned(),
                                    9.0_f64,
                                ),
                            ),
                            (50000.0_f64 / 18.0_f64),
                        ),
                    ),
                    r#"TB"#.to_owned(),
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
            0x8de537e9ff51813d,
            || {
                stress::concat(
                    r#"erDiagram
"#
                    .to_owned(),
                    stress::repeat_text(
                        r#"A direction B
"#
                        .to_owned(),
                        (50000.0_f64 / 14.0_f64),
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
fn case_004() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0xa454a42ebd1933d0,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
direction"#
                            .to_owned(),
                        stress::repeat_text(r#" "#.to_owned(), 50000.0_f64),
                    ),
                    r#"XX"#.to_owned(),
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
            0xdeb363341cf22166,
            || {
                stress::concat(
                    r#"erDiagram
"#
                    .to_owned(),
                    stress::repeat_text(
                        stress::concat(r#"accTitle"#.to_owned(), stress::repeat_text(r#" "#.to_owned(), 40.0_f64)),
                        (50000.0_f64 / 48.0_f64),
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
        stress::timed(
            3000,
            0x7119a7b124f2d902,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
"#
                        .to_owned(),
                        stress::repeat_text(
                            stress::concat(r#"1"#.to_owned(), stress::repeat_text(r#" "#.to_owned(), 99.0_f64)),
                            (50000.0_f64 / 100.0_f64),
                        ),
                    ),
                    r#"!"#.to_owned(),
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
fn case_007() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0x86c2f2742d8c1ac6,
            || {
                stress::concat(
                    r#"erDiagram
"#
                    .to_owned(),
                    stress::repeat_text(r#"|o}o}|o{|{||--.."#.to_owned(), (50000.0_f64 / 16.0_f64)),
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
            0x50b258b71dd81bc3,
            || {
                stress::concat(
                    r#"erDiagram
"#
                    .to_owned(),
                    stress::repeat_text(r#""a%"#.to_owned(), (50000.0_f64 / 3.0_f64)),
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
        stress::timed(
            3000,
            0x2471e61cdfbdbe8b,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"erDiagram
A {
"#
                            .to_owned(),
                            stress::repeat_text(r#"! "#.to_owned(), (50000.0_f64 / 4.0_f64)),
                        ),
                        r#"~"#.to_owned(),
                    ),
                    stress::repeat_text(r#" !"#.to_owned(), (50000.0_f64 / 4.0_f64)),
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
fn case_010() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0xa8727c80bad2f59f,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
A {
"#
                        .to_owned(),
                        stress::repeat_text(r#"!"#.to_owned(), 50000.0_f64),
                    ),
                    r#"~~"#.to_owned(),
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
        stress::timed(
            3000,
            0x4b167f5d4afe65ce,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
A {
"#
                        .to_owned(),
                        stress::repeat_text(
                            r#"a~ b
"#
                            .to_owned(),
                            (50000.0_f64 / 5.0_f64),
                        ),
                    ),
                    r#"}"#.to_owned(),
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
fn case_012() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0xd302f0350239fdbe,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
A {
"#
                        .to_owned(),
                        stress::repeat_text(r#"~a~ "#.to_owned(), (50000.0_f64 / 4.0_f64)),
                    ),
                    r#"}"#.to_owned(),
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
        stress::timed(
            3000,
            0xdb30fd2da2d20575,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
A {
"#
                        .to_owned(),
                        r#"`"#.to_owned(),
                    ),
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
fn case_014() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0x31958654d2b6845c,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
A {
"#
                        .to_owned(),
                        stress::repeat_text(r#"`a``b` "#.to_owned(), (50000.0_f64 / 7.0_f64)),
                    ),
                    r#"}"#.to_owned(),
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
fn case_015() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0xb28689b8aebe60c7,
            || {
                stress::concat(
                    r#"erDiagram
A {
"#
                    .to_owned(),
                    stress::repeat_text(r#"a b ""#.to_owned(), (50000.0_f64 / 5.0_f64)),
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
fn case_016() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0x44121ad6c7e8ad7f,
            || {
                stress::concat(
                    r#"erDiagram
accDescr {"#
                        .to_owned(),
                    stress::repeat_text(
                        r#" text
"#
                        .to_owned(),
                        (50000.0_f64 / 6.0_f64),
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
fn case_017() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xb43b0e89ea28dc14,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
A
style A "#
                            .to_owned(),
                        stress::repeat_text(r#":#"#.to_owned(), (50000.0_f64 / 2.0_f64)),
                    ),
                    r#"
"#
                    .to_owned(),
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
fn case_018() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xc6d159986caaa638,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
style "#
                            .to_owned(),
                        stress::repeat_lines(10000.0_f64, |_index| {
                            stress::concat(stress::concat(r#"e"#.to_owned(), crate::util::js::to_string(_index)), r#","#.to_owned())
                        }),
                    ),
                    r#"z fill:red
"#
                    .to_owned(),
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
fn case_019() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x82e5436178d46c4,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
A
style A "#
                            .to_owned(),
                        stress::repeat_lines(10000.0_f64, |_index| {
                            stress::concat(stress::concat(r#"x"#.to_owned(), crate::util::js::to_string(_index)), r#":1,"#.to_owned())
                        }),
                    ),
                    r#"fill:red
"#
                    .to_owned(),
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
fn case_020() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xc9bad5e61926d471,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            stress::concat(
                                r#"erDiagram
class "#
                                    .to_owned(),
                                stress::repeat_lines(10000.0_f64, |_index| {
                                    stress::concat(stress::concat(r#"e"#.to_owned(), crate::util::js::to_string(_index)), r#","#.to_owned())
                                }),
                            ),
                            r#"z "#.to_owned(),
                        ),
                        stress::repeat_lines(10000.0_f64, |_index| {
                            stress::concat(stress::concat(r#"c"#.to_owned(), crate::util::js::to_string(_index)), r#","#.to_owned())
                        }),
                    ),
                    r#"z
"#
                    .to_owned(),
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
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x4b91cb7ce511e510,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
A:::"#
                            .to_owned(),
                        stress::repeat_lines(20000.0_f64, |_index| {
                            stress::concat(stress::concat(r#"c"#.to_owned(), crate::util::js::to_string(_index)), r#","#.to_owned())
                        }),
                    ),
                    r#"z
"#
                    .to_owned(),
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
fn case_022() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x72fd2cc886efb4b8,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"erDiagram
"#
                            .to_owned(),
                            stress::repeat_text(
                                r#"subgraph s
"#
                                .to_owned(),
                                20000.0_f64,
                            ),
                        ),
                        r#"A
"#
                        .to_owned(),
                    ),
                    stress::repeat_text(
                        r#"end
"#
                        .to_owned(),
                        20000.0_f64,
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
            0x4be50878aae00652,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"erDiagram
"#
                            .to_owned(),
                            stress::repeat_lines(3000.0_f64, |_index| {
                                stress::concat(
                                    stress::concat(r#"subgraph s"#.to_owned(), crate::util::js::to_string(_index)),
                                    r#"
"#
                                    .to_owned(),
                                )
                            }),
                        ),
                        r#"A ||--|| B : x
"#
                        .to_owned(),
                    ),
                    stress::repeat_text(
                        r#"end
"#
                        .to_owned(),
                        3000.0_f64,
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
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x657c8a8b750aa745,
            || {
                stress::concat(
                    r#"erDiagram
"#
                    .to_owned(),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                stress::concat(
                                    stress::concat(r#"subgraph s"#.to_owned(), crate::util::js::to_string(_index)),
                                    r#"
n"#
                                    .to_owned(),
                                ),
                                crate::util::js::to_string(_index),
                            ),
                            r#"
end
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
fn case_025() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xa7b82fef30bba647,
            || {
                stress::concat(
                    r#"erDiagram
"#
                    .to_owned(),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        r#"subgraph s
A
B
C
end
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
fn case_026() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xe424bca6d10ab1f5,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
subgraph s
"#
                        .to_owned(),
                        stress::repeat_lines(10000.0_f64, |_index| {
                            stress::concat(
                                stress::concat(r#"n"#.to_owned(), crate::util::js::to_string(_index)),
                                r#"
"#
                                .to_owned(),
                            )
                        }),
                    ),
                    r#"end
"#
                    .to_owned(),
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
            0x6c213efe2c77e846,
            || {
                stress::concat(
                    r#"erDiagram
"#
                    .to_owned(),
                    stress::repeat_text(
                        r#"subgraph s
"#
                        .to_owned(),
                        (50000.0_f64 / 11.0_f64),
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
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xfcab3cbcc8801410,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
A {
"#
                        .to_owned(),
                        stress::repeat_lines(5000.0_f64, |_index| {
                            stress::concat(
                                stress::concat(
                                    stress::concat(
                                        stress::concat(r#"t"#.to_owned(), crate::util::js::to_string(_index)),
                                        r#" n"#.to_owned(),
                                    ),
                                    crate::util::js::to_string(_index),
                                ),
                                r#"
"#
                                .to_owned(),
                            )
                        }),
                    ),
                    r#"}"#.to_owned(),
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
fn case_029() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xb80213422a952559,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
A {
 string id "#
                            .to_owned(),
                        stress::repeat_text(r#"PK,"#.to_owned(), 10000.0_f64),
                    ),
                    r#"FK
}"#
                    .to_owned(),
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
            0x651cfd359c72fde3,
            || {
                stress::concat(
                    r#"erDiagram
"#
                    .to_owned(),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(r#"A { t n"#.to_owned(), crate::util::js::to_string(_index)),
                            r#" }
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
fn case_031() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x5dac53d827a2e696,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
A {
 string id ""#
                            .to_owned(),
                        stress::repeat_text(r#"word "#.to_owned(), (50000.0_f64 / 5.0_f64)),
                    ),
                    r#""
}"#
                    .to_owned(),
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
            0xad5aa81ecedf5e1,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
A {
 string "#
                            .to_owned(),
                        stress::repeat_text(r#"a"#.to_owned(), 50000.0_f64),
                    ),
                    r#"
}"#
                    .to_owned(),
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
            0x732cb4b8112de83d,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
A {
 "#
                        .to_owned(),
                        stress::repeat_text(r#"~"#.to_owned(), 50000.0_f64),
                    ),
                    r#" name
}"#
                    .to_owned(),
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
            0xe4901cacd5dfadb5,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
A {
 "#
                        .to_owned(),
                        stress::repeat_text(r#"a~b,"#.to_owned(), (50000.0_f64 / 4.0_f64)),
                    ),
                    r#" name
}"#
                    .to_owned(),
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
            0xd85b4db1fbc7c0d5,
            || {
                stress::concat(
                    stress::concat(
                        stress::repeat_text(r#"a~"#.to_owned(), (50000.0_f64 / 2.0_f64)),
                        stress::repeat_text(r#","#.to_owned(), 1000.0_f64),
                    ),
                    stress::repeat_text(r#"~b"#.to_owned(), (50000.0_f64 / 2.0_f64)),
                )
            },
            |source| {
                std::hint::black_box(crate::diagrams::common::generics::parse_generic_types(source));
                Ok(())
            },
        );
    });
}

#[test]
fn case_036() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x768082d92d9432fc,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
A {
 string id ""#
                            .to_owned(),
                        stress::repeat_text(r#"*a _b "#.to_owned(), (50000.0_f64 / 6.0_f64)),
                    ),
                    r#""
}"#
                    .to_owned(),
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
fn case_037() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xe1217eb2b908d4c5,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
""#
                        .to_owned(),
                        stress::repeat_text(r#"#35;&amp;<br>"#.to_owned(), (50000.0_f64 / 13.0_f64)),
                    ),
                    r#"""#.to_owned(),
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
            0xfb7d08a7637f236,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
A ||--o{ B : ""#
                            .to_owned(),
                        stress::repeat_text(r#"word "#.to_owned(), (50000.0_f64 / 5.0_f64)),
                    ),
                    r#"""#.to_owned(),
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
            0xcac182a49add8e99,
            || {
                stress::concat(
                    r#"erDiagram
"#
                    .to_owned(),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(r#"n"#.to_owned(), crate::util::js::to_string(_index)),
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
fn case_040() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x2989f9b577844114,
            || {
                stress::concat(
                    r#"erDiagram
"#
                    .to_owned(),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                stress::concat(
                                    stress::concat(r#"n"#.to_owned(), crate::util::js::to_string(_index)),
                                    r#" ||--o{ n"#.to_owned(),
                                ),
                                crate::util::js::to_string((_index + 1.0_f64)),
                            ),
                            r#" : r
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
fn case_041() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xad1af6e06df145b7,
            || {
                stress::concat(
                    r#"erDiagram
"#
                    .to_owned(),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(r#"c ||--o{ n"#.to_owned(), crate::util::js::to_string(_index)),
                            r#" : r
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
            0x99e8c959cac72017,
            || {
                stress::concat(
                    stress::concat(
                        r#"erDiagram
"#
                        .to_owned(),
                        stress::repeat_text(
                            r#"A ||--o{ B : r
"#
                            .to_owned(),
                            2000.0_f64,
                        ),
                    ),
                    stress::repeat_text(
                        r#"A ||--|| A : s
"#
                        .to_owned(),
                        2000.0_f64,
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
fn case_043() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x5e10e0e9ad144fde,
            || {
                let mut s = r#"erDiagram
"#
                .to_owned();
                let mut i = 0.0_f64;
                while (i < 50.0_f64) {
                    let mut j = (i + 1.0_f64);
                    while (j < 50.0_f64) {
                        s.push_str(&stress::concat(
                            stress::concat(
                                stress::concat(stress::concat(r#"n"#.to_owned(), crate::util::js::to_string(i)), r#" }o--o{ n"#.to_owned()),
                                crate::util::js::to_string(j),
                            ),
                            r#" : r
"#
                            .to_owned(),
                        ));
                        j += 1.0;
                    }
                    i += 1.0;
                }
                s
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
            0x2c5d963807ac6c91,
            || {
                stress::concat(
                    r#"erDiagram
subgraph a
A
end
subgraph b
B
end
"#
                    .to_owned(),
                    stress::repeat_text(
                        r#"a ||--|| b : r
A ||--|| b : s
"#
                        .to_owned(),
                        1000.0_f64,
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
            0xb8ebe01586d617f7,
            || {
                stress::concat(
                    r#"erDiagram
"#
                    .to_owned(),
                    stress::repeat_text(
                        r#"a["Some Alias"]
"#
                        .to_owned(),
                        10000.0_f64,
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
fn case_046() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let source = stress::concat(
            r#"erDiagram
"#
            .to_owned(),
            stress::repeat_text(
                r#"A ||--o{ B : r
"#
                .to_owned(),
                5000.0_f64,
            ),
        );
        stress::input_hash(&source, 0x620a9dde89db7f7f);
        let error = crate::render(&source, &crate::RenderOptions::default()).unwrap_err();
        assert!(error.message.contains("limit of 50000"), "{}", error.message);
    });
}
