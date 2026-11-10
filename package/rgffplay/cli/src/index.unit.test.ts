import {
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  findRootCached,
  MISE_MONOREPO,
} from '@monochromatic-dev/fs-path-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import spawn, { type SubprocessError, } from 'nano-spawn';

/** Mise monorepo root for spawn cwd, so the built bin path is invariant to the task's launch directory. */
const REPO_ROOT = await findRootCached({ marker: MISE_MONOREPO, },);

/** Built bin path, resolved from the monorepo root. */
const BIN_PATH = 'package/rgffplay/cli/dist/final/node/index.mjs';

/** Query word whose first letter every fixture file name repeats, so the glob built from it matches each fixture. */
const QUERY_WORD = 'sweet';

/** Fixture file names for the ambiguous-match case, whose count the bin reports back. */
const AMBIGUOUS_FIXTURE_NAMES: readonly string[] = [
  'sweet-angel.mp3',
  'sweet-devil.mp3',
];

/**
 Runs the built rgffplay bin as a subprocess and returns stdout, stderr, and exit code.
 
 @param args - CLI arguments to pass after `rgffplay`

 @param musicDir - directory the run searches, exported as `XDG_MUSIC_DIR` so a test never reads a real music folder; omitted, the bin resolves the directory itself
 
 @returns Stdout text, stderr text, and numeric exit code
 
 @example
 ```ts
 const result = await runRgffplay({ args: [] });
 // result.exitCode !== 0
 ```
 */
async function runRgffplay({
  args,
  musicDir,
}: {
  readonly args: readonly string[];
  readonly musicDir?: string;
},): Promise<{
  stdout: string;
  stderr: string;
  exitCode: number;
}> {
  try {
    const result = await spawn('node', [BIN_PATH, ...args,], {
      cwd: REPO_ROOT,
      env: musicDir === undefined ? {} : { XDG_MUSIC_DIR: musicDir, },
    },);
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

    //region Empty search: ripgrep's exit code 1 becomes a named error, before any playback

    it({
      name: 'names the glob when the search matches no file',
      fn: async () => {
        await using musicDir = await mkdtempDisposable(join(
          scratchRoot(),
          'rgffplay-no-match-',
        ),);
        const result = await runRgffplay({
          args: [QUERY_WORD,],
          musicDir: musicDir.path,
        },);
        expect(result.exitCode,).not.toBe(0,);
        expect(result.stderr,).toContain('No files matching glob',);
      },
    },),

    //endregion Empty search

    //region Ambiguous search: several matches are refused rather than played

    it({
      name: 'refuses an ambiguous query and reports how many files matched',
      fn: async () => {
        await using musicDir = await mkdtempDisposable(join(
          scratchRoot(),
          'rgffplay-ambiguous-',
        ),);
        await Promise.all(AMBIGUOUS_FIXTURE_NAMES.map(async function create(name,) {
          await writeFile(
            join(musicDir.path, name,),
            '',
          );
        },),);
        const result = await runRgffplay({
          args: [QUERY_WORD,],
          musicDir: musicDir.path,
        },);
        expect(result.exitCode,).not.toBe(0,);
        expect(result.stderr,)
          .toContain(`Ambiguous match: ${String(AMBIGUOUS_FIXTURE_NAMES.length,)} files found`,);
      },
    },),

    //endregion Ambiguous search
  ],
},);
