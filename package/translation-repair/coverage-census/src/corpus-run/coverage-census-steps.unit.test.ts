/**
 Tests the coverage census's steps that touch files and processes (ledger
 T8), each against a disposable directory: a command run under coverage with
 its markers and exit code read from its log, the coverage directory read
 twice in one order, and a bundle's map read with its sources named from the
 package and its text kept. What git says of a throwaway repository is
 `coverage-census-commit.unit.test.ts`. Names are cat-themed invention.

 @module
 */

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CoverageFileError,
  coverageReadings,
  readBundle,
  readUtf8Text,
  runSuite,
  SourceMapFileError,
  sourceLineAt,
  tallyCoverage,
  unloadedSourcesOf,
} from '../../dist/final/node/index.mjs';
import { refusalOrder, } from '@monochromatic-dev/translation-repair-test-fixture/ts/refusal-order.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';

/**
 Start of each case's throwaway directory name, so a leftover shows this
 file made it; each case binds its own `scratchDir` with `await using`.
 */
const SCRATCH_PREFIX = 'translation-repair-census-test-';

/**
 A coverage file of one process loading `nap.mjs` from the given build.

 @param prefix - build directory URL with a trailing slash

 @param napRanges - ranges of function nap as `[start, end, count]`, whole extent first

 @returns File's text
 */
function coverageText({
  prefix,
  napRanges,
}: {
  readonly prefix: string;
  readonly napRanges: readonly (readonly [number, number, number])[];
},): string {
  return JSON.stringify({
    result: [{
      url: `${prefix}nap.mjs`,
      functions: [
        {
          functionName: '',
          ranges: [{
            startOffset: 0,
            endOffset: 100,
            count: 1,
          },],
        },
        {
          functionName: 'nap',
          ranges: napRanges.map(([startOffset, endOffset, count,],) => ({
            startOffset,
            endOffset,
            count,
          })),
        },
      ],
    },],
  },);
}

/**
 Build directory URL the fixture coverage names.
 */
const PREFIX = 'file:///cattery/dist/final/node/';

/**
 Stand-in command text that prints a marker the runner writes, built from
 two halves when the stand-in runs. A marker written whole in the command
 would reach this suite's own log through the warning `runSuite` prints on a
 failed command, and a census of the whole suite would count it as a
 failing test and refuse to run.

 @param marker - marker to print, `PASS` or `FAIL`

 @param label - text after it

 @returns Statement printing `[<marker>] <label>`
 */
