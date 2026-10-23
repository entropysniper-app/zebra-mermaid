#![allow(unused_parens)]
// Generated from the fixed original stress callbacks and input-building AST.
// Input construction and all iterations remain inside the original timed block.
use crate::upstream_stress as stress;
// Original Vitest cases are sequential within each source file.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn case_000() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x64861f817754bc16,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        r#"classDef hot fill:#f00,color:white,font-weight:bold,stroke-width:2px,stroke:yellow
"#
                        .to_owned(),
                    ),
                    stress::repeat_lines(1500.0_f64, |_index| {
                        r#"A --> B
B --> A
class A hot
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
fn case_001() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0x68af938d267f82ba,
            || {
                stress::concat(
                    r#"stateDiagram-v2
"#
                    .to_owned(),
                    stress::repeat_text(r#"a direction XX "#.to_owned(), (50000.0_f64 / 15.0_f64)),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0xc36fc9a146528666,
            || {
                stress::concat(
                    r#"stateDiagram-v2
"#
                    .to_owned(),
                    stress::repeat_text(r#"direction TB direction lr "#.to_owned(), (50000.0_f64 / 26.0_f64)),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0xcceb5a70ef3afc5f,
            || {
                stress::concat(
                    r#"stateDiagram-v2
"#
                    .to_owned(),
                    stress::repeat_text(
                        stress::concat(
                            r#"direction"#.to_owned(),
                            stress::repeat_text(
                                r#"
"#
                                .to_owned(),
                                24.0_f64,
                            ),
                        ),
                        (50000.0_f64 / 33.0_f64),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0xfa9fcd0055003441,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        stress::repeat_text(r#"a b "#.to_owned(), (50000.0_f64 / 4.0_f64)),
                    ),
                    r#"direction TB"#.to_owned(),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0x8048620149d6167a,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"stateDiagram-v2
"#
                            .to_owned(),
                            stress::repeat_text(r#"a "#.to_owned(), (50000.0_f64 / 2.0_f64)),
                        ),
                        r#"""#.to_owned(),
                    ),
                    stress::repeat_text(r#"b "#.to_owned(), (50000.0_f64 / 2.0_f64)),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0x103074b88d38b19a,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        r#"state "#.to_owned(),
                    ),
                    stress::repeat_text(r#"<<fork> [[join] "#.to_owned(), (50000.0_f64 / 16.0_f64)),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0x2e803bf7f8a30201,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        r#"state "#.to_owned(),
                    ),
                    stress::repeat_text(r#"x <<choice>> "#.to_owned(), (50000.0_f64 / 13.0_f64)),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0xe24a8b0d09310053,
            || {
                stress::concat(
                    r#"stateDiagram-v2
"#
                    .to_owned(),
                    stress::repeat_text(r#"state "#.to_owned(), (50000.0_f64 / 6.0_f64)),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0xef7e5d655147d05c,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"stateDiagram-v2
"#
                            .to_owned(),
                            r#"state a"#.to_owned(),
                        ),
                        stress::repeat_text(r#" "#.to_owned(), 50000.0_f64),
                    ),
                    r#"b"#.to_owned(),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0x998a8a95fbeed893,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        r#"state "#.to_owned(),
                    ),
                    stress::repeat_text(r#"w "#.to_owned(), (50000.0_f64 / 2.0_f64)),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0xecbd81ec607a05ce,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            stress::concat(
                                r#"stateDiagram-v2
"#
                                .to_owned(),
                                stress::repeat_lines(2000.0_f64, |_index| {
                                    r#"state a
"#
                                    .to_owned()
                                }),
                            ),
                            r#"state "#.to_owned(),
                        ),
                        stress::repeat_text(r#"w"#.to_owned(), (50000.0_f64 / 2.0_f64)),
                    ),
                    r#" x {"#.to_owned(),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0x16f055cfbc7065bc,
            || {
                stress::concat(
                    r#"stateDiagram-v2
"#
                    .to_owned(),
                    stress::repeat_text(r#"a%%{"#.to_owned(), (50000.0_f64 / 4.0_f64)),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0x9cfbc0426df3d7ab,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        r#"state "x" as "#.to_owned(),
                    ),
                    stress::repeat_text(r#"%%{"#.to_owned(), (50000.0_f64 / 3.0_f64)),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0x4f2832193988fa8a,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        r#"note right of a
"#
                        .to_owned(),
                    ),
                    stress::repeat_text(
                        r#"
 "#
                        .to_owned(),
                        (50000.0_f64 / 2.0_f64),
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
fn case_015() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0xdf2674244565c704,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        r#"note right of a
"#
                        .to_owned(),
                    ),
                    stress::repeat_text(
                        r#" end not
"#
                        .to_owned(),
                        (50000.0_f64 / 9.0_f64),
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
fn case_016() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0xe8b00dacf382b5f4,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"stateDiagram-v2
"#
                            .to_owned(),
                            r#"note left of"#.to_owned(),
                        ),
                        stress::repeat_text(r#" "#.to_owned(), 50000.0_f64),
                    ),
                    r#": x"#.to_owned(),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0x2f3b51afb4c7250a,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"stateDiagram-v2
"#
                            .to_owned(),
                            r#"note ""#.to_owned(),
                        ),
                        stress::repeat_text(r#" "#.to_owned(), 50000.0_f64),
                    ),
                    r#"x" as n"#.to_owned(),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0x550936cc243737b7,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        r#"class a"#.to_owned(),
                    ),
                    stress::repeat_text(r#", "#.to_owned(), (50000.0_f64 / 2.0_f64)),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0x79191ede798078a5,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        r#"a "#.to_owned(),
                    ),
                    stress::repeat_text(r#":x"#.to_owned(), (50000.0_f64 / 2.0_f64)),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0xeec3f6526d5a1584,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        stress::repeat_text(r#"-"#.to_owned(), (50000.0_f64 / 2.0_f64)),
                    ),
                    stress::repeat_text(r#":"#.to_owned(), (50000.0_f64 / 2.0_f64)),
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
fn case_021() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xba637a9a1d888b3,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        r#"accDescr {"#.to_owned(),
                    ),
                    stress::repeat_text(r#"x "#.to_owned(), (50000.0_f64 / 2.0_f64)),
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
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0x839d96c3a0a3ec4b,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"stateDiagram-v2
"#
                            .to_owned(),
                            r#"accTitle"#.to_owned(),
                        ),
                        stress::repeat_text(r#" "#.to_owned(), 50000.0_f64),
                    ),
                    r#"x"#.to_owned(),
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
fn case_023() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        stress::timed(
            3000,
            0x8173c959c2da9523,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"stateDiagram-v2
"#
                            .to_owned(),
                            r#"scale 1"#.to_owned(),
                        ),
                        stress::repeat_text(r#" "#.to_owned(), 50000.0_f64),
                    ),
                    r#"x"#.to_owned(),
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
fn case_024() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x51cedb660e591b2c,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        stress::repeat_lines(6000.0_f64, |_index| {
                            r#"state s{
"#
                            .to_owned()
                        }),
                    ),
                    stress::repeat_text(
                        r#"}
"#
                        .to_owned(),
                        6000.0_f64,
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
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x6438883736e1e2f1,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"stateDiagram-v2
"#
                            .to_owned(),
                            stress::repeat_lines(3000.0_f64, |_index| {
                                stress::concat(
                                    stress::concat(r#"state s"#.to_owned(), crate::util::js::to_string(_index)),
                                    r#"{
"#
                                    .to_owned(),
                                )
                            }),
                        ),
                        r#"a-->b
"#
                        .to_owned(),
                    ),
                    stress::repeat_text(
                        r#"}
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
fn case_026() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x7adba753cf4585c1,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"stateDiagram-v2
"#
                            .to_owned(),
                            stress::repeat_lines(2500.0_f64, |_index| {
                                stress::concat(
                                    stress::concat(
                                        stress::concat(
                                            stress::concat(r#"state s"#.to_owned(), crate::util::js::to_string(_index)),
                                            r#"{
x"#
                                            .to_owned(),
                                        ),
                                        crate::util::js::to_string(_index),
                                    ),
                                    r#"
--
"#
                                    .to_owned(),
                                )
                            }),
                        ),
                        r#"a
"#
                        .to_owned(),
                    ),
                    stress::repeat_text(
                        r#"}
"#
                        .to_owned(),
                        2500.0_f64,
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
fn case_027() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x1577677bcbf70e32,
            || {
                stress::concat(
                    r#"stateDiagram-v2
"#
                    .to_owned(),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(r#"state s"#.to_owned(), crate::util::js::to_string(_index)),
                            r#"{
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
fn case_028() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x919d6e362826befe,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"stateDiagram-v2
"#
                            .to_owned(),
                            r#"state s{
"#
                            .to_owned(),
                        ),
                        stress::repeat_lines(8000.0_f64, |_index| {
                            stress::concat(
                                stress::concat(r#"a"#.to_owned(), crate::util::js::to_string(_index)),
                                r#"
--
"#
                                .to_owned(),
                            )
                        }),
                    ),
                    r#"z
}
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
fn case_029() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xd46802ecf12797a1,
            || {
                stress::concat(
                    r#"stateDiagram-v2
"#
                    .to_owned(),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                stress::concat(stress::concat(r#"n"#.to_owned(), crate::util::js::to_string(_index)), r#"-->n"#.to_owned()),
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
fn case_030() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x38384df671844318,
            || {
                stress::concat(
                    r#"stateDiagram-v2
"#
                    .to_owned(),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(r#"c-->n"#.to_owned(), crate::util::js::to_string(_index)),
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
fn case_031() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x716bde428288f97f,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        r#"state f <<fork>>
"#
                        .to_owned(),
                    ),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(r#"f-->n"#.to_owned(), crate::util::js::to_string(_index)),
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
fn case_032() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x8dd1bacb6fc91ee2,
            || {
                let mut s = r#"stateDiagram-v2
"#
                .to_owned();
                let mut i = 0.0_f64;
                while (i < 60.0_f64) {
                    let mut j = (i + 1.0_f64);
                    while (j < 60.0_f64) {
                        s.push_str(&stress::concat(
                            stress::concat(
                                stress::concat(stress::concat(r#"n"#.to_owned(), crate::util::js::to_string(i)), r#"-->n"#.to_owned()),
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
fn case_033() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xd4900c78030113fc,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        stress::repeat_text(
                            r#"a-->b
"#
                            .to_owned(),
                            3000.0_f64,
                        ),
                    ),
                    stress::repeat_text(
                        r#"a-->a
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
fn case_034() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xf3f9d7b85a97f9a0,
            || {
                stress::concat(
                    r#"stateDiagram-v2
"#
                    .to_owned(),
                    stress::repeat_text(
                        r#"[*]-->[*]
"#
                        .to_owned(),
                        (50000.0_f64 / 12.0_f64),
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
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xa21c212993ecf4c2,
            || {
                stress::concat(
                    r#"stateDiagram-v2
"#
                    .to_owned(),
                    stress::repeat_lines(2500.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                r#"note "#.to_owned(),
                                (if ((_index % 2.0_f64) != 0.0) { r#"left"#.to_owned() } else { r#"right"#.to_owned() }),
                            ),
                            r#" of a:n
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
fn case_036() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xc8a44d6baa8ef7ae,
            || {
                stress::concat(
                    r#"stateDiagram-v2
"#
                    .to_owned(),
                    stress::repeat_lines(2000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(r#"note right of s"#.to_owned(), crate::util::js::to_string(_index)),
                            r#":n
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
fn case_037() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xe900d1854fcb667c,
            || {
                stress::concat(
                    r#"stateDiagram-v2
"#
                    .to_owned(),
                    stress::repeat_text(
                        r#"a:x
"#
                        .to_owned(),
                        (50000.0_f64 / 4.0_f64),
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
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xac42b2bc47d0937,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        r#"a : "#.to_owned(),
                    ),
                    stress::repeat_text(r#"word "#.to_owned(), (50000.0_f64 / 5.0_f64)),
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
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x9096f768357141cd,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"stateDiagram-v2
"#
                            .to_owned(),
                            r#"note right of a
"#
                            .to_owned(),
                        ),
                        stress::repeat_text(
                            r#"word *b _c
"#
                            .to_owned(),
                            (50000.0_f64 / 11.0_f64),
                        ),
                    ),
                    r#"end note
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
fn case_040() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xdebac9d91b27ce91,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        r#"a --> b : "#.to_owned(),
                    ),
                    stress::repeat_text(r#"#35;<br>x "#.to_owned(), (50000.0_f64 / 10.0_f64)),
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
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x51fd54a2860bec49,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"stateDiagram-v2
"#
                            .to_owned(),
                            r#"state ""#.to_owned(),
                        ),
                        stress::repeat_text(r#"w "#.to_owned(), (50000.0_f64 / 2.0_f64)),
                    ),
                    r#"" as s {
a
}
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
fn case_042() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xf9564cb3b6c662f9,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        r#"a
"#
                        .to_owned(),
                    ),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(r#"class a c"#.to_owned(), crate::util::js::to_string(_index)),
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
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x14cd53694cf97ddf,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"stateDiagram-v2
"#
                            .to_owned(),
                            r#"class "#.to_owned(),
                        ),
                        stress::repeat_lines(8000.0_f64, |_index| {
                            stress::concat(stress::concat(r#"s"#.to_owned(), crate::util::js::to_string(_index)), r#","#.to_owned())
                        }),
                    ),
                    r#"z c
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
fn case_044() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x7d41cc2f0cfb05e3,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"stateDiagram-v2
"#
                            .to_owned(),
                            r#"a
classDef c "#
                                .to_owned(),
                        ),
                        stress::repeat_text(r#"fill:red,"#.to_owned(), (50000.0_f64 / 9.0_f64)),
                    ),
                    r#"x
class a c
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
fn case_045() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xabafcd8962c8ebad,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        r#"a-->b
"#
                        .to_owned(),
                    ),
                    stress::repeat_text(
                        r#"style a,b fill:red
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
fn case_046() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xd304c6d92c7f9a6d,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        r#"a-->b
"#
                        .to_owned(),
                    ),
                    stress::repeat_text(
                        r#"click a "u" "t"
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
fn case_047() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xdc83e39ed2a212ce,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"stateDiagram-v2
"#
                            .to_owned(),
                            r#"a
click a "&"#
                                .to_owned(),
                        ),
                        stress::repeat_text(r#"#38;"#.to_owned(), (50000.0_f64 / 4.0_f64)),
                    ),
                    r#"x" "t"
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
fn case_048() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xd757d90361ea449a,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            stress::concat(
                                r#"stateDiagram-v2
"#
                                .to_owned(),
                                stress::repeat_lines(1500.0_f64, |_index| {
                                    stress::concat(
                                        stress::concat(r#"state s"#.to_owned(), crate::util::js::to_string(_index)),
                                        r#"{
"#
                                        .to_owned(),
                                    )
                                }),
                            ),
                            r#"z
"#
                            .to_owned(),
                        ),
                        stress::repeat_text(
                            r#"}
"#
                            .to_owned(),
                            1500.0_f64,
                        ),
                    ),
                    stress::repeat_lines(1500.0_f64, |_index| {
                        stress::concat(
                            stress::concat(r#"s"#.to_owned(), crate::util::js::to_string(_index)),
                            r#"-->z
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
fn case_049() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x5cb9f9ffbd49ded2,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            stress::concat(
                                r#"stateDiagram-v2
"#
                                .to_owned(),
                                stress::repeat_lines(1500.0_f64, |_index| {
                                    stress::concat(
                                        stress::concat(r#"state s"#.to_owned(), crate::util::js::to_string(_index)),
                                        r#"{
"#
                                        .to_owned(),
                                    )
                                }),
                            ),
                            r#"z
"#
                            .to_owned(),
                        ),
                        stress::repeat_text(
                            r#"}
"#
                            .to_owned(),
                            1500.0_f64,
                        ),
                    ),
                    stress::repeat_lines(3000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(r#"o"#.to_owned(), crate::util::js::to_string(_index)),
                            r#"-->z
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
fn case_050() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xcfaeada69b061e02,
            || {
                stress::concat(
                    r#"stateDiagram-v2
"#
                    .to_owned(),
                    stress::repeat_lines(2000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                stress::concat(
                                    stress::concat(r#"state a"#.to_owned(), crate::util::js::to_string(_index)),
                                    r#"{
state a"#
                                        .to_owned(),
                                ),
                                crate::util::js::to_string(((_index + 1.0_f64) % 2000.0_f64)),
                            ),
                            r#"{
}
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
fn case_051() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xb2bf1a49e854e522,
            || {
                stress::concat(
                    stress::concat(
                        r#"stateDiagram-v2
"#
                        .to_owned(),
                        stress::repeat_lines(1200.0_f64, |_index| {
                            stress::concat(
                                stress::concat(
                                    stress::concat(
                                        stress::concat(
                                            stress::concat(
                                                stress::concat(r#"state a"#.to_owned(), crate::util::js::to_string(_index)),
                                                r#"{
state a"#
                                                    .to_owned(),
                                            ),
                                            crate::util::js::to_string(((_index + 1.0_f64) % 1200.0_f64)),
                                        ),
                                        r#"{
z"#
                                        .to_owned(),
                                    ),
                                    crate::util::js::to_string(_index),
                                ),
                                r#"
}
}
"#
                                .to_owned(),
                            )
                        }),
                    ),
                    stress::repeat_lines(3000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                stress::concat(stress::concat(r#"o"#.to_owned(), crate::util::js::to_string(_index)), r#"-->z"#.to_owned()),
                                crate::util::js::to_string((_index % 1200.0_f64)),
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
fn case_052() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xfc2a8644387e111c,
            || {
                stress::concat(
                    r#"stateDiagram-v2
"#
                    .to_owned(),
                    r#"state root {
 a --> b
}
root --> root
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
