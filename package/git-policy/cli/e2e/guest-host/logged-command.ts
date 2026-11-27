/**
 One command the host runs for a guest, with its own bound, and what ended it.

 The guest drivers keep one log per command so a failure names the file to read
 instead of the last lines of a screen: a guest build, an `mvm` push and a
 wrapper arm each answer with text somebody has to read afterwards, and two
 streams captured apart would lose the order they arrived in. A command whose
 output is progress a person watches, such as the guest recipe's own build,
 inherits this process's streams instead.

 The bound is the child's own: a command still running at it is signalled and
 waited for, so nothing here waits forever.

 @example
 ```ts
 await runToLog({ logDir, subject: 'tar --create', command: 'tar', args: ['--create'], log: 'host-package.log', timeout });
 ```
 */

import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';
import { open, } from 'node:fs/promises';
import { join, } from 'node:path';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import { hostLogFileMode, } from './guest-host-bounds.ts';

/**
 How one child ended.
 */
type Ending = {
  /**
   The status it exited with, or minus one when a signal ended it instead.
   */
  readonly status: number;
  /**
   The signal that ended it, or empty text when it exited by itself.
   */
  readonly signal: string;
};

/**
 Run one command with both streams on one log, and wait for it to end.

 @param logDir - the folder the log is written in

 @param subject - the words the refusal names the command by

 @param command - the program to run

 @param args - its arguments

 @param log - the log file's name in the log folder

 @param timeout - milliseconds the command may take

 @throws Error naming the command and its status, or what refused to start it

 @example
 ```ts
 await runToLog({ logDir: 'target/logs', subject: 'mvm list', command: 'mvm', args: ['list'], log: 'host-mvm-list.log', timeout: 120_000 });
 ```
 */
export async function runToLog({
  logDir,
  subject,
  command,
  args,
  log,
  timeout,
}: {
  readonly logDir: string;
  readonly subject: string;
  readonly command: string;
  readonly args: readonly string[];
  readonly log: string;
  readonly timeout: number;
},): Promise<void> {
  /**
   The log both streams go to, closed however the command ends.
   */
  await using output = await open(
    join(
      logDir,
      log,
    ),
    'w',
    hostLogFileMode,
  );
  /**
   The command, with its streams on the log and its own bound.
   */
  const child = spawn(
    command,
    args,
    {
      stdio: [
        'ignore',
        output.fd,
        output.fd,
      ],
      timeout,
    },
  );
  await waitForTheEnd({
    child,
    subject,
    log,
  },);
}

/**
 Run one command with this process's own streams, and wait for it to end.

 A command a person watches, such as the guest recipe's build of OpenZFS,
 writes where the driver's own output goes, so its progress is visible while it
 runs rather than only in a log afterwards.

 @param subject - the words the refusal names the command by

 @param command - the program to run

 @param args - its arguments

 @param timeout - milliseconds the command may take

 @throws Error naming the command and its status, or what refused to start it

 @example
 ```ts
 await runInheriting({ subject: 'node run-fixture.ts create', command: 'node', args: ['e2e/run-fixture.ts', 'create'], timeout: 7_200_000 });
 ```
 */
export async function runInheriting({
  subject,
  command,
  args,
  timeout,
}: {
  readonly subject: string;
  readonly command: string;
  readonly args: readonly string[];
  readonly timeout: number;
},): Promise<void> {
  /**
   The command, with this process's own streams and its own bound.
   */
  const child = spawn(
    command,
    args,
    {
      stdio: 'inherit',
      timeout,
    },
  );
  await waitForTheEnd({
    child,
    subject,
    log: '',
  },);
}

/**
 Run one command with no output kept, and report the status it exited with.

 What a command prints is not wanted everywhere: an `mvm exec` that only starts a
 detached guest script answers with nothing a person or a log needs, and the
 script's own log is where its work goes. The status is the caller's to judge,
 because the sentence a failed start deserves names the script it was starting.

 @param command - the program to run

 @param args - its arguments

 @param timeout - milliseconds the command may take

 @returns the status it exited with, or minus one when a signal ended it

 @throws Error when the command never started, or when the wait itself failed

 @example
 ```ts
 const status = await runIgnoring({ command: 'mvm', args: ['--backend', 'libvirt', 'exec', 'guest-one', '--', 'true'], timeout: 120_000 });
 ```
 */