function printMarker({
  marker,
  label,
}: {
  readonly marker: 'FAIL' | 'PASS';
  readonly label: string;
},): string {
  return `console.log("[" + "${marker}] ${label}");`;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runSuite.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS THE MARKERS A PASSING COMMAND PRINTED, with the coverage directory handed to it',
          fn: async () => {
            await using directory = await scratchDir({ prefix: SCRATCH_PREFIX, },);
            const coverageDirectory = join(
              directory.path,
              'coverage',
            );
            const logPath = join(
              directory.path,
              'suite.log',
            );
            expect(await runSuite({
              command: [
                process.execPath,
                '--eval',
                `${printMarker({
              marker: 'PASS',
              label: 'nap',
            },)} ${printMarker({
              marker: 'PASS',
              label: 'purr',
            },)} console.log(process.env.NODE_V8_COVERAGE);`,
              ],
              cwd: directory.path,
              coverageDirectory,
              logPath,
            },),).toEqual({
              passes: 2,
              failures: 0,
              exitCode: 0,
            },);
            expect(await readFile(
              logPath,
              'utf8',
            ),).toContain(coverageDirectory,);
          },
        },),
        it({
          name: 'READS A FAILING COMMAND\'S EXIT CODE AND FAIL MARKERS, and a command that never started as exit 1',
          fn: async () => {
            await using directory = await scratchDir({ prefix: SCRATCH_PREFIX, },);
            const logPath = join(
              directory.path,
              'suite.log',
            );
            expect(await runSuite({
              command: [
                process.execPath,
                '--eval',
                `${printMarker({
              marker: 'FAIL',
              label: 'knead',
            },)} process.exit(3);`,
              ],
              cwd: directory.path,
              coverageDirectory: join(
                directory.path,
                'coverage',
              ),
              logPath,
            },),).toEqual({
              passes: 0,
              failures: 1,
              exitCode: 3,
            },);
            expect((await runSuite({
              command: ['translation-repair-no-such-cat-program',],
              cwd: directory.path,
              coverageDirectory: join(
                directory.path,
                'coverage',
              ),
              logPath,
            },)).exitCode,).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: tallyCoverage.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS EVERY COVERAGE FILE TWICE: a block one process lists as cold ran in the other, and one file alone leaves it cold',
          fn: async () => {
            await using both = await scratchDir({ prefix: SCRATCH_PREFIX, },);
            await writeFile(
              join(
                both.path,
                'coverage-1.json',
              ),
              coverageText({
                prefix: PREFIX,
                napRanges: [[10, 90, 2,], [40, 60, 0,],],
              },),
            );
            await writeFile(
              join(
                both.path,
                'coverage-2.json',
              ),
              coverageText({
                prefix: PREFIX,
                napRanges: [[10, 90, 3,],],
              },),
            );
            const tally = await tallyCoverage({
              coverageDirectory: both.path,
              bundleUrlPrefix: PREFIX,
            },);
            expect(tally.coldStretches(),).toEqual([],);
            expect(tally.loadedBundles(),).toEqual(['nap.mjs',],);

            await using one = await scratchDir({ prefix: SCRATCH_PREFIX, },);
            await writeFile(
              join(
                one.path,
                'coverage-1.json',
              ),
              coverageText({
                prefix: PREFIX,
                napRanges: [[10, 90, 2,], [40, 60, 0,],],
              },),
            );
            expect((await tallyCoverage({
              coverageDirectory: one.path,
              bundleUrlPrefix: PREFIX,
            },)).coldStretches(),).toEqual([{
              bundle: 'nap.mjs',
              start: 40,
              end: 60,
              shape: { kind: 'block', },
            },],);
          },
        },),
        it({
          name: 'REFUSES A COVERAGE FILE THAT DOES NOT READ as V8 writes one',
          fn: async () => {
            await using directory = await scratchDir({ prefix: SCRATCH_PREFIX, },);
            await writeFile(
              join(
                directory.path,
                'coverage-1.json',
              ),
              JSON.stringify({ result: 'none', },),
            );
            await expect(tallyCoverage({
              coverageDirectory: directory.path,
              bundleUrlPrefix: PREFIX,
            },),).rejects.toThrow(CoverageFileError,);
          },
        },),
      ],
    },),

    describe({
      name: coverageReadings.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'YIELDS EACH FILE\'S BUNDLE SCRIPTS IN THE ORDER GIVEN',
          fn: async () => {
            await using directory = await scratchDir({ prefix: SCRATCH_PREFIX, },);
            const purrPrefix = 'file:///cattery/purr/';
            const paths = [
              join(
                directory.path,
                'b.json',
              ),
              join(
                directory.path,
                'a.json',
              ),
            ];
            await writeFile(
              paths[0] ?? '',
              coverageText({
                prefix: PREFIX,
                napRanges: [[10, 90, 1,],],
              },),
            );
            await writeFile(
              paths[1] ?? '',
              coverageText({
                prefix: purrPrefix,
                napRanges: [[10, 90, 1,],],
              },),
            );
            const counts = [];
            for await (const scripts of coverageReadings({
              paths,
              bundleUrlPrefix: PREFIX,
            },))
              counts.push(scripts.length,);
            expect(counts,).toEqual([1, 0,],);
          },
        },),
      ],
    },),

    describe({
      name: readBundle.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A BUNDLE AND ITS MAP, naming its sources from the package and keeping its text, which the '
            + 'census reads each cold stretch out of',
          fn: async () => {
            await using directory = await scratchDir({ prefix: SCRATCH_PREFIX, },);
            const distDirectory = join(
              directory.path,
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
                'nap.mjs',
              ),
              'x\ny',
            );
            await writeFile(
              join(
                distDirectory,
                'nap.mjs.map',
              ),
              JSON.stringify({
                version: 3,
                sources: ['../../../src/nap.ts',],
                names: [],
                mappings: ';AAAA',
              },),
            );
            const { lines, sources, text, } = await readBundle({
              distDirectory,
              packageDirectory: directory.path,
              bundle: 'nap.mjs',
            },);
            expect(text,).toBe('x\ny',);
            expect(sources,).toEqual(['src/nap.ts',],);
            expect(sourceLineAt({
              lines,
              offset: 2,
            },),).toEqual({
              kind: 'mapped',
              source: 'src/nap.ts',
              line: 1,
            },);
          },
        },),
        it({
          name: 'REFUSES A BUNDLE WHOSE MAP DOES NOT READ as a version 3 map',
          fn: async () => {
            await using directory = await scratchDir({ prefix: SCRATCH_PREFIX, },);
            await writeFile(
              join(
                directory.path,
                'nap.mjs',
              ),
              'x',
            );
            await writeFile(
              join(
                directory.path,
                'nap.mjs.map',
              ),
              JSON.stringify({ version: 2, },),
            );
            await expect(readBundle({
              distDirectory: directory.path,
              packageDirectory: directory.path,
              bundle: 'nap.mjs',
            },),).rejects.toThrow(SourceMapFileError,);
          },
        },),
      ],
    },),

    describe({
      name: unloadedSourcesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS EACH SOURCE ONCE, LEAVES OUT ONE A LOADED BUNDLE CARRIES, and counts each one\'s physical lines with its kind',
          fn: async () => {
            await using directory = await scratchDir({ prefix: SCRATCH_PREFIX, },);
            await mkdir(
              join(
                directory.path,
                'src',
                'corpus-run',
              ),
              { recursive: true, },
            );
            await writeFile(
              join(
                directory.path,
                'src',
                'nap.ts',
              ),
              'doze\nyawn\nstretch',
            );
            await writeFile(
              join(
                directory.path,
                'src',
                'corpus-run',
                'nap-probe.ts',
              ),
              'blink\n',
            );
            expect(await unloadedSourcesOf({
              packageDirectory: directory.path,
              carried: ['src/nap.ts', 'src/corpus-run/nap-probe.ts', 'src/nap.ts', 'src/purr.ts',],
              loadedSources: new Set(['src/purr.ts',],),
              entryFiles: new Set(['src/corpus-run/nap-probe.ts',],),
              readText: readUtf8Text,
            },),).toEqual([
              {
                source: 'src/corpus-run/nap-probe.ts',
                kind: 'entry file',
                lines: 2,
              },
              {
                source: 'src/nap.ts',
                kind: 'library source',
                lines: 3,
              },
            ],);
          },
        },),
        it({
          name: 'REFUSES with the first source\'s read when two unloaded sources cannot be read and the second '
            + 'source\'s read is refused first',
          fn: async () => {
            /**
             The two refusals the source reads end in.
             */
            const { refuseAtOnce, refuseAfterThat, } = refusalOrder();
            /**
             What the count refused with.
             */
            const refusal = await rejectionOf({
              promise: unloadedSourcesOf({
                packageDirectory: '/cats/package',
                carried: ['src/nap.ts', 'src/purr.ts',],
                loadedSources: new Set<string>(),
                entryFiles: new Set<string>(),
                readText: async function refusesSecondFirst({ path, },): Promise<string> {
                  return await (path.endsWith('nap.ts',)
                    ? refuseAfterThat(new Error('the nap source cannot be read',),)
                    : refuseAtOnce(new Error('the purr source cannot be read',),));
                },
              },),
            },);
            expect(String(refusal,),).toBe('Error: the nap source cannot be read',);
          },
        },),
      ],
    },),
  ],
},);
