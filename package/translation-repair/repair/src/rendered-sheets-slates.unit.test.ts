/**
 Tests for the slate sheets the repair fixtures render, as far as the sentence about a declined round goes.

 WHY THIS FILE EXISTS. The fixtures import the decline constant, so the text in a rendered sheet is whatever the constant
 holds, and nothing read the sentence as a judge meets it, between the line that says declining is a real answer and the
 line that asks for the JSON reply (re-check 3, N7). The case reads the sheet of the translate slate over wording that
 cannot ship and checks the clauses by their meaning (`decline-consequence-clauses.test-fixture.ts`), not by a copy of
 the constant's words (N1).

 THE SENTENCE IS TOLD IN THREE ROUNDS, and each has its own sheet here (re-check 5, the rendered-sheets gap): the first
 ask of a pair, the challenge of a caller that ships past a decline, and the challenge of a caller that defers. The
 sheet-wide scans read `slateSheets()`, so a sentence with no sheet here is a sentence they never read.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  type DeclineMode,
  unmetClausesOf,
  unmetDeclineClauses,
} from '@monochromatic-dev/translation-repair-adjudication/ts/decline-consequence-clauses.test-fixture.ts';
import { slateSheets, } from './rendered-sheets-slates.test-fixture.ts';

/**
 Name of the sheet a translate slate over withheld wording renders for the challenge that ships past a decline: the
 only fixture built with the decline consequence that ships or keeps.
 */
const WITHHELD_WORDING_SHEET = 'translate front matter slate';

/**
 Each round's sheet and the round whose clauses its sentence must carry.
 */
const ROUND_SHEETS: readonly {
  readonly name: string;
  readonly mode: DeclineMode;
}[] = [
  {
    name: 'translate first ask slate',
    mode: 'asked-again',
  },
  {
    name: WITHHELD_WORDING_SHEET,
    mode: 'ships',
  },
  {
    name: 'translate deferring challenge slate',
    mode: 'defers',
  },
];

/**
 Line that opens the sentence about declining in a rendered selection sheet.
 */
const SENTENCE_OPENS_AT = 'Declining is a real answer';

/**
 Line that follows the sentence about declining in a rendered selection sheet.
 */
const SENTENCE_CLOSES_AT = 'Reply with ONLY a JSON object';

/**
 The sentence about declining as a judge reads it on one rendered sheet.

 @param name - name of the rendered sheet

 @returns Text between the line that opens the sentence and the line that asks for the reply

 @throws {Error} when the fixtures render no sheet of that name, or the sheet lacks either bounding line

 @example
 ```ts
 const sentence = sentenceOn(WITHHELD_WORDING_SHEET,);
 ```
 */
function sentenceOn(name: string,): string {
  /**
   Rendered sheet of that name.
   */
  const sheet = slateSheets()
    .find(function isNamed(rendered,): boolean {
      return rendered.name === name;
    },);
  if (sheet === undefined)
    throw new Error(`the fixtures render no sheet named ${name}`,);

  const { text, } = sheet;

  /**
   Where the sentence opens in the sheet, -1 when the sheet has no such line.
   */
  const opens = text.indexOf(SENTENCE_OPENS_AT,);

  /**
   Where the line that asks for the reply stands, -1 when the sheet has no such line.
   */
  const closes = text.indexOf(SENTENCE_CLOSES_AT,);
  expect(opens,).toBeGreaterThanOrEqual(0,);
  expect(closes,).toBeGreaterThan(opens,);

  /**
   Text a judge reads between the two lines that bound the sentence.
   */
  const sentence = text.slice(
    opens,
    closes,
  );
  expect(sentence.length,).toBeGreaterThan(SENTENCE_OPENS_AT.length,);
  return sentence;
}

await describe({
  name: slateSheets.name,
  children: [
    it({
      name: 'SHOWS A JUDGE EVERY CLAUSE ABOUT DECLINING OVER WORDING THAT CANNOT SHIP, between the line that calls '
        + 'declining a real answer and the line that asks for the reply: declining does not by itself keep the '
        + 'wording, its own decline names no candidate, a tie backing only some candidates sends the panel to a '
        + 'run-off first, none named by two or more judges leaves the wording in place and reported, and one or more '
        + 'named by two or more judges ships the preferred of those by a fixed order',
      fn: async () => {
        /**
         What a judge of the challenge that ships past a decline reads.
         */
        const sentence = sentenceOn(WITHHELD_WORDING_SHEET,);
        expect(unmetDeclineClauses(sentence,),).toEqual([],);
      },
    },),

    it({
      name: 'SHOWS EACH ROUND OF A PAIR ITS OWN SENTENCE ON ITS OWN SHEET, so the sheet-wide scans read all three: the '
        + 'first ask settles nothing and asks the panel again, the challenge that ships past a decline ships or '
        + 'keeps, and the challenge that defers ships only a level tie, and none of the three carries the clauses '
        + 'of another',
      fn: async () => {
        expect(ROUND_SHEETS.map(function unmetOnItsOwnSheet({
          name,
          mode,
        },): readonly string[] {
          return unmetClausesOf({
            text: sentenceOn(name,),
            mode,
          },);
        },),).toEqual([
          [],
          [],
          [],
        ],);
        expect(ROUND_SHEETS.map(function carriedByTheOthers({ name, },): number {
          return ROUND_SHEETS
            .filter(function isAnotherRound(other,): boolean {
              return other.name !== name;
            },)
            .filter(function carriesTheOthersClauses({ mode, },): boolean {
              return unmetClausesOf({
                text: sentenceOn(name,),
                mode,
              },).length === 0;
            },).length;
        },),).toEqual([
          0,
          0,
          0,
        ],);
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
