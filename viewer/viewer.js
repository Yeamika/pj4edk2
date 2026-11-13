import {readPackage} from './package.mjs';
import {PreviewClock} from './clock.mjs';
import {PocketRenderer} from './renderer.mjs';
import {DomRenderer} from './dom-renderer.mjs';
const surface=document.querySelector('#surface'),metrics=document.querySelector('#metrics');
const wasmResponse=await fetch('runtime/pocketjs.wasm',{cache:'no-cache'});
if(!wasmResponse.ok) throw Error('PocketJS WASM unavailable: build the viewer first');
const renderers={wasm:new PocketRenderer(await WebAssembly.compile(await wasmResponse.arrayBuffer()),surface),direct:new DomRenderer(surface)};
let renderer=renderers.wasm,backendSwitch=0;
const events=[],actions=[];
let packageUi;
globalThis.bootEmit=json=>actions.push(JSON.parse(json));
globalThis.bootBakeName=json=>JSON.stringify(renderer.bakeName(JSON.parse(json)));
async function admitUi(bytes){
 const parsed=readPackage(bytes);
 if(renderer===renderers.direct)await renderers.direct.prepare(parsed.assets);
 packageUi=parsed;
 globalThis.viewerPackage={format:'.pocket',version:2,bytes:bytes.byteLength};
}
const response=await fetch('ui.pocket',{cache:'no-cache'});
if(!response.ok)throw Error('UI package unavailable: build ui.pocket first');
await admitUi(await response.arrayBuffer());
const entries=['LinuxA','LinuxB','Android','Windows','Darwin','Baremetal'].map((name,i)=>({id:name.toLowerCase(),name,path:'\\EFI\\'+name+'\\BOOTAA64.EFI',icon:name==='Windows'?'windows':'system',parameters:{}}));
const config={mode:'uefi',version:'0.1.0',viewport:[innerWidth,innerHeight],display_id:'main',timeout_seconds:0,default_entry_id:'linuxa',entries,battery_percent:76,external_power:true,power:{}};
let viewScale=1, powered=true, frameHandle=null, pointer=null;
const clock=new PreviewClock();
const samples=[],renderSamples=[],uiSamples=[],intervals=[];let updated=0;
const hostLog=[];
function fit(){viewScale=Math.min(1,document.querySelector('#preview').clientWidth/config.viewport[0]);surface.style.width=config.viewport[0]+'px';surface.style.height=config.viewport[1]+'px';surface.style.transform=`scale(${viewScale})`;document.querySelector('#preview').style.height=config.viewport[1]*viewScale+'px';}
function init(){if(frameHandle!==null)cancelAnimationFrame(frameHandle);frameHandle=null;powered=true;updateDeviceKeys();clock.reset();samples.length=0;renderSamples.length=0;uiSamples.length=0;intervals.length=0;updated=0;pointer=null;globalThis.viewerStats=null;metrics.textContent='UI preview · restarting…';surface.style.background='';events.length=0;actions.length=0;renderer.reset(...config.viewport,packageUi.assets);globalThis.ui=renderer.ui;Function(packageUi.javascript)();rimuksInit(JSON.stringify(config));fit();events.push({type:'charge_state',available:true,enabled:config.power.charging===true,requested:config.power.charging===true});schedule();}
function updateDeviceKeys(){for(const id of ['volume-up','volume-down'])document.getElementById(id).disabled=!powered;const power=document.getElementById('power-key');power.title=powered?'电源键：确认（Enter）':'电源键：开机';power.setAttribute('aria-label',powered?'电源键：确认':'电源键：开机');}
function sendKey(key){if(powered)events.push({type:'key',key});}
config.viewport=[1920,1200]; config.power={charging:true,voltage_mv:4100,current_ma:800};
init();
globalThis.viewerEngine={get backend(){return renderer.backend;},drawHash:()=>renderer.drawHash()};
addEventListener('resize',fit);
function point(e){const r=surface.getBoundingClientRect();return {x:(e.clientX-r.left)/viewScale,y:(e.clientY-r.top)/viewScale};}
addEventListener('keydown',e=>{if(e.target.closest('#controls, #toolbar'))return;if(['F2','Tab','Enter','ArrowUp','ArrowDown','ArrowLeft','ArrowRight','Backspace','Delete','Escape'].includes(e.key)||/^\d$/.test(e.key)){e.preventDefault();if(e.key==='Escape'){Rimuks.settingsOpen=false;Rimuks.keypadOpen=false;}else sendKey(e.key);}});
// Native touch events are handled explicitly (including Safari). Ignore the
// corresponding touch PointerEvents so a gesture is never delivered twice.
function beginPointer(e,kind,id){if(!powered||pointer)return;const p=point(e);pointer={...p,kind,id,moved:false};events.push({type:'pointer',...p,down:true});}
function movePointer(e){const p=point(e);if(pointer&&Math.hypot(p.x-pointer.x,p.y-pointer.y)>12*Rimuks.scale)pointer.moved=true;events.push({type:'pointer',...p,down:!!pointer});}
function endPointer(e,cancelled=false){if(!pointer)return;const p=point(e);events.push({type:'pointer',...p,down:false});if(!cancelled&&!pointer.moved)events.push({type:'click',...p});pointer=null;}
surface.addEventListener('pointerdown',e=>{if(e.pointerType==='touch')return;beginPointer(e,'pointer',e.pointerId);surface.setPointerCapture(e.pointerId);});
surface.addEventListener('pointermove',e=>{if(e.pointerType==='touch')return;movePointer(e);});
surface.addEventListener('pointerup',e=>{if(e.pointerType==='touch')return;endPointer(e);});
surface.addEventListener('pointercancel',e=>{if(e.pointerType==='touch')return;endPointer(e,true);});
surface.addEventListener('lostpointercapture',e=>{if(pointer?.kind==='pointer')endPointer(e,true);});
surface.addEventListener('touchstart',e=>{e.preventDefault();if(!pointer&&e.changedTouches.length){const t=e.changedTouches[0];beginPointer(t,'touch',t.identifier);}},{passive:false});
surface.addEventListener('touchmove',e=>{e.preventDefault();if(pointer?.kind==='touch'){const t=Array.from(e.touches).find(t=>t.identifier===pointer.id);if(t)movePointer(t);}},{passive:false});
for(const name of ['touchend','touchcancel'])surface.addEventListener(name,e=>{e.preventDefault();if(pointer?.kind==='touch'){const t=Array.from(e.changedTouches).find(t=>t.identifier===pointer.id);if(t)endPointer(t,name==='touchcancel');}},{passive:false});
surface.addEventListener('wheel',e=>{e.preventDefault();events.push({type:'scroll',delta:-e.deltaY});},{passive:false});
function handle(action){
 hostLog.push(`${new Date().toISOString()} ${JSON.stringify(action)}`);if(hostLog.length>200)hostLog.shift();
 if(action.type==='shutdown'){powered=false;updateDeviceKeys();if(frameHandle!==null)cancelAnimationFrame(frameHandle);frameHandle=null;pointer=null;events.length=0;surface.replaceChildren();surface.style.background='#081018';const off=document.createElement('div');Object.assign(off.style,{color:'#fff',position:'absolute',inset:'0',display:'grid',placeContent:'center',textAlign:'center',font:'28px Inter'});off.textContent='模拟设备已关机';surface.append(off);metrics.textContent='模拟设备已关机 · 点击顶部电源键开机';return;}
 if(action.type==='reboot'||action.type==='restart'){init();return;}

 if(action.type==='set_timeout'){config.timeout_seconds=action.seconds;saveDisk();events.push({type:'settings_saved',seconds:action.seconds});}
 else if(action.type==='set_default_entry'){config.default_entry_id=action.id;saveDisk();events.push({type:'default_entry_read',requested_id:action.id,id:action.id});}
 else if(action.type==='set_debug_enabled')events.push({type:'debug_configured',requested_enabled:action.enabled,enabled:action.enabled,success:true});
 else if(action.type==='set_charging_enabled'){config.power.charging=action.enabled&&config.external_power;config.power.current_ma=config.power.charging?Math.abs(config.power.current_ma||800):-Math.abs(config.power.current_ma||300);events.push({type:'charge_configured',requested_enabled:action.enabled,enabled:config.power.charging,success:true},{type:'power_state',battery_percent:config.battery_percent,external_power:config.external_power,power:config.power});}
 else if(action.type==='recovery'){if(action.operation==='reload-list'){try{loadDisk();}catch(e){$('error').textContent=e.message;events.push({type:'recovery_result',operation:action.operation,success:false});return;}}if(action.operation==='repair'){config.timeout_seconds=0;config.default_entry_id=config.entries[0]?.id||null;saveDisk();}events.push({type:'recovery_result',operation:action.operation,success:true,lines:action.operation==='diagnostics'?['PJ4EDK2 browser UI host','Display '+config.viewport.join('x'),'Virtual ESP files: '+disk.size,'No physical firmware or OS calls']:action.operation==='boot-log'?hostLog.slice(-100):['Simulated operation complete'],settings:action.operation==='repair'?{timeout_seconds:config.timeout_seconds,default_entry_id:config.default_entry_id}:undefined});}
 else if(action.type==='boot'){const box=document.createElement('div');Object.assign(box.style,{position:'absolute',left:action.display_regions[0].position[0]+'px',top:action.display_regions[0].position[1]+'px',padding:'16px',color:'white',font:'16px monospace',zIndex:500});box.textContent=action.id+' — UI preview only. Reload to return.';surface.append(box);}
}
function schedule(){if(powered&&frameHandle===null)frameHandle=requestAnimationFrame(frame);}
function frame(now){
 frameHandle=null;if(!powered)return;
 const elapsed=clock.tick(now),begin=performance.now();
 rimuksTick(elapsed,JSON.stringify(events.splice(0)));
 const uiMs=performance.now()-begin;
 for(const a of actions.splice(0))handle(a);
 if(!powered)return;
 const renderBegin=performance.now();renderer.paint();
 uiSamples.push(uiMs);renderSamples.push(performance.now()-renderBegin);samples.push(performance.now()-begin);
 if(clock.interval!==null)intervals.push(clock.interval);
 for(const values of [samples,uiSamples,renderSamples,intervals])if(values.length>600)values.shift();
 if(elapsed-updated>=500){
  const sorted=[...samples].sort((a,b)=>a-b),avg=a=>a.length?a.reduce((x,y)=>x+y,0)/a.length:0;
  globalThis.viewerStats={backend:renderer.backend,samples:samples.length,jsMeanMs:avg(uiSamples),renderMeanMs:avg(renderSamples),cpuMeanMs:avg(samples),cpuP95Ms:sorted[Math.floor(sorted.length*.95)],rafMeanMs:avg(intervals),rafMaxMs:Math.max(0,...intervals),width:config.viewport[0],height:config.viewport[1],elapsedMs:elapsed,memoryBytes:renderer.memoryBytes()};
  metrics.textContent=`${renderer.label} · ${config.viewport[0]}×${config.viewport[1]} · ${(elapsed/1000).toFixed(1)} s · UI ${viewerStats.jsMeanMs.toFixed(2)} ms · render ${viewerStats.renderMeanMs.toFixed(2)} ms · frame ${viewerStats.rafMeanMs.toFixed(1)} ms`;
  updated=elapsed;
 }
 schedule();
}
schedule();

