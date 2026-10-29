import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  applyToTable,
  materialise,
  resetTable,
  splitAt,
  substring,
  type Table,
} from './buffer-table.ts';
import type { Changeset, } from './changeset.ts';

/**
 * Builds a fresh table seeded with `text`.
 *
 * @param text - initial document text
 *
 * @returns initialised table
 */
function freshTable(text: string,): Table {
  const table: Table = {
    original: '',
    add: '',
    pieces: [],
    length: 0,
  };
  resetTable({
    table,
    text,
  },);
  return table;
}

await describe({
  name: '',
  children: [
    describe({
      name: resetTable.name,
      children: [
        it({
          name: 'creates one piece pointing at the original buffer for non-empty input',
          fn: async () => {
            const table = freshTable('hello',);
            expect(table.pieces,).toHaveLength(1,);
            expect(table.pieces[0]?.source,).toBe('original',);
            expect(table.length,).toBe(5,);
          },
        },),

        it({
          name: 'creates an empty piece list for empty input',
          fn: async () => {
            const table = freshTable('',);
            expect(table.pieces,).toHaveLength(0,);
            expect(table.length,).toBe(0,);
          },
        },),

        it({
          name: 'reset clears the add buffer',
          fn: async () => {
            const table = freshTable('a',);
            applyToTable({
              table,
              changeset: {
                from: 1,
                to: 1,
                insert: 'XYZ',
              },
            },);
            expect(table.add,).toBe('XYZ',);
            resetTable({
              table,
              text: 'fresh',
            },);
            expect(table.add,).toBe('',);
            expect(materialise({ table, },),).toBe('fresh',);
          },
        },),
      ],
    },),

    describe({
      name: materialise.name,
      children: [
        it({
          name: 'returns the full document text',
          fn: async () => {
            const table = freshTable('abc',);
            expect(materialise({ table, },),).toBe('abc',);
          },
        },),

        it({
          name: 'returns the empty string for an empty table',
          fn: async () => {
            const table = freshTable('',);
            expect(materialise({ table, },),).toBe('',);
          },
        },),
      ],
    },),

    describe({
      name: substring.name,
      children: [
        it({
          name: 'returns the requested half-open window',
          fn: async () => {
            const table = freshTable('abcdef',);
            expect(substring({
              table,
              from: 1,
              to: 4,
            },),)
              .toBe('bcd',);
          },
        },),

        it({
          name: 'clamps from below 0 and above length',
          fn: async () => {
            const table = freshTable('abc',);
            expect(substring({
              table,
              from: -5,
              to: 100,
            },),)
              .toBe('abc',);
          },
        },),

        it({
          name: 'returns empty string when from === to',
          fn: async () => {
            const table = freshTable('abc',);
            expect(substring({
              table,
              from: 2,
              to: 2,
            },),)
              .toBe('',);
          },
        },),

        it({
          name: 'reads across multiple pieces correctly',
          fn: async () => {
            const table = freshTable('abXYef',);
            // Splice in: replace 2..4 ('XY') with 'CD'.
            applyToTable({
              table,
              changeset: {
                from: 2,
                to: 4,
                insert: 'CD',
              },
            },);
            expect(materialise({ table, },),).toBe('abCDef',);
            expect(substring({
              table,
              from: 1,
              to: 5,
            },),)
              .toBe('bCDe',);
          },
        },),
      ],
    },),

    describe({
      name: splitAt.name,
      children: [
        it({
          name: 'returns 0 for at === 0',
          fn: async () => {
            const table = freshTable('abc',);
            expect(splitAt({
              table,
              at: 0,
            },),)
              .toBe(0,);
          },
        },),

        it({
          name: 'returns pieces.length for at === length',
          fn: async () => {
            const table = freshTable('abc',);
            expect(splitAt({
              table,
              at: 3,
            },),)
              .toBe(table.pieces.length,);
          },
        },),

        it({
          name: 'splits a piece into two and returns the right index',
          fn: async () => {
            const table = freshTable('abcdef',);
            const idx = splitAt({
              table,
              at: 3,
            },);
            expect(idx,).toBe(1,);
            expect(table.pieces,).toHaveLength(2,);
            expect(table.pieces[0]?.length,).toBe(3,);
            expect(table.pieces[1]?.length,).toBe(3,);
            // Materialised text unchanged.
            expect(materialise({ table, },),).toBe('abcdef',);
          },
        },),
      ],
    },),

    describe({
      name: applyToTable.name,
      children: [
        it({
          name: 'inserts at the start',
          fn: async () => {
            const table = freshTable('def',);
            applyToTable({
              table,
              changeset: {
                from: 0,
                to: 0,
                insert: 'abc',
              },
            },);
            expect(materialise({ table, },),).toBe('abcdef',);
            expect(table.length,).toBe(6,);
          },
        },),

        it({
          name: 'inserts in the middle',
          fn: async () => {
            const table = freshTable('abdef',);
            applyToTable({
              table,
              changeset: {
                from: 2,
                to: 2,
                insert: 'c',
              },
            },);
            expect(materialise({ table, },),).toBe('abcdef',);
          },
        },),

        it({
          name: 'replaces a substring',
          fn: async () => {
            const table = freshTable('abcdef',);
            applyToTable({
              table,
              changeset: {
                from: 2,
                to: 5,
                insert: 'XYZ',
              },
            },);
            expect(materialise({ table, },),).toBe('abXYZf',);
          },
        },),

        it({
          name: 'pure delete shrinks the document',
          fn: async () => {
            const table = freshTable('abcdef',);
            applyToTable({
              table,
              changeset: {
                from: 1,
                to: 4,
                insert: '',
              },
            },);
            expect(materialise({ table, },),).toBe('aef',);
            expect(table.length,).toBe(3,);
          },
        },),

        it({
          name: 'returned inverse, when applied, restores the original document',
          fn: async () => {
            const table = freshTable('hello world',);
            const cs: Changeset = {
              from: 6,
              to: 11,
              insert: 'planet',
            };
            const inv = applyToTable({
              table,
              changeset: cs,
            },);
            expect(materialise({ table, },),).toBe('hello planet',);
            applyToTable({
              table,
              changeset: inv,
            },);
            expect(materialise({ table, },),).toBe('hello world',);
          },
        },),

        it({
          name: 'throws on out-of-range from/to',
          fn: async () => {
            /* oxlint-disable no-restricted-syntax/no-regex -- asserts the buffer table rejects each out-of-range changeset with the documented `invalid changeset` message; literal phrase match, no backtracking */
            const table = freshTable('abc',);
            expect(function bogusFrom() {
              applyToTable({
                table,
                changeset: {
                  from: -1,
                  to: 0,
                  insert: '',
                },
              },);
            },)
              .toThrow(/invalid changeset/u,);
            expect(function bogusOrder() {
              applyToTable({
                table,
                changeset: {
                  from: 2,
                  to: 1,
                  insert: '',
                },
              },);
            },)
              .toThrow(/invalid changeset/u,);
            expect(function bogusTo() {
              applyToTable({
                table,
                changeset: {
                  from: 0,
                  to: 999,
                  insert: '',
                },
              },);
            },)
              .toThrow(/invalid changeset/u,);
            /* oxlint-enable no-restricted-syntax/no-regex */
          },
        },),

        it({
          name: '5000 single-char inserts produces correct text',
          fn: async () => {
            const table = freshTable('',);
            for (let loopIndex = 0; loopIndex < 5_000; loopIndex += 1) {
              applyToTable({
                table,
                changeset: {
                  from: loopIndex,
                  to: loopIndex,
                  insert: 'x',
                },
              },);
            }
            expect(table.length,).toBe(5_000,);
            expect(materialise({ table, },),).toBe('x'.repeat(5_000,),);
            // The piece count grows linearly under naive inserts; the
            // worker schedules a collapse on idle. Here we just verify
            // the data integrity.
          },
        },),

        it({
          name:
            '5000 inserts followed by collapse re-anchors to a single piece (verification 16b)',
          fn: async () => {
            const table = freshTable('',);
            for (let loopIndex = 0; loopIndex < 5_000; loopIndex += 1) {
              applyToTable({
                table,
                changeset: {
                  from: loopIndex,
                  to: loopIndex,
                  insert: 'x',
                },
              },);
            }
            expect(table.pieces.length,).toBeGreaterThan(1,);
            // The buffer worker's `scheduleCollapseIfNeeded` runs the
            // following two-line operation on idle when the piece count
            // exceeds COLLAPSE_THRESHOLD_NODES. Here we drive it
            // directly to assert the collapse reduces to a single
            // piece without losing data.
            resetTable({
              table,
              text: materialise({ table, },),
            },);
            expect(table.pieces,).toHaveLength(1,);
            expect(table.length,).toBe(5_000,);
            expect(materialise({ table, },),).toBe('x'.repeat(5_000,),);
          },
        },),
      ],
    },),
  ],
},);
