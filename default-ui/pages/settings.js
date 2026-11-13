(() => {
    const R = Rimuks, C = R.C;
    R.settingsDirty = () => R.pendingTimeout === null || R.pendingTimeout !== R.cfg.timeout_seconds;
    R.makeSettings = () => {
        R.settingsWidth = 448; R.settingsHeight = 220; R.settingsX = R.w - R.settingsWidth - 36; R.settingsY = 104; R.settingsVisual = 0; R.settingsFocus = 0;
        R.settingsDrawnVisible = null; R.settingsDrawnVisual = null; R.keypadOpen = false;
        R.settingsShade = R.box(0, R.L.top, R.w, R.h - R.L.top, 0x326584);
        R.prop(R.settingsShade, 'z', 80);
        R.settingsShadow = R.box(R.settingsX - 3, 111, 454, 220, C.shadow, R.root, 12);
        R.prop(R.settingsShadow, 'z', 99);
        R.settingsPanel = R.box(R.settingsX, R.settingsY, 448, 220, C.selected, R.root, 10);
        const panel = R.settingsPanel; R.prop(panel, 'z', 100); R.border(panel, C.softBorder);
        R.settingsLabel = R.text(24, 35, 190, '', 1, C.ink, panel);
        R.settingsInput = R.box(220, 24, 136, 44, C.footer, panel, 6);
        R.settingsValue = R.text(14, 8, 110, '', 6, C.ink, R.settingsInput);
        R.settingsCaret = R.box(14, 10, 1, 24, C.accent, R.settingsInput);
        R.text(364, 35, 24, 's', 1, C.muted, panel);
        R.settingsClose = R.button(400, 8, 28, 28, 'x', panel);
        R.makeDebugSetting(panel);
        R.settingsReload = R.button(24, 156, 240, 40, '', panel);
        R.settingsReloadLabel = R.text(0, 7, 240, 'Reload boot list', 1, C.ink, R.settingsReload);
        R.prop(R.settingsReloadLabel, 'align', 1);
        R.keypadKeys = ['1','2','3','4','5','6','7','8','9','Delete','0','Enter'];
        R.keypadButtons = R.keypadKeys.map((key, i) => R.button(448 + (i % 3) * 72,
            24 + Math.floor(i / 3) * 62, 64, 52, key, panel));
        R.settingsSave = R.button(284, 156, 132, 40, 'Save', panel, true);
        R.updateSettings();
    };
    R.openSettings = () => {
        if (R.booting || R.settingsSaving || R.defaultSaving) return;
        R.autoActive = false;
        if (R.settingsOpen) { R.settingsOpen = false; return; }
        R.settingsOpen = true; R.keypadOpen = false; R.pendingTimeout = R.cfg.timeout_seconds;
        R.pendingText = String(R.pendingTimeout); R.settingsFocus = 0; R.settingsSelectAll = true;
        R.settingsFailed = false; R.listReloadPending = false; R.listReloadFailed = false;
    };
    R.editSettings = text => {
        if (R.settingsSaving) return;
        R.pendingText = text; R.settingsFailed = false;
        R.pendingTimeout = /^\d{1,2}$/.test(text) && Number(text) <= 99 ? Number(text) : null;
    };
    R.keypadInput = key => {
        if (R.settingsSaving) return;
        if (key === 'Enter') { R.keypadOpen = false; R.saveSettings(); return; }
        if (key === 'Delete') R.editSettings(R.settingsSelectAll ? '' : R.pendingText.slice(0, -1));
        else {
            const text = R.settingsSelectAll ? key : R.pendingText + key;
            if (text.length <= 2) R.editSettings(text);
        }
        R.settingsSelectAll = false;
    };
    R.reloadBootList = () => {
        if (R.settingsSaving || R.listReloadPending) return;
        R.listReloadPending = true; R.listReloadFailed = false;
        R.autoActive = false;
        R.emit({type: 'recovery', operation: 'reload-list'});
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
            else if (e.key === 'Tab') R.settingsFocus = (R.settingsFocus + 1) % 4;
            else if (e.key === 'Enter') {
                if (R.settingsFocus === 1) R.toggleDebug(); else if (R.settingsFocus === 3) R.reloadBootList(); else R.saveSettings();
            }
            else if (R.settingsFocus === 0 && !R.settingsSaving) {
                if (['ArrowLeft', 'ArrowDown', 'ArrowRight', 'ArrowUp'].includes(e.key)) {
                    const delta = ['ArrowLeft', 'ArrowDown'].includes(e.key) ? -1 : 1;
                    R.editSettings(String(Math.max(0, Math.min(99, (R.pendingTimeout || 0) + delta))));
                    R.settingsSelectAll = true;
                } else if (e.key === 'SelectAll') R.settingsSelectAll = true;
                else if (e.key === 'Backspace' || e.key === 'Delete') {
                    R.editSettings(R.settingsSelectAll || e.key === 'Delete' ? '' : R.pendingText.slice(0, -1));
                    R.settingsSelectAll = false;
                } else if (/^\d$/.test(e.key)) {
                    const text = R.settingsSelectAll ? e.key : R.pendingText + e.key;
                    if (text.length <= 2) R.editSettings(text);
                    R.settingsSelectAll = false;
                }
            }
        } else if (e.type === 'click') {
            const x = R.settingsX, y = R.settingsY;
            if (!R.inside(e, x, y, R.settingsWidth, R.settingsHeight) || R.inside(e, x + 400, y + 8, 28, 28)) R.settingsOpen = false;
            else if (R.inside(e, x + 220, y + 24, 136, 44)) {
                R.settingsFocus = 0; R.settingsSelectAll = true; R.keypadOpen = true;
            } else if (R.inside(e, x + 24, y + 88, 392, 44)) {
                R.settingsFocus = 1; R.toggleDebug();
            } else if (R.inside(e, x + 24, y + 156, 240, 40)) {
                R.settingsFocus = 3; R.reloadBootList();
            } else if (R.keypadOpen && R.inside(e, x + 448, y + 24, 208, 238)) {
                const col = Math.floor((e.x - x - 448) / 72);
                const row = Math.floor((e.y - y - 24) / 62);
                if (col < 3 && row < 4 && R.inside(e, x + 448 + col * 72, y + 24 + row * 62, 64, 52)) {
                    R.settingsFocus = 0; R.keypadInput(R.keypadKeys[row * 3 + col]);
                }
            } else if (R.inside(e, x + 284, y + 156, 132, 40)) {
                R.settingsFocus = 2; R.saveSettings();
            }
        }
        return true;
    };
    R.updateSettings = () => {
        R.settingsVisual = R.follow(R.settingsVisual, R.settingsOpen ? 1 : 0, 55);
        const p = R.settingsVisual, visible = p > 0.005;
        for (const button of R.keypadButtons) R.show(button, R.settingsOpen && R.keypadOpen);
        R.settingsWidth = R.keypadOpen ? 680 : 448;
        R.settingsHeight = R.keypadOpen ? 286 : 220;
        R.settingsX = R.w - R.settingsWidth - 36;
        R.prop(R.settingsPanel, 'left', R.settingsX);
        R.prop(R.settingsPanel, 'w', R.settingsWidth);
        R.prop(R.settingsPanel, 'h', R.settingsHeight);
        R.prop(R.settingsShadow, 'left', R.settingsX - 3);
        R.prop(R.settingsShadow, 'w', R.settingsWidth + 6);
        R.prop(R.settingsShadow, 'h', R.settingsHeight);
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
        R.label(R.settingsReloadLabel, R.listReloadPending ? 'Reading...' : R.listReloadFailed ? 'Read failed - retry' : 'Reload boot list');
        R.border(R.settingsReload, R.listReloadFailed ? 0xb45a64 : R.settingsFocus === 3 ? C.ink : C.accent);
        R.updateDebugSetting();
    };
})();