const $=id=>document.getElementById(id), disk=new Map(), configPath='EFI/BOOT/pj4edk/config.json';
function showDisk(){$('tree').textContent=[...disk].sort().map(([p,b])=>p+'  ('+b.length+' bytes)').join('\n');}
function saveDisk(){disk.set(configPath,new TextEncoder().encode(JSON.stringify(config,null,2)));showDisk();}
function readEntries(value){if(!Array.isArray(value)||value.length>100||value.some(e=>!e||typeof e.id!=='string'||typeof e.name!=='string'||typeof e.path!=='string'))throw Error('启动项需要 id、name、path，最多 100 项');return value;}
function loadDisk(){const key=[...disk.keys()].find(p=>p.toLowerCase()===configPath.toLowerCase());if(!key)throw Error('虚拟盘中没有 '+configPath);const c=JSON.parse(new TextDecoder().decode(disk.get(key)));config.entries=readEntries(c.entries);if(Number.isInteger(c.timeout_seconds)&&c.timeout_seconds>=0&&c.timeout_seconds<=99)config.timeout_seconds=c.timeout_seconds;if(typeof c.default_entry_id==='string')config.default_entry_id=c.default_entry_id;$('entries').value=JSON.stringify(config.entries,null,2);init();}
function guarded(fn){return async()=>{try{$('error').textContent='';await fn();}catch(e){$('error').textContent=e.message;}};}
$('entries').value=JSON.stringify(entries,null,2);
$('apply').onclick=guarded(()=>{const w=Number($('width').value),h=Number($('height').value),b=Number($('battery').value);if(!Number.isInteger(w)||!Number.isInteger(h)||w<320||h<320||w>3840||h>3840||b<0||b>100)throw Error('请检查分辨率与电量');config.entries=readEntries(JSON.parse($('entries').value));config.viewport=[w,h];config.mode=$('mode').value;config.battery_percent=b;config.external_power=$('external').checked;const state=$('charging').value;config.power={charging:state==='unknown'?null:state==='charge',voltage_mv:Number($('voltage').value)||null,current_ma:state==='unknown'?null:state==='discharge'?-Math.abs(Number($('current').value)):state==='idle'?0:Math.abs(Number($('current').value))};init();});
$('folder').onchange=guarded(async()=>{const incoming=new Map();for(const f of $('folder').files){if(f.size>16*1024*1024)throw Error('单文件上限 16 MiB');const parts=f.webkitRelativePath.split('/');parts.shift();incoming.set(parts.join('/'),new Uint8Array(await f.arrayBuffer()));}if([...incoming.values()].reduce((n,b)=>n+b.length,0)>64*1024*1024)throw Error('虚拟盘上限 64 MiB');disk.clear();for(const [p,b]of incoming)disk.set(p,b);showDisk();});
$('load-disk').onclick=guarded(loadDisk);$('save-disk').onclick=guarded(saveDisk);
$('scan-disk').onclick=guarded(()=>{config.entries=[...disk.keys()].filter(p=>/\.efi$/i.test(p)).map((p,i)=>({id:'efi-'+i,name:p.split('/').slice(-2,-1)[0]||p,path:'\\'+p.replaceAll('/','\\'),icon:'system',parameters:{}}));$('entries').value=JSON.stringify(config.entries,null,2);init();});
$('export-disk').onclick=guarded(()=>{const data={schema:1,files:[...disk].map(([path,b])=>{let text='';for(let i=0;i<b.length;i+=8192)text+=String.fromCharCode(...b.subarray(i,i+8192));return {path,base64:btoa(text)};})};const a=document.createElement('a'),url=URL.createObjectURL(new Blob([JSON.stringify(data)],{type:'application/json'}));a.href=url;a.download='pj4edk2-virtual-esp.json';a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);});
$('import-disk').onchange=guarded(async()=>{const f=$('import-disk').files[0];if(!f)return;if(f.size>90*1024*1024)throw Error('镜像过大');const data=JSON.parse(await f.text());if(data.schema!==1||!Array.isArray(data.files))throw Error('无效镜像');const incoming=new Map();let total=0;for(const f of data.files){if(typeof f.path!=='string'||f.path.startsWith('/')||f.path.includes('\\')||f.path.split('/').some(p=>!p||p==='.'||p==='..'))throw Error('无效路径');const raw=atob(f.base64);total+=raw.length;if(total>64*1024*1024)throw Error('虚拟盘上限 64 MiB');incoming.set(f.path,Uint8Array.from(raw,c=>c.charCodeAt(0)));}disk.clear();for(const [p,b]of incoming)disk.set(p,b);showDisk();});
for(const entry of entries)disk.set(entry.path.slice(1).replaceAll('\\','/'),new TextEncoder().encode('UI preview placeholder; not an executable.'));
saveDisk();

