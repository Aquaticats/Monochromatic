import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ranWhatItSaysOf,
  suitePassCountOf,
  unitTestFilePattern,
  unitTestFilesOf,
} from './native-check-ran-what-it-says.ts';
import { createGateScratchDirectory } from './stand-in-gate-programs-for-tests.ts';

/**
 One suite line as the console sink and the harness write it.

 `formatRecord` puts the level and the ISO stamp first (`package/logger/core/src/sink/console.ts:214`), and a named
 suite's pass is one `info` record whose message opens with the `PASS` tag (`package/test-harness/core/src/describe.ts`,
 `runOnce`; `createVerdictLoggers` in `package/test-harness/core/src/verdict.ts`). The stamp's shape is the one a real
 run printed on 2026-11-27.

 @param name - the tag the suite is named by

 @returns the line the harness prints for that suite
 */
function suiteLine({ name }: { readonly name: string; },): string {
  return `[info] [2026-11-27T17:18:33.609Z] [${name}] [PASS] a test (13ms)`;
}

/**
 The lines a check printed, as one string.

 @param lines - the lines

 @returns the lines joined the way a check's output is
 */
function outputOf({ lines }: { readonly lines: readonly string[]; },): string {
  return `${lines.join('\n',)}\n`;
}

/**
 Writes one file into a fixture, making the directories above it.

 @param root - the fixture's directory

 @param path - the file's path inside the fixture

 @param text - what it holds
 */
async function writeFixtureFile({
  root,
  path,
  text = '',
}: {
  readonly root: string;
  readonly path: string;
  readonly text?: string;
},): Promise<void> {
  /**
   Where the file's own name starts, which is one past its last slash.
   */
  const nameStarts = path.lastIndexOf('/') + 1;
  /**
   The directory the file goes in.
   */
  const directory = join(
    root,
    path.slice(
      0,
      nameStarts
    ),
  );
  await mkdir(
    directory,
    { recursive: true, }
  );
  await writeFile(
    join(
      root,
      path
    ),
    text,
  );
}

/**
 A fixture checkout that holds one package of unit test files, two of which the harness runs.

 The other entries are what the harness never runs and what lies outside the package: an expensive file, a file under
 `node_modules`, a file without the unit test name, and a file of the next package.

 @param checkout - the fixture checkout

 @param packagePath - the package's path inside the checkout
 */
async function writePackageFixture({
  checkout,
  packagePath,
}: {
  readonly checkout: string;
  readonly packagePath: string;
},): Promise<void> {
  await writeFixtureFile({
    root: checkout,
    path: `${packagePath}/first.unit.test.ts`,
  });
  await writeFixtureFile({
    root: checkout,
    path: `${packagePath}/second.unit.test.ts`,
  });
  await writeFixtureFile({
    root: checkout,
    path: `${packagePath}/slow.expensive.unit.test.ts`,
  });
  await writeFixtureFile({
    root: checkout,
    path: `${packagePath}/node_modules/dep/dep.unit.test.ts`,
  });
  await writeFixtureFile({
    root: checkout,
    path: `${packagePath}/helper.ts`,
  });
  await writeFixtureFile({
    root: checkout,
    path: 'package/b/other.unit.test.ts',
  });
}

