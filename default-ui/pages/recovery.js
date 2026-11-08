(() => {
    const R = Rimuks, C = R.C;
    R.makeRecovery = () => {
        R.scene = 'recovery'; R.selected = 0; R.recoveryX = (R.w - 600) / 2;
        R.caps(R.recoveryX + 1, 123, 600, 'MAINTENANCE');
        R.text(R.recoveryX, 148, 600, 'Recovery', 3);
        R.recoveryHint = R.text(R.recoveryX + 1, 199, 600, 'Select a tool. Confirm with a second press.', 0, C.muted);
        R.recoveryPending = null; R.recoveryResult = null; R.recoveryPage = 0;
        R.recoveryItems = [
            ['diagnostics', 'Diagnostics', 'Review device and firmware information'],
            ['boot-log', 'Boot log', 'Inspect the most recent startup messages'],
            ['repair', 'Restore startup settings', 'Disable auto-start and restore the default entry'],
            ['continue', 'Return to boot', 'Go back to the startup list'],
        ];
        R.recoveryCards = R.recoveryItems.map((item, i) => {
            const card = R.box(R.recoveryX, 248 + i * 88, 600, 72, C.card, R.root, 10);
            R.box(20, 20, 32, 32, C.hover, card, 6);
            const index = R.text(20, 26, 32, String(i + 1).padStart(2, '0'), 5, C.accent, card);
            R.prop(index, 'align', 1);
            R.text(68, 11, 430, item[1], 6, C.ink, card);
            R.text(69, 42, 470, item[2], 0, C.muted, card);
            const badge = R.text(512, 17, 66, 'OPEN  >', 5, C.accent, card); R.prop(badge, 'align', 2);
            return {card, badge, hot: 0, active: i === 0 ? 1 : 0};
        });
        R.recoveryPanel = R.box(R.recoveryX, 248, 600, 350, C.card, R.root, 10);
        R.recoveryLines = Array.from({length: 10}, (_, i) => R.text(18, 16 + i * 25, 564, '', 0, C.ink, R.recoveryPanel));
        R.text(20, 306, 130, '< Previous', 5, C.accent, R.recoveryPanel);
        R.recoveryPageLabel = R.text(176, 306, 248, '', 5, C.muted, R.recoveryPanel);
        R.prop(R.recoveryPageLabel, 'align', 1);
        R.text(456, 306, 124, 'Next >', 5, C.accent, R.recoveryPanel);
        R.recoveryClose = R.text(R.recoveryX, 619, 600, 'Close result - Enter / Escape', 5, C.accent);
        R.prop(R.recoveryClose, 'align', 1);
        R.updateRecovery();
    };
    R.recoveryReply = e => {
        if (e.type !== 'recovery_result') return false;
        if (R.scene !== 'recovery' || e.operation !== R.recoveryPending) return true;
        R.recoveryPending = null; R.recoveryPage = 0;
        const lines = [];
        for (const line of e.lines || []) {
            const text = String(line).replace(/[\r\n\t]/g, ' ');
            for (let at = 0; at < Math.max(1, text.length) && lines.length < 256; at += 74)
                lines.push(text.slice(at, at + 74));
        }
        R.recoveryResult = {success: !!e.success, lines};
        if (e.success && e.settings) {
            R.cfg.timeout_seconds = e.settings.timeout_seconds;
            R.cfg.default_entry_id = e.settings.default_entry_id;
            R.pendingTimeout = e.settings.timeout_seconds; R.pendingText = String(R.pendingTimeout);
            R.autoActive = false;
        }
        return true;
    };
    R.recoveryEvent = e => {
        if (R.recoveryPending) return;
        if (R.recoveryResult) {
            const pages = Math.max(1, Math.ceil(R.recoveryResult.lines.length / 10));
            if ((e.type === 'key' && ['Enter', 'Escape'].includes(e.key)) ||
                (e.type === 'click' && R.inside(e, R.recoveryX, 610, 600, 38))) R.recoveryResult = null;
            else if ((e.type === 'key' && e.key === 'ArrowUp') ||
                (e.type === 'click' && R.inside(e, R.recoveryX, 542, 160, 48))) R.recoveryPage = Math.max(0, R.recoveryPage - 1);
            else if ((e.type === 'key' && e.key === 'ArrowDown') ||
                (e.type === 'click' && R.inside(e, R.recoveryX + 440, 542, 160, 48))) R.recoveryPage = Math.min(pages - 1, R.recoveryPage + 1);
            return;
        }
        let i = -1;
        if (e.type === 'click') {
            for (let n = 0; n < 4; n++) if (R.inside(e, R.recoveryX, 248 + n * 88, 600, 72)) i = n;
        } else if (e.type === 'key') {
            if (e.key === 'ArrowDown' || e.key === 'Tab') { R.selected = (R.selected + 1) % 4; R.armed = null; }
            if (e.key === 'ArrowUp') { R.selected = (R.selected + 3) % 4; R.armed = null; }
            if (e.key === 'Enter') i = R.selected;
        }
        if (i >= 0) {
            if (R.armed === i) {
                if (i === 3) R.enterBoot();
                else {
                    R.recoveryPending = R.recoveryItems[i][0];
                    R.emit({type: 'recovery', operation: R.recoveryPending}); R.armed = null;
                }
            } else { R.selected = i; R.armed = i; }
        }
    };
    R.updateRecovery = () => {
        const result = R.recoveryResult;
        R.show(R.recoveryPanel, !!result); R.show(R.recoveryClose, !!result);
        R.label(R.recoveryHint, R.recoveryPending ? 'Working...' : result ?
            (result.success ? 'Result - Use Up / Down to change page.' : 'Operation failed - Details below.') :
            'Select a tool. Confirm with a second press.');
        if (result) {
            R.recoveryLines.forEach((node, i) => R.label(node, result.lines[R.recoveryPage * 10 + i] || ''));
            R.label(R.recoveryPageLabel, (R.recoveryPage + 1) + ' / ' + Math.max(1, Math.ceil(result.lines.length / 10)));
        }
        R.recoveryCards.forEach((row, i) => {
        R.show(row.card, !result);
        const hot = !R.settingsOpen && R.inside(R.pointer, R.recoveryX, 248 + i * 88, 600, 72);
        row.hot = R.follow(row.hot, hot ? 1 : 0); row.active = R.follow(row.active, i === R.selected ? 1 : 0);
        let fill = R.mix(R.mix(C.card, C.hover, row.hot), C.selected, row.active);
        if (hot && R.pointer.down) fill = R.mix(fill, C.border, 0.22);
        R.fill(row.card, fill); R.border(row.card, R.mix(C.softBorder, C.accent, row.active * 0.72));
        R.show(row.badge, R.armed === i);
        });
    };
})();
