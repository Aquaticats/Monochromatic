/**
 Tests for the census command's procedure over a family of packages, with the
 steps that touch git, the suite, the coverage and the build handed in and
 scripted: which suites it runs and where, what it reads of every member's
 build before it spends a suite, how it asks for the coverage, and what it
 refuses. No case runs a suite. Directories and names are cat-themed
 invention.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';
import { pathToFileURL, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  type CensusFamily,
  type CoverageCensusSteps,
  createCoverageTally,
  isFamilyBundleName,
  type PlacedTally,
  runCoverageCensus,
} from '../../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { UNMINIFIED_BUNDLE, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/coverage-census-package.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';

/**
 What a scripted suite answers.
 */
type SuiteAnswer = {
  readonly passes: number;
  readonly failures: number;
  readonly exitCode: number;
};

/**
 What the scripted steps were asked.
 */
type FamilyCalls = {
  readonly commits: string[];
  readonly suites: {
    readonly command: readonly string[];
    readonly cwd: string;
    readonly coverageDirectory: string;
    readonly logPath: string;
  }[];
  readonly tallies: {
    readonly coverageDirectory: string;
    readonly bundleUrlPrefix: string;
    readonly isBundleName?: (bundle: string) => boolean;
  }[];
  readonly placements: {
    readonly packageDirectory: string;
    readonly distDirectory: string;
    readonly mapped: readonly string[];
    readonly unmapped: readonly string[];
  }[];
  readonly edits: string[];
};

/**
 A family of two members with a build and a third with a task file alone,
 written into a directory, and the family as the census finds it.

 @param directory - directory to write the members into

 @returns Family

 @example
 ```ts
 const family = await writeCatFamily({ directory: scratch.path, },);
 ```
 */
async function writeCatFamily({ directory, }: { readonly directory: string; },): Promise<CensusFamily> {
  await Promise.all([
    [
      'alpha',
      'nap.mjs',
    ],
    [
      'beta',
      'bee.mjs',
    ],
  ].map(async function member([name, bundle,],): Promise<void> {
    /**
     The member's build directory.
     */
    const distDirectory = join(
      directory,
      name ?? '',
      'dist',
      'final',
      'node',
    );
    await mkdir(
      distDirectory,
      { recursive: true, },
    );
    await writeFile(
      join(
        distDirectory,
        bundle ?? '',
      ),
      UNMINIFIED_BUNDLE,
      'utf8',
    );
    await writeFile(
      join(
        distDirectory,
        `${bundle ?? ''}.map`,
      ),
      '{}',
      'utf8',
    );
  },),);
  await mkdir(
    join(
      directory,
      'gamma',
    ),
    { recursive: true, },
  );
  return {
    directory,
    builds: [
      'alpha',
      'beta',
    ],
    suites: [
      'alpha',
      'beta',
      'gamma',
    ],
  };
}

/**
 Steps that answer each member's suite as scripted, recording what each was asked.

 @param answers - what the suite of each member answers by member name; a
 member not named answers a clean suite of two passes

 @returns Steps and calls they record

 @example
 ```ts
 const { steps, calls, } = scriptedFamilySteps({ answers: { beta: { passes: 0, failures: 1, exitCode: 1, }, }, },);
 ```
 */
function scriptedFamilySteps(
  { answers, }: { readonly answers: Readonly<Record<string, SuiteAnswer>>; },
): {
  readonly steps: CoverageCensusSteps;
  readonly calls: FamilyCalls;
} {
  /**
   Everything the steps were asked.
   */
  const calls: FamilyCalls = {
    commits: [],
    suites: [],
    tallies: [],
    placements: [],
    edits: [],
  };
  return {
    calls,
    steps: {
      packageCommit: async function committed({ packageDirectory, },): Promise<{
        readonly head: string;
        readonly clean: boolean;
      }> {
        calls.commits.push(packageDirectory,);
        return {
          head: 'abc123def',
          clean: true,
        };
      },
      runSuite: async function ranSuite(input,): Promise<SuiteAnswer> {
        calls.suites.push(input,);
        return answers[basename(input.cwd,)] ?? {
          passes: 2,
          failures: 0,
          exitCode: 0,
        };
      },
      tallyCoverage: async function tallied(input,): Promise<ReturnType<typeof createCoverageTally>> {
        calls.tallies.push(input,);
        return createCoverageTally();
      },
      placeTally: async function placed(
        {
          packageDirectory,
          distDirectory,
          bundleMaps,
        },
      ): Promise<PlacedTally> {
        calls.placements.push({
          packageDirectory,
          distDirectory,
          mapped: bundleMaps.mapped,
          unmapped: bundleMaps.unmapped,
        },);
        return {
          stretches: [],
          invariantThrows: [],
          uncalled: [],
          loadedSources: new Set(['alpha/src/nap.ts',],),
          unloadedBundles: [],
          unloadedSources: [],
        };
      },
      sourcesEditedSince: async function edited({ head, },): Promise<ReadonlySet<string>> {
        calls.edits.push(head,);
        return new Set<string>();
      },
    },
  };
}

