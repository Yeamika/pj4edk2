// Shared curve and card geometry, in logical pixels before viewport scaling.
(() => {
    const R = Rimuks, C = R.C;
    let previousCurve = null;
    R.makeBootCurve = () => {
        previousCurve = null;
        R.lineX = 44; R.dockGap = 20; R.panelX = R.lineX + 24 + 16;
        R.dockNameY = (R.L.top + R.h - R.L.footer) / 2;
        R.lineY = R.L.top + 48; R.lineEndY = R.h - R.L.footer - 16;
        R.lineSpan = R.lineEndY - R.lineY;
        R.rowGap = 36; R.arcDepth = 32; R.moveAt = 260; R.moveMs = 690;
        // A long, shallow right bulge; the outermost card aligns with the heading.
        R.arcEdge = R.cardX - R.rowGap - R.arcDepth;
        // One square-edged span per physical pixel row, including the longer docked line.
        // Cancel the page scale here so layout never rounds subpixel row heights to zero.
        R.curveRoot = R.box(0, 0, R.cfg.viewport[0], R.cfg.viewport[1], null);
        R.prop(R.curveRoot, 'originX', -0.5); R.prop(R.curveRoot, 'originY', -0.5);
        R.prop(R.curveRoot, 'scale', 1 / R.scale);
        R.linePixels = Math.max(1, Math.round(2 * R.scale));
        R.lines = [];
        const capacity = Math.ceil((R.lineEndY - (R.L.top + 8)) * R.scale);
        for (let i = 0; i < capacity; i++) {
            R.lines.push(R.box(0, 0, R.linePixels, 1, C.background, R.curveRoot));
        }
    };
    R.bootCurveState = elapsed => {
        const dock = R.booting ? R.smooth((elapsed - R.moveAt) / R.moveMs) : 0;
        return {dock, startY: R.lerp(R.lineY, R.L.top + 8, dock), endY: R.lineEndY};
    };
    R.arcXAtY = y => {
        const curve = R.curve;
        const u = R.clamp((y - curve.startY) / (curve.endY - curve.startY));
        const x = R.arcEdge + R.arcDepth * 4 * u * (1 - u);
        return R.lerp(x, R.lineX, curve.dock);
    };
    R.bootGeometry = (row, i, elapsed) => {
        const chosen = i === R.selected;
        const size = chosen ? R.smooth((elapsed - 40) / (R.moveAt - 40)) : 0;
        const dock = chosen ? R.curve.dock : 0;
        const width = R.lerp(R.cardW, row.compactW, size), height = R.lerp(R.cardH, 48, size);
        const angle = -90 * dock, rad = angle * Math.PI / 180;
        // Center the vertical name itself, regardless of the selected name's length.
        const dockDotY = R.dockNameY + row.compactW / 2 + R.dockGap;
        const dotY = R.lerp(R.rowY(i) + R.cardH / 2, dockDotY, dock);
        const dotX = R.arcXAtY(dotY), gap = R.lerp(R.rowGap, R.dockGap, dock);
        // The point slides along the actual curve while this attached offset
        // turns counterclockwise, carrying the name upward above the point.
        const left = dotX + gap * Math.cos(rad) + height / 2 * Math.sin(rad);
        const top = dotY + gap * Math.sin(rad) - height / 2 * Math.cos(rad);
        const right = left + Math.max(0, width * Math.cos(rad)) + Math.max(0, -height * Math.sin(rad));
        return {size, dock, width, height, left, top, angle, right, dotX, dotY};
    };
    R.updateBootCurve = () => {
        const count = Math.min(R.visibleRows, R.rows.length);
        const curve = R.curve;
        if (previousCurve && previousCurve.count === count &&
            previousCurve.startY === curve.startY && previousCurve.endY === curve.endY &&
            previousCurve.dock === curve.dock && previousCurve.scale === R.scale &&
            previousCurve.arcEdge === R.arcEdge && previousCurve.arcDepth === R.arcDepth &&
            previousCurve.lineX === R.lineX && previousCurve.linePixels === R.linePixels) return;
        const first = Math.round(R.curve.startY * R.scale);
        const rows = Math.round(R.curve.endY * R.scale) - first;
        for (let i = 0; i < R.lines.length; i++) {
            const id = R.lines[i], visible = count > 1 && i < rows;
            R.show(id, visible);
            if (!visible) continue;
            const y = (first + i + 0.5) / R.scale;
            const strength = 0.82 * Math.pow(Math.sin(Math.PI * (i + 0.5) / rows), 0.9);
            R.fill(id, R.mix(C.background, C.line, strength));
            R.prop(id, 'left', Math.round(R.arcXAtY(y) * R.scale - R.linePixels / 2));
            R.prop(id, 'top', first + i);
        }
        previousCurve = {count, startY: curve.startY, endY: curve.endY, dock: curve.dock,
            scale: R.scale, arcEdge: R.arcEdge, arcDepth: R.arcDepth,
            lineX: R.lineX, linePixels: R.linePixels};
    };
})();
