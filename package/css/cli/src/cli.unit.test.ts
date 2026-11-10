import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  findRootCached,
  MISE_MONOREPO,
} from '@monochromatic-dev/fs-path-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
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

//region Build fixture

/**
 One disposable directory holding the CSS a build run reads and writes, so no
 build output lands in the package or outlives the run.
 */
type DisposableBuildDirectory = {
  /**
   Absolute path of the CSS entry point.
   */
  readonly inputPath: string;
  /**
   Absolute path the build writes its bundle to.
   */
  readonly outputPath: string;
  /**
   Removes the directory and everything written into it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates the fixture one real build run reads, with an import and a mixin so
 a successful run has resolved both.
 
 @returns Entry path, output path, and the disposal hook that removes both.
 */
async function disposableBuildDirectory(): Promise<DisposableBuildDirectory> {
  /**
   Directory owning this run's inputs and output.
   */
  const directoryPath = await mkdtemp(join(
    scratchRoot(),
    'css-cli-bin-',
  ),);
  /**
   CSS entry point the build starts from.
   */
  const inputPath = join(
    directoryPath,
    'main.css',
  );
  await writeFile(
    join(directoryPath, 'shared.css',),
    '.shared { color: rebeccapurple; }',
  );
  await writeFile(
    inputPath,
    "@import './shared.css';\n\n@mixin --flex-center { display: flex; }\n\n.component { @apply --flex-center; }\n",
  );
  return {
    inputPath,
    outputPath: join(directoryPath, 'dist', 'bundle.css',),
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        directoryPath,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

//endregion Build fixture

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

    //region Build: the bin's real INPUT OUTPUT path, run against a disposable fixture

    it({
      name: 'builds INPUT into OUTPUT and exits 0',
      fn: async () => {
        await using build = await disposableBuildDirectory();
        const result = await runBuildCss({
          args: [build.inputPath, build.outputPath,],
        },);

        expect(result.exitCode,).toBe(0,);
        expect(result.stderr,).not.toContain('Error',);
        const written = await readFile(build.outputPath, 'utf8',);
        expect(written,).toContain('.shared { color: rebeccapurple; }',);
        expect(written,).toContain('display: flex',);
        expect(written,).not.toContain('@import',);
      },
    },),

    it({
      name: 'exits non-zero when an import target is missing',
      fn: async () => {
        await using build = await disposableBuildDirectory();
        await writeFile(
          build.inputPath,
          "@import './absent.css';\n",
        );
        const result = await runBuildCss({
          args: [build.inputPath, build.outputPath,],
        },);

        expect(result.exitCode,).not.toBe(0,);
        expect(result.stderr,).toContain('relative path not found',);
      },
    },),

    //endregion Build
  ],
},);
