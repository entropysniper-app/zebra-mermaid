// Keep generated test artifacts stable under cargo fmt. Formatting cannot replace the
// source/IR checks or the original assertions and input fingerprints.
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
export function formatRust(source) {
  const result=spawnSync('rustfmt',['--edition','2021','--emit','stdout'],{
    cwd:fileURLToPath(new URL('../..',import.meta.url)),input:source,encoding:'utf8',maxBuffer:64*1024*1024,
  });
  if(result.error||result.status!==0)throw Error(`Cannot format generated Rust: ${result.error??result.stderr}`);
  return result.stdout;
}
