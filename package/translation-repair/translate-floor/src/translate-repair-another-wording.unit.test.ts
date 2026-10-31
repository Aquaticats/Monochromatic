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

import { buildAnotherTranslatorRepairMessages, } from '../dist/final/node/index.mjs';

//region Another translator wording tests

/**
 Content of the turn that hands another translator the translation and the findings.

 @param findings - findings written for the author

 @returns The text of the last message

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

 @returns The text outside its quotations

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

 @returns The words, in order

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
         One finding of each kind, as the source of each writes it today.
         */
        const findings = [
          'This slice is LINE-STRUCTURED: every line stands as its own unit, so the rendering owes one line per line of the '
            + 'ORIGINAL and may never merge two into one. The rendering carries 1 line of content where the ORIGINAL has 2. Put '
            + 'back the line breaks that were merged, keeping the wording as it stands.',
          'The EXISTING TRANSLATION carries the block holding it as 1 line. The rendering carries 2. Drop the second rendering '
            + 'of the pair, keeping the wording used elsewhere.',
          'The link text for https://cat.example names 猫, whom this page\'s front matter declares "Mittens", but the link '
            + 'text "Cat" does not carry "Mittens". Keep the rest of the title as it was rendered.',
          'Your translation carries the sheet\'s own "ORIGINAL" block, which is evidence shown to the translator and never '
            + 'part of the passage.',
        ];
        const text = turnFor({ findings, },);
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
