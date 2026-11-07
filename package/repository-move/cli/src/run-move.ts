/**
 Entry of the copy tool: reads the options, reconciles the inventory, and runs the pass.

 @module
 */

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

/**
 Runs one pass.

 @param argv - arguments after the program name
 
 @throws {MoveStopError} when a step stops the run; every earlier step is recorded
 */
export async function runMove(argv: readonly string[],): Promise<void> {
  const options = parseMoveOptions(argv,);
  const clock = startDeadlineClock({ totalSeconds: options.totalDeadlineSeconds, },);
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
  if (options.pass === 'thaw') {
    await thaw({
      loadCurrent: async function loadCurrent() {
        return await reconcileInventory({ options, log: runner.log, },);
      },
      runner,
      stateFolder: options.stateFolder,
      run: options.run,
    },);
    return;
  }
  const subvolumes = await reconcileInventory({
    options,
    log: runner.log,
  },);
  if (options.pass === 'freeze') {
    await freeze({
      subvolumes,
      runner,
      stateFolder: options.stateFolder,
      run: options.run,
    },);
  }
  else {
    await runCopyPass({
      options,
      runner,
      subvolumes,
    },);
  }
}
