(() => {
    const R = Rimuks, C = R.C;
    R.settingsDirty = () => R.pendingTimeout === null || R.pendingTimeout !== R.cfg.timeout_seconds;
    R.makeSettings = () => {
        R.settingsX = R.w - 476; R.settingsY = 104; R.settingsVisual = 0; R.settingsFocus = 0;
        R.settingsDrawnVisible = null; R.settingsDrawnVisual = null;
        R.settingsShade = R.box(0, R.L.top, R.w, R.h - R.L.top, 0x326584);
        R.prop(R.settingsShade, 'z', 80);
        R.settingsShadow = R.box(R.settingsX - 3, 111, 446, 216, C.shadow, R.root, 12);
        R.prop(R.settingsShadow, 'z', 99);
        R.settingsPanel = R.box(R.settingsX, R.settingsY, 440, 216, C.selected, R.root, 10);
        const panel = R.settingsPanel; R.prop(panel, 'z', 100); R.border(panel, C.softBorder);
        R.settingsLabel = R.text(24, 35, 190, '', 1, C.ink, panel);
        R.settingsInput = R.box(220, 24, 136, 44, C.footer, panel, 6);
        R.settingsValue = R.text(14, 8, 110, '', 6, C.ink, R.settingsInput);
        R.settingsCaret = R.box(14, 10, 1, 24, C.accent, R.settingsInput);
        R.text(364, 35, 24, 's', 1, C.muted, panel);
        R.settingsClose = R.button(400, 8, 28, 28, 'x', panel);
        R.makeDebugSetting(panel);
        R.settingsSave = R.button(284, 156, 132, 40, 'Save', panel, true);
        R.updateSettings();
    };
    R.openSettings = () => {
        if (R.booting || R.settingsSaving || R.defaultSaving) return;
        R.autoActive = false;
        if (R.settingsOpen) { R.settingsOpen = false; return; }
        R.settingsOpen = true; R.pendingTimeout = R.cfg.timeout_seconds;
        R.pendingText = String(R.pendingTimeout); R.settingsFocus = 0; R.settingsSelectAll = true;
        R.settingsFailed = false;
    };
    R.editSettings = text => {
        if (R.settingsSaving) return;
        R.pendingText = text; R.settingsFailed = false;
        R.pendingTimeout = /^\d{1,3}$/.test(text) && Number(text) <= 120 ? Number(text) : null;
    };
    R.saveSettings = () => {
        if (R.settingsSaving || R.pendingTimeout === null || !R.settingsDirty()) return;
        R.settingsSaving = true; R.settingsSubmitted = R.pendingTimeout; R.settingsFailed = false;
        R.emit({type: 'set_timeout', seconds: R.pendingTimeout});
    };
    R.settingsEvent = e => {
        if (!R.settingsOpen) return R.settingsVisual > 0.01;
        if (e.type === 'key') {
            if (e.key === 'Escape') R.settingsOpen = false;
            else if (e.key === 'Tab') R.settingsFocus = (R.settingsFocus + 1) % 3;
            else if (e.key === 'Enter') {
                if (R.settingsFocus === 1) R.toggleDebug(); else R.saveSettings();
            }
            else if (R.settingsFocus === 0 && !R.settingsSaving) {
                if (['ArrowLeft', 'ArrowDown', 'ArrowRight', 'ArrowUp'].includes(e.key)) {
                    const delta = ['ArrowLeft', 'ArrowDown'].includes(e.key) ? -1 : 1;
                    R.editSettings(String(Math.max(0, Math.min(120, (R.pendingTimeout || 0) + delta))));
                    R.settingsSelectAll = true;
                } else if (e.key === 'SelectAll') R.settingsSelectAll = true;
                else if (e.key === 'Backspace' || e.key === 'Delete') {
                    R.editSettings(R.settingsSelectAll || e.key === 'Delete' ? '' : R.pendingText.slice(0, -1));
                    R.settingsSelectAll = false;
                } else if (/^\d$/.test(e.key)) {
                    const text = R.settingsSelectAll ? e.key : R.pendingText + e.key;
                    if (text.length <= 3) R.editSettings(text);
                    R.settingsSelectAll = false;
                }
            }
        } else if (e.type === 'click') {
            const x = R.settingsX, y = R.settingsY;
            if (!R.inside(e, x, y, 440, 216) || R.inside(e, x + 400, y + 8, 28, 28)) R.settingsOpen = false;
            else if (R.inside(e, x + 220, y + 24, 136, 44)) {
                R.settingsFocus = 0; R.settingsSelectAll = true;
            } else if (R.inside(e, x + 24, y + 88, 392, 44)) {
                R.settingsFocus = 1; R.toggleDebug();
            } else if (R.inside(e, x + 284, y + 156, 132, 40)) {
                R.settingsFocus = 2; R.saveSettings();
            }
        }
        return true;
    };
    R.updateSettings = () => {
        R.settingsVisual = R.follow(R.settingsVisual, R.settingsOpen ? 1 : 0, 55);
        const p = R.settingsVisual, visible = p > 0.005;
        R.settingsY = 104 + (1 - p) * 10;
        if (R.settingsDrawnVisible !== visible) {
            for (const node of [R.settingsPanel, R.settingsShadow, R.settingsShade]) R.show(node, visible);
            R.settingsDrawnVisible = visible;
        }
        if (R.settingsDrawnVisual !== p) {
            R.prop(R.settingsPanel, 'top', R.settingsY); R.opacity(R.settingsPanel, p);
            R.prop(R.settingsShadow, 'top', R.settingsY + 7); R.opacity(R.settingsShadow, p * 0.32);
            R.opacity(R.settingsShade, p * 0.13);
            R.settingsDrawnVisual = p;
        }
        R.label(R.settingsLabel, 'Auto boot delay' + (R.settingsDirty() ? ' *' : ''));
        R.label(R.settingsValue, R.pendingText);
        const invalid = R.pendingTimeout === null;
        R.border(R.settingsInput, invalid || R.settingsFailed ? 0xb45a64 : R.settingsFocus === 0 ? C.accent : C.softBorder);
        const caretOn = R.settingsOpen && R.settingsFocus === 0 && !R.settingsSaving && Math.floor(R.now / 500) % 2 === 0;
        // Keep the draw-list shape stable while blinking on the opaque input.
        // During fade transitions preserve normal alpha compositing.
        R.show(R.settingsCaret, p === 1 || caretOn);
        R.fill(R.settingsCaret, p === 1 && !caretOn ? C.footer : C.accent);
        R.prop(R.settingsCaret, 'left', 15 + ui.measureText(R.pendingText, 6));
        const enabled = !invalid && R.settingsDirty() && !R.settingsSaving;
        const hot = enabled && R.inside(R.pointer, R.settingsX + 284, R.settingsY + 156, 132, 40);
        R.fill(R.settingsSave, R.settingsFailed ? 0xb45a64 : hot ? C.header : C.accent);
        R.opacity(R.settingsSave, enabled ? 1 : 0.4);
        R.border(R.settingsSave, R.settingsFocus === 2 ? C.ink : C.accent, R.settingsFocus === 2 ? 2 : 1);
        R.updateDebugSetting();
    };
})();
