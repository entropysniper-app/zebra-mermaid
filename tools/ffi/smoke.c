#include <stdio.h>
#include <string.h>
#include "zebra_mermaid.h"

int main(void) {
    const char *src = "pie title Pets\n \"Dogs\" : 42\n \"Cats\" : 58";
    ZebraMermaidRenderResult r = {0};
    if (zebra_mermaid_abi_version() != 2) return 10;
    ZebraMermaidStatus s = zebra_mermaid_render_svg((const uint8_t *)src, strlen(src), true, NULL, 0, 16.0, 640.0,
                                                    ZEBRA_MERMAID_CANVAS_BACKGROUND_OPAQUE, &r);
    if (s != ZEBRA_MERMAID_OK) return 11;
    printf("ok %.0fx%.0f %zu bytes starts=%.4s\n", r.width, r.height, r.svg.len, (const char *)r.svg.ptr);
    zebra_mermaid_render_result_destroy(&r);
    s = zebra_mermaid_render_svg((const uint8_t *)"x", 1, false, NULL, 0, 16.0, 640.0, 0, &r);
    printf("status %d error=%.*s\n", (int)s, (int)r.error.len, (const char *)r.error.ptr);
    zebra_mermaid_render_result_destroy(&r);
    return s == ZEBRA_MERMAID_PARSE_FAILURE ? 0 : 12;
}
