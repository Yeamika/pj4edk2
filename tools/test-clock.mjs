import assert from 'node:assert/strict';
import {PreviewClock} from '../viewer/clock.mjs';
const clock=new PreviewClock();
assert.equal(clock.tick(50000),0);
assert.equal(clock.interval,null);
assert.equal(clock.tick(50016),16);
clock.reset();
assert.equal(clock.tick(90000),0);
assert.equal(clock.interval,null); // Do not count downtime as a slow frame.
assert.equal(clock.tick(90016),16);
assert.equal(clock.interval,16);
for(let i=0;i<20;i++){clock.reset();assert.equal(clock.tick(100000+i*1000),0);}
console.log('Preview restart clock PASS');
