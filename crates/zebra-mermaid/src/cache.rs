//! An optional cache for hosts that render the same diagrams repeatedly.
//!
//! Two levels: parsed diagrams, keyed by the source and the parse-time options; and finished
//! renders, keyed by the source and the full [`RenderOptions::render_identity`], which includes
//! every palette color, the theme, the font size, the canvas policy and the color mode. A
//! change of any color therefore never returns an earlier SVG, while the parse is reused: only
//! the drawing is redone with the new palette.
//!
//! Options with a custom measurer or icon resolver are rendered without the render cache, as
//! their identity cannot be described; their parse is still reused.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::engine::{parse, render_parsed, ParsedDiagram, RenderResult};
use crate::error::Error;
use crate::options::{ParseOptions, RenderOptions};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CacheStats {
    pub parse_hits: u64,
    pub parse_misses: u64,
    pub render_hits: u64,
    pub render_misses: u64,
    /// Renders that bypassed the render cache (custom measurer or icons).
    pub uncacheable: u64,
}

struct Lru<V> {
    map: HashMap<String, (V, usize)>,
    order: VecDeque<String>,
    capacity: usize,
    max_bytes: usize,
    bytes: usize,
}

impl<V: Clone> Lru<V> {
    fn new(capacity: usize, max_bytes: usize) -> Self {
        Lru { map: HashMap::new(), order: VecDeque::new(), capacity, max_bytes, bytes: 0 }
    }

    fn get(&mut self, key: &str) -> Option<V> {
        let v = self.map.get(key)?.0.clone();
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            let k = self.order.remove(pos).expect("present");
            self.order.push_back(k);
        }
        Some(v)
    }

    fn insert(&mut self, key: String, value: V, cost: usize) {
        if self.capacity == 0 || cost > self.max_bytes {
            return;
        }
        if let Some((_, old)) = self.map.remove(&key) {
            self.bytes -= old;
            self.order.retain(|k| k != &key);
        }
        while !self.order.is_empty() && (self.map.len() >= self.capacity || self.bytes + cost > self.max_bytes) {
            let oldest = self.order.pop_front().expect("non-empty");
            if let Some((_, c)) = self.map.remove(&oldest) {
                self.bytes -= c;
            }
        }
        self.bytes += cost;
        self.order.push_back(key.clone());
        self.map.insert(key, (value, cost));
    }

    fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
        self.bytes = 0;
    }
}

fn parse_identity(o: &ParseOptions) -> String {
    format!(
        "limit={:?}|edges={:?}|config={}",
        o.limit,
        o.max_edges,
        o.config.as_ref().map_or("-".to_string(), |c| crate::config::ConfigValue::Object(c.clone()).to_json_string())
    )
}

/// A least-recently-used cache of parses and renders with entry and byte budgets.
pub struct RenderCache {
    parsed: Lru<Arc<Result<ParsedDiagram, Error>>>,
    rendered: Lru<Arc<RenderResult>>,
    stats: CacheStats,
}

impl RenderCache {
    /// At most `entries` renders and `max_bytes` bytes of SVG and source; parses are kept up to
    /// the same entry count.
    pub fn new(entries: usize, max_bytes: usize) -> Self {
        RenderCache { parsed: Lru::new(entries, max_bytes), rendered: Lru::new(entries, max_bytes), stats: CacheStats::default() }
    }

    pub fn stats(&self) -> CacheStats {
        self.stats
    }

    pub fn clear(&mut self) {
        self.parsed.clear();
        self.rendered.clear();
    }

    /// The parse of `source` under `options`, from the cache when it was parsed before.
    pub fn parse(&mut self, source: &str, options: &ParseOptions) -> Arc<Result<ParsedDiagram, Error>> {
        let key = format!("{}\u{0}{}", parse_identity(options), source);
        if let Some(found) = self.parsed.get(&key) {
            self.stats.parse_hits += 1;
            return found;
        }
        self.stats.parse_misses += 1;
        let parsed = Arc::new(parse(source, options));
        let cost = key.len();
        self.parsed.insert(key, parsed.clone(), cost);
        parsed
    }

    /// Renders `source`, reusing an earlier result for the same source and render identity.
    pub fn render(&mut self, source: &str, options: &RenderOptions) -> Result<Arc<RenderResult>, Error> {
        let parsed = self.parse(source, &options.parse_options());
        let parsed = match parsed.as_ref() {
            Ok(p) => p,
            Err(e) => return Err(e.clone()),
        };
        if options.measurer.is_some() || options.icons.is_some() {
            self.stats.uncacheable += 1;
            return render_parsed(parsed, options).map(Arc::new);
        }
        let key = format!("{}\u{0}{}", options.render_identity(), source);
        if let Some(found) = self.rendered.get(&key) {
            self.stats.render_hits += 1;
            return Ok(found);
        }
        self.stats.render_misses += 1;
        let result = Arc::new(render_parsed(parsed, options)?);
        let cost = key.len() + result.svg.len();
        self.rendered.insert(key, result.clone(), cost);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ColorMode, MermaidPalette};

    const PIE: &str = "pie\n \"a\" : 1\n \"b\" : 2\n";

    #[test]
    fn colors_are_part_of_the_key_and_the_parse_is_reused() {
        let mut cache = RenderCache::new(16, 1 << 20);
        let mut a = MermaidPalette::light().swift_completed();
        let base = RenderOptions { color_mode: ColorMode::Resolved, palette: Some(a.clone()), ..Default::default() };
        let first = cache.render(PIE, &base).unwrap();
        a.series.as_mut().unwrap()[0] = "#123456".into();
        let changed = RenderOptions { palette: Some(a), ..base.clone() };
        let second = cache.render(PIE, &changed).unwrap();
        assert_ne!(first.svg, second.svg);
        assert!(second.svg.contains("#123456"));
        let again = cache.render(PIE, &base).unwrap();
        assert!(Arc::ptr_eq(&first, &again));
        let s = cache.stats();
        assert_eq!((s.render_misses, s.render_hits), (2, 1));
        assert_eq!((s.parse_misses, s.parse_hits), (1, 2));
    }

    #[test]
    fn budgets_evict_the_oldest() {
        let mut cache = RenderCache::new(2, 1 << 20);
        let o = RenderOptions::default();
        for src in ["pie\n \"a\" : 1", "pie\n \"b\" : 1", "pie\n \"c\" : 1"] {
            cache.render(src, &o).unwrap();
        }
        cache.render("pie\n \"a\" : 1", &o).unwrap();
        assert_eq!(cache.stats().render_misses, 4);
        let mut tiny = RenderCache::new(8, 10);
        tiny.render(PIE, &o).unwrap();
        tiny.render(PIE, &o).unwrap();
        assert_eq!(tiny.stats().render_hits, 0, "an entry over the byte budget is not kept");
    }

    #[test]
    fn errors_are_cached_as_parses() {
        let mut cache = RenderCache::new(4, 1 << 20);
        let o = RenderOptions::default();
        assert!(cache.render("pie\n \"a\" 1", &o).is_err());
        assert!(cache.render("pie\n \"a\" 1", &o).is_err());
        assert_eq!(cache.stats().parse_hits, 1);
    }
}
