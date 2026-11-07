/**
 Entry of the copy tool: reads the options, reconciles the inventory, and runs the pass.

 @module
 */

import { canonicalOptions, } from './canonical-path.ts';
import { createRunner, } from './command-runner.ts';
import { runCopyPass, } from './copy-pass.ts';
import { startDeadlineClock, } from './deadline-clock.ts';
import { runExternalPass, } from './external-pass.ts';
import {
  freeze,
  thaw,
} from './freeze-pass.ts';
import { reconcileInventory, } from './inventory.ts';
import { parseMoveOptions, } from './move-options.ts';
import { runVerifyPass, } from './verify-pass.ts';

/**
 Runs one pass.

 @param argv - arguments after the program name
 
 @throws MoveStopError when a step stops the run; every earlier step is recorded

 @example
 ```ts
 import { runMove, } from '\@monochromatic-dev/repository-move-cli';
 
 await runMove(argv,);
 ```
 */
export async function runMove(argv: readonly string[],): Promise<void> {
  /**
   The options of this invocation.
   */
  const givenOptions = parseMoveOptions(argv,);
  /**
   The options with the folders the run compares in their canonical spelling: a folder reached through a link and the
   same folder as Git records it are one subvolume, so the records and the placement use the one spelling.
   */
  const options = await canonicalOptions(givenOptions,);
  /**
   The clock of this step.
   */
  const clock = startDeadlineClock({ totalSeconds: options.totalDeadlineSeconds, },);
  /**
   The runner of this step.
   */
  const runner = createRunner({
    options,
    clock,
  },);
  if ((options.pass === 'external-first') || (options.pass === 'external-final')) {
    await runExternalPass({
      options,
      runner,
    },);
    return;
  }
  if ((options.pass === 'verify-first') || (options.pass === 'verify-final')) {
    await runVerifyPass({
      options,
      runner,
    },);
    return;
  }
  if (options.pass === 'thaw') {
    await thaw({
      loadCurrent: async function loadCurrent() {
        return await reconcileInventory({
          options: givenOptions,
          log: runner.log,
        },);
      },
      runner,
      stateFolder: options.stateFolder,
      run: options.run,
    },);
    return;
  }
  /**
   The subvolumes this step covers.
   */
  const subvolumes = await reconcileInventory({
    options: givenOptions,
    log: runner.log,
  },);
  await (options.pass === 'freeze'
    ? freeze({
      subvolumes,
      runner,
      stateFolder: options.stateFolder,
      run: options.run,
    },)
    : runCopyPass({
      options,
      runner,
      subvolumes,
    },));
}
