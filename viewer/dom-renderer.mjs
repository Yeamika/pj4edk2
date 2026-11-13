// Lightweight direct browser preview of the shared scripts, without PocketJS.
export class DomRenderer {
    backend='browser-dom';
    label='直接运行 DOM';
    constructor(surface) {
        this.surface=surface;
        const nodes=this.nodes=new Map(),textures=this.textures=new Map();
        const fonts=[14,18,24,32,54,12,20,18];
        const context=document.createElement('canvas').getContext('2d');
        const color=value=>{const n=Number(value)>>>0;return `rgba(${n&255},${n>>>8&255},${n>>>16&255},${(n>>>24)/255})`;};
        const transform=el=>{const p=el.props;el.style.transform=`scale(${p[130]??1}) rotate(${p[131]??0}deg)`;el.style.transformOrigin=`${((p[134]??0)+.5)*100}% ${((p[135]??0)+.5)*100}%`;};
        this.ui={
            createNode:kind=>{
                const el=document.createElement('div');el.props={};el.kind=kind;
                el.style.cssText='position:absolute;white-space:pre;pointer-events:none;border-style:solid;border-width:0;line-height:1.25';
                const id=this.next++;nodes.set(id,el);return id;
            },
            insertBefore(parent,id,before){nodes.get(parent).insertBefore(nodes.get(id),nodes.get(before)||null);},
            destroyNode(id){const el=nodes.get(id);if(!el)return;for(const [key,node]of nodes)if(node===el||el.contains(node))nodes.delete(key);el.remove();},
            setProp(id,key,value){
                const el=nodes.get(id);if(!el)return;if(el.props[key]===value)return;el.props[key]=value;const s=el.style;
                const px={1:'width',2:'height',25:'top',28:'left',68:'borderRadius',71:'borderWidth',100:'letterSpacing'};
                if(px[key])s[px[key]]=value+'px';
                else if(key===29)s.display=value===1?'none':'block';
                else if(key===30)s.overflow=value===1?'hidden':'visible';
                else if(key===31)s.zIndex=value;
                else if(key===64)s.backgroundColor=color(value);
                else if(key===69)s.opacity=value;
                else if(key===70)s.borderColor=color(value);
                else if(key===96)s.color=color(value);
                else if(key===97){s.fontSize=(fonts[value]||18)+'px';s.fontFamily='PJ4EDK2UI, monospace';s.fontVariantLigatures='none';}
                else if(key===98)s.textAlign=['left','center','right'][value]||'left';
                else if([130,131,134,135].includes(key))transform(el);
                else if([65,66,67].includes(key)&&el.props[65]!==undefined&&el.props[66]!==undefined)s.backgroundImage=`linear-gradient(to bottom,${color(el.props[65])},${color(el.props[66])})`;
            },
            setText(id,text){const el=nodes.get(id);if(el.textContent!==text)el.textContent=text;},
            measureText(text,slot){context.font=`${fonts[slot]||18}px PJ4EDK2UI`;return context.measureText(text).width;},
            setImage(id,handle){const el=nodes.get(id),t=textures.get(handle);if(t){el.style.cssText+=`;background:${color(t.fill)};color:${color(t.ink)};border:1px solid ${color(t.border)};border-radius:${t.radius}px;font:${fonts[t.font_slot]||20}px PJ4EDK2UI;font-variant-ligatures:none;padding:8px 20px`;el.textContent=t.text;}},
            freeTexture(handle){textures.delete(handle);},
        };
    }

    async prepare(assets) {
        const bytes=assets.get('font.ttf');
        if(this.fontBytes===bytes)return;
        const font=new FontFace('PJ4EDK2UI',bytes);
        await font.load();
        if(this.font)document.fonts.delete(this.font);
        document.fonts.add(font);this.font=font;this.fontBytes=bytes;
    }

    reset(){this.dispose();this.surface.props={};this.nodes.set(1,this.surface);this.next=2;}
    dispose(){this.nodes.clear();this.textures.clear();this.surface.replaceChildren();}
    paint(){}
    memoryBytes(){return null;}
    drawHash(){return null;}
    bakeName(card){const handle=this.next++;this.textures.set(handle,card);return {handle,width:card.size[0],height:card.size[1]};}
}
