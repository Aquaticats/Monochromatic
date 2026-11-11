/**
 Runs one program for a gate command without a shell, with a bound, and reports what it printed.

 The two programs these commands start are `systemd-run`, which answers as soon as the unit is queued, and `git`, whose
 queries on this repository's worktrees are not instant (`git status --short` of the gate worktree took 8.9 s on
 2026-11-10). Each gets a deadline, because nothing here may wait forever, and each is started from a complete argument
 array, so no argument is ever read by a shell.
 */
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

import { caughtValueText } from '@monochromatic-dev/caught-value-core/ts';

import { NativeGateError } from './native-gate-error.ts';
import { lastLines } from './native-release-host-control-requirements.ts';

/**
 Largest output a gate command may print before it is cut off: 16 MiB, the bound the scratch launcher gave
 `git status --short` of a worktree of this repository.
 */
const captureLimitBytes = 16_777_216;

/**
 Signal that stops a command which ran past its deadline; the same one the repository's other bounded runners send.
 */
const stopSignal = 'SIGKILL';

/**
 `execFile` with a promise result, typed through Node's own adapter contract, because the generic
 `promisify(execFile)` form trips `strict-void-return`
 (`doc/troubleshooting/oxlint-promisify-void-return.md`).
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end or for its deadline.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile);

/**
 What one gate command printed and how it ended.
 */
export type GateCommandOutcome = {
  /**
   Status the command exited with, or minus one when it never ran to an exit.
   */
  readonly status: number;
  /**
   What it printed on standard output.
   */
  readonly stdout: string;
  /**
   What it printed on standard error.
   */
  readonly stderr: string;
  /**
   Whether its deadline stopped it instead of it ending by itself.
   */
  readonly timedOut: boolean;
  /**
   Why it never ran to an exit, when it never did.
   */
  readonly problem?: string;
};

/**
 Reads one field of a caught value, which is an error object this cannot name.

 @param caught - what was caught, known to be an object

 @param key - the field's name

 @returns the field's value, as anything

 @example
 ```ts
 fieldOf({ caught, key: 'code' });
 ```
 */
