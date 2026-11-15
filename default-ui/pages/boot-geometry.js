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
        // Ten chord segments approximate the shallow parabola. Reuse the scene
        // parent: the curve adds exactly ten nodes, including no extra group.
        R.linePixels = Math.max(1, Math.round(2 * R.scale));
        R.lines = [];
        for (let i = 0; i < 10; i++) {
            const id = R.box(0, 0, R.linePixels / R.scale, 1, null);
            R.prop(id, 'originX', -0.5); R.prop(id, 'originY', -0.5);
            R.prop(id, 'gradDir', 1); // Local top to bottom, before rotation.
            R.lines.push(id);
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
        const first = Math.round(curve.startY * R.scale);
        const last = Math.round(curve.endY * R.scale);
        const width = R.linePixels / R.scale;
        const fade = u => R.mix(C.background, C.line,
            0.82 * Math.pow(Math.max(0, Math.sin(Math.PI * u)), 0.9));
        for (let i = 0; i < R.lines.length; i++) {
            const id = R.lines[i], visible = count > 1 && last > first;
            R.show(id, visible);
            if (!visible) continue;
            const u0 = i / R.lines.length, u1 = (i + 1) / R.lines.length;
            const y0 = (first + (last - first) * u0) / R.scale;
            const y1 = (first + (last - first) * u1) / R.scale;
            const x0 = R.arcXAtY(y0), x1 = R.arcXAtY(y1);
            const dx = x1 - x0, dy = y1 - y0, length = Math.hypot(dx, dy);
            const angle = -Math.atan2(dx, dy);
            // Anchor each thin rectangle on its centerline. A one-physical-
            // pixel joint overlap prevents gaps after layout/pixel snapping.
            R.prop(id, 'left', x0 - width / 2 * Math.cos(angle));
            R.prop(id, 'top', y0 - width / 2 * Math.sin(angle));
            R.prop(id, 'w', width);
            R.prop(id, 'h', length + (i + 1 < R.lines.length ? 1 / R.scale : 0));
            R.prop(id, 'rotate', angle * 180 / Math.PI);
            R.prop(id, 'gradFrom', R.color(fade(u0)));
            R.prop(id, 'gradTo', R.color(fade(u1)));
        }
        previousCurve = {count, startY: curve.startY, endY: curve.endY, dock: curve.dock,
            scale: R.scale, arcEdge: R.arcEdge, arcDepth: R.arcDepth,
            lineX: R.lineX, linePixels: R.linePixels};
    };
})();
