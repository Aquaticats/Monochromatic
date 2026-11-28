/**
 One command a guest driver's step runs, under its own bound, with the command's whole output kept in one named log.

 Every step of every guest driver is this shape: a program started from a complete argument array, its standard output
 and error written to a file in the log directory, and a bound after which the program and everything it started are
 stopped. The programs are `mvm`, the guest recipe's own driver and `tar`; none of them is read by a shell, so an argument
 is passed whole however it is spelled.

 A driver names its logs and its bounds. A command whose status is the step's own answer is started through
 {@link runStepCommand}, and one whose failure is a refusal with nothing to judge is started through
 {@link runRequiredStepCommand}. A command that prints nothing a run reads is started through
 {@link runQuietStepCommand}, one whose output belongs on the driver's own terminal through
 {@link runVisibleStepCommand}, and one whose status is a question the driver asks rather than a refusal it reports
 through {@link statusOf}.

 The command runs in a process group of its own, and its bound stops that group and everything it started, which is the
 answer of the three sets of guest helpers that survived: `mvm` starts a guest's command on the far side of a connection,
 so a bound that reached only the client would leave the guest working with nobody reading it. A group of its own is also
 what a person's interrupt key does not reach, so a step this process is still waiting for is recorded in
 `mvm-guest-live-steps.ts`, and a signal that ends the driver stops it and says so.
 */