await describe({
  name: `${runCoverageCensus.name} over a family`,
  children: [
    it({
      name: 'RUNS EACH SUITE MEMBER\'S SUITE IN ITS OWN DIRECTORY under one coverage directory with a log apiece, asks '
        + 'git about the family directory, and reads every member\'s bundles through one tally',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'coverage-census-family-run-', },);
        const family = await writeCatFamily({ directory: scratch.path, },);
        const { logger, } = capturingLoggerPair();
        const { steps, calls, } = scriptedFamilySteps({ answers: {}, },);

        await runCoverageCensus({
          line: lineOf({ command: 'coverage-census', typed: [], },),
          packageDirectory: join(
            scratch.path,
            'alpha',
          ),
          family,
          env: {
            XDG_CACHE_HOME: join(
              scratch.path,
              'cache',
            ),
          },
          steps,
          l: logger,
        },);

        expect(calls.suites.map(function shown({ command, cwd, },): unknown {
          return {
            command,
            cwd,
          };
        },),).toEqual([
          {
            command: [
              'mise',
              'run',
              'test:unit',
            ],
            cwd: join(
              scratch.path,
              'alpha',
            ),
          },
          {
            command: [
              'mise',
              'run',
              'test:unit',
            ],
            cwd: join(
              scratch.path,
              'beta',
            ),
          },
          {
            command: [
              'mise',
              'run',
              'test:unit',
            ],
            cwd: join(
              scratch.path,
              'gamma',
            ),
          },
        ],);
        expect(calls.suites.map(function logOf({ logPath, },): string {
          return basename(logPath,);
        },),).toEqual([
          'suite-alpha.log',
          'suite-beta.log',
          'suite-gamma.log',
        ],);
        expect(new Set(calls.suites.map(function coverageOf({ coverageDirectory, },): string {
          return coverageDirectory;
        },),).size,).toBe(1,);
        expect(calls.commits,).toEqual([scratch.path,],);
        expect(calls.tallies.map(function prefixOf({ bundleUrlPrefix, isBundleName, },): unknown {
          return {
            bundleUrlPrefix,
            isBundleName,
          };
        },),).toEqual([{
          bundleUrlPrefix: `${pathToFileURL(scratch.path,).href}/`,
          isBundleName: isFamilyBundleName,
        },],);
        expect(calls.placements,).toEqual([{
          packageDirectory: scratch.path,
          distDirectory: scratch.path,
          mapped: [
            'alpha/dist/final/node/nap.mjs',
            'beta/dist/final/node/bee.mjs',
          ],
          unmapped: [],
        },],);
        expect(printed.lines[0],).toBe('coverage-census at abc123def: the unit suite, 6 passes',);
      },
    },),
    it({
      name: 'RUNS ONLY THE MEMBER A NAMED TEST FILE STANDS IN, handing it the file by its absolute path',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'coverage-census-family-run-', },);
        const family = await writeCatFamily({ directory: scratch.path, },);
        const { logger, } = capturingLoggerPair();
        const { steps, calls, } = scriptedFamilySteps({ answers: {}, },);

        await runCoverageCensus({
          line: lineOf({
            command: 'coverage-census',
            typed: ['beta/src/bee.unit.test.ts',],
          },),
          packageDirectory: join(
            scratch.path,
            'alpha',
          ),
          family,
          env: {
            XDG_CACHE_HOME: join(
              scratch.path,
              'cache',
            ),
          },
          steps,
          l: logger,
        },);

        expect(calls.suites.map(function shown({ command, cwd, },): unknown {
          return {
            command,
            cwd,
          };
        },),).toEqual([{
          command: [
            'mise',
            'run',
            'test:unit',
            join(
              scratch.path,
              'beta',
              'src',
              'bee.unit.test.ts',
            ),
          ],
          cwd: join(
            scratch.path,
            'beta',
          ),
        },],);
        expect(printed.lines[0],).toBe('coverage-census at abc123def: 1 test file, 2 passes',);
      },
    },),
    it({
      name: 'REFUSES AT THE FIRST MEMBER WHOSE SUITE FAILED, naming it and its log, and runs no later member',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'coverage-census-family-run-', },);
        const family = await writeCatFamily({ directory: scratch.path, },);
        const { logger, } = capturingLoggerPair();
        const { steps, calls, } = scriptedFamilySteps({
          answers: {
            alpha: {
              passes: 1,
              failures: 1,
              exitCode: 1,
            },
          },
        },);

        /**
         What the census rejected with.
         */
        const refusal = await rejectionOf({
          promise: runCoverageCensus({
            line: lineOf({ command: 'coverage-census', typed: [], },),
            packageDirectory: join(
              scratch.path,
              'alpha',
            ),
            family,
            env: {
              XDG_CACHE_HOME: join(
                scratch.path,
                'cache',
              ),
            },
            steps,
            l: logger,
          },),
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: the suite of alpha failed (exit 1, 1 FAIL marker); its log is ${calls.suites[0]?.logPath ?? ''}; `
            + 'a census of a failing suite would count code its failing tests skipped',
        );
        expect(calls.suites.length,).toBe(1,);
        expect(calls.tallies,).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES A FAMILY WHOSE SUITES PRINTED NO PASS MARKER between them, since it ran no test',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'coverage-census-family-run-', },);
        const family = await writeCatFamily({ directory: scratch.path, },);
        const { logger, } = capturingLoggerPair();
        const quiet = {
          passes: 0,
          failures: 0,
          exitCode: 0,
        };
        const { steps, calls, } = scriptedFamilySteps({
          answers: {
            alpha: quiet,
            beta: quiet,
            gamma: quiet,
          },
        },);

        /**
         What the census rejected with.
         */
        const refusal = await rejectionOf({
          promise: runCoverageCensus({
            line: lineOf({ command: 'coverage-census', typed: [], },),
            packageDirectory: join(
              scratch.path,
              'alpha',
            ),
            family,
            env: {
              XDG_CACHE_HOME: join(
                scratch.path,
                'cache',
              ),
            },
            steps,
            l: logger,
          },),
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,).startsWith(
          'StatedRefusalError: the suite printed no PASS marker, so it ran no test; its logs are in ',
        ),).toBe(true,);
        expect(calls.suites.length,).toBe(3,);
        expect(calls.tallies,).toEqual([],);
      },
    },),
    it({
      name: 'READS EVERY MEMBER\'S BUILD BEFORE IT SPENDS A SUITE, refusing the member that has none',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'coverage-census-family-run-', },);
        const family = await writeCatFamily({ directory: scratch.path, },);
        await mkdir(
          join(
            scratch.path,
            'delta',
          ),
          { recursive: true, },
        );
        const { logger, } = capturingLoggerPair();
        const { steps, calls, } = scriptedFamilySteps({ answers: {}, },);

        /**
         What the census rejected with.
         */
        const refusal = await rejectionOf({
          promise: runCoverageCensus({
            line: lineOf({ command: 'coverage-census', typed: [], },),
            packageDirectory: join(
              scratch.path,
              'alpha',
            ),
            family: {
              ...family,
              builds: [
                'alpha',
                'delta',
              ],
            },
            env: {
              XDG_CACHE_HOME: join(
                scratch.path,
                'cache',
              ),
            },
            steps,
            l: logger,
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
        expect(calls.suites,).toEqual([],);
      },
    },),
  ],
},);
