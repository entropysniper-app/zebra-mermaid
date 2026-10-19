//! Calls the C ABI as Zebra's Swift service does.

use std::{ptr, slice, str};

use zebra_mermaid_ffi::*;

const PIE: &[u8] = b"pie title Pets\n \"Dogs\" : 42\n \"Cats\" : 58";

fn call(source: &[u8], dark: bool, palette: &[u8], font: f64, width: f64, canvas: u32) -> (ZebraMermaidStatus, ZebraMermaidRenderResult) {
    let mut result = ZebraMermaidRenderResult::default();
    let status = unsafe {
        zebra_mermaid_render_svg(
            if source.is_empty() { ptr::null() } else { source.as_ptr() },
            source.len(),
            dark,
            if palette.is_empty() { ptr::null() } else { palette.as_ptr() },
            palette.len(),
            font,
            width,
            canvas,
            &mut result,
        )
    };
    (status, result)
}

fn text(buffer: &ZebraMermaidOwnedBuffer) -> String {
    if buffer.ptr.is_null() {
        return String::new();
    }
    unsafe { str::from_utf8(slice::from_raw_parts(buffer.ptr, buffer.len)).unwrap().to_string() }
}

#[test]
fn renders_and_releases_owned_output() {
    assert_eq!(zebra_mermaid_abi_version(), 2);
    let (status, mut result) = call(PIE, false, b"", 16.0, 640.0, 1);
    assert_eq!(status, ZebraMermaidStatus::Ok);
    assert!(result.width > 0.0 && result.height > 0.0);
    let svg = text(&result.svg);
    assert!(svg.starts_with("<svg"));
    assert!(!svg.contains("data-mermaid-canvas-background=\"opaque\""));
    assert!(result.error.ptr.is_null());
    unsafe { zebra_mermaid_render_result_destroy(&mut result) };
    assert!(result.svg.ptr.is_null());
    assert_eq!(result.width, 0.0);
    // Releasing twice, or a null pointer, is harmless.
    unsafe { zebra_mermaid_render_result_destroy(&mut result) };
    unsafe { zebra_mermaid_render_result_destroy(ptr::null_mut()) };
    unsafe { zebra_mermaid_buffer_destroy(ptr::null_mut()) };
}

#[test]
fn palette_theme_and_width_independence() {
    let palette = include_bytes!("../../../docs/research/zebra-theme-palette.example.json");
    let (s1, mut a) = call(PIE, false, palette, 16.0, 300.0, 0);
    let (s2, mut b) = call(PIE, false, palette, 16.0, 3000.0, 0);
    assert_eq!((s1, s2), (ZebraMermaidStatus::Ok, ZebraMermaidStatus::Ok));
    assert_eq!(text(&a.svg), text(&b.svg));
    assert!(text(&a.svg).contains("#2563EB"));
    let (_, mut dark) = call(PIE, true, b"", 16.0, 640.0, 0);
    assert!(text(&dark.svg).contains("#18181B"));
    unsafe {
        zebra_mermaid_render_result_destroy(&mut a);
        zebra_mermaid_render_result_destroy(&mut b);
        zebra_mermaid_render_result_destroy(&mut dark);
    }
}

#[test]
fn errors_carry_status_and_message() {
    let cases: [(&[u8], &[u8], f64, f64, u32, ZebraMermaidStatus, &str); 6] = [
        (PIE, b"", 0.0, 640.0, 0, ZebraMermaidStatus::InvalidArgument, "Invalid Mermaid layout metrics."),
        (PIE, b"", 16.0, 0.0, 0, ZebraMermaidStatus::InvalidArgument, "Invalid Mermaid layout metrics."),
        (b"", b"", 16.0, 640.0, 0, ZebraMermaidStatus::InvalidArgument, "Invalid Mermaid source."),
        (&[0xff, 0xfe], b"", 16.0, 640.0, 0, ZebraMermaidStatus::InvalidUtf8, "Invalid Mermaid source."),
        (PIE, b"{}", 16.0, 640.0, 0, ZebraMermaidStatus::InvalidArgument, "Invalid Mermaid palette."),
        (PIE, b"", 16.0, 640.0, 7, ZebraMermaidStatus::InvalidArgument, "Invalid Mermaid canvas background policy."),
    ];
    for (source, palette, font, width, canvas, status, message) in cases {
        let (got, mut result) = call(source, false, palette, font, width, canvas);
        assert_eq!(got, status, "{message}");
        assert_eq!(text(&result.error), message);
        assert!(result.svg.ptr.is_null());
        unsafe { zebra_mermaid_render_result_destroy(&mut result) };
    }
    let (got, mut result) = call(b"pie\n \"a\" 1", false, b"", 16.0, 640.0, 0);
    assert_eq!(got, ZebraMermaidStatus::ParseFailure);
    assert!(text(&result.error).starts_with("Parse error on line 2"));
    unsafe { zebra_mermaid_render_result_destroy(&mut result) };
    let status = unsafe { zebra_mermaid_render_svg(PIE.as_ptr(), PIE.len(), false, ptr::null(), 0, 16.0, 640.0, 0, ptr::null_mut()) };
    assert_eq!(status, ZebraMermaidStatus::InvalidArgument);
}
