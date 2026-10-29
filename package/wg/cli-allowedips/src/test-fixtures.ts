import { spawn as spawnChild, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';
import { text, } from 'node:stream/consumers';
import { fileURLToPath, } from 'node:url';

/**
 Disposable temporary directory used by built-CLI tests.
 */
export type TempDir = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Captured built-CLI process result.
 */
export type CliResult = {
  readonly exitCode: number;
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Built command path resolved relative to test helper.
 */
const BIN_PATH = fileURLToPath(new URL(
  '../dist/final/node/index.mjs',
  import.meta.url,
),);

/**
 Error raised when child closes without numeric exit code.
 */
class ProcessExitError extends Error {
  /**
   Stable error type name.
   */
  override name = 'ProcessExitError';
}

/**
 Creates disposable temporary directory for input-file fixtures.
 
 @returns Directory removed recursively by asynchronous disposer.
 
 @example
 ```ts
 await using directory = await makeTempDir();
 ```
 */
export async function makeTempDir(): Promise<TempDir> {
  /**
   Fresh operating-system temporary path.
   */
  const path = await mkdtemp(join(
    tmpdir(),
    'wg-allowedips-',
  ),);
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Runs built command and captures output and process failure.
 
 @param args - Arguments passed after built command path.
 
 @returns Exit code and exact output streams.
 
 @example
 ```ts
 await runCli({ args: [] });
 ```
 */
export async function runCli(
  { args, }: { readonly args: readonly string[]; },
): Promise<CliResult> {
  /**
   Built command subprocess with output captured as text.
   */
  const subprocess = spawnChild(
    'node',
    [
      BIN_PATH,
      ...args,
    ],
    { stdio: 'pipe', },
  );
  /**
   Exact stdout,
   stderr,
   and close event awaited concurrently.
   */
  const [stdout, stderr,] = await Promise.all([
    text(subprocess.stdout,),
    text(subprocess.stderr,),
    once(
      subprocess,
      'close',
    ),
  ],);
  /**
   Numeric process exit code established by close event.
   */
  const { exitCode, } = subprocess;
  if (exitCode === null)
    throw new ProcessExitError('Built CLI closed without a numeric exit code.',);
  return {
    exitCode,
    stdout,
    stderr,
  };
}
