/**
 Tests for the slate sheets the repair fixtures render, as far as the sentence about a declined round goes.

 WHY THIS FILE EXISTS. The fixtures import the decline constant, so the text in a rendered sheet is whatever the constant
 holds, and nothing read the sentence as a judge meets it, between the line that says declining is a real answer and the
 line that asks for the JSON reply (re-check 3, N7). The case reads the sheet of the translate slate over wording that
 cannot ship and checks the clauses by their meaning (`decline-consequence-clauses.test-fixture.ts`), not by a copy of
 the constant's words (N1).

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { unmetDeclineClauses, } from '@monochromatic-dev/translation-repair-adjudication/ts/decline-consequence-clauses.test-fixture.ts';
import { slateSheets, } from './rendered-sheets-slates.test-fixture.ts';

/**
 Name of the sheet a translate slate over withheld wording renders: the only fixture built with the decline
 consequence for wording that cannot ship.
 */
const WITHHELD_WORDING_SHEET = 'translate front matter slate';

/**
 Line that opens the sentence about declining in a rendered selection sheet.
 */
const SENTENCE_OPENS_AT = 'Declining is a real answer';

/**
 Line that follows the sentence about declining in a rendered selection sheet.
 */
const SENTENCE_CLOSES_AT = 'Reply with ONLY a JSON object';

await describe({
  name: slateSheets.name,
  children: [
    it({
      name: 'SHOWS A JUDGE EVERY CLAUSE ABOUT DECLINING OVER WORDING THAT CANNOT SHIP, between the line that calls '
        + 'declining a real answer and the line that asks for the reply: declining does not by itself keep the '
        + 'wording, its own decline names no candidate, none named by two or more judges leaves the wording in place '
        + 'and reported, and one or more named by two or more judges ships the preferred of those by a fixed order',
      fn: async () => {
        /**
         Rendered sheet of the translate slate over withheld wording.
         */
        const sheet = slateSheets()
          .find(function isWithheldWording({ name, },): boolean {
            return name === WITHHELD_WORDING_SHEET;
          },);
        if (sheet === undefined)
          throw new Error(`the fixtures render no sheet named ${WITHHELD_WORDING_SHEET}`,);

        /**
         Text a judge reads between the two lines that bound the sentence.
         */
        const sentence = sheet.text
          .slice(
            sheet.text
              .indexOf(SENTENCE_OPENS_AT,),
            sheet.text
              .indexOf(SENTENCE_CLOSES_AT,),
          );
        expect(sentence.length,).toBeGreaterThan(SENTENCE_OPENS_AT.length,);
        expect(unmetDeclineClauses(sentence,),).toEqual([],);
      },
    },),

    it({
      name: 'SHOWS THE CLAUSES ON NO OTHER SLATE SHEET, which is the control: the sheets whose caller keeps trusted text '
        + 'or has nothing to keep tell a judge something else, so the reader finds the clauses only where they belong',
      fn: async () => {
        expect(slateSheets()
          .filter(function carriesEveryClause({ text, },): boolean {
            return unmetDeclineClauses(text,).length === 0;
          },)
          .map(function toName({ name, },): string {
            return name;
          },),).toEqual([WITHHELD_WORDING_SHEET,],);
      },
    },),
  ],
},);
