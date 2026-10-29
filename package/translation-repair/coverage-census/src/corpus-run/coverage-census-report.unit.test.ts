/**
 Tests the coverage census's report (ledger T8): how a mapped stretch is
 recorded, one record per source piece, the refusal of a census that places an
 uncalled function outside its own source's stretches (ledger M67), how
 sources are sorted into kinds, the rows and totals, and, from
 `coverage-census-baseline.ts`, how a batch's run reads an earlier census's
 stretches, which claimed sources that census holds no stretch in, and which
 were edited since its commit. They share the census stretch fixture
 (`coverage-census.test-fixture.ts`). Paths and names are cat-themed
 invention.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  baselineStatusesOf,
  censusStretchesOf,
  editedClaimsOf,
  emptyClaimsOf,
  isUnmappedSource,
  kindTotalsOf,
  type MappedStretch,
  requirePlacedFunctions,
  sourceKindOf,
  sourceRowsOf,
  unmappedSourceOf,
} from '../../dist/final/node/index.mjs';
import { recorded, } from './coverage-census.test-fixture.ts';

/**
 A block stretch in `nap.mjs` from bundle offset 10 to 20.
 */
const BLOCK = {
  bundle: 'nap.mjs',
  start: 10,
  end: 20,
  shape: { kind: 'block', },
} as const;

/**
 A mapped source line.

 @param source - source file

 @param line - 1-based line

 @returns Line
 */