export async function runIgnoring({
  command,
  args,
  timeout,
}: {
  readonly command: string;
  readonly args: readonly string[];
  readonly timeout: number;
},): Promise<number> {
  /**
   The command, with no stream of its own kept and its own bound.
   */
  const child = spawn(
    command,
    args,
    {
      stdio: 'ignore',
      timeout,
    },
  );
  /**
   How the command ended.
   */
  const ending = await endingOf({
    child,
    subject: command,
  },);
  return ending.status;
}

/**
 Run one command with no output kept, and report the status it exited with.

 A command whose failure is the answer, such as a guest that has not come back
 from a reboot yet, is asked for its status rather than refused: the caller is
 waiting for a yes, and every other answer is a no.

 @param command - the program to run

 @param args - its arguments

 @param timeout - milliseconds the command may take

 @returns the status it exited with, or minus one when a signal ended it or it
   never ran

 @example
 ```ts
 await statusOf({ command: 'mvm', args: ['--backend', 'libvirt', 'exec', 'guest-one', '--', 'true'], timeout: 10_000 });
 ```
 */
export async function statusOf({
  command,
  args,
  timeout,
}: {
  readonly command: string;
  readonly args: readonly string[];
  readonly timeout: number;
},): Promise<number> {
  /**
   The command, with no stream of its own kept and its own bound.
   */
  const child = spawn(
    command,
    args,
    {
      stdio: 'ignore',
      timeout,
    },
  );
  try {
    /**
     What the close event carried.
     */
    const closeArguments: readonly unknown[] = await once(
      child,
      'close',
    );
    /**
     The status the close event carried, in its own position.
     */
    const [status,] = closeArguments;
    return statusOfClose({ status, },);
  }
  catch (caught) {
    return -1;
  }
}

/**
 Wait for one child to end, and refuse a status or a signal that says it failed.

 @param child - the running child - the running child

 @param subject - the words the refusal names it by

 @param log - the log the command's output went to, empty when it went to this
   process's own streams

 @throws Error naming the subject, how it ended, and where its output is

 @example
 ```ts
 await waitForTheEnd({ child, subject: 'mvm list', log: 'host-mvm-list.log' });
 ```
 */
async function waitForTheEnd({
  child,
  subject,
  log,
}: {
  readonly child: ChildProcess;
  readonly subject: string;
  readonly log: string;
},): Promise<void> {
  /**
   Where the command's output can be read, as the refusal's own sentence.
   */
  const output = (log === '') ? '' : `; see ${log}`;
  /**
   How the command ended.
   */
  const ending = await endingOf({
    child,
    subject,
  },);
  if (ending.signal !== '')
    throw new Error(`${subject} ended by signal ${ending.signal}${output}`);
  if (ending.status !== 0)
    throw new Error(`${subject} exited ${String(ending.status)}${output}`);
}

/**
 Wait for one child to end, and report what ended it.

 A child the host cannot start emits `error` and never `close`, so the wait
 gives the refusal that named it rather than a hang.

 @param child - the running child

 @param subject - the words the refusal names it by

 @returns its status and the signal that ended it

 @throws Error when the child never started, or when the wait itself failed

 @example
 ```ts
 const ending = await endingOf({ child, subject: 'mvm list' });
 ```
 */
async function endingOf({
  child,
  subject,
}: {
  readonly child: ChildProcess;
  readonly subject: string;
},): Promise<Ending> {
  try {
    /**
     What the close event carried: a status, or the signal that replaced it.
     */
    const closeArguments: readonly unknown[] = await once(
      child,
      'close',
    );
    /**
     The status and the signal the close event carried, one of which is null.
     */
    const [status, signal,] = closeArguments;
    return {
      status: statusOfClose({ status, },),
      signal: signalOfClose({ signal, },),
    };
  }
  catch (caught) {
    throw new Error(
      `${subject} never ran: ${caughtValueText(caught,)}`,
      { cause: caught, },
    );
  }
}

/**
 The status one close event carried, or minus one when a signal replaced it.

 @param status - what the close event gave in the status position

 @returns the status, or minus one when it is not a number

 @example
 ```ts
 statusOfClose({ status: 0 });
 ```
 */
function statusOfClose({
  status,
}: {
  readonly status: unknown;
},): number {
  if ((typeof status) === 'number') return status;
  return -1;
}

/**
 The signal one close event carried, or empty text when the child exited by
 itself.

 @param signal - what the close event gave in the signal position

 @returns the signal's name, or empty text when there is none

 @example
 ```ts
 signalOfClose({ signal: null });
 ```
 */
function signalOfClose({
  signal,
}: {
  readonly signal: unknown;
},): string {
  if ((typeof signal) === 'string') return signal;
  return '';
}
