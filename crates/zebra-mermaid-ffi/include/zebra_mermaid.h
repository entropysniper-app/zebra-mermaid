#ifndef ZEBRA_MERMAID_BRIDGE_H
#define ZEBRA_MERMAID_BRIDGE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum ZebraMermaidStatus {
    ZEBRA_MERMAID_OK = 0,
    ZEBRA_MERMAID_INVALID_ARGUMENT = 1,
    ZEBRA_MERMAID_INVALID_UTF8 = 2,
    ZEBRA_MERMAID_PARSE_FAILURE = 3,
    ZEBRA_MERMAID_OUTPUT_TOO_LARGE = 4,
    ZEBRA_MERMAID_INTERNAL_PANIC = 255
} ZebraMermaidStatus;

typedef enum ZebraMermaidCanvasBackground {
    ZEBRA_MERMAID_CANVAS_BACKGROUND_OPAQUE = 0,
    ZEBRA_MERMAID_CANVAS_BACKGROUND_TRANSPARENT = 1
} ZebraMermaidCanvasBackground;

typedef struct ZebraMermaidOwnedBuffer {
    uint8_t *ptr;
    size_t len;
    size_t capacity;
} ZebraMermaidOwnedBuffer;

typedef struct ZebraMermaidRenderResult {
    ZebraMermaidOwnedBuffer svg;
    ZebraMermaidOwnedBuffer error;
    double width;
    double height;
} ZebraMermaidRenderResult;

uint32_t zebra_mermaid_abi_version(void);
void zebra_mermaid_buffer_destroy(ZebraMermaidOwnedBuffer *buffer);
void zebra_mermaid_render_result_destroy(
    ZebraMermaidRenderResult *result);

ZebraMermaidStatus zebra_mermaid_render_svg(
    const uint8_t *source_utf8,
    size_t source_utf8_len,
    bool dark_theme,
    const uint8_t *palette_json_utf8,
    size_t palette_json_utf8_len,
    double font_size,
    double target_width,
    uint32_t canvas_background,
    ZebraMermaidRenderResult *out_result);

#ifdef __cplusplus
}
#endif

#endif
