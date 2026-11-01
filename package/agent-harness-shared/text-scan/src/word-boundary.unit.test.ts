import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  containsAnyOfWordBounded,
  containsWordBoundedPhrase,
  PHRASE_NOT_FOUND,
} from './index.ts';

await describe({
  name: 'text-scan word-boundary phrase lookup',
  children: [
    describe({
      name: containsWordBoundedPhrase.name,
      children: [
        it({
          name: 'matches phrase at the start of input',
          fn: async () => {
            expect(containsWordBoundedPhrase({
              haystack: 'maybe tomorrow',
              phrase: 'maybe',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'matches phrase at the end of input',
          fn: async () => {
            expect(containsWordBoundedPhrase({
              haystack: 'tomorrow maybe',
              phrase: 'maybe',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'matches phrase in the middle case-insensitively',
          fn: async () => {
            expect(containsWordBoundedPhrase({
              haystack: 'Yes, MAYBE tomorrow',
              phrase: 'maybe',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'rejects matches embedded in larger words',
          fn: async () => {
            expect(containsWordBoundedPhrase({
              haystack: 'maybenot',
              phrase: 'maybe',
            },),).toBe(false,);
            expect(containsWordBoundedPhrase({
              haystack: 'methinks',
              phrase: 'think',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'matches phrase containing internal spaces',
          fn: async () => {
            expect(containsWordBoundedPhrase({
              haystack: 'I think so',
              phrase: 'i think',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'treats punctuation as a boundary',
          fn: async () => {
            expect(containsWordBoundedPhrase({
              haystack: "That's demonstrably false.",
              phrase: 'demonstrably false',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'returns false for empty phrase',
          fn: async () => {
            expect(containsWordBoundedPhrase({
              haystack: 'anything',
              phrase: '',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'works when phrase starts or ends with non-word char',
          fn: async () => {
            expect(containsWordBoundedPhrase({
              haystack: 'see AGENTS.md for details',
              phrase: 'agents.md',
            },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: containsAnyOfWordBounded.name,
      children: [
        it({
          name: 'returns the first matching phrase',
          fn: async () => {
            expect(containsAnyOfWordBounded({
              haystack: 'maybe later',
              phrases: ['probably', 'maybe', 'perhaps',],
            },),).toEqual({ phrase: 'maybe', },);
          },
        },),
        it({
          name: 'returns PHRASE_NOT_FOUND when no phrase matches',
          fn: async () => {
            expect(containsAnyOfWordBounded({
              haystack: 'the deploy succeeded',
              phrases: ['probably', 'maybe', 'perhaps',],
            },),).toBe(PHRASE_NOT_FOUND,);
          },
        },),
      ],
    },),
  ],
},);
