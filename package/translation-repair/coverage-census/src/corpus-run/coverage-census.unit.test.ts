/**
 Tests for the coverage census command at its boundary: the built command
 run in a child process whose environment carries no provider key, over
 package directories the cases build, so it stops at each refusal it states
 before it runs any suite.

 THE COMMAND IS NEVER RUN TO ITS END HERE. Past the checks of its build
 directory it runs the unit suite in a child, which would run this very case
 inside itself, so every case stops at a refusal: a command line it will not
 read, a baseline it cannot read, or a build directory that is no coverage
 build. The census past those checks is cased through the functions it moved
 into (`coverage-census-run.unit.test.ts` and the files beside it), with the
 steps that touch git, the suite and the coverage handed in.

 EACH CASE RUNS IN A SCRATCH DIRECTORY as the package directory, because the
 command reads its build directory from the working directory it stands in.
 The child is started by `runBuiltCommand`, which removes every variable whose
 name ends in `_API_KEY` and every one whose name starts `TRANSLATION_REPAIR_`
 before it adds the cache home the case names.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ChildRun,
  runBuiltCommand,
} from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { COVERAGE_CENSUS_ENTRY, } from './built-entry.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 What the command's usage line says, as every command-line refusal ends.
 */
const USAGE = 'Usage: coverage-census [--baseline <census.json>] [--source <src/file.ts> ...] [<unit test file> ...]';

/**
 Where a package directory's build lives.

 @param packageDirectory - package directory the command stands in

 @returns Build directory the command reads

 @example
 ```ts
 const dist = distOf({ packageDirectory: scratch.path, },);
 ```
 */
function distOf({ packageDirectory, }: { readonly packageDirectory: string; },): string {
  return join(
    packageDirectory,
    'dist',
    'final',
    'node',
  );
}

/**
 Runs the built command with a package directory of the case's making.

 @param packageDirectory - scratch directory the command stands in, which
 also holds the cache home it would write its census under

 @param args - command's arguments

 @returns Exit code and both streams

 @example
 ```ts
 const run = await runInPackage({ packageDirectory: scratch.path, args: ['--whiskers',], },);
 ```
 */
async function runInPackage(
  {
    packageDirectory,
    args,
  }: {
    readonly packageDirectory: string;
    readonly args: readonly string[];
  },
): Promise<ChildRun> {
  return await runBuiltCommand({
    command: 'coverage-census',
    args,
    packageEntry: COVERAGE_CENSUS_ENTRY,
    cwd: packageDirectory,
    env: {
      XDG_CACHE_HOME: join(
        packageDirectory,
        'cache',
      ),
    },
  },);
}

