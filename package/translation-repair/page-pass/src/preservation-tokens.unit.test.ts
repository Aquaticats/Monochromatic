/**
 Tests for the two index scans the preservation gate compares text with.

 BOTH ARE REACHED ONLY THROUGH `preservation-check.ts`, which asks them one
 question each about whole passages and then reports a verdict about an edit.
 That makes every rule here visible to the suite only as a gate outcome, and a
 gate has two outcomes while these have sixteen branches between them. What
 follows asks each rule directly.

 THE THREE RULES THAT COST SOMETHING WHEN THEY WERE WRONG, each named in the
 module's own comments and each pinned here. A colon is not a sentence end,
 and reading it as one made a deleted contributor name invisible. A
 sentence-initial capital is not a name, and reading it as one rejected edits
 for losing "Moreover". A two-letter capital is not a name, because initials
 and abbreviations vanish in any ordinary rewrite.

 WHY IDEOGRAPHS GET THEIR OWN CASES. `contentTokens` drops one-character
 tokens, since a lone ASCII letter carries nothing, and a one-character
 Chinese token is a whole word. The two rules meet in the same loop and the
 second one has to win.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  contentTokens,
  properNouns,
} from '../dist/final/node/index.mjs';

/**
 Renders a name set as a sorted array, so a case reads as what it claims.

 @param text - text to scan

 @returns Names found, in a stable order

 @example
 ```ts
 expect(namesIn({ text: 'The cat met Bilibi.', },),).toEqual(['bilibi',],);
 ```
 */
