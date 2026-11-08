// Small code-drawn icons; no image decoding or extra font glyphs are needed.
(() => {
    const R = Rimuks, C = R.C;
    R.icon = (kind, x, y, parent, ink = C.accent) => {
        const root = R.box(x, y, 32, 32, null, parent);
        const bar = (x, y, w, h) => R.box(x, y, w, h, ink, root);
        if (kind === 'windows') {
            for (const x of [3, 18]) for (const y of [3, 18]) bar(x, y, 11, 11);
        } else if (kind === 'terminal') {
            const frame = R.box(1, 3, 30, 26, null, root, 3); R.border(frame, ink, 2);
            R.text(5, 6, 24, '>_', 0, ink, root);
        } else if (kind === 'recovery') {
            bar(12, 3, 8, 26); bar(3, 12, 26, 8);
        } else if (kind === 'shutdown') {
            bar(14, 1, 4, 17);
            bar(6, 6, 4, 4); bar(22, 6, 4, 4);
            bar(2, 10, 4, 14); bar(26, 10, 4, 14);
            bar(6, 24, 4, 4); bar(22, 24, 4, 4); bar(10, 28, 12, 4);
        } else {
            const screen = R.box(1, 3, 30, 22, null, root, 3); R.border(screen, ink, 2);
            bar(13, 25, 6, 3); bar(7, 28, 18, 2);
            bar(7, 9, 6, 6); bar(17, 9, 7, 2); bar(17, 13, 7, 2);
        }
        return root;
    };
    R.iconButton = (x, y, kind, label, parent) => {
        const root = R.box(x, y, 140, 36, C.footer, parent, 8);
        const icon = R.icon(kind, 12, 10, root, C.ink);
        R.prop(icon, 'originX', -0.5); R.prop(icon, 'originY', -0.5); R.prop(icon, 'scale', 0.5);
        R.text(40, 5, 94, label, 1, C.ink, root);
        return root;
    };
    R.checkmark = parent => {
        const mark = R.box(0, 0, 40, 40, null, parent);
        // Pixel steps keep the tick crisp at the same sampling policy as the page.
        for (let i = 0; i < 5; i++) R.box(8 + i * 2, 19 + i * 2, 4, 4, C.white, mark);
        for (let i = 0; i < 9; i++) R.box(16 + i * 2, 27 - i * 2, 4, 4, C.white, mark);
        return mark;
    };
})();
