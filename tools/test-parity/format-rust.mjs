// Keep generated test artifacts stable under cargo fmt. Formatting cannot replace the
// source/IR checks or the original assertions and input fingerprints.
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {mkdirSync,mkdtempSync,writeFileSync,readFileSync,unlinkSync,rmdirSync} from 'node:fs';
import {join} from 'node:path';
export function formatRust(source) {
  const root=fileURLToPath(new URL('../..',import.meta.url)),scratch=join(root,'target/test-parity');
  mkdirSync(scratch,{recursive:true});
  const dir=mkdtempSync(join(scratch,'format-')),path=join(dir,'generated.rs');
  try {
    writeFileSync(path,source);
    // A closed input file avoids a large-stdin EOF hang in the child-process pipe.
    const result=spawnSync('rustfmt',['--edition','2021','--config-path',join(root,'rustfmt.toml'),'--emit','files',path],{
      cwd:root,encoding:'utf8',maxBuffer:64*1024*1024,
    });
    if(result.error||result.status!==0)throw Error(`Cannot format generated Rust: ${result.error??result.stderr}`);
    return readFileSync(path,'utf8');
  } finally {unlinkSync(path);rmdirSync(dir);}
}
