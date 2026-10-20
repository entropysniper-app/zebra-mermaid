//! Observable scanner lifecycle contracts, including spill and panic paths.
use super::*;
use std::cell::RefCell;
use std::panic::{catch_unwind, AssertUnwindSafe};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Event {
    Create(u8),
    Call(u8, usize),
    Drop(u8),
    Panic,
    Pattern,
    Opener,
}
thread_local! { static EVENTS: RefCell<Vec<Event>> = const { RefCell::new(Vec::new()) }; }
fn record(event: Event) {
    EVENTS.with(|events| events.borrow_mut().push(event));
}

struct Probe(u8);
impl Probe {
    fn called(&self, count: usize) {
        record(Event::Call(self.0, count));
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        record(Event::Drop(self.0));
    }
}
fn make(letter: u8) -> Scanner {
    record(Event::Create(letter));
    let probe = Probe(letter);
    let mut count = 0;
    Box::new(move |src: &str, at: usize| {
        count += 1;
        probe.called(count);
        (src.as_bytes().get(at) == Some(&letter)).then_some(at + 1)
    })
}
fn a() -> Scanner {
    make(b'a')
}
fn b() -> Scanner {
    make(b'b')
}
fn c() -> Scanner {
    make(b'c')
}
fn d() -> Scanner {
    make(b'd')
}
fn e() -> Scanner {
    make(b'e')
}
fn f() -> Scanner {
    make(b'f')
}
fn panic_factory() -> Scanner {
    record(Event::Panic);
    panic!("scanner factory control");
}

fn table(len: usize) -> Vec<TokenType> {
    let mut types = vec![TokenType::literal("unused", "~"); len];
    let makers: [fn() -> Scanner; 6] = [a, b, c, d, e, f];
    for (index, first) in ["a", "b", "c", "d", "e", "f"].into_iter().enumerate() {
        types[index] = TokenType {
            name: first,
            pattern: Pattern::Scanner(makers[index]),
            first: Some(first),
            hidden: false,
            opener: None,
            longer: &[],
        };
    }
    types[6] =
        TokenType { name: "panic", pattern: Pattern::Scanner(panic_factory), first: Some("p"), hidden: false, opener: None, longer: &[] };
    types
}

fn expected(source: &str, panic: bool) -> Vec<Event> {
    let mut counts = [0usize; 6];
    let mut out = Vec::new();
    for letter in source.bytes().take_while(|c| (b'a'..=b'f').contains(c)) {
        let index = (letter - b'a') as usize;
        if counts[index] == 0 {
            out.push(Event::Create(letter));
        }
        counts[index] += 1;
        out.push(Event::Call(letter, counts[index]));
    }
    if panic {
        out.push(Event::Panic);
    }
    // The original type-indexed Vec drops live handles in table order, regardless of
    // the order their factories ran. Unused factories produce no handle to drop.
    for (index, count) in counts.into_iter().enumerate() {
        if count > 0 {
            out.push(Event::Drop(b'a' + index as u8));
        }
    }
    out
}

#[test]
fn scanner_laziness_state_and_drop_order_survive_small_and_large_tables() {
    for len in [7, 63, 64, 65, 130] {
        for source in ["fedcbaafedcba", "fedcbaafedcba?"] {
            EVENTS.with(|events| events.borrow_mut().clear());
            let types = table(len);
            let result = tokenize(source, &types, DiagramType::Info);
            if source.ends_with('?') {
                let error = result.unwrap_err();
                assert_eq!(error.code, crate::ErrorCode::Syntax);
                assert_eq!(error.column, source.len());
            } else {
                let tokens = result.unwrap();
                let mut kinds: Vec<_> = source.bytes().map(|c| (c - b'a') as usize).collect();
                kinds.push(EOF);
                assert_eq!(tokens.kinds, kinds);
                assert_eq!(tokens.starts, (0..=source.len()).collect::<Vec<_>>());
            }
            assert_eq!(EVENTS.with(|events| events.borrow().clone()), expected(source, false), "{len} {source}");
        }
    }
}

#[test]
fn scanner_factory_panics_preserve_drop_order_preserve_live_handles() {
    for len in [7, 63, 64, 65, 130] {
        for source in ["fedcp", "fedcbap"] {
            EVENTS.with(|events| events.borrow_mut().clear());
            let types = table(len);
            let result = catch_unwind(AssertUnwindSafe(|| tokenize(source, &types, DiagramType::Info)));
            assert!(result.is_err());
            assert_eq!(EVENTS.with(|events| events.borrow().clone()), expected(source, true), "{len} {source}");
        }
    }
}

fn failed_pattern(_: &str, _: usize) -> Option<usize> {
    record(Event::Pattern);
    None
}
fn opener(_: &str, at: usize) -> Option<usize> {
    record(Event::Opener);
    Some(at + 1)
}

#[test]
fn failed_openers_are_disabled_in_small_and_large_tables() {
    for len in [2, 63, 64, 65, 130] {
        EVENTS.with(|events| events.borrow_mut().clear());
        let mut types = vec![TokenType::literal("unused", "~"); len];
        types[len - 2] = TokenType::func("failed", failed_pattern, Some("@")).with_opener(opener);
        types[len - 1] = TokenType::literal("at", "@");
        let tokens = tokenize("@@@", &types, DiagramType::Info).unwrap();
        assert_eq!(tokens.kinds, vec![len - 1, len - 1, len - 1, EOF]);
        assert_eq!(tokens.starts, vec![0, 1, 2, 3]);
        assert_eq!(tokens.ends, vec![1, 2, 3, 3]);
        assert_eq!(EVENTS.with(|events| events.borrow().clone()), vec![Event::Pattern, Event::Opener]);
    }
}

#[test]
fn scanners_at_table_boundary_keep_full_indices() {
    for len in [7, 63, 64, 65, 130] {
        EVENTS.with(|events| events.borrow_mut().clear());
        let mut types = vec![TokenType::literal("unused", "~"); len];
        types[len - 1] = TokenType { name: "a", pattern: Pattern::Scanner(a), first: Some("a"), hidden: false, opener: None, longer: &[] };
        let tokens = tokenize("aaa", &types, DiagramType::Info).unwrap();
        assert_eq!(tokens.kinds, vec![len - 1, len - 1, len - 1, EOF]);
        assert_eq!(tokens.starts, vec![0, 1, 2, 3]);
        assert_eq!(tokens.ends, vec![1, 2, 3, 3]);
        assert_eq!(EVENTS.with(|events| events.borrow().clone()), expected("aaa", false));
    }
}
