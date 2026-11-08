(() => {
    const R = Rimuks, C = R.C;
    R.makeChrome = () => {
        const top = R.box(0, 0, R.w, R.L.top, C.header); R.prop(top, 'z', 30);
        R.box(0, R.L.top - 1, R.w, 1, C.headerEdge, top);
        R.loaderCells = [];
        for (let i = 0; i < 9; i++) {
            R.loaderCells.push(R.box(36 + (i % 3) * 12, 26 + Math.floor(i / 3) * 12,
                8, 8, C.white, top, 1));
        }
        const brand = R.text(88, 17, 220, 'RIMUKS', 2, C.white, top);
        R.prop(brand, 'tracking', 1.4);
        R.caps(89, 49, 220, 'BOOT ENVIRONMENT', 0xd0e6f3, top);
        const stage = R.box(R.w / 2 - 84, 24, 168, 36, C.headerHover, top, 4);
        R.stageLabel = R.text(12, 6, 144, R.cfg.mode === 'uefi-loader' ? 'uefi + load' : R.cfg.mode,
            7, C.white, stage); R.prop(R.stageLabel, 'align', 1);
        R.show(stage, R.w >= 1000);
        R.menuX = R.w - 152;
        R.menuButton = R.box(R.menuX, 22, 116, 40, C.header, top, 8); R.menuHot = 0;
        R.text(12, 9, 70, 'Settings', 0, C.white, R.menuButton);
        for (let i = 0; i < 3; i++) R.box(84, 13 + i * 6, 18, 2, C.white, R.menuButton);
        R.makeBatteryStatus(top);
        R.footer = R.box(0, R.h - R.L.footer, R.w, R.L.footer, C.footer);
        R.prop(R.footer, 'z', 3); R.box(0, 0, R.w, 1, 0xd4e8f4, R.footer);
        R.text(36, 22, 140, R.ellipsize('v' + R.cfg.version, 140, 0), 0, C.muted, R.footer);
        R.footerExtras = R.box(0, 0, R.w, R.L.footer, null, R.footer);
        R.box(188, 20, 1, 24, 0xcbdfea, R.footerExtras);
        R.countLabel = R.text(220, 20, 165, '', 1, C.ink, R.footerExtras);
        R.countTrack = R.box(390, 30, R.w - 426, 4, 0xd5e7f1, R.footerExtras, 2);
        R.countFill = R.box(0, 0, R.w - 426, 4, C.accent, R.countTrack, 2);
        R.manualHelp = R.box(220, 0, R.w - 256, R.L.footer, null, R.footerExtras);
        R.helpLabel = R.text(0, 22, 390, 'Automatic start paused', 0, C.muted, R.manualHelp);
        const keysX = R.w - 642;
        R.navigateKeys = R.box(keysX, 0, 174, 64, null, R.manualHelp);
        R.keycap(0, 20, 38, 'Up', R.navigateKeys);
        R.keycap(44, 20, 48, 'Down', R.navigateKeys);
        R.text(102, 23, 72, 'Choose', 5, C.muted, R.navigateKeys);
        R.confirmKeys = R.box(keysX + 186, 0, 200, 64, null, R.manualHelp);
        R.keycap(0, 20, 50, 'Enter', R.confirmKeys);
        R.helpAction = R.text(62, 23, 120, 'Select', 5, C.muted, R.confirmKeys);
    };
    R.updateChrome = () => {
        if (R.scene === 'low-battery') { R.waiting = false; return; }
        R.updateBatteryStatus();
        const waiting = !!(R.hostWaiting || R.settingsSaving || R.debugPending !== null || R.defaultSaving || R.poweringOff || R.autoActive || (R.booting && !R.emitted));
        R.waiting = waiting;
        const ring = [0, 1, 2, 5, 8, 7, 6, 3], phase = (R.now / 1200 * 8) % 8;
        R.loaderCells.forEach((cell, i) => {
            const distance = (phase - ring.indexOf(i) + 8) % 8;
            const alpha = waiting ? (i === 4 ? 0.16 : 0.14 + 0.86 * Math.pow(1 - distance / 8, 2)) :
                (i === 4 ? (Math.floor(R.now / 500) % 2 === 0 ? 1 : 0.16) : 1);
            R.opacity(cell, alpha);
        });
        const hot = !R.booting && (R.settingsOpen || R.inside(R.pointer, R.menuX, 22, 116, 40));
        R.menuHot = R.follow(R.menuHot, hot ? 1 : 0);
        R.fill(R.menuButton, R.mix(C.header, C.headerHover, R.menuHot));
        R.opacity(R.menuButton, R.booting ? 1 - 0.55 * R.ease((R.now - R.bootAt) / 160) : 1);
        const active = R.autoActive && !R.booting && !R.settingsOpen;
        R.show(R.countLabel, active); R.show(R.countTrack, active); R.show(R.manualHelp, !active);
        R.opacity(R.footerExtras, R.booting ? 1 - R.ease((R.now - R.bootAt) / 160) : 1);
        const label = R.bootError || (R.settingsOpen ? 'Boot delay applies on the next start' : R.scene === 'boot' ?
            (R.cfg.timeout_seconds === 0 ? 'Manual startup' : 'Automatic start paused') :
            (R.scene === 'recovery' ? 'Recovery tools' : 'Power status'));
        R.label(R.helpLabel, label);
        const hasEntries = R.scene !== 'boot' || R.rows.length > 0;
        R.show(R.navigateKeys, R.w >= 1000 && !R.settingsOpen && R.scene !== 'charging' && hasEntries);
        R.show(R.confirmKeys, R.w >= 1000 && (R.settingsOpen || hasEntries));
        R.label(R.helpAction, R.settingsOpen ? (R.settingsFocus === 1 ? 'Toggle' : 'Save') : R.scene === 'charging' ? 'Continue' :
            R.armed === null ? 'Select' : R.scene === 'recovery' ? 'Open' : 'Start');
        if (active) {
            const remaining = Math.max(0, R.cfg.timeout_seconds - R.now / 1000);
            R.label(R.countLabel, 'Boot in ' + Math.ceil(remaining) + ' s');
            R.prop(R.countFill, 'w', (R.w - 426) * remaining / R.cfg.timeout_seconds);
        }
    };
})();
