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
const BIN_PATH = 'package/vmsync/cli/dist/final/node/index.mjs';

/**
 Runs the built vmsync bin as a subprocess and returns stdout, stderr, and exit code.
 
 @param args - CLI arguments to pass after `vmsync`
 @returns Stdout text, stderr text, and numeric exit code
 
 @example
 ```ts
 const result = await runVmsync({ args: ['--help'] });
 // result.exitCode === 0
 ```
 */
async function runVmsync({ args, }: { args: readonly string[]; },): Promise<{
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
  name: 'vmsync bin (built artifact smoke test)',
  children: [
    //region Help: --help is the only inert path (every subcommand provisions or mutates VM disk images)

    it({
      name: 'prints help and exits 0 with --help',
      fn: async () => {
        const result = await runVmsync({ args: ['--help',], },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toContain('vmsync',);
        expect(result.stdout,).toContain('Usage:',);
        expect(result.stdout,).toContain('import',);
        expect(result.stdout,).toContain('sync',);
      },
    },),

    //endregion Help
  ],
},);