await describe({
  name: 'what a passing check says it ran',
  children: [
    it({
      name: 'names the harness\'s own pattern for the files a unit run covers',
      fn: async () => {
        expect(unitTestFilePattern,).toBe('**/*.unit.test.ts',);
      },
    }),
    it({
      name: 'reads a package\'s unit test files the way the harness discovers them',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ran-what-', });
        await writePackageFixture({
          checkout: scratch.path,
          packagePath: 'package/a',
        });
        expect(await unitTestFilesOf({ packageDirectory: join(
          scratch.path,
          'package/a'
        ), }),).toEqual([
          'first.unit.test.ts',
          'second.unit.test.ts',
        ],);
      },
    }),
    it({
      name: 'counts one suite for each [PASS] line and never a per-test one',
      fn: async () => {
        /**
         One run's output: two suites, one per-test line at `debug`, and a test's own text that is no suite line.
         */
        const output = outputOf({
          lines: [
            suiteLine({ name: 'first', }),
            // A per-test pass is logged at `debug`, which the console sink writes to standard error and hides.
            '[debug] [2026-11-27T17:18:33.609Z] [a test] [PASS] (13ms)',
            suiteLine({ name: 'second', }),
            'a test said [PASS] somewhere in its own text',
            '',
          ],
        });
        expect(suitePassCountOf({ output, }),).toBe(2,);
        expect(suitePassCountOf({ output: outputOf({ lines: [], }), }),).toBe(0,);
      },
    }),
    it({
      name: 'refuses a unit test run whose output reports no suite',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ran-what-', });
        await writePackageFixture({
          checkout: scratch.path,
          packagePath: 'package/a',
        });
        /**
         What a check that exited 0 having printed nothing earns.
         */
        const ran = await ranWhatItSaysOf({
          task: '//package/a:test:unit',
          output: outputOf({ lines: ['ok 2 tests',], }),
          worktree: scratch.path,
        });
        expect(ran.refused,).toBe(true,);
        /**
         The refusal's words.
         */
        const refusal = ran.lines.join('\n',);
        expect(refusal,).toContain('no [PASS] suite line',);
        // The count it expected is the package's own: two files the harness would run.
        expect(refusal,).toContain('2 unit test files',);
        expect(refusal,).toContain('reports at least 2',);
      },
    }),
    it({
      name: 'refuses a unit test run that reported fewer suites than the package holds files',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ran-what-', });
        await writePackageFixture({
          checkout: scratch.path,
          packagePath: 'package/a',
        });
        /**
         What a check earns that reported one suite where its package holds two files.
         */
        const ran = await ranWhatItSaysOf({
          task: '//package/a:test:unit',
          output: outputOf({ lines: [suiteLine({ name: 'first', }),], }),
          worktree: scratch.path,
        });
        expect(ran.refused,).toBe(true,);
        /**
         The refusal's words.
         */
        const refusal = ran.lines.join('\n',);
        expect(refusal,).toContain('1 suite reported [PASS]',);
        expect(refusal,).toContain('2 unit test files',);
      },
    }),
    it({
      name: 'refuses a unit test run of a package that holds no unit test file',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ran-what-', });
        await mkdir(
          join(
            scratch.path,
            'package/a'
          ),
          { recursive: true, },
        );
        /**
         What a check earns for a package with nothing to run.
         */
        const ran = await ranWhatItSaysOf({
          task: '//package/a:test:unit',
          output: outputOf({ lines: [], }),
          worktree: scratch.path,
        });
        expect(ran.refused,).toBe(true,);
        expect(ran.lines.join('\n',),).toContain('holds no unit test file',);
      },
    }),
    it({
      name: 'names the files and the suites of a run that reported every one',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ran-what-', });
        await writePackageFixture({
          checkout: scratch.path,
          packagePath: 'package/a',
        });
        /**
         What a check earns that reported a suite for each file and one more inside the second.
         */
        const ran = await ranWhatItSaysOf({
          task: '//package/a:test:unit',
          output: outputOf({
            lines: [
              suiteLine({ name: 'first', }),
              suiteLine({ name: 'second', }),
              suiteLine({ name: 'second again', }),
            ],
          }),
          worktree: scratch.path,
        });
        expect(ran.refused,).toBe(false,);
        expect(ran.lines,).toEqual(['//package/a:test:unit: 2 unit test files, and 3 suites reported [PASS]',],);
      },
    }),
    it({
      name: 'finds a package below the folder the checks ran in, or above it',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ran-what-', });
        await writePackageFixture({
          checkout: scratch.path,
          packagePath: 'package/a',
        });
        /**
         What the check earns when `mise` gave it the package's own folder as its working directory.
         */
        const ran = await ranWhatItSaysOf({
          task: '//package/a:test:unit',
          output: outputOf({
            lines: [
              suiteLine({ name: 'first', }),
              suiteLine({ name: 'second', }),
            ],
          }),
          worktree: join(
            scratch.path,
            'package/a'
          ),
        });
        expect(ran.refused,).toBe(false,);
        expect(ran.lines,).toEqual(['//package/a:test:unit: 2 unit test files, and 2 suites reported [PASS]',],);
      },
    }),
    it({
      name: 'refuses a unit test run whose package no folder holds',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ran-what-', });
        /**
         What a check earns when the task names a path no folder at or above the run's own holds.
         */
        const ran = await ranWhatItSaysOf({
          task: '//package/absent:test:unit',
          output: outputOf({ lines: [suiteLine({ name: 'a', }),], }),
          worktree: scratch.path,
        });
        expect(ran.refused,).toBe(true,);
        /**
         The refusal's words.
         */
        const refusal = ran.lines.join('\n',);
        expect(refusal,).toContain('no folder at or above',);
        expect(refusal,).toContain('package/absent',);
      },
    }),
    it({
      name: 'counts the suites of a test:unit task that names no package, and derives no file count',
      fn: async () => {
        /**
         What the repository's own unit run earns, whose file count this command cannot derive.
         */
        const ran = await ranWhatItSaysOf({
          task: '//:test:unit',
          output: outputOf({ lines: [suiteLine({ name: 'a', }),], }),
          worktree: import.meta.dirname,
        });
        expect(ran.refused,).toBe(false,);
        expect(ran.lines,).toEqual(['//:test:unit: 1 suite reported [PASS], and no count of unit test files is derived for a task that names no package',],);
      },
    }),
    it({
      name: 'refuses a test:unit task that names no package and reported no suite',
      fn: async () => {
        /**
         What the repository's own unit run earns when its output holds no suite line.
         */
        const ran = await ranWhatItSaysOf({
          task: '//:test:unit',
          output: '',
          worktree: import.meta.dirname,
        });
        expect(ran.refused,).toBe(true,);
        expect(ran.lines.join('\n',),).toContain('no [PASS] suite line',);
      },
    }),
    it({
      name: 'says a type check reports no count of the files it checked',
      fn: async () => {
        /**
         What a passing type check earns.
         */
        const ran = await ranWhatItSaysOf({
          task: '//package/a:lint:types',
          output: '',
          worktree: import.meta.dirname,
        });
        expect(ran.refused,).toBe(false,);
        /**
         The statement's words.
         */
        const statement = ran.lines.join('\n',);
        expect(statement,).toContain('reports no count of the files it checked',);
        // The flag that would report it, which the task does not pass, is named.
        expect(statement,).toContain('--extendedDiagnostics',);
      },
    }),
    it({
      name: 'answers nothing for a task whose status is all its output can say',
      fn: async () => {
        /**
         What a check of anything but a unit run and a type check earns.
         */
        const ran = await ranWhatItSaysOf({
          task: '//package/a:native:clippy',
          output: outputOf({ lines: [suiteLine({ name: 'a', }),], }),
          worktree: import.meta.dirname,
        });
        expect(ran,).toEqual({
          lines: [],
          refused: false,
        },);
      },
    }),
  ],
});
