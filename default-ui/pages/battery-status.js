(() => {
    const R = Rimuks, C = R.C;
    R.powerFacts = power => {
        power = power || {};
        return {
            charging: typeof power.charging === 'boolean' ? power.charging : null,
            voltage_mv: Number.isInteger(power.voltage_mv) && power.voltage_mv > 0 && power.voltage_mv <= 0xffffffff ? power.voltage_mv : null,
            current_ma: Number.isInteger(power.current_ma) && power.current_ma >= -2147483648 && power.current_ma <= 2147483647 ? power.current_ma : null,
        };
    };
    R.powerDescription = () => {
        const power = R.cfg.power;
        if (power.charging === true) return 'Charging';
        if (power.current_ma !== null && power.current_ma < 0) return 'Discharging';
        return power.charging === false ? 'Not charging' : 'Power unknown';
    };
    R.powerReadings = () => {
        const power = R.cfg.power;
        const voltage = power.voltage_mv === null ? '-- V' : (power.voltage_mv / 1000).toFixed(3) + ' V';
        const current = power.current_ma === null ? '-- mA' : (power.current_ma > 0 ? '+' : '') + power.current_ma + ' mA';
        return voltage + '  ' + current;
    };
    R.chargingBolt = (x, y, parent) => {
        const bolt = R.box(x, y, 14, 22, null, parent);
        for (let i = 0; i < 6; i++) R.box(8 - i, 1 + i, 4, 2, 0xffe68b, bolt);
        R.box(3, 8, 9, 3, 0xffe68b, bolt);
        for (let i = 0; i < 6; i++) R.box(8 - i, 11 + i, 4, 2, 0xffe68b, bolt);
        return bolt;
    };
    R.makeBatteryStatus = parent => {
        R.chargeX = R.menuX - 384;
        R.powerToggleButton = R.box(R.chargeX, 22, 116, 40, C.headerHover, parent, 8);
        R.chargeLabel = R.text(4, 11, 108, '', 5, C.white, R.powerToggleButton);
        R.prop(R.chargeLabel, 'align', 1);
        R.powerStatus = R.box(R.menuX - 258, 20, 242, 46, null, parent);
        R.powerOutline = R.box(0, 6, 24, 14, null, R.powerStatus);
        R.border(R.powerOutline, C.white);
        R.powerTerminal = R.box(24, 10, 3, 6, C.white, R.powerStatus);
        R.powerFill = R.box(3, 3, 18, 8, C.white, R.powerOutline);
        R.powerPercent = R.text(36, 1, 64, '', 0, C.white, R.powerStatus);
        R.prop(R.powerPercent, 'h', 26);
        R.powerBolt = R.chargingBolt(102, 2, R.powerStatus);
        R.powerStateLabel = R.text(124, 3, 118, '', 5, C.white, R.powerStatus);
        R.powerTelemetry = R.text(0, 28, 242, '', 5, C.white, R.powerStatus);
        R.prop(R.powerStateLabel, 'h', 22); R.prop(R.powerTelemetry, 'h', 18);
    };
    R.updateBatteryStatus = () => {
        if (!R.powerStatus) return;
        R.show(R.powerStatus, R.scene === 'boot' || R.scene === 'recovery');
        R.show(R.powerToggleButton, R.chargeAvailable && (R.scene === 'boot' || R.scene === 'recovery'));
        R.border(R.powerToggleButton, R.chargeFocused ? C.white : C.headerHover);
        R.label(R.chargeLabel, R.chargePending !== null ? 'Working...' : R.chargeFailed ? 'Failed / retry' :
            R.chargeRequested === true && R.chargeEnabled === false ? 'Waiting USB' : R.chargeEnabled === null ? 'Stop charge' : R.chargeEnabled ? 'Charge on' : 'Charge off');
        const value = R.cfg.battery_percent, color = R.cfg.power.charging === true ? 0xb5f4cf : value !== null && value <= 10 ? 0xffb2a7 : C.white;
        R.border(R.powerOutline, color); R.fill(R.powerTerminal, color); R.fill(R.powerFill, color);
        R.show(R.powerFill, value !== null && value > 0);
        R.prop(R.powerFill, 'w', value === null ? 0 : Math.round(18 * value / 100));
        R.label(R.powerPercent, value === null ? '--' : (globalThis.bootBatteryEstimate ? '~' : '') + value + '%'); R.ink(R.powerPercent, color);
        R.show(R.powerBolt, R.cfg.power.charging === true);
        R.label(R.powerStateLabel, R.powerDescription());
        R.label(R.powerTelemetry, R.powerReadings());
    };
    R.requestCharge = () => {
        if (!R.chargeAvailable || R.chargePending !== null) return;
        // Unknown state permits a stop request, never a speculative enable.
        R.chargePending = R.chargeRequested === false;
        R.chargeFailed = false; R.autoActive = false; R.armed = null;
        R.emit({type: 'set_charging_enabled', enabled: R.chargePending});
    };
    R.chargeReply = event => {
        if (event.type !== 'charge_state' && event.type !== 'charge_configured') return false;
        for (const model of [R, ...(globalThis.rimuksStageParents || []).map(parent => parent.model)]) {
            model.chargeEnabled = typeof event.enabled === 'boolean' ? event.enabled : null;
            if (event.type === 'charge_state') { model.chargeAvailable = event.available === true; model.chargeRequested = typeof event.requested === 'boolean' ? event.requested : model.chargeEnabled; }
            else { model.chargePending = null; model.chargeFailed = !event.success; if (event.success) model.chargeRequested = event.requested_enabled; }
            if (!model.chargeAvailable) model.chargeFocused = false;
        }
        return true;
    };
    R.chargeEvent = e => {
        if (R.settingsOpen || !['boot', 'recovery'].includes(R.scene)) return false;
        if (e.type === 'click') {
            R.actionFocus = null; R.chargeFocused = false;
            if (R.chargeAvailable && R.inside(e, R.chargeX, 22, 116, 40)) { R.requestCharge(); return true; }
            return false;
        }
        if (e.type !== 'key' || R.scene !== 'boot') return false;
        const controls = R.chargeAvailable ? ['recovery', 'shutdown', 'charge'] : ['recovery', 'shutdown'];
        if (R.actionFocus) {
            if (e.key === 'Enter') {
                if (R.actionFocus === 'charge') R.requestCharge();
                else if (R.actionFocus === 'recovery') R.openRecovery();
                else R.requestShutdown();
                return true;
            }
            if (['ArrowUp', 'ArrowDown', 'Tab'].includes(e.key)) {
                const index = controls.indexOf(R.actionFocus) + (e.key === 'ArrowUp' ? -1 : 1);
                R.armed = null;
                if (index >= 0 && index < controls.length) R.actionFocus = controls[index];
                else {
                    R.actionFocus = null; R.selected = index < 0 ? R.rows.length - 1 : 0;
                    R.scrollOffset = Math.max(0, R.selected - R.visibleRows + 1);
                }
                R.chargeFocused = R.actionFocus === 'charge'; return true;
            }
        } else if ((e.key === 'ArrowUp' && R.selected === 0) ||
                   (e.key === 'ArrowDown' && R.selected === R.rows.length - 1)) {
            R.actionFocus = e.key === 'ArrowUp' ? controls[controls.length - 1] : controls[0];
            R.chargeFocused = R.actionFocus === 'charge'; R.armed = null; return true;
        }
        return false;
    };
    R.powerReply = event => {
        if (event.type !== 'power_state') return false;
        const value = event.battery_percent;
        if (value !== null && (!Number.isInteger(value) || value < 0 || value > 100)) return true;
        for (const model of [R, ...(globalThis.rimuksStageParents || []).map(parent => parent.model)]) {
            model.cfg.battery_percent = value; model.cfg.external_power = event.external_power;
            model.cfg.power = model.powerFacts(event.power);
        }
        return true;
    };
})();
