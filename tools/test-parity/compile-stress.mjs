#!/usr/bin/env node
import {formatRust} from './format-rust.mjs';
// Translate the input-building AST, not evaluated renderer output. Input construction remains timed.
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import vm from 'node:vm';
const root=fileURLToPath(new URL('../..',import.meta.url));
const ts=createRequire(resolve(root,'.research/pele/website/package.json'))('typescript');
const inventory=JSON.parse(readFileSync(resolve(root,'tests/upstream/inventory.json')));
const ports=JSON.parse(readFileSync(resolve(root,'tests/upstream/ports.json'))),artifacts=[],files=[],profileCases=[];
const sha=s=>createHash('sha256').update(s).digest('hex');
const rustString=s=>{if(s.includes('\r'))return '"'+Array.from(s,c=>c==='\\'?'\\\\':c==='"'?'\\"':c.codePointAt(0)<32?'\\u{'+c.codePointAt(0).toString(16)+'}':c).join('')+'".to_owned()';for(let n=1;;n++){const h='#'.repeat(n);if(!s.includes('"'+h))return `r${h}"${s}"${h}.to_owned()`}};
const fnv=s=>{let h=0xcbf29ce484222325n;for(const b of new TextEncoder().encode(s))h=BigInt.asUintN(64,(h^BigInt(b))*0x100000001b3n);return '0x'+h.toString(16)};
const compact=n=>n.getText().replace(/\s/g,'');
const owned=new Set();
for(const type of ['sequence','class','agentflow','er','state','requirement']){
 const file=`tests/${type}/stress.test.ts`,source=readFileSync(resolve(root,'tests/upstream/source',file),'utf8');
 const expected=inventory.files.find(f=>f.path===file);if(sha(source)!==expected.sha256)throw Error('Frozen source changed '+file);
 const sf=ts.createSourceFile(file,source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS),constants=new Map();
 for(const statement of sf.statements)if(ts.isVariableStatement(statement))for(const d of statement.declarationList.declarations)if(ts.isIdentifier(d.name))constants.set(d.name.text,d.initializer);
 const array=constants.get('CASES');if(!ts.isArrayLiteralExpression(array))throw Error('CASES must be an array');
 const schema=[];const visit=n=>{if(ts.isCallExpression(n)&&ts.isIdentifier(n.expression)&&n.expression.text==='it')schema.push(n);ts.forEachChild(n,visit)};visit(sf);
 if(schema.length!==(type==='state'?1:2))throw Error('Unreviewed stress registration schema');
 const [timed,limit]=schema,body=timed.arguments[1].body.statements;
 if(body.length!==3||compact(body[0])!=='conststarted=performance.now();'||compact(body[1])!=='try{run();}catch(error){expect(error,(errorasError).message).toBeInstanceOf(PeleError);}')throw Error('Original timing/error assertions changed');
 const assertion=body[2].expression;
 if(!ts.isCallExpression(assertion)||compact(assertion.expression)!=='expect(performance.now()-started).toBeLessThan'||!ts.isNumericLiteral(assertion.arguments[0]))throw Error('Original performance assertion changed');
 const budget=Number(assertion.arguments[0].text),timeout=Number(timed.arguments[2].text);
 const original=inventory.tests.filter(t=>t.file===file);if(original.length!==array.elements.length+(limit?1:0))throw Error('Expansion count changed');
 const call=(operation,source,options)=>({operation,source,options});
 const js=ts.transpileModule(sf.statements.filter(n=>!ts.isImportDeclaration(n)&&ts.isVariableStatement(n)).map(n=>n.getText(sf)).join('\n')+'\nCASES;',{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.None}}).outputText;
 const rows=vm.runInNewContext(js,{render:(s,o)=>call('render',s,o),parse:(s,o)=>call('parse',s,o),tokenize:s=>call('tokenize',s),ClassMember:class{constructor(s,k){return call('member',s,k)}},parseGenericTypes:s=>call('generics',s),genericTypes:s=>call('generics',s),metricsMeasurer:'metrics'},{timeout:10000});
 const val=(kind,code)=>({kind,code});
 const stringify=v=>v.kind==='string'?v.code:v.kind==='number'?`crate::util::js::to_string(${v.code})`:(()=>{throw Error('Cannot stringify '+v.kind)})();
 const truthy=v=>v.kind==='boolean'?v.code:v.kind==='number'?`(${v.code} != 0.0)`:v.kind==='string'?`(!${v.code}.is_empty())`:(()=>{throw Error('Cannot test truthiness')})();
 function expr(n,scope=new Map()){
  if(ts.isParenthesizedExpression(n))return expr(n.expression,scope);
  if(ts.isStringLiteral(n)||ts.isNoSubstitutionTemplateLiteral(n))return val('string',rustString(n.text));
  if(ts.isNumericLiteral(n))return val('number',`${Number(n.text)}.0_f64`);
  if(ts.isIdentifier(n)){
   if(scope.has(n.text))return scope.get(n.text);
   if(constants.has(n.text))return expr(constants.get(n.text),scope);
   throw Error('Unsupported identifier '+n.text);
  }
  if(ts.isBinaryExpression(n)){
   const a=expr(n.left,scope),b=expr(n.right,scope),op=n.operatorToken.getText(sf);
   if(op==='+'&&(a.kind==='string'||b.kind==='string'))return val('string',`stress::concat(${stringify(a)}, ${stringify(b)})`);
   if(a.kind!=='number'||b.kind!=='number')throw Error('Unreviewed numeric operands');
   const operator=({'===':'==','!==':'!=','+':'+','-':'-','*':'*','/':'/','%':'%','<':'<','>':'>','<=':'<=','>=':'>='})[op];if(!operator)throw Error('Unsupported operator '+op);
   return val(['===','!==','<','>','<=','>='].includes(op)?'boolean':'number',`(${a.code} ${operator} ${b.code})`);
  }
  if(ts.isConditionalExpression(n)){
   const a=expr(n.whenTrue,scope),b=expr(n.whenFalse,scope);if(a.kind!==b.kind)throw Error('Mixed ternary');
   return val(a.kind,`(if ${truthy(expr(n.condition,scope))} { ${a.code} } else { ${b.code} })`);
  }
  if(ts.isTemplateExpression(n)){
   const pieces=[rustString(n.head.text)];for(const s of n.templateSpans)pieces.push(stringify(expr(s.expression,scope)),rustString(s.literal.text));
   return val('string',pieces.reduce((a,b)=>`stress::concat(${a}, ${b})`));
  }
  if(ts.isCallExpression(n)){
   if(ts.isPropertyAccessExpression(n.expression)&&n.expression.name.text==='join'){
    const from=n.expression.expression;
    if(!ts.isCallExpression(from)||from.expression.getText(sf)!=='Array.from'||from.arguments.length!==2||n.arguments.length!==1||!ts.isStringLiteral(n.arguments[0]))throw Error('Unsupported array join');
    const [options,callback]=from.arguments;
    if(!ts.isObjectLiteralExpression(options)||options.properties.length!==1||!ts.isPropertyAssignment(options.properties[0])||options.properties[0].name.getText(sf)!=='length'||!ts.isArrowFunction(callback)||callback.parameters.length!==2||ts.isBlock(callback.body))throw Error('Unsupported Array.from callback');
    const inner=new Map(scope);inner.set(callback.parameters[1].name.text,val('number','array_index'));
    return val('string',`stress::join_lines(${expr(options.properties[0].initializer,scope).code}, |array_index| { ${stringify(expr(callback.body,inner))} }, &${rustString(n.arguments[0].text)})`);
   }
   if(ts.isIdentifier(n.expression)&&n.expression.text==='diagram'){if(n.arguments.length!==1)throw Error('diagram arity');const fn=constants.get('diagram');if(!ts.isArrowFunction(fn)||fn.parameters.length!==1)throw Error('diagram helper');const inner=new Map(scope);inner.set(fn.parameters[0].name.text,expr(n.arguments[0],scope));return expr(fn.body,inner);}
   if(ts.isIdentifier(n.expression)&&n.expression.text==='repeat'){
    if(compact(constants.get('repeat'))!=="(count:number,line:(i:number)=>string):string=>Array.from({length:count},(_,i)=>line(i)).join('')")throw Error('repeat helper changed');
    const fn=n.arguments[1];if(!ts.isArrowFunction(fn)||fn.parameters.length>1||ts.isBlock(fn.body))throw Error('repeat callback');
    const inner=new Map(scope);if(fn.parameters.length)inner.set(fn.parameters[0].name.text,val('number','_index'));
    return val('string',`stress::repeat_lines(${expr(n.arguments[0],scope).code}, |_index| { ${stringify(expr(fn.body,inner))} })`);
   }
   if(ts.isPropertyAccessExpression(n.expression)&&n.expression.name.text==='repeat'){
    if(n.arguments.length!==1)throw Error('repeat arity');return val('string',`stress::repeat_text(${stringify(expr(n.expression.expression,scope))}, ${expr(n.arguments[0],scope).code})`);
   }
  }
  throw Error('Unsupported input expression '+ts.SyntaxKind[n.kind]+': '+n.getText(sf));
 }
 function operation(fn){
  if(!ts.isArrowFunction(fn)||fn.parameters.length||fn.modifiers?.some(m=>m.kind===ts.SyntaxKind.AsyncKeyword))throw Error('Unreviewed stress callback shape');
  let run=fn.body,prefix='',scope=new Map();
  if(ts.isBlock(run)){
   // Preserve both loop bounds, the j=i+1 start, and every += in the original complete graph.
   const [decl,loop,ret]=run.statements;
   if(run.statements.length!==3||!ts.isVariableStatement(decl)||!ts.isForStatement(loop)||!ts.isReturnStatement(ret))throw Error('Unsupported callback block');
   const d=decl.declarationList.declarations[0];if(d.name.text!=='s')throw Error('Unexpected accumulator');
   prefix=`let mut s = ${stringify(expr(d.initializer,scope))};\n`;
   const emitLoop=(loop,level,scope)=>{
    const d=loop.initializer?.declarations?.[0];if(!d||!ts.isIdentifier(d.name)||!ts.isPostfixUnaryExpression(loop.incrementor)||loop.incrementor.operator!==ts.SyntaxKind.PlusPlusToken||loop.incrementor.operand.text!==d.name.text)throw Error('Unsupported loop');
    const name=d.name.text,inner=new Map(scope);inner.set(name,val('number',name));
    let code=`let mut ${name} = ${expr(d.initializer,scope).code};\nwhile ${truthy(expr(loop.condition,inner))} {\n`;
    if(ts.isForStatement(loop.statement))code+=emitLoop(loop.statement,level+1,inner);
    else{const e=loop.statement.expression;if(!ts.isBinaryExpression(e)||e.operatorToken.kind!==ts.SyntaxKind.PlusEqualsToken||e.left.getText(sf)!=='s')throw Error('Loop body changed');code+=`s.push_str(&${stringify(expr(e.right,inner))});\n`;}
    return code+`${name} += 1.0;\n}\n`;
   };
   prefix+=emitLoop(loop,0,scope);scope.set('s',val('string','s'));run=ret.expression;
  }
  if(!(ts.isCallExpression(run)||ts.isNewExpression(run))||!ts.isIdentifier(run.expression))throw Error('Unsupported operation');
  const names={render:'render',parse:'parse',tokenize:'tokenize',ClassMember:'member',parseGenericTypes:'generics',genericTypes:'generics'};
  const op=names[run.expression.text];if(!op)throw Error('Unknown operation');
  if(['render','parse'].includes(op)&&compact(run.arguments[1])!=='big')throw Error('Noncanonical stress options');
  if(op==='member'&&run.arguments[1].text!=='method')throw Error('Unknown member kind');
  return {op,code:`{ ${prefix}${stringify(expr(run.arguments[0],scope))} }`};
 }
 let rust='#![allow(unused_parens)]\n// Generated from the fixed original stress callbacks and input-building AST.\n// Input construction and all iterations remain inside the original timed block.\nuse crate::upstream_stress as stress;\n// Original Vitest cases are sequential within each source file.\nstatic SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());\n';const records=[];
 const makeOp=op=>op==='render'?`crate::render(source, &options).map(|result| { std::hint::black_box(result); })`:op==='parse'?`crate::parse(source, &options).map(|result| { std::hint::black_box(result); })`:op==='tokenize'?`{ std::hint::black_box(super::lexer::tokenize(${type==='sequence'?'&source.encode_utf16().collect::<Vec<_>>()':'source'})); Ok(()) }`:op==='member'?`{ std::hint::black_box(super::Member::new(source, true)); Ok(()) }`:`{ std::hint::black_box(crate::diagrams::common::generics::parse_generic_types(source)); Ok(()) }`;
 for(let i=0;i<array.elements.length;i++){
  const row=array.elements[i],o=original[i],name=row.elements[0].text,fn=row.elements[1];
  if(name!==o.name||name!==rows[i][0]||o.timeoutMs!==timeout||o.mode!=='run')throw Error('Original stress identity changed');
  const originalCall=rows[i][1]();if(typeof originalCall.source!=='string'||!originalCall.source.isWellFormed())throw Error('Unsupported source encoding');
  const x=operation(fn);if(x.op!==originalCall.operation)throw Error('Operation changed');
  if(['render','parse'].includes(x.op)&&(originalCall.options.limit!==Infinity||JSON.stringify(originalCall.options)!=='{"measurer":"metrics","limit":null}'))throw Error('Stress options changed');
  const hash=fnv(originalCall.source);
  if(x.op==='render')profileCases.push({name:`stress: ${type}/${i} ${name}`,inputs:[originalCall.source],unlimitedInput:true});
  rust+=`\n#[test]\nfn case_${String(i).padStart(3,'0')}() {\n    let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());\n    stress::bounded(${timeout}, || {\n`;
  if(['render','parse'].includes(x.op))rust+=`let options = crate::${x.op==='render'?'RenderOptions':'ParseOptions'} { limit: Some(usize::MAX), ..Default::default() };\n`;
  rust+=`stress::timed(${budget}, ${hash}, || ${x.code}, |source| ${makeOp(x.op)});\n    });\n}\n`;
  records.push({id:o.id,index:i,name:o.fullName,mode:o.mode,operation:x.op,budgetMs:budget,timeoutMs:timeout,sourceBytes:Buffer.byteLength(originalCall.source),sourceUtf16Length:originalCall.source.length,sourceFnv64:hash,inputBuilderSha256:sha(fn.getText(sf))});owned.add(o.id);
 }
 // The final limit test has no inner performance timer. Retain every original assertion.
 if(limit){
 const o=original.at(-1);if(o.name!=='refuses source over the default limit'||limit.arguments[0].text!==o.name)throw Error('Unknown final stress test');
 const captured=[];const expect=actual=>({toThrow(pattern){if(String(pattern)!=='/limit of 50000/')throw Error('Limit pattern changed');const value=actual();captured.push({...value,assertion:'throwsLimit'})},toContain(value){if(value!=='<svg'||!actual?.source)throw Error('SVG assertion changed');captured.push({...actual,assertion:'containsSvg'})}});
 const code=ts.transpileModule(limit.arguments[1].getText(sf),{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText;
 const strConsts=Object.fromEntries([...constants].filter(([k,n])=>ts.isNumericLiteral(n)||ts.isStringLiteral(n)).map(([k,n])=>[k,ts.isNumericLiteral(n)?Number(n.text):n.text]));
 vm.runInNewContext('('+code.trim().replace(/;$/,'')+')()', {...strConsts,diagram:s=>'classDiagram\n'+s,big:{measurer:'metrics',limit:Infinity},render:(source,options)=>({operation:'render',source,options,svg:{source,options}}),expect});
 if(captured.length!==(type==='sequence'?2:1))throw Error('Limit assertion count changed');
 rust+=`\n#[test]\nfn case_${String(o.index).padStart(3,'0')}() { let _guard = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner()); stress::bounded(${o.timeoutMs}, || {\n`;
 const calls=[];const findCalls=n=>{if(ts.isCallExpression(n)&&ts.isIdentifier(n.expression)&&n.expression.text==='render')calls.push(n);ts.forEachChild(n,findCalls)};findCalls(limit.arguments[1]);
 if(calls.length!==captured.length)throw Error('Limit render count changed');
 for(let i=0;i<calls.length;i++){
  const x=captured[i],input=stringify(expr(calls[i].arguments[0]));
  rust+=`{ let source = ${input}; stress::input_hash(&source, ${fnv(x.source)});\n`;
  if(x.assertion==='throwsLimit')rust+='let error = crate::render(&source, &crate::RenderOptions::default()).unwrap_err();\nassert!(error.message.contains("limit of 50000"), "{}", error.message);\n';
  else rust+='let result = crate::render(&source, &crate::RenderOptions { limit: Some(usize::MAX), ..Default::default() }).unwrap();\nassert!(result.svg.contains("<svg"));\n';
  rust+='}\n';
 }
 rust+='}); }\n';owned.add(o.id);records.push({id:o.id,index:o.index,name:o.fullName,mode:o.mode,timeoutMs:o.timeoutMs,assertions:captured.map(x=>({kind:x.assertion,sourceFnv64:fnv(x.source),sourceBytes:Buffer.byteLength(x.source)}))});
 }
 const target=`crates/zebra-mermaid/tests/upstream/${type}_stress_generated.rs`;
 artifacts.push([target,formatRust(rust)]);files.push({source:file,sourceSha256:expected.sha256,records});
}
const manifest='tests/upstream/ir/stress.json';artifacts.push([manifest,JSON.stringify({files},null,2)+'\n']);
const previous=ports.ports.filter(p=>owned.has(p.upstreamId));ports.ports=ports.ports.filter(p=>!owned.has(p.upstreamId));
for(const f of files){const type=f.source.split('/')[1];for(const r of f.records)ports.ports.push({upstreamId:r.id,sourceSha256:f.sourceSha256,target:{kind:'rust',binary:'lib',file:`crates/zebra-mermaid/tests/upstream/${type}_stress_generated.rs`,test:`diagrams::${type}::upstream_stress::case_${String(r.index).padStart(3,'0')}`,features:[type]},assertionReview:'Original stress callback AST translated into native Rust input construction, retaining repeat truncation, every loop/conditional/template expression, operation, metrics measurer, Infinity source limit and default output/edge limits. The original elapsed-time threshold includes input construction and execution; only native Error is accepted, with panics remaining failures. Original Vitest timeout is preserved. Input FNV-64 is independently calculated by executing only the original JavaScript input builder with inert operation stubs and checked outside the measured region. Default-limit error regex and subsequent SVG assertion, if present, remain separate assertions.'});}
const sort=a=>[...a].sort((a,b)=>a.upstreamId.localeCompare(b.upstreamId));
if(process.argv.includes('--check')){for(const [path,text]of artifacts)if(readFileSync(resolve(root,path),'utf8')!==text)throw Error('Generated stress artifact differs: '+path);if(JSON.stringify(sort(previous))!==JSON.stringify(sort(ports.ports.filter(p=>owned.has(p.upstreamId)))))throw Error('Stress mapping differs');}
else{mkdirSync(resolve(root,'tests/upstream/ir'),{recursive:true});for(const[path,text]of artifacts)writeFileSync(resolve(root,path),text);writeFileSync(resolve(root,'tests/upstream/ports.json'),JSON.stringify(ports,null,2)+'\n')}
if(process.argv.includes('--emit-profile-cases')){const directory=resolve(root,'target/profiling/stress');mkdirSync(directory,{recursive:true});writeFileSync(resolve(directory,'cases.json'),JSON.stringify(profileCases,null,2)+'\n')}
console.log(JSON.stringify({files:files.map(f=>({source:f.source,tests:f.records.length})),translated:owned.size}));
