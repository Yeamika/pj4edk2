// The checkmark changes only after the host saves and reads the settings back.
(() => {
    const R = Rimuks, C = R.C;
    R.resolvedDefault = () => R.cfg.default_entry_id || (R.cfg.entries[0] || {}).id || null;
    R.requestDefault = i => {
        const id = R.cfg.entries[i].id;
        if (R.defaultSaving || R.booting || R.resolvedDefault() === id) return;
        R.defaultSaving = id; R.defaultSaveFailed = false;
        R.emit({type: 'set_default_entry', id});
    };
    R.defaultReply = event => {
        if (!['default_entry_read', 'default_entry_save_failed'].includes(event.type)) return false;
        if (event.requested_id !== R.defaultSaving) return true;
        if (event.type === 'default_entry_read') {
            R.cfg.default_entry_id = event.id;
            R.defaultSaveFailed = false;
        } else R.defaultSaveFailed = true;
        R.defaultSaving = null;
        return true;
    };
    R.defaultControlX = geometry => geometry.left + geometry.width + 20;
    R.updateDefaultControl = (row, i, geometry, visible, alpha) => {
        const checked = R.cfg.entries[i].id === R.resolvedDefault();
        R.show(row.defaultBox, visible && alpha > 0);
        R.prop(row.defaultBox, 'left', R.defaultControlX(geometry));
        R.prop(row.defaultBox, 'top', R.rowY(i) + 8);
        R.opacity(row.defaultBox, alpha);
        R.fill(row.defaultBox, checked ? C.accent : C.card);
        R.border(row.defaultBox, checked ? C.accent : C.border, 2);
        R.show(row.defaultTick, checked);
        R.label(row.defaultLabel, R.defaultSaving === R.cfg.entries[i].id ? 'SAVING' : checked ? 'DEFAULT' : '');
    };
})();
