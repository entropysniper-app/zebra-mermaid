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
            0x1fc9220ebe2640f4,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
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
            0xfa6924c8b24ef223,
            || {
                stress::concat(
                    r#"agentflow-beta TB
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
            0x90838eb1d59cd960,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
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
            0xea2e1524b869cfb1,
            || {
                stress::concat(
                    r#"agentflow-beta TB
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
            0xe9a20d620bbf0bc,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"direction"#.to_owned(),
                        ),
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
            0xb7b3fbaea76e9b16,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
"#
                        .to_owned(),
                        stress::repeat_text(r#" "#.to_owned(), 50000.0_f64),
                    ),
                    r#"a"#.to_owned(),
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
            0xf0640290c453afb9,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"a"#.to_owned(),
                        ),
                        stress::repeat_text(r#" "#.to_owned(), 50000.0_f64),
                    ),
                    r#"-b"#.to_owned(),
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
            0x2db8eb265060e8b5,
            || {
                stress::concat(
                    r#"agentflow-beta TB
"#
                    .to_owned(),
                    stress::repeat_text(
                        stress::concat(
                            stress::concat(r#"a"#.to_owned(), stress::repeat_text(r#" "#.to_owned(), 99.0_f64)),
                            r#"-b
"#
                            .to_owned(),
                        ),
                        (50000.0_f64 / 103.0_f64),
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
fn case_008() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0xcdf65d8c3b986a61,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            stress::concat(
                                r#"agentflow-beta TB
"#
                                .to_owned(),
                                r#"a"#.to_owned(),
                            ),
                            stress::repeat_text(r#" "#.to_owned(), (50000.0_f64 / 2.0_f64)),
                        ),
                        r#"-"#.to_owned(),
                    ),
                    stress::repeat_text(r#"."#.to_owned(), (50000.0_f64 / 2.0_f64)),
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
            0x5241f6daf02feec4,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            stress::concat(
                                r#"agentflow-beta TB
"#
                                .to_owned(),
                                r#"a"#.to_owned(),
                            ),
                            stress::repeat_text(r#" "#.to_owned(), (50000.0_f64 / 2.0_f64)),
                        ),
                        stress::repeat_text(r#"-"#.to_owned(), (50000.0_f64 / 2.0_f64)),
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
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0x96570fc681b63c6e,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
"#
                        .to_owned(),
                        stress::repeat_text(r#" "#.to_owned(), 50000.0_f64),
                    ),
                    r#"%x"#.to_owned(),
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
            0xe3fa9fb47dd75b2,
            || {
                stress::concat(
                    r#"agentflow-beta TB
"#
                    .to_owned(),
                    stress::repeat_text(
                        stress::concat(
                            stress::repeat_text(r#" "#.to_owned(), 99.0_f64),
                            r#"%%{
"#
                            .to_owned(),
                        ),
                        (50000.0_f64 / 103.0_f64),
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
fn case_012() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0x92982835f6f7727f,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"a -- "#.to_owned(),
                        ),
                        stress::repeat_text(r#" "#.to_owned(), 50000.0_f64),
                    ),
                    r#"-b"#.to_owned(),
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
            0x48b2eda1a527f230,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
"#
                        .to_owned(),
                        r#"a -- "#.to_owned(),
                    ),
                    stress::repeat_text(r#" -"#.to_owned(), (50000.0_f64 / 2.0_f64)),
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
            0xbf9831a41d5b24b0,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"a -- x"#.to_owned(),
                        ),
                        stress::repeat_text(r#" "#.to_owned(), (50000.0_f64 / 2.0_f64)),
                    ),
                    stress::repeat_text(r#"-"#.to_owned(), (50000.0_f64 / 2.0_f64)),
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
            0x76ab0320055a7e6,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"a -- "#.to_owned(),
                        ),
                        stress::repeat_text(r#"x "#.to_owned(), (50000.0_f64 / 2.0_f64)),
                    ),
                    r#"--> b"#.to_owned(),
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
            0xc04e4fc65bf35553,
            || {
                stress::concat(
                    r#"agentflow-beta TB
"#
                    .to_owned(),
                    stress::repeat_text(r#"a@"#.to_owned(), (50000.0_f64 / 2.0_f64)),
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
            0x8991e0938769e318,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
"#
                        .to_owned(),
                        stress::repeat_text(r#"@"#.to_owned(), 50000.0_f64),
                    ),
                    r#"{"#.to_owned(),
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
            0x31c48a765cf670ef,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
"#
                        .to_owned(),
                        r#"a[""#.to_owned(),
                    ),
                    stress::repeat_text(r#"x%"#.to_owned(), (50000.0_f64 / 2.0_f64)),
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
            0x8def5ae6dfb68d9d,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
"#
                        .to_owned(),
                        r#"a["`"#.to_owned(),
                    ),
                    stress::repeat_text(r#"x "#.to_owned(), (50000.0_f64 / 2.0_f64)),
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
        stress::timed(
            3000,
            0xf8f7b5ed0d38699c,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"a["#.to_owned(),
                        ),
                        stress::repeat_text(
                            r#"x %% c
"#
                            .to_owned(),
                            (50000.0_f64 / 7.0_f64),
                        ),
                    ),
                    r#"]"#.to_owned(),
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
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0x7da8bd1c23b3ae55,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"a(-"#.to_owned(),
                        ),
                        stress::repeat_text(
                            r#"x %% c
"#
                            .to_owned(),
                            (50000.0_f64 / 7.0_f64),
                        ),
                    ),
                    r#"-)"#.to_owned(),
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
fn case_022() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0x7a841226d908640a,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"a[/"#.to_owned(),
                        ),
                        stress::repeat_text(
                            r#"x %% c
"#
                            .to_owned(),
                            (50000.0_f64 / 7.0_f64),
                        ),
                    ),
                    r#"/]"#.to_owned(),
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
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0x5a68305356a3f9b6,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
"#
                        .to_owned(),
                        r#"a"#.to_owned(),
                    ),
                    stress::repeat_text(r#"[("#.to_owned(), (50000.0_f64 / 2.0_f64)),
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
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0x5b83d10b90db4f97,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
"#
                        .to_owned(),
                        r#"a@{"#.to_owned(),
                    ),
                    stress::repeat_text(r#" k: "v","#.to_owned(), (50000.0_f64 / 8.0_f64)),
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
fn case_025() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0xcdca71ec626cbe52,
            || {
                stress::concat(
                    r#"agentflow-beta TB
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
fn case_026() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        stress::timed(
            3000,
            0xfa710ab04eb1d373,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
"#
                        .to_owned(),
                        r#"accDescr {"#.to_owned(),
                    ),
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
fn case_027() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x1bd20e5a9724e7a,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
"#
                        .to_owned(),
                        stress::repeat_text(
                            r#"  %% note
"#
                            .to_owned(),
                            10000.0_f64,
                        ),
                    ),
                    r#"a --> b
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
fn case_028() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xd9e550699980dc13,
            || {
                stress::concat(
                    r#"agentflow-beta TB
"#
                    .to_owned(),
                    stress::repeat_text(
                        stress::concat(r#"}"#.to_owned(), stress::repeat_text(r#" "#.to_owned(), 49.0_f64)),
                        (50000.0_f64 / 50.0_f64),
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
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xc4dd351d441a8770,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"A
style A "#
                                .to_owned(),
                        ),
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
fn case_030() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x4815965ae3c50068,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"A
style A "#
                                .to_owned(),
                        ),
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
fn case_031() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x629b3be30dd1a855,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            stress::concat(
                                stress::concat(
                                    r#"agentflow-beta TB
"#
                                    .to_owned(),
                                    r#"class "#.to_owned(),
                                ),
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
fn case_032() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x2eb564fe7139cafc,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"A:::"#.to_owned(),
                        ),
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
fn case_033() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xd03566f92efb0762,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"a --> b
linkStyle "#
                                .to_owned(),
                        ),
                        stress::repeat_text(r#"0,"#.to_owned(), 10000.0_f64),
                    ),
                    r#"0 stroke:red
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
fn case_034() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x704c7009b852d54,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            stress::repeat_text(
                                r#"flow s
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
fn case_035() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x4889b1e9f825abf2,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            stress::repeat_lines(3000.0_f64, |_index| {
                                stress::concat(
                                    stress::concat(r#"flow s"#.to_owned(), crate::util::js::to_string(_index)),
                                    r#"
"#
                                    .to_owned(),
                                )
                            }),
                        ),
                        r#"A --> B
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
fn case_036() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x898508beaa73f01a,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            stress::repeat_lines(3000.0_f64, |_index| {
                                stress::concat(
                                    stress::concat(r#"flow s"#.to_owned(), crate::util::js::to_string(_index)),
                                    r#"@{ view: collapsed }
"#
                                    .to_owned(),
                                )
                            }),
                        ),
                        r#"A --> B
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
fn case_037() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x16fadc125cc7a18e,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            stress::concat(
                                r#"agentflow-beta TB
"#
                                .to_owned(),
                                stress::repeat_lines(1500.0_f64, |_index| {
                                    stress::concat(
                                        stress::concat(r#"flow s"#.to_owned(), crate::util::js::to_string(_index)),
                                        r#"
"#
                                        .to_owned(),
                                    )
                                }),
                            ),
                            r#"A
"#
                            .to_owned(),
                        ),
                        stress::repeat_text(
                            r#"end
"#
                            .to_owned(),
                            1500.0_f64,
                        ),
                    ),
                    stress::repeat_text(
                        r#"A --> B
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
fn case_038() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xaa1a2634ab4f7ed5,
            || {
                stress::concat(
                    r#"agentflow-beta TB
"#
                    .to_owned(),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                stress::concat(
                                    stress::concat(r#"flow s"#.to_owned(), crate::util::js::to_string(_index)),
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
fn case_039() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x7480a0942c909833,
            || {
                stress::concat(
                    r#"agentflow-beta TB
"#
                    .to_owned(),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        r#"flow s
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
fn case_040() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x53a6a32fd3652d2f,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"flow s
"#
                            .to_owned(),
                        ),
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
fn case_041() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::ParseOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x848fd7fb8b11c995,
            || {
                stress::concat(
                    r#"agentflow-beta TB
"#
                    .to_owned(),
                    stress::repeat_text(
                        r#"flow s
"#
                        .to_owned(),
                        (50000.0_f64 / 7.0_f64),
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
fn case_042() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x8c4a30308c5faa,
            || {
                stress::concat(
                    r#"agentflow-beta TB
"#
                    .to_owned(),
                    stress::repeat_lines(3000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                stress::concat(
                                    stress::concat(
                                        stress::concat(
                                            stress::concat(r#"flow f"#.to_owned(), crate::util::js::to_string(_index)),
                                            r#"
 a"#
                                            .to_owned(),
                                        ),
                                        crate::util::js::to_string(_index),
                                    ),
                                    r#" --> f"#.to_owned(),
                                ),
                                crate::util::js::to_string((_index + 1.0_f64)),
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
fn case_043() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xae3e9791780f0613,
            || {
                stress::concat(
                    r#"agentflow-beta TB
"#
                    .to_owned(),
                    stress::repeat_lines(2000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                stress::concat(
                                    stress::concat(
                                        stress::concat(
                                            stress::concat(r#"flow f"#.to_owned(), crate::util::js::to_string(_index)),
                                            r#"
 a"#
                                            .to_owned(),
                                        ),
                                        crate::util::js::to_string(_index),
                                    ),
                                    r#" --> f"#.to_owned(),
                                ),
                                crate::util::js::to_string((if (_index == 0.0_f64) { 1999.0_f64 } else { (_index - 1.0_f64) })),
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
fn case_044() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xba7421c27477c0d4,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            stress::concat(
                                r#"agentflow-beta TB
"#
                                .to_owned(),
                                r#"flow f
"#
                                .to_owned(),
                            ),
                            stress::repeat_lines(3000.0_f64, |_index| {
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
                    ),
                    stress::repeat_lines(3000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                r#"global
n"#
                                .to_owned(),
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
fn case_045() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xe188a9d2d03fdae7,
            || {
                stress::concat(
                    r#"agentflow-beta TB
"#
                    .to_owned(),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(r#"connector c"#.to_owned(), crate::util::js::to_string(_index)),
                            r#"["C"]@{ protocol: "mcp" }
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
fn case_046() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x5e67abd6886fd822,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
"#
                        .to_owned(),
                        r#"a
"#
                        .to_owned(),
                    ),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                stress::concat(
                                    stress::concat(r#"a@{ k"#.to_owned(), crate::util::js::to_string(_index)),
                                    r#": "#.to_owned(),
                                ),
                                crate::util::js::to_string(_index),
                            ),
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
fn case_047() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x5f737eab8ea7b70a,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
"#
                        .to_owned(),
                        r#"a e1@--> b
"#
                        .to_owned(),
                    ),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                stress::concat(
                                    stress::concat(r#"e1@{ k"#.to_owned(), crate::util::js::to_string(_index)),
                                    r#": "#.to_owned(),
                                ),
                                crate::util::js::to_string(_index),
                            ),
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
fn case_048() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xae41d86a401a6770,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"a@{
"#
                            .to_owned(),
                        ),
                        stress::repeat_lines(5000.0_f64, |_index| {
                            stress::concat(
                                stress::concat(r#"  k"#.to_owned(), crate::util::js::to_string(_index)),
                                r#": "v",
"#
                                .to_owned(),
                            )
                        }),
                    ),
                    r#"}
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
fn case_049() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x9b9388ea8ca16d33,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"a@{ k: "#.to_owned(),
                        ),
                        stress::repeat_text(r#"["#.to_owned(), (50000.0_f64 / 2.0_f64)),
                    ),
                    r#" }
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
fn case_050() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x567b1944dc75021d,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"a@{
"#
                            .to_owned(),
                        ),
                        stress::repeat_text(
                            r#"  k: 'a,
"#
                            .to_owned(),
                            (50000.0_f64 / 9.0_f64),
                        ),
                    ),
                    r#"}
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
fn case_051() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x675227c37e8288bd,
            || {
                stress::concat(
                    r#"agentflow-beta TB
"#
                    .to_owned(),
                    stress::repeat_lines(4000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(r#"n"#.to_owned(), crate::util::js::to_string(_index)),
                            r#"((x))
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
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x64cf1b79110ae02d,
            || {
                stress::concat(
                    r#"agentflow-beta TB
"#
                    .to_owned(),
                    stress::repeat_lines(3000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(r#"n"#.to_owned(), crate::util::js::to_string(_index)),
                            r#"@{ shape: tri }
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
fn case_053() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xbdc0ffaa12ad4efe,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"a[""#.to_owned(),
                        ),
                        stress::repeat_text(r#"word "#.to_owned(), (50000.0_f64 / 5.0_f64)),
                    ),
                    r#""]"#.to_owned(),
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
            0xbb060c3d0ab76edc,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"a["`"#.to_owned(),
                        ),
                        stress::repeat_text(r#"*a _b "#.to_owned(), (50000.0_f64 / 6.0_f64)),
                    ),
                    r#"`"]"#.to_owned(),
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
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x29d6068f2bd08ea8,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"a[""#.to_owned(),
                        ),
                        stress::repeat_text(r#"#35;&amp;<br>"#.to_owned(), (50000.0_f64 / 13.0_f64)),
                    ),
                    r#""]"#.to_owned(),
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
fn case_056() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xe02996786c99b1fa,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"a -- ""#.to_owned(),
                        ),
                        stress::repeat_text(r#"word "#.to_owned(), (50000.0_f64 / 5.0_f64)),
                    ),
                    r#"" --> b"#.to_owned(),
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
fn case_057() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xd1444e23a3cb0c85,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
"#
                        .to_owned(),
                        stress::repeat_text(r#"a"#.to_owned(), 50000.0_f64),
                    ),
                    r#" --> b"#.to_owned(),
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
fn case_058() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x2175a23b94b932d5,
            || {
                stress::concat(
                    r#"agentflow-beta TB
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
fn case_059() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x738dae3fcb4b2d88,
            || {
                stress::concat(
                    r#"agentflow-beta TB
"#
                    .to_owned(),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(
                                stress::concat(
                                    stress::concat(r#"n"#.to_owned(), crate::util::js::to_string(_index)),
                                    r#" --> n"#.to_owned(),
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
fn case_060() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x2cef6755188f07f5,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"n"#.to_owned(),
                        ),
                        stress::repeat_lines(5000.0_f64, |_index| {
                            stress::concat(stress::concat(r#" --> n"#.to_owned(), crate::util::js::to_string(_index)), r#""#.to_owned())
                        }),
                    ),
                    r#"
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
fn case_061() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x4aec6224634c43df,
            || {
                stress::concat(
                    r#"agentflow-beta TB
"#
                    .to_owned(),
                    stress::repeat_lines(5000.0_f64, |_index| {
                        stress::concat(
                            stress::concat(r#"c --> n"#.to_owned(), crate::util::js::to_string(_index)),
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
fn case_062() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x5236134063744ea2,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            stress::concat(
                                r#"agentflow-beta TB
"#
                                .to_owned(),
                                stress::repeat_lines(150.0_f64, |_index| {
                                    stress::concat(
                                        stress::concat(r#"a"#.to_owned(), crate::util::js::to_string(_index)),
                                        r#" & "#.to_owned(),
                                    )
                                }),
                            ),
                            r#"a --> b"#.to_owned(),
                        ),
                        stress::repeat_lines(150.0_f64, |_index| {
                            stress::concat(stress::concat(r#" & b"#.to_owned(), crate::util::js::to_string(_index)), r#""#.to_owned())
                        }),
                    ),
                    r#"
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
fn case_063() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x33aeb163ec1b1ad3,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
"#
                        .to_owned(),
                        stress::repeat_text(
                            r#"A --> B
"#
                            .to_owned(),
                            2000.0_f64,
                        ),
                    ),
                    stress::repeat_text(
                        r#"A --> A
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
fn case_064() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x6e131b72e03e72f4,
            || {
                let mut s = r#"agentflow-beta TB
"#
                .to_owned();
                let mut i = 0.0_f64;
                while (i < 50.0_f64) {
                    let mut j = (i + 1.0_f64);
                    while (j < 50.0_f64) {
                        s.push_str(&stress::concat(
                            stress::concat(
                                stress::concat(stress::concat(r#"n"#.to_owned(), crate::util::js::to_string(i)), r#" --> n"#.to_owned()),
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
fn case_065() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0x86079b988aec566e,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"flow f
"#
                            .to_owned(),
                        ),
                        stress::repeat_lines(1200.0_f64, |_index| {
                            stress::concat(
                                stress::concat(r#"n"#.to_owned(), crate::util::js::to_string(_index)),
                                r#" --> f
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
                crate::render(source, &options).map(|result| {
                    std::hint::black_box(result);
                })
            },
        );
    });
}

#[test]
fn case_066() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xc1cf07e7ad44cb39,
            || {
                stress::concat(
                    stress::concat(
                        stress::concat(
                            r#"agentflow-beta TB
"#
                            .to_owned(),
                            r#"flow f
a --> b
"#
                            .to_owned(),
                        ),
                        stress::repeat_text(
                            r#"a --> f
f --> b
"#
                            .to_owned(),
                            2500.0_f64,
                        ),
                    ),
                    r#"end
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
fn case_067() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(30000, || {
        let options = crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() };
        stress::timed(
            3000,
            0xbd80e79dcca00cd1,
            || {
                stress::concat(
                    stress::concat(
                        r#"agentflow-beta TB
"#
                        .to_owned(),
                        r#"flow a
A
end
flow b
B
end
"#
                        .to_owned(),
                    ),
                    stress::repeat_text(
                        r#"a --> b
A -.- b
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
fn case_068() {
    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    stress::bounded(60000, || {
        let source = stress::concat(
            r#"agentflow-beta TB
"#
            .to_owned(),
            stress::repeat_text(
                r#"A --> B
"#
                .to_owned(),
                7000.0_f64,
            ),
        );
        stress::input_hash(&source, 0x2043d71ec3ce2353);
        let error = crate::render(&source, &crate::RenderOptions::default()).unwrap_err();
        assert!(error.message.contains("limit of 50000"), "{}", error.message);
    });
}
