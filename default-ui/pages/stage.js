// Same QuickJS realm, separate page state and clipped drawing roots per stage.
globalThis.rimuksStageParents = globalThis.rimuksStageParents || [];
globalThis.rimuksSuspendStage = () => {
    if (rimuksStageParents.length >= 8) throw new Error('stage nesting limit');
    rimuksStageParents.push({model: Rimuks, init: rimuksInit, tick: rimuksTick, state: rimuksState});
};
globalThis.rimuksResumeStage = () => {
    if (!rimuksStageParents.length) throw new Error('no parent stage');
    if (Rimuks.bootImage) ui.freeTexture(Rimuks.bootImage.handle);
    if (Rimuks.screenRoot) ui.destroyNode(Rimuks.screenRoot);
    const parent = rimuksStageParents.pop();
    globalThis.Rimuks = parent.model; globalThis.rimuksInit = parent.init;
    globalThis.rimuksTick = parent.tick; globalThis.rimuksState = parent.state;
    Rimuks.booting = false; Rimuks.emitted = false; Rimuks.animationProgress = 0;
    Rimuks.autoActive = false; Rimuks.armed = null; Rimuks.enterScene();
};
