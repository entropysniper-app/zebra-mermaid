//! 1:1 port: Pele tests/render/edges.test.ts (MIT, kepano 2026).
use zebra_mermaid::svg::edges::{default_reach, route_path, EdgePath, LEAD};
fn path(route: &[f64], curve: Option<&str>, start: f64, end: f64) -> EdgePath {
    route_path(route, curve, start, end, default_reach(start), default_reach(end))
}
fn numbers(s: &str, spaces: bool) -> bool {
    !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit() || matches!(c, b'-' | b'.' | b',') || (spaces && c == b' '))
}
fn curve(route: &[f64], start: f64, end: f64, reach: Option<(f64, f64)>) -> Vec<f64> {
    let (sr, er) = reach.unwrap_or((default_reach(start), default_reach(end)));
    let d = route_path(route, None, start, end, sr, er).d;
    // Exact language of /^M[-\d.,]+(L[-\d.,]+)?C[-\d., ]+(L[-\d.,]+)?$/.
    let rest = d.strip_prefix('M').expect("path starts with M");
    let (before, after) = rest.split_once('C').expect("one curve");
    let first: Vec<_> = before.split('L').collect();
    assert!((1..=2).contains(&first.len()) && first.iter().all(|s| numbers(s, false)), "{d}");
    let last: Vec<_> = after.split('L').collect();
    assert!((1..=2).contains(&last.len()) && numbers(last[0], true) && last.get(1).is_none_or(|s| numbers(s, false)), "{d}");
    last[0].split([' ', ',']).map(|s| if s.is_empty() { 0.0 } else { s.parse().unwrap() }).collect()
}
fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 0.5 * 10_f64.powi(-5), "{actual} != {expected}");
}
#[test]
fn draws_one_curve_between_two_points() {
    for (across, along) in [(100.0, 48.0), (-100.0, 48.0), (100.0, 200.0), (30.0, 48.0)] {
        let down = path(&[0.0, 0.0, 0.0, across, along, 0.0], None, 0.0, 0.0);
        assert!(down.d.strip_prefix("M0,0C").is_some_and(|s| numbers(s, true)));
        let c = curve(&[0.0, 0.0, 0.0, across, along, 0.0], 0.0, 0.0, None);
        let (x1, y1, x2, y2, x, y) = (c[0], c[1], c[2], c[3], c[4], c[5]);
        assert_eq!([x1, x2, x, y], [0.0, across, across, along]);
        assert!(y1 > 0.0);
        assert!(y2 < along);
        assert_eq!([down.sdx + 0.0, down.sdy, down.edx + 0.0, down.edy], [0.0, -1.0, 0.0, 1.0]);
        let side = curve(&[0.0, 0.0, 1.0, along, across, 1.0], 0.0, 0.0, None);
        assert_eq!(&side[..4], &[y1, 0.0, y2, across]);
    }
}
#[test]
fn turns_sooner_on_a_run_wider_than_the_gap() {
    let even = curve(&[0.0, 0.0, 0.0, 200.0, 48.0, 0.0], 0.0, 0.0, None);
    close(even[1], 24.0);
    close(even[3], 24.0);
    let c = curve(&[0.0, 0.0, 0.0, 480.0, 48.0, 0.0], 0.0, 0.0, None);
    close(c[1], 19.2);
    close(c[3], 28.8);
    let far = curve(&[0.0, 0.0, 0.0, -4800.0, 48.0, 0.0], 0.0, 0.0, None);
    close(far[1], 12.0);
    close(far[3], 36.0);
    let side = curve(&[0.0, 0.0, 1.0, 40.0, 400.0, 1.0], 0.0, 0.0, None);
    close(side[0], 16.0);
    close(side[2], 24.0);
}
#[test]
fn runs_straight_into_a_marker() {
    let route = [0.0, 0.0, 0.0, 40.0, 48.0, 0.0];
    let arrow = path(&route, None, 0.0, 7.0);
    let segment = arrow.d.split_once('C').unwrap().1.strip_suffix(" 40,35L40,41").unwrap();
    assert!(numbers(segment, true));
    assert_eq!(&curve(&route, 0.0, 7.0, None)[4..], &[40.0, 48.0 - 7.0 - LEAD]);
    assert!(path(&route, None, 7.0, 0.0).d.starts_with("M0,7L0,13C"));
    assert_eq!(&curve(&route, 0.0, 0.0, Some((0.0, 7.0 + LEAD)))[4..], &[40.0, 35.0]);
    assert_eq!(&curve(&route, 0.0, 8.0, Some((0.0, 8.0 + LEAD)))[4..], &curve(&route, 0.0, 7.0, Some((0.0, 8.0 + LEAD)))[4..]);
    assert_eq!(&curve(&[0.0, 0.0, 0.0, 15.0, 19.0, 0.0], 0.0, 7.0, None)[4..], &[15.0, 9.0]);
    assert_eq!(path(&[0.0, 0.0, 0.0, 0.0, 48.0, 0.0], None, 0.0, 7.0).d, "M0,0L0,41");
}
#[test]
fn keeps_both_ends_heading_along_flow() {
    for across in [40.0, 96.0, -96.0, 900.0] {
        let p = path(&[0.0, 0.0, 0.0, across, 48.0, 0.0], None, 0.0, 7.0);
        assert_eq!([p.sdx + 0.0, p.sdy], [0.0, -1.0]);
        assert_eq!([p.edx + 0.0, p.edy], [0.0, 1.0]);
    }
}
#[test]
fn turns_corners_and_preserves_linear_routes() {
    let r = [0.0, 0.0, 0.0, 0.0, 20.0, 0.0, 100.0, 20.0, 0.0, 100.0, 48.0, 0.0];
    let p = path(&r, None, 0.0, 0.0);
    let body = p.d.strip_prefix("M0,0C").unwrap().strip_suffix(" 100,48").unwrap();
    let (a, b) = body.split_once(" 50,20C").unwrap();
    assert!(numbers(a, true) && numbers(b, true));
    assert_eq!(path(&r, Some("step"), 0.0, 0.0).d, "M0,0L0,8Q0,20 12,20L88,20Q100,20 100,32L100,48");
    assert_eq!(
        path(&[0.0, 0.0, 0.0, 0.0, 20.0, 0.0, 10.0, 20.0, 0.0, 10.0, 48.0, 0.0], Some("step"), 0.0, 0.0).d,
        "M0,0L0,15Q0,20 5,20Q10,20 10,25L10,48"
    );
    assert_eq!(path(&r, Some("linear"), 0.0, 0.0).d, "M0,0L0,20L100,20L100,48");
    assert_eq!(path(&[0.0, 0.0, 0.0, 0.0, 20.0, 0.0, 0.0, 48.0, 0.0], None, 0.0, 0.0).d, "M0,0L0,48");
}
#[test]
fn turns_stepped_routes_at_right_angles() {
    let r = [0.0, 0.0, 0.0, 100.0, 48.0, 0.0];
    let step = path(&r, Some("step"), 0.0, 7.0);
    assert_eq!(step.d, "M0,0L0,12Q0,24 12,24L88,24Q100,24 100,36L100,41");
    assert_eq!([step.edx, step.edy], [0.0, 1.0]);
    assert_eq!(path(&r, Some("stepAfter"), 0.0, 0.0).d, "M0,0L0,36Q0,48 12,48L100,48");
    assert_eq!(path(&[0.0, 0.0, 1.0, 48.0, 100.0, 1.0], Some("stepBefore"), 0.0, 0.0).d, "M0,0L0,88Q0,100 12,100L48,100");
}