import { spawn, } from 'node:child_process';
import {
  open,
  readFile,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';

import {
  type CommandDeadline,
  stopAtDeadline,
} from './command-deadline.ts';
import { LOG_FILE_MODE, } from './mvm-guest-bounds.ts';
import {
  GuestDriverError,
  refusalOf,
} from './mvm-guest-error.ts';
import { holdTheLiveStep, } from './mvm-guest-live-steps.ts';
import { runGateCommand, } from './native-gate-run.ts';

/**
 The status a command that ended without exiting reports: minus one, the value the gate commands use for the same case.
 */
const NO_EXIT_STATUS = -1;

/**
 The value the deadline's release carries, which no exit status pair can be, so a stopped command is told from an ended
 one without reading which promise won.
 */
const STOPPED_BY_THE_DEADLINE: unique symbol = Symbol(
  'its bound elapsed before the command exited',
);

/**
 One command's own end: its exit status, or minus one when a signal ended it, and that signal's name.
 */
type StepCommandEnding = readonly [
  number,
  string,
];

/**
 How waiting for one step command ended, before the caller reads the status it exited with.
 */
type StepCommandEnd =
  | {
    readonly kind: 'exited';
    readonly status: number;
  }
    | { readonly kind: 'stopped'; }
    | {
      readonly kind: 'refused';
      readonly cause: unknown
    };

/**
 The command as a refusal names it: the program and every argument, spelled as a person would type them.

 @param program - the program the step starts

 @param argumentWords - its arguments, each kept whole

 @returns the program and its arguments, separated by spaces

 @example
 ```ts
 stepTextOf({ program: 'mvm', argumentWords: ['list'] }); // 'mvm list'
 ```
 */
function stepTextOf({
  program,
  argumentWords,
}: {
  readonly program: string;
  readonly argumentWords: readonly string[];
},): string {
  return `${program} ${argumentWords.join(' ',)}`;
}

/**
 The deadline's own release, told apart from an exit status by being this module's one symbol.

 @param deadline - the stop this command's bound makes

 @returns the sentinel, once the grace after the stop has passed

 @example
 ```ts
 await releasedAsSentinel({ deadline });
 ```
 */
async function releasedAsSentinel({
  deadline,
}: {
  readonly deadline: CommandDeadline;
},): Promise<typeof STOPPED_BY_THE_DEADLINE> {
  await deadline.released();
  return STOPPED_BY_THE_DEADLINE;
}

/**
 The race between one command's own end and the release after its bound stopped it.

 The race is kept whole rather than raising straight out of it: a command that could not be started rejects its own
 promise, and the caller has to end the deadline's timer whichever way the race went, or a command named on a path that
 does not exist would leave this process alive until the bound passed.

 @param end - the command's own end, as its exit status and the signal that ended it

 @param deadline - the stop this command's bound makes

 @returns how the race ended, with a command that could not start reported as a refusal

 @example
 ```ts
 const outcome = await endedOrRefused({ end, deadline });
 ```
 */
async function endedOrRefused({
  end,
  deadline,
}: {
  readonly end: Promise<StepCommandEnding>;
  readonly deadline: CommandDeadline;
},): Promise<StepCommandEnd> {
  try {
    /**
     Whichever came first: the command's own end, or the release after its bound stopped it.
     */
    const winner = await Promise.race([
      end,
      releasedAsSentinel({ deadline, },),
    ],);
    if ((typeof winner) === 'symbol')
      return { kind: 'stopped', };
    return {
      kind: 'exited',
      status: winner[0],
    };
  }
  catch (caught) {
    return {
      kind: 'refused',
      cause: caught,
    };
  }
}

/**
 The status one started command exited with, once it ended inside its bound.

 A command its bound stopped refuses with the deadline's own sentence, and one that never started refuses with the
 reason the start gave.

 @param child - the started command, opened so its own bound can reach what it started

 @param step - the command as a refusal names it

 @param timeoutMilliseconds - how long the command may run before it and everything it started are stopped

 @returns the status the command exited with, or minus one when a signal ended it

 @throws GuestDriverError when the command never started, or when its bound stopped it

 @example
 ```ts
 const status = await awaitStepCommand({ child, step: 'mvm list', timeoutMilliseconds: 120_000 });
 ```
 */
async function awaitStepCommand({
  child,
  step,
  timeoutMilliseconds,
}: {
  readonly child: ReturnType<typeof spawn>;
  readonly step: string;
  readonly timeoutMilliseconds: number;
},): Promise<number> {
  /**
   The command's own end: its exit status, or minus one when a signal ended it, and that signal's name.
   */
  const end = Promise.withResolvers<StepCommandEnding>();
  child.once(
    'close',
    function ended(
      status,
      signal,
    ) {
      end.resolve([
        status ?? NO_EXIT_STATUS,
        signal ?? '',
      ],);
    },
  );
  child.once(
    'error',
    function refused(cause: unknown) {
      end.reject(cause,);
    },
  );
  /**
   The stop this command's bound makes, which names the bound and the command it stopped.
   */
  const deadline = stopAtDeadline({
    child,
    deadlineMilliseconds: timeoutMilliseconds,
    refusal: `${step} was still running after ${String(timeoutMilliseconds,)} ms, the bound it is given, so it and everything it started were stopped.`,
  },);
  /**
   The command as a step this process has started, held so a signal that ends the driver stops it, and forgotten when this
   wait answers.
   */
  using _held = holdTheLiveStep({
    child,
    step,
  },);
  /**
   How the wait ended, with the deadline's timer ended before any of it is refused.
   */
  const outcome = await endedOrRefused({
    end: end.promise,
    deadline,
  },);
  deadline.end();
  if (outcome.kind === 'refused')
    throw refusalOf({ caught: outcome.cause, },);
  if (outcome.kind === 'stopped')
    throw new GuestDriverError(deadline.refusal,);
  return outcome.status;
}

/**
 Run one step command with its whole output kept in one named log, and answer the status it exited with.

 Both streams go to the one file, which is truncated first: a step's standard error has always been kept beside its
 standard output in that step's own log, so a failure is read from the same place as a success.

 @param logDirectory - the directory the step's logs live in

 @param program - the program to run, found on `PATH`

 @param argumentWords - its arguments, each passed whole

 @param log - the log's name inside the log directory

 @param timeoutMilliseconds - how long it may run before it and everything it started are stopped

 @returns the status the command exited with, or minus one when a signal ended it

 @throws GuestDriverError when it never started, or when its bound stopped it

 @example
 ```ts
 const status = await runStepCommand({
   logDirectory: '/logs',
   program: 'mvm',
   argumentWords: ['list'],
   log: 'mvm-list.log',
   timeoutMilliseconds: 120_000,
 });
 ```
 */
export async function runStepCommand({
  logDirectory,
  program,
  argumentWords,
  log,
  timeoutMilliseconds,
}: {
  readonly logDirectory: string;
  readonly program: string;
  readonly argumentWords: readonly string[];
  readonly log: string;
  readonly timeoutMilliseconds: number;
},): Promise<number> {
  /**
   The one log this step's streams are written to, closed however the wait ends.
   */
  await using output = await open(
    join(
      logDirectory,
      log,
    ),
    'w',
    LOG_FILE_MODE,
  );
  /**
   The started command, leading a process group of its own so its bound reaches everything it starts.
   */
  const child = spawn(
    program,
    [...argumentWords,],
    {
      stdio: [
        'ignore',
        output.fd,
        output.fd,
      ],
      detached: true,
    },
  );
  return await awaitStepCommand({
    child,
    step: stepTextOf({
      program,
      argumentWords,
    },),
    timeoutMilliseconds,
  },);
}

/**
 Run one step command that has to succeed, and refuse with the command and its status when it did not.

 @param logDirectory - the directory the step's logs live in

 @param program - the program to run, found on `PATH`

 @param argumentWords - its arguments, each passed whole

 @param log - the log's name inside the log directory

 @param timeoutMilliseconds - how long it may run before it and everything it started are stopped

 @throws GuestDriverError when it never started, when its bound stopped it, or when it exited nonzero

 @example
 ```ts
 await runRequiredStepCommand({
   logDirectory: '/logs',
   program: 'mvm',
   argumentWords: ['push', 'guest-a', '/logs/subject.tar.gz', '/root/subject.tar.gz'],
   log: 'guest-push.log',
   timeoutMilliseconds: 120_000,
 });
 ```
 */
export async function runRequiredStepCommand({
  logDirectory,
  program,
  argumentWords,
  log,
  timeoutMilliseconds,
}: {
  readonly logDirectory: string;
  readonly program: string;
  readonly argumentWords: readonly string[];
  readonly log: string;
  readonly timeoutMilliseconds: number;
},): Promise<void> {
  /**
   The status the command exited with.
   */
  const status = await runStepCommand({
    logDirectory,
    program,
    argumentWords,
    log,
    timeoutMilliseconds,
  },);
  if (status !== 0) {
    throw new GuestDriverError(`${stepTextOf({
      program,
      argumentWords,
    },)} exited ${String(status,)}; see ${log}`,);
  }
}

/**
 Run one step command that prints nothing a run reads, and answer the status it exited with.

 A detached guest script is started this way: the launcher's own text ends with `exit 0`, and everything the script it
 starts prints goes to the guest's shared folder rather than to this command, so the command's output is discarded
 instead of kept in a log nothing would read.

 @param program - the program to run, found on `PATH`

 @param argumentWords - its arguments, each passed whole

 @param timeoutMilliseconds - how long it may run before it and everything it started are stopped

 @returns the status the command exited with, or minus one when a signal ended it

 @throws GuestDriverError when it never started, or when its bound stopped it

 @example
 ```ts
 const status = await runQuietStepCommand({
   program: 'mvm',
   argumentWords: ['exec', 'guest-a', '--', 'sh', '/mnt/shared/build.log.launch.sh'],
   timeoutMilliseconds: 120_000,
 });
 ```
 */
export async function runQuietStepCommand({
  program,
  argumentWords,
  timeoutMilliseconds,
}: {
  readonly program: string;
  readonly argumentWords: readonly string[];
  readonly timeoutMilliseconds: number;
},): Promise<number> {
  /**
   The started command, leading a process group of its own so its bound reaches everything it starts.
   */
  const child = spawn(
    program,
    [...argumentWords,],
    {
      stdio: 'ignore',
      detached: true,
    },
  );
  return await awaitStepCommand({
    child,
    step: stepTextOf({
      program,
      argumentWords,
    },),
    timeoutMilliseconds,
  },);
}

/**
 Run one step command whose output belongs on the driver's own terminal, and answer the status it exited with.

 The guest recipe's steps print as they work and take minutes each, so their output is the run's own rather than a file
 the driver would then have to read back.

 @param program - the program to run, found on `PATH`

 @param argumentWords - its arguments, each passed whole

 @param timeoutMilliseconds - how long it may run before it and everything it started are stopped

 @returns the status the command exited with, or minus one when a signal ended it

 @throws GuestDriverError when it never started, or when its bound stopped it

 @example
 ```ts
 const status = await runVisibleStepCommand({
   program: 'node',
   argumentWords: ['run-fixture.ts', 'create'],
   timeoutMilliseconds: 7_200_000,
 });
 ```
 */
export async function runVisibleStepCommand({
  program,
  argumentWords,
  timeoutMilliseconds,
}: {
  readonly program: string;
  readonly argumentWords: readonly string[];
  readonly timeoutMilliseconds: number;
},): Promise<number> {
  /**
   The started command, leading a process group of its own so its bound reaches everything it starts.
   */
  const child = spawn(
    program,
    [...argumentWords,],
    {
      stdio: 'inherit',
      detached: true,
    },
  );
  return await awaitStepCommand({
    child,
    step: stepTextOf({
      program,
      argumentWords,
    },),
    timeoutMilliseconds,
  },);
}

/**
 Read one log back, by its own path.

 A step keeps one log per command in the run's log folder, and a driver reads the log it judges by the name it gave it;
 the path is the driver's own `join` because a run also reads a log a pull or a copy put there.

 @param path - the log's path on this host

 @returns the log's whole text

 @example
 ```ts
 const printed = await readLog({ path: '/logs/lock-arm.log' });
 ```
 */
export async function readLog({ path, }: { readonly path: string; },): Promise<string> {
  return await readFile(
    path,
    'utf8',
  );
}

/**
 The status one command exited with, for a driver asking a question rather than reporting a refusal.

 A command whose failure is the answer, such as a guest that has not come back from a reboot yet, is asked for its
 status rather than refused: the caller is waiting for a yes, and every other answer is a no.

 @param command - the program to run, found on `PATH`

 @param args - its arguments, each passed whole

 @param timeoutMilliseconds - how long it may run before it and everything it started are stopped

 @returns the status it exited with, or minus one when a signal ended it or it never ran

 @example
 ```ts
 await statusOf({ command: 'mvm', args: ['exec', 'guest-one', '--', 'true'], timeoutMilliseconds: 10_000 });
 ```
 */
export async function statusOf({
  command,
  args,
  timeoutMilliseconds,
}: {
  readonly command: string;
  readonly args: readonly string[];
  readonly timeoutMilliseconds: number;
},): Promise<number> {
  /**
   How the command ended, which a command that never ran answers with minus one.
   */
  const outcome = await runGateCommand({
    program: command,
    args,
    deadlineMilliseconds: timeoutMilliseconds,
  },);
  return outcome.status;
}

/**
 Everything one command on this host wrote to its standard output, and no log of it.

 The doctor driver asks `mvm list` which guest names are held before it creates one. That question is answered by
 whatever the listing printed, and by nothing else: a listing that could not run answers empty text, as the capture the
 drivers used to make did, because it never read the status either.

 @param command - the program to run, found on `PATH`

 @param args - its arguments, each passed whole

 @param timeoutMilliseconds - how long it may run before it and everything it started are stopped

 @returns everything the command wrote to its standard output, empty when it could not run

 @example
 ```ts
 const listed = await captureHostCommand({ command: 'mvm', args: ['list'], timeoutMilliseconds: HOST_COMMAND_TIMEOUT_MS });
 ```
 */
export async function captureHostCommand({
  command,
  args,
  timeoutMilliseconds,
}: {
  readonly command: string;
  readonly args: readonly string[];
  readonly timeoutMilliseconds: number;
},): Promise<string> {
  /**
   How the command ended, whose standard output is the answer.
   */
  const outcome = await runGateCommand({
    program: command,
    args,
    deadlineMilliseconds: timeoutMilliseconds,
  },);
  return outcome.stdout;
}
