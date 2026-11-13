// A fresh preview starts on its first animation frame, not page navigation.
export class PreviewClock {
    constructor() { this.reset(); }
    reset() { this.origin=null; this.previous=null; this.interval=null; }
    tick(timestamp) {
        if(this.origin===null) this.origin=timestamp;
        this.interval=this.previous===null ? null : Math.max(0,timestamp-this.previous);
        this.previous=timestamp;
        return Math.max(0,timestamp-this.origin);
    }
}
