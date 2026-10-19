# Porting guide: Pele TypeScript → zebra-mermaid Rust

Rules for porting a Pele 1.6.0 module (commit `44da152b`, snapshot in `.research/pele/`) into `crates/zebra-mermaid`. The goal is **byte-identical SVG** to Pele in CSS color mode for every input, plus Zebra semantic roles in resolved mode. Read `docs/ZEBRA-THEME-COMPATIBILITY.md` for the theme contract.

## 1. Fidelity first

- Port the algorithm, not an approximation: same order of operations, same loops, same tie-breaks, same floating-point expression order (`a + b - c` is not `a + (b - c)`). Do not "simplify" arithmetic.
- Every `Math.*` call goes through `crate::util::math` (sin, cos, atan2, pow, hypot, …) or `crate::util::js` (`round`, `max`, `min`, `sign`). Never use `f64::round` (half away from zero) where JS has `Math.round` (half up); use `js::round`. `Math.max/min` propagate NaN: use `js::max/js::min` (f64::max ignores NaN).
- Number to string: `String(n)` / template `${n}` → `js::to_string(n)`; `toFixed` → `js::to_fixed`; `toPrecision` → `js::to_precision`; coordinates → `svg::builder::num(x)` (implements `Display`).
- Parsing numbers: `Number(s)` → `js::number`, `parseFloat` → `js::parse_float`, `parseInt` → `js::parse_int`.
- Strings: JS strings are UTF-16. Rust strings are UTF-8 and offsets are bytes.
  - `s.length` → `js::len16(s)` when the value matters (limits, positions in messages, `slice` arithmetic); `s.slice(a, b)` with UTF-16 offsets → `js::slice16`.
  - Lexers scanning ASCII delimiters can work on bytes; any test of a non-ASCII character class must decode the char (`util::chars::char_at`). A JS loop that steps one code unit past a non-BMP character sees a surrogate; reproduce what the JS condition does on surrogates.
  - `trim/trimStart/trimEnd` → `js::trim*` (JS white space includes U+FEFF and excludes U+0085, unlike Rust's). `\s` → `util::chars::is_space`. `\w` → ASCII word (`chars::is_word`). `.` excludes `\n \r    `.
  - `toLowerCase/toUpperCase` → `to_lowercase/to_uppercase`; regex `/i` without `/u` folds ASCII only.
- Regular expressions: there is no regex crate. Write each sticky/global expression as a hand matcher with the same semantics, including greedy/lazy choice, backtracking and lookarounds. Comment the original expression above the matcher. Watch for quadratic rescans (Pele adds memoizing scanners for those; port the scanner).
- Maps: JS `Map` and object insertion order → `indexmap::IndexMap`; JS objects with array-index keys iterate those first (see `config::ConfigMap::keys`).
- Sorting: JS `Array.prototype.sort` is stable; use `sort_by` (stable) with the same comparator; never `sort_unstable`.
- Recursion: if input can drive recursion depth, add an explicit depth limit returning `Error::limit("Diagram is nested too deeply.")` (Pele's `guarded` turns a JS stack overflow into that error). Rust cannot catch stack overflow.
- No `unsafe`, no `panic!` on input-dependent paths, no `unwrap()` on values derived from input unless provably present. Use `?` with `crate::error::Error`.

## 2. Module layout and contract

A diagram type `x` lives in `crates/zebra-mermaid/src/diagrams/<module>/` (module names are fixed in `diagrams/mod.rs`, e.g. `git_graph`, `quadrant`, `treeview`). It must provide:

```rust
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct XModel { ... }                         // name fixed in diagrams/mod.rs

pub(crate) fn parse(input: &ParseInput) -> Result<XModel, Error>;
pub(crate) fn render(model: &XModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error>;
```

`ParseInput` gives the preprocessed, entity-encoded source ending in `\n` (exactly what Pele's `diagram.parse` receives), the config, the front-matter title, `line_offset` and `limits.max_edges`. `RenderCtx` gives `options` (Pele's `RenderOptions`, `responsive` settled), `measure()` (measurer + icons, replaces Pele's `measurer`), `icons()` (Pele's `options.icons`), `font_size()`, `padding()`, and theming (below).

Errors: `crate::error::syntax_error(DiagramType::X, src, byte_offset, detail, lexical)` reproduces Pele's `syntaxError` (line, UTF-16 column, snippet, message). `Error::semantic(...)`, `Error::limit(...)`, `.with_type(...)`, `.at(src, offset)`. Keep messages byte-identical to Pele.

Shared code already ported (use it, do not duplicate): `text::{layout_label, Label, Span, style}`, `text::entities::decode_entities`, `svg::builder::{esc, esc_text, num, label_svg, span_style, shared_style, text_style}`, `svg::root::{svg_document, Accessible}`, `svg::title::with_title`, `svg::theme::{resolve_style, class_names, series_color (do not call directly, see theming), font_style}`, `util::url::{safe_url, sanitize_url, link_url, image_url, linked, rel_attr, LinkInfo}`, `config::yaml::parse_yaml`, `diagrams::common::{tokens, fit_width, fit, fit_text, literal_label, generics, caseless, icon, scan, accdescr}`. If Pele code you port needs a shared module that is not ported yet, stop and report it instead of writing your own copy.

**Do not edit files outside your diagram's directory** (and your own test files), except where your task says so. Other people edit the shared files at the same time.

## 3. Colors: semantic roles

Every color Pele writes (`var(--_bg)`, `var(--_fg)`, `var(--_m)`, `var(--_l)`, `var(--_s)`, `var(--_a)`, `var(--_b)`, `var(--_c)`, `seriesColor(i)`, `CRITICAL`, `color-mix(...)`) must go through the render context, naming the semantic role of that part:

```rust
ctx.paint(Role::GroupStroke, "var(--_b)")   // CSS mode: the Pele literal; resolved: the role color
ctx.series(i)                               // seriesColor(i) / chart palette
ctx.theming.series_line(i)                  // series used as a line color (branches, curves)
ctx.theming.critical()                      // CRITICAL
ctx.theming.text_on(fill, "var(--_bg)")     // text on a colored fill: contrast-picked in resolved mode
ctx.resolved()                              // true in resolved mode
```

Pick the role by what the part *is*, not by which Pele variable it used: a cluster outline is `GroupStroke` even though Pele writes `var(--_b)`; a connector is `Line`; an arrowhead is `Arrow`; a node fill is `NodeFill`; a node outline is `NodeStroke`; secondary/axis/caption text is `SecondaryText`; a label patch over a line is `Background`; notes are `WarningBackground`/`WarningText`; "critical"/error parts are `DangerBackground`/`DangerText`; done/success `SuccessBackground`/`SuccessText`; active/selected `Selection`. Roles are listed in `crate::theme::Role` with comments. Zebra's existing renderer semantics are in `.research/zebra-theme-contract/Vendor/ZebraMermaidBridge/renderer/src/theme.rs` (`apply_theme`, `apply_layout`, `apply_layout_config`) — follow them where they name a role for the same part (e.g. sequence note = warning, activation = selection/accent).

CSS-mode output must stay byte-identical to Pele. Resolved-mode output must contain no `var(`, `color-mix(`, `--_`, `--pele-`. Where Pele relies on inheritance or a CSS expression that has no single resolved value (e.g. `color-mix` tints), choose the role color in resolved mode and keep the geometry identical between modes. Author styles (`style`, `classDef`, `linkStyle`) are written as Pele writes them (in `style` attributes), so they keep precedence in both modes.

Never post-process the SVG with global string replacement of colors.

## 4. Tests you must add and run

Use a private target dir: `export CARGO_TARGET_DIR=target/agents/<your-name>`. Build only your types: `cargo test -p zebra-mermaid --no-default-features --features "<feature list>" ...` (feature names = type ids, e.g. `gitGraph`, `quadrantChart`, `treeView`; see `crates/zebra-mermaid/Cargo.toml`).

1. Differential (byte-exact against Pele):
   - `--test compat_corpus` (docs corpus, `tests/fixtures/corpus/<type>.jsonl`)
   - `--test compat_harvest` (committed sample) and `ZM_FULL_HARVEST=1 ... --test compat_harvest harvested_full` (all inputs from Pele's own test suite for your type, in `target/compat/harvest/fixtures/<type>.jsonl`; run `node tools/compat/harvest.mjs --fixtures` only if that directory is missing).
   - A case that cannot match must be justified, listed in `tests/fixtures/known-differences.json` (tell the coordinator; do not edit shared files yourself), and never be the result of a shortcut.
   - To inspect what Pele does, use the oracle: `echo '{"op":"render","source":"...","options":{}}' | node tools/compat/oracle.mjs`, or write a small `.mjs` in the repository root that `import { loadOracle } from './tools/compat/oracle.mjs'` and calls internals (`o.render`, `o.parse`, …). The oracle bundles Pele's `src`; to reach other internals, add exports to a private copy of `tools/compat/oracle-entry.ts` and bundle that copy yourself (do not edit the shared entry).
2. Zebra theme (resolved mode): a test file `crates/zebra-mermaid/tests/zebra_theme_<type>.rs` using `common::theme::{sentinel, assert_no_css_runtime, resolved_options}`: render representative inputs with the sentinel palette, assert each applicable role's color appears on the right element/attribute (not just anywhere), list the roles that do not apply and assert they do not appear, assert opaque/transparent differ only by the canvas rectangle, and that only colors differ between two palettes (geometry unchanged).
3. Unit tests for any tricky matcher you hand-wrote.
4. `rustfmt --edition 2021 <your files>` (not `cargo fmt` on the workspace) and `cargo clippy` for your feature set with no new warnings in your files.

## 5. Report

List: files created, differential results per suite (counts, e.g. `412/412 identical`), known differences with reasons, theme roles mapping table for your type (part → role → Pele literal), anything not done. Do not claim results you did not run.
