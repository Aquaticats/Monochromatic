/**
 Branch coverage for the collapsed-output executor's failure handling.

 Every case runs the executor in a child process, so the value nano-spawn throws
 is the one it really throws, the captured text is what a task sees, and the
 status is the one the process exits with.

 @module
 */

import {
  spawnSync,
} from 'node:child_process';
import {
  fileURLToPath,
} from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Fixtures

/**
 Status a child exits with when a case wants a failure it chose.
 */
const FAILING_EXIT_CODE = 3;

/**
 Status the executor uses when the failed child reported none of its own.
 */
const UNREPORTED_STATUS = 1;

/**
 Successful status, which a case expects when the call leaves the code alone.
 */
const SUCCESS_STATUS = 0;

/**
 Bound a case gives a child that is meant to finish, in milliseconds.
 */
const FINISHING_CHILD_BOUND_MS = 60_000;

/**
 Bound a case gives a child that is meant to outlive it, in milliseconds.

 The child sleeps a minute when nothing ends it, and the value keeps the case
 well under a second on a loaded host.
 */
const OUTLIVED_BOUND_MS = 300;

/**
 Bound a case gives a child that must print before the bound ends it, in
 milliseconds.

 A case that asserts the printed text has to outlast the child's own start,
 which the shell keeps to milliseconds; a Node child of the same test file took
 2.5 s to start under a full-suite fanout on 2026-11-10, which is why this case
 runs a shell rather than another Node process. The value leaves the shell 60x
 the start time it needs and still keeps the case to two seconds.
 */
const OUTLIVED_PRINTING_BOUND_MS = 2_000;

/**
 Status used when the wrapper child was ended by a signal instead of exiting.
 */
const SIGNALED_STATUS = -1;

/**
 Bound on the wrapper child itself, so a wedged case ends instead of hanging.
 */
const WRAPPER_BOUND_MS = 120_000;

/**
 Built testing seam the wrapper child imports, as an absolute file URL.
 */
const TESTING_ENTRY = new URL('../dist/final/node/testing.mjs', import.meta.url,).href;

/**
 What one wrapper child left behind.
 */
type ChildOutcome = {
  /**
   Status the child process exited with, or {@link SIGNALED_STATUS} when a signal ended it.
   */
  readonly status: number;
  /**
   Text the child wrote to standard output.
   */
  readonly stdout: string;
  /**
   Text the child wrote to standard error.
   */
  readonly stderr: string;
};

/**
 Runs the executor in a child process and reports what that process left.

 @param options - Arguments the executor receives in the child

 @returns the child's status and both captured streams

 @example
 ```ts
 const outcome = runExecutorInChild({ command: 'node', commandArgs: [], verbose: false, allowFailure: false, commandTimeoutMs: 1_000 });
 ```
 */
function runExecutorInChild(options: {
  readonly command: string;
  readonly commandArgs: readonly string[];
  readonly verbose: boolean;
  readonly allowFailure: boolean;
  readonly commandTimeoutMs: number;
},): ChildOutcome {
  /**
   Program the child runs: import the built seam, then run the executor once.
   */
  const program = `const { executeWithCollapsedOutput } = await import(${JSON.stringify(TESTING_ENTRY,)});
await executeWithCollapsedOutput(${JSON.stringify(options,)});
`;
  const child = spawnSync(
    process.execPath,
    [
      '--input-type=module',
      '--eval',
      program,
    ],
    {
      encoding: 'utf8',
      timeout: WRAPPER_BOUND_MS,
    },
  );
  return {
    status: child.status ?? SIGNALED_STATUS,
    stdout: child.stdout,
    stderr: child.stderr,
  };
}

//endregion Fixtures

