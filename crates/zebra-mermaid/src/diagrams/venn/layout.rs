//! Venn layout: circle placement, label spots and region outlines. Ported from Pele
//! `src/diagrams/venn/layout.ts`. Every expression keeps the original's order of operations.

use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;

use indexmap::IndexMap;

use super::VennSubset;
use crate::util::{js, math};
use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Circle {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub r: f64,
}

/// Where the content of a region goes: the point inside all of `inside` and outside every other
/// circle that is farthest from their outlines, and how far that is. A margin of 0 or less means
/// the circles as placed leave the region empty.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Spot {
    pub x: f64,
    pub y: f64,
    pub margin: f64,
}

const TAU: f64 = PI * 2.0;

/// For every overlap of three or more sets, adds the pairwise overlaps it implies and the diagram
/// does not state, a quarter of the smaller set each, so that the circles are drawn overlapping.
pub(crate) fn ensure_pairwise_subsets(subsets: &[VennSubset]) -> Vec<VennSubset> {
    let sorted = |sets: &[String]| {
        let mut sets = sets.to_vec();
        sets.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
        sets
    };
    // `JSON.stringify([...subset.sets].sort())`: a list of strings has one JSON form.
    let mut present: HashSet<Vec<String>> = HashSet::new();
    let mut sizes: HashMap<&str, f64> = HashMap::new();
    for subset in subsets {
        present.insert(sorted(&subset.sets));
        if subset.sets.len() == 1 {
            sizes.insert(&subset.sets[0], subset.size);
        }
    }
    let mut added: Vec<VennSubset> = Vec::new();
    for subset in subsets {
        if subset.sets.len() < 3 {
            continue;
        }
        let members = sorted(&subset.sets);
        for i in 0..members.len() - 1 {
            for j in i + 1..members.len() {
                let pair = vec![members[i].clone(), members[j].clone()];
                if present.contains(&pair) {
                    continue;
                }
                present.insert(pair.clone());
                let size = match (sizes.get(pair[0].as_str()), sizes.get(pair[1].as_str())) {
                    (Some(&a), Some(&b)) => js::min(a, b) / 4.0,
                    _ => 2.5,
                };
                added.push(VennSubset { sets: pair, size, label: Some(String::new()) });
            }
        }
    }
    let mut all = subsets.to_vec();
    all.extend(added);
    all
}

/// The area two circles share when their centers are d apart.
pub(crate) fn lens_area(r1: f64, r2: f64, d: f64) -> f64 {
    if d >= r1 + r2 {
        return 0.0;
    }
    let small = js::min(r1, r2);
    if d <= (r1 - r2).abs() {
        return PI * small * small;
    }
    let a1 = r1 * r1 * math::acos((d * d + r1 * r1 - r2 * r2) / (2.0 * d * r1));
    let a2 = r2 * r2 * math::acos((d * d + r2 * r2 - r1 * r1) / (2.0 * d * r2));
    let a3 = 0.5 * math::sqrt(js::max((-d + r1 + r2) * (d + r1 - r2) * (d - r1 + r2) * (d + r1 + r2), 0.0));
    a1 + a2 - a3
}

/// How far apart two circles sit for their overlap to have the given area. The area falls as the
/// distance grows, so bisection finds it.
pub(crate) fn distance_for_overlap(r1: f64, r2: f64, area: f64) -> f64 {
    let mut low = (r1 - r2).abs();
    let mut high = r1 + r2;
    // `!(area > 0)`, which NaN passes.
    if area.is_nan() || area <= 0.0 {
        return high;
    }
    if area >= PI * math::pow(js::min(r1, r2), 2.0) {
        return low;
    }
    for _ in 0..48 {
        let middle = (low + high) / 2.0;
        if lens_area(r1, r2, middle) > area {
            low = middle;
        } else {
            high = middle;
        }
    }
    (low + high) / 2.0
}

/// How far a point is from the nearest outline it must stay within or without. Gives up, with
/// some value below `floor`, as soon as the point cannot reach `floor`.
fn margin(circles: &[Circle], inside: &[usize], outside: &[usize], x: f64, y: f64, floor: f64) -> f64 {
    let mut best = f64::INFINITY;
    for &i in inside {
        let c = &circles[i];
        let dx = x - c.x;
        let dy = y - c.y;
        let room = c.r - math::sqrt(dx * dx + dy * dy);
        if room < best {
            if room < floor {
                return room;
            }
            best = room;
        }
    }
    for &i in outside {
        let c = &circles[i];
        let dx = x - c.x;
        let dy = y - c.y;
        let room = math::sqrt(dx * dx + dy * dy) - c.r;
        if room < best {
            if room < floor {
                return room;
            }
            best = room;
        }
    }
    best
}

