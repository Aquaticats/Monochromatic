/**
 Paths and steps shared by the `mutation:*` container entry points
 (`./mutation-score.ts`, `./mutation-sweep.ts`, `./mutation-differential.ts`).

 Inside the image from `../container/mutation.Containerfile`, `/upstream`
 already holds the pinned StrykerJS, Vitest, and esbuild install. The
 deepmerge-ts checkout is mounted read-only at `/checkout`, the output
 directory writable at `/out`, and this repo read-only at its host path.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { cp, } from 'node:fs/promises';
import { join, } from 'node:path';

/**
 Read-only deepmerge-ts checkout mount.
 */
export const CHECKOUT = '/checkout';

/**
 Writable project directory holding the pinned tool install.
 */
export const UPSTREAM = '/upstream';

/**
 Writable output mount.
 */
export const OUT = '/out';

/**
 Checkout entries upstream's Vitest suite and Stryker need; everything else
 (lockfile, build config, docs) stays out of the image.
 */
const STAGED_ENTRIES = [
  'src',
  'tests',
  'tsconfig.json',
  'vitest.config.ts',
] as const;

/**
 Error for a container step that exited unsuccessfully.
 */
export class MutationStepError extends Error {
  /**
   @param message - Failed step and its exit.
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'MutationStepError';
  }
}

/**
 Copy the checkout's source, tests, and configs into `/upstream`.

 @example
 ```ts
 await stageCheckout();
 ```
 */
export async function stageCheckout(): Promise<void> {
  await Promise.all(STAGED_ENTRIES.map(async function stage(entry,) {
    await cp(
      join(
        CHECKOUT,
        entry,
      ),
      join(
        UPSTREAM,
        entry,
      ),
      {
        force: true,
        recursive: true,
      },
    );
  },),);
}

/**
 Run one command, streaming its output.

 @param command - Executable.

 @param args - Arguments.

 @param cwd - Working directory.

 @throws {@link MutationStepError} When the command exits unsuccessfully.

 @example
 ```ts
 await runStep({ args: ['run',], command: 'npx', cwd: UPSTREAM, });
 ```
 */
export async function runStep(
  {
    command,
    args,
    cwd,
  }: {
    readonly command: string;
    readonly args: readonly string[];
    readonly cwd: string;
  },
): Promise<void> {
  /**
   Child running the step.
   */
  const child = spawn(
    command,
    args,
    {
      cwd,
      stdio: 'inherit',
    },
  );
  /**
   Exit code and signal once the step closed.
   */
  const closed: readonly unknown[] = await once(
    child,
    'close',
  );
  /**
   Exit code, `null` when a signal ended the step.
   */
  const [code,] = closed;
  if (code !== 0)
    throw new MutationStepError(`${command} ${args.join(' ',)} exited with ${String(code,)}`,);
}
