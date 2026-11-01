/**
 Guards class eighty (shi_Yumiaoya, 2026-09-22): the ORIGINAL quotes a film
 line in Chinese with the film's own English directly beside it, and its
 attribution the same way. The line floor counts such a pair as one line owed
 (class forty-seven) and the archive carries the English alone, but every
 line-structure wording on the sheets said "never drop a line" and the judges'
 criterion called a dropped line FAULTY, so the writers rendered the Chinese
 line a second time in English beside the film's own words (shi_Yumiaoya10)
 or kept it in Chinese on an English page (shi_Yumiaoya4 to 9), and the
 judges split between "omits the first quoted line" and "keeps the original
 English line after adding a new translation". The three wordings now say
 that such a pair is one line whose English is already its rendering.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { buildEditorAddendum, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  TRANSLATE_LINE_STRUCTURE_CRITERION,
  TRANSLATE_LINE_STRUCTURE_RULE,
} from '../dist/final/node/index.mjs';

/**
 Clause every line-structure wording must carry, read off the rule verbatim
 so a rewording that drops the pair case fails here.
 */
const BILINGUAL_PAIR =
  'A line the ORIGINAL gives twice, once in Chinese and once in English directly beside it '
    + '(a quotation in both languages, and its attribution the same way), is ONE line whose English is already its rendering: '
    + 'carry that English line once, as the ORIGINAL has it; a rendering carrying the English line alone for such a pair has dropped nothing, '
    + 'and one carrying the Chinese line, or a second English wording of it, beside the English has invented a line.';

await describe({
  name: 'a Chinese line with its own English beside it is one line on every sheet (class eighty, shi_Yumiaoya)',
  children: [
    it({
      name: 'the translators\' line-structure rule carries the clause, so the writers stop rendering the Chinese half again',
      fn: async () => {
        expect(TRANSLATE_LINE_STRUCTURE_RULE,).toContain(BILINGUAL_PAIR,);
      },
    },),
    it({
      name: 'the judges\' line-structure criterion carries the clause, so a candidate carrying the English alone is not "missing a line"',
      fn: async () => {
        expect(TRANSLATE_LINE_STRUCTURE_CRITERION,).toContain(BILINGUAL_PAIR,);
      },
    },),
    it({
      name: 'the editors\' line-structure addendum carries the clause',
      fn: async () => {
        const addendum = buildEditorAddendum({ baseAddendum: 'The cat slept on the windowsill.', lineStructured: true, },);
        expect(addendum,).toContain(BILINGUAL_PAIR,);
      },
    },),
    it({
      name: 'an ungoverned addendum carries no line rule at all',
      fn: async () => {
        const addendum = buildEditorAddendum({ baseAddendum: 'The cat slept on the windowsill.', lineStructured: false, },);
        expect(addendum.includes(BILINGUAL_PAIR,),).toBe(false,);
      },
    },),
  ],
},);
