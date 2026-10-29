/**
 Tests the coverage census's printed report (ledger T8): the heading's
 commit, scope and passes; a line per kind; the invariant throws counted
 apart from them, by kind; the bundles no test loaded with the library
 sources only they carry; library rows alone; each invariant throw by source
 and line with what it throws; the outermost
 uncalled functions in package source by source and line number, anonymous
 ones named so; and a baseline reading's counts, unproven stretches,
 stretches cold since, claimed sources the baseline holds no stretch in, and
 sources edited since. Paths and names are cat-themed invention.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  baselineReportLines,
  type CensusSummary,
  censusReportLines,
  type MappedFunction,
} from '../../dist/final/node/index.mjs';
import { recorded, } from './coverage-census.test-fixture.ts';

/**
 An uncalled function in a source at a line.

 @param source - source file

 @param line - 1-based line

 @param name - name V8 gave it

 @param nested - whether it sits inside another uncalled function

 @returns Mapped function
 */
function uncalledAt({
  source,
  line,
  name,
  nested,
}: {
  readonly source: string;
  readonly line: number;
  readonly name: string;
  readonly nested: boolean;
},): MappedFunction {
  return {
    bundle: 'nap.mjs',
    start: 0,
    end: 1,
    name,
    nested,
    at: {
      kind: 'mapped',
      source,
      line,
    },
  };
}

/**
 A census with one row and total of each kind, invariant throws in two
 kinds, two unloaded sources, and uncalled functions of every sort the
 report filters.
 */
