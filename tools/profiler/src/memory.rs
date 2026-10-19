//! Repeated ownership checks with a counting System allocator, separate from CPU timing.
mod allocator;
use allocator::*;
use serde_json::{json, Value};
use std::{
    hint::black_box,
    io::Read,
    sync::{atomic::Ordering::Relaxed, Arc},
    time::Instant,
};
use zebra_mermaid::{parse, render, render_parsed, RenderCache, RenderOptions};
use zebra_mermaid_ffi::*;

#[global_allocator]
static ALLOCATOR: Counted = Counted;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Live {
    bytes: usize,
    blocks: usize,
}
fn live() -> Live {
    Live { bytes: LIVE.load(Relaxed), blocks: BLOCKS.load(Relaxed) }
}

struct Case {
    source: String,
    options: RenderOptions,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Digest {
    rendered: usize,
    rejected: usize,
    bytes: usize,
    hash: u64,
}
impl Digest {
    fn record(&mut self, success: bool, text: &[u8]) {
        if success {
            self.rendered += 1;
        } else {
            self.rejected += 1;
        }
        self.bytes += text.len();
        for b in text {
            self.hash = (self.hash ^ *b as u64).wrapping_mul(0x100000001b3);
        }
    }
}

fn ffi_call(source: &[u8], palette: &[u8], font: f64, d: &mut Digest) {
    let mut result = ZebraMermaidRenderResult::default();
    // SAFETY: all input slices remain live during the call, result is writable and empty;
    // buffers are read only within the lengths provided by the library, then released.
    unsafe {
        let status =
            zebra_mermaid_render_svg(source.as_ptr(), source.len(), false, palette.as_ptr(), palette.len(), font, 640.0, 0, &mut result);
        let buffer = if status == ZebraMermaidStatus::Ok { &result.svg } else { &result.error };
        let bytes = if buffer.len == 0 { &[] } else { std::slice::from_raw_parts(buffer.ptr, buffer.len) };
        d.record(status == ZebraMermaidStatus::Ok, black_box(bytes));
        // Exercise both the combined destroy and independent buffer ownership paths.
        if d.rendered % 2 == 0 {
            zebra_mermaid_buffer_destroy(&mut result.svg);
            zebra_mermaid_buffer_destroy(&mut result.error);
        }
        zebra_mermaid_render_result_destroy(&mut result);
        zebra_mermaid_render_result_destroy(&mut result);
    }
    assert!(result.svg.ptr.is_null() && result.error.ptr.is_null());
}

fn batch(mode: &str, cases: &[Case], cache: &mut RenderCache) -> Digest {
    let mut d = Digest::default();
    for c in cases {
        if mode == "ffi" {
            ffi_call(c.source.as_bytes(), &[], 16.0, &mut d);
            continue;
        }
        let result = match mode {
            "render" => render(&c.source, &c.options),
            "parsed" => parse(&c.source, &c.options.parse_options()).and_then(|p| {
                // Keep the clone alive after the original drops: shared Class declarations
                // must survive, then be released with the final owner.
                let clone = p.clone();
                drop(p);
                drop(render_parsed(&clone, &c.options)?);
                render_parsed(&clone, &c.options)
            }),
            "cache" => {
                let a = cache.render(&c.source, &c.options);
                let b = cache.render(&c.source, &c.options);
                match (a, b) {
                    (Ok(a), Ok(b)) => {
                        assert!(Arc::ptr_eq(&a, &b));
                        d.record(true, b.svg.as_bytes());
                    }
                    (Err(a), Err(b)) => {
                        assert_eq!(a.message, b.message);
                        d.record(false, b.message.as_bytes());
                    }
                    _ => panic!("cache changed render result"),
                }
                continue;
            }
            _ => panic!("unknown mode"),
        };
        match black_box(result) {
            Ok(r) => d.record(true, r.svg.as_bytes()),
            Err(e) => d.record(false, e.message.as_bytes()),
        }
    }
    if mode == "ffi" {
        let pie = b"pie\n\"a\":1";
        for (source, palette, font) in [
            (&[][..], &[][..], 16.0),
            (&[0xff, 0xfe][..], &[][..], 16.0),
            (pie.as_slice(), b"{}".as_slice(), 16.0),
            (pie.as_slice(), &[], 0.0),
            (b"pie\n\"a\" 1".as_slice(), &[], 16.0),
        ] {
            ffi_call(source, palette, font, &mut d);
        }
    }
    cache.clear();
    d
}

fn ownership_controls() {
    let before = live();
    let mut retained = vec![7u8; 4096];
    let (pointer, length, capacity) = (retained.as_mut_ptr(), retained.len(), retained.capacity());
    std::mem::forget(retained);
    let leaked = live();
    assert_eq!(leaked.bytes - before.bytes, 4096);
    assert_eq!(leaked.blocks - before.blocks, 1);
    // SAFETY: this Vec's allocation was deliberately forgotten above and is reconstructed
    // exactly once with the unchanged pointer, length and capacity.
    unsafe {
        drop(Vec::from_raw_parts(pointer, length, capacity));
    }
    assert_eq!(live(), before);

    let mut cache = RenderCache::new(2, 1 << 20);
    let options = RenderOptions::default();
    let a = "pie\n\"a\":1";
    let parsed = cache.parse(a, &options.parse_options());
    let parsed_weak = Arc::downgrade(&parsed);
    let output = cache.render(a, &options).unwrap();
    let output_weak = Arc::downgrade(&output);
    drop(parsed);
    drop(output);
    for source in ["pie\n\"b\":1", "pie\n\"c\":1"] {
        drop(cache.render(source, &options).unwrap());
    }
    assert!(parsed_weak.upgrade().is_none() && output_weak.upgrade().is_none(), "eviction retained an owner");
    let held = cache.render(a, &options).unwrap();
    let weak = Arc::downgrade(&held);
    cache.clear();
    assert!(weak.upgrade().is_some(), "caller-owned output must survive clear");
    drop(held);
    assert!(weak.upgrade().is_none(), "last output owner was not released");
    drop((weak, parsed_weak, output_weak, cache));
    assert_eq!(live(), before);
}

#[inline(never)]
fn negative_control() {
    // Deliberate leak only in a separate control process, never in renderer code.
    black_box(Box::leak(vec![0x6bu8; 4096].into_boxed_slice()));
}

fn main() {
    let mode = std::env::args().nth(1).expect("render | parsed | cache | ffi | negative-control");
    if mode == "negative-control" {
        negative_control();
        return;
    }
    let rounds: usize = std::env::args().nth(2).unwrap_or_else(|| "1000".into()).parse().unwrap();
    assert!(rounds > 0);
    let path = std::env::var_os("ZM_PROFILE_CASES").unwrap_or_else(|| "target/bench/cases.json".into());
    let source: Vec<Value> = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let cases: Vec<_> = source
        .iter()
        .flat_map(|c| {
            c["inputs"].as_array().unwrap().iter().map(|s| Case {
                source: s.as_str().unwrap().into(),
                options: RenderOptions {
                    limit: (c["unlimitedInput"] == true).then_some(usize::MAX),
                    now: Some(1792368000000.0),
                    ..Default::default()
                },
            })
        })
        .collect();
    drop(source);
    println!("{}", json!({"event":"loaded", "mode":mode,"inputs":cases.len(),"rounds":rounds}));
    let acknowledge = std::env::var_os("ZM_MEMORY_ACK").is_some();
    let mut stdin = std::io::stdin().lock();
    if acknowledge {
        stdin.read_exact(&mut [0u8]).unwrap();
    }
    // Initialise lazy tables before testing cache ownership in isolation.
    drop(render("pie\n\"a\":1", &Default::default()).unwrap());
    ownership_controls();
    let outside = live();
    let mut cache = RenderCache::new(8, 64 << 20);
    let expected = batch(&mode, &cases, &mut cache);
    for _ in 0..4 {
        assert_eq!(batch(&mode, &cases, &mut cache), expected);
    }
    let baseline = live();
    let started = Instant::now();
    PEAK.store(baseline.bytes, Relaxed);
    TRACK.store(true, Relaxed);
    for round in 1..=rounds {
        let actual = batch(&mode, &cases, &mut cache);
        let after = live();
        assert_eq!(actual, expected, "result changed at round {round}");
        assert_eq!(after, baseline, "retained allocations at round {round}");
        if round % (rounds / 20).max(1) == 0 {
            TRACK.store(false, Relaxed);
            println!(
                "{}",
                json!({"event":"checkpoint", "round":round,"liveBytes":after.bytes,"liveBlocks":after.blocks,
                "retainedBytes":0,"retainedBlocks":0,"elapsedMs":started.elapsed().as_secs_f64()*1000.0})
            );
            if acknowledge {
                stdin.read_exact(&mut [0u8]).unwrap();
            }
            TRACK.store(true, Relaxed);
        }
    }
    TRACK.store(false, Relaxed);
    let peak = PEAK.load(Relaxed).saturating_sub(baseline.bytes);
    let cache_capacity = baseline.bytes as i128 - outside.bytes as i128;
    drop(cache);
    let final_live = live();
    // Some diagram families initialise process-lifetime lazy data during the warm-up.
    // Compare cache drop against a warmed, empty cache of identical capacity below.
    let mut warmed_cache = RenderCache::new(8, 64 << 20);
    assert_eq!(batch(&mode, &cases, &mut warmed_cache), expected);
    drop(warmed_cache);
    assert_eq!(live(), final_live, "cache drop retained allocations");
    println!(
        "{}",
        json!({"event":"complete","mode":mode,"rounds":rounds,"inputsPerRound":cases.len(),
        "renderedPerRound":expected.rendered,"rejectedPerRound":expected.rejected,"elapsedMs":started.elapsed().as_secs_f64()*1000.0,
        "retainedBytesPerRound":0,"retainedBlocksPerRound":0,"retainedBytesAfterRepeatedCacheDrop":0,
        "warmupAndCacheCapacityBytes":cache_capacity,"peakAdditionalLiveBytes":peak,
        "allocations":ALLOCATIONS.load(Relaxed),"reallocations":REALLOCATIONS.load(Relaxed),
        "controlledForgottenAllocationDetectedBytes":4096,"controlledForgottenAllocationDetectedBlocks":1,
        "cacheEvictionWeakOwnersReleased":true,"callerHeldOutputReleasedAfterDrop":true})
    );
}
