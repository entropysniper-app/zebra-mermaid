# zebra-mermaid

A pure Rust Mermaid-to-SVG library, ported from Pele and aligned with **1.13.0**
at `3a2f075ee0c3d914f0fad3545fc0f193693975ac`. It supports all 33 diagram types
by default and needs no JavaScript, browser, or external layout program at runtime.

```toml
[dependencies]
zebra-mermaid = "=0.1.0"
```

```rust
use zebra_mermaid::{render, RenderOptions};

fn main() -> Result<(), zebra_mermaid::Error> {
    let result = render("flowchart LR\n  A[Start] --> B[Done]", &RenderOptions::default())?;
    assert_eq!(result.diagram_type.as_str(), "flowchart");
    println!("{}", result.svg);
    Ok(())
}
```

The default output uses Pele CSS variables. For native SVG consumers, choose
`ColorMode::Resolved` and supply the host's `MermaidPalette`. The library also
provides parsed-model reuse, an optional bounded `RenderCache`, custom text
measurement, and built-in RaTeX math with embedded font outlines.

All diagram types can be selected with Cargo features. Use
`default-features = false` plus the required diagram features for a smaller build.
The optional `serde` feature serializes models and errors. Rust 1.96 or later is
required; this release was verified with rustc 1.96.1.

The CLI is published separately as `zebra-mermaid-cli`; C ABI v2 is in
`zebra-mermaid-ffi`.

The full differential and upstream test suite runs from the Git repository with
its versioned fixtures. Those development assets are excluded from the crates.io
archive. Original-test acceptance is still incomplete: 2,952 of 7,110 tests pass,
with 7 preserved upstream skips and 4,151 unmapped cases. Output comparisons and
diagram coverage do not replace that acceptance requirement. See the repository's
[implementation status](https://github.com/entropysniper-app/zebra-mermaid/blob/main/docs/IMPLEMENTATION-STATUS.md).

MIT licensed. `THIRD_PARTY_NOTICES.md` contains the notices for derived code and
dependencies. [Source and rendering gallery](https://github.com/entropysniper-app/zebra-mermaid).
