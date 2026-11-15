// Run with node tools/tests/test-boot-curve.mjs. No firmware or renderer needed.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
const source=fs.readFileSync(new URL('../../default-ui/pages/boot-geometry.js',import.meta.url),'utf8');
for(const scale of [.5,1,1.5,2,3]){
 let next=0,writes=0;const nodes=new Map();
 const R={C:{background:0xc3dff0,line:0x729fbb},L:{top:84,footer:64},h:800,cardX:384,scale,cfg:{viewport:[1280*scale,800*scale]},visibleRows:4,rows:[{},{},{},{}],clamp:x=>Math.max(0,Math.min(1,x)),lerp:(a,b,t)=>a+(b-a)*t,smooth:x=>{x=R.clamp(x);return x*x*(3-2*x);},mix:()=>0,color:x=>x,
 box:(x,y,w,h)=>{const id=++next;nodes.set(id,{left:x,top:y,w,h});return id;},
 prop:(id,k,v)=>{writes++;nodes.get(id)[k]=v;},show:(id,on)=>R.prop(id,'display',on?0:1)};
 vm.runInNewContext(source,{Rimuks:R});R.makeBootCurve();assert.equal(nodes.size,10);
 for(const dock of [0,.1,.5,.9,1]){
  R.curve={dock,startY:R.lerp(R.lineY,R.L.top+8,dock),endY:R.lineEndY};R.updateBootCurve();
  const cached=writes;R.updateBootCurve();assert.equal(writes,cached,'unchanged curve must not cross the property bridge');
  const first=Math.round(R.curve.startY*scale)/scale,last=Math.round(R.curve.endY*scale)/scale;
  for(let i=0;i<10;i++){
   const n=nodes.get(R.lines[i]);assert.equal(n.display,0);assert.ok(Object.values(n).every(Number.isFinite));
   const a=n.rotate*Math.PI/180;
   const x0=n.left+n.w/2*Math.cos(a),y0=n.top+n.w/2*Math.sin(a);
   const expectedY=first+(last-first)*i/10;assert.ok(Math.abs(y0-expectedY)<1e-9);assert.ok(Math.abs(x0-R.arcXAtY(expectedY))<1e-9);
   const y1=first+(last-first)*(i+1)/10,x1=R.arcXAtY(y1);
   const length=Math.hypot(x1-x0,y1-y0);assert.ok(Math.abs(n.h-length-(i<9?1/scale:0))<1e-9);
   assert.ok(Math.abs(x0-Math.sin(a)*length-x1)<1e-9,'rotated segment must reach the next chord');
   const mid=(y0+y1)/2;assert.ok(Math.abs((x0+x1)/2-R.arcXAtY(mid))<=.321,'shallow-curve chord error must remain below .321 logical px');
  }
 }
 R.rows=[{}];R.updateBootCurve();for(const n of nodes.values())assert.equal(n.display,1,'one entry hides the curve');
 R.rows=[];R.updateBootCurve();for(const n of nodes.values())assert.equal(n.display,1,'zero entries hides the curve');
}
console.log('PASS: exactly ten nodes, chord geometry, rotation endpoints, cached updates, five scales, empty/single-entry visibility');
