(() => {
    const R = Rimuks;
    R.enterScene = () => {
        if (R.bootImage) { ui.freeTexture(R.bootImage.handle); R.bootImage = null; }
        if (R.sceneRoot) ui.destroyNode(R.sceneRoot);
        R.sceneRoot = R.box(0, 0, R.w, R.h, null, R.screenRoot); R.prop(R.sceneRoot, 'z', 5);
        R.root = R.sceneRoot;
        if (R.cfg.mode === 'low-battery') R.makeLowBattery();
        else if (R.cfg.mode === 'charging') R.makeCharging();
        else if (R.cfg.mode === 'recovery') R.makeRecovery();
        else R.makeBoot();
        R.root = R.screenRoot;
    };
    R.enterBoot = () => {
        R.emit({type: 'enter_boot'}); R.cfg.mode = R.returnMode || 'loader'; R.autoActive = false; R.armed = null;
        R.label(R.stageLabel, R.cfg.mode === 'uefi-loader' ? 'uefi + load' : R.cfg.mode); R.enterScene();
    };
    globalThis.rimuksInit = (json, placementJSON = 'null') => {
        R.cfg = JSON.parse(json);
        R.cfg.power = R.powerFacts(R.cfg.power);
        const placement = JSON.parse(placementJSON);
        // Phone hosts use a 720-wide portrait canvas; desktop dimensions stay unchanged.
        R.scale = placement ? Math.min(1, R.cfg.viewport[0] / 960, R.cfg.viewport[1] / 540) :
            R.cfg.viewport[1] > R.cfg.viewport[0] ?
            R.cfg.viewport[0] / 720 : Math.min(R.cfg.viewport[0] / 1280, R.cfg.viewport[1] / 720);
        R.viewportOrigin = placement ? placement.position : [0, 0];
        R.w = R.cfg.viewport[0] / R.scale; R.h = R.cfg.viewport[1] / R.scale;
        R.now = 0; R.dt = 16; R.armed = null; R.booting = false; R.emitted = false;
        R.inputKind = 'pointer'; R.pointer = {x: -1, y: -1, down: false};
        R.settingsOpen = false; R.pendingTimeout = R.cfg.timeout_seconds; R.animationProgress = 0;
        R.pendingText = String(R.pendingTimeout); R.settingsSaving = false; R.settingsFailed = false;
        R.hostWaiting = false; R.waiting = false;
        R.chargeAvailable = false; R.chargeEnabled = null; R.chargeRequested = null; R.chargePending = null;
        R.chargeFailed = false; R.chargeFocused = false; R.actionFocus = null;
        R.debugEnabled = false; R.debugPending = null; R.debugFailed = false;
        R.defaultSaving = null; R.defaultSaveFailed = false; R.poweringOff = false;
        R.bootError = null;
        const background = R.cfg.mode === 'low-battery' ? 0x000000 : R.C.background;
        if (!placement) ui.setProp(1, R.P.bg, R.color(background));
        R.screenRoot = R.box(R.viewportOrigin[0], R.viewportOrigin[1], R.w, R.h, background, 1);
        R.prop(R.screenRoot, 'z', placement ? placement.layer : 0);
        R.prop(R.screenRoot, 'overflow', 1);
        R.prop(R.screenRoot, 'originX', -0.5); R.prop(R.screenRoot, 'originY', -0.5);
        R.prop(R.screenRoot, 'scale', R.scale); R.root = R.screenRoot;
        if (R.cfg.mode === 'low-battery') R.enterScene();
        else { R.makeChrome(); R.enterScene(); R.makeSettings(); }
        R.autoActive = R.scene === 'boot' && R.cfg.timeout_seconds > 0 &&
            R.cfg.entries.some(entry => entry.id === R.resolvedDefault());
        R.updateChrome();
    };
    globalThis.rimuksTick = (now, json) => {
        const next = Math.max(R.now, now); R.dt = Math.min(64, next - R.now); R.now = next;
        const events = JSON.parse(json);
        for (const event of events) {
            if (R.powerReply(event)) continue;
            if (R.chargeReply(event)) continue;
            if (R.recoveryReply(event)) continue;
            if (event.type === 'boot_failed') {
                R.autoActive = false;
                R.bootError = 'Startup failed: ' + String(event.status);
                continue;
            }
            if (event.type === 'shutdown_failed') {
                R.poweringOff = false;
                if (R.hint) R.label(R.hint, 'Shutdown failed. You can retry.');
                continue;
            }
            if (R.debugReply(event)) continue;
            if (R.defaultReply(event)) continue;
            if (event.type === 'waiting') { R.hostWaiting = event.active; continue; }
            if (event.type === 'settings_saved' || event.type === 'settings_save_failed') {
                if (R.settingsSaving) {
                    if (event.type === 'settings_saved' && event.seconds === R.settingsSubmitted) {
                        R.cfg.timeout_seconds = event.seconds; R.pendingTimeout = event.seconds;
                        R.pendingText = String(event.seconds); R.settingsFailed = false;
                    } else R.settingsFailed = true;
                    R.settingsSaving = false;
                }
                continue;
            }
            if (R.booting || R.poweringOff) break;
            if (R.scene === 'low-battery') continue;
            if (event.type === 'pointer') {
                if (event.down && !R.pointer.down) R.autoActive = false;
                R.pointer = {x: (event.x - R.viewportOrigin[0]) / R.scale,
                    y: (event.y - R.viewportOrigin[1]) / R.scale, down: event.down};
                R.inputKind = 'pointer'; continue;
            }
            R.autoActive = false; R.inputKind = event.type === 'key' ? 'keyboard' : 'pointer';
            const e = event.type === 'click' ? {type: 'click', x: (event.x - R.viewportOrigin[0]) / R.scale,
                y: (event.y - R.viewportOrigin[1]) / R.scale} : event;
            if ((e.type === 'click' && R.inside(e, R.menuX, 22, 116, 40)) ||
                (e.type === 'key' && e.key === 'F2')) { R.openSettings(); continue; }
            if (R.settingsEvent(e)) continue;
            if (R.chargeEvent(e)) continue;
            if (R.scene === 'boot') R.bootEvent(e);
            else if (R.scene === 'recovery') R.recoveryEvent(e);
            else R.chargingEvent(e);
        }
        if (R.autoActive && R.now >= R.cfg.timeout_seconds * 1000) R.beginBoot();
        if (R.scene === 'boot') { R.updateBoot(); R.updateBootActionFocus(); }
        else if (R.scene === 'charging') R.updateCharging();
        else if (R.scene === 'recovery') R.updateRecovery();
        else if (R.scene === 'low-battery') R.updateLowBattery();
        if (R.scene !== 'low-battery') R.updateSettings();
        R.updateChrome();
        for (const parent of (globalThis.rimuksStageParents || [])) {
            parent.model.now = R.now; parent.model.dt = R.dt; parent.model.updateChrome();
        }
    };
    globalThis.rimuksState = () => JSON.stringify({
        mode: R.cfg.mode, viewport: R.cfg.viewport, viewport_origin: R.viewportOrigin,
        waiting: R.waiting, settings_dirty: R.settingsDirty(), settings_saving: R.settingsSaving,
        battery_percent: R.cfg.battery_percent, external_power: R.cfg.external_power,
        charge_available: R.chargeAvailable, charge_enabled: R.chargeEnabled,
        charge_pending: R.chargePending !== null, charge_failed: R.chargeFailed, charge_focused: R.chargeFocused,
        power: R.cfg.power, power_status: R.powerDescription(), power_readings: R.powerReadings(),
        debug_enabled: R.debugEnabled, debug_pending: R.debugPending !== null, debug_failed: R.debugFailed,
        default_entry_id: R.resolvedDefault(), default_saving: !!R.defaultSaving,
        default_save_failed: R.defaultSaveFailed,
        recovery_pending: R.recoveryPending || null,
        recovery_result: !!R.recoveryResult,
        recovery_success: R.recoveryResult ? R.recoveryResult.success : null,
        recovery_page: R.recoveryPage || 0,
        scene: R.scene, selected: R.selected, armed: R.armed, auto_active: R.autoActive,
        settings_open: R.settingsOpen, timeout_seconds: R.cfg.timeout_seconds,
        booting: R.booting, animation_progress: R.animationProgress, emitted: R.emitted,
    });
})();
