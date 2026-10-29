/**
 Tests what the coverage census reads (ledger T8): the suite's markers
 counted by occurrence rather than by line, its command line with every
 refusal strict `parseArgs` and the census add, and an earlier census's
 stretches and loaded sources, read only from a file of the current census
 format: a file of format 2, written before invariant throws were counted
 apart from cold code, is refused by name rather than read as though its
 stretches were cold code alone. The census file a run writes is read back
 by the same reader. Paths are cat-themed invention.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  CENSUS_FORMAT,
  CensusBaselineError,
  censusFileText,
  FAIL_MARKER,
  markerCount,
  PASS_MARKER,
  readBaselineCensus,
  readBaselineFile,
  readCensusArguments,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';

/**
 One stretch as the census writes it.
 */
const STRETCH = {
  bundle: 'nap.mjs',
  start: 0,
  end: 4,
  name: '',
  source: 'src/nap.ts',
  startLine: 1,
  endLine: 2,
} as const;

/**
 One invariant throw as the census writes it, apart from the stretches.
 */
const INVARIANT_THROW = {
  ...STRETCH,
  start: 9,
  end: 14,
  startLine: 7,
  endLine: 7,
  thrown: ['Error',],
} as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: markerCount.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS OCCURRENCES, NOT LINES: two tests\' markers on one line of a parallel log count twice',
          fn: async () => {
            const text = `${PASS_MARKER} nap${PASS_MARKER} purr\n${FAIL_MARKER} knead\n`;
            expect(markerCount({
              text,
              marker: PASS_MARKER,
            },),).toBe(2,);
            expect(markerCount({
              text,
              marker: FAIL_MARKER,
            },),).toBe(1,);
            expect(markerCount({
              text: '',
              marker: PASS_MARKER,
            },),).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: readCensusArguments.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS TEST FILES, A BASELINE AND EVERY SOURCE NAMED, and nothing when nothing is written',
          fn: async () => {
            expect(readCensusArguments({
              line: lineOf({
                command: 'coverage-census',
                typed: [
                  'src/nap.unit.test.ts',
                  '--baseline',
                  '/tmp/census.json',
                  '--source',
                  'src/nap.ts',
                  '--source',
                  'src/purr.ts',
                ],
              },),
            },),).toEqual({
              testFiles: ['src/nap.unit.test.ts',],
              baseline: ['/tmp/census.json',],
              sources: ['src/nap.ts', 'src/purr.ts',],
            },);
            expect(readCensusArguments({
              line: lineOf({
                command: 'coverage-census',
                typed: [],
              },),
            },),).toEqual({
              testFiles: [],
              baseline: [],
              sources: [],
            },);
          },
        },),
        it({
          name: 'REFUSES a flag written last, a flag followed by another flag, an unknown flag, and --source with no --baseline to read',
          fn: async () => {
            for (
              const typed of [
                ['--baseline',],
                ['--baseline', '--source', 'src/nap.ts',],
                ['--nap',],
                ['--source', 'src/nap.ts',],
              ]
            ) {
              expect(() => readCensusArguments({
                line: lineOf({
                  command: 'coverage-census',
                  typed,
                },),
              },),).toThrow(StatedRefusalError,);
            }
          },
        },),
      ],
    },),

    describe({
      name: readBaselineCensus.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE COMMIT, COLD STRETCHES AND LOADED SOURCES OF A CENSUS FILE of the current format, taken from '
            + 'a tree matching its commit, leaving the invariant throws the file lists apart out of the stretches',
          fn: async () => {
            expect(readBaselineCensus({
              path: '/tmp/census.json',
              text: JSON.stringify({
                format: CENSUS_FORMAT,
                head: 'abc',
                clean: true,
                stretches: [STRETCH,],
                invariantThrows: [INVARIANT_THROW,],
                loadedSources: ['src/nap.ts', 'src/purr.ts',],
              },),
            },),).toEqual({
              head: 'abc',
              stretches: [STRETCH,],
              loadedSources: new Set(['src/nap.ts', 'src/purr.ts',],),
            },);
          },
        },),
        it({
          name: 'REFUSES A CENSUS OF FORMAT 2, WRITTEN BEFORE INVARIANT THROWS WERE COUNTED APART, saying to take the '
            + 'baseline again: its stretches hold them among the cold code with nothing to tell them by, so read as '
            + 'cold code each would read as ran against a run that leaves them out',
          fn: async () => {
            const refusal = caught(function readsFormatTwo() {
              readBaselineCensus({
                path: '/tmp/census.json',
                text: JSON.stringify({
                  format: 2,
                  head: 'abc',
                  clean: true,
                  stretches: [STRETCH, INVARIANT_THROW,],
                  loadedSources: ['src/nap.ts',],
                },),
              },);
            },);
            expect(refusal,).toBeInstanceOf(CensusBaselineError,);
            expect(String(refusal,),).toBe(
              'CensusBaselineError: baseline /tmp/census.json does not read as a census this command wrote: it is '
                + 'census format 2, written before invariant throws were counted apart: its cold stretches hold them '
                + 'and record no text to tell them by, and this run leaves them out of its own, so each would read as '
                + 'ran; take the baseline again',
            );
          },
        },),
        it({
          name: 'REFUSES A CENSUS FILE CUT SHORT as its own refusal saying it is not JSON, naming the file and quoting '
            + 'none of its text, where the parser\'s refusal reached the command as a fault in the command',
          fn: async () => {
            const refusal = caught(function readsCutShort() {
              readBaselineCensus({
                path: '/tmp/census.json',
                text: JSON.stringify({
                  format: CENSUS_FORMAT,
                  head: 'abc',
                  clean: true,
                  stretches: [STRETCH,],
                },)
                  .slice(
                  0,
                  40,
                  ),
              },);
            },);
            expect(refusal,).toBeInstanceOf(CensusBaselineError,);
            expect(String(refusal,),).toBe(
              'CensusBaselineError: baseline /tmp/census.json does not read as a census this command wrote: it is '
                + 'not JSON',
            );
          },
        },),
        it({
          name: 'REFUSES A CENSUS NAMING NO COMMIT, one not saying whether its tree matched it, and one taken with '
            + 'uncommitted changes: a later reading tells an edited source by comparing the tree with that commit, '
            + 'and a census of uncommitted code has lines no commit holds',
          fn: async () => {
            for (const [text, says,] of [
              [
                JSON.stringify({
                  format: CENSUS_FORMAT,
                  clean: true,
                  stretches: [STRETCH,],
                  invariantThrows: [],
                  loadedSources: [],
                },),
                'it names no commit it was taken at',
              ],
              [
                JSON.stringify({
                  format: CENSUS_FORMAT,
                  head: 'abc',
                  stretches: [STRETCH,],
                  invariantThrows: [],
                  loadedSources: [],
                },),
                'it does not say whether its tree matched that commit',
              ],
              [
                JSON.stringify({
                  format: CENSUS_FORMAT,
                  head: 'abc',
                  clean: false,
                  stretches: [STRETCH,],
                  invariantThrows: [],
                  loadedSources: [],
                },),
                'it was taken with uncommitted changes under the package, so its lines match no commit a later reading '
                  + 'can compare the tree with; commit, then take the baseline again',
              ],
            ] as const) {
              /**
               The read, repeated for each check.
               */
              const read = () =>
                readBaselineCensus({
                  path: '/tmp/census.json',
                  text,
                },);
              expect(read,).toThrow(CensusBaselineError,);
              expect(read,).toThrow(`baseline /tmp/census.json does not read as a census this command wrote: ${says}`,);
            }
          },
        },),
        it({
          name: 'REFUSES, each for its own reason, a file that is not an object, one of format 1 or none (the '
            + 'census before ledger M67), one with no stretches, one holding a stretch missing a field, one with no '
            + 'invariant throws, one holding an invariant throw missing a field, naming something not a class as '
            + 'thrown, naming nothing thrown, or written as a bare class name, one with no loaded sources, and one '
            + 'whose loaded sources hold something not a path',
          fn: async () => {
            for (const [text, says,] of [
              ['null', 'it is not an object',],
              [
                JSON.stringify({
                  head: 'abc',
                  stretches: [STRETCH,],
                  loadedSources: [],
                },),
                `it is not census format ${String(CENSUS_FORMAT,)}`,
              ],
              [
                JSON.stringify({
                  format: 1,
                  stretches: [STRETCH,],
                  loadedSources: [],
                },),
                `it is not census format ${String(CENSUS_FORMAT,)}`,
              ],
              [
                JSON.stringify({
                  format: CENSUS_FORMAT,
                  head: 'abc',
                  invariantThrows: [],
                  loadedSources: [],
                },),
                'it has no list of stretches',
              ],
              [
                JSON.stringify({
                  format: CENSUS_FORMAT,
                  stretches: [{
                    ...STRETCH,
                    startLine: '1',
                  },],
                  invariantThrows: [],
                  loadedSources: [],
                },),
                'it has no list of stretches',
              ],
              [
                JSON.stringify({
                  format: CENSUS_FORMAT,
                  stretches: [STRETCH,],
                  loadedSources: [],
                },),
                'it has no list of invariant throws',
              ],
              [
                JSON.stringify({
                  format: CENSUS_FORMAT,
                  stretches: [STRETCH,],
                  invariantThrows: [{
                    ...INVARIANT_THROW,
                    endLine: '7',
                  },],
                  loadedSources: [],
                },),
                'it has no list of invariant throws',
              ],
              [
                JSON.stringify({
                  format: CENSUS_FORMAT,
                  stretches: [STRETCH,],
                  invariantThrows: [{
                    ...INVARIANT_THROW,
                    thrown: ['Error', 7,],
                  },],
                  loadedSources: [],
                },),
                'it has no list of invariant throws',
              ],
              [
                JSON.stringify({
                  format: CENSUS_FORMAT,
                  stretches: [STRETCH,],
                  invariantThrows: [STRETCH,],
                  loadedSources: [],
                },),
                'it has no list of invariant throws',
              ],
              [
                JSON.stringify({
                  format: CENSUS_FORMAT,
                  stretches: [STRETCH,],
                  invariantThrows: ['Error',],
                  loadedSources: [],
                },),
                'it has no list of invariant throws',
              ],
              [
                JSON.stringify({
                  format: CENSUS_FORMAT,
                  stretches: [STRETCH,],
                  invariantThrows: [],
                },),
                'it has no list of loaded sources',
              ],
              [
                JSON.stringify({
                  format: CENSUS_FORMAT,
                  stretches: [STRETCH,],
                  invariantThrows: [],
                  loadedSources: ['src/nap.ts', 7,],
                },),
                'it has no list of loaded sources',
              ],
            ] as const) {
              /**
               The read, repeated for each check.
               */
              const read = () =>
                readBaselineCensus({
                  path: '/tmp/census.json',
                  text,
                },);
              expect(read,).toThrow(CensusBaselineError,);
              expect(read,).toThrow(says,);
            }
          },
        },),
      ],
    },),

    describe({
      name: censusFileText.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WRITES WHAT A RUN FOUND IN THE CURRENT FORMAT, the invariant throws apart from the cold stretches, each '
            + 'uncalled function with its source and line or, where no map places it, its bundle and line 0, and '
            + 'the loaded sources sorted; readBaselineCensus READS THE FILE BACK as its commit, cold stretches and '
            + 'loaded sources',
          fn: async () => {
            const uncalled = {
              bundle: 'nap.mjs',
              start: 20,
              end: 30,
              name: 'doze',
              nested: false,
            };
            const text = censusFileText({
              head: 'abc',
              clean: true,
              testFiles: ['src/nap.unit.test.ts',],
              passes: 2,
              stretches: [STRETCH,],
              invariantThrows: [INVARIANT_THROW,],
              uncalled: [
                {
                  ...uncalled,
                  at: {
                    kind: 'mapped',
                    source: 'src/nap.ts',
                    line: 4,
                  },
                },
                {
                  ...uncalled,
                  name: '',
                  nested: true,
                  at: { kind: 'unmapped', },
                },
              ],
              loadedSources: new Set(['src/purr.ts', 'src/nap.ts',],),
              unloadedBundles: ['knead.mjs',],
              unloadedSources: [{
                source: 'src/knead.ts',
                kind: 'library source',
                lines: 3,
              },],
            },);
            expect(JSON.parse(text,),).toEqual({
              format: CENSUS_FORMAT,
              head: 'abc',
              clean: true,
              testFiles: ['src/nap.unit.test.ts',],
              passes: 2,
              stretches: [STRETCH,],
              invariantThrows: [INVARIANT_THROW,],
              uncalled: [
                {
                  ...uncalled,
                  source: 'src/nap.ts',
                  line: 4,
                },
                {
                  ...uncalled,
                  name: '',
                  nested: true,
                  source: '(unmapped) nap.mjs',
                  line: 0,
                },
              ],
              loadedSources: ['src/nap.ts', 'src/purr.ts',],
              unloadedBundles: ['knead.mjs',],
              unloadedSources: [{
                source: 'src/knead.ts',
                kind: 'library source',
                lines: 3,
              },],
            },);
            expect(readBaselineCensus({
              path: '/tmp/census.json',
              text,
            },),).toEqual({
              head: 'abc',
              stretches: [STRETCH,],
              loadedSources: new Set(['src/nap.ts', 'src/purr.ts',],),
            },);
          },
        },),
      ],
    },),

    describe({
      name: readBaselineFile.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A CENSUS FILE the operator named, as readBaselineCensus reads its text',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'census-baseline-', },);
            /**
             Where the earlier census stands.
             */
            const path = join(
              scratch.path,
              'census.json',
            );
            await writeFile(
              path,
              JSON.stringify({
                format: CENSUS_FORMAT,
                head: 'abc',
                clean: true,
                stretches: [STRETCH,],
                invariantThrows: [],
                loadedSources: ['src/nap.ts',],
              },),
            );
            expect(await readBaselineFile({ path, },),).toEqual({
              head: 'abc',
              stretches: [STRETCH,],
              loadedSources: new Set(['src/nap.ts',],),
            },);
          },
        },),
        it({
          name: 'REFUSES A BASELINE THAT IS NOT THERE as its own refusal naming the file and the filesystem code, '
            + 'where the read\'s own error reached the command',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'census-baseline-', },);
            /**
             A path nothing stands at.
             */
            const path = join(
              scratch.path,
              'purr.json',
            );
            /**
             What the read raised, taken as data.
             */
            let refusal: unknown;
            try {
              await readBaselineFile({ path, },);
            }
            catch (error) {
              refusal = error;
            }
            expect(refusal,).toBeInstanceOf(CensusBaselineError,);
            expect(String(refusal,),).toBe(
              `CensusBaselineError: baseline ${path} does not read as a census this command wrote: it could not be `
                + 'read (ENOENT)',
            );
          },
        },),
        it({
          name: 'REFUSES A BASELINE THAT IS A DIRECTORY as its own refusal naming the path and the filesystem code, '
            + 'and keeps the failure as the refusal\'s cause',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'census-baseline-', },);
            /**
             What the read raised, taken as data.
             */
            let refusal: unknown;
            try {
              await readBaselineFile({ path: scratch.path, },);
            }
            catch (error) {
              refusal = error;
            }
            expect(refusal,).toBeInstanceOf(CensusBaselineError,);
            expect(String(refusal,),).toBe(
              `CensusBaselineError: baseline ${scratch.path} does not read as a census this command wrote: it could `
                + 'not be read (EISDIR)',
            );
            expect((refusal as Error).cause,).toBeInstanceOf(Error,);
          },
        },),
        it({
          name: 'REFUSES A BASELINE THAT IS NOT A CENSUS as a stated refusal, so the command prints it as the '
            + 'operator\'s mistake and not as a fault of its own',
          fn: async () => {
            /**
             What the reader raised for text that is not JSON.
             */
            const refusal = caught(function readsText(): unknown {
              return readBaselineCensus({
                path: '/tmp/purr.json',
                text: 'purr',
              },);
            },);
            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(refusal,).toBeInstanceOf(CensusBaselineError,);
            expect(String(refusal,),).toBe(
              'CensusBaselineError: baseline /tmp/purr.json does not read as a census this command wrote: it is not '
                + 'JSON',
            );
          },
        },),
      ],
    },),
  ],
},);
