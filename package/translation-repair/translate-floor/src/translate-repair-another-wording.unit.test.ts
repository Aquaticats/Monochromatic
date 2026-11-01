/**
 Tests for the words the follow-up turn for another translator carries (ledger B1145, the owner's "Other model, own name"
 of 2026-10-31).

 NO CLAIM THAT THE TEXT IS THE ASKED MODEL'S REACHES IT, and no word of the text under revision is rewritten. A finding is
 written to the author ("Your translation ..."), and the other translator reads it with the article in place of the
 possessive; a quotation inside a finding is the model's own text and stays exactly as it was. The five findings that
 spoke to the author in other words (the lines it carries, the wording it chose, the breaks it merged, the way it rendered
 a title, the evidence shown to it) are written without a second person, so nothing but the article is swapped.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildAnotherTranslatorRepairMessages,
  validateTranslatedSlice,
} from '../dist/final/node/index.mjs';

//region Another translator wording tests

/**
 Content of the turn that hands another translator the translation and the findings.

 @param findings - findings written for the author

 @returns Text of the last message

 @example
 ```ts
 const text = turnFor({ findings: ['Your translation drops footnote 1.',], },);
 ```
 */
function turnFor({ findings, }: { readonly findings: readonly string[]; },): string {
  const last = buildAnotherTranslatorRepairMessages({
    priorMessages: [{ role: 'user', content: 'the sheet', },],
    priorTranslation: 'The cat naps.',
    findings,
  },).at(-1,);
  return ((typeof last?.content) === 'string') ? last.content : '';
}

/**
 Cuts the quotations (double quotes and backticks) out of a text.

 @param text - text to read

 @returns Text outside its quotations

 @example
 ```ts
 // yields 'a  b'
 const outside = outsideQuotations({ text: 'a "cat" b', },);
 ```
 */
function outsideQuotations({ text, }: { readonly text: string; },): string {
  return text
    .split('"',)
    .filter(function isOutsideDouble(
      _piece,
      at,
    ): boolean {
      return (at % 2) === 0;
    },)
    .join(' ',)
    .split('`',)
    .filter(function isOutsideTicks(
      _piece,
      at,
    ): boolean {
      return (at % 2) === 0;
    },)
    .join(' ',);
}

/**
 Cuts a text into its lower-case words of ASCII letters.

 @param text - text to read

 @returns Words, in order

 @example
 ```ts
 // yields ['your', 'cat']
 const words = wordsOf({ text: 'Your cat.', },);
 ```
 */
function wordsOf({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Words read so far.
   */
  const words: string[] = [];
  /**
   Letters of the word being read.
   */
  const letters: string[] = [];
  for (const character of `${text.toLowerCase()} `) {
    if ((character >= 'a') && (character <= 'z')) {
      letters.push(character,);
      continue;
    }
    if (letters.length > 0)
      words.push(letters.join('',),);
    letters.length = 0;
  }
  return words;
}

await describe({
  name: buildAnotherTranslatorRepairMessages.name,
  concurrency: 1,
  children: [
    it({
      name: 'SWAPS THE POSSESSIVE FOR THE ARTICLE OUTSIDE QUOTATIONS ONLY: a word of the model\'s own text inside double '
        + 'quotes or backticks is never rewritten',
      fn: async () => {
        const text = turnFor({
          findings: [
            'Your translation carries the pronoun untranslated as "Your Ta" and your `your cat` stays: Keep it.',
          ],
        },);
        expect(text,).toContain(
          '- The translation carries the pronoun untranslated as "Your Ta" and the `your cat` stays: Keep it.',
        );
        // A mark with no closing mark opens no quotation, so the words after it are swapped like any other.
        expect(turnFor({ findings: ['Your translation keeps a lone " mark and your `cat` too.',], },),).toContain(
          '- The translation keeps a lone " mark and the `cat` too.',
        );
      },
    },),

    it({
      name: 'LEAVES NO SECOND PERSON IN A FINDING OF EACH OF THE FIVE KINDS THAT SPOKE TO THE AUTHOR, so the only change '
        + 'between the two turns is the article',
      fn: async () => {
        /**
         The findings the publication rule writes for a candidate of each kind, read from the rule itself and not typed here
         (ledger B1166): the merged lines, the repeated line, the pair beyond its bound, the link text without its declared
         name and the sheet's block copied in.
         */
        const found = [
          {
            says: 'every line stands as its own unit',
            sourceText: '猫醒了。\n太阳很暖。',
            candidateText: 'The cat wakes. The sun is warm.',
            lineStructured: true,
          },
          {
            says: 'Drop the repeat',
            sourceText: '猫在睡觉。',
            candidateText: 'The cat naps.\nThe cat naps.',
            lineStructured: true,
          },
          {
            says: 'Drop the second ',
            sourceText: '> 愿每只猫都能找到属于它的阳光。\n> May every cat find a sunbeam.',
            candidateText: '> May every cat find its own sunshine.\n> May every cat find a sunbeam.',
            pageText: '> May every cat find a sunbeam.',
            lineStructured: true,
          },
          {
            says: 'A declared name inside a title takes its declared form',
            sourceText: '[猫](https://cat.example)',
            candidateText: '[Cat](https://cat.example)',
            declared: [{ source: '猫', rendering: 'Mittens', },],
            lineStructured: false,
          },
          {
            says: 'which is evidence shown to',
            sourceText: '猫在睡觉。',
            pageText: 'The cat is sleeping.',
            candidateText: 'The cat is sleeping.\n\n===== ATTESTED DETAILS =====\n- the cat is asleep',
            lineStructured: false,
          },
        ].flatMap(function findingsOf({ says, ...candidate },): readonly string[] {
          /**
           What the rule made of this candidate.
           */
          const verdict = validateTranslatedSlice(candidate,);
          /**
           Findings that carry the words this kind is known by, which show its producer ran and not another.
           */
          const own = (verdict.kind === 'invalid')
            ? verdict.findings.filter(function isOwn(finding,): boolean {
              return finding.includes(says,);
            },)
            : [];
          // A KIND THAT PRODUCED NOTHING WOULD LEAVE THE CASE TRUE OF AN EMPTY LIST, so each must name itself once.
          expect(own.length,).toBe(1,);
          return own;
        },);
        expect(found.length,).toBe(5,);
        const text = turnFor({ findings: found, },);
        /**
         The findings as the turn lists them, below the heading.
         */
        const listed = text.slice(text.indexOf('FINDINGS',),);
        expect(wordsOf({ text: outsideQuotations({ text: listed, },), },).filter(function isSecondPerson(word,): boolean {
          return (word === 'you') || (word === 'your') || (word === 'yours');
        },),).toEqual([],);
      },
    },),

    it({
      name: 'CALLS THE TEXT UNDER REVISION BY ONE NAME AND NEVER CLAIMS THE ORIGINAL WAS RENDERED BY ANOTHER TRANSLATOR, '
        + 'since the sheet calls the passage the ORIGINAL and the translation is what another translator wrote',
      fn: async () => {
        const text = turnFor({ findings: ['Your translation drops footnote 1.',], },);
        expect(text,).not.toContain('The ORIGINAL passage was rendered by another translator',);
        expect(text,).toContain('translation to revise was written by another translator',);
      },
    },),
  ],
},);

//endregion Another translator wording tests
