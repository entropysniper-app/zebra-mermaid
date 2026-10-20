# zebra-mermaid

A pure Rust library and CLI for rendering Mermaid diagrams to SVG. Ported from [Pele](https://github.com/obsidianmd/pele/tree/8603bf3313add9143cd417015d4a6bd6c522dada) and aligned with **1.7.0**, targeting **Mermaid 12.1.0** syntax.

- **33 diagram types** enabled by default, with Cargo features for smaller builds.
- **Native rendering** with no JavaScript, browser, or external layout process at runtime.
- **Host-controlled appearance** with CSS variables for web content and resolved colors for native SVG consumers.
- **Library, CLI, and C ABI** with parsed-model reuse, an optional render cache, batch conversion, and structured errors.

[Quick start](#quick-start) · [Rendering comparison](#rendering-comparison) · [Performance](#performance) · [Library API](#library-api) · [Validation](#validation)

## Quick start

Requires Rust 1.96 or later; verified with rustc 1.96.1. Install directly from GitHub:

```sh
cargo install --git https://github.com/entropysniper-app/zebra-mermaid \
  --rev e0a9e608a2c19b0066b6710b76e4f95dc81c1926 \
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

<!-- four-engine-benchmark:start -->
## Performance

Current-version results on identical inputs, measured as **milliseconds per complete batch**. Each value is the median of three independent runs; parsing, layout, and SVG generation are included, while startup and warm-up are excluded. Lower is faster within the same row.

| Diagram | Shared / total samples | Mermaid.js (ms) | Pele (ms) | zebra-mermaid (ms) | mermaid-rs-renderer (ms) |
| --- | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 / 6 | 90.40000 | 0.31368 | 0.19788 | 93.69671 |
| block | 6 / 6 | 82.00000 | 0.09453 | 0.07110 | 0.16108 |
| c4 | 6 / 6 | 100.20000 | 0.46335 | 0.31600 | 1.13248 |
| class | 5 / 6 | 125.10000 | 0.23300 | 0.15064 | 0.38698 |
| er | 6 / 6 | 83.40000 | 0.24643 | 0.15743 | 6.47056 |
| eventmodeling | 0 / 6 | — | — | — | — |
| gantt | 6 / 6 | 16.40000 | 0.26032 | 0.14723 | 0.40528 |
| git | 6 / 6 | 107.00000 | 0.42475 | 0.21608 | 0.62659 |
| info | 0 / 2 | — | — | — | — |
| journey | 1 / 1 | 3.00000 | 0.01599 | 0.00851 | 0.04095 |
| kanban | 3 / 3 | 26.70000 | 0.05185 | 0.03761 | 0.12656 |
| mindmap | 6 / 6 | 86.50000 | 0.09944 | 0.05227 | 0.35289 |
| packet | 4 / 4 | 6.65000 | 0.06471 | 0.03052 | 2.36506 |
| pie | 3 / 3 | 4.70000 | 0.03312 | 0.01876 | 0.07304 |
| quadrant | 3 / 3 | 8.95000 | 0.05430 | 0.03420 | 0.08506 |
| requirement | 6 / 6 | 85.20000 | 0.23241 | 0.14683 | 1.84171 |
| sankey | 6 / 6 | 32.10000 | 0.36112 | 0.10551 | 0.43326 |
| sequence | 4 / 6 | 24.60000 | 0.08942 | 0.05784 | 0.71722 |
| state | 6 / 6 | 125.70000 | 0.32237 | 0.19279 | 5.45860 |
| timeline | 6 / 6 | 30.30000 | 0.20446 | 0.10524 | 0.38077 |
| xychart | 6 / 6 | 25.90000 | 0.13866 | 0.07540 | 0.21637 |
| agentflow | 0 / 6 | — | — | — | — |
| architecture | 6 / 6 | 72.50000 | 0.22967 | 0.09720 | 0.26008 |
| cynefin | 0 / 3 | — | — | — | — |
| ishikawa | 0 / 1 | — | — | — | — |
| radar | 3 / 3 | 6.15000 | 0.08749 | 0.03203 | 0.09142 |
| railroad | 0 / 6 | — | — | — | — |
| swimlane | 0 / 6 | — | — | — | — |
| treemap | 6 / 6 | 15.50000 | 0.16388 | 0.06758 | 0.25794 |
| treeview | 0 / 6 | — | — | — | — |
| usecase | 0 / 6 | — | — | — | — |
| venn | 0 / 6 | — | — | — | — |
| wardley | 0 / 6 | — | — | — | — |

The shared benchmark set contains **110/167 samples across 22 diagram types**. A dash means no input passed type and SVG checks in all four engines. Layout and font measurement differ between engines. See the [benchmark report](docs/BENCHMARK.md) for versions, coverage, methodology, and raw results.
<!-- four-engine-benchmark:end -->

<!-- output-contract-benchmark:start -->
Across all **167 samples and 33 types**, the equal-weight geometric mean Rust/Pele speedup is **1.79× at API return** and **1.89× when the output must be ready as contiguous UTF-8 bytes**. The latter includes Node's encoding and Buffer allocation; Rust already returns UTF-8. See the [output-contract benchmark](docs/OUTPUT-BENCHMARK.md) for the complete table and method.
<!-- output-contract-benchmark:end -->

## Library API

Add the library to your application's `Cargo.toml`. Both installation examples pin the same revision for reproducible builds:

```toml
[dependencies]
zebra-mermaid = { git = "https://github.com/entropysniper-app/zebra-mermaid", rev = "e0a9e608a2c19b0066b6710b76e4f95dc81c1926" }
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

### Themes and native integration

The library defaults to CSS variables (`--pele-*`); the CLI defaults to resolved colors for standalone SVGs. Choose `ColorMode::Resolved` for native image consumers, supply your host's current `MermaidPalette`, and select an opaque or transparent canvas. No separate theme file is required for the Zebra host; `--palette` is an optional CLI input.

The `zebra` adapter preserves the host's theme and width-independent layout contract. The C ABI v2 interface is available in [`zebra_mermaid.h`](crates/zebra-mermaid-ffi/include/zebra_mermaid.h). App integration is tracked separately from library and ABI tests.

## Validation

Diagram coverage, output equality, and original-test acceptance are separate measures:

| Measure | Current result |
| --- | --- |
| Diagram types enabled by default | **33/33** |
| Documentation response comparisons | **761/761**: 702 successful renders and 59 expected rejections |
| Workspace tests | **3,226 passed**, 0 failed, 5 ignored |
| Original-test acceptance | **2,785/6,826 passed** (40.80%); 7 preserved skips, 4,034 unmapped |

**Full original-test acceptance is incomplete.** Two tests execute successfully in Rust but retain their original skip status and are excluded from acceptance passes. Rendering samples and benchmark results do not replace this requirement. See the [test mapping](docs/UPSTREAM-TEST-PARITY.md), [implementation status](docs/IMPLEMENTATION-STATUS.md), and [validation evidence](docs/research/svg-adapter-optimization-2026-10-20/validation.json).

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