function at({
  source,
  line,
}: {
  readonly source: string;
  readonly line: number;
},) {
  return {
    kind: 'mapped',
    source,
    line,
  } as const;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: censusStretchesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RECORDS EACH PIECE UNDER ITS OWN SOURCE, offsets and lines, so no source after the first drops out (ledger M67)',
          fn: async () => {
            const stretch: MappedStretch = {
              ...BLOCK,
              pieces: [
                {
                  start: 10,
                  end: 14,
                  span: {
                    kind: 'mapped',
                    source: 'src/nap.ts',
                    firstLine: 4,
                    lastLine: 7,
                  },
                },
                {
                  start: 14,
                  end: 20,
                  span: {
                    kind: 'mapped',
                    source: 'src/purr.ts',
                    firstLine: 1,
                    lastLine: 9,
                  },
                },
              ],
            };
            expect(censusStretchesOf({ stretch, },),).toEqual([
              {
                bundle: 'nap.mjs',
                start: 10,
                end: 14,
                name: '',
                source: 'src/nap.ts',
                startLine: 4,
                endLine: 7,
              },
              {
                bundle: 'nap.mjs',
                start: 14,
                end: 20,
                name: '',
                source: 'src/purr.ts',
                startLine: 1,
                endLine: 9,
              },
            ],);
          },
        },),
        it({
          name: 'RECORDS AN UNMAPPED PIECE under its bundle at line 0, and names an uncalled function on its first piece alone',
          fn: async () => {
            expect(censusStretchesOf({
              stretch: {
                ...BLOCK,
                shape: {
                  kind: 'function',
                  name: 'doze',
                },
                pieces: [
                  {
                    start: 10,
                    end: 12,
                    span: { kind: 'unmapped', },
                  },
                  {
                    start: 12,
                    end: 20,
                    span: {
                      kind: 'mapped',
                      source: 'src/nap.ts',
                      firstLine: 4,
                      lastLine: 4,
                    },
                  },
                ],
              },
            },),).toEqual([
              {
                bundle: 'nap.mjs',
                start: 10,
                end: 12,
                name: 'doze',
                source: '(unmapped) nap.mjs',
                startLine: 0,
                endLine: 0,
              },
              {
                bundle: 'nap.mjs',
                start: 12,
                end: 20,
                name: '',
                source: 'src/nap.ts',
                startLine: 4,
                endLine: 4,
              },
            ],);
          },
        },),
      ],
    },),

    describe({
      name: requirePlacedFunctions.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS A CENSUS PLACING EVERY UNCALLED FUNCTION in a stretch of its own bundle and source, unmapped ones at line 0',
          fn: async () => {
            const uncalled = {
              bundle: 'nap.mjs',
              start: 0,
              end: 1,
              name: 'doze',
              nested: false,
            };
            expect(() =>
              requirePlacedFunctions({
                stretches: [
                  recorded({
                    source: 'src/purr.ts',
                    startLine: 2,
                    endLine: 6,
                  },),
                  recorded({
                    source: '(unmapped) nap.mjs',
                    startLine: 0,
                    endLine: 0,
                  },),
                ],
                uncalled: [
                  {
                    ...uncalled,
                    at: at({
                      source: 'src/purr.ts',
                      line: 6,
                    },),
                  },
                  {
                    ...uncalled,
                    at: { kind: 'unmapped', },
                  },
                ],
              },)
              ,).not.toThrow();
          },
        },),
        it({
          name: 'REFUSES THE SHAPE LEDGER M67 LEFT: a function in a second source whose cold code was recorded under the first, '
            + 'or in a stretch of another bundle',
          fn: async () => {
            const inPurr = {
              bundle: 'nap.mjs',
              start: 0,
              end: 1,
              name: 'doze',
              nested: false,
              at: at({
                source: 'src/purr.ts',
                line: 3,
              },),
            };
            expect(() =>
              requirePlacedFunctions({
                stretches: [recorded({
                  source: 'src/nap.ts',
                  startLine: 172,
                  endLine: 172,
                },),],
                uncalled: [inPurr,],
              },)
              ,).toThrow(StatedRefusalError,);
            expect(() =>
              requirePlacedFunctions({
                stretches: [{
                  ...recorded({
                    source: 'src/purr.ts',
                    startLine: 1,
                    endLine: 9,
                  },),
                  bundle: 'purr.mjs',
                },],
                uncalled: [inPurr,],
              },)
              ,).toThrow(StatedRefusalError,);
          },
        },),
      ],
    },),

    describe({
      name: sourceKindOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SORTS SOURCES INTO unmapped code, other packages, runner entry files and library source',
          fn: async () => {
            const entryFiles = new Set(['src/corpus-run/nap-probe.ts',],);
            expect([
              '(unmapped) nap.mjs',
              '../../module/whisker/src/index.ts',
              'src/corpus-run/nap-probe.ts',
              'src/nap.ts',
            ].map((source,) =>
              sourceKindOf({
                source,
                entryFiles,
              },)
            ),).toEqual(['unmapped', 'other package', 'entry file', 'library source',],);
          },
        },),
      ],
    },),

    describe({
      name: isUnmappedSource.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE SOURCE unmappedSourceOf GIVES A BUNDLE AS UNMAPPED, and a source path as mapped, even one '
            + 'whose later part reads "(unmapped)"',
          fn: async () => {
            expect(unmappedSourceOf({ bundle: 'nap-AAAA.mjs', },),).toBe('(unmapped) nap-AAAA.mjs',);
            expect([
              unmappedSourceOf({ bundle: 'nap-AAAA.mjs', },),
              'src/nap.ts',
              'src/(unmapped) nap.ts',
            ].map((source,) => isUnmappedSource({ source, },)),).toEqual([true, false, false,],);
          },
        },),
      ],
    },),

    describe({
      name: sourceRowsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS DISTINCT LINES AND THE UNCALLED FUNCTIONS STARTING IN EACH SOURCE, most cold lines first, ties by name',
          fn: async () => {
            const uncalled = {
              bundle: 'nap.mjs',
              start: 0,
              end: 1,
              name: 'doze',
              nested: false,
            };
            expect(sourceRowsOf({
              stretches: [
                recorded({
                  source: 'src/purr.ts',
                  startLine: 2,
                  endLine: 3,
                },),
                recorded({
                  source: 'src/nap.ts',
                  startLine: 1,
                  endLine: 3,
                },),
                recorded({
                  source: 'src/nap.ts',
                  startLine: 2,
                  endLine: 4,
                },),
                recorded({
                  source: 'src/knead.ts',
                  startLine: 5,
                  endLine: 6,
                },),
              ],
              uncalled: [
                {
                  ...uncalled,
                  at: at({
                    source: 'src/nap.ts',
                    line: 1,
                  },),
                },
                {
                  ...uncalled,
                  at: { kind: 'unmapped', },
                },
              ],
              entryFiles: new Set(),
            },),).toEqual([
              {
                source: 'src/nap.ts',
                kind: 'library source',
                stretches: 2,
                lines: 4,
                uncalled: 1,
              },
              {
                source: 'src/knead.ts',
                kind: 'library source',
                stretches: 1,
                lines: 2,
                uncalled: 0,
              },
              {
                source: 'src/purr.ts',
                kind: 'library source',
                stretches: 1,
                lines: 2,
                uncalled: 0,
              },
            ],);
          },
        },),
      ],
    },),

    describe({
      name: kindTotalsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TOTALS EACH KIND HOLDING A ROW, library source first, leaving out kinds with none',
          fn: async () => {
            const row = {
              source: 'src/nap.ts',
              kind: 'library source',
              stretches: 2,
              lines: 4,
              uncalled: 1,
            } as const;
            expect(kindTotalsOf({
              rows: [
                {
                  ...row,
                  source: '../../module/whisker/src/index.ts',
                  kind: 'other package',
                },
                row,
                {
                  ...row,
                  source: 'src/purr.ts',
                },
              ],
            },),).toEqual([
              {
                kind: 'library source',
                files: 2,
                stretches: 4,
                lines: 8,
                uncalled: 2,
              },
              {
                kind: 'other package',
                files: 1,
                stretches: 2,
                lines: 4,
                uncalled: 1,
              },
            ],);
          },
        },),
      ],
    },),

    describe({
      name: baselineStatusesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A STRETCH AS STILL COLD where a later cold stretch shares a line, RAN where none does, and NOT LOADED where its source was not loaded',
          fn: async () => {
            const baseline = [
              recorded({
                source: 'src/nap.ts',
                startLine: 3,
                endLine: 5,
              },),
              recorded({
                source: 'src/nap.ts',
                startLine: 8,
                endLine: 9,
              },),
              recorded({
                source: 'src/purr.ts',
                startLine: 1,
                endLine: 2,
              },),
            ];
            expect(baselineStatusesOf({
              baseline,
              current: [
                recorded({
                  source: 'src/nap.ts',
                  startLine: 5,
                  endLine: 7,
                },),
              ],
              loadedSources: new Set(['src/nap.ts',],),
              sources: new Set(),
              edited: new Set(),
            },).map((read,) => read.status),).toEqual(['still cold', 'ran', 'not loaded',],);
          },
        },),
        it({
          name:
            'LEAVES OUT EVERY STRETCH OF A SOURCE EDITED SINCE THE BASELINE, whose lines there now name other code: '
              + 'counting it as ran because nothing cold overlaps its old lines is how a moved stretch read as run',
          fn: async () => {
            expect(baselineStatusesOf({
              baseline: [
                recorded({
                  source: 'src/nap.ts',
                  startLine: 3,
                  endLine: 5,
                },),
                recorded({
                  source: 'src/purr.ts',
                  startLine: 1,
                  endLine: 2,
                },),
              ],
              current: [
                recorded({
                  source: 'src/nap.ts',
                  startLine: 14,
                  endLine: 16,
                },),
              ],
              loadedSources: new Set(['src/nap.ts', 'src/purr.ts',],),
              sources: new Set(),
              edited: new Set(['src/nap.ts',],),
            },).map((read,) => [read.stretch.source, read.status,]),).toEqual([['src/purr.ts', 'ran',],],);
          },
        },),
        it({
          name: 'READS ONLY THE CLAIMED SOURCES when any are named',
          fn: async () => {
            expect(baselineStatusesOf({
              baseline: [
                recorded({
                  source: 'src/nap.ts',
                  startLine: 3,
                  endLine: 5,
                },),
                recorded({
                  source: 'src/purr.ts',
                  startLine: 1,
                  endLine: 2,
                },),
              ],
              current: [],
              loadedSources: new Set(['src/nap.ts', 'src/purr.ts',],),
              sources: new Set(['src/purr.ts',],),
              edited: new Set(),
            },).map((read,) => [read.stretch.source, read.status,]),).toEqual([['src/purr.ts', 'ran',],],);
          },
        },),
        it({
          name: 'LEAVES OUT UNMAPPED CODE, which has no lines to compare: a rebuild renames its bundle, so the '
            + 'same stretch read as not loaded at every census (ledger B105)',
          fn: async () => {
            expect(baselineStatusesOf({
              baseline: [
                recorded({
                  source: unmappedSourceOf({ bundle: 'nap-AAAA.mjs', },),
                  startLine: 0,
                  endLine: 0,
                },),
                recorded({
                  source: 'src/purr.ts',
                  startLine: 1,
                  endLine: 2,
                },),
              ],
              current: [
                recorded({
                  source: unmappedSourceOf({ bundle: 'nap-BBBB.mjs', },),
                  startLine: 0,
                  endLine: 0,
                },),
              ],
              loadedSources: new Set([unmappedSourceOf({ bundle: 'nap-BBBB.mjs', },), 'src/purr.ts',],),
              sources: new Set(),
              edited: new Set(),
            },).map((read,) => [read.stretch.source, read.status,]),).toEqual([['src/purr.ts', 'ran',],],);
          },
        },),
      ],
    },),

    describe({
      name: emptyClaimsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES EACH CLAIMED SOURCE WITH NO BASELINE STRETCH, sorted, with whether each census loaded it and the '
            + 'cold stretches this run left in it, and leaves a claimed source the baseline holds a stretch in',
          fn: async () => {
            expect(emptyClaimsOf({
              baseline: {
                head: 'c0ffee123',
                stretches: [
                  recorded({
                    source: 'src/nap.ts',
                    startLine: 3,
                    endLine: 5,
                  },),
                ],
                loadedSources: new Set(['src/nap.ts', 'src/purr.ts',],),
              },
              edited: new Set(),
              current: [
                recorded({
                  source: 'src/knead.ts',
                  startLine: 2,
                  endLine: 2,
                },),
                recorded({
                  source: 'src/nap.ts',
                  startLine: 3,
                  endLine: 5,
                },),
              ],
              loadedSources: new Set(['src/knead.ts', 'src/nap.ts', 'src/purr.ts',],),
              sources: new Set(['src/yawn.ts', 'src/purr.ts', 'src/nap.ts', 'src/knead.ts',],),
            },),).toEqual([
              {
                source: 'src/knead.ts',
                loadedAtBaseline: false,
                loadedNow: true,
                coldNow: 1,
              },
              {
                source: 'src/purr.ts',
                loadedAtBaseline: true,
                loadedNow: true,
                coldNow: 0,
              },
              {
                source: 'src/yawn.ts',
                loadedAtBaseline: false,
                loadedNow: false,
                coldNow: 0,
              },
            ],);
          },
        },),
        it({
          name: 'NAMES NOTHING when the batch claims no source, since then every baseline stretch is read',
          fn: async () => {
            expect(emptyClaimsOf({
              baseline: {
                head: 'c0ffee123',
                stretches: [],
                loadedSources: new Set(),
              },
              edited: new Set(),
              current: [],
              loadedSources: new Set(['src/purr.ts',],),
              sources: new Set(),
            },),).toEqual([],);
          },
        },),
        it({
          name:
            'LEAVES OUT A CLAIMED SOURCE EDITED SINCE THE BASELINE, which the edited claims name instead: "ran whole '
              + 'there" says nothing of code written after it',
          fn: async () => {
            expect(emptyClaimsOf({
              baseline: {
                head: 'c0ffee123',
                stretches: [],
                loadedSources: new Set(['src/purr.ts', 'src/yawn.ts',],),
              },
              edited: new Set(['src/purr.ts',],),
              current: [],
              loadedSources: new Set(['src/purr.ts', 'src/yawn.ts',],),
              sources: new Set(['src/purr.ts', 'src/yawn.ts',],),
            },).map((claim,) => claim.source),).toEqual(['src/yawn.ts',],);
          },
        },),
      ],
    },),

    describe({
      name: editedClaimsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'NAMES EACH CLAIMED SOURCE EDITED SINCE THE BASELINE, sorted, with whether this run loaded it and the cold '
              + 'stretches it left there, whether or not the baseline holds a stretch in it, and leaves an unedited one',
          fn: async () => {
            expect(editedClaimsOf({
              baseline: {
                head: 'c0ffee123',
                stretches: [
                  recorded({
                    source: 'src/nap.ts',
                    startLine: 3,
                    endLine: 5,
                  },),
                ],
                loadedSources: new Set(['src/nap.ts', 'src/purr.ts',],),
              },
              edited: new Set(['src/nap.ts', 'src/yawn.ts', 'src/knead.ts',],),
              current: [
                recorded({
                  source: 'src/nap.ts',
                  startLine: 12,
                  endLine: 12,
                },),
                recorded({
                  source: 'src/nap.ts',
                  startLine: 20,
                  endLine: 21,
                },),
              ],
              loadedSources: new Set(['src/nap.ts', 'src/purr.ts',],),
              sources: new Set(['src/yawn.ts', 'src/purr.ts', 'src/nap.ts',],),
            },),).toEqual([
              {
                source: 'src/nap.ts',
                loadedNow: true,
                coldNow: 2,
              },
              {
                source: 'src/yawn.ts',
                loadedNow: false,
                coldNow: 0,
              },
            ],);
          },
        },),
        it({
          name:
            'NAMES ONLY SOURCES A CENSUS HOLDS A STRETCH IN when the batch claims none, so a reading of every source '
              + 'does not list each document and test edited since',
          fn: async () => {
            expect(editedClaimsOf({
              baseline: {
                head: 'c0ffee123',
                stretches: [
                  recorded({
                    source: 'src/nap.ts',
                    startLine: 3,
                    endLine: 5,
                  },),
                  recorded({
                    source: 'src/purr.ts',
                    startLine: 1,
                    endLine: 1,
                  },),
                ],
                loadedSources: new Set(['src/nap.ts', 'src/purr.ts',],),
              },
              edited: new Set(['src/nap.ts', 'doc/whiskers.md', 'src/nap.unit.test.ts',],),
              current: [],
              loadedSources: new Set(['src/nap.ts', 'src/purr.ts',],),
              sources: new Set(),
            },),).toEqual([
              {
                source: 'src/nap.ts',
                loadedNow: true,
                coldNow: 0,
              },
            ],);
          },
        },),
        it({
          name:
            'NAMES A SOURCE ADDED SINCE THE BASELINE THAT THIS RUN LEFT COLD when the batch claims none, though the '
              + 'baseline holds no stretch there: read from the baseline\'s sources alone, its cold code was counted nowhere '
              + '(ledger B61)',
          fn: async () => {
            expect(editedClaimsOf({
              baseline: {
                head: 'c0ffee123',
                stretches: [
                  recorded({
                    source: 'src/nap.ts',
                    startLine: 3,
                    endLine: 5,
                  },),
                ],
                loadedSources: new Set(['src/nap.ts',],),
              },
              edited: new Set(['src/knead.ts', 'src/knead.unit.test.ts',],),
              current: [
                recorded({
                  source: 'src/knead.ts',
                  startLine: 4,
                  endLine: 6,
                },),
                recorded({
                  source: 'src/nap.ts',
                  startLine: 3,
                  endLine: 5,
                },),
              ],
              loadedSources: new Set(['src/knead.ts', 'src/nap.ts',],),
              sources: new Set(),
            },),).toEqual([
              {
                source: 'src/knead.ts',
                loadedNow: true,
                coldNow: 1,
              },
            ],);
          },
        },),
      ],
    },),
  ],
},);
