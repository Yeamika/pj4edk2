import {createWasmUiSync} from '../runtime/platforms/web/wasm-ops.js';
const staticOperations=new Set(['createNode','destroyNode','insertBefore','setProp','setText','setImage','loadFontAtlas','uploadTexture','freeTexture']);

// The upstream embedding owns nodes, font metrics, layout and rasterization.
// The browser only presents its RGBA framebuffer on a canvas.
export class PocketRenderer {
    backend='pocketjs-wasm-software';
    label='PocketJS WASM · pixel';
    constructor(module, surface) { this.module=module; this.surface=surface; }

    reset(width, height, assets) {
        this.host?.dispose();
        this.host=createWasmUiSync(this.module,{width,height});
        this.revision=0; this.paintedRevision=-1; this.lastHash=null;this.dynamicOperations=false;
        // Resource uploads can change pixels without changing draw commands.
        // Count operations as well as hashing the upstream retained draw list.
        this.ui=Object.fromEntries(Object.entries(this.host.ops).map(([name,value])=>[
            name,typeof value!=='function'||name==='measureText' ? value : (...args)=>{
                const result=value(...args);this.revision++;
                if(!staticOperations.has(name))this.dynamicOperations=true;
                return result;
            }
        ]));
        this.assets=assets;
        for(let slot=0;slot<8;slot++) {
            if(!this.ui.loadFontAtlas(assets.get(`fonts/${slot}.atlas`))) throw Error(`Invalid font atlas ${slot}`);
        }
        this.canvas=document.createElement('canvas');
        this.canvas.width=width; this.canvas.height=height;
        this.canvas.style.cssText='display:block;width:100%;height:100%;image-rendering:pixelated';
        this.context=this.canvas.getContext('2d',{alpha:false});
        if(!this.context) throw Error('Canvas presentation unavailable');
        this.imageData=null;
        this.surface.replaceChildren(this.canvas);
    }

    paint() {
        this.host.tick();
        if(!this.dynamicOperations&&this.revision===this.paintedRevision)return;
        const hash=this.host.drawHash();
        if(hash===this.lastHash&&this.revision===this.paintedRevision)return;
        const pixels=this.host.renderIncremental();
        if(!this.imageData||this.imageData.data.buffer!==pixels.buffer||this.imageData.data.byteOffset!==pixels.byteOffset){
            const view=new Uint8ClampedArray(pixels.buffer,pixels.byteOffset,pixels.byteLength);
            this.imageData=new ImageData(view,this.canvas.width,this.canvas.height);
        }
        this.context.putImageData(this.imageData,0,0);
        this.lastHash=hash;this.paintedRevision=this.revision;
    }

    memoryBytes(){return this.host.exports.memory.buffer.byteLength;}
    drawHash(){return this.host.drawHash().toString(16);}
    dispose(){this.host?.dispose();this.host=undefined;}

    // Match the UEFI name-card bake through the same upstream UI and rasterizer.
    bakeName(card) {
        const [width,height]=card.size;
        if(!Number.isFinite(width)||!Number.isFinite(height)||width<1||height<1||width>1024||height>512) throw Error('Invalid name-card size');
        const w=2**Math.ceil(Math.log2(width)),h=2**Math.ceil(Math.log2(height));
        const bake=createWasmUiSync(this.module,{width:w,height:h});
        try {
            const ops=bake.ops;
            if(!ops.loadFontAtlas(this.assets.get(`fonts/${card.font_slot}.atlas`))) throw Error('Name-card font unavailable');
            const box=ops.createNode(0);
            for(const [key,value] of [[1,width],[2,height],[64,card.fill],[70,card.border],[71,1],[68,card.radius],[30,1]]) ops.setProp(box,key,value);
            ops.insertBefore(1,box,0);
            const text=ops.createNode(1);
            for(const [key,value] of [[24,1],[28,card.text_origin[0]],[25,card.text_origin[1]],[1,Math.max(0,width-2*card.text_origin[0])],[96,card.ink],[97,card.font_slot]]) ops.setProp(text,key,value);
            ops.setText(text,card.text); ops.insertBefore(box,text,0);
            bake.tick();
            const pixels=bake.renderPremultiplied().slice();
            for(let i=0;i<pixels.length;i+=4) {
                const alpha=pixels[i+3];
                for(let channel=0;channel<3;channel++) pixels[i+channel]=alpha===0 ? (card.fill>>>(8*channel))&255 : Math.min(255,Math.floor((pixels[i+channel]*255+Math.floor(alpha/2))/alpha));
            }
            const handle=this.ui.uploadTexture(pixels,w,h,3);
            if(handle<0) throw Error('Name-card texture allocation failed');
            return {handle,width:w,height:h};
        } finally { bake.dispose(); }
    }
}
