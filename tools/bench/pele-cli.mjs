// Process-start reference helper. Pele itself is a library, not this CLI.
import { readFileSync } from 'node:fs';
import { render } from '../../.research/pele/dist/pele.js';
process.env.TZ='UTC';
process.stdout.write(render(readFileSync(0,'utf8'),{now:1792368000000}).svg+'\n');
