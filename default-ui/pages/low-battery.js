// Static graphics only. The BSP decides the protection threshold and exit policy.
(() => {
    const R = Rimuks;
    R.makeLowBattery = () => {
        R.scene = 'low-battery'; R.selected = null; R.autoActive = false;
        const x = R.w / 2, y = R.h / 2, outline = 0x858b92, red = 0xdf514d;
        const battery = R.box(x - 46, y - 116, 92, 140, null);
        R.border(battery, outline, 3);
        R.box(x - 12, y - 122, 24, 6, outline);
        R.lowBatteryFill = R.box(10, 122, 72, 8, red, battery);
        R.lowBatteryPlug = R.box(x - 14, y + 66, 28, 22, null);
        R.lowBatteryPins = [[5, -11, 3, 11], [20, -11, 3, 11], [12, 22, 3, 17], [12, 36, 17, 3]]
            .map(part => R.box(...part, 0x545b63, R.lowBatteryPlug));
        R.updateLowBattery();
    };
    R.updateLowBattery = () => {
        const percent = R.cfg.battery_percent;
        const fill = Math.max(8, Math.round(120 * (percent === null ? 0 : percent) / 100));
        R.prop(R.lowBatteryFill, 'top', 130 - fill); R.prop(R.lowBatteryFill, 'h', fill);
        const ink = R.cfg.external_power ? 0xadb5be : 0x545b63;
        R.border(R.lowBatteryPlug, ink, 3);
        for (const part of R.lowBatteryPins) R.fill(part, ink);
    };
})();
