//! C ABI version 2 of zebra-mermaid: the function, types, status codes and ownership rules of
//! Zebra's native Mermaid bridge (`ZebraMermaidBridge.h`), backed by this renderer.
//!
//! The header is `include/zebra_mermaid.h`, the same declarations as Zebra's bridge header. A
//! result's buffers are owned by Rust and must be released with
//! `zebra_mermaid_render_result_destroy` (or `zebra_mermaid_buffer_destroy`). No Rust panic
//! crosses the boundary: one is reported as `ZEBRA_MERMAID_INTERNAL_PANIC`.
//!
//! Rendering follows `zebra_mermaid::zebra::render_abi_v2`: resolved colors, Zebra's palette
//! contract, no links or images, and geometry that does not depend on `target_width` (which is
//! validated and otherwise unused), so a host may cache SVGs without the width in their key.
//!
//! This crate has not been linked into Zebra; replacing Zebra's existing binary needs that
//! integration and its tests first.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::{ptr, slice};

use zebra_mermaid::zebra::{render_abi_v2, ZebraStatus, ABI_VERSION};

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZebraMermaidStatus {
    Ok = 0,
    InvalidArgument = 1,
    InvalidUtf8 = 2,
    ParseFailure = 3,
    OutputTooLarge = 4,
    InternalPanic = 255,
}

impl From<ZebraStatus> for ZebraMermaidStatus {
    fn from(s: ZebraStatus) -> Self {
        match s {
            ZebraStatus::Ok => ZebraMermaidStatus::Ok,
            ZebraStatus::InvalidArgument => ZebraMermaidStatus::InvalidArgument,
            ZebraStatus::InvalidUtf8 => ZebraMermaidStatus::InvalidUtf8,
            ZebraStatus::ParseFailure => ZebraMermaidStatus::ParseFailure,
            ZebraStatus::OutputTooLarge => ZebraMermaidStatus::OutputTooLarge,
            ZebraStatus::InternalPanic => ZebraMermaidStatus::InternalPanic,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ZebraMermaidOwnedBuffer {
    pub ptr: *mut u8,
    pub len: usize,
    pub capacity: usize,
}

impl Default for ZebraMermaidOwnedBuffer {
    fn default() -> Self {
        ZebraMermaidOwnedBuffer { ptr: ptr::null_mut(), len: 0, capacity: 0 }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct ZebraMermaidRenderResult {
    pub svg: ZebraMermaidOwnedBuffer,
    pub error: ZebraMermaidOwnedBuffer,
    pub width: f64,
    pub height: f64,
}

fn owned_buffer(mut bytes: Vec<u8>) -> ZebraMermaidOwnedBuffer {
    if bytes.is_empty() {
        return ZebraMermaidOwnedBuffer::default();
    }
    let buffer = ZebraMermaidOwnedBuffer { ptr: bytes.as_mut_ptr(), len: bytes.len(), capacity: bytes.capacity() };
    std::mem::forget(bytes);
    buffer
}

/// At most 512 bytes of a message, as Zebra's bridge copies them.
fn write_error(result: &mut ZebraMermaidRenderResult, message: &str) {
    result.error = owned_buffer(message.as_bytes().iter().copied().take(512).collect());
}

/// The bytes of an argument: `None` for an empty buffer, `Err` for a null pointer with a length.
unsafe fn bytes<'a>(pointer: *const u8, length: usize) -> Result<Option<&'a [u8]>, ()> {
    if length == 0 {
        return Ok(None);
    }
    if pointer.is_null() {
        return Err(());
    }
    // SAFETY: the caller promises `pointer` is valid for `length` bytes for the call.
    Ok(Some(unsafe { slice::from_raw_parts(pointer, length) }))
}

#[no_mangle]
pub extern "C" fn zebra_mermaid_abi_version() -> u32 {
    ABI_VERSION
}

/// Releases a buffer returned by this library and empties it. Null and empty buffers are fine.
///
/// # Safety
/// `buffer` must be null or point to a buffer filled by this library and not yet released.
#[no_mangle]
pub unsafe extern "C" fn zebra_mermaid_buffer_destroy(buffer: *mut ZebraMermaidOwnedBuffer) {
    if buffer.is_null() {
        return;
    }
    // SAFETY: checked non-null; the caller owns it.
    let buffer = unsafe { &mut *buffer };
    if !buffer.ptr.is_null() {
        // SAFETY: ptr, len and capacity came from a Vec<u8> this library forgot.
        unsafe { drop(Vec::from_raw_parts(buffer.ptr, buffer.len, buffer.capacity)) };
    }
    *buffer = ZebraMermaidOwnedBuffer::default();
}

/// Releases both buffers of a result and zeroes its size.
///
/// # Safety
/// `result` must be null or point to a result filled by `zebra_mermaid_render_svg`.
#[no_mangle]
pub unsafe extern "C" fn zebra_mermaid_render_result_destroy(result: *mut ZebraMermaidRenderResult) {
    if result.is_null() {
        return;
    }
    // SAFETY: checked non-null; the caller owns it.
    let result = unsafe { &mut *result };
    // SAFETY: each buffer was filled by this library.
    unsafe {
        zebra_mermaid_buffer_destroy(&mut result.svg);
        zebra_mermaid_buffer_destroy(&mut result.error);
    }
    result.width = 0.0;
    result.height = 0.0;
}

/// Renders Mermaid source to SVG. See the module documentation.
///
/// # Safety
/// `source_utf8` must be valid for `source_utf8_len` bytes and `palette_json_utf8` for
/// `palette_json_utf8_len` bytes (it may be null when the length is 0); `out_result` must
/// point to writable memory for one result, which is overwritten without being released.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn zebra_mermaid_render_svg(
    source_utf8: *const u8,
    source_utf8_len: usize,
    dark_theme: bool,
    palette_json_utf8: *const u8,
    palette_json_utf8_len: usize,
    font_size: f64,
    target_width: f64,
    canvas_background: u32,
    out_result: *mut ZebraMermaidRenderResult,
) -> ZebraMermaidStatus {
    if out_result.is_null() {
        return ZebraMermaidStatus::InvalidArgument;
    }
    // SAFETY: checked non-null; the caller provides writable memory.
    let result = unsafe { &mut *out_result };
    // The previous contents are not released: the header says the result is overwritten.
    unsafe { ptr::write(result, ZebraMermaidRenderResult::default()) };

    let operation = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: the caller's promises about the argument buffers.
        let source = match unsafe { bytes(source_utf8, source_utf8_len) } {
            Ok(Some(s)) => s,
            // A null or empty source is refused, as Zebra's bridge refuses it.
            _ => b"" as &[u8],
        };
        // SAFETY: as above.
        let palette = match unsafe { bytes(palette_json_utf8, palette_json_utf8_len) } {
            Ok(p) => p,
            Err(()) => return Err((ZebraStatus::InvalidArgument, "Invalid Mermaid palette.".to_string())),
        };
        render_abi_v2(source, dark_theme, palette, font_size, target_width, canvas_background)
    }));

    match operation {
        Ok(Ok(rendered)) => {
            result.svg = owned_buffer(rendered.svg.into_bytes());
            result.width = rendered.width;
            result.height = rendered.height;
            ZebraMermaidStatus::Ok
        }
        Ok(Err((status, message))) => {
            write_error(result, &message);
            status.into()
        }
        Err(_) => {
            write_error(result, "Native Mermaid renderer panicked.");
            ZebraMermaidStatus::InternalPanic
        }
    }
}
