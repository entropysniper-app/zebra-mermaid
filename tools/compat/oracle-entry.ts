// Entry point bundled by tools/compat/oracle.mjs. It re-exports the public API and the
// internals of the pinned Pele snapshot that the Rust port is compared against.
export { render, parse, detectType, register } from '../../.research/pele/src/core.js';
export { all } from '../../.research/pele/src/diagrams/registry.js';
export { preprocess, encodeEntities } from '../../.research/pele/src/preprocess.js';
export { parseYaml } from '../../.research/pele/src/util/yaml.js';
export { sanitizeUrl, safeUrl } from '../../.research/pele/src/util/url.js';
export { layoutLabel } from '../../.research/pele/src/text/label.js';
export { metricsMeasurer } from '../../.research/pele/src/text/measurer.js';
export { decodeEntities } from '../../.research/pele/src/text/entities.js';
export { num, esc } from '../../.research/pele/src/svg/builder.js';
export { resolveStyle } from '../../.research/pele/src/svg/theme.js';
