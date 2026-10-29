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
const BIN_PATH = 'package/llm/image-diff/dist/final/node/cli.mjs';

/**
 Runs the built image-diff bin as a subprocess and returns stdout, stderr, and exit code.
 
 @param args - CLI arguments to pass after `image-diff`
 @returns Stdout text, stderr text, and numeric exit code
 
 @example
 ```ts
 const result = await runImageDiff({ args: ['--help'] });
 // result.exitCode === 0
 ```
 */
async function runImageDiff({ args, }: { args: readonly string[]; },): Promise<{
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
  name: 'image-diff bin (built artifact smoke test)',
  children: [
    //region Help: --help is inert (no network), so it exercises the built bin end-to-end without API calls

    it({
      name: 'prints help and exits 0 with --help',
      fn: async () => {
        const result = await runImageDiff({ args: ['--help',], },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toContain('image-diff',);
        expect(result.stdout,).toContain('Usage:',);
        expect(result.stdout,).toContain('compare',);
        expect(result.stdout,).toContain('embed',);
      },
    },),

    //endregion Help
  ],
},);
