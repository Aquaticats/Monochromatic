/**
 Run one packaging probe command for `./recall-packaging.ts` and collect
 its exit code and combined output.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';

/**
 Per-probe limit; a probe that hangs counts as failing.
 */
const PROBE_TIMEOUT_MS = 300_000;

/**
 Run one probe command and collect its exit code and output.

 @param command - Executable.

 @param args - Arguments.

 @param cwd - Consumer directory.

 @returns Exit code (`-1` when a signal ended it) and combined output.

 @example
 ```ts
 await runProbe({ args: ['--version',], command: 'node', cwd: '/tmp', });
 ```
 */
export async function runProbe(
  {
    command,
    args,
    cwd,
  }: {
    readonly command: string;
    readonly args: readonly string[];
    readonly cwd: string;
  },
): Promise<{
  readonly code: number;
  readonly output: string;
}> {
  /**
   Probe process.
   */
  const child = spawn(
    command,
    args,
    {
      cwd,
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
      timeout: PROBE_TIMEOUT_MS,
    },
  );
  /**
   Output chunks.
   */
  const chunks: Buffer[] = [];
  child.stdout
    .on(
    'data',
    function collectStdout(chunk: Buffer,) {
      chunks.push(chunk,);
    },
    );
  child.stderr
    .on(
    'data',
    function collectStderr(chunk: Buffer,) {
      chunks.push(chunk,);
    },
    );
  /**
   Exit code once both streams closed.
   */
  const closed: readonly unknown[] = await once(
    child,
    'close',
  );
  /**
   Exit code, `null` when a signal ended the probe.
   */
  const [code,] = closed;
  return {
    code: ((typeof code) === 'number') ? code : -1,
    output: Buffer.concat(chunks,)
      .toString('utf8',),
  };
}
