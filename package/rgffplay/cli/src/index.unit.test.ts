import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  findRootCached,
  MISE_MONOREPO,
} from '@monochromatic-dev/fs-path-core/ts';
import spawn, { type SubprocessError, } from 'nano-spawn';

/** Mise monorepo root for spawn cwd, so the built bin path is invariant to the task's launch directory. */
const REPO_ROOT = await findRootCached({ marker: MISE_MONOREPO, },);

/** Built bin path, resolved from the monorepo root. */
const BIN_PATH = 'package/rgffplay/cli/dist/final/node/index.mjs';

/**
 Runs the built rgffplay bin as a subprocess and returns stdout, stderr, and exit code.
 
 @param args - CLI arguments to pass after `rgffplay`
 @returns Stdout text, stderr text, and numeric exit code
 
 @example
 ```ts
 const result = await runRgffplay({ args: [] });
 // result.exitCode !== 0
 ```
 */
async function runRgffplay({ args, }: { args: readonly string[]; },): Promise<{
  stdout: string;
  stderr: string;
  exitCode: number;
}> {
  try {
    const result = await spawn('node', [BIN_PATH, ...args,], { cwd: REPO_ROOT, },);
    return {
      exitCode: 0,
      stderr: result.stderr,
      stdout: result.stdout,
    };
  }
  catch (error: unknown) {
    const spawnError = error as SubprocessError;
    return {
      exitCode: spawnError.exitCode ?? 1,
      stderr: spawnError.stderr,
      stdout: spawnError.stdout,
    };
  }
}

await describe({
  name: 'rgffplay bin (built artifact smoke test)',
  children: [
    //region No-args: the only inert path (prints usage and throws BEFORE any ripgrep/ffplay spawn, so no audio playback)

    it({
      name: 'prints usage and exits non-zero when no arguments given',
      fn: async () => {
        const result = await runRgffplay({ args: [], },);
        expect(result.exitCode,).not.toBe(0,);
        expect(result.stderr,).toContain('Usage: rgffplay',);
      },
    },),

    //endregion No-args
  ],
},);
