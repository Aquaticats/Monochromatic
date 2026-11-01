// Reproduction fixture for performance.logging.md. Run only in a bounded container.
import { performance } from 'node:perf_hooks';
import {
  createLogger,
  sinks,
  tagged
} from '../../package/logger/core/dist/final/node/index.mjs';

const mode = process.argv[2];
if (![
  'noop',
  'suppressed',
  'emitted'
].includes(mode)) {
  throw new Error('Choose noop, suppressed, or emitted');
}
if ((mode === 'emitted') !== (process.env
  .MONOCHROMATIC_VERBOSE
  === 'true')) {
  throw new Error('Only emitted mode requires MONOCHROMATIC_VERBOSE=true');
}

const iterations = mode === 'emitted' ? 200 : 1_400;
const rounds = 10;
const sink = mode === 'noop' ? sinks.createNoopSink() : sinks.createConsoleSink();
const {
  logger,
  initPromise
} = createLogger({ sinks: [sink] });
await initPromise;
let controlChecksum = 0;

async function sample(
  level,
  stackControl = false
) {
  const start = performance.now();
  for (let index = 0; index < iterations; index += 1) {
    const entry = tagged({
      tag: 'parseCss',
      l: logger
    });
    const message = `parsing ${String(index)} characters`;
    if (stackControl) {
      const {stack} = new Error();
      controlChecksum += stack?.length ?? 0;
      entry[level](`${message}${stack?.length ?? 0}`);
    } else {
      entry[level](message);
    }
    // Drain each enabled console entry individually, exposing its stack-capture cost.
    if (mode === 'emitted') await Promise.resolve();
  }
  await logger.flush();
  return ((performance.now() - start) * 1_000) / iterations;
}

// Warm both paths before alternating the measured order to reduce drift bias.
for (let index = 0; index < 3; index += 1) {
  await sample('trace');
  await sample('debug');
}
const results = {
  mode,
  iterations,
  rounds,
  node: process.version,
  traceUs: [],
  debugUs: [],
  stackControlUs: [],
  controlChecksum,
};
for (let index = 0; index < rounds; index += 1) {
  const order = (index % 2) === 0 ? [
    'trace',
    'debug'
  ] : [
    'debug',
    'trace'
  ];
  for (const level of order) {
    results[`${level}Us`]
      .push(Number((await sample(level)).toFixed(3)));
  }
}
if (mode === 'noop') {
  for (let index = 0; index < rounds; index += 1) {
    results.stackControlUs
      .push(Number((await sample(
        'debug',
        true
      )).toFixed(3)));
  }
  results.controlChecksum = controlChecksum;
}
console.log(JSON.stringify(results));
