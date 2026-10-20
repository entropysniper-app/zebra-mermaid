import {readFileSync,writeFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {resolve} from 'node:path';
const root=fileURLToPath(new URL('../..',import.meta.url)),dir=resolve(root,'docs/render-comparison');
const report=JSON.parse(readFileSync(resolve(dir,'results.json'),'utf8'));
const data=report.results,types=[...new Set(data.map(r=>r.type))];
const template=readFileSync(resolve(root,'tools/compare/report-template.html'),'utf8');
const embedded=JSON.stringify(report).replaceAll('<','\\u003c');
writeFileSync(resolve(dir,'index.html'),template.replace('__PELE_VERSION__',report.metadata.pele.version).replace('__REPORT_DATA__',()=>embedded));
let md=`# Rendering comparison

[Open the interactive gallery](index.html). Each row shows the same source rendered by zebra-mermaid, Pele, and Mermaid.js. Filter by diagram type, source, or result, and click a preview to inspect the full SVG.

The sample set is selected by the unmodified \`sampleGroups()\` function at commit \`${report.metadata.pele.commit}\`: **${types.length} diagram types and ${data.length} samples**, including its built-in examples and original selection order. Engine versions: zebra-mermaid 0.1.0, Pele ${report.metadata.pele.version}, Mermaid.js ${report.metadata.mermaid.version}.

| Diagram | Samples | zebra-mermaid | Pele | Mermaid.js | Identical Rust/Pele SVG and dimensions |
| --- | ---: | ---: | ---: | ---: | ---: |
`;
for(const type of types){const rows=data.filter(r=>r.type===type);md+=`| ${type} | ${rows.length} | ${rows.filter(r=>r.rust.ok).length} | ${rows.filter(r=>r.pele.ok).length} | ${rows.filter(r=>r.mermaid.ok).length} | ${rows.filter(r=>r.rustPeleEqual).length}/${rows.length} |\n`}
md+=`| **Total** | **${data.length}** | **${data.filter(r=>r.rust.ok).length}** | **${data.filter(r=>r.pele.ok).length}** | **${data.filter(r=>r.mermaid.ok).length}** | **${data.filter(r=>r.rustPeleEqual).length}/${data.length}** |\n\n`;
md+=`## Method

The gallery uses the Playground's default dark appearance. Its original base, theme, and playground stylesheets are compiled directly, and the browser supplies the resulting CSS variables. Rust/Pele share the system font stack, site icons, and browser font measurements; their parsing, layout, and SVG generation remain independent. Mermaid.js uses its own layout with \`theme: dark\`. Authored front matter remains active.

The canvas is 760 px wide with 712 px available for layout. Time is fixed to 2026-10-19 00:00 UTC. Style values and source hashes are recorded in [playground-style.json](playground-style.json).

${data.filter(r=>r.mermaid.rawXmlValid===false).length} Mermaid.js responses contain HTML labels. They are inserted into the browser DOM and exported with XMLSerializer so the SVG files can open independently. Original returned strings are also preserved. This export step is outside rendering comparisons and does not count as a rendering failure.

[results.json](results.json) records source files, dimensions, raw and exported SVG hashes, blocked external requests, and warnings. Rust/Pele use the original site icon resolver; no extra icon packs are registered for Mermaid.js. Sample success does not establish complete original-test acceptance or semantic and pixel equivalence across all engines.

## Reproduce

Use the pinned reference checkout, its locked website dependencies, and Playwright with Chromium installed:

\`\`\`sh
cargo build --release --locked -p zebra-mermaid --example playground_sample
node tools/compare/render-samples.mjs
node tools/compare/build-report.mjs
node tools/compare/readme-previews.mjs
\`\`\`

Set \`PLAYWRIGHT_MODULE\` and \`PLAYWRIGHT_BROWSERS_PATH\` if using a separate installation. \`CARGO_TARGET_DIR\` can select another build directory. Open \`index.html\` locally or serve it with any static HTTP server.
`;
writeFileSync(resolve(dir,'summary.md'),md);
console.log(JSON.stringify({samples:data.length,rust:data.filter(r=>r.rust.ok).length,pele:data.filter(r=>r.pele.ok).length,mermaid:data.filter(r=>r.mermaid.ok).length,equal:data.filter(r=>r.rustPeleEqual).length}));
