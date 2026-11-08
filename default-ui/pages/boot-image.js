// Bake once, then rotate an ordinary image through the stock PocketJS API.
(() => {
    const R = Rimuks, C = R.C;
    R.prepareBootImage = () => {
        const row = R.rows[R.selected], entry = R.cfg.entries[R.selected];
        const asset = JSON.parse(bootBakeName(JSON.stringify({
            text: R.ellipsize(entry.name, row.compactW - 40, 6), size: [row.compactW, 48],
            radius: 8, fill: R.color(C.selected), ink: R.color(C.ink),
            border: R.color(R.mix(C.softBorder, C.accent, 0.72)), font_slot: 6, text_origin: [20, 8]
        })));
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