function namesIn(
  { text, }: { readonly text: string; },
): readonly string[] {
  return [...properNouns({ text, },),].toSorted();
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: contentTokens.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'lowercases every word run and drops the stop words, which is the '
            + 'control the rest of these cases depart from one rule at a time',
          fn: async () => {
            expect(contentTokens({ text: 'She kept the Klipper videos', },),)
              .toEqual([
                'kept',
                'klipper',
                'videos',
              ],);
          },
        },),

        it({
          name: 'DROPS a one-character Latin word, since a lone letter is an '
            + 'initial or a list marker and its loss says nothing about whether an '
            + 'edit deleted anything',
          fn: async () => {
            expect(contentTokens({ text: 'X ray', },),).toEqual(['ray',],);
          },
        },),

        it({
          name: 'KEEPS every ideograph as its own token, one character each, '
            + 'against the same length rule that just dropped a lone Latin letter: '
            + 'a one-character Chinese token is a whole word',
          fn: async () => {
            expect(contentTokens({ text: '猫在窗台', },),)
              .toEqual([
                '猫',
                '在',
                '窗',
                '台',
              ],);
          },
        },),

        it({
          name: 'KEEPS an ideograph beyond the first plane as one token, which a scan by UTF-16 unit read '
            + 'as two halves, neither an ideograph, and dropped (ledger B21)',
          fn: async () => {
            expect(contentTokens({ text: '\u{20BB7}\u{732B}\u{7761}', },),)
              .toEqual([
                '\u{20BB7}',
                '\u{732B}',
                '\u{7761}',
              ],);
          },
        },),

        it({
          name: 'ends a Latin run at an ideograph and resumes after it, so a '
            + 'mixed line yields both scripts in document order rather than one '
            + 'token spanning the change of script',
          fn: async () => {
            expect(contentTokens({ text: 'Klipper猫video', },),)
              .toEqual([
                'klipper',
                '猫',
                'video',
              ],);
          },
        },),

        it({
          name: 'holds an apostrophe and a hyphen inside a word, so a contraction '
            + 'and a compound each stay one token instead of splitting into parts '
            + 'that no longer match the text they came from',
          fn: async () => {
            expect(contentTokens({ text: "She can't stay well-known", },),)
              .toEqual([
                "can't",
                'stay',
                'well-known',
              ],);
          },
        },),

        it({
          name: 'keeps an accented word whole, accent composed or combining, where ASCII letters alone cut it (ledger B18)',
          fn: async () => {
            expect(contentTokens({ text: 'Émile napped at the Cafe\u{0301}', },),)
              .toEqual([
                'émile',
                'napped',
                'cafe\u{0301}',
              ],);
          },
        },),

        it({
          name: 'keeps a digit run, since a year or a count is content whose loss '
            + 'is exactly the kind of deletion this gate exists to notice',
          fn: async () => {
            expect(contentTokens({ text: '2026 cats', },),)
              .toEqual([
                '2026',
                'cats',
              ],);
          },
        },),

        it({
          name: 'emits a run that reaches the end of the text, which is the arm '
            + 'the scan carries an extra position for: without it every passage '
            + 'would silently lose its last word',
          fn: async () => {
            expect(contentTokens({ text: 'Klipper', },),).toEqual(['klipper',],);
          },
        },),

        it({
          name: 'returns nothing for empty text rather than a token standing for '
            + 'the absence',
          fn: async () => {
            expect(contentTokens({ text: '', },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: properNouns.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a capital inside a sentence as a name, and leaves the '
            + 'sentence-initial capital beside it alone',
          fn: async () => {
            expect(namesIn({ text: 'The cat met Bilibi today.', },),)
              .toEqual(['bilibi',],);
          },
        },),

        it({
          name: 'REFUSES a capital that opens a sentence, which is an ordinary '
            + 'word wearing a capital. An earlier draft read these as names and '
            + 'rejected edits for losing "Moreover"',
          fn: async () => {
            expect(namesIn({ text: 'Cats nap. Moreover they purr.', },),)
              .toEqual([],);
          },
        },),

        it({
          name: 'REFUSES a capital opening a sentence that ended in a full-width '
            + 'stop, since this corpus is Chinese and its sentences end in 。 as '
            + 'often as in a period',
          fn: async () => {
            expect(namesIn({ text: '猫猫睡觉。Moreover they purr.', },),)
              .toEqual([],);
          },
        },),

        it({
          name: 'ACCEPTS a name after a colon, which is the case the whole '
            + 'sentence-end list is written around: reading a colon as a '
            + 'terminator is what made a deleted contributor name invisible',
          fn: async () => {
            expect(namesIn({ text: 'Contributor: Bilibi kept the archive.', },),)
              .toEqual(['bilibi',],);
          },
        },),

        it({
          name: 'REFUSES a two-letter capital, since initials and abbreviations '
            + 'are what short capitalized runs almost always are and an ordinary '
            + 'rewrite explains their disappearance',
          fn: async () => {
            expect(namesIn({ text: 'The cat met Jo today.', },),).toEqual([],);
          },
        },),

        it({
          name: 'ACCEPTS a three-letter capital, which is where the length floor '
            + 'sits: a floor is a boundary rather than something to clear by a '
            + 'margin',
          fn: async () => {
            expect(namesIn({ text: 'The cat met Ann today.', },),)
              .toEqual(['ann',],);
          },
        },),

        it({
          name: 'REFUSES a capital in the middle of a word, so a camel-cased '
            + 'product name yields one candidate rather than one per hump',
          fn: async () => {
            expect(namesIn({ text: 'macBook naps', },),).toEqual([],);
          },
        },),

        it({
          name: 'walks back over an opening quotation mark to find what really '
            + 'precedes the word, and REFUSES the name it finds a sentence end '
            + 'behind',
          fn: async () => {
            expect(namesIn({ text: 'Cats nap. "Bilibi purrs."', },),).toEqual([],);
          },
        },),

        it({
          name: 'walks back over the same quotation mark and ACCEPTS the name '
            + 'where a comma turns up instead, which is the other half of that '
            + 'rule and the half that keeps quoted names visible',
          fn: async () => {
            expect(namesIn({ text: 'The cat, "Bilibi" purrs.', },),)
              .toEqual(['bilibi',],);
          },
        },),

        it({
          name: 'takes a hyphenated name whole and does not scan inside it again, '
            + 'so the capital after the hyphen yields no second candidate',
          fn: async () => {
            expect(namesIn({ text: 'The cat met Anne-Marie today.', },),)
              .toEqual(['anne-marie',],);
          },
        },),

        it({
          name: 'reports one name however often it appears, since this answers '
            + 'which names the text carries rather than how heavily it leans on '
            + 'them',
          fn: async () => {
            expect(namesIn({ text: 'A cat met Bilibi, and Bilibi purred.', },),)
              .toEqual(['bilibi',],);
          },
        },),

        it({
          name: 'returns nothing for empty text rather than a name standing for '
            + 'the absence',
          fn: async () => {
            expect(namesIn({ text: '', },),).toEqual([],);
          },
        },),

        it({
          name: 'finds a name opening on an accented capital or carrying an accent, which ASCII capitals '
            + 'alone missed, so the gate notices it lost (ledger B18)',
          fn: async () => {
            expect(namesIn({ text: 'A cat met Émile and Château Miaou.', },),)
              .toEqual([
                'château',
                'miaou',
                'émile',
              ],);
          },
        },),
      ],
    },),
  ],
},);
