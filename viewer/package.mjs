// Reader for upstream PocketJS v2 containers, restricted to the UI host profile.
const PROFILE='94c1629af49148b04cbfc404abfd1208cd97ec2592ac9cff72805495fcb41975';
const PLAN='{"host":"pj4edk2-ui","version":1,"font_slots":8}';
const decoder=new TextDecoder('utf-8',{fatal:true});
const align=n=>Math.ceil(n/16)*16;
function fnv(chunks){let h=0xcbf29ce484222325n;for(const bytes of chunks)for(const b of bytes)h=BigInt.asUintN(64,(h^BigInt(b))*0x100000001b3n);return h;}
export function readPackage(buffer){
 const b=new Uint8Array(buffer);if(b.length<104||b.length>16*1024*1024)throw Error('UI package size');
 const v=new DataView(b.buffer,b.byteOffset,b.byteLength),u=n=>v.getUint32(n,true),end=b.length-8;
 const range=(o,n)=>{if(!Number.isSafeInteger(o)||!Number.isSafeInteger(n)||o<0||n<0||o>end||n>end-o)throw Error('Package bounds');return b.subarray(o,o+n);};
 if(u(0)!==1414218576||u(4)!==2||u(8)!==0||u(12)!==1)throw Error('Expected single-variant .pocket v2 UI package');
 if(u(24)!==0||u(28)!==0||u(16)!==96||u(20)!==10)throw Error('Package header layout');
 if(fnv([b.subarray(0,end)])!==v.getBigUint64(end,true))throw Error('Package footer hash');
 const table=32;if(u(table)!==96||u(table+4)!==10||decoder.decode(range(96,10))!=='pj4edk2-ui'||u(table+8)!==1||u(table+20)!==0)throw Error('Package target/ABI');
 const profile=Array.from(range(table+32,32),x=>x.toString(16).padStart(2,'0')).join('');if(profile!==PROFILE)throw Error('UI profile mismatch');
 const count=u(table+12),directory=u(table+16);if(count!==4||directory!==align(106))throw Error('UI section directory');
 range(directory,count*16);let cursor=align(directory+count*16);const sections=[];
 for(let i=0;i<count;i++){const at=directory+i*16;if(u(at)!==i+1||u(at+4)!==0||u(at+8)!==cursor)throw Error('UI section layout');const data=range(cursor,u(at+12));sections.push(data);cursor=align(cursor+data.length);}
 if(cursor!==end||fnv(sections)!==v.getBigUint64(table+24,true))throw Error('Variant integrity');
 let at=0;const identity=sections[0];for(let i=0;i<3;i++){if(at+2>identity.length)throw Error('Identity');const n=identity[at]|identity[at+1]<<8;at+=2;if(at+n>identity.length)throw Error('Identity');decoder.decode(identity.subarray(at,at+n));at+=n;}
 if(decoder.decode(sections[1])!==PLAN)throw Error('UI plan mismatch');
 const js=sections[2];if(!js.length||js[js.length-1]!==0||js.subarray(0,-1).includes(0))throw Error('JavaScript terminator');
 const pak=sections[3],pv=new DataView(pak.buffer,pak.byteOffset,pak.byteLength),p=n=>pv.getUint32(n,true);
 if(pak.length<32||p(0)!==0x4b504344||pv.getUint16(4,true)!==1)throw Error('Asset pack');
 const assets=new Map(),items=p(8),dir=p(12),names=p(16);if(dir>pak.length||names>pak.length||items>Math.floor((pak.length-dir)/24))throw Error('Asset directory');
 function assetRange(o,n){if(o>pak.length||n>pak.length-o)throw Error('Asset bounds');return pak.subarray(o,o+n);}
 for(let i=0;i<items;i++){const a=dir+i*24,name=assetRange(names+p(a+12),pv.getUint16(a+16,true));if(!name.length||name.some(c=>c===0||c>127))throw Error('Asset name');const key=decoder.decode(name);if(assets.has(key))throw Error('Duplicate asset');assets.set(key,assetRange(p(a+4),p(a+8)));}
 for(let i=0;i<8;i++){const a=assets.get(`fonts/${i}.atlas`);if(!a||a.length<24||decoder.decode(a.subarray(0,4))!=='DCFA'||a[4]!==3||a[5]!==0||a[12]!==i)throw Error('Font atlas');}
 if(!assets.has('font.ttf'))throw Error('Browser font missing');
 return {javascript:decoder.decode(js.subarray(0,-1)),assets};
}
