(() => {
    const R = Rimuks, C = R.C;
    R.makeBootActions = () => {
        R.bootActionsY = R.h - R.L.footer - 50;
        R.bootActions = R.box(R.cardX, R.bootActionsY, R.cardW, 36, null);
        R.recoveryButton = R.iconButton(0, 0, 'recovery', 'Recovery', R.bootActions);
        R.shutdownButton = R.iconButton(152, 0, 'shutdown', 'Shutdown', R.bootActions);
        R.border(R.recoveryButton, C.border); R.border(R.shutdownButton, C.border);
    };
    R.openRecovery = () => {
        if (R.defaultSaving) return;
        R.actionFocus = null; R.chargeFocused = false;
        R.returnMode = R.cfg.mode; R.cfg.mode = 'recovery'; R.armed = null;
        R.label(R.stageLabel, 'recovery'); R.enterScene();
    };
    R.requestShutdown = () => {
        if (!R.poweringOff && !R.fastbootBusy) { R.poweringOff = true; R.emit({type: 'shutdown'}); }
    };
    R.updateBootActionFocus = () => {
        if (R.scene !== 'boot') return;
        R.border(R.recoveryButton, R.actionFocus === 'recovery' ? C.accent : C.border);
        R.border(R.shutdownButton, R.actionFocus === 'shutdown' ? C.accent : C.border);
    };
    R.bootActionEvent = e => {
        if (e.type !== 'click') return false;
        if (R.inside(e, R.cardX, R.bootActionsY, 140, 36)) {
            R.openRecovery();
            return true;
        }
        if (R.inside(e, R.cardX + 152, R.bootActionsY, 140, 36)) {
            R.requestShutdown();
            return true;
        }
        return false;
    };
})();
