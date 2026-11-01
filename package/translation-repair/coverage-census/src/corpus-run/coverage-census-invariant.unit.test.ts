/**
 Tests what the coverage census says of its invariant throws (ledger T8),
 the stretches it counts apart from cold code: one row for each, by source
 and line with the kind of its source and what it throws; the count printed
 beside the cold counts, by kind; and the list a reader audits. They share
 the census stretch fixture (`coverage-census.test-fixture.ts`). Paths and
 names are cat-themed invention.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  invariantThrowCountLine,
  invariantThrowListLines,
  type InvariantThrowRow,
  invariantThrowRowsOf,
} from '../../dist/final/node/index.mjs';
import { recorded, } from './coverage-census.test-fixture.ts';

/**
 Invariant throws in three kinds of source, in report order.
 */
const ROWS: readonly InvariantThrowRow[] = [
  {
    source: '../../module/whisker/src/index.ts',
    kind: 'other package',
    startLine: 7,
    endLine: 7,
    thrown: ['WhiskerInvariantError',],
  },
  {
    source: 'src/corpus-run/nap-probe.ts',
    kind: 'entry file',
    startLine: 4,
    endLine: 6,
    thrown: ['Error',],
  },
  {
    source: 'src/nap.ts',
    kind: 'library source',
    startLine: 8,
    endLine: 9,
    thrown: ['Error',],
  },
  {
    source: 'src/nap.ts',
    kind: 'library source',
    startLine: 30,
    endLine: 30,
    thrown: ['Error', 'NapInvariantError',],
  },
];

/**
 What heads the list of invariant throws.
 */
const HEADING = 'invariant throws by source and line (each stretch nothing but throws of an Error whose message '
  + 'begins "unreachable:" or of a class whose name ends in InvariantError), with what each throws:';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: invariantThrowRowsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES EACH INVARIANT THROW BY ITS SOURCE, THAT SOURCE\'S KIND, ITS LINES AND WHAT IT THROWS, sorted by '
            + 'source and then line whatever order the census placed them in',
          fn: async () => {
            expect(invariantThrowRowsOf({
              invariantThrows: [
                {
                  ...recorded({
                    source: 'src/nap.ts',
                    startLine: 30,
                    endLine: 30,
                  },),
                  thrown: ['Error', 'NapInvariantError',],
                },
                {
                  ...recorded({
                    source: 'src/corpus-run/nap-probe.ts',
                    startLine: 4,
                    endLine: 6,
                  },),
                  thrown: ['Error',],
                },
                {
                  ...recorded({
                    source: 'src/nap.ts',
                    startLine: 8,
                    endLine: 9,
                  },),
                  thrown: ['Error',],
                },
                {
                  ...recorded({
                    source: '../../module/whisker/src/index.ts',
                    startLine: 7,
                    endLine: 7,
                  },),
                  thrown: ['WhiskerInvariantError',],
                },
              ],
              entryFiles: new Set(['src/corpus-run/nap-probe.ts',],),
            },),).toEqual(ROWS,);
          },
        },),
      ],
    },),

    describe({
      name: invariantThrowCountLine.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS THE STRETCHES AND, BESIDE THE COUNT, EACH KIND HOLDING ANY in the report\'s order of kinds; one '
            + 'as 1 stretch; none as 0 stretches with no kind beside it',
          fn: async () => {
            expect([
              ROWS,
              ROWS.slice(
                0,
                1,
              ),
              [],
            ].map((rows,) => invariantThrowCountLine({ rows, },)),).toEqual([
              'invariant throws, counted apart from the cold stretches: 4 stretches (library source 2, entry file 1, '
                + 'other package 1)',
              'invariant throws, counted apart from the cold stretches: 1 stretch (other package 1)',
              'invariant throws, counted apart from the cold stretches: 0 stretches',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: invariantThrowListLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HEADS THE LIST WITH WHAT THE CENSUS TOOK FOR AN INVARIANT THROW, then names each by source, lines, the '
            + 'kind of its source and every class it throws; the heading stands alone when there is none',
          fn: async () => {
            expect(invariantThrowListLines({ rows: ROWS, },),).toEqual([
              HEADING,
              '  ../../module/whisker/src/index.ts:7-7 (other package): WhiskerInvariantError',
              '  src/corpus-run/nap-probe.ts:4-6 (entry file): Error',
              '  src/nap.ts:8-9 (library source): Error',
              '  src/nap.ts:30-30 (library source): Error, NapInvariantError',
            ],);
            expect(invariantThrowListLines({ rows: [], },),).toEqual([HEADING,],);
          },
        },),
      ],
    },),
  ],
},);