/// Finds the roomiest point of a region by sampling the smallest of its circles on a grid and then
/// walking uphill from the best sample. Ties go to the point nearest the middle of the region, so
/// a symmetric arrangement gets symmetric labels.
pub(crate) fn find_spot(circles: &[Circle], inside: &[usize], outside: &[usize]) -> Spot {
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut smallest = &circles[inside[0]];
    let count = inside.len() as f64;
    for &i in inside {
        let c = &circles[i];
        cx += c.x / count;
        cy += c.y / count;
        if c.r < smallest.r {
            smallest = c;
        }
    }
    let r = smallest.r;
    let mut x = cx;
    let mut y = cy;
    let mut best = margin(circles, inside, outside, x, y, f64::NEG_INFINITY);
    let mut off = 0.0;
    let mut consider = |px: f64, py: f64, x: &mut f64, y: &mut f64| -> bool {
        let m = margin(circles, inside, outside, px, py, best - r * 1e-9);
        // `!(m > best - r * 1e-9)`, which NaN passes.
        let floor = best - r * 1e-9;
        if m.is_nan() || floor.is_nan() || m <= floor {
            return false;
        }
        let distance = math::hypot(px - cx, py - cy);
        if m > best + r * 1e-9 || distance < off - r * 1e-9 {
            best = m;
            off = distance;
            *x = px;
            *y = py;
            return true;
        }
        false
    };
    const GRID: i32 = 10;
    let grid = f64::from(GRID);
    for gy in -GRID..=GRID {
        for gx in -GRID..=GRID {
            if gx * gx + gy * gy <= GRID * GRID {
                consider(smallest.x + (f64::from(gx) * r) / grid, smallest.y + (f64::from(gy) * r) / grid, &mut x, &mut y);
            }
        }
    }
    let mut step = r / grid;
    let mut round = 0;
    while round < 40 && step > r * 1e-6 {
        let mut moved = false;
        for k in 0..8 {
            let angle = (f64::from(k) * TAU) / 8.0;
            let (px, py) = (x + math::cos(angle) * step, y + math::sin(angle) * step);
            if consider(px, py, &mut x, &mut y) {
                moved = true;
            }
        }
        if !moved {
            step /= 2.0;
        }
        round += 1;
    }
    Spot { x, y, margin: best }
}

