import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  applyChangeset,
  type Changeset,
  composeChangesets,
  invertChangeset,
  mapOffsetThroughChangeset,
  NOT_COMPOSABLE,
} from './changeset.ts';

await describe({
  name: '',
  children: [
    describe({
      name: applyChangeset.name,
      children: [
        it({
          name: 'inserts at the start',
          fn: async () => {
            expect(applyChangeset({
              changeset: {
                from: 0,
                to: 0,
                insert: 'x',
              },
              before: 'ab',
            },),)
              .toBe('xab',);
          },
        },),

        it({
          name: 'inserts at the end',
          fn: async () => {
            expect(applyChangeset({
              changeset: {
                from: 2,
                to: 2,
                insert: 'x',
              },
              before: 'ab',
            },),)
              .toBe('abx',);
          },
        },),

        it({
          name: 'replaces a substring',
          fn: async () => {
            expect(applyChangeset({
              changeset: {
                from: 1,
                to: 4,
                insert: 'XYZ',
              },
              before: 'abcde',
            },),)
              .toBe('aXYZe',);
          },
        },),

        it({
          name: 'pure delete (insert empty) shrinks the string',
          fn: async () => {
            expect(applyChangeset({
              changeset: {
                from: 1,
                to: 3,
                insert: '',
              },
              before: 'abcde',
            },),)
              .toBe('ade',);
          },
        },),

        it({
          name: 'is pure: does not mutate the input string',
          fn: async () => {
            const before = 'hello';
            applyChangeset({
              changeset: {
                from: 0,
                to: 5,
                insert: 'world',
              },
              before,
            },);
            expect(before,).toBe('hello',);
          },
        },),
      ],
    },),

    describe({
      name: invertChangeset.name,
      children: [
        it({
          name: 'inverse + apply restores the original buffer',
          fn: async () => {
            const before = 'helloworld';
            const cs: Changeset = {
              from: 5,
              to: 10,
              insert: 'x',
            };
            const after = applyChangeset({
              changeset: cs,
              before,
            },);
            const inv = invertChangeset({
              changeset: cs,
              before,
            },);
            expect(applyChangeset({
              changeset: inv,
              before: after,
            },),)
              .toBe(before,);
          },
        },),

        it({
          name: 'inverse of pure insert is a delete',
          fn: async () => {
            const inv = invertChangeset({
              changeset: {
                from: 2,
                to: 2,
                insert: 'XY',
              },
              before: 'abcd',
            },);
            expect(inv,).toEqual({
              from: 2,
              to: 4,
              insert: '',
            },);
          },
        },),

        it({
          name: 'inverse of pure delete is an insert of the removed text',
          fn: async () => {
            const inv = invertChangeset({
              changeset: {
                from: 1,
                to: 3,
                insert: '',
              },
              before: 'abcde',
            },);
            expect(inv,).toEqual({
              from: 1,
              to: 1,
              insert: 'bc',
            },);
          },
        },),

        it({
          name: 'inverse of inverse equals the original (round-trip)',
          fn: async () => {
            const before = 'abcdef';
            const cs: Changeset = {
              from: 2,
              to: 4,
              insert: 'XYZ',
            };
            const after = applyChangeset({
              changeset: cs,
              before,
            },);
            const inv = invertChangeset({
              changeset: cs,
              before,
            },);
            const inv2 = invertChangeset({
              changeset: inv,
              before: after,
            },);
            expect(inv2,).toEqual(cs,);
          },
        },),
      ],
    },),

    describe({
      name: mapOffsetThroughChangeset.name,
      children: [
        it({
          name: 'leaves offsets before the window unchanged',
          fn: async () => {
            expect(mapOffsetThroughChangeset({
              changeset: {
                from: 5,
                to: 8,
                insert: 'X',
              },
              offset: 3,
            },),)
              .toBe(3,);
          },
        },),

        it({
          name: 'collapses an in-window offset to from + insert.length',
          fn: async () => {
            expect(mapOffsetThroughChangeset({
              changeset: {
                from: 5,
                to: 10,
                insert: 'XY',
              },
              offset: 7,
            },),)
              .toBe(7,);
            // boundaries
            expect(mapOffsetThroughChangeset({
              changeset: {
                from: 5,
                to: 10,
                insert: 'XY',
              },
              offset: 5,
            },),)
              .toBe(5,);
            expect(mapOffsetThroughChangeset({
              changeset: {
                from: 5,
                to: 10,
                insert: 'XY',
              },
              offset: 10,
            },),)
              .toBe(7,);
          },
        },),

        it({
          name: 'shifts post-window offsets by net length delta',
          fn: async () => {
            // Positive delta: insert grows the buffer.
            expect(mapOffsetThroughChangeset({
              changeset: {
                from: 2,
                to: 4,
                insert: 'XYZ',
              },
              offset: 6,
            },),)
              .toBe(7,);
            // Negative delta: pure delete.
            expect(mapOffsetThroughChangeset({
              changeset: {
                from: 2,
                to: 5,
                insert: '',
              },
              offset: 8,
            },),)
              .toBe(5,);
          },
        },),
      ],
    },),

    describe({
      name: composeChangesets.name,
      children: [
        it({
          name: 'composes successive single-char inserts at the same caret',
          fn: async () => {
            const composed = composeChangesets({
              a: {
                from: 5,
                to: 5,
                insert: 'h',
              },
              b: {
                from: 6,
                to: 6,
                insert: 'i',
              },
            },);
            expect(composed,).toEqual({
              from: 5,
              to: 5,
              insert: 'hi',
            },);
          },
        },),

        it({
          name: 'composition is associative for chained single-char inserts',
          fn: async () => {
            const a: Changeset = {
              from: 0,
              to: 0,
              insert: 'a',
            };
            const b: Changeset = {
              from: 1,
              to: 1,
              insert: 'b',
            };
            const c: Changeset = {
              from: 2,
              to: 2,
              insert: 'c',
            };
            const ab = composeChangesets({
              a,
              b,
            },);
            const bc = composeChangesets({
              a: b,
              b: c,
            },);
            if ((ab === NOT_COMPOSABLE) || (bc === NOT_COMPOSABLE))
              throw new Error('expected composable inserts',);
            const left = composeChangesets({
              a: ab,
              b: c,
            },);
            const right = composeChangesets({
              a,
              b: bc,
            },);
            expect(left,).toEqual(right,);
          },
        },),

        it({
          name: 'returns NOT_COMPOSABLE when b is not adjacent to a (gap in caret)',
          fn: async () => {
            const composed = composeChangesets({
              a: {
                from: 5,
                to: 5,
                insert: 'h',
              },
              b: {
                from: 8,
                to: 8,
                insert: 'i',
              },
            },);
            expect(composed,).toBe(NOT_COMPOSABLE,);
          },
        },),

        it({
          name: 'returns NOT_COMPOSABLE when b is not a pure insert',
          fn: async () => {
            const composed = composeChangesets({
              a: {
                from: 5,
                to: 5,
                insert: 'h',
              },
              b: {
                from: 6,
                to: 7,
                insert: '',
              },
            },);
            expect(composed,).toBe(NOT_COMPOSABLE,);
          },
        },),

        it({
          name: 'composed result, when applied, equals applying a then b',
          fn: async () => {
            const before = 'hello';
            const a: Changeset = {
              from: 5,
              to: 5,
              insert: 'X',
            };
            const after1 = applyChangeset({
              changeset: a,
              before,
            },);
            const b: Changeset = {
              from: 6,
              to: 6,
              insert: 'Y',
            };
            const after2 = applyChangeset({
              changeset: b,
              before: after1,
            },);
            const composed = composeChangesets({
              a,
              b,
            },);
            if (composed === NOT_COMPOSABLE)
              throw new Error('expected composable inserts',);
            expect(applyChangeset({
              changeset: composed,
              before,
            },),)
              .toBe(after2,);
          },
        },),
      ],
    },),
  ],
},);
