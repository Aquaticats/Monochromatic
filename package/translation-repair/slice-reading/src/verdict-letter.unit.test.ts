/**
 Tests for the one-character rule that separates a verdict letter from a word
 beginning with the same letter.

 Both sheet readers depend on this and each exercises it only through its own
 format, so the rule itself is tested here rather than twice at a distance.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  opensWithVerdict,
  trimLeadingDelimiters,
  VERDICT_DELIMITERS,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: opensWithVerdict.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'accepts a letter standing entirely alone, the commonest answer a '
            + 'grader leaves',
          fn: async () => {
            expect(
              opensWithVerdict({
                answer: 'Y',
                letter: 'Y',
              },),
            ).toBe(true,);
          },
        },),

        it({
          name: 'accepts a letter followed by any listed delimiter, so a grader '
            + 'may write a rationale without losing their verdict',
          fn: async () => {
            for (const delimiter of VERDICT_DELIMITERS)
              expect(
                opensWithVerdict({
                  answer: `N${delimiter}the source does quote this`,
                  letter: 'N',
                },),
              ).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES a letter that merely began a word, which is the whole '
            + 'reason the rule exists: "Not enough context to grade" is a refusal '
            + 'and would otherwise be counted as a false-positive verdict',
          fn: async () => {
            expect(
              opensWithVerdict({
                answer: 'Not enough context to grade',
                letter: 'N',
              },),
            ).toBe(false,);
            expect(
              opensWithVerdict({
                answer: 'Yes if you squint',
                letter: 'Y',
              },),
            ).toBe(false,);
          },
        },),

        it({
          name: 'refuses an answer that does not open with the letter at all, '
            + 'including one that merely contains it later',
          fn: async () => {
            expect(
              opensWithVerdict({
                answer: 'probably N',
                letter: 'N',
              },),
            ).toBe(false,);
            expect(
              opensWithVerdict({
                answer: '',
                letter: 'Y',
              },),
            ).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: trimLeadingDelimiters.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'drops the punctuation joining a verdict to its rationale, leaving '
            + 'the rationale a grader actually wrote',
          fn: async () => {
            expect(
              trimLeadingDelimiters({ text: ', the source does quote this', },),
            ).toBe('the source does quote this',);
            expect(
              trimLeadingDelimiters({ text: '.  but the warmer word is naps', },),
            ).toBe('but the warmer word is naps',);
          },
        },),

        it({
          name: 'returns empty for a bare verdict, so an answer with no rationale '
            + 'carries no invented one',
          fn: async () => {
            expect(trimLeadingDelimiters({ text: '', },),).toBe('',);
            expect(trimLeadingDelimiters({ text: '   ', },),).toBe('',);
            expect(trimLeadingDelimiters({ text: '.', },),).toBe('',);
          },
        },),

        it({
          name: 'keeps rationale text that itself starts with punctuation once a '
            + 'non-delimiter has been reached, since only the join is noise',
          fn: async () => {
            expect(
              trimLeadingDelimiters({ text: ': "naps" reads warmer', },),
            ).toBe('"naps" reads warmer',);
          },
        },),
      ],
    },),
  ],
},);
