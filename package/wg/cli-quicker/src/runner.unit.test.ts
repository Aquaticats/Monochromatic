import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  CommandError,
  CommandSpawnError,
  CommandTimeoutError,
} from '../dist/final/node/errors.mjs';
import {
  run,
  runAllowingFailure,
} from '../dist/final/node/runner.mjs';

/**
 Name of a program no test host provides.
 */
const MISSING_PROGRAM = 'wg-quicker-no-such-program-for-the-test';

/**
 Deadline a test gives a command it means to outlive, in milliseconds.
 Short enough for a test, and the runner takes any bound the caller gives.
 */
const TEST_DEADLINE_MILLISECONDS = 200;

/**
 Runs a call that must refuse and hands back what it threw.

 @param call - Call expected to reject.

 @returns What the call threw.
 */
async function refusalOf(call: () => Promise<unknown>,): Promise<unknown> {
  try {
    await call();
  }
  catch (error: unknown) {
    return error;
  }
  throw new Error('Expected the call to refuse, but it returned',);
}

await describe({
  name: 'runner',
  children: [
    it({
      name: 'names a program that cannot be started as a spawn failure, with the command it tried and the reason the system gave',
      fn: async () => {
        const refusal = await refusalOf(() => run({ command: MISSING_PROGRAM, args: [], },),);
        expect((refusal as Error).name,).toBe('CommandSpawnError',);
        expect((refusal as Error).message,).toContain(MISSING_PROGRAM,);
        expect((refusal as Error).message,).toContain('ENOENT',);
        expect((refusal as CommandSpawnError).command,).toBe(MISSING_PROGRAM,);
      },
    },),
    it({
      name: 'refuses to report a program that cannot be started as an allowed failure, because a program that never ran is not an exit code',
      fn: async () => {
        const refusal = await refusalOf(() => runAllowingFailure({ command: MISSING_PROGRAM, args: [], },),);
        expect((refusal as Error).name,).toBe('CommandSpawnError',);
        expect((refusal as Error).message,).toContain(MISSING_PROGRAM,);
      },
    },),
    it({
      name: 'returns the exit code and both streams of a command that ran and failed, so an allowed failure stays an answer and not a refusal',
      fn: async () => {
        const result = await runAllowingFailure({
          command: 'sh',
          args: ['-c', 'echo out; echo err >&2; exit 3',],
        },);
        expect(result.exitCode,).toBe(3,);
        expect(result.stdout,).toBe('out\n',);
        expect(result.stderr,).toBe('err\n',);
      },
    },),
    it({
      name: 'still reports a command that failed as a typed command failure, so the exit code and stderr reach the person',
      fn: async () => {
        const refusal = await refusalOf(() => run({
          command: 'sh',
          args: ['-c', 'echo reason >&2; exit 4',],
        },),);
        expect((refusal as Error).name,).toBe('CommandError',);
        expect((refusal as CommandError).exitCode,).toBe(4,);
        expect((refusal as CommandError).stderr,).toBe('reason\n',);
      },
    },),
    it({
      name: 'ends a command that outlives its deadline, and names the command and the limit it passed',
      fn: async () => {
        const started = performance.now();
        const refusal = await refusalOf(() => run({
          command: 'sleep',
          args: ['30',],
          deadlineMilliseconds: TEST_DEADLINE_MILLISECONDS,
        },),);
        const elapsed = performance.now() - started;
        expect((refusal as Error).name,).toBe('CommandTimeoutError',);
        expect((refusal as CommandTimeoutError).command,).toBe('sleep',);
        expect((refusal as CommandTimeoutError).deadlineMilliseconds,).toBe(TEST_DEADLINE_MILLISECONDS,);
        expect((refusal as Error).message,).toContain(String(TEST_DEADLINE_MILLISECONDS,),);
        expect(elapsed,).toBeLessThan(10_000,);
      },
    },),
    it({
      name: 'ends a command that outlives its deadline on the allowed-failure path too, so a wedged tool cannot hold a teardown',
      fn: async () => {
        const refusal = await refusalOf(() => runAllowingFailure({
          command: 'sleep',
          args: ['30',],
          deadlineMilliseconds: TEST_DEADLINE_MILLISECONDS,
        },),);
        expect((refusal as Error).name,).toBe('CommandTimeoutError',);
      },
    },),
    it({
      name: 'leaves a command that finishes inside its deadline alone, with both streams and its exit code',
      fn: async () => {
        const result = await run({
          command: 'sh',
          args: ['-c', 'echo out; echo err >&2',],
          deadlineMilliseconds: 30_000,
        },);
        expect(result.stdout,).toBe('out\n',);
        expect(result.stderr,).toBe('err\n',);
      },
    },),
  ],
},);
