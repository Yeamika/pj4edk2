// A session-only choice. Saved settings and boot parameters never enable debug.
(() => {
    const R = Rimuks, C = R.C;
    R.makeDebugSetting = panel => {
        R.debugButton = R.box(24, 88, 392, 44, C.footer, panel, 6);
        R.text(14, 10, 264, 'PocketJSDebug', 1, C.ink, R.debugButton);
        R.debugBadge = R.box(308, 8, 70, 28, C.card, R.debugButton, 4);
        R.debugLabel = R.text(0, 4, 70, 'OFF', 0, C.muted, R.debugBadge);
        R.prop(R.debugLabel, 'align', 1);
    };
    R.toggleDebug = () => {
        if (R.debugPending !== null) return;
        R.debugFailed = false; R.debugPending = !R.debugEnabled;
        R.emit({type: 'set_debug_enabled', enabled: R.debugPending});
    };
    R.debugReply = event => {
        if (event.type === 'debug_state') {
            R.debugEnabled = event.enabled;
            for (const parent of (globalThis.rimuksStageParents || []))
                parent.model.debugEnabled = event.enabled;
            return true;
        }
        if (event.type !== 'debug_configured') return false;
        for (const model of [R, ...(globalThis.rimuksStageParents || []).map(parent => parent.model)]) {
            model.debugEnabled = event.enabled;
            if (model.debugPending === event.requested_enabled) {
                model.debugPending = null; model.debugFailed = !event.success;
            }
        }
        return true;
    };
    R.updateDebugSetting = () => {
        const pending = R.debugPending !== null, focused = R.settingsFocus === 1;
        const hot = R.inside(R.pointer, R.settingsX + 24, R.settingsY + 88, 392, 44);
        R.fill(R.debugButton, hot && !pending ? C.hover : C.footer);
        R.border(R.debugButton, R.debugFailed ? 0xb45a64 : focused ? C.accent : C.softBorder,
            focused ? 2 : 1);
        R.fill(R.debugBadge, R.debugEnabled ? C.accent : C.card);
        R.ink(R.debugLabel, R.debugEnabled ? C.white : C.muted);
        R.label(R.debugLabel, pending ? '...' : R.debugEnabled ? 'ON' : 'OFF');
        R.opacity(R.debugButton, pending ? 0.65 : 1);
    };
})();