const CENSUS: CensusSummary = {
  head: 'abc123def',
  clean: true,
  testFiles: [],
  passes: 12,
  totals: [
    {
      kind: 'library source',
      files: 1,
      stretches: 2,
      lines: 5,
      uncalled: 3,
    },
    {
      kind: 'entry file',
      files: 1,
      stretches: 1,
      lines: 4,
      uncalled: 0,
    },
  ],
  rows: [
    {
      source: 'src/nap.ts',
      kind: 'library source',
      stretches: 2,
      lines: 5,
      uncalled: 3,
    },
    {
      source: 'src/corpus-run/nap-probe.ts',
      kind: 'entry file',
      stretches: 1,
      lines: 4,
      uncalled: 0,
    },
  ],
  invariantThrows: [
    {
      source: '../../module/whisker/src/index.ts',
      kind: 'other package',
      startLine: 7,
      endLine: 7,
      thrown: ['WhiskerInvariantError',],
    },
    {
      source: 'src/nap.ts',
      kind: 'library source',
      startLine: 20,
      endLine: 21,
      thrown: ['Error',],
    },
    {
      source: 'src/nap.ts',
      kind: 'library source',
      startLine: 30,
      endLine: 30,
      thrown: ['Error', 'NapInvariantError',],
    },
  ],
  uncalled: [
    uncalledAt({
      source: 'src/nap.ts',
      line: 10,
      name: 'doze',
      nested: false,
    },),
    uncalledAt({
      source: 'src/nap.ts',
      line: 9,
      name: '',
      nested: false,
    },),
    uncalledAt({
      source: 'src/nap.ts',
      line: 11,
      name: 'yawn',
      nested: true,
    },),
    uncalledAt({
      source: '../../module/whisker/src/index.ts',
      line: 3,
      name: 'twitch',
      nested: false,
    },),
    {
      ...uncalledAt({
        source: 'src/nap.ts',
        line: 1,
        name: 'blink',
        nested: false,
      },),
      at: { kind: 'unmapped', },
    },
  ],
  unloadedBundles: ['nap-probe.mjs',],
  unloadedSources: [
    {
      source: 'src/corpus-run/nap-probe.ts',
      kind: 'entry file',
      lines: 40,
    },
    {
      source: 'src/corpus-run/nap-layout.ts',
      kind: 'library source',
      lines: 20,
    },
  ],
  censusPath: '/tmp/census.json',
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: censusReportLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PRINTS THE HEADING, KINDS, INVARIANT THROWS COUNTED APART BY KIND, UNLOADED BUNDLES, LIBRARY ROWS, EACH '
            + 'INVARIANT THROW by source and line with what it throws, AND OUTERMOST UNCALLED FUNCTIONS in package '
            + 'source by line number',
          fn: async () => {
            expect(censusReportLines({ census: CENSUS, },),).toEqual([
              `coverage-census at ${CENSUS.head}: the unit suite, ${String(CENSUS.passes,)} passes`,
              'library source: 1 file, 2 stretches over 5 lines, 3 functions never called',
              'entry file: 1 file, 1 stretch over 4 lines, 0 functions never called',
              'invariant throws, counted apart from the cold stretches: 3 stretches (library source 2, other package 1)',
              'bundles no test loaded: 1, carrying 2 sources and 60 physical lines',
              '  library source only those bundles carry: src/corpus-run/nap-layout.ts',
              'library source by cold lines:',
              '  src/nap.ts: 2 stretches, 5 lines, 3 never called',
              'invariant throws by source and line (each stretch nothing but throws of an Error whose message '
                + 'begins "unreachable:" or of a class whose name ends in InvariantError), with what each throws:',
              '  ../../module/whisker/src/index.ts:7-7 (other package): WhiskerInvariantError',
              '  src/nap.ts:20-21 (library source): Error',
              '  src/nap.ts:30-30 (library source): Error, NapInvariantError',
              'functions never called in package source, outermost:',
              '  src/nap.ts:9 (anonymous)',
              '  src/nap.ts:10 doze',
              `census written to ${CENSUS.censusPath}`,
            ],);
          },
        },),
        it({
          name: 'SAYS WHEN THE FILES DIFFERED FROM THE COMMIT, and how many test files ran when some were named',
          fn: async () => {
            expect(censusReportLines({
              census: {
                ...CENSUS,
                clean: false,
                testFiles: ['src/nap.unit.test.ts', 'src/purr.unit.test.ts',],
              },
            },)[0],).toBe(`coverage-census at ${CENSUS.head} with uncommitted changes: 2 test files, ${String(CENSUS.passes,)} passes`,);
          },
        },),
      ],
    },),

    describe({
      name: baselineReportLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS EACH STATUS, then lists every stretch not proven run',
          fn: async () => {
            const stretch = recorded({
              source: 'src/nap.ts',
              startLine: 3,
              endLine: 5,
            },);
            expect(baselineReportLines({
              path: '/tmp/census.json',
              head: 'c0ffee123',
              statuses: [
                {
                  stretch,
                  status: 'ran',
                },
                {
                  stretch: {
                    ...stretch,
                    startLine: 8,
                    endLine: 8,
                  },
                  status: 'still cold',
                },
                {
                  stretch: {
                    ...stretch,
                    source: 'src/purr.ts',
                  },
                  status: 'not loaded',
                },
              ],
              coldSince: [],
              emptyClaims: [],
              editedClaims: [],
            },),).toEqual([
              'against /tmp/census.json at c0ffee123: ran 1, still cold 1, cold since then 0, not loaded 1, claimed '
                + 'sources with no stretch there 0, sources edited since then 0',
              '  still cold: src/nap.ts:8-8',
              '  not loaded: src/purr.ts:3-5',
            ],);
          },
        },),
        it({
          name: 'COUNTS AND NAMES EACH STRETCH COLD SINCE THE BASELINE, saying whether the baseline ran its lines or never '
            + 'loaded its source (ledger B61)',
          fn: async () => {
            expect(baselineReportLines({
              path: '/tmp/census.json',
              head: 'c0ffee123',
              statuses: [],
              coldSince: [
                {
                  stretch: recorded({
                    source: 'src/nap.ts',
                    startLine: 8,
                    endLine: 9,
                  },),
                  loadedAtBaseline: true,
                },
                {
                  stretch: recorded({
                    source: 'src/knead.ts',
                    startLine: 2,
                    endLine: 2,
                  },),
                  loadedAtBaseline: false,
                },
              ],
              emptyClaims: [],
              editedClaims: [],
            },),).toEqual([
              'against /tmp/census.json at c0ffee123: ran 0, still cold 0, cold since then 2, not loaded 0, claimed '
                + 'sources with no stretch there 0, sources edited since then 0',
              '  cold since then (ran there): src/nap.ts:8-9',
              '  cold since then (not loaded there): src/knead.ts:2-2',
            ],);
          },
        },),
        it({
          name:
            'COUNTS AND NAMES EACH SOURCE EDITED SINCE THE BASELINE, whose baseline lines name other code now, with '
              + 'whether this run loaded it and how many cold stretches it left, which is all that speaks for it',
          fn: async () => {
            expect(baselineReportLines({
              path: '/tmp/census.json',
              head: 'c0ffee123',
              statuses: [],
              coldSince: [],
              emptyClaims: [],
              editedClaims: [
                {
                  source: 'src/nap.ts',
                  loadedNow: true,
                  coldNow: 2,
                },
                {
                  source: 'src/purr.ts',
                  loadedNow: false,
                  coldNow: 0,
                },
              ],
            },),).toEqual([
              'against /tmp/census.json at c0ffee123: ran 0, still cold 0, cold since then 0, not loaded 0, claimed '
                + 'sources with no stretch there 0, sources edited since then 2',
              '  edited since c0ffee123, so its baseline lines name other code; this run loaded it and left 2 cold '
                + 'stretches: src/nap.ts',
              '  edited since c0ffee123, so its baseline lines name other code; this run did not load it: src/purr.ts',
            ],);
          },
        },),
        it({
          name: 'COUNTS AND NAMES EACH CLAIMED SOURCE WITH NO BASELINE STRETCH, saying whether the baseline ran it whole '
            + 'or never loaded it, and whether this run loaded it and how many cold stretches it left',
          fn: async () => {
            expect(baselineReportLines({
              path: '/tmp/census.json',
              head: 'c0ffee123',
              statuses: [],
              coldSince: [],
              editedClaims: [],
              emptyClaims: [
                {
                  source: 'src/knead.ts',
                  loadedAtBaseline: false,
                  loadedNow: true,
                  coldNow: 0,
                },
                {
                  source: 'src/purr.ts',
                  loadedAtBaseline: true,
                  loadedNow: true,
                  coldNow: 2,
                },
                {
                  source: 'src/yawn.ts',
                  loadedAtBaseline: false,
                  loadedNow: false,
                  coldNow: 0,
                },
              ],
            },),).toEqual([
              'against /tmp/census.json at c0ffee123: ran 0, still cold 0, cold since then 0, not loaded 0, claimed '
                + 'sources with no stretch there 3, sources edited since then 0',
              '  no baseline stretch (not loaded there, so the baseline proves nothing of it); '
                + 'this run loaded it and left 0 cold stretches: src/knead.ts',
              '  no baseline stretch (ran whole there); this run loaded it and left 2 cold stretches: src/purr.ts',
              '  no baseline stretch (not loaded there, so the baseline proves nothing of it); '
                + 'this run did not load it: src/yawn.ts',
            ],);
          },
        },),
      ],
    },),
  ],
},);