function fieldOf({
  caught,
  key,
}: {
  readonly caught: object;
  readonly key: string;
}): unknown {
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
 textFieldOf('a');
 ```
 */
function textFieldOf(value: unknown): string {
  return ((typeof value) === 'string') ? value : '';
}

/**
 Reads the status a command exited with.

 @param value - the status the failure carries

 @returns the status, or minus one when the failure carries none

 @example
 ```ts
 statusFieldOf(undefined);
 ```
 */
function statusFieldOf(value: unknown): number {
  return ((typeof value) === 'number') ? value : -1;
}

/**
 Reads whether the deadline stopped the command.

 @param value - the `killed` field the failure carries

 @returns true when `execFile` stopped the command itself

 @example
 ```ts
 stopFieldOf(true);
 ```
 */
function stopFieldOf(value: unknown): boolean {
  return value === true;
}

/**
 Reads what a failed command reported.

 @param caught - what the failed command threw

 @returns its status, its output, whether the deadline stopped it, and why it never ran when it did not

 @example
 ```ts
 failureOf(new Error('x'));
 ```
 */
function failureOf(caught: unknown): GateCommandOutcome {
  if (((typeof caught) !== 'object') || (caught === null)) {
    return {
      status: -1,
      stdout: '',
      stderr: '',
      timedOut: false,
      problem: String(caught),
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
  }));
  /**
   Whether `execFile` stopped the command because its deadline passed.
   */
  const timedOut = stopFieldOf(fieldOf({
    caught: failure,
    key: 'killed',
  }));
  return {
    status,
    stdout: textFieldOf(fieldOf({
      caught: failure,
      key: 'stdout',
    })),
    stderr: textFieldOf(fieldOf({
      caught: failure,
      key: 'stderr',
    })),
    timedOut,
    ...(timedOut || (status !== (-1)) ? {} : { problem: textFieldOf(fieldOf({
      caught: failure,
      key: 'message',
    })) || caughtValueText(caught) }),
  };
}

/**
 Runs one gate command and reports how it ended.

 @param program - the program, found on `PATH`

 @param args - its arguments, each kept whole

 @param deadlineMilliseconds - how long it may run before its whole tree is stopped

 @param environment - its environment, which defaults to this process's

 @param workingDirectory - where it runs, which defaults to this process's directory

 @returns its status, its output and whether the deadline stopped it

 @example
 ```ts
 await runGateCommand({ program: 'git', args: ['-C', '/w', 'status', '--short'], deadlineMilliseconds: 600_000 });
 ```
 */
export async function runGateCommand({
  program,
  args,
  deadlineMilliseconds,
  environment = process.env,
  workingDirectory = process.cwd(),
}: {
  readonly program: string;
  readonly args: readonly string[];
  readonly deadlineMilliseconds: number;
  readonly environment?: NodeJS.ProcessEnv;
  readonly workingDirectory?: string;
}): Promise<GateCommandOutcome> {
  try {
    /**
     What the command printed, which the caller reads as the answer or as the refusal.
     */
    const done = await execFileAsync(
      program,
      [...args],
      {
        encoding: 'utf8',
        env: environment,
        cwd: workingDirectory,
        maxBuffer: captureLimitBytes,
        timeout: deadlineMilliseconds,
        killSignal: stopSignal,
      },
    );
    return {
      status: 0,
      stdout: done.stdout,
      stderr: done.stderr,
      timedOut: false,
    };
  }
  catch (caught) {
    // A status, a deadline and a refused start all arrive here, and the command's own output is what the caller names.
    return failureOf(caught);
  }
}

/**
 How a gate command is named in a refusal: the program and its first argument, which is enough to tell the step apart
 without echoing every argument.

 @param program - the program

 @param args - its arguments

 @returns the step's name

 @example
 ```ts
 stepNameOf({ program: 'git', args: ['-C', '/w', 'checkout', '--quiet', '--detach', 'abc'] });
 ```
 */
export function stepNameOf({
  program,
  args,
}: {
  readonly program: string;
  readonly args: readonly string[];
}): string {
  return args.length === 0
    ? program
    : `${program} ${args[0] ?? ''}`;
}

/**
 Runs one gate command and refuses unless it ran to a zero exit inside its deadline.

 @param program - the program, found on `PATH`

 @param args - its arguments, each kept whole

 @param deadlineMilliseconds - how long it may run before its whole tree is stopped

 @param environment - its environment, which defaults to this process's

 @param workingDirectory - where it runs, which defaults to this process's directory

 @returns its status and output, which the caller reads as the step's answer

 @throws NativeGateError naming the step, the status or deadline, and the command's last lines

 @example
 ```ts
 const answered = await runGateCommandOrThrow({ program: 'git', args: ['rev-parse', 'main'], deadlineMilliseconds: 600_000 });
 ```
 */
export async function runGateCommandOrThrow({
  program,
  args,
  deadlineMilliseconds,
  environment = process.env,
  workingDirectory = process.cwd(),
}: {
  readonly program: string;
  readonly args: readonly string[];
  readonly deadlineMilliseconds: number;
  readonly environment?: NodeJS.ProcessEnv;
  readonly workingDirectory?: string;
}): Promise<GateCommandOutcome> {
  /**
   How the command ended.
   */
  const outcome = await runGateCommand({
    program,
    args,
    deadlineMilliseconds,
    environment,
    workingDirectory,
  });
  /**
   How a refusal names the step.
   */
  const step = stepNameOf({
    program,
    args,
  });
  if (outcome.timedOut)
    throw new NativeGateError(`${step} was still running after ${String(deadlineMilliseconds)} ms, the bound it is given, so it was stopped and told nothing.`);
  if (outcome.problem !== undefined)
    throw new NativeGateError(`${step} could not be run: ${outcome.problem}`);
  if (outcome.status !== 0) {
    throw new NativeGateError(`${step} failed with status ${String(outcome.status)} and said: ${lastLines(`${outcome.stderr}\n${outcome.stdout}`)
      .join(' | ')}`);
  }
  return outcome;
}
