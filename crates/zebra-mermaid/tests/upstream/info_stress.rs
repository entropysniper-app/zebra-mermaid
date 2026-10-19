//! 1:1 port: Pele tests/info/stress.test.ts (MIT, kepano 2026).
use std::time::{Duration, Instant};
use zebra_mermaid::diagrams::info::parse_info;
use zebra_mermaid::{render, Error, RenderOptions};
const N: usize = 50000;
fn timed(run: impl FnOnce() -> Result<(), Error>) {
    let started = Instant::now();
    // The Result's Error type corresponds to the original instanceof PeleError assertion.
    // Panics are not caught and therefore fail this test.
    let _: Result<(), Error> = run();
    assert!(started.elapsed() < Duration::from_millis(3000));
}
fn draw(src: String) -> Result<(), Error> {
    render(&src, &RenderOptions { limit: Some(usize::MAX), ..Default::default() }).map(|_| ())
}
fn parse(src: String) -> Result<(), Error> {
    parse_info(&src).map(|_| ())
}
#[test]
fn title_50000() {
    timed(|| draw(format!("info\ntitle {}", "word ".repeat(N / 5))))
}
#[test]
fn titles_6000() {
    timed(|| draw(format!("info\n{}", "title t\n".repeat(N / 8))))
}
#[test]
fn title_entities_and_breaks() {
    timed(|| draw(format!("info\ntitle {}", "#35;&amp;<br>".repeat(N / 13))))
}
#[test]
fn blank_lines_25000() {
    timed(|| draw(format!("info{}showInfo", "\n ".repeat(N / 2))))
}
#[test]
fn line_of_spaces() {
    timed(|| parse(format!("info{}showInfo", " ".repeat(N))))
}
#[test]
fn comments_20000() {
    timed(|| parse(format!("info\n{}", "%% c\n".repeat(N / 5))))
}
#[test]
fn unclosed_directives() {
    timed(|| parse(format!("info\n{}", "%%{\n".repeat(N / 4))))
}
#[test]
fn front_matter_fences() {
    timed(|| parse(format!("info\n{}", "---\n".repeat(N / 4))))
}
#[test]
fn unclosed_description() {
    timed(|| parse(format!("info\naccDescr {{{}", " a\n".repeat(N / 3))))
}
#[test]
fn closed_description_50000() {
    timed(|| draw(format!("info\naccDescr {{{}}}", " a\n".repeat(N / 3))))
}
#[test]
fn long_accessible_title() {
    timed(|| draw(format!("info\naccTitle: {}", "x".repeat(N))))
}
#[test]
fn keyword_10000() {
    timed(|| parse("info ".repeat(N / 5)))
}
#[test]
fn show_info_run_together() {
    timed(|| parse(format!("info {}", "showInfo".repeat(N / 8))))
}
