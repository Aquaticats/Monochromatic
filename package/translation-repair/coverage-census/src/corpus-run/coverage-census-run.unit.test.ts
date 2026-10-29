/**
 Tests for the census command's procedure, with the steps that touch git, the
 suite, the coverage and the build handed in and scripted: what it reads
 before it spends a suite, what it asks the suite to run, where it keeps the
 census, and every refusal. No case runs a suite, so none builds or runs the
 package's own tests inside a test. Paths and names are cat-themed invention.

 @module
 */

import {
  access,
  readdir,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import { pathToFileURL, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { isMissingPathError, } from '@monochromatic-dev/translation-repair-typography';
import {
  CensusBaselineError,
  type CoverageCensusSteps,
  createCoverageTally,
  type PlacedTally,
  type readBaselineFile,
  readBaselines,
  runCoverageCensus,
} from '../../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { refusalOrder, } from '@monochromatic-dev/translation-repair-test-fixture/ts/refusal-order.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  makeBuiltPackage,
  UNMINIFIED_BUNDLE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/coverage-census-package.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';

/**
 What the scripted steps were asked, in order, by step.
 */
type StepCalls = {
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
  }[];
  readonly placements: string[];
  readonly edits: string[];
};

/**
 Steps that answer as a clean suite of three passes over a tree matching its
 commit, recording what each was asked.

 @param suite - what the scripted suite answers

 @param head - commit the scripted git names

 @param clean - whether the scripted git says the files match it

 @returns Steps and calls they record

 @example
 ```ts
 const { steps, calls, } = scriptedSteps({ suite: { passes: 3, failures: 0, exitCode: 0, }, head: 'abc123def', clean: true, },);
 ```
 */
function scriptedSteps(
  {
    suite,
    head,
    clean,
  }: {
    readonly suite: { readonly passes: number; readonly failures: number; readonly exitCode: number; };
    readonly head: string;
    readonly clean: boolean;
  },
): {
  readonly steps: CoverageCensusSteps;
  readonly calls: StepCalls;
} {
  /**
   Everything the steps were asked.
   */
  const calls: StepCalls = {
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
          head,
          clean,
        };
      },
      runSuite: async function ranSuite(input,): Promise<typeof suite> {
        calls.suites.push(input,);
        return suite;
      },
      tallyCoverage: async function tallied(input,): Promise<ReturnType<typeof createCoverageTally>> {
        calls.tallies.push(input,);
        return createCoverageTally();
      },
      placeTally: async function placed({ distDirectory, },): Promise<PlacedTally> {
        calls.placements.push(distDirectory,);
        return {
          stretches: [],
          invariantThrows: [],
          uncalled: [],
          loadedSources: new Set(['src/nap.ts',],),
          unloadedBundles: [],
          unloadedSources: [],
        };
      },
      sourcesEditedSince: async function edited({ head: since, },): Promise<ReadonlySet<string>> {
        calls.edits.push(since,);
        return new Set<string>();
      },
    },
  };
}

/**
 Whether anything stands at a path.

 @param path - path to look at

 @returns Whether it is there

 @example
 ```ts
 const there = await isPresent({ path: built.path, },);
 ```
 */
async function isPresent({ path, }: { readonly path: string; },): Promise<boolean> {
  try {
    await access(path,);
  }
  catch (error) {
    if (!isMissingPathError({ error, },))
      throw error;
    return false;
  }
  return true;
}

/**
 A package directory whose build the census can read.

 @returns Package, with one unminified bundle and its map

 @example
 ```ts
 await using built = await readablePackage();
 ```
 */
async function readablePackage(): ReturnType<typeof makeBuiltPackage> {
  return await makeBuiltPackage({
    files: {
      'nap.mjs': UNMINIFIED_BUNDLE,
      'nap.mjs.map': '{}',
    },
  },);
}

