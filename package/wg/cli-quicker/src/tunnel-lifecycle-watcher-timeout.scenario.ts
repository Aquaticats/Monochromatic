/**
 Scenario run by `tunnel-lifecycle-watcher-readiness.integration.test.ts` for a watcher that never registers.
 It starts the real route watcher through `startBypassWatcher` while the watcher's own module load is held longer
 than the readiness deadline, and reports how the start refused and what it left behind.
 It changes nothing outside the runtime directory the test gives it.

 The watcher asks the system manager for the sentinel's own service when one is reachable,
 so this scenario refuses to run where a system manager can be reached.

 @module
 */

import {
  access,
  mkdir,
  readdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

/**
 Milliseconds the watcher's module load is held, which is longer than the readiness deadline.

 The deadline is ten seconds; a watcher that has not registered by then has failed,
 and the start must refuse instead of waiting for it.
 */
const HELD_MODULE_LOAD_MILLISECONDS = 30_000;

/**
 Reports whether a system manager can be reached, which this scenario must not ask for a unit.

 @returns Whether the system manager's own socket is there.
 */
async function systemManagerIsReachable(): Promise<boolean> {
  try {
    await access('/run/systemd/private',);
    return true;
  }
  catch (error) {
    // No socket means no manager to ask, which is what this scenario needs.
    void error;
    return false;
  }
}

if (await systemManagerIsReachable()) {
  throw new Error(
    'This scenario asks the system manager for the sentinel service when one is reachable; run it in a container without a system manager.',
  );
}

/**
 Directory the test gives the scenario for its runtime state.
 */
const scratch = process.argv[2] ?? '';
if (scratch === '')
  throw new Error('The scenario needs its scratch directory as the first argument.',);

/**
 Runtime directory of the CLI, which holds the watcher record this scenario reads.
 */
const runtime = join(
  scratch,
  'run',
);
await mkdir(
  runtime,
  { recursive: true, },
);
process.env
  .WG_QUICKER_RUNTIME_DIRECTORY = runtime;

/**
 Program that holds a child node's module load without using the processor.
 */
const heldModuleLoad = join(
  scratch,
  'held-module-load.cjs',
);
await writeFile(
  heldModuleLoad,
  `Atomics.wait(new Int32Array(new SharedArrayBuffer(4,),), 0, 0, ${String(HELD_MODULE_LOAD_MILLISECONDS,)});\n`,
);
process.env
  .NODE_OPTIONS = `--require=${heldModuleLoad}`;

/**
 Path the CLI keeps the watcher's record under.
 */
const { bypassStatePath, } = await import('../dist/final/node/tunnel-bypass-path.mjs',);
/**
 The start this scenario drives.
 */
const { startBypassWatcher, } = await import('../dist/final/node/tunnel-bypass-watch-service.mjs',);

/**
 State of the interface whose route watcher never registers.
 */
const state = {
  version: 2,
  interfaceName: 'wgltimeout',
  mark: 100,
  table: 52_000,
  preference: 50,
  ownerId: 'watcher-timeout-scenario',
  routes: [],
} as const;
await writeFile(
  bypassStatePath({ interfaceName: state.interfaceName, },),
  JSON.stringify(state,),
);

/**
 When the start was asked for, in milliseconds.
 */
const started = performance.now();
/**
 What the start answered, with an empty message when it accepted the watcher.
 */
const outcome = { failure: '', };
try {
  await startBypassWatcher({ state, },);
}
catch (error) {
  outcome.failure = String(error,);
}
/**
 How long the start took.
 */
const elapsedMilliseconds = Math.round(performance.now() - started,);
/**
 What the runtime directory holds afterwards, which is a record when the program could not prove the watcher gone.
 */
const left = await readdir(runtime,);
console.log(JSON.stringify({
  accepted: outcome.failure === '',
  elapsedMilliseconds,
  failure: outcome.failure,
  recordLeft: left.some(function isRecord(name,): boolean {
    return name.endsWith('.watcher.json',);
  },),
},),);
