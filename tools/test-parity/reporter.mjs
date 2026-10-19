// Vitest reporter for the fixed Pele reference; records every collected test, including skip/todo.
import { writeFileSync, readFileSync } from 'node:fs';
import { relative } from 'node:path';
import { createHash } from 'node:crypto';

export default class ParityReporter {
  onInit(ctx) { this.root = ctx.config.root; }
  onTestRunEnd(modules, errors, reason) {
    const files = modules.map(module => {
      const path = relative(this.root, module.moduleId).replaceAll('\\', '/');
      const sourceSha256 = createHash('sha256').update(readFileSync(module.moduleId)).digest('hex');
      const tests = [...module.children.allTests()].map((test, index) => {
        const ancestry = [];
        for (let parent = test.parent; parent.type !== 'module'; parent = parent.parent) ancestry.unshift(parent.name);
        return {
          index, name: test.name, ancestry, fullName: test.fullName,
          location: test.location ?? null, options: test.options, result: test.result(),
        };
      });
      return { path, sourceSha256, errors: module.errors(), tests };
    }).sort((a,b) => a.path.localeCompare(b.path, 'en'));
    if (!process.env.PELE_PARITY_REPORT) throw new Error('PELE_PARITY_REPORT must name the output file');
    writeFileSync(process.env.PELE_PARITY_REPORT, JSON.stringify({ reason, errors, files }, null, 2) + '\n');
  }
}
