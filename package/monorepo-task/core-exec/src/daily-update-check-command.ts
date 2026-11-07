/**
 Runs one package check with a time bound: the check's output is kept at its start, where the first error is, and
 never copied to the journal, and the whole process group is stopped when the bound passes.

 @module
 */

import { spawn, } from 'node:child_process';

import {
  endOf,
  stopGroupAfter,
} from './daily-update-command.ts';
import { reproduceCommand, } from './daily-update-package-check-discovery.ts';
import type {
  CheckOutcome,
  PackageCheck,
} from './daily-update-package-check-types.ts';

/**
 Characters of a check's output kept from its start.
 Chosen so the first error with its location fits well inside it: a compiler prints its first error within the first
 kilobytes, and a larger head only costs memory when many checks run side by side.
 */
export const OUTPUT_HEAD_CHARACTERS = 20_000;

/**
 Milliseconds in one second.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 Tenths of a second in one second, for rounding durations to one decimal.
 */
const TENTHS_PER_SECOND = 10;

/**
 Keeps the start of a child's output.

 @param child - the child process, started with piped output

 @param head - where the start of the output is kept

 @example
 ```ts
 keepHead({ child, head: { text: '', }, },);
 ```
 */
export function keepHead({
  child,
  head,
}: {
  readonly child: ReturnType<typeof spawn>;
  readonly head: { text: string; };
},): void {
  for (const stream of [
    child.stdout,
    child.stderr,
  ]) {
    stream?.on(
      'data',
      function keepChunk(chunk: Buffer,) {
        if (head.text
          .length
          < OUTPUT_HEAD_CHARACTERS)
          head.text = `${head.text}${chunk.toString('utf8',)}`.slice(
            0,
            OUTPUT_HEAD_CHARACTERS,
          );
      },
    );
  }
}

/**
 Runs one package check in the repository.

 @param repository - absolute repository path, the working directory

 @param check - the check

 @param environment - extra environment for the check

 @returns how it ended

 @example
 ```ts
 const outcome = await runCheckBounded({ repository: '/r', check, environment: {}, },);
 ```
 */
export async function runCheckBounded({
  repository,
  check,
  environment,
}: {
  readonly repository: string;
  readonly check: PackageCheck;
  readonly environment: Readonly<Record<string, string>>;
},): Promise<CheckOutcome> {
  /**
   When the check started.
   */
  const startedAt = Date.now();
  /**
   Program and arguments, the package's own `mise` task.
   */
  const [program = 'mise', ...args] = reproduceCommand(check,)
    .split(' ',);
  /**
   The check in its own process group, so one signal reaches everything it started.
   */
  const child = spawn(
    program,
    args,
    {
      cwd: repository,
      detached: true,
      env: {
        ...process.env,
        ...environment,
      },
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Start of the check's output.
   */
  const head = { text: '', };
  keepHead({
    child,
    head,
  },);
  /**
   The pending stop of the check's process group, cancelled when this function ends.
   */
  using stopper = stopGroupAfter({
    pid: child.pid ?? 0,
    timeoutMs: check.timeoutMs,
  },);
  /**
   How the check ended.
   */
  const end = await endOf(child,);
  return {
    exitCode: end.exitCode,
    timedOut: stopper.fired(),
    seconds: Math.round(((Date.now() - startedAt) / MILLISECONDS_PER_SECOND) * TENTHS_PER_SECOND,) / TENTHS_PER_SECOND,
    outputHead: head.text,
  };
}
