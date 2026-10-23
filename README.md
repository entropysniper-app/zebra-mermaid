# zebra-mermaid

A pure Rust library and CLI for rendering Mermaid diagrams to SVG. Ported from [Pele](https://github.com/obsidianmd/pele/tree/e7f092a4de147cc32210913c20e36eaa5fcd0ef1) and aligned with **1.9.0**, targeting **Mermaid 12.1.0** syntax.

- **33 diagram types** enabled by default, with Cargo features for smaller builds.
- **Native rendering** with no JavaScript, browser, or external layout process at runtime.
- **Built-in mathematics** with [RaTeX](https://github.com/erweixin/RaTeX), bundled math fonts, and self-contained SVG glyphs.
- **Host-controlled appearance** with CSS variables for web content and resolved colors for native SVG consumers.
- **Library, CLI, and C ABI** with parsed-model reuse, an optional render cache, batch conversion, and structured errors.

[Quick start](#quick-start) · [Rendering comparison](#rendering-comparison) · [Performance](#performance) · [Library API](#library-api) · [Validation](#validation)

## Quick start

Requires Rust 1.96 or later; verified with rustc 1.96.1. Install directly from GitHub:

```sh
cargo install --git https://github.com/entropysniper-app/zebra-mermaid \
  --rev 2f2c3b097dcaa1e9d15a448f9cda0822049301a3 \
  --locked zebra-mermaid-cli

zebra-mermaid render diagram.mmd -o diagram.svg
zebra-mermaid render diagram.mmd --theme dark \
  --canvas-background transparent -o dark.svg
zebra-mermaid check diagram.mmd --format json
zebra-mermaid batch ./diagrams --out-dir ./svg --jobs 4
```

Use `-` to read standard input. Output files are written atomically; add `--force` to replace an existing file. Run `zebra-mermaid --help` for layout, font, link, image, and resource-limit options.

<!-- sample-comparison:start -->
## Rendering comparison

The same **167 samples across 33 diagram types** rendered by three engines. Each source is preserved unchanged.

| Renderer | Successful samples |
| --- | ---: |
| zebra-mermaid | **167/167** |
| Pele | **167/167** |
| Mermaid.js | **167/167** |

zebra-mermaid and Pele produce **identical SVG bytes and dimensions for 167/167 samples**. They share browser font measurements and the Playground's dark theme; parsing, layout, and SVG generation run independently. Mermaid.js uses its own layout with the dark theme.

[Browse all 167 samples](docs/render-comparison/index.html) · [Summary and method](docs/render-comparison/summary.md) · [Sources, dimensions, and hashes](docs/render-comparison/results.json)

The table shows the first sample of each diagram type. Click a preview to open its SVG. Previews scale to fit their cells; actual dimensions are recorded in the gallery.

| Diagram / source | zebra-mermaid | Pele | Mermaid.js |
| --- | --- | --- | --- |
| **flowchart**<br>[flowchart-01](docs/render-comparison/sources/flowchart-01.mmd) | [<img src="docs/render-comparison/previews/flowchart-01.rust.png" width="240" alt="flowchart-01 rust">](docs/render-comparison/svg/flowchart-01.rust.svg) | [<img src="docs/render-comparison/previews/flowchart-01.pele.png" width="240" alt="flowchart-01 pele">](docs/render-comparison/svg/flowchart-01.pele.svg) | [<img src="docs/render-comparison/previews/flowchart-01.mermaid.png" width="240" alt="flowchart-01 mermaid">](docs/render-comparison/svg/flowchart-01.mermaid.svg) |
| **block**<br>[block-01](docs/render-comparison/sources/block-01.mmd) | [<img src="docs/render-comparison/previews/block-01.rust.png" width="240" alt="block-01 rust">](docs/render-comparison/svg/block-01.rust.svg) | [<img src="docs/render-comparison/previews/block-01.pele.png" width="240" alt="block-01 pele">](docs/render-comparison/svg/block-01.pele.svg) | [<img src="docs/render-comparison/previews/block-01.mermaid.png" width="240" alt="block-01 mermaid">](docs/render-comparison/svg/block-01.mermaid.svg) |
| **c4**<br>[c4-01](docs/render-comparison/sources/c4-01.mmd) | [<img src="docs/render-comparison/previews/c4-01.rust.png" width="240" alt="c4-01 rust">](docs/render-comparison/svg/c4-01.rust.svg) | [<img src="docs/render-comparison/previews/c4-01.pele.png" width="240" alt="c4-01 pele">](docs/render-comparison/svg/c4-01.pele.svg) | [<img src="docs/render-comparison/previews/c4-01.mermaid.png" width="240" alt="c4-01 mermaid">](docs/render-comparison/svg/c4-01.mermaid.svg) |
| **class**<br>[class-01](docs/render-comparison/sources/class-01.mmd) | [<img src="docs/render-comparison/previews/class-01.rust.png" width="240" alt="class-01 rust">](docs/render-comparison/svg/class-01.rust.svg) | [<img src="docs/render-comparison/previews/class-01.pele.png" width="240" alt="class-01 pele">](docs/render-comparison/svg/class-01.pele.svg) | [<img src="docs/render-comparison/previews/class-01.mermaid.png" width="240" alt="class-01 mermaid">](docs/render-comparison/svg/class-01.mermaid.svg) |
| **er**<br>[er-01](docs/render-comparison/sources/er-01.mmd) | [<img src="docs/render-comparison/previews/er-01.rust.png" width="240" alt="er-01 rust">](docs/render-comparison/svg/er-01.rust.svg) | [<img src="docs/render-comparison/previews/er-01.pele.png" width="240" alt="er-01 pele">](docs/render-comparison/svg/er-01.pele.svg) | [<img src="docs/render-comparison/previews/er-01.mermaid.png" width="240" alt="er-01 mermaid">](docs/render-comparison/svg/er-01.mermaid.svg) |
| **eventmodeling**<br>[eventmodeling-01](docs/render-comparison/sources/eventmodeling-01.mmd) | [<img src="docs/render-comparison/previews/eventmodeling-01.rust.png" width="240" alt="eventmodeling-01 rust">](docs/render-comparison/svg/eventmodeling-01.rust.svg) | [<img src="docs/render-comparison/previews/eventmodeling-01.pele.png" width="240" alt="eventmodeling-01 pele">](docs/render-comparison/svg/eventmodeling-01.pele.svg) | [<img src="docs/render-comparison/previews/eventmodeling-01.mermaid.png" width="240" alt="eventmodeling-01 mermaid">](docs/render-comparison/svg/eventmodeling-01.mermaid.svg) |
| **gantt**<br>[gantt-01](docs/render-comparison/sources/gantt-01.mmd) | [<img src="docs/render-comparison/previews/gantt-01.rust.png" width="240" alt="gantt-01 rust">](docs/render-comparison/svg/gantt-01.rust.svg) | [<img src="docs/render-comparison/previews/gantt-01.pele.png" width="240" alt="gantt-01 pele">](docs/render-comparison/svg/gantt-01.pele.svg) | [<img src="docs/render-comparison/previews/gantt-01.mermaid.png" width="240" alt="gantt-01 mermaid">](docs/render-comparison/svg/gantt-01.mermaid.svg) |
| **git**<br>[git-01](docs/render-comparison/sources/git-01.mmd) | [<img src="docs/render-comparison/previews/git-01.rust.png" width="240" alt="git-01 rust">](docs/render-comparison/svg/git-01.rust.svg) | [<img src="docs/render-comparison/previews/git-01.pele.png" width="240" alt="git-01 pele">](docs/render-comparison/svg/git-01.pele.svg) | [<img src="docs/render-comparison/previews/git-01.mermaid.png" width="240" alt="git-01 mermaid">](docs/render-comparison/svg/git-01.mermaid.svg) |
| **info**<br>[info-01](docs/render-comparison/sources/info-01.mmd) | [<img src="docs/render-comparison/previews/info-01.rust.png" width="240" alt="info-01 rust">](docs/render-comparison/svg/info-01.rust.svg) | [<img src="docs/render-comparison/previews/info-01.pele.png" width="240" alt="info-01 pele">](docs/render-comparison/svg/info-01.pele.svg) | [<img src="docs/render-comparison/previews/info-01.mermaid.png" width="240" alt="info-01 mermaid">](docs/render-comparison/svg/info-01.mermaid.svg) |
| **journey**<br>[journey-01](docs/render-comparison/sources/journey-01.mmd) | [<img src="docs/render-comparison/previews/journey-01.rust.png" width="240" alt="journey-01 rust">](docs/render-comparison/svg/journey-01.rust.svg) | [<img src="docs/render-comparison/previews/journey-01.pele.png" width="240" alt="journey-01 pele">](docs/render-comparison/svg/journey-01.pele.svg) | [<img src="docs/render-comparison/previews/journey-01.mermaid.png" width="240" alt="journey-01 mermaid">](docs/render-comparison/svg/journey-01.mermaid.svg) |
| **kanban**<br>[kanban-01](docs/render-comparison/sources/kanban-01.mmd) | [<img src="docs/render-comparison/previews/kanban-01.rust.png" width="240" alt="kanban-01 rust">](docs/render-comparison/svg/kanban-01.rust.svg) | [<img src="docs/render-comparison/previews/kanban-01.pele.png" width="240" alt="kanban-01 pele">](docs/render-comparison/svg/kanban-01.pele.svg) | [<img src="docs/render-comparison/previews/kanban-01.mermaid.png" width="240" alt="kanban-01 mermaid">](docs/render-comparison/svg/kanban-01.mermaid.svg) |
| **mindmap**<br>[mindmap-01](docs/render-comparison/sources/mindmap-01.mmd) | [<img src="docs/render-comparison/previews/mindmap-01.rust.png" width="240" alt="mindmap-01 rust">](docs/render-comparison/svg/mindmap-01.rust.svg) | [<img src="docs/render-comparison/previews/mindmap-01.pele.png" width="240" alt="mindmap-01 pele">](docs/render-comparison/svg/mindmap-01.pele.svg) | [<img src="docs/render-comparison/previews/mindmap-01.mermaid.png" width="240" alt="mindmap-01 mermaid">](docs/render-comparison/svg/mindmap-01.mermaid.svg) |
| **packet**<br>[packet-01](docs/render-comparison/sources/packet-01.mmd) | [<img src="docs/render-comparison/previews/packet-01.rust.png" width="240" alt="packet-01 rust">](docs/render-comparison/svg/packet-01.rust.svg) | [<img src="docs/render-comparison/previews/packet-01.pele.png" width="240" alt="packet-01 pele">](docs/render-comparison/svg/packet-01.pele.svg) | [<img src="docs/render-comparison/previews/packet-01.mermaid.png" width="240" alt="packet-01 mermaid">](docs/render-comparison/svg/packet-01.mermaid.svg) |
| **pie**<br>[pie-01](docs/render-comparison/sources/pie-01.mmd) | [<img src="docs/render-comparison/previews/pie-01.rust.png" width="240" alt="pie-01 rust">](docs/render-comparison/svg/pie-01.rust.svg) | [<img src="docs/render-comparison/previews/pie-01.pele.png" width="240" alt="pie-01 pele">](docs/render-comparison/svg/pie-01.pele.svg) | [<img src="docs/render-comparison/previews/pie-01.mermaid.png" width="240" alt="pie-01 mermaid">](docs/render-comparison/svg/pie-01.mermaid.svg) |
| **quadrant**<br>[quadrant-01](docs/render-comparison/sources/quadrant-01.mmd) | [<img src="docs/render-comparison/previews/quadrant-01.rust.png" width="240" alt="quadrant-01 rust">](docs/render-comparison/svg/quadrant-01.rust.svg) | [<img src="docs/render-comparison/previews/quadrant-01.pele.png" width="240" alt="quadrant-01 pele">](docs/render-comparison/svg/quadrant-01.pele.svg) | [<img src="docs/render-comparison/previews/quadrant-01.mermaid.png" width="240" alt="quadrant-01 mermaid">](docs/render-comparison/svg/quadrant-01.mermaid.svg) |
| **requirement**<br>[requirement-01](docs/render-comparison/sources/requirement-01.mmd) | [<img src="docs/render-comparison/previews/requirement-01.rust.png" width="240" alt="requirement-01 rust">](docs/render-comparison/svg/requirement-01.rust.svg) | [<img src="docs/render-comparison/previews/requirement-01.pele.png" width="240" alt="requirement-01 pele">](docs/render-comparison/svg/requirement-01.pele.svg) | [<img src="docs/render-comparison/previews/requirement-01.mermaid.png" width="240" alt="requirement-01 mermaid">](docs/render-comparison/svg/requirement-01.mermaid.svg) |
| **sankey**<br>[sankey-01](docs/render-comparison/sources/sankey-01.mmd) | [<img src="docs/render-comparison/previews/sankey-01.rust.png" width="240" alt="sankey-01 rust">](docs/render-comparison/svg/sankey-01.rust.svg) | [<img src="docs/render-comparison/previews/sankey-01.pele.png" width="240" alt="sankey-01 pele">](docs/render-comparison/svg/sankey-01.pele.svg) | [<img src="docs/render-comparison/previews/sankey-01.mermaid.png" width="240" alt="sankey-01 mermaid">](docs/render-comparison/svg/sankey-01.mermaid.svg) |
| **sequence**<br>[sequence-01](docs/render-comparison/sources/sequence-01.mmd) | [<img src="docs/render-comparison/previews/sequence-01.rust.png" width="240" alt="sequence-01 rust">](docs/render-comparison/svg/sequence-01.rust.svg) | [<img src="docs/render-comparison/previews/sequence-01.pele.png" width="240" alt="sequence-01 pele">](docs/render-comparison/svg/sequence-01.pele.svg) | [<img src="docs/render-comparison/previews/sequence-01.mermaid.png" width="240" alt="sequence-01 mermaid">](docs/render-comparison/svg/sequence-01.mermaid.svg) |
| **state**<br>[state-01](docs/render-comparison/sources/state-01.mmd) | [<img src="docs/render-comparison/previews/state-01.rust.png" width="240" alt="state-01 rust">](docs/render-comparison/svg/state-01.rust.svg) | [<img src="docs/render-comparison/previews/state-01.pele.png" width="240" alt="state-01 pele">](docs/render-comparison/svg/state-01.pele.svg) | [<img src="docs/render-comparison/previews/state-01.mermaid.png" width="240" alt="state-01 mermaid">](docs/render-comparison/svg/state-01.mermaid.svg) |
| **timeline**<br>[timeline-01](docs/render-comparison/sources/timeline-01.mmd) | [<img src="docs/render-comparison/previews/timeline-01.rust.png" width="240" alt="timeline-01 rust">](docs/render-comparison/svg/timeline-01.rust.svg) | [<img src="docs/render-comparison/previews/timeline-01.pele.png" width="240" alt="timeline-01 pele">](docs/render-comparison/svg/timeline-01.pele.svg) | [<img src="docs/render-comparison/previews/timeline-01.mermaid.png" width="240" alt="timeline-01 mermaid">](docs/render-comparison/svg/timeline-01.mermaid.svg) |
| **xychart**<br>[xychart-01](docs/render-comparison/sources/xychart-01.mmd) | [<img src="docs/render-comparison/previews/xychart-01.rust.png" width="240" alt="xychart-01 rust">](docs/render-comparison/svg/xychart-01.rust.svg) | [<img src="docs/render-comparison/previews/xychart-01.pele.png" width="240" alt="xychart-01 pele">](docs/render-comparison/svg/xychart-01.pele.svg) | [<img src="docs/render-comparison/previews/xychart-01.mermaid.png" width="240" alt="xychart-01 mermaid">](docs/render-comparison/svg/xychart-01.mermaid.svg) |
| **agentflow**<br>[agentflow-01](docs/render-comparison/sources/agentflow-01.mmd) | [<img src="docs/render-comparison/previews/agentflow-01.rust.png" width="240" alt="agentflow-01 rust">](docs/render-comparison/svg/agentflow-01.rust.svg) | [<img src="docs/render-comparison/previews/agentflow-01.pele.png" width="240" alt="agentflow-01 pele">](docs/render-comparison/svg/agentflow-01.pele.svg) | [<img src="docs/render-comparison/previews/agentflow-01.mermaid.png" width="240" alt="agentflow-01 mermaid">](docs/render-comparison/svg/agentflow-01.mermaid.svg) |
| **architecture**<br>[architecture-01](docs/render-comparison/sources/architecture-01.mmd) | [<img src="docs/render-comparison/previews/architecture-01.rust.png" width="240" alt="architecture-01 rust">](docs/render-comparison/svg/architecture-01.rust.svg) | [<img src="docs/render-comparison/previews/architecture-01.pele.png" width="240" alt="architecture-01 pele">](docs/render-comparison/svg/architecture-01.pele.svg) | [<img src="docs/render-comparison/previews/architecture-01.mermaid.png" width="240" alt="architecture-01 mermaid">](docs/render-comparison/svg/architecture-01.mermaid.svg) |
| **cynefin**<br>[cynefin-01](docs/render-comparison/sources/cynefin-01.mmd) | [<img src="docs/render-comparison/previews/cynefin-01.rust.png" width="240" alt="cynefin-01 rust">](docs/render-comparison/svg/cynefin-01.rust.svg) | [<img src="docs/render-comparison/previews/cynefin-01.pele.png" width="240" alt="cynefin-01 pele">](docs/render-comparison/svg/cynefin-01.pele.svg) | [<img src="docs/render-comparison/previews/cynefin-01.mermaid.png" width="240" alt="cynefin-01 mermaid">](docs/render-comparison/svg/cynefin-01.mermaid.svg) |
| **ishikawa**<br>[ishikawa-01](docs/render-comparison/sources/ishikawa-01.mmd) | [<img src="docs/render-comparison/previews/ishikawa-01.rust.png" width="240" alt="ishikawa-01 rust">](docs/render-comparison/svg/ishikawa-01.rust.svg) | [<img src="docs/render-comparison/previews/ishikawa-01.pele.png" width="240" alt="ishikawa-01 pele">](docs/render-comparison/svg/ishikawa-01.pele.svg) | [<img src="docs/render-comparison/previews/ishikawa-01.mermaid.png" width="240" alt="ishikawa-01 mermaid">](docs/render-comparison/svg/ishikawa-01.mermaid.svg) |
| **radar**<br>[radar-01](docs/render-comparison/sources/radar-01.mmd) | [<img src="docs/render-comparison/previews/radar-01.rust.png" width="240" alt="radar-01 rust">](docs/render-comparison/svg/radar-01.rust.svg) | [<img src="docs/render-comparison/previews/radar-01.pele.png" width="240" alt="radar-01 pele">](docs/render-comparison/svg/radar-01.pele.svg) | [<img src="docs/render-comparison/previews/radar-01.mermaid.png" width="240" alt="radar-01 mermaid">](docs/render-comparison/svg/radar-01.mermaid.svg) |
| **railroad**<br>[railroad-01](docs/render-comparison/sources/railroad-01.mmd) | [<img src="docs/render-comparison/previews/railroad-01.rust.png" width="240" alt="railroad-01 rust">](docs/render-comparison/svg/railroad-01.rust.svg) | [<img src="docs/render-comparison/previews/railroad-01.pele.png" width="240" alt="railroad-01 pele">](docs/render-comparison/svg/railroad-01.pele.svg) | [<img src="docs/render-comparison/previews/railroad-01.mermaid.png" width="240" alt="railroad-01 mermaid">](docs/render-comparison/svg/railroad-01.mermaid.svg) |
| **swimlane**<br>[swimlane-01](docs/render-comparison/sources/swimlane-01.mmd) | [<img src="docs/render-comparison/previews/swimlane-01.rust.png" width="240" alt="swimlane-01 rust">](docs/render-comparison/svg/swimlane-01.rust.svg) | [<img src="docs/render-comparison/previews/swimlane-01.pele.png" width="240" alt="swimlane-01 pele">](docs/render-comparison/svg/swimlane-01.pele.svg) | [<img src="docs/render-comparison/previews/swimlane-01.mermaid.png" width="240" alt="swimlane-01 mermaid">](docs/render-comparison/svg/swimlane-01.mermaid.svg) |
| **treemap**<br>[treemap-01](docs/render-comparison/sources/treemap-01.mmd) | [<img src="docs/render-comparison/previews/treemap-01.rust.png" width="240" alt="treemap-01 rust">](docs/render-comparison/svg/treemap-01.rust.svg) | [<img src="docs/render-comparison/previews/treemap-01.pele.png" width="240" alt="treemap-01 pele">](docs/render-comparison/svg/treemap-01.pele.svg) | [<img src="docs/render-comparison/previews/treemap-01.mermaid.png" width="240" alt="treemap-01 mermaid">](docs/render-comparison/svg/treemap-01.mermaid.svg) |
| **treeview**<br>[treeview-01](docs/render-comparison/sources/treeview-01.mmd) | [<img src="docs/render-comparison/previews/treeview-01.rust.png" width="240" alt="treeview-01 rust">](docs/render-comparison/svg/treeview-01.rust.svg) | [<img src="docs/render-comparison/previews/treeview-01.pele.png" width="240" alt="treeview-01 pele">](docs/render-comparison/svg/treeview-01.pele.svg) | [<img src="docs/render-comparison/previews/treeview-01.mermaid.png" width="240" alt="treeview-01 mermaid">](docs/render-comparison/svg/treeview-01.mermaid.svg) |
| **usecase**<br>[usecase-01](docs/render-comparison/sources/usecase-01.mmd) | [<img src="docs/render-comparison/previews/usecase-01.rust.png" width="240" alt="usecase-01 rust">](docs/render-comparison/svg/usecase-01.rust.svg) | [<img src="docs/render-comparison/previews/usecase-01.pele.png" width="240" alt="usecase-01 pele">](docs/render-comparison/svg/usecase-01.pele.svg) | [<img src="docs/render-comparison/previews/usecase-01.mermaid.png" width="240" alt="usecase-01 mermaid">](docs/render-comparison/svg/usecase-01.mermaid.svg) |
| **venn**<br>[venn-01](docs/render-comparison/sources/venn-01.mmd) | [<img src="docs/render-comparison/previews/venn-01.rust.png" width="240" alt="venn-01 rust">](docs/render-comparison/svg/venn-01.rust.svg) | [<img src="docs/render-comparison/previews/venn-01.pele.png" width="240" alt="venn-01 pele">](docs/render-comparison/svg/venn-01.pele.svg) | [<img src="docs/render-comparison/previews/venn-01.mermaid.png" width="240" alt="venn-01 mermaid">](docs/render-comparison/svg/venn-01.mermaid.svg) |
| **wardley**<br>[wardley-01](docs/render-comparison/sources/wardley-01.mmd) | [<img src="docs/render-comparison/previews/wardley-01.rust.png" width="240" alt="wardley-01 rust">](docs/render-comparison/svg/wardley-01.rust.svg) | [<img src="docs/render-comparison/previews/wardley-01.pele.png" width="240" alt="wardley-01 pele">](docs/render-comparison/svg/wardley-01.pele.svg) | [<img src="docs/render-comparison/previews/wardley-01.mermaid.png" width="240" alt="wardley-01 mermaid">](docs/render-comparison/svg/wardley-01.mermaid.svg) |

<!-- sample-comparison:end -->

## Performance

The comparison uses complete parsing, layout, and SVG generation on the same sources. Each engine runs in three independent processes with alternating order; validation and builds run outside timing. The full table covers every diagram type.

<!-- output-contract-benchmark:start -->
Across all **167 samples and 33 types**, the equal-weight geometric mean Rust/Pele speedup is **1.76× at API return** and **1.89× when the output must be ready as contiguous UTF-8 bytes**. The latter includes Node's encoding and Buffer allocation; Rust already returns UTF-8. See the [output-contract benchmark](docs/OUTPUT-BENCHMARK.md) for the complete table and method.
<!-- output-contract-benchmark:end -->

Continuous optimization remains stopped. This release receives a finite verification run; complete upstream-test acceptance remains unfinished.

Compiled code size is also an optimization target, alongside the full 33-type 2× performance target. Current full-feature release sizes (unstripped, thin LTO, one codegen unit):

| Artifact | File bytes | Instruction text bytes |
| --- | ---: | ---: |
| CLI | 9,237,040 | 5,649,348 |
| FFI dynamic library | 8,811,776 | 5,326,912 |
| FFI static archive | 62,027,464 | 6,400,400 |

Built-in math adds parser, layout, SVG generation, and bundled font data to these artifacts. The static archive includes LLVM bitcode and unselected object code; its size is not the final host application's link increment. These are the current [release-size results](docs/CODE-SIZE-OPTIMIZATION.md), with paired comparisons, fingerprints and section totals. Size, speed, allocations and RSS are reported separately; all diagram types and correctness requirements remain enabled.

## Library API

Add the library to your application's `Cargo.toml`. Both installation examples pin the same revision for reproducible builds:

```toml
[dependencies]
zebra-mermaid = { git = "https://github.com/entropysniper-app/zebra-mermaid", rev = "2f2c3b097dcaa1e9d15a448f9cda0822049301a3" }
```

```rust
use zebra_mermaid::{
    parse, render_parsed, ColorMode, MermaidThemeMode, ParseOptions, RenderOptions,
};

fn main() -> Result<(), zebra_mermaid::Error> {
    let diagram = parse("graph TD\n  A[Start] --> B[Done]", &ParseOptions::default())?;
    let options = RenderOptions {
        color_mode: ColorMode::Resolved,
        theme: MermaidThemeMode::Dark,
        ..Default::default()
    };
    let result = render_parsed(&diagram, &options)?;

    assert_eq!(result.diagram_type.as_str(), "flowchart");
    assert_eq!(result.mermaid_type.as_str(), "flowchart-v2");
    println!("{}", result.svg);
    Ok(())
}
```

Use `render(source, &options)` for a single call, or keep a parsed diagram and render it repeatedly with different appearance or width options. `RenderCache` adds bounded caching. A host can provide its own `TextMeasurer`; the default uses deterministic Arial advance metrics.

### Mathematics

Write TeX between `$$` delimiters in labels. Math renders by default in flowchart, sequence, state, class, ER, mindmap, kanban, block, requirement, usecase, and swimlane diagrams:

```mermaid
flowchart LR
  A["Energy $$E=mc^2$$"] --> B["Ratio $$\frac{a}{b}$$"]
```

No math callback or JavaScript typesetter is needed. RaTeX 0.1.15 performs parsing and layout in Rust; bundled KaTeX fonts become SVG glyph outlines, so ordinary formulas need no external fonts or stylesheet. Formula dimensions participate in wrapping, node sizing, and line spacing. Native output uses concrete theme colors. Diagram titles, class members, and other diagram types retain plain text; invalid TeX and unavailable glyphs remain visible as source. Native image restrictions also apply to math.

[Open a standalone math example](docs/math/flowchart.svg) · [Source](docs/math/flowchart.mmd)

### Themes and native integration

The library defaults to CSS variables (`--pele-*`); the CLI defaults to resolved colors for standalone SVGs. Choose `ColorMode::Resolved` for native image consumers, supply your host's current `MermaidPalette`, and select an opaque or transparent canvas. No separate theme file is required for the Zebra host; `--palette` is an optional CLI input.

The `zebra` adapter preserves the host's theme and width-independent layout contract. The C ABI v2 interface is available in [`zebra_mermaid.h`](crates/zebra-mermaid-ffi/include/zebra_mermaid.h). App integration is tracked separately from library and ABI tests.

## Validation

Diagram coverage, output equality, and original-test acceptance are separate measures:

| Measure | Current result |
| --- | --- |
| Diagram types enabled by default | **33/33** |
| Documentation response comparisons | **761/761**: 702 successful renders and 59 expected rejections |
| Workspace tests | **3,271 passed**, 0 failed, 5 ignored |
| Original-test acceptance | **2,808/6,971 passed** (40.28%); 7 preserved skips, 4,156 unmapped |

**Full original-test acceptance is incomplete.** Two tests execute successfully in Rust but retain their original skip status and are excluded from acceptance passes. Rendering samples and benchmark results do not replace this requirement. See the [test mapping](docs/UPSTREAM-TEST-PARITY.md), [implementation status](docs/IMPLEMENTATION-STATUS.md), and [validation evidence](docs/research/pele-1.9.0-alignment-2026-10-23/validation.json).

```sh
cargo test --workspace --all-features --locked -- --test-threads=1
cargo test -p zebra-mermaid --no-default-features --test mermaid_type
node tools/test-parity/check.mjs --run
node tools/test-parity/check.mjs --require-complete
```

The last command enforces each original test's timeout and exits nonzero while unmapped or skipped cases remain. Node is used only for development references, fixture generation, and parity auditing.

## Project layout

| Path | Purpose |
| --- | --- |
| `crates/zebra-mermaid` | Parsing, layout, text measurement, SVG output, themes, and cache |
| `crates/zebra-mermaid-cli` | Single-file, validation, detection, and batch commands |
| `crates/zebra-mermaid-ffi` | C ABI v2 bridge and header |
| `tests/upstream` | Frozen original tests and individual Rust mappings |
| `tests/fixtures` | Independent reference inputs and expected outputs |
| `tools/compare` | Rendering gallery and README preview generation |
| `tools/bench` | Reproducible performance comparisons |

Development checkouts and local browser caches live in the ignored `.research/` directory.

## License

MIT. See [third-party notices](THIRD_PARTY_NOTICES.md) for attribution and dependency licenses.
