/**
 Guards the whole-package audit of 2026-09-27: the house policy asks for
 Canadian English and itself wrote "capitalised", "judgement" and
 "penalises", the apparatus clause and the contest policy wrote
 "characterisation", the consolidation writer "characterise", and the critic
 and editor "humor". The old spelling test read four words of one block;
 this one reads every rendered sheet (`rendered-sheets.test-fixture.ts`).

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isAsciiLetter, } from '@monochromatic-dev/translation-repair-markup';
import { renderedSheets, } from './rendered-sheets.test-fixture.ts';

/**
 Spellings Canadian English writes otherwise, British -ise forms and
 American forms alike, each as a whole lower-case word.
 */
const NON_CANADIAN_FORMS: ReadonlySet<string> = new Set([
  'analyse',
  'analysed',
  'apologise',
  'behavior',
  'canceled',
  'capitalise',
  'capitalised',
  'capitalising',
  'catalog',
  'center',
  'characterisation',
  'characterise',
  'characterised',
  'color',
  'counselor',
  'criticise',
  'defense',
  'emphasise',
  'favor',
  'favorite',
  'flavor',
  'gray',
  'honor',
  'humor',
  'judgement',
  'localisation',
  'minimise',
  'neighbor',
  'neutralise',
  'normalise',
  'normalised',
  'normalises',
  'offense',
  'organise',
  'organised',
  'penalise',
  'penalised',
  'penalises',
  'prioritise',
  'programme',
  'programmes',
  'realise',
  'realised',
  'recognise',
  'recognised',
  'romanise',
  'romanised',
  'summarise',
  'summarised',
  'traveled',
  'uncharacterised',
],);

/**
 Every word of a text, lower-cased, in order.

 A LINEAR PASS that closes a word at each character that is no letter.

 @param text - sheet to read

 @returns Words in order

 @example
 ```ts
 wordsOf({ text: 'Read with judgement.', },); // ['read', 'with', 'judgement']
 ```
 */
function wordsOf({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Words closed so far and the one being read, as one list whose last item
   may be empty. Code points on purpose, not graphemes: only an ASCII letter
   continues a word, and any other code point closes it whatever grapheme it
   belongs to.
   */
  const words = Array.from(text,).reduce<string[]>(
    function readCharacter(
      read,
      character,
    ): string[] {
      if (isAsciiLetter({ character, },)) {
        read[read.length - 1] = `${read.at(-1,) ?? ''}${character.toLowerCase()}`;
        return read;
      }
      if (read.at(-1,) !== '')
        read.push('',);
      return read;
    },
    ['',],
  );
  return words.filter(function nonEmpty(word,): boolean {
    return word !== '';
  },);
}

await describe({
  name: 'every model-facing sheet is written in Canadian English',
  children: [
    it({
      name: 'WRITES NO British -ise form and no American spelling on any rendered sheet',
      fn: async () => {
        /**
         Every sheet and word pair that departs from Canadian spelling.
         */
        const departures = renderedSheets().flatMap(function departuresOf(sheet,): readonly string[] {
          return wordsOf({ text: sheet.text, },)
            .filter(function isNonCanadian(word,): boolean {
              return NON_CANADIAN_FORMS.has(word,);
            },)
            .map(function labelled(word,): string {
              return `${sheet.name}: ${word}`;
            },);
        },);
        expect([...new Set(departures,),],).toEqual([],);
      },
    },),
    it({
      name: 'READS every sheet it claims to, so an empty scan means something',
      fn: async () => {
        expect(renderedSheets().filter(function tooShort(sheet,): boolean {
          return wordsOf({ text: sheet.text, },).length < 2;
        },),).toEqual([],);
        expect(wordsOf({ text: 'Read it with judgement.', },).some(function isNonCanadian(word,): boolean {
          return NON_CANADIAN_FORMS.has(word,);
        },),).toBe(true,);
      },
    },),
  ],
},);
