/**
 The child of `pass-stop-signals.unit.test.ts`: holds a runs directory's lock and a run log as a pass does, installs the
 stop-signal handlers, says it is ready and waits for a signal, which the case sends to this process and to no other.

 Past the shared hang stop it throws, which leaves a code no handler leaves, so a case whose handler never ran fails.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { lockRunsDir, } from '@monochromatic-dev/translation-repair-run';
import {
  installRunLog,
  openRunLog,
} from '@monochromatic-dev/translation-repair-typography';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { handlingStopSignals, } from '../../dist/final/node/index.mjs';

/**
 Runs directory the case made, the first argument.
 */
const [runsDir = '',] = process.argv
  .slice(2,);

/**
 Holds a lock and a run log as a pass does, says it is ready and waits for a stop signal.

 @throws Error when no stop signal reaches the process within the hang stop

 @example
 ```ts
 await holdUntilSignalled();
 ```
 */
async function holdUntilSignalled(): Promise<void> {
  /**
   The lock of the runs directory, as a pass holds it.
   */
  await using lock = await lockRunsDir({ runsDir, },);

  /**
   The run log of the runs directory.
   */
  await using runLog = await openRunLog({ runDir: runsDir, },);

  /**
   The install of that log.
   */
  using _installed = installRunLog({ log: runLog, },);

  /**
   The handlers of the stop signals, which release the lock.
   */
  using _signals = handlingStopSignals({
    log: runLog,
    release: async function releaseLock(): Promise<void> {
      await lock[Symbol.asyncDispose]();
    },
  },);

  console.log('ready',);
  await wait(HANG_STOP_MS,);
  throw new Error('no stop signal reached the child within the hang stop',);
}

await holdUntilSignalled();
