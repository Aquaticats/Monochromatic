/**
 Tests for the difference walk behind `toEqual` failure rendering:
 {@link valueDifferenceLines},
 {@link errorDifferenceLines},
 and the copy-for-copy entry matching they share.

 Every branch of the walk shows up here: list entries one side lacks,
 reordered lists,
 missing and extra and changed record keys,
 map keys and values,
 differing string lines,
 scalar pairs,
 kind mismatches,
 equal-value fallbacks,
 bounded output,
 and self-referential values.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  DEFAULT_MAX_DIFFERENCES,
  createEntryKeyRegistry,
  entryDifference,
  entryKey,
  errorDifferenceLines,
  valueDifferenceLines,
} from '@monochromatic-dev/test-harness-core-diagnostic';

await describe({
  name: '',
  children: [
    //region valueDifferenceLines

    describe({
      name: valueDifferenceLines.name,
      children: [
        //region Lists

        it({
          name: 'names entries one list holds and the other lacks',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: [1, 2,],
              expected: [1, 3,],
            },),).toEqual([
              'only in actual: 2',
              'only in expected: 3',
            ],);
          },
        },),
        it({
          name: 'names one changed entry in a long list instead of both lists',
          fn: async () => {
            /**
             Width of each finding in the acceptance scenario,
             past chai's truncation threshold.
             */
            const FINDING_WIDTH = 174;
            /**
             Finding count in the acceptance scenario.
             */
            const FINDING_COUNT = 31;
            /**
             Listings as the assertion expected them.
             */
            const listed = Array.from(
              {
                length: FINDING_COUNT,
              },
              function listedFinding(_, index,) {
                return `cat.ts#nap${String(index,)
                  .padStart(2, '0',)}#`
                  .padEnd(FINDING_WIDTH, 'x',);
              },
            );
            /**
             Findings as the assertion found them,
             with one entry changed.
             */
            const found = listed.map(function changedFinding(entry, index,) {
              return index === 17
                ? `${entry.slice(0, FINDING_WIDTH - 4,)}DIFF`
                : entry;
            },);
            /**
             Difference lines for the one changed entry.
             */
            const lines = valueDifferenceLines({
              actual: found,
              expected: listed,
            },);
            expect(lines.length,)
              .toBe(2,);
            expect(lines[0],)
              .toBe(`only in actual: ${JSON.stringify(found[17],)}`,);
            expect(lines[1],)
              .toBe(`only in expected: ${JSON.stringify(listed[17],)}`,);
          },
        },),
        it({
          name: 'pairs duplicate entries copy for copy',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: [1, 1, 2,],
              expected: [1, 2, 2,],
            },),).toEqual([
              'only in actual: 1',
              'only in expected: 2',
            ],);
          },
        },),
        it({
          name: 'names diverging positions when the same entries sit in another order',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: [2, 1,],
              expected: [1, 2,],
            },),).toEqual([
              'the same entries sit in another order',
              'at position 0: actual 2 and expected 1',
              'at position 1: actual 1 and expected 2',
            ],);
          },
        },),
        it({
          name: 'says nothing structural when an unordered kind only reordered',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: new Set([1, 2,],),
              expected: new Set([2, 1,],),
            },),).toEqual([
              'no structural difference found: the values are deeply equal',
            ],);
          },
        },),

        //endregion Lists

        //region Records

        it({
          name: 'names a changed record key under its key',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: {
                a: 1,
              },
              expected: {
                a: 2,
              },
            },),).toEqual([
              'key "a": actual 1 and expected 2',
            ],);
          },
        },),
        it({
          name: 'names missing and extra record keys',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: {
                a: 1,
              },
              expected: {
                b: 1,
              },
            },),).toEqual([
              'key "a": only in actual: 1',
              'key "b": only in expected: 1',
            ],);
          },
        },),
        it({
          name: 'prefixes nested record differences with each key',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: {
                a: {
                  b: 1,
                },
              },
              expected: {
                a: {
                  b: 2,
                },
              },
            },),).toEqual([
              'key "a": key "b": actual 1 and expected 2',
            ],);
          },
        },),
        it({
          name: 'reports a shared value under both sibling keys holding it',
          fn: async () => {
            /**
             Value both actual keys share.
             */
            const found = {
              m: 1,
            };
            /**
             Value both expected keys share.
             */
            const listed = {
              m: 2,
            };
            expect(valueDifferenceLines({
              actual: {
                a: found,
                b: found,
              },
              expected: {
                a: listed,
                b: listed,
              },
            },),).toEqual([
              'key "a": key "m": actual 1 and expected 2',
              'key "b": key "m": actual 1 and expected 2',
            ],);
          },
        },),
        it({
          name: 'pairs record keys by symbol identity',
          fn: async () => {
            /**
             Symbol key both records hold.
             */
            const shared = Symbol('shared key held by both records',);
            expect(valueDifferenceLines({
              actual: {
                [shared]: 1,
              },
              expected: {
                [shared]: 2,
              },
            },),).toEqual([
              'key Symbol(shared key held by both records): actual 1 and expected 2',
            ],);
          },
        },),
        it({
          name: 'names equal-description symbol keys as separate keys',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: {
                [Symbol('equal description distinct identity',)]: 1,
              },
              expected: {
                [Symbol('equal description distinct identity',)]: 2,
              },
            },),).toEqual([
              'key Symbol(equal description distinct identity): only in actual: 1',
              'key Symbol(equal description distinct identity): only in expected: 2',
            ],);
          },
        },),

        //endregion Records

        //region Maps

        it({
          name: 'names a changed map value under its key',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: new Map([['k', 1,],],),
              expected: new Map([['k', 2,],],),
            },),).toEqual([
              '"k": actual 1 and expected 2',
            ],);
          },
        },),
        it({
          name: 'names missing and extra map keys with their values',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: new Map([['a', 1,],],),
              expected: new Map([['b', 2,],],),
            },),).toEqual([
              'only in actual: "a" => 1',
              'only in expected: "b" => 2',
            ],);
          },
        },),
        it({
          name: 'pairs deep-equal map keys without shared identity',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: new Map([[{
                k: 1,
              }, 'a',],],),
              expected: new Map([[{
                k: 1,
              }, 'b',],],),
            },),).toEqual([
              '{ "k": 1 }: string differs at line 1 (line counts: actual 1, expected 1)',
              '{ "k": 1 }: line 1: actual a',
              '{ "k": 1 }: line 1: expected b',
            ],);
          },
        },),

        //endregion Maps

        //region Strings

        it({
          name: 'names the differing string lines with surrounding context',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: 'a\nx\nz',
              expected: 'a\ny\nz',
            },),).toEqual([
              'string differs at line 2 (line counts: actual 3, expected 3)',
              'context line 1: a',
              'line 2: actual x',
              'line 2: expected y',
              'context line 3: z',
            ],);
          },
        },),
        it({
          name: 'names extra string lines on the side holding them',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: 'a\nb',
              expected: 'a\nb\nc',
            },),).toEqual([
              'string differs at line 3 (line counts: actual 2, expected 3)',
              'context line 2: b',
              'line 3: expected c',
            ],);
          },
        },),

        //endregion Strings

        //region Leaves and kinds

        it({
          name: 'renders both leaves when their renderings differ',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: 2,
              expected: 3,
            },),).toEqual([
              'actual 2 and expected 3',
            ],);
          },
        },),
        it({
          name: 'keeps the sign of zero in both leaves',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: -0,
              expected: 0,
            },),).toEqual([
              'actual -0 and expected +0',
            ],);
          },
        },),
        it({
          name: 'names a reference difference when equal renderings hide it',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: function f() { /* one body */ },
              expected: function f() { /* another body */ },
            },),).toEqual([
              'different references with the same rendering: [Function: f]',
            ],);
          },
        },),
        it({
          name: 'names carried error fields on both sides',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: Object.assign(new Error('m',), {
                code: 1,
              },),
              expected: Object.assign(new Error('m',), {
                code: 2,
              },),
            },),).toEqual([
              'actual Error: m { "code": 1 } and expected Error: m { "code": 2 }',
            ],);
          },
        },),
        it({
          name: 'names both sides of a kind mismatch by kind and size',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: [1,],
              expected: {
                a: 1,
              },
            },),).toEqual([
              'kind differs: actual is Array of 1 items, expected is Object with 1 keys',
            ],);
          },
        },),

        //endregion Leaves and kinds

        //region Equality fallbacks

        it({
          name: 'falls back to the same value when nothing differs by identity',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: Number.NaN,
              expected: Number.NaN,
            },),).toEqual([
              'no difference found: actual and expected are the same value',
            ],);
          },
        },),
        it({
          name: 'falls back to deep equality when equal values differ by reference',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: [1,],
              expected: [1,],
            },),).toEqual([
              'no structural difference found: the values are deeply equal',
            ],);
          },
        },),

        //endregion Equality fallbacks

        //region Bounded output

        it({
          name: 'caps rendered lines and names the suppressed count',
          fn: async () => {
            expect(valueDifferenceLines({
              actual: [1, 2, 3, 4,],
              expected: [],
              maxDifferences: 2,
            },),).toEqual([
              'only in actual: 1',
              'only in actual: 2',
              'and 2 more differences',
            ],);
          },
        },),
        it({
          name: 'exports a default cap generous enough for realistic scan failures',
          fn: async () => {
            expect(DEFAULT_MAX_DIFFERENCES,)
              .toBe(100,);
          },
        },),

        //endregion Bounded output

        //region Cycles

        it({
          name: 'stops on a revisited self-referential pair',
          fn: async () => {
            /**
             Self-referential actual value.
             */
            const found = {
              v: 1,
              self: undefined as unknown,
            };
            found.self = found;
            /**
             Self-referential expected value.
             */
            const listed = {
              v: 2,
              self: undefined as unknown,
            };
            listed.self = listed;
            expect(valueDifferenceLines({
              actual: found,
              expected: listed,
            },),).toEqual([
              'key "v": actual 1 and expected 2',
            ],);
          },
        },),

        //endregion Cycles
      ],
    },),

    //endregion valueDifferenceLines

    //region errorDifferenceLines

    describe({
      name: errorDifferenceLines.name,
      children: [
        it({
          name: 'stays silent for an error comparing no values',
          fn: async () => {
            expect(errorDifferenceLines({
              error: new Error('boom',),
            },),).toEqual([],);
          },
        },),
        it({
          name: 'stays silent when chai marked the fields uncomparable',
          fn: async () => {
            expect(errorDifferenceLines({
              error: {
                actual: [1,],
                expected: [2,],
                showDiff: false,
              },
            },),).toEqual([],);
          },
        },),
        it({
          name: 'stays silent when neither side can hold truncated content',
          fn: async () => {
            expect(errorDifferenceLines({
              error: {
                actual: false,
                expected: true,
                showDiff: true,
              },
            },),).toEqual([],);
          },
        },),
        it({
          name: 'stays silent for a toThrow failure that threw nothing',
          fn: async () => {
            expect(errorDifferenceLines({
              error: {
                actual: undefined,
                expected: null,
              },
            },),).toEqual([],);
          },
        },),
        it({
          name: 'walks compared containers under the error',
          fn: async () => {
            expect(errorDifferenceLines({
              error: {
                actual: [1,],
                expected: [2,],
              },
            },),).toEqual([
              'only in actual: 1',
              'only in expected: 2',
            ],);
          },
        },),
        it({
          name: 'walks compared strings under the error',
          fn: async () => {
            expect(errorDifferenceLines({
              error: {
                actual: 'm',
                expected: 'x',
              },
            },),).toEqual([
              'string differs at line 1 (line counts: actual 1, expected 1)',
              'line 1: actual m',
              'line 1: expected x',
            ],);
          },
        },),
        it({
          name: 'names both sides when only one side can hold long content',
          fn: async () => {
            expect(errorDifferenceLines({
              error: {
                actual: 1,
                expected: [2,],
              },
            },),).toEqual([
              'kind differs: actual is 1, expected is Array of 1 items',
            ],);
          },
        },),
        it({
          name: 'names a reference difference between compared functions',
          fn: async () => {
            expect(errorDifferenceLines({
              error: {
                actual: function f() { /* one body */ },
                expected: function f() { /* another body */ },
              },
            },),).toEqual([
              'different references with the same rendering: [Function: f]',
            ],);
          },
        },),
      ],
    },),

    //endregion errorDifferenceLines

    //region entryDifference

    describe({
      name: entryDifference.name,
      children: [
        it({
          name: 'pairs equal entries across sides and names both leftovers',
          fn: async () => {
            expect(entryDifference({
              actual: [1, 1,],
              expected: [1, 2,],
              registry: createEntryKeyRegistry(),
            },),).toEqual({
              actualNotExpected: [1,],
              expectedNotActual: [2,],
              reordered: false,
            },);
          },
        },),
        it({
          name: 'reports order alone when every entry paired',
          fn: async () => {
            expect(entryDifference({
              actual: [2, 1,],
              expected: [1, 2,],
              registry: createEntryKeyRegistry(),
            },),).toEqual({
              actualNotExpected: [],
              expectedNotActual: [],
              reordered: true,
            },);
          },
        },),
        it({
          name: 'reads two empty lists as equal in content and order',
          fn: async () => {
            expect(entryDifference({
              actual: [],
              expected: [],
              registry: createEntryKeyRegistry(),
            },),).toEqual({
              actualNotExpected: [],
              expectedNotActual: [],
              reordered: false,
            },);
          },
        },),
      ],
    },),

    //endregion entryDifference

    //region entryKey

    describe({
      name: entryKey.name,
      children: [
        it({
          name: 'keys NaN with itself and keeps zero signs apart',
          fn: async () => {
            /**
             Shared identity table for reference kinds.
             */
            const registry = createEntryKeyRegistry();
            expect(entryKey({
              value: Number.NaN,
              registry,
            },),).toBe(entryKey({
              value: Number.NaN,
              registry,
            },),);
            expect(entryKey({
              value: -0,
              registry,
            },),).not.toBe(entryKey({
              value: 0,
              registry,
            },),);
          },
        },),
        it({
          name: 'keys symbols by identity, not description',
          fn: async () => {
            /**
             Shared identity table for reference kinds.
             */
            const registry = createEntryKeyRegistry();
            /**
             Symbol key held by both sides.
             */
            const shared = Symbol('shared key held by both records',);
            expect(entryKey({
              value: shared,
              registry,
            },),).toBe(entryKey({
              value: shared,
              registry,
            },),);
            expect(entryKey({
              value: Symbol('equal description distinct identity',),
              registry,
            },),).not.toBe(entryKey({
              value: Symbol('equal description distinct identity',),
              registry,
            },),);
          },
        },),
      ],
    },),

    //endregion entryKey
  ],
},);
