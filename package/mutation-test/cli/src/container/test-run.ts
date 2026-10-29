/**
 Test execution with process-group timeout kill.
 
 @example
 ```ts
 await runTests({ cwd: '/work/packages/module/fs-path', tests: ['src/a.unit.test.ts'], timeoutMs: 5000 });
 ```
 */

import { spawn as nodeSpawn, } from 'node:child_process';
import { once, } from 'node:events';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

/**
 Module logger for container-side test execution.
 */
const l = tagged({ tag: 'mutation-test-container', },);

/**
 Outcome of one full selected-test run.
 */
export type TestRunOutcome = {
  readonly kind: 'passed' | 'failed' | 'timeout';
  readonly durationMs: number;
  readonly detail: string;
};

/**
 Runs one test file under plain node in its own process group.
 
 The child leads a process group (detached) so a timeout kill also
 reaps any grandchildren a mutant may have spawned; a mutant is
 arbitrary bad code and must not outlive its verdict.
 
 @param options - Working directory, test file, and timeout.
 
 @returns Exit outcome for this file.
 
 @example
 ```ts
 await runOneTest({ cwd, test: 'src/a.unit.test.ts', timeoutMs: 5000 });
 ```
 */
async function runOneTest(options: {
  readonly cwd: string;
  readonly test: string;
  readonly timeoutMs: number;
},): Promise<TestRunOutcome> {
  /**
   Logger scoped to this test invocation.
   */
  const rl = tagged({
    tag: runOneTest.name,
    l,
  },);
  /**
   Start timestamp for duration measurement.
   */
  const startedAt = performance.now();
  /**
   Mutable timeout marker shared with the kill timer.
   */
  const state = { timedOut: false, };
  /**
   Detached child so it leads its own killable process group.
   */
  const child = nodeSpawn(
    'node',
    [options.test,],
    {
      cwd: options.cwd,
      detached: true,
      // Inherit so baseline failures land in the container log; mutant
      // runs are noisy but the old tool streamed runner output too.
      stdio: 'inherit',
    },
  );
  /**
   Timeout handle killing the whole process group.
   */
  const timer = setTimeout(
    function killGroup(): void {
      state.timedOut = true;
      rl.warn(`timeout after ${String(options.timeoutMs,)}ms: ${options.test}`,);

      if ((typeof child.pid) === 'number') {
        try {
          process.kill(
            -child.pid,
            'SIGKILL',
          );
        }
        catch (error) {
          rl.warn(`process group kill failed: ${String(error,)}`,);
        }
      }
    },
    options.timeoutMs,
  );
  /**
   Scope guard clearing the kill timer on every exit path.
   */
  using stopTimer = {
    [Symbol.dispose](): void {
      clearTimeout(timer,);
    },
  };

  try {
    /**
     Exit event payload (code, signal) from the child process; `once`
     rejects when the child emits `error` (spawn failure). Kept as
     unknowns since only equality checks and stringification follow.
     */
    const exitInfo: readonly unknown[] = await once(
      child,
      'exit',
    );
    /**
     Exit code, number or null per child_process semantics.
     */
    const [
      code,
      signal,
    ] = exitInfo;

    if (state.timedOut)
      return {
        kind: 'timeout',
        durationMs: performance.now() - startedAt,
        detail: `timeout running ${options.test}`,
      };

    if (code === 0)
      return {
        kind: 'passed',
        durationMs: performance.now() - startedAt,
        detail: '',
      };

    return {
      kind: 'failed',
      durationMs: performance.now() - startedAt,
      detail: `${options.test} exited with ${JSON.stringify(code ?? signal
        ?? 'unknown',)}`,
    };
  }
  catch (error) {
    rl.error(`spawn failed for ${options.test}: ${String(error,)}`,);
    return {
      kind: 'failed',
      durationMs: performance.now() - startedAt,
      detail: `spawn error: ${String(error,)}`,
    };
  }
}

/**
 Runs selected tests in order, stopping at the first failure.
 
 First failure wins because a failing test already proves the mutant
 detected; remaining tests add wall time without information.
 
 @param options - Working directory, test files, and per-run timeout.
 
 @returns Aggregate outcome across the executed prefix of tests.
 
 @example
 ```ts
 await runTests({ cwd, tests: ['src/a.unit.test.ts'], timeoutMs: 5000 });
 ```
 */
export async function runTests(options: {
  readonly cwd: string;
  readonly tests: readonly string[];
  readonly timeoutMs: number;
},): Promise<TestRunOutcome> {
  /**
   Aggregate start timestamp across all files.
   */
  const startedAt = performance.now();

  /* oxlint-disable no-await-in-loop */
  for (const test of options.tests) {
    /**
     Outcome for this test file; sequential by design, since running
     later files after a failure adds wall time without information.
     */
    const outcome = await runOneTest({
      cwd: options.cwd,
      test,
      timeoutMs: options.timeoutMs,
    },);

    if (outcome.kind !== 'passed')
      return {
        kind: outcome.kind,
        durationMs: performance.now() - startedAt,
        detail: outcome.detail,
      };
  }
  /* oxlint-enable no-await-in-loop */

  return {
    kind: 'passed',
    durationMs: performance.now() - startedAt,
    detail: '',
  };
}
