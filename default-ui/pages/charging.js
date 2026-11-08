(() => {
    const R = Rimuks, C = R.C;
    R.makeCharging = () => {
        R.scene = 'charging'; R.selected = null; R.chargeHot = 0;
        const x = R.w / 2;
        const eyebrow = R.caps(x - 250, 123, 500, 'POWER'); R.prop(eyebrow, 'align', 1);
        R.chargeTitle = R.text(x - 250, 148, 500, R.powerDescription(), 3);
        R.prop(R.chargeTitle, 'align', 1);
        const subtitle = R.text(x - 250, 199, 500, 'Ready for your next start.', 0, C.muted); R.prop(subtitle, 'align', 1);
        const card = R.box(x - 240, 264, 480, 186, C.selected, R.root, 14); R.border(card, C.softBorder);
        const battery = R.box(36, 54, 156, 78, null, card, 10); R.border(battery, C.accent, 2);
        R.box(192, 79, 7, 28, C.accent, card, 2);
        const percent = R.cfg.battery_percent;
        R.batteryFill = R.box(44, 62, percent === null ? 0 : 140 * percent / 100, 62, C.accent, card, 5);
        R.chargeValue = R.text(232, 41, 220, percent === null ? '--' : percent + '%', 4, C.ink, card);
        R.chargeHint = R.text(235, 116, 210, percent === null ? 'Battery level unavailable' : 'Battery level', 0, C.muted, card);
        R.chargeBolt = R.chargingBolt(208, 48, card);
        R.chargeReadings = R.text(36, 153, 408, R.powerReadings(), 0, C.muted, card);
        R.prop(R.chargeTitle, 'h', 51); R.prop(R.chargeReadings, 'h', 26);
        R.chargeStatus = R.text(x - 260, 478, 520,
            R.cfg.external_power === null ? 'Waiting for power information' : R.cfg.external_power ? 'External power connected' : 'Connect a charger to continue charging', 0, C.muted);
        R.prop(R.chargeStatus, 'align', 1);
        R.chargeButton = R.button(x - 124, 538, 248, 44, 'Continue to boot', R.root, true);
    };
    R.chargingEvent = e => {
        if ((e.type === 'click' && R.inside(e, R.w / 2 - 124, 538, 248, 44)) ||
            (e.type === 'key' && e.key === 'Enter')) R.enterBoot();
    };
    R.updateCharging = () => {
        const percent = R.cfg.battery_percent;
        R.prop(R.batteryFill, 'w', percent === null ? 0 : 140 * percent / 100);
        R.label(R.chargeValue, percent === null ? '--' : percent + '%');
        R.label(R.chargeHint, percent === null ? 'Battery level unavailable' : 'Battery level');
        R.label(R.chargeTitle, R.powerDescription());
        R.show(R.chargeBolt, R.cfg.power.charging === true);
        R.label(R.chargeReadings, R.powerReadings());
        R.label(R.chargeStatus, R.cfg.external_power === null ? 'Waiting for power information' : R.cfg.external_power ? 'External power connected' : 'Connect a charger to continue charging');
        R.opacity(R.batteryFill, R.cfg.power.charging === true ? 0.9 + Math.sin(R.now / 1100) * 0.1 : 1);
        const hover = !R.settingsOpen && R.inside(R.pointer, R.w / 2 - 124, 538, 248, 44);
        R.chargeHot = R.follow(R.chargeHot, hover ? 1 : 0);
        R.fill(R.chargeButton, R.mix(C.accent, C.ink, R.chargeHot * (R.pointer.down ? 0.5 : 0.22)));
    };
})();
