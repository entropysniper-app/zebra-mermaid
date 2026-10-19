// Independent ECMAScript reference; never derives expected output from the Rust formatter.
import {readFileSync,writeFileSync} from 'node:fs';
const values=[0,-0,1,-1,0.5,-0.5,Number.MAX_SAFE_INTEGER,Number.MIN_SAFE_INTEGER,2**53,-(2**53),1e16,1e20,1e21,1e-6,1e-7,NaN,Infinity,-Infinity];
for(let p=0;p<=53;p++)for(const d of [-1,-.5,0,.5,1])values.push(2**p+d,-(2**p+d));
let seed=42n;
for(let i=0;i<1024;i++){
 seed=(6364136223846793005n*seed+1442695040888963407n)&((1n<<64n)-1n);
 const x=Number(seed&((1n<<53n)-1n));values.push(x,-x);
}
const seen=new Set(),rows=[];
for(const x of values){
 const view=new DataView(new ArrayBuffer(8));view.setFloat64(0,x);
 const bits=view.getBigUint64(0).toString(16).padStart(16,'0');
 if(!seen.has(bits)){seen.add(bits);rows.push({bits,expected:String(x)})}
}
const text=rows.map(x=>JSON.stringify(x)).join('\n')+'\n',path='tests/fixtures/unit/js-safe-integers.jsonl';
if(process.argv.includes('--check')){if(readFileSync(path,'utf8')!==text)throw Error('Number reference differs')}
else writeFileSync(path,text);
console.log(`Verified ${rows.length} Node reference values (${process.version}).`);