await describe({
  name: runCoverageCensus.name,
  children: [
    it({
      name: 'ASKS THE SUITE TO RUN THE NAMED TEST FILES under coverage in the package directory, keeps the census and '
        + 'the suite log under the cache directory and removes the raw coverage when it is done',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using built = await readablePackage();
        const { logger, lines, } = capturingLoggerPair();
        const { steps, calls, } = scriptedSteps({
          suite: {
            passes: 3,
            failures: 0,
            exitCode: 0,
          },
          head: 'abc123def',
          clean: true,
        },);
        await runCoverageCensus({
          line: lineOf({
            command: 'coverage-census',
            typed: ['src/nap.unit.test.ts',],
          },),
          packageDirectory: built.path,
          env: {
            XDG_CACHE_HOME: join(
              built.path,
              'cache',
            ),
          },
          steps,
          l: logger,
        },);

        /**
         The one suite run asked for.
         */
        const [suite,] = calls.suites;
        expect(calls.suites.length,).toBe(1,);
        expect({
          command: suite?.command,
          cwd: suite?.cwd,
        },).toEqual({
          command: [
            'mise',
            'run',
            'test:unit',
            'src/nap.unit.test.ts',
          ],
          cwd: built.path,
        },);

        /**
         Where the coverage, the census and the log were kept.
         */
        const coverageHome = join(
          built.path,
          'cache',
          'translation-repair',
          'coverage',
        );
        const reportDirectory = dirname(suite?.logPath ?? '',);
        expect(dirname(reportDirectory,),).toBe(coverageHome,);
        expect(dirname(suite?.coverageDirectory ?? '',),).toBe(coverageHome,);
        expect(suite?.logPath,).toBe(join(
          reportDirectory,
          'suite.log',
        ),);
        expect(await readdir(coverageHome,),).toEqual([reportDirectory.slice(coverageHome.length + 1,),],);
        expect(await readdir(reportDirectory,),).toEqual(['census.json',],);
        expect(calls.tallies,).toEqual([{
          coverageDirectory: suite?.coverageDirectory,
          bundleUrlPrefix: `${pathToFileURL(join(
            built.path,
            'dist',
            'final',
            'node',
          ),)
            .href}/`,
        },],);
        expect({
          commits: calls.commits,
          placements: calls.placements,
          edits: calls.edits,
        },).toEqual({
          commits: [built.path,],
          placements: [join(
            built.path,
            'dist',
            'final',
            'node',
          ),],
          edits: [],
        },);
        expect(lines,).toEqual([
          'bundles with no source map, read only where the census must place code in them: none',
          `at abc123def; coverage in ${suite?.coverageDirectory}, report in ${reportDirectory}`,
        ],);
        expect(printed.lines[0],).toBe('coverage-census at abc123def: 1 test file, 3 passes',);
        expect(printed.lines.at(-1,),).toBe(`census written to ${join(
          reportDirectory,
          'census.json',
        )}`,);
      },
    },),
    it({
      name: 'ASKS FOR THE WHOLE UNIT SUITE where no test file is named, and says the tree has uncommitted changes '
        + 'where git says the files do not match the commit',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using built = await readablePackage();
        const { logger, lines, } = capturingLoggerPair();
        const { steps, calls, } = scriptedSteps({
          suite: {
            passes: 1,
            failures: 0,
            exitCode: 0,
          },
          head: 'abc123def',
          clean: false,
        },);
        await runCoverageCensus({
          line: lineOf({
            command: 'coverage-census',
            typed: [],
          },),
          packageDirectory: built.path,
          env: {
            XDG_CACHE_HOME: join(
              built.path,
              'cache',
            ),
          },
          steps,
          l: logger,
        },);
        expect(calls.suites.map(function commandOf({ command, },): readonly string[] {
          return command;
        },),).toEqual([[
          'mise',
          'run',
          'test:unit',
        ],],);
        /**
         The one suite run asked for.
         */
        const [suite,] = calls.suites;
        expect(lines[1],).toBe(
          `at abc123def with uncommitted changes; coverage in ${suite?.coverageDirectory}, report in ${dirname(suite?.logPath ?? '',)}`,
        );
        expect(printed.lines[0],).toBe('coverage-census at abc123def with uncommitted changes: the unit suite, 1 pass',);
      },
    },),
    it({
      name: 'READS EACH BASELINE BEFORE THE SUITE and prints its reading after the report, asking git for the '
        + 'baseline\'s commit',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using built = await readablePackage();

        /**
         An earlier census this command wrote, clean, with one cold stretch.
         */
        const baseline = join(
          built.path,
          'before.json',
        );
        await writeFile(
          baseline,
          JSON.stringify({
            format: 3,
            head: 'c0ffee123',
            clean: true,
            stretches: [{
              bundle: 'nap.mjs',
              start: 0,
              end: 1,
              name: '',
              source: 'src/nap.ts',
              startLine: 3,
              endLine: 5,
            },],
            invariantThrows: [],
            loadedSources: ['src/nap.ts',],
          },),
          'utf8',
        );
        const { logger, } = capturingLoggerPair();
        const { steps, calls, } = scriptedSteps({
          suite: {
            passes: 3,
            failures: 0,
            exitCode: 0,
          },
          head: 'abc123def',
          clean: true,
        },);
        await runCoverageCensus({
          line: lineOf({
            command: 'coverage-census',
            typed: [
              '--baseline',
              baseline,
              '--source',
              'src/nap.ts',
              'src/nap.unit.test.ts',
            ],
          },),
          packageDirectory: built.path,
          env: {
            XDG_CACHE_HOME: join(
              built.path,
              'cache',
            ),
          },
          steps,
          l: logger,
        },);
        expect(calls.edits,).toEqual(['c0ffee123',],);
        expect(printed.lines.at(-1,),).toBe(
          `against ${baseline} at c0ffee123: ran 1, still cold 0, cold since then 0, not loaded 0, claimed sources `
            + 'with no stretch there 0, sources edited since then 0',
        );
      },
    },),
    it({
      name: 'REFUSES --source WITH NO --baseline before it reads the build or asks anything of the steps',
      fn: async () => {
        await using built = await readablePackage();
        const { logger, } = capturingLoggerPair();
        const { steps, calls, } = scriptedSteps({
          suite: {
            passes: 1,
            failures: 0,
            exitCode: 0,
          },
          head: 'abc123def',
          clean: true,
        },);
        try {
          await runCoverageCensus({
            line: lineOf({
              command: 'coverage-census',
              typed: [
                '--source',
                'src/nap.ts',
              ],
            },),
            packageDirectory: built.path,
            env: {},
            steps,
            l: logger,
          },);
        }
        catch (error) {
          expect(error,).toBeInstanceOf(StatedRefusalError,);
          expect(String(error,),).toBe(
            'StatedRefusalError: --source picks the baseline stretches to read, so it needs --baseline '
              + '<census.json> beside it',
          );
          expect(calls,).toEqual({
            commits: [],
            suites: [],
            tallies: [],
            placements: [],
            edits: [],
          },);
          return;
        }
        throw new Error('the census ran with a --source and no baseline',);
      },
    },),
    it({
      name: 'REFUSES A BASELINE IT CANNOT READ before the suite, so no suite is spent and no cache directory is made',
      fn: async () => {
        await using built = await readablePackage();
        const { logger, } = capturingLoggerPair();
        const { steps, calls, } = scriptedSteps({
          suite: {
            passes: 1,
            failures: 0,
            exitCode: 0,
          },
          head: 'abc123def',
          clean: true,
        },);

        /**
         A baseline path nothing is at.
         */
        const baseline = join(
          built.path,
          'no-such-census.json',
        );
        try {
          await runCoverageCensus({
            line: lineOf({
              command: 'coverage-census',
              typed: [
                '--baseline',
                baseline,
              ],
            },),
            packageDirectory: built.path,
            env: {
              XDG_CACHE_HOME: join(
                built.path,
                'cache',
              ),
            },
            steps,
            l: logger,
          },);
        }
        catch (error) {
          expect(String(error,),).toBe(
            `CensusBaselineError: baseline ${baseline} does not read as a census this command wrote: it could not `
              + 'be read (ENOENT)',
          );
          expect({
            calls,
            cacheMade: await isPresent({
              path: join(
                built.path,
                'cache',
              ),
            },),
          },).toEqual({
            calls: {
              commits: [],
              suites: [],
              tallies: [],
              placements: [],
              edits: [],
            },
            cacheMade: false,
          },);
          return;
        }
        throw new Error('the census went on past a baseline it could not read',);
      },
    },),
    it({
      name: 'REFUSES A BUILD WITH NO BUNDLE before it asks git or the suite anything',
      fn: async () => {
        await using built = await makeBuiltPackage({ files: {}, },);
        const { logger, } = capturingLoggerPair();
        const { steps, calls, } = scriptedSteps({
          suite: {
            passes: 1,
            failures: 0,
            exitCode: 0,
          },
          head: 'abc123def',
          clean: true,
        },);
        try {
          await runCoverageCensus({
            line: lineOf({
              command: 'coverage-census',
              typed: [],
            },),
            packageDirectory: built.path,
            env: {},
            steps,
            l: logger,
          },);
        }
        catch (error) {
          expect(String(error,),).toBe(
            `StatedRefusalError: ${built.distDirectory} holds no bundle, so no build has written it; run the `
              + 'census through mise run //package/translation-repair/coverage-census:coverage-census, which builds with '
              + 'maps first',
          );
          expect(calls.commits,).toEqual([],);
          return;
        }
        throw new Error('the census ran on a build with no bundle',);
      },
    },),
    it({
      name: 'REFUSES A SUITE THAT EXITED WITH A FAILURE as stated, naming its exit code, its log and why a failing suite is not counted, reading nothing of its coverage and removing the raw coverage',
      fn: async () => {
        await using built = await readablePackage();
        const { logger, } = capturingLoggerPair();
        const { steps, calls, } = scriptedSteps({
          suite: {
            passes: 3,
            failures: 0,
            exitCode: 1,
          },
          head: 'abc123def',
          clean: true,
        },);
        try {
          await runCoverageCensus({
            line: lineOf({
              command: 'coverage-census',
              typed: [],
            },),
            packageDirectory: built.path,
            env: {
              XDG_CACHE_HOME: join(
                built.path,
                'cache',
              ),
            },
            steps,
            l: logger,
          },);
        }
        catch (error) {
          expect(error,).toBeInstanceOf(StatedRefusalError,);

          /**
           Where the scripted suite was told to write its log.
           */
          const logPath = calls.suites[0]?.logPath ?? '';
          expect(String(error,),).toBe(
            `StatedRefusalError: the suite failed (exit 1, 0 FAIL markers); its log is ${logPath}; a census of a `
              + 'failing suite would count code its failing tests skipped',
          );
          expect({
            tallies: calls.tallies,
            placements: calls.placements,
            kept: (await readdir(join(
              built.path,
              'cache',
              'translation-repair',
              'coverage',
            ),)).length,
          },).toEqual({
            tallies: [],
            placements: [],
            kept: 1,
          },);
          return;
        }
        throw new Error('the census counted a suite that failed',);
      },
    },),
    it({
      name: 'REFUSES A SUITE THAT EXITED CLEAN BUT PRINTED ONE FAIL MARKER, saying "1 FAIL marker" in the singular',
      fn: async () => {
        await using built = await readablePackage();
        const { logger, } = capturingLoggerPair();
        const { steps, calls, } = scriptedSteps({
          suite: {
            passes: 3,
            failures: 1,
            exitCode: 0,
          },
          head: 'abc123def',
          clean: true,
        },);
        try {
          await runCoverageCensus({
            line: lineOf({
              command: 'coverage-census',
              typed: [],
            },),
            packageDirectory: built.path,
            env: {
              XDG_CACHE_HOME: join(
                built.path,
                'cache',
              ),
            },
            steps,
            l: logger,
          },);
        }
        catch (error) {
          expect(error,).toBeInstanceOf(StatedRefusalError,);

          /**
           Where the scripted suite was told to write its log.
           */
          const logPath = calls.suites[0]?.logPath ?? '';
          expect(String(error,),).toBe(
            `StatedRefusalError: the suite failed (exit 0, 1 FAIL marker); its log is ${logPath}; a census of a `
              + 'failing suite would count code its failing tests skipped',
          );
          expect({
            tallies: calls.tallies,
            placements: calls.placements,
            kept: (await readdir(join(
              built.path,
              'cache',
              'translation-repair',
              'coverage',
            ),)).length,
          },).toEqual({
            tallies: [],
            placements: [],
            kept: 1,
          },);
          return;
        }
        throw new Error('the census counted a suite that failed',);
      },
    },),
    it({
      name: 'REFUSES A SUITE THAT PRINTED SEVERAL FAIL MARKERS, saying "FAIL markers" in the plural',
      fn: async () => {
        await using built = await readablePackage();
        const { logger, } = capturingLoggerPair();
        const { steps, calls, } = scriptedSteps({
          suite: {
            passes: 3,
            failures: 2,
            exitCode: 1,
          },
          head: 'abc123def',
          clean: true,
        },);
        try {
          await runCoverageCensus({
            line: lineOf({
              command: 'coverage-census',
              typed: [],
            },),
            packageDirectory: built.path,
            env: {
              XDG_CACHE_HOME: join(
                built.path,
                'cache',
              ),
            },
            steps,
            l: logger,
          },);
        }
        catch (error) {
          expect(error,).toBeInstanceOf(StatedRefusalError,);

          /**
           Where the scripted suite was told to write its log.
           */
          const logPath = calls.suites[0]?.logPath ?? '';
          expect(String(error,),).toBe(
            `StatedRefusalError: the suite failed (exit 1, 2 FAIL markers); its log is ${logPath}; a census of a `
              + 'failing suite would count code its failing tests skipped',
          );
          expect({
            tallies: calls.tallies,
            placements: calls.placements,
            kept: (await readdir(join(
              built.path,
              'cache',
              'translation-repair',
              'coverage',
            ),)).length,
          },).toEqual({
            tallies: [],
            placements: [],
            kept: 1,
          },);
          return;
        }
        throw new Error('the census counted a suite that failed',);
      },
    },),
    it({
      name: 'REFUSES A SUITE THAT EXITED CLEAN WITH NO PASS MARKER as stated, naming its log and why a suite that ran '
        + 'no test is not counted, reading nothing of its coverage, rather than printing a census of nothing',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using built = await readablePackage();
        const { logger, } = capturingLoggerPair();
        const { steps, calls, } = scriptedSteps({
          suite: {
            passes: 0,
            failures: 0,
            exitCode: 0,
          },
          head: 'abc123def',
          clean: true,
        },);
        try {
          await runCoverageCensus({
            line: lineOf({
              command: 'coverage-census',
              typed: [],
            },),
            packageDirectory: built.path,
            env: {
              XDG_CACHE_HOME: join(
                built.path,
                'cache',
              ),
            },
            steps,
            l: logger,
          },);
        }
        catch (error) {
          expect(error,).toBeInstanceOf(StatedRefusalError,);

          /**
           Where the scripted suite was told to write its log.
           */
          const logPath = calls.suites[0]?.logPath ?? '';
          expect(String(error,),).toBe(
            `StatedRefusalError: the suite printed no PASS marker, so it ran no test; its log is ${logPath}; a `
              + 'census of a suite that ran nothing would count every line of the package as code no test ran',
          );
          expect({
            tallies: calls.tallies,
            placements: calls.placements,
            printed: printed.lines,
            kept: (await readdir(join(
              built.path,
              'cache',
              'translation-repair',
              'coverage',
            ),)).length,
          },).toEqual({
            tallies: [],
            placements: [],
            printed: [],
            kept: 1,
          },);
          return;
        }
        throw new Error(`the census counted a suite that ran no test, printing: ${printed.lines.join(' | ',)}`,);
      },
    },),
    it({
      name: 'REFUSES with the first named baseline\'s refusal when two baselines cannot be read and the second '
        + 'baseline\'s read is refused first',
      fn: async () => {
        /**
         The two refusals the baseline reads end in.
         */
        const { refuseAtOnce, refuseAfterThat, } = refusalOrder();
        /**
         What the reading refused with.
         */
        const refusal = await rejectionOf({
          promise: readBaselines({
            paths: ['/cats/before.json', '/cats/older.json',],
            readBaseline: async function refusesSecondFirst({ path, },): ReturnType<typeof readBaselineFile> {
              /**
               What the read of this file refuses with.
               */
              const unreadable = new CensusBaselineError({
                path,
                says: 'it could not be read (ENOENT)',
                cause: new Error('the cat basket is empty',),
              },);
              return await ((path === '/cats/before.json') ? refuseAfterThat(unreadable,) : refuseAtOnce(unreadable,));
            },
          },),
        },);
        expect(String(refusal,),).toBe(
          'CensusBaselineError: baseline /cats/before.json does not read as a census this command wrote: it could '
            + 'not be read (ENOENT)',
        );
      },
    },),
  ],
},);
