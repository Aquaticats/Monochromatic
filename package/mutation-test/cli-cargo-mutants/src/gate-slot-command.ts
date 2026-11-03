/**
 Run the gated command at the lowest priority, with the caller's standard streams, and report how it ended as an exit status.

 The gate process stays alive for as long as the command runs, because its claim names this process: a termination signal
 sent to the gate is passed on to the command, and the gate ends only after the command has, so the slot is not given back
 while the command still runs. (A gate killed by `SIGKILL` cannot pass anything on; its claim then reads as dead and the slot
 is free again while the command may still run. A `systemctl stop` of the unit that runs the gate ends the command too.)
 */
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { constants } from 'node:os';

import {
  niceProgram,
  slotNiceAdjustment,
} from './constants.ts';

/**
 Exit status a shell reports for a command that cannot be found.
 */
const commandNotFoundStatus = 127;

/**
 Offset a shell adds to a signal number to report a command a signal ended.
 */
const signalStatusBase = 128;

/**
 Signals the gate passes on to the command.
 */
const forwardedSignals: readonly NodeJS.Signals[] = [
  'SIGINT',
  'SIGTERM',
  'SIGHUP',
];

/**
 Pass the gate's termination signals on to a child until the returned value is disposed.

 @param child - the command's process

 @returns the forwarding; declare it with `using` so the handlers are removed when the command ends

 @example
 ```ts
 using forwarding = forwardSignals(child);
 ```
 */
function forwardSignals(child: ReturnType<typeof spawn>): Disposable {
  /**
   One handler per signal, kept to be removed.
   */
  const handlers = forwardedSignals.map(function handlerFor(signal) {
    /**
     Pass the signal on.
     */
    function forward(): void {
      child.kill(signal);
    }
    process.on(
      signal,
      forward,
    );
    return {
      signal,
      forward,
    };
  });
  return {
    [Symbol.dispose](): void {
      for (const {
        signal,
        forward
      } of handlers)
        process.off(
          signal,
          forward,
        );
    },
  };
}

/**
 Run a command niced to the lowest priority and wait for it.

 @param command - the command and its arguments

 @returns the command's exit status; 128 plus the signal number when a signal ended it; 127 when it could not be started

 @example
 ```ts
 const status = await runNicedCommand({ command: ['mise', 'run', 'test'] });
 ```
 */
export async function runNicedCommand({ command }: { readonly command: readonly string[] }): Promise<number> {
  /**
   The command under `nice`, with the caller's standard streams.
   */
  const child = spawn(
    niceProgram,
    [
      `--adjustment=${String(slotNiceAdjustment)}`,
      ...command
    ],
    { stdio: 'inherit' },
  );
  /**
   Passes the gate's signals on until the command has ended.
   */
  using forwarding = forwardSignals(child);
  try {
    await once(
      child,
      'close',
    );
    /**
     How the command ended: an exit code, or the signal that ended it.
     */
    const {
      exitCode,
      signalCode
    } = child;
    if (exitCode !== null)
      return exitCode;
    return signalStatusBase + (signalCode === null ? 0 : constants.signals[signalCode]);
  }
  catch (error) {
    console.error(`${niceProgram} could not run ${command.join(' ')}: ${String(error)}`);
    return commandNotFoundStatus;
  }
}
