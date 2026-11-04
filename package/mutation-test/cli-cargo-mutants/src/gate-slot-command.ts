/**
 Run the gated command at the lowest priority, with the caller's standard streams, and report how it ended as an exit status.

 The gate process stays alive for as long as the command runs, because its claim names this process: a termination signal
 sent to the gate is passed on to the command's whole process group, and the gate ends only after every process of the group
 has, so the slot is not given back while the command or any worker it started still runs. The command is started in a group
 of its own for that, so a signal from the terminal reaches it through the gate, and a command that reads the terminal is not
 in the terminal's foreground group. A command that ends on its own is not held for the background processes it left. (A
 gate killed by `SIGKILL` cannot pass anything on; its claim then reads as dead and the slot is free again while the command
 may still run. A `systemctl stop` of the unit that runs the gate ends the command too.)
 */
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { constants } from 'node:os';
import { setInterval as everyInterval } from 'node:timers/promises';

import {
  niceProgram,
  slotNiceAdjustment,
} from './constants.ts';
import { errorCodeOf } from './process-identity.ts';

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
 The code of the error a signal to a process or group that does not exist raises.
 */
const noSuchProcessCode = 'ESRCH';

/**
 How long the gate waits between looks at whether the command's group has emptied, in milliseconds.
 */
const groupLookIntervalMs = 100;

/**
 Tell whether a process group still has a process, a process that exited but was not yet reaped included.

 @param group - the group's id, the id of the command's process

 @returns false when the kernel reports no such group

 @example
 ```ts
 groupHasProcess(4242);
 ```
 */
function groupHasProcess(group: number): boolean {
  try {
    process.kill(
      -group,
      0,
    );
    return true;
  }
  catch (error) {
    return errorCodeOf(error) !== noSuchProcessCode;
  }
}

/**
 Wait until no process of the command's group is left; the forwarding stays in place meanwhile, so a further signal still reaches the group.

 @param group - the group's id

 @example
 ```ts
 await waitForGroupToEnd(4242);
 ```
 */
async function waitForGroupToEnd(group: number): Promise<void> {
  if (!groupHasProcess(group))
    return;
  for await (const _lookAgain of everyInterval(groupLookIntervalMs))
    if (!groupHasProcess(group))
      return;
}

/**
 Pass the gate's termination signals on to the command's process group until the returned value is disposed.

 @param child - the command's process, the leader of its group

 @param passedOn - receives each signal passed on, so the caller knows whether the workers must be waited for

 @returns the forwarding; declare it with `using` so the handlers are removed when the command ends

 @example
 ```ts
 using forwarding = forwardSignals({ child, passedOn: [] });
 ```
 */
function forwardSignals({
  child,
  passedOn,
}: {
  readonly child: ReturnType<typeof spawn>;
  readonly passedOn: NodeJS.Signals[];
}): Disposable {
  /**
   One handler per signal, kept to be removed.
   */
  const handlers = forwardedSignals.map(function handlerFor(signal) {
    /**
     Pass the signal on to the whole group, since the command's workers are its members.
     */
    function forward(): void {
      passedOn.push(signal);
      if (child.pid === undefined)
        return;
      try {
        process.kill(
          -child.pid,
          signal,
        );
      }
      catch (error) {
        // The group has ended already, which is what the signal asked for.
        if (errorCodeOf(error) !== noSuchProcessCode)
          throw error;
      }
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
    {
      stdio: 'inherit',
      detached: true,
    },
  );
  /**
   The signals passed on to the command's group, which are what makes the gate wait for the workers.
   */
  const passedOn: NodeJS.Signals[] = [];
  /**
   Passes the gate's signals on until the command has ended.
   */
  using forwarding = forwardSignals({
    child,
    passedOn,
  });
  try {
    await once(
      child,
      'close',
    );
    // After a forwarded termination the workers are waited for; a command that ended on its own is not held for them.
    if ((passedOn.length > 0) && (child.pid !== undefined))
      await waitForGroupToEnd(child.pid);
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