await describe({
  name: 'executeWithCollapsedOutput',
  children: [
    it({
      name: 'discards the output of a child that succeeded and leaves the exit status alone',
      fn: async () => {
        const outcome = runExecutorInChild({
          command: process.execPath,
          commandArgs: [
            '--eval',
            `process.stdout.write('collapsed standard output\\n'); process.stderr.write('collapsed standard error\\n')`,
          ],
          verbose: false,
          allowFailure: false,
          commandTimeoutMs: FINISHING_CHILD_BOUND_MS,
        },);
        expect(outcome.status,).toBe(SUCCESS_STATUS,);
        expect(outcome.stdout,).toBe('',);
        expect(outcome.stderr,).toBe('',);
      },
    },),
    it({
      name: 'dumps the output a failed child captured and keeps its exit status',
      fn: async () => {
        const outcome = runExecutorInChild({
          command: process.execPath,
          commandArgs: [
            '--eval',
            `process.stdout.write('child standard output\\n'); process.stderr.write('child standard error\\n'); process.exitCode = ${String(FAILING_EXIT_CODE,)}`,
          ],
          verbose: false,
          allowFailure: false,
          commandTimeoutMs: FINISHING_CHILD_BOUND_MS,
        },);
        expect(outcome.status,).toBe(FAILING_EXIT_CODE,);
        expect(outcome.stdout,).toContain('child standard output',);
        expect(outcome.stderr,).toContain('child standard error',);
      },
    },),
    it({
      name: 'leaves the exit status alone when the call allows failure',
      fn: async () => {
        const outcome = runExecutorInChild({
          command: process.execPath,
          commandArgs: [
            '--eval',
            `process.stdout.write('allowed failure\\n'); process.exitCode = ${String(FAILING_EXIT_CODE,)}`,
          ],
          verbose: false,
          allowFailure: true,
          commandTimeoutMs: FINISHING_CHILD_BOUND_MS,
        },);
        expect(outcome.status,).toBe(SUCCESS_STATUS,);
        expect(outcome.stdout,).toContain('allowed failure',);
      },
    },),
    it({
      name: 'names the command it could not start, the reason, and what to check',
      fn: async () => {
        const missingCommand = 'no-such-command-for-the-collapsed-output-test';
        const outcome = runExecutorInChild({
          command: missingCommand,
          commandArgs: [],
          verbose: false,
          allowFailure: false,
          commandTimeoutMs: FINISHING_CHILD_BOUND_MS,
        },);
        // A command that never started reports no status of its own.
        expect(outcome.status,).toBe(UNREPORTED_STATUS,);
        expect(outcome.stderr,).toContain(missingCommand,);
        expect(outcome.stderr,).toContain('ENOENT',);
        expect(outcome.stderr,).toContain('PATH',);
      },
    },),
    it({
      name: 'names the signal that ended the child',
      fn: async () => {
        const outcome = runExecutorInChild({
          command: process.execPath,
          commandArgs: ['--eval', `process.kill(process.pid, 'SIGKILL')`],
          verbose: false,
          allowFailure: false,
          commandTimeoutMs: FINISHING_CHILD_BOUND_MS,
        },);
        expect(outcome.stderr,).toContain('[task-depends] command terminated by signal: SIGKILL',);
        // A child that ended by a signal reports no exit status of its own.
        expect(outcome.status,).toBe(UNREPORTED_STATUS,);
      },
    },),
    it({
      name: 'names the bound that ended the child and raises its status',
      fn: async () => {
        const outcome = runExecutorInChild({
          command: process.execPath,
          commandArgs: ['--eval', 'setTimeout(function never(): void {}, 60_000)'],
          verbose: false,
          allowFailure: false,
          commandTimeoutMs: OUTLIVED_BOUND_MS,
        },);
        expect(outcome.stderr,).toContain('--command-timeout-seconds',);
        expect(outcome.status,).toBe(UNREPORTED_STATUS,);
      },
    },),
    it({
      name: 'dumps what a timed-out child printed before the bound ended it',
      fn: async () => {
        const outcome = runExecutorInChild({
          command: 'sh',
          commandArgs: [
            '-c',
            'echo printed-before-the-bound; echo warned-before-the-bound >&2; sleep 60',
          ],
          verbose: false,
          allowFailure: false,
          commandTimeoutMs: OUTLIVED_PRINTING_BOUND_MS,
        },);
        expect(outcome.stdout,).toContain('printed-before-the-bound',);
        expect(outcome.stderr,).toContain('warned-before-the-bound',);
        expect(outcome.status,).toBe(UNREPORTED_STATUS,);
      },
    },),
  ],
},);
