// Bake once, then rotate an ordinary image through the stock PocketJS API.
(() => {
    const R = Rimuks, C = R.C;
    R.nameCardWidth = entry => Math.ceil(Math.max(184, Math.min(340, ui.measureText(entry.name, 6) + 40)));
    R.nameCardSpec = entry => {
        const width = R.nameCardWidth(entry);
        return {
            text: R.ellipsize(entry.name, width - 40, 6), size: [width, 48],
            radius: 8, fill: R.color(C.selected), ink: R.color(C.ink),
            border: R.color(R.mix(C.softBorder, C.accent, 0.72)), font_slot: 6, text_origin: [20, 8]
        };
    };
    // UEFI pre-bakes exactly the specs that the browser requests at boot time.
    // This hook only needs the loaded font atlases, before page initialization.
    globalThis.rimuksNameCards = configJson => JSON.stringify(JSON.parse(configJson).entries.map(R.nameCardSpec));
    R.prepareBootImage = () => {
        const entry = R.cfg.entries[R.selected];
        const asset = JSON.parse(bootBakeName(JSON.stringify(R.nameCardSpec(entry))));
        const id = ui.createNode(2);
        R.prop(id, 'pos', 1); R.prop(id, 'w', asset.width); R.prop(id, 'h', asset.height);
        R.prop(id, 'originX', -0.5); R.prop(id, 'originY', -0.5); R.prop(id, 'z', 2);
        ui.setImage(id, asset.handle); ui.insertBefore(R.sceneRoot, id, 0); R.show(id, false);
        R.bootImage = {id, handle: asset.handle};
    };
    R.updateBootImage = (geometry, elapsed) => {
        if (!R.bootImage) return;
        const id = R.bootImage.id;
        R.show(id, elapsed >= R.moveAt);
        R.prop(id, 'left', geometry.left); R.prop(id, 'top', geometry.top);
        R.prop(id, 'rotate', geometry.angle);
    };
})();
