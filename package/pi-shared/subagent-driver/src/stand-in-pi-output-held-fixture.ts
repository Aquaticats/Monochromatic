/**
 The parts of the stand-in for Pi that keep its output open and leave it late: the tool that holds the output of a process that has
 gone, and the signals of a Pi that ignores SIGTERM and then leaves by itself with a success status.

 A test needs both to make an exit reach the driver only after the ending's own bounds are spent. A process killed by a signal is
 reported with the signal and never with a status, so no stand-in a signal ends can show what a run records when Pi leaves by
 itself after the driver gave up waiting.

 @module
 */

import { spawn, } from 'node:child_process';
import { appendFile, } from 'node:fs/promises';
import process, { exit, } from 'node:process';

/**
 Seconds the tool lives, so a failed test cannot leave it running.
 */
const TOOL_SECONDS = '30';

/**
 Milliseconds after the SIGTERM it ignores at which this stand-in leaves by itself: far enough before the last stage's signal that
 none is sent at all, and late enough that a connection whose exit deadline outlasts the ending's stages hears of the exit only
 after them.
 */
const LEAVES_AFTER_SIGTERM_MILLISECONDS = 4_000;

/**
 The scenario in which Pi keeps its output open through a tool it started, ignores its input closing, ignores SIGTERM, and leaves
 by itself with a success status.
 */
export const LEAVES_AFTER_SIGTERM_SCENARIO = 'leaves-after-sigterm-with-output-held';

/**
 Starts a tool that holds this process's output open after this process has gone, and logs its process number.

 @param log - file the stand-in logs the tool's process number to

 @example
 ```ts
 await holdThisOutputOpen({ log: '/tmp/a/commands.log' },);
 ```
 */
export async function holdThisOutputOpen({ log, }: { readonly log: string },): Promise<void> {
  /**
   The tool, which inherits this process's descriptors and so keeps them open after this process ends.
   */
  const tool = spawn(
    'sleep',
    [TOOL_SECONDS,],
    {
      stdio: [
        'ignore',
        'inherit',
        'inherit',
      ],
      detached: true,
    },
  );
  tool.unref();
  await appendFile(
    log,
    `tool ${String(tool.pid,)}\n`,
  );
}

/**
 Ignores SIGTERM and leaves by itself with a success status a little after it, which is the ending no process a signal kills can
 play: the signal changes nothing, and the status says the process left of its own accord.

 @param scenario - what the stand-in was asked to play, which decides whether these signals are armed

 @example
 ```ts
 armALeavingThatOutlastsTheSignal({ scenario: 'leaves-after-sigterm-with-output-held' },);
 ```
 */
export function armALeavingThatOutlastsTheSignal({ scenario, }: { readonly scenario: string },): void {
  if (scenario !== LEAVES_AFTER_SIGTERM_SCENARIO) {
    return;
  }
  process.on(
    'SIGTERM',
    function leaveAfterTheSignal() {
      setTimeout(
        function leaveByItself() {
          // A success status, which a process the signal ended never leaves with.
          exit(0,);
        },
        LEAVES_AFTER_SIGTERM_MILLISECONDS,
      );
    },
  );
}