/// Places one circle per set, with areas in proportion to the sizes of the sets and each pair as
/// far apart as its overlap asks: one, two and three sets exactly, more by relaxing a ring of
/// circles toward those distances. Lengths are in units of the square root of a size.
pub(crate) fn layout_circles(subsets: &[VennSubset]) -> Vec<Circle> {
    let mut sizes: IndexMap<&str, f64> = IndexMap::new();
    for subset in subsets {
        for id in &subset.sets {
            sizes.entry(id.as_str()).or_insert(10.0);
        }
    }
    for subset in subsets {
        if subset.sets.len() == 1 {
            sizes.insert(&subset.sets[0], subset.size);
        }
    }

    let mut index: HashMap<&str, usize> = HashMap::new();
    let mut circles: Vec<Circle> = Vec::new();
    let mut largest = 0.0;
    for &size in sizes.values() {
        if size.is_finite() && size > largest {
            largest = size;
        }
    }
    if largest == 0.0 {
        largest = 10.0;
    }
    let widest = math::sqrt(largest / PI);
    for (&id, &size) in &sizes {
        index.insert(id, circles.len());
        // A set too small to hold its name, or with no size at all, still gets a circle that shows.
        let r = if size.is_finite() && size > 0.0 { math::sqrt(size / PI) } else { 0.0 };
        circles.push(Circle { id: id.to_string(), x: 0.0, y: 0.0, r: js::max(r, widest * 0.3) });
    }
    let n = circles.len();
    if n < 2 {
        return circles;
    }

    // The overlap each pair was given, by the indexes of its circles.
    let mut overlaps: HashMap<usize, f64> = HashMap::new();
    for subset in ensure_pairwise_subsets(subsets) {
        if subset.sets.len() != 2 {
            continue;
        }
        let (Some(&a), Some(&b)) = (index.get(subset.sets[0].as_str()), index.get(subset.sets[1].as_str())) else {
            continue;
        };
        if a != b {
            overlaps.insert(a.min(b) * n + a.max(b), subset.size);
        }
    }
    // How far apart each overlapping pair should be. Pairs that do not overlap only have to keep a gap.
    let mut wanted: HashMap<usize, f64> = HashMap::with_capacity(overlaps.len());
    for (&key, &area) in &overlaps {
        wanted.insert(key, distance_for_overlap(circles[key / n].r, circles[key % n].r, area));
    }
    let gap = widest * 0.15;
    let distance = |circles: &[Circle], a: usize, b: usize| -> f64 {
        match wanted.get(&(a * n + b)) {
            Some(&d) => d,
            None => circles[a].r + circles[b].r + gap,
        }
    };

    if n == 2 {
        circles[1].x = distance(&circles, 0, 1);
        return circles;
    }

    if n == 3 {
        // Two sets side by side and the third below, the corners of a triangle whose sides are the
        // three distances. An overlap of all three has to exist, so the triangle shrinks until it does.
        let mut ab = distance(&circles, 0, 1);
        let mut ac = distance(&circles, 0, 2);
        let mut bc = distance(&circles, 1, 2);
        let triple = subsets.iter().any(|subset| subset.sets.iter().collect::<HashSet<_>>().len() == 3);
        for _ in 0..40 {
            let x = if ab > 0.0 { (ab * ab + ac * ac - bc * bc) / (2.0 * ab) } else { 0.0 };
            circles[1].x = ab;
            circles[2].x = js::max(-ac, js::min(ac, x));
            circles[2].y = math::sqrt(js::max(ac * ac - x * x, 0.0));
            if !triple || find_spot(&circles, &[0, 1, 2], &[]).margin > widest * 0.12 {
                break;
            }
            ab *= 0.94;
            ac *= 0.94;
            bc *= 0.94;
        }
        return circles;
    }

    let mut total = 0.0;
    for c in &circles {
        total += 2.0 * c.r + gap;
    }
    let rounds = js::min(300.0, (3e6 / (n as f64 * n as f64)).floor()) as usize;
    if n <= 64 {
        let ring = total / TAU;
        for (i, c) in circles.iter_mut().enumerate() {
            let angle = (i as f64 * TAU) / n as f64 - PI / 2.0;
            c.x = math::cos(angle) * ring;
            c.y = math::sin(angle) * ring;
        }
    } else {
        let columns = math::sqrt(n as f64).ceil() as usize;
        let cell = 2.0 * widest + gap;
        for (i, c) in circles.iter_mut().enumerate() {
            c.x = (i % columns) as f64 * cell;
            c.y = (i / columns) as f64 * cell;
        }
    }
    let mut dx = vec![0.0f64; n];
    let mut dy = vec![0.0f64; n];
    let mut pulls = vec![0i32; n];
    for _ in 0..rounds {
        dx.fill(0.0);
        dy.fill(0.0);
        pulls.fill(0);
        for a in 0..n {
            for b in a + 1..n {
                let overlap = wanted.get(&(a * n + b)).copied();
                let apart = circles[a].r + circles[b].r + gap;
                let mut ux = circles[b].x - circles[a].x;
                let mut uy = circles[b].y - circles[a].y;
                let mut length = math::hypot(ux, uy);
                if overlap.is_none() && length >= apart {
                    continue;
                }
                if length < 1e-9 {
                    ux = math::cos((a + b) as f64);
                    uy = math::sin((a + b) as f64);
                    length = 1.0;
                }
                let movement = (length - overlap.unwrap_or(apart)) / 2.0 / length;
                dx[a] += ux * movement;
                dy[a] += uy * movement;
                dx[b] -= ux * movement;
                dy[b] -= uy * movement;
                pulls[a] += 1;
                pulls[b] += 1;
            }
        }
        for i in 0..n {
            if pulls[i] == 0 {
                continue;
            }
            circles[i].x += dx[i] / f64::from(pulls[i]);
            circles[i].y += dy[i] / f64::from(pulls[i]);
        }
    }
    circles
}

struct Corner {
    x: f64,
    y: f64,
    a: usize,
    b: usize,
}

