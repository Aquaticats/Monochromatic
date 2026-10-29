/**
 Tests for the settlement that refuses edits losing markup no sibling writes.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  markupDelta,
  markupSourceKeys,
  settleMarkupMoves,
  type MarkupDelta,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: settleMarkupMoves.name,
      children: [
        it({
          name: 'KEEPS an edit whose losses a standing sibling writes, since one written atom excuses '
            + 'one loss',
          fn: async () => {
            /**
             One edit losing the tag its sibling writes.
             */
            const deltas: readonly MarkupDelta[] = [
              {
                unexcused: [{ kind: 'tag', value: '<br />', },],
                gained: [],
              },
              {
                unexcused: [],
                gained: [{ kind: 'tag', value: '<br />', },],
              },
            ];
            expect(settleMarkupMoves({ deltas, },),).toEqual([[], []],);
          },
        },),

        it({
          name: 'REFUSES both edits when two lose the same key and one sibling writes it, since nothing says '
            + 'which loss the sibling answered, and keeps the sibling',
          fn: async () => {
            /**
             Two edits losing the same tag and a third writing it once.
             */
            const deltas: readonly MarkupDelta[] = [
              {
                unexcused: [{ kind: 'tag', value: '<br />', },],
                gained: [],
              },
              {
                unexcused: [{ kind: 'tag', value: '<br />', },],
                gained: [],
              },
              {
                unexcused: [],
                gained: [{ kind: 'tag', value: '<br />', },],
              },
            ];
            expect(settleMarkupMoves({ deltas, },),).toEqual([['tag',], ['tag',], [],],);
          },
        },),
      ],
    },),

    describe({
      name: markupSourceKeys.name,
      children: [
        it({
          name: 'READS the key of each atom a source text carries, its kind and its value joined by a NUL',
          fn: async () => {
            expect([...markupSourceKeys({ sourceText: 'a `c` <br />', },),],).toEqual([
              'inline-code\u0000`c`',
              'tag\u0000<br />',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: markupDelta.name,
      children: [
        it({
          name: 'READS a loss nothing excuses as unexcused and a write nothing consumes as gained',
          fn: async () => {
            expect(markupDelta({
              before: '<br />',
              after: '',
              removableQuotes: [],
              sourceKeys: new Set<string>(),
            },),).toEqual({
              unexcused: [{ kind: 'tag', value: '<br />', },],
              gained: [],
            },);
            expect(markupDelta({
              before: '',
              after: '<br />',
              removableQuotes: [],
              sourceKeys: new Set<string>(),
            },),).toEqual({
              unexcused: [],
              gained: [{ kind: 'tag', value: '<br />', },],
            },);
          },
        },),

        it({
          name: 'PAIRS a loss with the same-kind write that re-marks it, both sides leaving the delta',
          fn: async () => {
            expect(markupDelta({
              before: '<br />',
              after: '<x />',
              removableQuotes: [],
              sourceKeys: new Set<string>(),
            },),).toEqual({
              unexcused: [],
              gained: [],
            },);
          },
        },),

        it({
          name: 'LET GO a loss the envelope\'s removable quote carries, since the claim may take it',
          fn: async () => {
            expect(markupDelta({
              before: '<br />',
              after: '',
              removableQuotes: ['<br />',],
              sourceKeys: new Set<string>(),
            },),).toEqual({
              unexcused: [],
              gained: [],
            },);
          },
        },),

        it({
          name: 'HOLDS a loss the source itself carries as unexcused however the edit writes, since the '
            + 'markup was copied rather than authored',
          fn: async () => {
            expect(markupDelta({
              before: '<br />',
              after: '<x />',
              removableQuotes: [],
              sourceKeys: markupSourceKeys({ sourceText: '<br />', },),
            },),).toEqual({
              unexcused: [{ kind: 'tag', value: '<br />', },],
              gained: [{ kind: 'tag', value: '<x />', },],
            },);
          },
        },),

        it({
          name: 'PAIRS a loss with a write the source carries of another kind, where no same-kind write '
            + 'stands beside it',
          fn: async () => {
            expect(markupDelta({
              before: '`c`',
              after: '<br />',
              removableQuotes: [],
              sourceKeys: markupSourceKeys({ sourceText: '<br />', },),
            },),).toEqual({
              unexcused: [],
              gained: [],
            },);
          },
        },),
      ],
    },),
  ],
},);