await describe({
  name: 'coverage-census as built',
  children: [
    it({
      name: 'REFUSES A FLAG IT DOES NOT READ as stated and exits 6 with its usage line, printing nothing else',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'coverage-census-built-', },);
        expect(await runInPackage({
          packageDirectory: scratch.path,
          args: ['--whiskers',],
        },),).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `coverage-census: --whiskers is not a flag this command reads. ${USAGE}\n`,
        },);
      },
    },),
    it({
      name: 'REFUSES --baseline WRITTEN WITHOUT A VALUE as stated and exits 6 with its usage line',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'coverage-census-built-', },);
        expect(await runInPackage({
          packageDirectory: scratch.path,
          args: ['--baseline',],
        },),).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `coverage-census: --baseline needs a value written after it. ${USAGE}\n`,
        },);
      },
    },),
    it({
      name: 'REFUSES --source WITH NO --baseline as stated and exits 6',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'coverage-census-built-', },);
        expect(await runInPackage({
          packageDirectory: scratch.path,
          args: [
            '--source',
            'src/nap.ts',
          ],
        },),).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'coverage-census: --source picks the baseline stretches to read, so it needs --baseline '
            + '<census.json> beside it\n',
        },);
      },
    },),
    it({
      name: 'REFUSES A BASELINE FILE THAT IS NOT THERE as stated, naming the path and the filesystem code, and '
        + 'exits 6 before it looks at any build',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'coverage-census-built-', },);

        /**
         A baseline path nothing is at.
         */
        const baseline = join(
          scratch.path,
          'no-such-census.json',
        );
        expect(await runInPackage({
          packageDirectory: scratch.path,
          args: [
            '--baseline',
            baseline,
          ],
        },),).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `coverage-census: baseline ${baseline} does not read as a census this command wrote: it could `
            + 'not be read (ENOENT)\n',
        },);
      },
    },),
    it({
      name: 'REFUSES A BASELINE FILE THAT IS NOT JSON as stated, without repeating a word of it, and exits 6',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'coverage-census-built-', },);

        /**
         A baseline that holds cat talk rather than a census.
         */
        const baseline = join(
          scratch.path,
          'purring.json',
        );
        await writeFile(
          baseline,
          'purr purr purr\n',
          'utf8',
        );
        expect(await runInPackage({
          packageDirectory: scratch.path,
          args: [
            '--baseline',
            baseline,
          ],
        },),).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `coverage-census: baseline ${baseline} does not read as a census this command wrote: it is not JSON\n`,
        },);
      },
    },),
    it({
      name: 'REFUSES A PACKAGE DIRECTORY WITH NO BUILD DIRECTORY as stated, naming the filesystem code and the way out, '
        + 'and exits 6 rather than 5 as a fault with frames',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'coverage-census-built-', },);
        expect(await runInPackage({
          packageDirectory: scratch.path,
          args: [],
        },),).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `coverage-census: ${distOf({ packageDirectory: scratch.path, },)} could not be listed (ENOENT), so `
            + 'no build has written it there; run the census from the package directory through mise run '
            + '//package/translation-repair/coverage-census:coverage-census, which builds with maps first\n',
        },);
      },
    },),
    it({
      name: 'REFUSES A BUILD DIRECTORY HOLDING NO BUNDLE as stated, naming it and the task that builds with maps, '
        + 'and exits 6 before any suite',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'coverage-census-built-', },);
        await mkdir(
          distOf({ packageDirectory: scratch.path, },),
          { recursive: true, },
        );
        expect(await runInPackage({
          packageDirectory: scratch.path,
          args: [],
        },),).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `coverage-census: ${distOf({ packageDirectory: scratch.path, },)} holds no bundle, so no build `
            + 'has written it; run the census through mise run //package/translation-repair/coverage-census:coverage-census, '
            + 'which builds with maps first\n',
        },);
      },
    },),
    it({
      name: 'REFUSES A BUILD WITH NO SOURCE MAP as the normal build, in the singular for one bundle, and exits 6',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'coverage-census-built-', },);

        /**
         Build directory holding one bundle with no map beside it.
         */
        const dist = distOf({ packageDirectory: scratch.path, },);
        await mkdir(
          dist,
          { recursive: true, },
        );
        await writeFile(
          join(
            dist,
            'nap.mjs',
          ),
          'function nap() {}\n',
          'utf8',
        );
        expect(await runInPackage({
          packageDirectory: scratch.path,
          args: [],
        },),).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `coverage-census: ${dist} holds 1 bundle and no source map beside any of them, so it is the `
            + 'normal build; run the census through mise run //package/translation-repair/coverage-census:coverage-census, '
            + 'which builds with maps first\n',
        },);
      },
    },),
    it({
      name: 'REFUSES A MINIFIED BUILD as stated, naming the ledger entry and the way out, and exits 6 before any '
        + 'suite',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'coverage-census-built-', },);

        /**
         Build directory holding one bundle with a map and no module region comment.
         */
        const dist = distOf({ packageDirectory: scratch.path, },);
        await mkdir(
          dist,
          { recursive: true, },
        );
        await writeFile(
          join(
            dist,
            'nap.mjs',
          ),
          'function nap() {}\n',
          'utf8',
        );
        await writeFile(
          join(
            dist,
            'nap.mjs.map',
          ),
          '{}',
          'utf8',
        );
        expect(await runInPackage({
          packageDirectory: scratch.path,
          args: [],
        },),).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `coverage-census: ${dist} holds 1 bundle with a source map and no module region comment in any `
            + 'of them, so the build was minified, and minification folds guards into expressions the coverage '
            + 'gives no range, which would read as run (ledger M79): run the census through mise run '
            + '//package/translation-repair/coverage-census:coverage-census, whose build keeps the code as written, or '
            + 'restore minify: false in rolldown.coverage.config.ts if it was changed\n',
        },);
      },
    },),
  ],
},);
