/**
 Tests for how the census finds the family it measures and reads its builds:
 which sibling packages are members, what their coverage builds hold, which
 member each named test file belongs to, and which scripts of a coverage file
 are bundles. Directories, files and names are cat-themed invention.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  bundleScriptsOf,
  censusScopeOf,
  familySuiteRunsOf,
  isFamilyBundleName,
  readUtf8Text,
  requireFamilyCoverageBuild,
} from '../../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import { UNMINIFIED_BUNDLE, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/coverage-census-package.test-fixture.ts';

/**
 One member of a scratch family.
 */
type MemberFiles = {
  /**
   Whether the member has a coverage build config.
   */
  readonly coverageConfig: boolean;

  /**
   Whether the member has a task file, which its unit suite runs through.
   */
  readonly taskFile: boolean;

  /**
   Text of each file of the member's build directory by name, or none where
   the member has no build.
   */
  readonly built?: Readonly<Record<string, string>>;
};

/**
 Writes a family of members into a directory.

 @param directory - directory to write the members into

 @param members - each member's files by member name

 @example
 ```ts
 await writeFamily({ directory: scratch.path, members: { alpha: { coverageConfig: true, taskFile: true, }, }, },);
 ```
 */
async function writeFamily(
  {
    directory,
    members,
  }: {
    readonly directory: string;
    readonly members: Readonly<Record<string, MemberFiles>>;
  },
): Promise<void> {
  await Promise.all(Object.entries(members,)
    .map(async function written([name, files,],): Promise<void> {
      /**
       The member's directory.
       */
      const memberDirectory = join(
        directory,
        name,
      );
      await mkdir(
        memberDirectory,
        { recursive: true, },
      );
      if (files.coverageConfig)
        await writeFile(
          join(
            memberDirectory,
            'rolldown.coverage.config.ts',
          ),
          '',
          'utf8',
        );
      if (files.taskFile)
        await writeFile(
          join(
            memberDirectory,
            'mise.toml',
          ),
          '',
          'utf8',
        );
      if (files.built === undefined)
        return;
      /**
       The member's build directory.
       */
      const distDirectory = join(
        memberDirectory,
        'dist',
        'final',
        'node',
      );
      await mkdir(
        distDirectory,
        { recursive: true, },
      );
      await Promise.all(Object.entries(files.built,)
        .map(async function builtFile([file, text,],): Promise<void> {
          await writeFile(
            join(
              distDirectory,
              file,
            ),
            text,
            'utf8',
          );
        },),);
    },),);
}

/**
 A family of three members: two with a coverage build and a task file, one
 with a task file alone, as the sources scans' package has.

 @param directory - directory to write the members into

 @example
 ```ts
 await writeCatFamily({ directory: scratch.path, },);
 ```
 */
