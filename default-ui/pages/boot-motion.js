// Clear details, slide down the curve, rotate the name upward, reveal the next stage.
(() => {
    const R = Rimuks, C = R.C;
    R.updateBoot = () => {
        if (!R.booting && !(R.listDrag && R.listDrag.moved)) R.visualScroll = R.follow(R.visualScroll, R.scrollOffset, 85);
        R.prop(R.listClip, 'overflow', R.booting ? 0 : 1);
        const elapsed = R.booting ? Math.max(0, R.now - R.bootAt) : 0;
        const raw = R.clamp(elapsed / 1000);
        const detail = 1 - R.ease(elapsed / 160), others = 1 - R.ease(elapsed / 210);
        R.animationProgress = raw;
        R.curve = R.bootCurveState(elapsed);
        const selected = R.selected === null ? null : R.rows[R.selected];
        const selectedGeometry = selected ? R.bootGeometry(selected, R.selected, elapsed) : null;
        if (selectedGeometry) R.updateBootImage(selectedGeometry, elapsed);
        R.updateBootCurve();
        R.rows.forEach((row, i) => {
            const chosen = i === R.selected;
            const visible = i + 1 > R.visualScroll && i < R.visualScroll + R.visibleRows;
            const hover = visible && !R.settingsOpen && !R.booting && R.hitBootRow(R.pointer, row, i);
            row.hot = R.follow(row.hot, hover ? 1 : 0);
            row.active = R.follow(row.active, chosen ? 1 : 0, 65);
            const pressed = hover && R.pointer.down;
            const geometry = R.bootGeometry(row, i, elapsed);
            const {width, height, left, top, dotY} = geometry, p = geometry.size;
            const padding = R.lerp(22, 20, p), rowAlpha = chosen ? 1 : others;
            R.updateDefaultControl(row, i, geometry, visible, detail);
            R.opacity(row.icon, detail);
            row.dotX = geometry.dotX;
            for (const node of [row.card, row.shadow, row.dot, row.halo, row.leader]) R.show(node, visible);
            R.prop(row.halo, 'left', row.dotX - 13); R.prop(row.halo, 'top', dotY - 13);
            R.opacity(row.halo, row.active * 0.65 * detail);
            R.prop(row.dot, 'left', row.dotX - 7); R.prop(row.dot, 'top', dotY - 7);
            R.opacity(row.dotCore, 1 - row.active); R.opacity(row.dot, rowAlpha);
            R.prop(row.leader, 'left', row.dotX + 14); R.prop(row.leader, 'top', dotY);
            R.prop(row.leader, 'w', Math.max(0, left - row.dotX - 22));
            R.gradient(row.leader, R.mix(C.background, C.line, 0.12 + row.active * 0.22), C.background);
            R.opacity(row.leader, detail);
            for (const node of [row.card, row.shadow]) {
                R.prop(node, 'left', left); R.prop(node, 'top', top + (node === row.shadow ? 3 : 0));
                R.prop(node, 'w', width); R.prop(node, 'h', height); R.prop(node, 'radius', R.lerp(10, 8, p));
            }
            // The live text/card finishes shrinking before the image starts turning.
            R.show(row.card, visible && !(chosen && R.booting && elapsed >= R.moveAt));
            R.opacity(row.shadow, row.active * 0.22 * detail);
            let fill = R.mix(R.mix(C.card, C.hover, row.hot), C.selected, row.active);
            if (pressed) fill = R.mix(fill, C.border, 0.22);
            R.fill(row.card, fill); R.opacity(row.card, rowAlpha);
            R.border(row.card, R.mix(C.softBorder, C.accent, row.active * 0.72 + row.hot * 0.15),
                chosen && R.inputKind === 'keyboard' && !R.booting ? 1.5 : 1);
            const nameWidth = width - padding * 2 - 148 * (1 - p);
            R.prop(row.name, 'left', padding + 48 * (1 - p)); R.prop(row.name, 'top', R.lerp(11, 8, p) + (pressed ? 1 : 0));
            R.prop(row.name, 'w', nameWidth);
            R.label(row.name, R.ellipsize(R.cfg.entries[i].name, nameWidth, 6));
            R.opacity(row.path, detail); R.show(row.path, detail > 0);
            R.prop(row.badge, 'left', width - 104); R.opacity(row.badge, detail);
            R.show(row.badge, R.armed === i && detail > 0);
            const armed = R.armed === i;
            R.fill(row.badge, armed ? C.accent : C.hover); R.ink(row.badgeText, armed ? C.white : C.accent);
            R.label(row.badgeText, armed ? 'START  >' : '');
            R.show(row.arrow, !chosen && row.hot > 0.01 && detail > 0); R.opacity(row.arrow, row.hot * detail);
        });
        R.opacity(R.bootHeading, detail); R.opacity(R.bootLegend, detail); R.opacity(R.bootActions, detail);
        if (R.rows.length) {
            R.label(R.hint, R.armed !== null ? 'Press the selected entry again to start.' :
                R.autoActive ? 'Select an entry to pause automatic startup.' : 'Choose an entry, then press it again to start.');
            if (R.defaultSaving) R.label(R.hint, 'Saving default entry and reading configuration...');
            else if (R.defaultSaveFailed) R.label(R.hint, 'Could not save the default. Press its checkbox to retry.');
            R.label(R.legend, '');
        }
        R.label(R.rangeLabel, R.rows.length > R.visibleRows ?
            (Math.floor(R.scrollOffset) + 1) + '-' + Math.min(R.rows.length, Math.ceil(R.scrollOffset) + R.visibleRows) + ' of ' + R.rows.length : '');
        const reveal = R.smooth((elapsed - 230) / 720);
        // Track the rotating card's actual right edge, leaving a 16 px gutter.
        const left = Math.min(R.w, Math.max(R.lerp(R.w, R.panelX, reveal),
            selectedGeometry ? selectedGeometry.right + 16 : R.w));
        R.prop(R.panel, 'left', left); R.prop(R.panel, 'w', R.w - left);
        if (R.booting && raw === 1 && !R.emitted && !R.fastbootBusy) {
            R.emitted = true;
            const entry = R.cfg.entries[R.selected];
            const x = Math.round(left * R.scale), y = Math.round(R.L.top * R.scale);
            R.emit({type: 'boot', id: entry.id, path: entry.path, display_regions: [{
                id: 'boot-output', display_id: R.cfg.display_id,
                position: [x + R.viewportOrigin[0], y + R.viewportOrigin[1]],
                size: [R.cfg.viewport[0] - x, R.cfg.viewport[1] - y]
            }]});
        }
    };
})();
