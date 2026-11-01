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
const BIN_PATH = 'package/css/cli/dist/final/node/cli.mjs';

/**
 Runs the built build-css bin as a subprocess and returns stdout, stderr, and exit code.
 
 @param args - CLI arguments to pass after `build-css`
 @returns Stdout text, stderr text, and numeric exit code
 
 @example
 ```ts
 const result = await runBuildCss({ args: ['--help'] });
 // result.exitCode === 0
 ```
 */
async function runBuildCss({ args, }: { args: readonly string[]; },): Promise<{
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
  name: 'build-css bin (built artifact smoke test)',
  children: [
    //region Help: --help is inert (optique exits before reading/writing any CSS file), exercising the built bin safely

    it({
      name: 'prints help and exits 0 with --help',
      fn: async () => {
        const result = await runBuildCss({ args: ['--help',], },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toContain('build-css',);
        expect(result.stdout,).toContain('Usage:',);
        expect(result.stdout,).toContain('INPUT',);
        expect(result.stdout,).toContain('OUTPUT',);
      },
    },),

    //endregion Help
  ],
},);
