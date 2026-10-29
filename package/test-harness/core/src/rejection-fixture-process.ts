/**
 Runs rejection fixtures in disposable child processes. @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { mkdtempDisposable, } from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';
import { text, } from 'node:stream/consumers';
import { fileURLToPath, } from 'node:url';

/**
 Child output retained for assertions at the process boundary.
 */
export type RejectionProcessResult = {
  /**
   Exit status proving whether the file failed.
   */
  readonly code: number;
  /**
   Captured standard error, including runtime diagnostics.
   */
  readonly stderr: string;
  /**
   Captured standard output, including harness verdicts.
   */
  readonly stdout: string;
};

/**
 Upper bound for fixtures that would otherwise leave timers running.
 */
const CHILD_TIMEOUT_MS = 20_000;

/**
 Executes one fixture without letting its rejection terminate the parent test.

 @param scenario - fixture branch to exercise

 @param nodeArgs - runtime modes exercised independently of ambient Node options

 @returns child status and both diagnostic streams

 @throws Error when the child is terminated without an exit status

 @example
 ```ts
 const result = await runRejectionProcess({ scenario: 'late', });
 expect(result.code).toBe(1);
 ```
 */
export async function runRejectionProcess({
  scenario,
  nodeArgs = [],
}: {
  readonly nodeArgs?: readonly string[];
  readonly scenario: string;
},): Promise<RejectionProcessResult> {
  /**
   Disposable cwd prevents fixture logger writes in the repository.
   */
  await using directory = await mkdtempDisposable(join(
    tmpdir(),
    'module-test-rejection-',
  ),);
  /**
   Absolute fixture path stays valid after entering the disposable cwd.
   */
  const fixture = fileURLToPath(new URL(
    'rejection-fixture-scenario.ts',
    import.meta.url,
  ),);
  /**
   Child owns all intentionally rejected promises and process listeners.
   */
  const child = spawn(
    process.execPath,
    [
      ...nodeArgs,
      fixture,
      scenario,
    ],
    {
      cwd: directory.path,
      env: {
        ...process.env,
        MONOCHROMATIC_VERBOSE: 'true',
        NODE_OPTIONS: '',
      },
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
      timeout: CHILD_TIMEOUT_MS,
    },
  );
  /**
   Read both pipes concurrently so neither can block child termination.
   */
  const [stdout, stderr, closeEvent,] = await Promise.all([
    text(child.stdout,),
    text(child.stderr,),
    once(
      child,
      'close',
    ) as Promise<readonly unknown[]>,
  ],);
  /**
   Runtime check narrows Node's untyped event payload before returning it.
   */
  const [code,] = closeEvent;
  if ((typeof code) !== 'number')
    throw new Error(`Rejection fixture ${scenario} did not exit normally: ${stderr}`,);
  return {
    code,
    stderr,
    stdout,
  };
}
