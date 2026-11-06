# zebra-mermaid-ffi

C ABI v2 for the pure Rust `zebra-mermaid` renderer, aligned with **Pele 1.13.0**.
It produces SVG for 33 Mermaid diagram types with built-in RaTeX math and the
Zebra host palette contract.

```toml
[dependencies]
zebra-mermaid-ffi = "=0.1.0"
```

The crate builds a Rust library, static library, and dynamic library. The C
declarations are shipped as `include/zebra_mermaid.h`. Follow that header's
ownership, buffer, and release-function requirements. The ABI version is separate
from the crate version; this package implements ABI **2**.

The default feature enables all implemented diagrams. Native output resolves
theme colors and applies the existing Zebra link/image policies. A separate theme
JSON file is not required by the host. Rust 1.96 or later is required; runtime
rendering is pure Rust.

Host application integration and complete upstream-test acceptance remain
unfinished. The full ABI and differential regression suite runs from the Git
repository with its fixtures. See the repository's
[implementation status](https://github.com/entropysniper-app/zebra-mermaid/blob/main/docs/IMPLEMENTATION-STATUS.md).

MIT licensed; see `LICENSE` and `THIRD_PARTY_NOTICES.md`.
[Source and C header](https://github.com/entropysniper-app/zebra-mermaid/tree/main/crates/zebra-mermaid-ffi).