/// The outline of the region all the given circles share, as an SVG path, or '' if they share none.
pub(crate) fn region_path(circles: &[&Circle], format: impl Fn(f64) -> String) -> String {
    let whole = |c: &Circle| -> String {
        format!(
            "M{},{}A{},{} 0 1 1 {},{}A{},{} 0 1 1 {},{}Z",
            format(c.x - c.r),
            format(c.y),
            format(c.r),
            format(c.r),
            format(c.x + c.r),
            format(c.y),
            format(c.r),
            format(c.r),
            format(c.x - c.r),
            format(c.y)
        )
    };
    if circles.len() == 1 {
        return whole(circles[0]);
    }
    let within = |x: f64, y: f64, skip: usize, other: usize| -> bool {
        for (k, c) in circles.iter().enumerate() {
            if k != skip && k != other && math::hypot(x - c.x, y - c.y) > c.r * (1.0 + 1e-9) {
                return false;
            }
        }
        true
    };
    // The corners of the region: where two outlines cross inside all the other circles.
    let mut corners: Vec<Corner> = Vec::new();
    for i in 0..circles.len() {
        for j in i + 1..circles.len() {
            let a = circles[i];
            let b = circles[j];
            let d = math::hypot(b.x - a.x, b.y - a.y);
            if d >= a.r + b.r || d <= (a.r - b.r).abs() || d == 0.0 {
                continue;
            }
            let along = (a.r * a.r - b.r * b.r + d * d) / (2.0 * d);
            let up = math::sqrt(js::max(a.r * a.r - along * along, 0.0));
            let mx = a.x + ((b.x - a.x) * along) / d;
            let my = a.y + ((b.y - a.y) * along) / d;
            for sign in [1.0, -1.0] {
                let x = mx + (sign * up * (b.y - a.y)) / d;
                let y = my - (sign * up * (b.x - a.x)) / d;
                if within(x, y, i, j) {
                    corners.push(Corner { x, y, a: i, b: j });
                }
            }
        }
    }
    if corners.len() < 2 {
        // No corners: either one circle lies inside all the others, or there is nothing shared.
        let mut smallest = 0;
        for (k, c) in circles.iter().enumerate() {
            if c.r < circles[smallest].r {
                smallest = k;
            }
        }
        let s = circles[smallest];
        for (k, c) in circles.iter().enumerate() {
            if k != smallest && math::hypot(c.x - s.x, c.y - s.y) + s.r > c.r * (1.0 + 1e-9) {
                return String::new();
            }
        }
        return whole(s);
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    let count = corners.len() as f64;
    for p in &corners {
        cx += p.x / count;
        cy += p.y / count;
    }
    // `corners.sort((p, q) => atan2(p) - atan2(q))`, stable. An insertion sort cannot fail on an
    // order that NaN would make inconsistent, and a region has few corners.
    let angles: Vec<f64> = corners.iter().map(|p| math::atan2(p.y - cy, p.x - cx)).collect();
    let mut order: Vec<usize> = (0..corners.len()).collect();
    for i in 1..order.len() {
        let item = order[i];
        let mut j = i;
        while j > 0 && angles[order[j - 1]] - angles[item] > 0.0 {
            order[j] = order[j - 1];
            j -= 1;
        }
        order[j] = item;
    }
    let corners: Vec<&Corner> = order.iter().map(|&k| &corners[k]).collect();
    let mut d = String::new();
    for i in 0..corners.len() {
        let from = corners[i];
        let to = corners[(i + 1) % corners.len()];
        if i == 0 {
            d = format!("M{},{}", format(from.x), format(from.y));
        }
        // The side between two corners is an arc of a circle that passes through both; of those, the
        // one whose arc stays inside the others.
        let mut arc: Option<usize> = None;
        let mut large = 0;
        let mut narrowest = f64::INFINITY;
        for k in [from.a, from.b] {
            if k != to.a && k != to.b {
                continue;
            }
            let c = circles[k];
            let start = math::atan2(from.y - c.y, from.x - c.x);
            let mut sweep = math::atan2(to.y - c.y, to.x - c.x) - start;
            if sweep <= 0.0 {
                sweep += TAU;
            }
            let mx = c.x + c.r * math::cos(start + sweep / 2.0);
            let my = c.y + c.r * math::sin(start + sweep / 2.0);
            let width = math::hypot(mx - (from.x + to.x) / 2.0, my - (from.y + to.y) / 2.0);
            if width < narrowest {
                narrowest = width;
                arc = Some(k);
                large = if sweep > PI { 1 } else { 0 };
            }
        }
        match arc {
            None => write!(d, "L{},{}", format(to.x), format(to.y)).unwrap(),
            Some(k) => {
                let r = circles[k].r;
                write!(d, "A{},{} 0 {} 1 {},{}", format(r), format(r), large, format(to.x), format(to.y)).unwrap();
            }
        }
    }
    d + "Z"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lens_and_distance() {
        assert_eq!(lens_area(1.0, 1.0, 2.0), 0.0);
        assert_eq!(lens_area(1.0, 2.0, 0.5), PI);
        let d = distance_for_overlap(1.0, 1.0, 1.0);
        assert!((lens_area(1.0, 1.0, d) - 1.0).abs() < 1e-9);
        assert_eq!(distance_for_overlap(1.0, 2.0, 0.0), 3.0);
    }

    #[test]
    fn pairs_for_triples() {
        let subset = |sets: &[&str], size: f64| VennSubset { sets: sets.iter().map(|s| s.to_string()).collect(), size, label: None };
        let all =
            ensure_pairwise_subsets(&[subset(&["A"], 8.0), subset(&["B"], 4.0), subset(&["A", "B", "C"], 1.0), subset(&["A", "C"], 2.0)]);
        let added: Vec<_> = all[4..].iter().map(|s| (s.sets.join(","), s.size)).collect();
        assert_eq!(added, vec![("A,B".to_string(), 1.0), ("B,C".to_string(), 2.5)]);
    }
}
