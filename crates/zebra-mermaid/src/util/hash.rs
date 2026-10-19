//! A hasher for maps keyed by the renderer's own indices.
//!
//! Layout and routing keep maps from node, edge and rank numbers that the code assigns itself,
//! so no author of a diagram can choose keys that collide. Those maps take this multiplicative
//! hasher, which does a fraction of the work of the standard library's SipHash. Maps keyed by
//! text from the diagram keep the standard hasher.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default, Clone, Copy)]
pub struct IndexHasher(u64);

const MULTIPLIER: u64 = 0x517c_c1b7_2722_0a95;

impl IndexHasher {
    #[inline]
    fn add(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(MULTIPLIER);
    }
}

impl Hasher for IndexHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.add(b as u64);
        }
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add(i as u64);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.add(i as u64);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.add(i as u64);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add(i);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add(i as u64);
    }

    #[inline]
    fn write_i64(&mut self, i: i64) {
        self.add(i as u64);
    }
}

pub type IndexBuild = BuildHasherDefault<IndexHasher>;
/// A map keyed by indices the renderer assigns.
pub type IndexedMap<K, V> = HashMap<K, V, IndexBuild>;
/// A set of indices the renderer assigns.
pub type IndexedSet<K> = HashSet<K, IndexBuild>;
