/**
 Guards the whole-package audit of 2026-09-27 on the house policy's own
 wording, each sentence a place where the rule said something other than
 what the corpus and the owner's rulings ask:

 - no producing sheet said the house rules outrank the sheet's own rules, so
   the refiner's "keep every word left in the original language" sat beside
   the rule rendering Ta, titles and handles, unresolved;
 - the reader-protection sample "the page says that she ended her life" stood
   in a bullet opening on "a death or an attempt", the class seventy-nine
   shape class one hundred twenty-four's later sentence contradicts;
 - "Entries are written in the third person" against contributors writing
   我, whom the kept-subject rule renders as I;
 - "MtF is trans woman, as 药娘 is" narrower than the owner's "trans girl or
   trans woman";
 - "kept and glossed" never said the kept term is not left in Han, and runs
   shipped "大证 (...)" in English prose;
 - the positional-word rule read as a ban on "earlier" and "below" in any
   sense;
 - the corner-bracket rule's second example kept quotation marks the rule
   said are dropped;
 - the measuring sheets' reader-protection line named deaths only;
 - the rendering audit called a tense English supplies an altered time.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { HOUSE_POLICY_BLOCK, } from '@monochromatic-dev/translation-repair-house-rule';
import { renderedSheets, } from './rendered-sheets.test-fixture.ts';

/**
 Rendered text of one named sheet.

 @param name - stage the sheet is written for

 @returns Sheet text, empty where no sheet has the name

 @example
 ```ts
 sheetNamed({ name: 'resolution', },);
 ```
 */
function sheetNamed({ name, }: { readonly name: string; },): string {
  return renderedSheets().find(function named(sheet,): boolean {
    return sheet.name === name;
  },)?.text ?? '';
}

/**
 The sheets that measure a text against the original, which carry the measuring block of the house policy.
 */
const MEASURING_SHEETS: readonly string[] = [
  'naturalness review',
  'rendering audit',
  'resolution',
  'restoration judge',
  'introduced defect',
];

/**
 The words that open the paragraph of the measuring block that holds the reader-protection sentence.
 */
const MEASURING_PARAGRAPH = 'SUPPORTED BY THE ORIGINAL IS NOT ENOUGH TO MAKE SOMETHING MISSING A DEFECT';

/**
 The reader-protection sentence every measuring sheet carries.
 */
const MEASURING_SENTENCE = 'about how a death or an attempt happened, where it happened when the place was the means, '
  + 'or about a medication';

await describe({
  name: 'the house policy says what the corpus and the owner ask (audit of 2026-09-27)',
  children: [
    it({
      name: 'SAYS THE HOUSE RULES OUTRANK any other rule on the same sheet',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain(
          'where another rule on the same sheet disagrees with one of them, the house rule wins',
        );
      },
    },),
    it({
      name: 'NEVER TELLS the refiner to keep a word left in the original language (ledger S10)',
      fn: async () => {
        // Precedence alone left the sentence standing as an instruction the
        // house rules then overrule; the sheet now says the same thing they do.
        expect(sheetNamed({ name: 'refiner', },),).not.toContain('any word left in the original language',);
        expect(sheetNamed({ name: 'refiner', },),).toContain('in the form the house rules give it',);
      },
    },),
    it({
      name: 'TELLS the comparative refiner a house correction is a rewrite to make, since the polish gate now '
        + 'prefers one (ledger S11)',
      fn: async () => {
        // The sheet asked only whether a paragraph read awkwardly, so a life
        // told in the present tense in fluent English was left alone.
        expect(sheetNamed({ name: 'refiner', },),).toContain('bringing it into line is a clear improvement',);
      },
    },),
    it({
      name: 'NEVER PRESCRIBES "ended her life" for an attempt she survived',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).not.toContain('the page says that she ended her life, and keeps',);
        expect(HOUSE_POLICY_BLOCK,).toContain(
          'the page says that she ended her life, or, where she survived, that she attempted suicide',
        );
      },
    },),
    it({
      name: 'KEEPS A CONTRIBUTOR\'S FIRST PERSON while the entry narrates the person in the third',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).not.toContain('- Entries are written in the third person.',);
        expect(HOUSE_POLICY_BLOCK,).toContain('written in the first person (我) stay in the first person',);
      },
    },),
    it({
      name: 'RENDERS MtF AND 药娘 as the owner ruled: trans woman or trans girl',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain('MtF is trans woman or trans girl, as 药娘 is',);
      },
    },),
    it({
      name: 'KEEPS A TERM WITH NO ENGLISH EQUIVALENT in English letters, never in Han',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain(
          'is kept in English letters (translated literally, or romanized where it is a name), never left in Han',
        );
      },
    },),
    it({
      name: 'BANS POSITIONAL REFERENCES, not the words "earlier" or "below" about time, height or rank',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).not.toContain(
          'Do not write "below", "above", "the following" or "earlier" in English prose.',
        );
        expect(HOUSE_POLICY_BLOCK,).toContain('about time, height or rank',);
      },
    },),
    it({
      name: 'DROPS CORNER BRACKETS around a name used to refer, and quotes one being introduced',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain('around a name or a term used to refer to its thing they are dropped',);
        expect(HOUSE_POLICY_BLOCK,).toContain('around a name or a word being introduced or talked about',);
      },
    },),
    it({
      name: 'TELLS EVERY MEASURING SHEET reader protection covers attempts and a place that was the means, and no '
        + 'other sheet carries the paragraph (re-check 7, nit 4)',
      fn: async () => {
        // The five sheets splice MEASUREMENT_POLICY_BLOCK each in its own file, so
        // a fault planted at one splice leaves the others whole; the case reads all.
        /**
         Names of the sheets that carry the paragraph of the measuring block, read from the rendered sheets.
         */
        const carrying = renderedSheets()
          .filter(function carriesParagraph({ text, },): boolean {
            return text.includes(MEASURING_PARAGRAPH,);
          },)
          .map(function nameOf({ name, },): string {
            return name;
          },);
        expect({
          unlisted: carrying.filter(function isUnlisted(name,): boolean {
            return !MEASURING_SHEETS.includes(name,);
          },),
          missing: MEASURING_SHEETS.filter(function isMissing(name,): boolean {
            return !carrying.includes(name,);
          },),
          lacking: MEASURING_SHEETS.filter(function lacksSentence(name,): boolean {
            return !sheetNamed({ name, },).includes(MEASURING_SENTENCE,);
          },),
        },).toEqual({
          unlisted: [],
          missing: [],
          lacking: [],
        },);
      },
    },),
    it({
      name: 'NEVER CALLS A TENSE English supplies an altered time on the rendering audit',
      fn: async () => {
        expect(sheetNamed({ name: 'rendering audit', },),).not.toContain('tense, date, duration',);
        expect(sheetNamed({ name: 'rendering audit', },),).toContain(
          'a tense English supplies for tenseless Chinese is never this defect',
        );
      },
    },),
  ],
},);
