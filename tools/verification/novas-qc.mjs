// Copyright (c) Jack Asher
// SPDX-License-Identifier: MIT
import {readFileSync,writeFileSync,readdirSync,existsSync} from 'node:fs';
import {execFileSync,spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
const main=async()=>{
const root=resolve(import.meta.dirname,'../..');
const [endpoint,reportPath,dataset,...flags]=process.argv.slice(2);
if(!endpoint||!reportPath||dataset?.startsWith('--')||flags.some(flag=>flag!=='--wasm'))throw new Error('Usage: node tools/verification/novas-qc.mjs URL REPORT [DATASET] [--wasm]');
const useWasm=flags.includes('--wasm');
const lock=JSON.parse(readFileSync(resolve(root,'tools/verification/novas-lock.json')));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const sourceFiles=()=>[...readdirSync(resolve(root,'src'),{recursive:true}).filter(name=>name.endsWith('.rs')).map(name=>'src/'+name),
  'Cargo.toml','Cargo.lock','rust-toolchain.toml',...(existsSync(resolve(root,'build.rs'))?['build.rs']:[]),'examples/novas_qc_capture.rs','tools/verification/novas-qc.mjs','tools/verification/novas-lock.json',
  'examples/configurable-chart/build-wasm.ts','tools/ci/versions.ts','examples/configurable-chart/wasm/Cargo.toml','examples/configurable-chart/wasm/Cargo.lock',
  ...readdirSync(resolve(root,'examples/configurable-chart/wasm/src'),{recursive:true}).filter(name=>name.endsWith('.rs')).map(name=>'examples/configurable-chart/wasm/src/'+name)];
const sourceHash=()=>{const h=createHash('sha256');for(const file of sourceFiles().sort())h.update(file+'\0').update(readFileSync(resolve(root,file))).update('\0');return h.digest('hex');};
const snapshotHash=sourceHash();
const assert=(condition,message)=>{if(!condition)throw new Error(message);};
const get=async(path,input)=>{
  const response=await fetch(new URL(path,endpoint),input?{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(input),signal:AbortSignal.timeout(20000)}:{signal:AbortSignal.timeout(20000)});
  assert(response.ok,`Benchmark HTTP ${response.status}`);return response.json();
};
const health=await get('/health');
for(const [key,expected]of Object.entries(lock.provenance))assert(health.provenance?.[key]===expected,`Benchmark ${key} mismatch`);
const identity=JSON.stringify(health.provenance);
const checked=async(path,input)=>{const result=await get(path,input);assert(JSON.stringify(result.provenance)===identity,'Benchmark identity changed');assert(result.jdTT===input.jdTT&&result.zodiac===input.zodiac,'Reference input mismatch');
  if(input.houseSystem){
    assert(result.jdUT1===input.jdUT1&&result.location?.latitude===input.location.latitude&&result.location?.longitude===input.location.longitude&&result.geometry?.requestedSystem===input.houseSystem,'Reference geometry input mismatch');
    assert(Array.isArray(result.geometry.cusps)&&result.geometry.cusps.length===12&&result.geometry.cusps.every(Number.isFinite),'Reference cusp coverage mismatch');
    assert(result.geometry.angles&&Object.keys(result.geometry.angles).sort().join(',')==='ascendant,descendant,imumCoeli,midheaven'&&Object.values(result.geometry.angles).every(Number.isFinite),'Reference angle coverage mismatch');
  }
  return result;};
const cases=[];
const add=(utc,zodiac='lahiri',chart=false,geometry={})=>cases.push({id:`case-${cases.length}`,utc,zodiac,chart,latitude:0,longitude:0,houseSystem:'whole-sign',...geometry});
for(let year=1801;year<2200;year+=2)add(`${year}-06-15T12:00:00Z`);
for(let day=0;day<80;day++)add(new Date(Date.UTC(2026,0,1)+day*8.88*3600000).toISOString());
let seed=0x4e4f5641;
for(let index=0;index<128;index++){
  seed=(Math.imul(seed,1664525)+1013904223)>>>0;
  add(new Date(Date.UTC(1801,0,1)+(seed/2**32)*(Date.UTC(2199,0,1)-Date.UTC(1801,0,1))).toISOString());
}
if(dataset)for(let index=0;index<125;index++)for(const zodiac of ['tropical','lahiri','fagan-bradley'])add(new Date(Date.UTC(1851+index*2,index%12,15,12)).toISOString(),zodiac,true,{latitude:[-65,-45,0,45,65][index%5],longitude:-175+(index%24)*15,houseSystem:['whole-sign','equal','placidus'][index%3]});
const datasetBytes=dataset?readFileSync(resolve(dataset)):null;
const captured=spawnSync('cargo',['run','--quiet','--locked','--target-dir',resolve(root,'target/novas-qc'),'--features','generation','--example','novas_qc_capture','--',...(dataset?[resolve(dataset)]:[])],{cwd:root,input:JSON.stringify(cases),encoding:'utf8',maxBuffer:32*1024*1024});
assert(captured.status===0,captured.stderr||'Native capture failed');
const values=JSON.parse(captured.stdout);
assert(Array.isArray(values)&&values.length===cases.length,'Capture count mismatch');
assert(new Set(values.map(value=>value.id)).size===cases.length,'Duplicate capture');
let wasmCalculate,data,wasmArtifacts=null;
const nativeBinaryHash=hash(readFileSync(resolve(root,'target/novas-qc/debug/examples/novas_qc_capture')));
if(dataset){data=readFileSync(resolve(dataset));assert(hash(data)===hash(datasetBytes),'Dataset changed during capture');}
if(useWasm){
  assert(data,'Wasm QC requires a dataset');
  const localBindgen=resolve(root,'target/qc-tools/bin/wasm-bindgen');
  const build=spawnSync('bun',['run','build:wasm'],{cwd:resolve(root,'examples/configurable-chart'),env:{...process.env,CARGO_TARGET_DIR:resolve(root,'examples/configurable-chart/wasm/target'),WASM_BINDGEN:process.env.WASM_BINDGEN??(existsSync(localBindgen)?localBindgen:'wasm-bindgen')},encoding:'utf8',maxBuffer:8*1024*1024});
  assert(build.status===0,build.stderr||'Wasm build failed');
  wasmArtifacts={wasmHash:hash(readFileSync(resolve(root,'examples/configurable-chart/engine/chart_bg.wasm'))),glueHash:hash(readFileSync(resolve(root,'examples/configurable-chart/engine/chart.js'))),rebuilt:true};
  const module=await import(pathToFileURL(resolve(root,'examples/configurable-chart/engine/chart.js')));
  await module.default({module_or_path:readFileSync(resolve(root,'examples/configurable-chart/engine/chart_bg.wasm'))});wasmCalculate=module.calculate;
}
const rows=[];const errors=[];
const row=(id,path,actual,expected,tolerance,angular=false)=>{
  assert(Number.isFinite(actual)&&Number.isFinite(expected),`Nonfinite/missing ${id} ${path}`);
  const delta=angular?((actual-expected+540)%360+360)%360-180:actual-expected;
  rows.push({id,path,actual,expected,delta,tolerance,pass:Math.abs(delta)<=tolerance});
};
const supported=['SUN','MOON','MERCURY','VENUS','MARS','JUPITER','SATURN','URANUS','NEPTUNE','PLUTO','EARTH'];
for(let index=0;index<cases.length;index++){
  const input=cases[index],value=values[index];
  assert(value.id===input.id&&value.utc===input.utc&&value.zodiac===input.zodiac,'Capture input/order mismatch');
  const reference=await checked('/v1/offsets',{jdTT:value.effectiveJdTT,zodiac:'lahiri'});
  assert(reference.offsets.models.reportedNutation==='IAU-2000B-77-ERFA','Wrong nutation model');
  row(input.id,'reportedLahiri.degrees',value.reportedLahiri,reference.offsets.reported,1e-6/3600);
  row(input.id,'nutation.longitude.arcseconds',value.nutationArcseconds.longitude,reference.offsets.nutation.dpsi2000b*3600,1e-6);
  row(input.id,'nutation.obliquity.arcseconds',value.nutationArcseconds.obliquity,reference.offsets.nutation.deps2000b*3600,1e-6);
  if(!input.chart)continue;
  if(value.chart?.error){errors.push({id:input.id,target:'native',error:value.chart.error});continue;}
  const chart=value.chart?.value;assert(chart,'Missing chart');
  const compareChart=async(chart,target)=>{
    assert(chart.configuration.reference===input.zodiac,'Chart zodiac mismatch');
    assert(new Set(chart.bodies.map(body=>body.name)).size===chart.bodies.length,'Duplicate chart body');
    const physical=await checked('/v1/calculate',{jdTT:value.physicalJdTT,zodiac:input.zodiac,bodies:supported});
    for(const name of supported){
      const body=chart.bodies.find(body=>body.name===name);assert(body,`Missing ${name}`);
      for(const field of ['longitude','declination','speed'])row(input.id,`${target}.endToEnd.${name}.${field}`,body[field],physical.bodies[name][field],1/3600,field==='longitude');
    }
    const geometry=await checked('/v1/calculate',{jdTT:value.effectiveJdTT,jdUT1:value.effectiveJdUT1,zodiac:input.zodiac,bodies:['SUN'],location:{latitude:input.latitude,longitude:input.longitude},houseSystem:input.houseSystem,polarHouses:'porphyry'});
    assert(chart.requested_house_system===input.houseSystem,'Requested house system mismatch');
    if(chart.house_system!==geometry.geometry.actualSystem)errors.push({id:input.id,target,error:'Actual house system mismatch',actual:chart.house_system,expected:geometry.geometry.actualSystem});
    assert(Array.isArray(chart.cusps)&&chart.cusps.length===12,'House cusp coverage mismatch');
    chart.cusps.forEach((cusp,i)=>row(input.id,`${target}.matchedTime.cusps.${i}`,cusp,geometry.geometry.cusps[i],1/3600,true));
    for(const [name,key]of Object.entries({ASCENDANT_SYMBOL:'ascendant',MIDHEAVEN:'midheaven',DESCENDANT:'descendant',IMUM_COELI:'imumCoeli'})){
      const body=chart.bodies.find(body=>body.name===name);assert(body,`Missing angle ${name}`);
      row(input.id,`${target}.matchedTime.${name}.longitude`,body.longitude,geometry.geometry.angles[key],1/3600,true);
    }
    const offset=await checked('/v1/offsets',{jdTT:value.effectiveJdTT,zodiac:input.zodiac});
    row(input.id,`${target}.matchedModel.reportedOffset`,chart.reference_offset_degrees,offset.offsets.reported,input.zodiac==='lahiri'?1e-6/3600:1/3600);
  };
  await compareChart(chart,'native');
  if(wasmCalculate){
    let wasm;
    try{wasm=JSON.parse(wasmCalculate(data,JSON.stringify({instant:{scale:'utc',value:input.utc},latitude:input.latitude,longitude:input.longitude,house_system:input.houseSystem,configuration:{reference:input.zodiac,divisions:'equal',ophiuchus:null}})));}
    catch(error){errors.push({id:input.id,target:'wasm',error:String(error)});continue;}
    await compareChart(wasm,'wasm');
  }
}
assert(sourceHash()===snapshotHash,'Source changed during QC');
const report={execution:{platform:process.platform,arch:process.arch,node:process.version,rust:execFileSync('rustc',['--version'],{cwd:root,encoding:'utf8'}).trim()},sourceHash:snapshotHash,nativeBinaryHash,wasmArtifacts,scope:dataset?'major-planets-houses-and-reported-offsets':'reported-lahiri-and-original-nutation',fullChartCoverage:false,
  omitted:['polar-library-house-QC','nodes','Ceres','Chiron','True Sky'],cases:cases.length,chartCases:cases.filter(c=>c.chart).length,targets:useWasm?['native','wasm']:['native'],
  pass:errors.length===0&&rows.every(row=>row.pass),rows,errors,benchmark:health.provenance,
  sourceRevision:execFileSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'}).trim(),
  workingTreeDirty:execFileSync('git',['status','--porcelain'],{cwd:root,encoding:'utf8'}).trim().length>0,
  datasetHash:data?hash(data):null,heldOutSeed:'0x4e4f5641',
  timePolicy:'reported offsets matched to captured library effective TT; end-to-end planetary outputs compared at physical TT corresponding to UTC; house geometry compared at captured effective TT and UT1'};
writeFileSync(reportPath,JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({pass:report.pass,cases:report.cases,rows:rows.length,failures:rows.filter(row=>!row.pass).length,errors:errors.length}));
process.exitCode=report.pass?0:1;

};
try{await main();}catch(error){
  const failure={status:"setup-error",pass:false,error:error.message};
  console.error(JSON.stringify(failure));
  const reportPath=process.argv[3];
  if(reportPath&&!reportPath.startsWith("--")){try{writeFileSync(reportPath,JSON.stringify(failure,null,2)+"\n");}catch{/* Keep the original setup failure on stderr. */}}
  process.exitCode=2;
}
