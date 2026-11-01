import assert from 'node:assert/strict';
import { createLogger, } from '../dist/final/neutral/index.mjs';

//region Synchronous producer bursts retain completions by operation rather than record

/** Explicit collection separates retained completion state from discarded log records. */
const collect = globalThis.gc;
if ((typeof collect) !== 'function')
  throw new Error('The logger burst capacity task requires --expose-gc',);
/** Shared completed sink operation remains valid for every delivered record. */
const completed = Promise.resolve();
/** Delivery count proves the optimization cannot discard records. */
const state = { delivered: 0, };
/** Consumer-facing artifact receives an ordinary sink with a shared completion. */
const { logger, initPromise, } = createLogger({
  sinks: [{
    verify: function verify(): Promise<boolean> {
      return Promise.resolve(true,);
    },
    write: function write(): Promise<void> {
      state.delivered += 1;
      return completed;
    },
  },],
},);
await initPromise;
collect();
/** Heap checkpoint precedes a producer frame with no opportunity to drain microtasks. */
const before = process.memoryUsage().heapUsed;
for (let index = 0; index < 100_000; index += 1)
  logger.trace('synchronous producer burst',);
collect();
/** Pending state must not multiply for writes sharing one completion. */
const retainedHeapBytes = process.memoryUsage().heapUsed - before;
await logger.flush();
assert.equal(state.delivered, 100_000,);
assert.ok(retainedHeapBytes < 8_388_608,
  `Shared sink completion retained ${retainedHeapBytes} heap bytes`,);
process.stdout.write(`${JSON.stringify({ records: state.delivered, retainedHeapBytes, },)}\n`,);

//endregion Synchronous producer bursts retain completions by operation rather than record
