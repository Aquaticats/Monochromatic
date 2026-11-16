/**
 Runs one command and hands back what it printed, so the parts of the watch that ask the machine questions can be given the
 answers in a test instead of asking it.

 @module
 */

import {
  type ChildProcess,
  execFile,
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';
import {
  extname,
  join,
} from 'node:path';
import { fileURLToPath, } from 'node:url';
import { promisify, } from 'node:util';
import {
  MILLISECONDS_PER_SECOND,
  TEARDOWN_ANSWER_MILLISECONDS,
} from './driver-constants.ts';

/**
 Bytes in one mebibyte.
 */
const BYTES_PER_MEBIBYTE = 1_048_576;

/**
 How many mebibytes of output a command may print before it is cut off. `ps -e` with its arguments printed 560 KB on this
 machine with 5,348 processes on 2026-11-08, and a machine with many more processes, or much longer command lines, still fits.
 */
const COMMAND_OUTPUT_MEBIBYTES = 8;

/**
 The largest output a command may print.
 */
const COMMAND_OUTPUT_BYTES: number = COMMAND_OUTPUT_MEBIBYTES * BYTES_PER_MEBIBYTE;

/**
 Seconds a question of the machine may take. `systemctl show` and `ps -e` each answered in tens of milliseconds on this machine
 on 2026-11-08, so a command still running after half a minute is one that will not answer, and the look must not wait for it.
 */
const COMMAND_TIMEOUT_SECONDS = 30;

/**
 Milliseconds a question of the machine may take when the caller names no bound of its own.
 */
const COMMAND_TIMEOUT_MILLISECONDS: number = COMMAND_TIMEOUT_SECONDS * MILLISECONDS_PER_SECOND;

/**
 `execFile` with a promise result, which never goes through a shell. Typed through Node's own adapter contract, because the
 generic `promisify(execFile)` form trips `strict-void-return`
 (`doc/troubleshooting/oxlint-promisify-void-return.md`).
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 What one command printed and how it ended.
 */
export type CommandOutput = {
  /**
   The status the command exited with, or minus one when it never ran to an exit.
   */
  readonly status: number;

  /**
   What it printed.
   */
  readonly stdout: string;

  /**
   What it complained about.
   */
  readonly stderr: string;

  /**
   Why it never ran to an exit, when it never did.
   */
  readonly problem?: string;

  /**
   Whether the bound this module was given is what ended the command.
   */
  readonly killedByItsOwnBound?: boolean;

  /**
   The signal that ended the command, when a signal ended it rather than an exit.
   */
  readonly signal?: string;
};

/**
 Asks the machine one question.
 */
export type RunCommand = (request: {
  readonly command: string;
  readonly words: readonly string[];

  /**
   The folder the command runs in, which is this process's own when it names none.
   */
  readonly folder?: string;

  /**
   Variables to add to the environment the command inherits, for a program that would ask a question without them.
   */
  readonly environment?: Readonly<Record<string, string>>;

  /**
   How long the command may take, when the caller wants a bound other than the one this module ships.
   */
  readonly timeoutMilliseconds?: number;
}) => Promise<CommandOutput>;

/**
 Reads one field of a caught value, which is an error object this cannot name.

 @param caught - what was caught, known to be an object

 @param key - the field's name

 @returns the field's value, as anything

 @example
 ```ts
 const code = fieldOf({ caught, key: 'code', },);
 ```
 */
function fieldOf({
  caught,
  key,
}: {
  readonly caught: object;
  readonly key: string;
},): unknown {
  return Reflect.get(
    caught,
    key,
  );
}

/**
 Reads a field that should hold text.

 @param value - the field's value

 @returns the text, or empty text when the field holds anything else

 @example
 ```ts
 const shown = textFieldOf('',);
 ```
 */
function textFieldOf(value: unknown,): string {
  return ((typeof value) === 'string') ? value : '';
}

/**
 Reads the status a command exited with.

 @param value - the status the failure carries

 @returns the status, or minus one when the failure carries none

 @example
 ```ts
 const status = statusFieldOf(undefined,);
 ```
 */
function statusFieldOf(value: unknown,): number {
  return ((typeof value) === 'number') ? value : -1;
}

/**
 Reads whether the bound this module set is what ended the command, which Node reports as `killed` and a killed command says
 nothing else about.

 @param caught - the failure the command threw

 @returns whether the bound ended it

 @example
 ```ts
 const killedByTheBound = killedByItsOwnBoundIn({ caught, },);
 ```
 */
function killedByItsOwnBoundIn({ caught, }: { readonly caught: object },): boolean {
  return fieldOf({
    caught,
    key: 'killed',
  },) === true;
}

/**
 Reads what a failed command reported.

 @param caught - what the failed command threw

 @returns its status, its output, and why it never ran to an exit when it never did

 @example
 ```ts
 const failed = failureOf(new Error('x'),);
 ```
 */
function failureOf(caught: unknown,): CommandOutput {
  if (((typeof caught) !== 'object') || (caught === null)) {
    return {
      status: -1,
      stdout: '',
      stderr: '',
      problem: String(caught,),
    };
  }
  /**
   The failure in the shape this reads.
   */
  const failure: object = caught;
  /**
   The status the command exited with.
   */
  const status = statusFieldOf(fieldOf({
    caught: failure,
    key: 'code',
  },),);
  /**
   The signal that ended the command, empty when it exited rather than being signalled.
   */
  const signal = textFieldOf(fieldOf({
    caught: failure,
    key: 'signal',
  },),);
  return {
    status,
    stdout: textFieldOf(fieldOf({
      caught: failure,
      key: 'stdout',
    },),),
    stderr: textFieldOf(fieldOf({
      caught: failure,
      key: 'stderr',
    },),),
    ...(status === (-1) ? { problem: textFieldOf(fieldOf({
      caught: failure,
      key: 'message',
    },),), } : {}),
    // A command that a signal ended is a command with no status, whether the signal was this module's own bound or somebody
    // else's, and the two are told apart by `killed` alone: Node sets it when the bound is what it signalled.
    ...((signal === '') ? {} : { signal, }),
    ...(killedByItsOwnBoundIn({ caught: failure, },) ? { killedByItsOwnBound: true, } : {}),
  };
}

/**
 Runs one command.

 @param command - the program, found on `PATH`

 @param words - its arguments, each kept whole

 @param timeoutMilliseconds - how long it may take before it is killed

 @param folder - the folder it runs in, which is this process's own when it names none

 @param environment - variables to add to the environment it inherits

 @returns what it printed and how it ended

 @example
 ```ts
 const shown = await runCommandWithChildProcess({ command: 'systemctl', words: ['--user', 'show', 'pi-a.service',], },);
 ```
 */
export async function runCommandWithChildProcess({
  command,
  words,
  timeoutMilliseconds = COMMAND_TIMEOUT_MILLISECONDS,
  folder,
  environment,
}: {
  readonly command: string;
  readonly words: readonly string[];
  readonly timeoutMilliseconds?: number;
  readonly folder?: string;
  readonly environment?: Readonly<Record<string, string>>;
},): Promise<CommandOutput> {
  try {
    /**
     What the command printed.
     */
    const done = await execFileAsync(
      command,
      [...words,],
      {
        encoding: 'utf8',
        maxBuffer: COMMAND_OUTPUT_BYTES,
        timeout: timeoutMilliseconds,
        killSignal: 'SIGKILL',
        ...(folder === undefined ? {} : { cwd: folder, }),
        ...(environment === undefined ? {} : { env: {
          ...process.env,
          ...environment,
        }, }),
      },
    );
    return {
      status: 0,
      stdout: done.stdout,
      stderr: done.stderr,
    };
  }
  catch (caught) {
    // A command that exits non-zero, or that ran past its own bound and was killed, rejects here, and its own output is what
    // the caller reads.
    return failureOf(caught,);
  }
}

/**
 What a command that showed its own output ended with.
 */
export type ShownCommandEnding = {
  /**
   The status it exited with, or nothing when a signal ended it.
   */
  readonly status?: number;

  /**
   Why it could not be started, when it could not be.
   */
  readonly startProblem?: string;

  /**
   Whether the bound the caller gave is what ended it, which is no exit of its own.
   */
  readonly endedByItsOwnBound?: boolean;
};

/**
 How long a program that has been sent the polite signal is given to close before it is killed.

 Five seconds, the answer `TEARDOWN_ANSWER_MILLISECONDS` already gives a program at teardown: a program that has not closed five
 seconds after its polite signal is not going to, and its caller must not wait on it.
 */
const POLITE_SIGNAL_GRACE_MILLISECONDS: number = TEARDOWN_ANSWER_MILLISECONDS;

/**
 The caller's own bound: the polite signal at the bound, a kill a stated grace after it, and an answer at the bound itself.

 The answer is the bound's own, never the program's: a program that ignores the polite signal would otherwise hold its caller for as
 long as it likes, which the review of `main...090e416990` reproduced on 2026-11-15 with a child that ignored `SIGTERM` and a
 caller with a 100 ms bound still unsettled after a second.

 What such a bound leaves behind is the program itself: sent the polite signal at the bound, and killed a grace later if it has not
 closed. Nothing of the caller waits on it, because the child and both timers are unref-ed, and the program's streams are the
 caller's own (a terminal, or a unit's output file), so no pipe of the caller keeps it alive.

 @param child - the process the bound is on

 @param timeoutMilliseconds - when the bound passes

 @returns the answer that settles at the bound, and how to clear it when the program closes first

 @example
 ```ts
 const bound = aBoundOf({ child, timeoutMilliseconds: 1_000, },);
 ```
 */
function aBoundOf({
  child,
  timeoutMilliseconds,
}: {
  readonly child: ChildProcess;
  readonly timeoutMilliseconds: number;
},): {
  readonly answer: Promise<ShownCommandEnding>;
  readonly stop: () => void;
} {
  /**
   The timer the bound is kept by, which the caller may clear.
   */
  const keeping: { timer?: NodeJS.Timeout } = {};
  /**
   The answer the bound settles, which no program of its own can postpone.
   */
  // A timer is a callback API, and this promise settles once: at the bound, or never when the program closes first.
  // oxlint-disable-next-line promise/avoid-new -- the one bridge the package needs around a timer, which settles exactly once
  const answer = new Promise<ShownCommandEnding>(function settleAtTheBound(resolve,) {
    // The one bridge this package needs around a timer: a callback API that settles once, which the rule about promises allows.
    keeping.timer = setTimeout(
      function endAtItsBound() {
        // The polite signal, so a launcher ended this way still prints the line it ends with.
        child.kill('SIGTERM',);
        // The forced kill must not depend on this caller living: a program started detached waits the grace and kills, so the program
        // is ended even when this process is gone by then, which a timer of its own could not promise.
        sendTheForcedKillLater({ child, },);
        // The caller does not wait for the program, so nothing it holds keeps the caller alive.
        child.unref();
        resolve({ endedByItsOwnBound: true, },);
      },
      timeoutMilliseconds,
    );
    keeping.timer
      .unref();
  },);
  return {
    answer,
    stop: function clearTheBound() {
      if (keeping.timer !== undefined) {
        clearTimeout(keeping.timer,);
      }
    },
  };
}

/**
 The program that waits a stated grace and kills a program its bound ended, beside this module in a checkout and in a build.

 Its name carries this module own extension, because Node runs the TypeScript source of a checkout and the built file of an install.
 */
const FORCED_KILL_ENTRY = join(
  import.meta.dirname,
  `end-a-program-after-a-grace${extname(import.meta.filename,)}`,
);

/**
 Starts the program that kills a program its bound ended, a stated grace after the polite signal.

 It is started detached, so it is nobody child: the caller may answer and exit at once, and the kill still happens. When that program
 cannot be started here, the kill falls back to a timer of this process, which keeps this process alive until it fires, because a kill
 that never happens is worse than a caller that outlives its answer by a grace.

 @param child - the program the bound is on

 @example
 ```ts
 sendTheForcedKillLater({ child, },);
 ```
 */
function sendTheForcedKillLater({ child, }: { readonly child: ChildProcess },): void {
  /**
   The number this caller got back when it started the program.
   */
  const {pid} = child;
  // A program that never started has nothing to kill later.
  if (pid === undefined) {
    return;
  }
  try {
    /**
     The killer, which outlives this caller.
     */
    const killer = spawn(
      process.execPath,
      [
        FORCED_KILL_ENTRY,
        `--pid=${String(pid,)}`,
        `--grace-milliseconds=${String(POLITE_SIGNAL_GRACE_MILLISECONDS,)}`,
      ],
      {
        detached: true,
        stdio: 'ignore',
      },
    );
    killer.unref();
  }
  catch (error) {
    /**
     The kill this process keeps alive until it fires.
     */
    const theKill = setTimeout(
      function killIt() {
        child.kill('SIGKILL',);
      },
      POLITE_SIGNAL_GRACE_MILLISECONDS,
    );
    theKill.ref();
  }
}

/**
 Waits for one command to close, and answers how it did.

 @param child - the process

 @returns its status, or why it could not be started

 @example
 ```ts
 const closing = await closingOfACommand(child);
 ```
 */
async function closingOfACommand(child: ChildProcess,): Promise<ShownCommandEnding> {
  try {
    /**
     What the process reported when it closed, which is its status or the signal that ended it.
     */
    const closed: readonly unknown[] = await once(
      child,
      'close',
    );
    /**
     The status the process exited with, or nothing when a signal ended it.
     */
    const [status,] = closed;
    return ((typeof status) === 'number') ? { status, } : {};
  }
  catch (error) {
    // `once` rejects when the process could not be started at all, and that is the one failure a caller reports rather than reads
    // as an exit.
    return { startProblem: String(error,) };
  }
}

/**
 Runs one command with the caller's own streams, so what it prints reaches the terminal as it prints it, and answers with how it
 ended.

 This is the shape a launcher needs: a run's progress must show while it runs, and its status is what the caller answers with.

 @param command - the program, found on `PATH`

 @param words - its arguments, each kept whole

 @param folder - the folder it runs in, which is this process's own when it names none

 @param environment - the environment it gets, which is this process's own when it names none

 @param timeoutMilliseconds - how long it may run, which the caller gives when a command that never ends must not hold the caller

 @returns how it ended, which never throws for a failure of the command itself

 @example
 ```ts
 const ended = await runCommandShowingItsOutput({ command: 'systemd-run', words: ['--user', '--wait', '--unit=pi-a', 'true',], },);
 ```
 */
export async function runCommandShowingItsOutput({
  command,
  words,
  folder,
  environment,
  timeoutMilliseconds,
}: {
  readonly command: string;
  readonly words: readonly string[];
  readonly folder?: string;
  readonly environment?: NodeJS.ProcessEnv;

  /**
   How long it may run, when a command that never ends must not hold its caller.
   */
  readonly timeoutMilliseconds?: number;
},): Promise<ShownCommandEnding> {
  /**
   The command's process.
   */
  const child = spawn(
    command,
    [...words,],
    {
      stdio: 'inherit',
      ...(folder === undefined ? {} : { cwd: folder, }),
      ...(environment === undefined ? {} : { env: environment, }),
    },
  );
  /**
   How the command ended when it closed on its own, or why it could not be started.
   */
  const closing = closingOfACommand(child,);
  /**
   How the caller's own bound ends the wait, or nothing when the caller gave none.
   */
  const bound = (timeoutMilliseconds === undefined)
    ? undefined
    : aBoundOf({
      child,
      timeoutMilliseconds,
    },);
  /**
   How the command ended, which is its own close or the bound, whichever came first.
   */
  /**
   How the command ended, which is its own close or the bound, whichever came first.
   */
  const ending = (bound === undefined)
    ? await closing
    : await Promise.race([
      closing,
      bound.answer,
    ],);

  if (bound !== undefined) {
    bound.stop();
  }
  return ending;
}
