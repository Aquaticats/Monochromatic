import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  containsErThanMost,
  ER_NOT_FOUND,
  findErThanMost,
} from './uncertainty-phrases.ts';

await describe({
  name: 'uncertainty-phrases comparative scan',
  children: [
    describe({
      name: findErThanMost.name,
      children: [
        it({
          name: 'returns the matched fragment for a simple comparative',
          fn: async () => {
            expect(findErThanMost('this is bigger than most lengths',),).toBe(
              'bigger than most',
            );
          },
        },),
        it({
          name: 'matches case-insensitively and preserves original case',
          fn: async () => {
            expect(findErThanMost('FASTER THAN MOST options',),).toBe('FASTER THAN MOST',);
          },
        },),
        it({
          name: 'returns ER_NOT_FOUND when there is no word prefix',
          fn: async () => {
            expect(findErThanMost('just er than most',),).toBe(ER_NOT_FOUND,);
          },
        },),
        it({
          name: 'returns ER_NOT_FOUND when the phrase begins the string',
          fn: async () => {
            expect(findErThanMost('er than most',),).toBe(ER_NOT_FOUND,);
          },
        },),
        it({
          name: 'returns ER_NOT_FOUND when a trailing word char breaks the boundary',
          fn: async () => {
            expect(findErThanMost('faster than mostly',),).toBe(ER_NOT_FOUND,);
          },
        },),
        it({
          name: 'walks back to the start of the comparative word',
          fn: async () => {
            expect(findErThanMost('it was inner than most circles',),).toBe('inner than most',);
          },
        },),
        it({
          name: 'returns ER_NOT_FOUND for the empty string',
          fn: async () => {
            expect(findErThanMost('',),).toBe(ER_NOT_FOUND,);
          },
        },),
        it({
          name: 'returns ER_NOT_FOUND when absent',
          fn: async () => {
            expect(findErThanMost('all good here',),).toBe(ER_NOT_FOUND,);
          },
        },),
        it({
          name: 'finds the match after many failing occurrences (linear scan)',
          fn: async () => {
            // every ` er than most` is space-prefixed (no word char) so all fail,
            // except the final word-prefixed one.
            const text = `${' er than most'.repeat(20_000,)} bigger than most`;
            expect(findErThanMost(text,),).toBe('bigger than most',);
          },
        },),
      ],
    },),
    describe({
      name: containsErThanMost.name,
      children: [
        it({
          name: 'returns true when a comparative hedge is present',
          fn: async () => {
            expect(containsErThanMost('this is faster than most options',),).toBe(true,);
          },
        },),
        it({
          name: 'returns false when absent',
          fn: async () => {
            expect(containsErThanMost('all good',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
