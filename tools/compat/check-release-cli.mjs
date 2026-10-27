// Release CLI wiring: independent Pele expectations for the four monospace parts,
// and concrete native colors/radii for the new fill-aware rendering paths.
import {readFileSync,writeFileSync} from 'node:fs';
import {execFileSync,spawnSync} from 'node:child_process';
import assert from 'node:assert/strict';
import {loadOracle} from './oracle.mjs';
const {commit}=JSON.parse(readFileSync('tests/upstream/inventory.json'));
assert.equal(execFileSync('git',['-C','.research/pele','rev-parse','HEAD'],{encoding:'utf8'}).trim(),commit);
const {version}=JSON.parse(readFileSync('.research/pele/package.json'));
const oracle=await loadOracle(),binary='target/release/zebra-mermaid';
const run=(source,args=[])=>{
 const p=spawnSync(binary,['render','-','--format','json',...args],{input:source,encoding:'utf8'});
 assert.equal(p.status,0,p.stderr);return JSON.parse(p.stdout);
};
const cases=[
 ['members','classDiagram\nclass A {\n  +title: string\n}'],
 ['attributes','erDiagram\n  CAR {\n    string registrationNumber PK "The plate number, as issued by the registration office of the country"\n    int wheels FK, UK\n  }'],
 ['files','treeView-beta\n    src/\n        App.tsx ## main\n'],
 ['messages','sequenceDiagram\n  A->>B: a message in the monospace font\n  B->>B: a message to itself\n  Note over A,B: a note'],
];
let compared=0;
for(const [part,source] of cases)for(const maxWidth of [undefined,240])for(const mono of [false,true]){
 const options={...(maxWidth===undefined?{}:{maxWidth}),...(mono?{monospace:[part]}:{})};
 const expected=oracle.render(source,options);
 const actual=run(source,['--color-mode','css',...(mono?['--monospace',part]:[]),...(maxWidth===undefined?[]:['--max-width',String(maxWidth)])]);
 for(const key of ['svg','width','height'])assert.equal(actual[key],expected[key],`${part}, ${maxWidth}, ${mono}, ${key}`);
 compared++;
}
const native=[
 'flowchart LR\nsubgraph S\n A --> B\nend\nB --> C\nstyle S fill:black',
 'treemap\n"big": 1000\n"small": 4',
 'sequenceDiagram\nbox black Group\nactor A\nparticipant B\nend\nA->>B: hi',
];
for(const source of native){const actual=run(source);assert.doesNotMatch(actual.svg,/var\(|color-mix\(|light-dark\(/);}
const files=run(cases[2][1],['--zebra','--monospace','files']);assert.doesNotMatch(files.svg,/var\(/);assert.match(files.svg,/font-family="monospace"/);
const invalid=spawnSync(binary,['render','-','--monospace','member'],{input:cases[0][1],encoding:'utf8'});assert.equal(invalid.status,2);assert.equal(invalid.stdout,'');assert.match(invalid.stderr,/--monospace/);
const cliVersion=execFileSync(binary,['--version'],{encoding:'utf8'}).trim();assert.ok(cliVersion.includes(version));
const result={version,commit,cliVersion,cssComparisons:compared,nativeChecks:native.length+1,invalidOptionChecks:1,success:true};
writeFileSync(`target/alignment-${version}/cli-validation.json`,JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify(result));
