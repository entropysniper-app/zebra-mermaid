//! Independent ports of Pele tests/render/treemap.test.ts layout callbacks (MIT).
use super::{layout_treemap, Box, Insets};
use crate::diagrams::treemap::TreemapNode;
fn leaf(name: &str, value: f64) -> TreemapNode {
    TreemapNode { name: name.into(), value: Some(value), children: None, class_selector: None, css_compiled_styles: None }
}
fn section(name: &str, children: Vec<usize>) -> TreemapNode {
    TreemapNode { name: name.into(), value: None, children: Some(children), class_selector: None, css_compiled_styles: None }
}
fn insets() -> Insets {
    Insets { gap: 4., header: 28., short: 18., side: 6. }
}
fn layout(nodes: &[TreemapNode], roots: &[usize], w: f64, h: f64, ins: &Insets) -> Vec<Box> {
    let (all, placed) = layout_treemap(nodes, roots, w, h, ins);
    placed.into_iter().map(|i| all[i].clone()).collect()
}
fn overlap(a: &Box, b: &Box) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}
#[test]
fn headerless_section_full_area() {
    let nodes = [section("s", vec![1, 2]), leaf("a", 2.), section("t", vec![3, 4]), leaf("b", 1.), leaf("c", 1.)];
    let bs = layout(&nodes, &[0], 300., 30., &insets());
    let mut names = bs.iter().map(|b| nodes[b.node].name.as_str()).collect::<Vec<_>>();
    names.sort();
    assert_eq!(names, ["a", "b", "c", "s", "t"]);
    let s = bs.iter().find(|b| nodes[b.node].name == "s").unwrap();
    assert!(!s.headed);
    for b in &bs {
        if std::ptr::eq(b, s) {
            continue;
        }
        assert!(b.w * b.h > 0.);
        assert!(b.x >= s.x);
        assert!(b.y >= s.y);
        assert!(b.x + b.w <= s.x + s.w);
        assert!(b.y + b.h <= s.y + s.h);
    }
    assert!(layout(&nodes, &[0], 300., 300., &insets()).iter().find(|b| nodes[b.node].name == "s").unwrap().headed);
}
#[test]
fn thinner_than_gap_and_zero_room() {
    let nodes = [leaf("big", 397.), leaf("thin", 3.)];
    let bs = layout(&nodes, &[0, 1], 400., 100., &insets());
    assert_eq!(bs.iter().map(|b| nodes[b.node].name.as_str()).collect::<Vec<_>>(), ["big", "thin"]);
    assert!(bs[1].w > 0.);
    assert!(!overlap(&bs[0], &bs[1]));
    let nodes = [leaf("big", 1e6), section("s", vec![2]), leaf("speck", 1.)];
    let bs = layout(&nodes, &[0, 1], 100., 100., &insets());
    assert_eq!(bs.iter().map(|b| nodes[b.node].name.as_str()).collect::<Vec<_>>(), ["big", "s", "speck"]);
    for b in bs.iter().skip(1) {
        assert_eq!(b.w * b.h, 0.);
    }
}
#[test]
fn finite_boxes_inside_area() {
    let nodes = [leaf("big", 397.), leaf("thin", 3.)];
    for b in layout(&nodes, &[0, 1], 400., 100., &Insets { gap: 40., ..insets() }) {
        assert!(b.x + b.w <= 400.);
    }
    let nodes = [leaf("big", 1e20), leaf("small", 1.)];
    let bs = layout(&nodes, &[0, 1], 640., 400., &insets());
    assert_eq!(bs.iter().map(|b| nodes[b.node].name.as_str()).collect::<Vec<_>>(), ["big", "small"]);
    for b in bs {
        for n in [b.x, b.y, b.w, b.h] {
            assert!(n.is_finite() && n >= 0.);
        }
    }
}
