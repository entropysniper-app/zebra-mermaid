//! Borrowed lookup with the map's own hasher; allocate a String key only on insertion.
use indexmap::map::raw_entry_v1::{RawEntryApiV1, RawEntryMut};
use indexmap::IndexMap;
use std::hash::BuildHasher;

pub(crate) fn or_insert_with<'a, V, S: BuildHasher>(map: &'a mut IndexMap<String, V, S>, key: &str, make: impl FnOnce() -> V) -> &'a mut V {
    // String and str hash the same bytes. Both raw operations use this exact map's
    // hasher, preserving its random seed, equality checks and insertion order.
    let hash = map.hasher().hash_one(key);
    match map.raw_entry_mut_v1().from_key_hashed_nocheck(hash, key) {
        RawEntryMut::Occupied(entry) => entry.into_mut(),
        RawEntryMut::Vacant(entry) => entry.insert_hashed_nocheck(hash, key.to_owned(), make()).1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::RandomState;
    use std::hash::Hasher;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[derive(Default)]
    struct Collisions;
    struct SameHash;
    impl Hasher for SameHash {
        fn finish(&self) -> u64 {
            0
        }
        fn write(&mut self, _: &[u8]) {}
    }
    impl BuildHasher for Collisions {
        type Hasher = SameHash;
        fn build_hasher(&self) -> SameHash {
            SameHash
        }
    }

    fn check_operations<S: BuildHasher + Default>() {
        let mut old = IndexMap::<String, Vec<usize>, S>::with_hasher(S::default());
        let mut new = IndexMap::<String, Vec<usize>, S>::with_hasher(S::default());
        let keys = ["", "a", "a\0", "a\0b", "A", "é", "e\u{301}", "中😀", "__proto__", " a ", "prefix-long-a", "prefix-long-b"];
        let mut seed = 0x51e17_u32;
        let mut old_calls = Vec::new();
        let mut new_calls = Vec::new();
        for i in 0..512 {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let key = keys[seed as usize % keys.len()];
            if i % 13 == 0 {
                assert_eq!(old.shift_remove(key), new.shift_remove(key));
            } else if i % 7 == 0 {
                old.entry(key.to_owned()).or_default().push(i);
                new.entry(key.to_owned()).or_default().push(i);
            } else {
                old.entry(key.to_owned())
                    .or_insert_with(|| {
                        old_calls.push(i);
                        Vec::new()
                    })
                    .push(i);
                or_insert_with(&mut new, key, || {
                    new_calls.push(i);
                    Vec::new()
                })
                .push(i);
            }
            assert_eq!(old_calls, new_calls);
            assert_eq!(old.iter().collect::<Vec<_>>(), new.iter().collect::<Vec<_>>());
            for key in keys {
                assert_eq!(old.get(key), new.get(key));
                assert_eq!(old.get(&key.to_owned()), new.get(&key.to_owned()));
            }
        }
    }

    #[test]
    fn borrowed_entries_interoperate_with_owned_entries_and_collisions() {
        check_operations::<RandomState>();
        check_operations::<Collisions>();
    }

    #[test]
    fn factories_are_lazy_and_panics_do_not_insert_or_reorder() {
        let mut map = IndexMap::new();
        map.insert("first".to_owned(), 1);
        map.insert("second".to_owned(), 2);
        assert_eq!(*or_insert_with(&mut map, "first", || panic!("occupied factory")), 1);
        let failure = catch_unwind(AssertUnwindSafe(|| {
            or_insert_with(&mut map, "missing", || -> usize { panic!("vacant factory") });
        }));
        assert!(failure.is_err());
        assert_eq!(map.iter().map(|(k, v)| (k.as_str(), *v)).collect::<Vec<_>>(), vec![("first", 1), ("second", 2)]);
        assert_eq!(*or_insert_with(&mut map, "missing", || 3), 3);
        assert_eq!(map.get_index(2), Some((&"missing".to_owned(), &3)));
    }
}
