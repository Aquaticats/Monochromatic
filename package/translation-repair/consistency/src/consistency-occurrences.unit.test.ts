/**
 Tests for telling where a text stands inside another by plain equality of
 characters: nowhere, exactly once, or more than once, overlapping stands
 included.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { occurrencesOf, } from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: occurrencesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS NOWHERE FOR A TEXT THE OTHER DOES NOT HOLD, and the offset of the one place it stands at the start, '
            + 'in the middle and at the end',
          fn: async () => {
            expect([
              occurrencesOf({ haystack: 'The cat naps.', needle: 'dog', },),
              occurrencesOf({ haystack: 'The cat naps.', needle: 'The', },),
              occurrencesOf({ haystack: 'The cat naps.', needle: 'cat', },),
              occurrencesOf({ haystack: 'The cat naps.', needle: 'naps.', },),
              occurrencesOf({ haystack: '', needle: 'cat', },),
            ],).toEqual([
              { kind: 'none', },
              { kind: 'once', at: 0, },
              { kind: 'once', at: 4, },
              { kind: 'once', at: 8, },
              { kind: 'none', },
            ],);
          },
        },),
        it({
          name: 'SAYS MANY FOR A TEXT THAT STANDS MORE THAN ONCE, overlapping stands included',
          fn: async () => {
            expect([
              occurrencesOf({ haystack: 'The cat naps. The cat naps.', needle: 'The cat', },),
              occurrencesOf({ haystack: 'aaa', needle: 'aa', },),
              occurrencesOf({ haystack: 'cat cat', needle: 'cat', },),
            ],).toEqual([
              { kind: 'many', },
              { kind: 'many', },
              { kind: 'many', },
            ],);
          },
        },),
      ],
    },),
  ],
},);