$('power-on').onclick=()=>{config.mode=$('mode').value;init();};

$('power-off').onclick=()=>handle({type:'shutdown'});
$('volume-down').onclick=()=>sendKey('ArrowDown');
$('volume-up').onclick=()=>sendKey('ArrowUp');
$('power-key').onclick=()=>{if(powered)sendKey('Enter');else init();};

function toggleSettings(open){$('controls').hidden=!open;$('settings-shade').hidden=!open;$('toggle-settings').setAttribute('aria-expanded',String(open));}
$('toggle-settings').onclick=()=>toggleSettings($('controls').hidden);
$('settings-shade').onclick=()=>toggleSettings(false);
$('apply-settings').onclick=()=>$('apply').click();
addEventListener('keydown',e=>{if(e.key==='Escape'&&!$('controls').hidden){e.preventDefault();toggleSettings(false);$('toggle-settings').focus();}});

$('ui-package').onchange=guarded(async()=>{const f=$('ui-package').files[0];if(!f)return;if(f.size>16*1024*1024)throw Error('UI 包上限 16 MiB');await admitUi(await f.arrayBuffer());init();});

$('backend').onchange=guarded(async()=>{
 const selected=$('backend').value,revision=++backendSwitch;
 const next=renderers[selected];if(!next)throw Error('Unknown preview backend');
 if(selected==='direct')await next.prepare(packageUi.assets);
 if(revision!==backendSwitch)return;
 renderer.dispose();renderer=next;init();
});
