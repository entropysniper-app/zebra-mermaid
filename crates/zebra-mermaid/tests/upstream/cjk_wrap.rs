//! 1:1 port: Pele tests/render/cjk-wrap.test.ts (MIT, kepano 2026).
use zebra_mermaid::text::{layout_label, Icons, Label, Measure};
use zebra_mermaid::util::chars::is_space;
use zebra_mermaid::{render, MetricsMeasurer, RenderOptions, TextMeasurer};

const JA: &str = "在庫引当の結果を基幹システムへ連携し、出荷指示を倉庫管理システムに送信して完了を確認する";

fn layout(raw: &str, max: f64) -> Label {
    let icons = Icons::new(None);
    layout_label(Some(raw), false, &Measure::new(&MetricsMeasurer, &icons), 16.0, max, 0)
}

fn lines(raw: &str, max: f64) -> Vec<String> {
    layout(raw, max).lines.into_iter().map(|line| line.into_iter().map(|span| span.text).collect()).collect()
}

#[test]
fn wraps_japanese_and_keeps_every_character() {
    let out = lines(JA, 200.0);
    assert!(out.len() > 1);
    assert_eq!(out.join(""), JA);
    for width in layout(JA, 200.0).widths {
        assert!(width <= 200.0);
    }
}

#[test]
fn keeps_closing_punctuation_and_opening_brackets_on_the_correct_line() {
    for max in (40..=240).step_by(8) {
        let source = format!("「{JA}」（確認済み）。");
        let out = lines(&source, max as f64);
        assert_eq!(out.join(""), source);
        for line in out {
            assert!(!line.starts_with(['、', '。', '，', '．', '）', '」', '』', '】']));
            assert!(!line.ends_with(['「', '『', '（', '【']));
        }
    }
}

#[test]
fn wraps_mixed_latin_and_japanese_without_splitting_latin_words() {
    let out = lines("AWS Lambda から呼び出されるバッチ処理の完了通知を送信する", 160.0);
    assert!(out.len() > 1);
    assert_eq!(
        out.join("").chars().filter(|&c| !is_space(c)).collect::<String>(),
        "AWSLambdaから呼び出されるバッチ処理の完了通知を送信する"
    );
    assert!(out.join("|").contains("Lambda"));
}

#[test]
fn keeps_english_and_korean_wrapping_on_word_boundaries() {
    assert_eq!(lines("one two three four five six", 80.0), ["one two", "three four", "five six"]);
    let ko = lines("안녕하세요 반갑습니다 여러분 모두", 90.0);
    assert!(ko.iter().all(|line| !line.ends_with(is_space)));
    for word in ["안녕하세요", "반갑습니다", "여러분", "모두"] {
        assert!(ko.iter().any(|line| line.contains(word)));
    }
}

#[test]
fn keeps_short_japanese_and_long_unbroken_latin_words_whole() {
    assert_eq!(lines("在庫確認", 200.0), ["在庫確認"]);
    assert_eq!(lines("Supercalifragilisticexpialidocious", 60.0), ["Supercalifragilisticexpialidocious"]);
}

#[test]
fn makes_a_long_japanese_flowchart_label_narrower() {
    let label = "在庫引当の結果を基幹システムへ連携し出荷指示を倉庫管理システムに送信する";
    let wrapped = render(&format!("flowchart TD\n  A[{label}] --> B[完了]"), &RenderOptions::default()).unwrap().svg;
    let view_box = wrapped.split("viewBox=\"").nth(1).unwrap().split('"').next().unwrap();
    let width: f64 = view_box.split_whitespace().nth(2).unwrap().parse().unwrap();
    assert!(width < MetricsMeasurer.width(label, 16.0, 0));
}
