/**
 Scenario run by `tunnel-lifecycle-watcher-readiness.integration.test.ts` in a container without a
 system manager.
 It starts the real route watcher through `startBypassWatcher` while the watcher's own module load is
 held for a moment, which is what a cold start under load looks like from the call that waits for it,
 and reports whether the call accepted the watcher or killed it before it registered.
 It changes nothing outside the runtime directory the test gives it.

 The watcher asks the system manager for the sentinel's own service,
 so this scenario refuses to run where a system manager can be reached:
 on such a machine it would ask that manager to start a unit.

 @module
 */

import { existsSync, } from 'node:fs';
import { mkdir, writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

/**
 Milliseconds the watcher's module load is held, which stands in for a cold start under load.

 Longer than the count of short sleeps the readiness wait used to be,
 and far shorter than a start that has actually failed.
 */
const HELD_MODULE_LOAD_MILLISECONDS = 1_500;

if (existsSync('/run/systemd/private')) {
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
 * Runtime directory of the CLI, which holds the watcher record this scenario reads.
 */
const runtime = join(scratch, 'run',);
await mkdir(runtime, { recursive: true, },);
process.env.WG_QUICKER_RUNTIME_DIRECTORY = runtime;

/**
 Program that holds a child node's module load without using the processor.
 */
const heldModuleLoad = join(scratch, 'held-module-load.cjs',);
await writeFile(
  heldModuleLoad,
  `Atomics.wait(new Int32Array(new SharedArrayBuffer(4,),), 0, 0, ${String(HELD_MODULE_LOAD_MILLISECONDS,)});\n`,
);
// The watcher and the sentinel are child processes, so they inherit this and start slowly;
// this process was started before the variable was set and reads nothing slowly.
process.env.NODE_OPTIONS = `--require=${heldModuleLoad}`;

const { bypassStatePath, } = await import('../dist/final/node/tunnel-bypass-path.mjs',);
const { startBypassWatcher, stopBypassWatcher, } = await import(
  '../dist/final/node/tunnel-bypass-watch-service.mjs',
);

/**
 State of the interface whose route watcher the scenario starts.
 */
const state = {
  version: 2,
  interfaceName: 'wglready',
  mark: 100,
  table: 52_000,
  preference: 50,
  ownerId: 'watcher-readiness-scenario',
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
 What the start threw, absent when it accepted the watcher.
 */
const failure = await startBypassWatcher({ state, },).then(
  () => undefined,
  (error: unknown) => (error as Error).message,
);
/**
 How long the start took.
 */
const elapsedMilliseconds = Math.round(performance.now() - started,);
if (failure === undefined) {
  await stopBypassWatcher({ state, },);
}
console.log(JSON.stringify({
  registered: failure === undefined,
  elapsedMilliseconds,
  failure,
},),);
