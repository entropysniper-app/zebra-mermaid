# Rendering comparison

[Open the interactive gallery](index.html). Each row shows the same source rendered by zebra-mermaid, Pele, and Mermaid.js. Filter by diagram type, source, or result, and click a preview to inspect the full SVG.

The sample set is selected by the unmodified `sampleGroups()` function at commit `8603bf3313add9143cd417015d4a6bd6c522dada`: **33 diagram types and 167 samples**, including its built-in examples and original selection order. Engine versions: zebra-mermaid 0.1.0, Pele 1.7.0, Mermaid.js 12.1.0.

| Diagram | Samples | zebra-mermaid | Pele | Mermaid.js | Identical Rust/Pele SVG and dimensions |
| --- | ---: | ---: | ---: | ---: | ---: |
| flowchart | 6 | 6 | 6 | 6 | 6/6 |
| block | 6 | 6 | 6 | 6 | 6/6 |
| c4 | 6 | 6 | 6 | 6 | 6/6 |
| class | 6 | 6 | 6 | 6 | 6/6 |
| er | 6 | 6 | 6 | 6 | 6/6 |
| eventmodeling | 6 | 6 | 6 | 6 | 6/6 |
| gantt | 6 | 6 | 6 | 6 | 6/6 |
| git | 6 | 6 | 6 | 6 | 6/6 |
| info | 2 | 2 | 2 | 2 | 2/2 |
| journey | 1 | 1 | 1 | 1 | 1/1 |
| kanban | 3 | 3 | 3 | 3 | 3/3 |
| mindmap | 6 | 6 | 6 | 6 | 6/6 |
| packet | 4 | 4 | 4 | 4 | 4/4 |
| pie | 3 | 3 | 3 | 3 | 3/3 |
| quadrant | 3 | 3 | 3 | 3 | 3/3 |
| requirement | 6 | 6 | 6 | 6 | 6/6 |
| sankey | 6 | 6 | 6 | 6 | 6/6 |
| sequence | 6 | 6 | 6 | 6 | 6/6 |
| state | 6 | 6 | 6 | 6 | 6/6 |
| timeline | 6 | 6 | 6 | 6 | 6/6 |
| xychart | 6 | 6 | 6 | 6 | 6/6 |
| agentflow | 6 | 6 | 6 | 6 | 6/6 |
| architecture | 6 | 6 | 6 | 6 | 6/6 |
| cynefin | 3 | 3 | 3 | 3 | 3/3 |
| ishikawa | 1 | 1 | 1 | 1 | 1/1 |
| radar | 3 | 3 | 3 | 3 | 3/3 |
| railroad | 6 | 6 | 6 | 6 | 6/6 |
| swimlane | 6 | 6 | 6 | 6 | 6/6 |
| treemap | 6 | 6 | 6 | 6 | 6/6 |
| treeview | 6 | 6 | 6 | 6 | 6/6 |
| usecase | 6 | 6 | 6 | 6 | 6/6 |
| venn | 6 | 6 | 6 | 6 | 6/6 |
| wardley | 6 | 6 | 6 | 6 | 6/6 |
| **Total** | **167** | **167** | **167** | **167** | **167/167** |

## Method

The gallery uses the Playground's default dark appearance. Its original base, theme, and playground stylesheets are compiled directly, and the browser supplies the resulting CSS variables. Rust/Pele share the system font stack, site icons, and browser font measurements; their parsing, layout, and SVG generation remain independent. Mermaid.js uses its own layout with `theme: dark`. Authored front matter remains active.

The canvas is 760 px wide with 712 px available for layout. Time is fixed to 2026-10-19 00:00 UTC. Style values and source hashes are recorded in [playground-style.json](playground-style.json).

17 Mermaid.js responses contain HTML labels. They are inserted into the browser DOM and exported with XMLSerializer so the SVG files can open independently. Original returned strings are also preserved. This export step is outside rendering comparisons and does not count as a rendering failure.

[results.json](results.json) records source files, dimensions, raw and exported SVG hashes, blocked external requests, and warnings. Rust/Pele use the original site icon resolver; no extra icon packs are registered for Mermaid.js. Sample success does not establish complete original-test acceptance or semantic and pixel equivalence across all engines.

## Reproduce

Use the pinned reference checkout, its locked website dependencies, and Playwright with Chromium installed:

```sh
cargo build --release --locked -p zebra-mermaid --example playground_sample
node tools/compare/render-samples.mjs
node tools/compare/build-report.mjs
node tools/compare/readme-previews.mjs
```

Set `PLAYWRIGHT_MODULE` and `PLAYWRIGHT_BROWSERS_PATH` if using a separate installation. `CARGO_TARGET_DIR` can select another build directory. Open `index.html` locally or serve it with any static HTTP server.
