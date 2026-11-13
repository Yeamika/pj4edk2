(() => {
    const R = Rimuks, C = R.C;
    R.makeBoot = () => {
        R.scene = 'boot';
        const preferred = R.cfg.entries.findIndex(entry => entry.id === R.resolvedDefault());
        R.selected = R.cfg.entries.length ? Math.max(0, preferred) : null;
        R.cardX = (R.w - 960) / 2 + 224; R.cardW = Math.min(600, R.w - R.cardX - 112); R.cardH = 72;
        R.listY = 248; R.rowStep = 88;
        R.visibleRows = Math.max(1, Math.min(4, Math.floor((R.h - 332) / R.rowStep)));
        R.scrollOffset = Math.max(0, (R.selected || 0) - R.visibleRows + 1);
        R.visualScroll = R.scrollOffset; R.listDrag = null;
        R.bootHeading = R.box(R.cardX, 0, R.cardW, 232, null);
        R.caps(1, 123, 380, 'BOOT OPTIONS', C.muted, R.bootHeading);
        R.title = R.text(0, 148, R.cardW, 'Choose your system', 3, C.ink, R.bootHeading);
        R.hint = R.text(1, 199, R.cardW, 'Select an entry. Confirm with a second press.', 0, C.muted, R.bootHeading);
        const count = R.text(R.cardW - 124, 123, 124,
            String(R.cfg.entries.length).padStart(2, '0') + ' available', 0, C.muted, R.bootHeading);
        R.prop(count, 'align', 2);
        const rowParent = R.root;
        R.listClip = R.box(0, R.listY - 4, R.w, R.visibleRows * R.rowStep, null);
        R.prop(R.listClip, 'overflow', 1); R.prop(R.listClip, 'z', 1);
        R.root = R.box(0, 4 - R.listY, R.w, R.h, null, R.listClip);
        R.rows = R.cfg.entries.map((entry, i) => {
            const halo = R.box(0, 0, 26, 26, C.softBorder, R.root, 13);
            const dot = R.box(0, 0, 14, 14, C.accent, R.root, 7);
            const dotCore = R.box(2, 2, 10, 10, C.background, dot, 5);
            const leader = R.box(0, 0, 1, 1, C.line);
            const shadow = R.box(R.cardX, 0, R.cardW, R.cardH, C.shadow, R.root, 11);
            const card = R.box(R.cardX, 0, R.cardW, R.cardH, C.card, R.root, 10);
            R.prop(card, 'overflow', 1);
            const icon = R.icon(entry.icon, 22, 20, card);
            const name = R.text(70, 11, R.cardW - 192, entry.name, 6, C.ink, card);
            const path = R.text(71, 42, R.cardW - 96, R.ellipsize(entry.path, R.cardW - 96, 0), 0, C.muted, card);
            const badge = R.box(R.cardW - 104, 14, 82, 24, C.hover, card, 5);
            const badgeText = R.text(0, 3, 82, '', 5, C.accent, badge); R.prop(badgeText, 'align', 1);
            const arrow = R.text(R.cardW - 44, 20, 22, '>', 1, C.accent, card);
            const compactW = R.nameCardWidth(entry);
            const defaultBox = R.box(0, 0, 40, 40, C.card, R.root, 6);
            R.prop(defaultBox, 'z', 2);
            const defaultTick = R.checkmark(defaultBox);
            const defaultLabel = R.text(-18, 44, 76, '', 5, C.muted, defaultBox);
            R.prop(defaultLabel, 'align', 1);
            return {dot, dotCore, halo, leader, shadow, card, name, path, icon, badge, badgeText, arrow,
                defaultBox, defaultTick, defaultLabel, compactW, dotX: 0, hot: 0, active: i === R.selected ? 1 : 0};
        });
        R.root = rowParent;
        R.makeBootCurve();
        for (const row of R.rows) {
            for (const node of [row.halo, row.dot, row.shadow, row.card]) R.prop(node, 'z', 1);
        }
        R.bootLegend = R.box(R.cardX, 225, R.cardW, 18, null);
        R.legend = R.text(0, 0, 420, 'Your selection stays on this screen until confirmed.', 5, C.muted, R.bootLegend);
        R.rangeLabel = R.text(R.cardW - 120, 0, 120, '', 5, C.muted, R.bootLegend);
        R.prop(R.rangeLabel, 'align', 2);
        if (!R.rows.length) {
            R.label(R.title, 'No boot entries'); R.label(R.hint, 'No startup options are available.');
            R.label(R.legend, 'Check the boot configuration, then try again.');
        }
        R.panel = R.box(R.w, R.L.top, 0, R.h - R.L.top, 0x000000);
        R.prop(R.panel, 'z', 10); R.makeBootActions(); R.updateBoot();
    };
    R.rowY = i => R.listY + (i - R.visualScroll) * R.rowStep;
    R.limitScroll = value => Math.max(0, Math.min(Math.max(0, R.rows.length - R.visibleRows), value));
    R.bootPointer = point => {
        if (R.scene !== 'boot' || R.settingsOpen || R.booting) { R.listDrag = null; return; }
        if (point.down) {
            if (!R.listDrag && !R.pointer.down && point.y >= R.L.top + 80 && point.y < R.bootActionsY) {
                R.listDrag = {y: point.y, offset: R.visualScroll, moved: false}; R.scrollOffset = R.visualScroll;
            }
            const drag = R.listDrag;
            if (drag) {
                if (Math.abs(point.y - drag.y) > 12) drag.moved = true;
                if (drag.moved) { R.scrollOffset = R.limitScroll(drag.offset + (drag.y - point.y) / R.rowStep); R.visualScroll = R.scrollOffset; R.armed = null; }
            }
        } else if (R.listDrag) {
            if (R.listDrag.moved) R.scrollOffset = Math.round(R.scrollOffset);
            R.listDrag = null;
        }
    };
    R.beginBoot = () => {
        if (R.selected === null || R.booting || R.defaultSaving || R.poweringOff || R.fastbootBusy) return;
        if (R.selected < R.scrollOffset) R.scrollOffset = R.selected;
        if (R.selected >= R.scrollOffset + R.visibleRows) R.scrollOffset = R.selected - R.visibleRows + 1;
        R.prepareBootImage();
        R.autoActive = false; R.booting = true; R.bootAt = R.now; R.armed = R.selected;
    };
    R.selectEntry = i => {
        if (R.booting || i < 0 || i >= R.rows.length) return;
        if (R.armed === i) R.beginBoot();
        else { R.selected = i; R.armed = i; }
    };
    R.bootEvent = e => {
        if (R.bootActionEvent(e)) return;
        if (e.type === 'click') {
            for (let i = 0; i < R.rows.length; i++) {
                if (e.y < R.listY || e.y >= R.listY + R.visibleRows * R.rowStep) break;
                const geometry = R.bootGeometry(R.rows[i], i, 0);
                if (R.inside(e, R.defaultControlX(geometry) - 4, R.rowY(i) + 4, 48, 64)) {
                    R.requestDefault(i); return;
                }
                if (R.hitBootRow(e, R.rows[i], i)) { R.selectEntry(i); break; }
            }
        } else if (e.type === 'scroll') {
            R.scrollOffset = Math.max(0, Math.min(Math.max(0, R.rows.length - R.visibleRows),
                Math.round(R.scrollOffset) + (e.delta > 0 ? -1 : 1)));
        } else if (e.type === 'key' && R.rows.length) {
            if (['ArrowDown', 'ArrowUp', 'Tab'].includes(e.key)) {
                const delta = e.key === 'ArrowUp' ? -1 : 1;
                R.selected = (R.selected + delta + R.rows.length) % R.rows.length; R.armed = null;
                if (R.selected < R.scrollOffset) R.scrollOffset = R.selected;
                if (R.selected >= R.scrollOffset + R.visibleRows) R.scrollOffset = R.selected - R.visibleRows + 1;
            } else if (e.key === 'Enter') R.selectEntry(R.selected);
        }
    };
    R.hitBootRow = (e, row, i) => {
        const y = R.rowY(i), dotX = R.arcXAtY(y + R.cardH / 2);
        return R.inside(e, dotX + R.rowGap, y, R.cardW, R.cardH) ||
            R.inside(e, dotX - 20, y + R.cardH / 2 - 20, 40, 40);
    };
})();
