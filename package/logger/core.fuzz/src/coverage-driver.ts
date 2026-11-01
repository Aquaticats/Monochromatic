/**
 Deterministic coverage driver for the module-logger fuzz coverage gate.

 Imports the runtime package's source through its `/ts` subpath, so V8
 coverage attributes to `package/logger/core/src`, and replays fixed
 scenarios through every orchestration branch and every Node-reachable
 sink branch. Run under `NODE_V8_COVERAGE` (and `--localstorage-file`, so
 the localStorage sink elects) by the `fuzz:coverage` task, then
 summarized by `coverage-report.ts`.

 A reachability harness, not an oracle: it asserts nothing, because the
 property suite owns correctness. Console output, breadcrumbs included, is
 silenced for the run so the gate's log stays readable; every breadcrumb
 this driver provokes is deliberate.

 @module
 */

import { exerciseConsoleSink, } from './coverage-console.ts';
import { exerciseKeysAndQuota, } from './coverage-keys-quota.ts';
import { exerciseNodeSinks, } from './coverage-node-sinks.ts';
import { exerciseOrchestration, } from './coverage-orchestration.ts';
import { exerciseWebStorageSinks, } from './coverage-web-storage.ts';

/**
 Console methods the sinks and breadcrumbs can call.
 */
const CONSOLE_METHODS = [
  'debug',
  'error',
  'info',
  'trace',
  'warn',
] as const;

/**
 Stands in for every console method during the run.
 */
function ignoreOutput(): void {
  // The gate reads coverage, not output.
}

/**
 Replaces every console method with a no-op for the run; disposing restores
 them.

 @returns Disposable restoring the original methods.

 @example
 ```ts
 using _quiet = silenceConsole();
 ```
 */
function silenceConsole(): Disposable {
  /**
   Original methods by name.
   */
  const originals = new Map<string, unknown>();
  for (const method of CONSOLE_METHODS) {
    originals.set(
      method,
      console[method],
    );
    console[method] = ignoreOutput;
  }
  return {
    [Symbol.dispose](): void {
      for (const [method, original,] of originals)
        Object.defineProperty(
          console,
          method,
          {
            configurable: true,
            value: original,
            writable: true,
          },
        );
    },
  };
}

/**
 Console silence held for the whole run.
 */
using _quiet = silenceConsole();
await exerciseOrchestration();
await exerciseConsoleSink();
await exerciseNodeSinks();
await exerciseWebStorageSinks();
exerciseKeysAndQuota();
