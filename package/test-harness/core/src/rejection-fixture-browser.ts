/**
 Node-global absence controls for the runtime-neutral descriptor path.
 @module
 */

import process from 'node:process';
import { logger, } from '@monochromatic-dev/logger-core/ts';
import { it, } from '../dist/final/neutral/index.mjs';

/**
 Supplies a successful body independent of runtime-global probes.

 @example
 await body();
 */
function body(): Promise<void> {
  return Promise.resolve();
}

/**
 Exercises successful execution without a Node-compatible global process object.

 @param scenario - missing global or process shim without a Node version

 @example
 await runBrowserScenario('no-node-global');
 */
export async function runBrowserScenario(scenario: string,): Promise<void> {
  await logger.flush();
  /**
   Imported native process remains usable after replacing its global binding.
   */
  const original = globalThis.process;
  /**
   No observer should be activated by the runtime-neutral branch.
   */
  const before = process.listenerCount('unhandledRejection',);
  /**
   Restore the process global before the child exits or emits assertions.
   */
  using restore = {
    [Symbol.dispose](): void {
      Reflect.set(
        globalThis,
        'process',
        original,
      );
    },
  };
  Reflect.set(
    globalThis,
    'process',
    scenario === 'no-node-global' ? undefined : { versions: {}, },
  );
  await it({
    name: scenario,
    fn: body,
  },);
  console.log(`LISTENER_DELTA=${String(process.listenerCount('unhandledRejection',) - before,)}`,);
}
