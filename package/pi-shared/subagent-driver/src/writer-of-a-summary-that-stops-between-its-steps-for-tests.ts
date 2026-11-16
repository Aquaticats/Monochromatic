#!/usr/bin/env node
/**
 A writer of one run's summary, stopped between two of its steps so a test can kill it there.

 A process on this machine may be killed between any two instructions, so the state such a kill leaves is an ordinary state that a
 later writer has to survive. Reaching a chosen step and stopping there is how a test reaches that state without racing a real write.

 The folder, the step to stop at, and the summary to write as JSON come from the command line. When the writer reaches the step, it
 says so on its standard error and waits: the test reads that line and kills this process. The wait has a bound of its own, so a test
 that fails to kill this writer still ends.

 @module
 */

import { setTimeout as wait, } from 'node:timers/promises';
import { summaryTheStoppingWriterPublishes, } from './summary-the-stopping-writer-publishes-for-tests.ts';
import { writeSummary, } from './write-summary.ts';

/**
 How long this writer waits to be killed, which is no wait a passing test ever reaches.
 */
const WAIT_TO_BE_KILLED_MILLISECONDS = 120_000;

/**
 The words this writer was started with: the folder to write into, the step to stop at, and the marker of its last message.
 */
const [runFolder, stopAt, marker,] = process
  .argv
  .slice(2,);

if (
  (runFolder === undefined)
    || (stopAt === undefined)
    || (marker === undefined)
) {
  console.error('usage: writer-of-a-summary-that-stops-between-its-steps-for-tests <run folder> <step> <marker>',);
  process.exitCode = 2;
}
else {
  await writeSummary({
    runFolder,
    summary: summaryTheStoppingWriterPublishes({ marker, },),
    betweenSteps: async function stopAtTheStep(step,) {
      if (step !== stopAt) {
        return;
      }
      console.error(`stopped-at ${step}`,);
      // Waits to be killed, and ends by itself if the test that started this writer never does.
      await wait(WAIT_TO_BE_KILLED_MILLISECONDS,);
    },
  },);
}