async function writeCatFamily({ directory, }: { readonly directory: string; },): Promise<void> {
  await writeFamily({
    directory,
    members: {
      alpha: {
        coverageConfig: true,
        taskFile: true,
        built: {
          'nap.mjs': UNMINIFIED_BUNDLE,
          'nap.mjs.map': '{}',
          'index.mjs': 'export {};\n',
        },
      },
      beta: {
        coverageConfig: true,
        taskFile: true,
        built: {
          'bee.mjs': UNMINIFIED_BUNDLE,
          'bee.mjs.map': '{}',
        },
      },
      gamma: {
        coverageConfig: false,
        taskFile: true,
      },
    },
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: censusScopeOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS THE MEMBERS BESIDE A PACKAGE WITH A COVERAGE BUILD CONFIG: those with a config build, those with a '
            + 'task file run their unit suite, each in code-point order',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'coverage-census-family-', },);
            await writeCatFamily({ directory: scratch.path, },);

            expect(await censusScopeOf({
              packageDirectory: join(
                scratch.path,
                'alpha',
              ),
            },),).toEqual({
              kind: 'family',
              family: {
                directory: scratch.path,
                builds: [
                  'alpha',
                  'beta',
                ],
                suites: [
                  'alpha',
                  'beta',
                  'gamma',
                ],
              },
            },);
          },
        },),
        it({
          name: 'FINDS THE DIRECTORY ALONE where the package has no coverage build config, so the census reads that package alone',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'coverage-census-family-', },);
            await writeCatFamily({ directory: scratch.path, },);

            expect(await censusScopeOf({
              packageDirectory: join(
                scratch.path,
                'gamma',
              ),
            },),).toEqual({ kind: 'alone', },);
            expect(await censusScopeOf({ packageDirectory: scratch.path, },),).toEqual({ kind: 'alone', },);
          },
        },),
      ],
    },),
    describe({
      name: requireFamilyCoverageBuild.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS EVERY MEMBER\'S COVERAGE BUILD and names each bundle by its path from the family directory, '
            + 'split by whether a map stands beside it',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'coverage-census-family-', },);
            await writeCatFamily({ directory: scratch.path, },);
            const { logger, } = capturingLoggerPair();

            expect(await requireFamilyCoverageBuild({
              family: {
                directory: scratch.path,
                builds: [
                  'alpha',
                  'beta',
                ],
                suites: [
                  'alpha',
                  'beta',
                  'gamma',
                ],
              },
              l: logger,
              readText: readUtf8Text,
            },),).toEqual({
              mapped: [
                'alpha/dist/final/node/nap.mjs',
                'beta/dist/final/node/bee.mjs',
              ],
              unmapped: ['alpha/dist/final/node/index.mjs',],
            },);
          },
        },),
        it({
          name: 'REFUSES A MEMBER WITH NO BUILD DIRECTORY as the single package does, naming that member\'s directory',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'coverage-census-family-', },);
            await writeCatFamily({ directory: scratch.path, },);
            await writeFamily({
              directory: scratch.path,
              members: {
                delta: {
                  coverageConfig: true,
                  taskFile: true,
                },
              },
            },);
            const { logger, } = capturingLoggerPair();

            /**
             What the read rejected with.
             */
            const refusal = await rejectionOf({
              promise: requireFamilyCoverageBuild({
                family: {
                  directory: scratch.path,
                  builds: [
                    'alpha',
                    'delta',
                  ],
                  suites: [],
                },
                l: logger,
                readText: readUtf8Text,
              },),
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(
              String(refusal,).includes(join(
                scratch.path,
                'delta',
                'dist',
                'final',
                'node',
              ),),
            ).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: familySuiteRunsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RUNS EVERY SUITE MEMBER\'S WHOLE SUITE where no test file is named, each in its own directory',
          fn: async () => {
            expect(familySuiteRunsOf({
              family: {
                directory: '/cats',
                builds: ['alpha',],
                suites: [
                  'alpha',
                  'gamma',
                ],
              },
              testFiles: [],
            },),).toEqual([
              {
                member: 'alpha',
                cwd: '/cats/alpha',
                files: [],
              },
              {
                member: 'gamma',
                cwd: '/cats/gamma',
                files: [],
              },
            ],);
          },
        },),
        it({
          name: 'GIVES EACH NAMED TEST FILE TO THE MEMBER IT STANDS IN, as an absolute path whether it was named from the '
            + 'family directory or absolutely, and runs no other member',
          fn: async () => {
            expect(familySuiteRunsOf({
              family: {
                directory: '/cats',
                builds: ['alpha',],
                suites: [
                  'alpha',
                  'gamma',
                ],
              },
              testFiles: [
                'gamma/src/purr.unit.test.ts',
                '/cats/gamma/src/nap.unit.test.ts',
              ],
            },),).toEqual([{
              member: 'gamma',
              cwd: '/cats/gamma',
              files: [
                '/cats/gamma/src/purr.unit.test.ts',
                '/cats/gamma/src/nap.unit.test.ts',
              ],
            },],);
          },
        },),
        it({
          name: 'REFUSES A TEST FILE OUTSIDE THE FAMILY\'S SUITES as stated, naming the file and the forms it reads',
          fn: async () => {
            /**
             What the grouping threw for a file outside the family.
             */
            let outside: unknown;
            try {
              familySuiteRunsOf({
                family: {
                  directory: '/cats',
                  builds: ['alpha',],
                  suites: ['alpha',],
                },
                testFiles: ['/dogs/src/bark.unit.test.ts',],
              },);
            }
            catch (error) {
              outside = error;
            }
            /**
             What it threw for a file in a directory that runs no suite.
             */
            let stranger: unknown;
            try {
              familySuiteRunsOf({
                family: {
                  directory: '/cats',
                  builds: ['alpha',],
                  suites: ['alpha',],
                },
                testFiles: ['delta/src/purr.unit.test.ts',],
              },);
            }
            catch (error) {
              stranger = error;
            }

            expect(outside,).toBeInstanceOf(StatedRefusalError,);
            expect(String(outside,),).toBe(
              'StatedRefusalError: /dogs/src/bark.unit.test.ts is not a test file of a package beside this one; name it '
                + 'from the family directory (/cats), as <package>/src/<file>.unit.test.ts, or by its absolute path',
            );
            expect(stranger,).toBeInstanceOf(StatedRefusalError,);
            expect(String(stranger,),).toBe(
              'StatedRefusalError: /cats/delta/src/purr.unit.test.ts is not a test file of a package beside this one; '
                + 'name it from the family directory (/cats), as <package>/src/<file>.unit.test.ts, or by its absolute path',
            );
          },
        },),
      ],
    },),
    describe({
      name: isFamilyBundleName.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS A FILE OF A MEMBER\'S BUILD DIRECTORY and nothing else: not a source, not a file deeper or shallower, '
            + 'not a build directory with no member',
          fn: async () => {
            expect(isFamilyBundleName('alpha/dist/final/node/nap.mjs',),).toBe(true,);
            expect(isFamilyBundleName('alpha/src/nap.ts',),).toBe(false,);
            expect(isFamilyBundleName('alpha/dist/final/node/deeper/nap.mjs',),).toBe(false,);
            expect(isFamilyBundleName('alpha/dist/final/nap.mjs',),).toBe(false,);
            expect(isFamilyBundleName('/dist/final/node/nap.mjs',),).toBe(false,);
            expect(isFamilyBundleName('alpha/dist/final/node/',),).toBe(false,);
            expect(isFamilyBundleName('alpha/dist/other/node/nap.mjs',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: bundleScriptsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS ONLY THE SCRIPTS A BUNDLE PATTERN ACCEPTS, so a source file a test loaded from the family directory '
            + 'is not read as a bundle',
          fn: async () => {
            /**
             A coverage file of one process that loaded a bundle and a source file of the family.
             */
            const text = JSON.stringify({
              result: [
                {
                  url: 'file:///cats/alpha/dist/final/node/nap.mjs',
                  functions: [{
                    functionName: 'nap',
                    ranges: [{
                      startOffset: 0,
                      endOffset: 4,
                      count: 1,
                    },],
                  },],
                },
                {
                  url: 'file:///cats/alpha/src/nap.test-fixture.ts',
                  functions: [{
                    functionName: 'fixture',
                    ranges: [{
                      startOffset: 0,
                      endOffset: 4,
                      count: 1,
                    },],
                  },],
                },
              ],
            },);

            expect(bundleScriptsOf({
              path: '/raw/coverage-1.json',
              text,
              bundleUrlPrefix: 'file:///cats/',
              isBundleName: isFamilyBundleName,
            },).map(function bundleOf({ bundle, },): string {
              return bundle;
            },),).toEqual(['alpha/dist/final/node/nap.mjs',],);
            expect(bundleScriptsOf({
              path: '/raw/coverage-1.json',
              text,
              bundleUrlPrefix: 'file:///cats/',
            },).map(function bundleOf({ bundle, },): string {
              return bundle;
            },),).toEqual([
              'alpha/dist/final/node/nap.mjs',
              'alpha/src/nap.test-fixture.ts',
            ],);
          },
        },),
      ],
    },),
  ],
},);
