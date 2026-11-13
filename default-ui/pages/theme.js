// Shared visual rules. All dimensions are logical pixels on a 1280 x 720 canvas.
globalThis.Rimuks = (() => {
    const P = { w: 1, h: 2, pos: 24, top: 25, left: 28, display: 29,
        overflow: 30, z: 31, bg: 64, gradFrom: 65, gradTo: 66, gradDir: 67,
        radius: 68, opacity: 69, border: 70, borderWidth: 71,
        fg: 96, font: 97, align: 98, tracking: 100,
        scale: 130, rotate: 131, originX: 134, originY: 135 };
    const C = { background: 0xc3dff0, header: 0x367ba9, headerEdge: 0x5994ba,
        headerHover: 0x488bb6, footer: 0xedf6fb, ink: 0x214d69, muted: 0x5c8097,
        line: 0x729fbb, white: 0xffffff, card: 0xd6e9f4, hover: 0xe5f1f8,
        selected: 0xf7fcff, accent: 0x327eaf, border: 0xa6c9dd,
        softBorder: 0xb7d5e6, shadow: 0x8bb7d0 };
    const R = { P, C, L: { top: 84, footer: 64 }, root: 1, now: 0, dt: 16,
        scale: 1, w: 1280, h: 720, pointer: {x: -1, y: -1, down: false} };
    R.color = rgb => (0xff000000 | ((rgb & 255) << 16) | (rgb & 0xff00) | (rgb >>> 16)) >>> 0;
    // This page owns its property animations. Keep unchanged values out of
    // the JS/native bridge and avoid marking retained layout dirty each tick.
    const properties = new Map(), labels = new Map(), measurements = new Map();
    R.prop = (id, key, value) => {
        let saved = properties.get(id);
        if (!saved) { saved = {}; properties.set(id, saved); }
        if (saved[key] === value) return;
        ui.setProp(id, P[key], value); saved[key] = value;
    };
    R.destroy = id => {
        ui.destroyNode(id); properties.clear(); labels.clear();
    };
    R.box = (x, y, w, h, fill, parent = R.root, radius = 0) => {
        const id = ui.createNode(0);
        R.prop(id, 'pos', 1); R.prop(id, 'left', x); R.prop(id, 'top', y);
        R.prop(id, 'w', w); R.prop(id, 'h', h);
        if (fill !== null) R.prop(id, 'bg', R.color(fill));
        if (radius) R.prop(id, 'radius', radius);
        ui.insertBefore(parent, id, 0); return id;
    };
    R.text = (x, y, w, text, slot = 1, fill = C.ink, parent = R.root) => {
        const id = ui.createNode(1);
        R.prop(id, 'pos', 1); R.prop(id, 'left', x); R.prop(id, 'top', y);
        R.prop(id, 'w', w); R.prop(id, 'font', slot); R.prop(id, 'fg', R.color(fill));
        R.label(id, text); ui.insertBefore(parent, id, 0); return id;
    };
    R.border = (id, fill, width = 1) => { R.prop(id, 'border', R.color(fill)); R.prop(id, 'borderWidth', width); };
    R.show = (id, on) => R.prop(id, 'display', on ? 0 : 1);
    R.label = (id, value) => {
        const text = String(value);
        if (labels.get(id) === text) return;
        ui.setText(id, text); labels.set(id, text);
    };
    R.fill = (id, fill) => R.prop(id, 'bg', R.color(fill));
    R.ink = (id, fill) => R.prop(id, 'fg', R.color(fill));
    R.opacity = (id, value) => R.prop(id, 'opacity', value);
    R.inside = (e, x, y, w, h) => e.x >= x && e.y >= y && e.x < x + w && e.y < y + h;
    R.emit = action => bootEmit(JSON.stringify(action));
    R.clamp = t => Math.max(0, Math.min(1, t));
    R.ease = t => 1 - Math.pow(1 - R.clamp(t), 3);
    R.smooth = t => { t = R.clamp(t); return t * t * t * (t * (t * 6 - 15) + 10); };
    R.lerp = (a, b, t) => a + (b - a) * t;
    R.follow = (a, b, ms = 90) => Math.abs(a - b) < 0.005 ? b : R.lerp(a, b, 1 - Math.exp(-R.dt / ms));
    R.mix = (a, b, t) => [16, 8, 0].reduce((rgb, shift) =>
        rgb | (Math.round(R.lerp((a >>> shift) & 255, (b >>> shift) & 255, t)) << shift), 0);
    R.gradient = (id, from, to) => {
        R.prop(id, 'gradFrom', R.color(from)); R.prop(id, 'gradTo', R.color(to)); R.prop(id, 'gradDir', 3);
    };
    R.caps = (x, y, w, text, fill = C.muted, parent = R.root) => {
        const id = R.text(x, y, w, text, 5, fill, parent); R.prop(id, 'tracking', 1); return id;
    };
    R.button = (x, y, w, h, label, parent, accent = false) => {
        const id = R.box(x, y, w, h, accent ? C.accent : C.footer, parent, 8);
        const t = R.text(0, (h - 26) / 2, w, label, 1, accent ? C.white : C.ink, id);
        R.prop(t, 'align', 1); return id;
    };
    R.keycap = (x, y, w, label, parent) => {
        const id = R.box(x, y, w, 24, C.selected, parent, 5); R.border(id, C.softBorder);
        const text = R.text(0, 3, w, label, 5, C.ink, id); R.prop(text, 'align', 1); return id;
    };
    R.ellipsize = (value, width, slot) => {
        const measure = text => {
            const key = slot + ':' + text;
            if (!measurements.has(key)) {
                if (measurements.size >= 512) measurements.clear();
                measurements.set(key, ui.measureText(text, slot));
            }
            return measurements.get(key);
        };
        if (measure(value) <= width) return value;
        while (value.length && measure(value + '...') > width) value = value.slice(0, -1);
        return value + '...';
    };
    return R;
})();
