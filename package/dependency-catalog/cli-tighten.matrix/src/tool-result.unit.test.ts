/**
 Unit tests for the tool-outcome classification.
 
 Pure host tests (no containers): run real subprocesses through `nano-spawn` and
 classify their rejection, pinning the difference between a tool that ran and
 exited non-zero (its stdout and stderr belong in the assertion message) and a
 tool that never started or was killed (nothing may be concluded about
 catalog-tighten, so the scenario must fail loudly instead).
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import spawn from 'nano-spawn';

import {
  type ToolResult,
  toolResultFromSpawnError,
} from './tool-result.ts';

//region Fixtures

/**
 Deadline for each fixture subprocess; all of them exit immediately.
 */
const FIXTURE_TIMEOUT_MS = 10_000;

/**
 Line the stand-in tool writes to stdout, standing in for catalog-tighten's tighten line.
 */
const STDOUT_LINE = 'picomatch: >=4.0.0 -> >=4.0.2';

/**
 Line the stand-in tool writes to stderr, standing in for catalog-tighten's MISS line.
 */
const STDERR_LINE = 'MISS  picomatch';

/**
 Stand-in tool script that writes both streams and then exits non-zero.
 */
const BOTH_STREAMS_THEN_EXIT_SCRIPT = [
  `console.log(${JSON.stringify(STDOUT_LINE)});`,
  `console.error(${JSON.stringify(STDERR_LINE)});`,
  'process.exit(3);',
].join(
  '\n',
);

/**
 Stand-in tool script that kills itself, so `spawn` rejects with a signal and no exit code.
 */
const SELF_KILL_SCRIPT = 'process.kill(process.pid, "SIGKILL");';

/**
 Command name no environment provides; spawning it fails with ENOENT before any process exists.
 */
const MISSING_COMMAND = 'catalog-tighten-matrix-no-such-command';

/**
 Runs a stand-in tool and returns the value its `spawn` call rejected with.
 
 @param exec - JavaScript source the stand-in tool runs
 
 @returns rejection value from `spawn`
 
 @example
 ```ts
 await spawnRejection('process.exit(3)')
 ```
 */
async function spawnRejection(exec: string,): Promise<unknown> {
  try {
    await spawn(
      process.execPath,
      ['--eval', exec,],
      { timeout: FIXTURE_TIMEOUT_MS, },
    );
  }
  catch (error) {
    return error;
  }
  throw new Error('the stand-in tool was expected to fail',);
}

/**
 Runs a {@link ToolResult} assertion body, returning the value it threw.
 
 @param fn - assertion body that may throw
 
 @returns thrown value, or undefined when the body returned
 
 @example
 ```ts
 await thrownBy(() => toolResultFromSpawnError(error))
 ```
 */
async function thrownBy(fn: () => ToolResult,): Promise<unknown> {
  try {
    fn();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

//endregion Fixtures

await describe({
  name: toolResultFromSpawnError.name,
  children: [
    //region A tool that ran and exited non-zero

    it({
      name: 'reports the tool exit with its stdout and stderr',
      fn: async () => {
        /**
         Rejection from a stand-in tool that wrote both streams and exited 3.
         */
        const error = await spawnRejection(BOTH_STREAMS_THEN_EXIT_SCRIPT,);
        /**
         Classification of that rejection.
         */
        const result = toolResultFromSpawnError(error,);
        expect(result.ok,).toBe(false,);
        expect(result.output,).toContain(STDOUT_LINE,);
        expect(result.output,).toContain(STDERR_LINE,);
      },
    },),

    //endregion A tool that ran and exited non-zero

    //region Nothing the tool did

    it({
      name: 'fails the scenario when the command does not exist',
      fn: async () => {
        /**
         Rejection from a command that never started.
         */
        let error: unknown;
        try {
          await spawn(
            MISSING_COMMAND,
            [],
            { timeout: FIXTURE_TIMEOUT_MS, },
          );
        }
        catch (caught) {
          error = caught;
        }
        /**
         What the classification threw instead of reporting a tool exit.
         */
        const thrown = await thrownBy(function classify() {
          return toolResultFromSpawnError(error,);
        },);
        expect(thrown,).toBeInstanceOf(Error,);
        expect((thrown as Error).message,).toContain('never started',);
      },
    },),

    it({
      name: 'fails the scenario when the tool was killed instead of exiting',
      fn: async () => {
        /**
         Rejection from a stand-in tool that kills itself.
         */
        const error = await spawnRejection(SELF_KILL_SCRIPT,);
        /**
         What the classification threw instead of reporting a tool exit.
         */
        const thrown = await thrownBy(function classify() {
          return toolResultFromSpawnError(error,);
        },);
        expect(thrown,).toBeInstanceOf(Error,);
        expect((thrown as Error).message,).toContain('SIGKILL',);
      },
    },),

    it({
      name: 'rethrows a rejection that is not a subprocess failure',
      fn: async () => {
        /**
         Failure the classification has no interpretation for, standing in for a library bug.
         */
        const error = new Error('unexpected rejection',);
        /**
         What the classification threw.
         */
        const thrown = await thrownBy(function classify() {
          return toolResultFromSpawnError(error,);
        },);
        expect(thrown,).toBe(error,);
      },
    },),

    //endregion Nothing the tool did
  ],
},);
