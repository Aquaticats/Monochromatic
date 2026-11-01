import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { containsBoundedAccessTag, } from './tag-validation.ts';

/** Count of near-miss repetitions used to exercise the walk across many candidates. */
const LONG_RUN = 1_000;

/** Tag reused across the boundary cases below. */
const TAG = '@public';

await describe({
  name: containsBoundedAccessTag.name,
  children: [
    it({
      name: 'returns false for empty text',
      fn: async () => {
        expect(containsBoundedAccessTag({ text: '', tag: TAG, },),).toBe(false,);
      },
    },),
    it({
      name: 'returns false when the tag does not occur',
      fn: async () => {
        expect(containsBoundedAccessTag({ text: 'mypublic', tag: TAG, },),).toBe(false,);
      },
    },),
    it({
      name: 'returns true for a whitespace-bounded tag',
      fn: async () => {
        expect(containsBoundedAccessTag({ text: 'see @public here', tag: TAG, },),).toBe(true,);
      },
    },),
    it({
      name: 'returns true when the tag spans the whole string',
      fn: async () => {
        expect(containsBoundedAccessTag({ text: '@public', tag: TAG, },),).toBe(true,);
      },
    },),
    it({
      name: 'returns false when the preceding char is not a boundary',
      fn: async () => {
        expect(containsBoundedAccessTag({ text: 'x@public', tag: TAG, },),).toBe(false,);
      },
    },),
    it({
      name: 'returns false when the following char is not a boundary',
      fn: async () => {
        expect(containsBoundedAccessTag({ text: '@publicfoo', tag: TAG, },),).toBe(false,);
      },
    },),
    it({
      name: 'treats a trailing asterisk as a valid boundary',
      fn: async () => {
        expect(containsBoundedAccessTag({ text: '@public*', tag: TAG, },),).toBe(true,);
      },
    },),
    it({
      name: 'treats tab and newline as valid boundaries',
      fn: async () => {
        expect(containsBoundedAccessTag({ text: '\t@public\n', tag: TAG, },),).toBe(true,);
      },
    },),
    it({
      name: 'advances past an invalid candidate to find a later valid one',
      fn: async () => {
        expect(containsBoundedAccessTag({ text: 'x@publicfoo @public', tag: TAG, },),).toBe(true,);
      },
    },),
    it({
      name: 'returns false across a long run of invalid candidates',
      fn: async () => {
        expect(
          containsBoundedAccessTag({ text: 'x@public'.repeat(LONG_RUN,), tag: TAG, },),
        ).toBe(false,);
      },
    },),
    it({
      name: 'returns true when a valid candidate follows a long run of invalid ones',
      fn: async () => {
        expect(
          containsBoundedAccessTag({ text: `${'x@public'.repeat(LONG_RUN,)} @public`, tag: TAG, },),
        ).toBe(true,);
      },
    },),
  ],
},);
