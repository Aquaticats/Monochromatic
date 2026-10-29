/**
 Tests the composite candidate the archive block review offers beside the
 reviewers' whole revisions (ledger B712): when two or more revisions change
 pairwise disjoint stretches of the block's lines, the hunks they made are
 applied to the block together and offered as one more candidate, after the
 revisions and before the block as the archive wrote it. A revision may add or
 remove lines (ledger B803, which replaced the refusal of a revision whose line
 count differs from the block's).

 Every case drives the real stage over a scripted bench, one reply per seat,
 and reads the slate off the stage's own log line and the winner off the
 outcome. Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  archiveBlockSelectionEvidence,
  CompositeNotOnSlateError,
  runArchiveBlockReviewStage,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import { scriptedClient, } from './scripted-review-client.test-fixture.ts';

/** Four seats, so two revisions leave two judges with no stake in the composite. */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/** The one answer that retains the block, which no block of these cases spells. */
const RETAIN = 'retain';

/** Original section, whose words a retention quotes. */
const SOURCE = '猫在窗边安静地睡觉。';

/** Four-line block under review. */
const BLOCK = [
  'The cat sleeps by the window.',
  'It purrs at the birds.',
  'It won an award last spring.',
  'The dog barks at the mailman.',
].join('\n',);

/** Lines of the block, for building revisions of it. */
const [LINE_1, LINE_2, LINE_3, LINE_4,] = BLOCK.split('\n',) as [string, string, string, string];

/** Revision of the first line only. */
const FIRST_LINE = [
  'The cat naps by the window.',
  LINE_2,
  LINE_3,
  LINE_4,
].join('\n',);

/** Revision of the third line only. */
const THIRD_LINE = [
  LINE_1,
  LINE_2,
  'It won a ribbon last spring.',
  LINE_4,
].join('\n',);

/** The first two revisions' changed lines together. */
const FIRST_AND_THIRD = [
  'The cat naps by the window.',
  LINE_2,
  'It won a ribbon last spring.',
  LINE_4,
].join('\n',);

/** Revision of the first line again, in other words, so it overlaps the first revision. */
const FIRST_LINE_AGAIN = [
  'The cat dozes by the window.',
  LINE_2,
  LINE_3,
  LINE_4,
].join('\n',);

/** What a case read off the stage. */
type Review = {
  /**
   Stage outcome.
   */
  readonly outcome: Awaited<ReturnType<typeof runArchiveBlockReviewStage>>;
  /**
   Every line the stage logged.
   */
  readonly lines: readonly string[];
  /**
   Every request the bench was sent, serialized.
   */
  readonly prompts: readonly string[];
};

/**
 Runs the stage over a block, one answer per seat.

 @param answers - what each seat of the roster answers, in roster order

 @param best - candidate number every judge names

 @param blockText - block under review

 @param targetText - page the block stands at the start of; the block alone by default

 @returns Outcome and log lines

 @example
 ```ts
 const { outcome, lines, } = await review({ answers: [FIRST_LINE, THIRD_LINE, RETAIN, RETAIN,], best: 3, });
 ```
 */
async function review(
  {
    answers,
    best,
    blockText = BLOCK,
    targetText = blockText,
  }: {
    readonly answers: readonly string[];
    readonly best: number;
    readonly blockText?: string;
    readonly targetText?: string;
  },
): Promise<Review> {
  /**
   Lines the stage logged.
   */
  const lines: string[] = [];
  /**
   Requests the bench was sent.
   */
  const prompts: string[] = [];
  const outcome = await runArchiveBlockReviewStage({
    client: scriptedClient({
      prompts,
      replyFor: function replyFor({ schema, modelId, },): unknown {
        if (schema !== 'archive_block_review') {
          return {
            best,
            assessment: 'The candidate carries the fixes the others lack.',
          };
        }
        /**
         This seat's answer.
         */
        const answer = answers[ROSTER.indexOf(modelId as typeof ROSTER[number],)] ?? RETAIN;
        if (answer === RETAIN) {
          return {
            disposition: 'source-supported',
            sourceQuote: '窗边安静地睡觉',
            replacementText: '',
            assessment: 'The original supports this block.',
          };
        }
        return {
          disposition: 'revise',
          sourceQuote: '',
          replacementText: answer,
          assessment: `Some wording is unsupported: ${JSON.stringify(answer,)} is the fix.`,
        };
      },
    },),
    modelIds: ROSTER,
    sourceText: SOURCE,
    targetText,
    blockText,
    blockOffset: targetText.indexOf(blockText,),
    priorFindings: [],
    signal: new AbortController().signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    l: capturingLogger({ messages: lines, },),
    fanOut: 'whole-bench',
    // PINNED TO THE FIRST SEAT, so the candidates are numbered in roster order whatever the sheet says:
    // the bench's rotation is seeded by the sheet text, and an edit to the reply instruction moved it.
    benchStart: 0,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    outcome,
    lines,
    prompts,
  };
}

/**
 Whether the stage logged a line holding every part.

 @param lines - lines the stage logged

 @param parts - texts one line must hold together

 @returns Whether a line holds them all
 */
function logged(
  {
    lines,
    parts,
  }: {
    readonly lines: readonly string[];
    readonly parts: readonly string[];
  },
): boolean {
  return lines.some(function holdsAll(line,): boolean {
    return parts.every(function holds(part,): boolean {
      return line.includes(part,);
    },);
  },);
}

/**
 The candidates count the stage logged for the slate.

 @param lines - lines the stage logged

 @returns Line saying how many candidates the judges compared
 */
function slateLine({ lines, }: { readonly lines: readonly string[]; },): string {
  return lines.find(function isSlate(line,): boolean {
    return line.includes('archive review: comparing',);
  },) ?? '';
}

/**
 One assessment the select sheet carries, as the sheet words it.
 */
type SheetAssessment = {
  readonly disposition: string;
  readonly proposedAction: string;
  readonly candidate: number;
  readonly finding: string;
  readonly coversLines?: readonly number[];
  readonly fromCandidate?: number;
};

/**
 What the select sheet the judges were sent shows of the slate.
 */
type SelectSheet = {
  /**
   Candidate texts by the number the sheet gives them.
   */
  readonly candidates: ReadonlyMap<number, string>;
  /**
   Prior review assessments in the order the sheet lists them.
   */
  readonly assessments: readonly SheetAssessment[];
};

/**
 Reads the select sheet off the requests a review sent.

 @param prompts - serialized requests of the bench

 @returns First select sheet: its candidates and its prior review assessments

 @example
 ```ts
 const sheet = selectSheetOf({ prompts, },);
 ```
 */
function selectSheetOf({ prompts, }: { readonly prompts: readonly string[]; },): SelectSheet {
  /**
   The request that carries the assessments, which only a select round sends.
   */
  const request = prompts.find(function isSelect(prompt,): boolean {
    return prompt.includes('PRIOR REVIEW ASSESSMENTS',);
  },) ?? '';
  /**
   Text of the sheet's user message.
   */
  const sheet = (JSON.parse(request,) as readonly { readonly role: string; readonly content: string; }[])
    .filter(function isUser({ role, },): boolean {
      return role === 'user';
    },)
    .map(function contentOf({ content, },): string {
      return content;
    },)
    .join('\n',);
  /**
   The sheet's lines, the assessments' block opening two lines after its label.
   */
  const sheetLines = sheet.split('\n',);
  /**
   Position of the label that opens the assessments' block.
   */
  const labelAt = sheetLines.findIndex(function isLabel(line,): boolean {
    return line.startsWith('PRIOR REVIEW ASSESSMENTS',);
  },);
  return {
    candidates: new Map(sheet.split('\nCANDIDATE ',)
      .slice(1,)
      .map(function toEntry(part,): readonly [number, string] {
        /**
         The part from the rule that opens the candidate's text.
         */
        const fromText = part.slice(part.indexOf('\n=====\n',) + '\n=====\n'.length,);
        return [Number(part.split('\n',)[0],), fromText.slice(0, fromText.lastIndexOf('\n=====',),),] as const;
      },),),
    assessments: JSON.parse(sheetLines[labelAt + 2] ?? '[]',) as readonly SheetAssessment[],
  };
}

/**
 Number the sheet gives the candidate with a text.

 @param sheet - what the judges were sent

 @param text - candidate text

 @returns Its number, or zero where the sheet holds no such candidate
 */
function numberOf({ sheet, text, }: { readonly sheet: SelectSheet; readonly text: string; },): number {
  return [...sheet.candidates.entries(),].find(function holdsText([, candidate,],): boolean {
    return candidate === text;
  },)?.[0] ?? 0;
}

await describe({
  name: 'archive composite revision',
  children: [
    it({
      name: 'OFFERS THE LINES OF TWO REVISIONS THAT CHANGED DISJOINT LINES AS ONE CANDIDATE (ledger B712): the '
        + 'composite is candidate 3, after the two revisions and before the block as the archive wrote it, and '
        + 'wins when the judges name it',
      fn: async () => {
        const { outcome, lines, } = await review({
          answers: [FIRST_LINE, THIRD_LINE, RETAIN, RETAIN,],
          best: 3,
        },);
        expect(slateLine({ lines, },),).toContain('comparing 2 admissible revisions in 4 candidates',);
        expect(outcome.kind,).toBe('revised',);
        expect(outcome.text,).toBe(FIRST_AND_THIRD,);
        // Its contributors, in the order the revisions stand, own the stake that discounts their ballots.
        expect(logged({
          lines,
          parts: [`candidate 3 from composite(${ROSTER[0]} + ${ROSTER[1]}) won`,],
        },),).toBe(true,);
        expect(logged({
          lines,
          parts: [
            'archive composite built',
            '2 revisions',
            'disjoint',
          ],
        },),).toBe(true,);
        expect(outcome.findings.some(function recordsComposite(finding,): boolean {
          return finding.startsWith('archive-composite-offered',) && finding.includes(ROSTER[0],);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'CARRIES THE CONTRIBUTORS\' ASSESSMENTS ON THE COMPOSITE, each marked with the lines its reviewer changed: '
        + 'no reviewer proposed the composite, so matching assessments by proposed text left the one candidate '
        + 'the judges could not check against anyone\'s reasons',
      fn: async () => {
        const { prompts, } = await review({
          answers: [FIRST_LINE, THIRD_LINE, RETAIN, RETAIN,],
          best: 3,
        },);
        const sheet = selectSheetOf({ prompts, },);
        const composite = numberOf({ sheet, text: FIRST_AND_THIRD, },);
        const first = numberOf({ sheet, text: FIRST_LINE, },);
        const third = numberOf({ sheet, text: THIRD_LINE, },);
        expect(composite,).toBeGreaterThan(0,);
        /** The contributors in the order the slate numbers them, with the lines each changed. */
        const contributors = [
          { number: first, text: FIRST_LINE, lines: [1,], },
          { number: third, text: THIRD_LINE, lines: [3,], },
        ].toSorted(function bySlate(left, right,): number {
          return left.number - right.number;
        },);
        expect(sheet.assessments.filter(function onComposite({ candidate, },): boolean {
          return candidate === composite;
        },),).toEqual(contributors.map(function asAssessment({ number, text, lines, },): SheetAssessment {
          return {
            disposition: 'revise',
            proposedAction: 'revise',
            candidate: composite,
            finding: `Some wording is unsupported: ${JSON.stringify(text,)} is the fix.`,
            coversLines: lines,
            fromCandidate: number,
          };
        },),);
        // The contributors' own assessments stay where they were, whole-candidate and unmarked.
        for (const { number, text, } of contributors) {
          expect(sheet.assessments.filter(function onRevision({ candidate, },): boolean {
            return candidate === number;
          },),).toEqual([{
            disposition: 'revise',
            proposedAction: 'revise',
            candidate: number,
            finding: `Some wording is unsupported: ${JSON.stringify(text,)} is the fix.`,
          },],);
        }
      },
    },),
    it({
      name: 'CARRIES EVERY ASSESSMENT OF A REVISION SEVERAL REVIEWERS WROTE IDENTICALLY, since each of them backs '
        + 'the lines that revision changed',
      fn: async () => {
        const { prompts, } = await review({
          answers: [FIRST_LINE, FIRST_LINE, THIRD_LINE, RETAIN,],
          best: 3,
        },);
        const sheet = selectSheetOf({ prompts, },);
        const composite = numberOf({ sheet, text: FIRST_AND_THIRD, },);
        expect(composite,).toBeGreaterThan(0,);
        expect(sheet.assessments.filter(function onComposite({ candidate, },): boolean {
          return candidate === composite;
        },).map(function linesOf({ coversLines, },): string {
          return JSON.stringify(coversLines,);
        },).toSorted(),).toEqual(['[1]', '[1]', '[3]',],);
      },
    },),
    it({
      name: 'MARKS NO ASSESSMENT AS COVERING LINES WHEN NO COMPOSITE IS OFFERED: overlapping revisions leave '
        + 'every assessment on the candidate its reviewer proposed',
      fn: async () => {
        const { prompts, } = await review({
          answers: [FIRST_LINE, FIRST_LINE_AGAIN, RETAIN, RETAIN,],
          best: 1,
        },);
        const { assessments, } = selectSheetOf({ prompts, },);
        expect(assessments.length,).toBeGreaterThan(0,);
        expect(assessments.some(function marked({ coversLines, },): boolean {
          return coversLines !== undefined;
        },),).toBe(false,);
      },
    },),
    it({
      name: 'REFUSES A COMPOSITE THAT IS NOT ON THE SLATE it is numbered against, naming how many candidates the '
        + 'slate holds, since its assessments would otherwise be filed under a number the judges never see',
      fn: async () => {
        /** A candidate standing for the composite, which the slate does not hold. */
        const stray = {
          producer: { kind: 'composite', contributors: [ROSTER[0],], } as const,
          value: FIRST_AND_THIRD,
          rendered: FIRST_AND_THIRD,
        };
        /** The evidence asked for with a composite the slate does not hold. */
        const strayEvidence = (): unknown => archiveBlockSelectionEvidence({
          sourceText: SOURCE,
          targetText: BLOCK,
          blockText: BLOCK,
          voices: [],
          candidates: [{ producer: { kind: 'incumbent', matched: [], } as const, value: BLOCK, rendered: BLOCK, },],
          composite: { candidate: stray, contributions: [{ candidate: 1, changed: [0,], },], },
          priorFindings: [],
        },);
        expect(strayEvidence,).toThrow(CompositeNotOnSlateError,);
        expect(strayEvidence,).toThrow('not one of the 1 candidate the judges are shown',);
      },
    },),
    it({
      name: 'KEEPS THE BLOCK AS THE ARCHIVE WROTE IT LAST: the judges naming candidate 4 retain the block',
      fn: async () => {
        const { outcome, lines, } = await review({
          answers: [FIRST_LINE, THIRD_LINE, RETAIN, RETAIN,],
          best: 4,
        },);
        expect(slateLine({ lines, },),).toContain('in 4 candidates',);
        expect(outcome.kind,).toBe('retained',);
        expect(outcome.text,).toBe(BLOCK,);
      },
    },),
    it({
      name: 'BUILDS NO COMPOSITE WHEN THE CHANGED LINES OVERLAP, and says which lines and which candidates: two '
        + 'revisions of the first line',
      fn: async () => {
        const { outcome, lines, } = await review({
          answers: [FIRST_LINE, FIRST_LINE_AGAIN, RETAIN, RETAIN,],
          best: 1,
        },);
        expect(slateLine({ lines, },),).toContain('in 3 candidates',);
        expect(logged({
          lines,
          parts: [
            'archive composite not built',
            'overlap',
            'line 1',
            'candidates 1 and 2',
          ],
        },),).toBe(true,);
        expect(outcome.text,).toBe(FIRST_LINE,);
      },
    },),
    it({
      name: 'TREATS A LINE TWO REVISIONS CHANGE IDENTICALLY AS AN OVERLAP: agreeing on a line is no reason to '
        + 'merge revisions that differ elsewhere, so no composite is built',
      fn: async () => {
        /** Revision changing the first line as the first revision does, and the last line as well. */
        const sameFirstLineAndMore = [
          'The cat naps by the window.',
          LINE_2,
          LINE_3,
          'The dog howls at the mailman.',
        ].join('\n',);
        const { lines, } = await review({
          answers: [FIRST_LINE, sameFirstLineAndMore, RETAIN, RETAIN,],
          best: 1,
        },);
        expect(slateLine({ lines, },),).toContain('in 3 candidates',);
        expect(logged({
          lines,
          parts: [
            'archive composite not built',
            'overlap',
            'line 1',
          ],
        },),).toBe(true,);
      },
    },),
    it({
      name: 'BUILDS NO COMPOSITE FROM THREE REVISIONS OF WHICH ONLY TWO ARE DISJOINT: choosing the pair would '
        + 'make the shipped text depend on which reviewers happened to answer, so the judges choose among the '
        + 'whole revisions and the block',
      fn: async () => {
        const { outcome, lines, } = await review({
          answers: [FIRST_LINE, THIRD_LINE, FIRST_LINE_AGAIN, RETAIN,],
          best: 1,
        },);
        expect(slateLine({ lines, },),).toContain('comparing 3 admissible revisions in 4 candidates',);
        expect(logged({
          lines,
          parts: [
            'archive composite not built',
            'overlap',
            'candidates 1 and 3',
          ],
        },),).toBe(true,);
        expect(outcome.text,).toBe(FIRST_LINE,);
      },
    },),
    it({
      name: 'BUILDS A COMPOSITE FROM THREE DISJOINT REVISIONS, every changed line of every one of them',
      fn: async () => {
        /** Revision of the second line only. */
        const secondLine = [
          LINE_1,
          'It chirps back at the birds.',
          LINE_3,
          LINE_4,
        ].join('\n',);
        const { outcome, lines, } = await review({
          answers: [FIRST_LINE, THIRD_LINE, secondLine, RETAIN,],
          best: 4,
        },);
        expect(slateLine({ lines, },),).toContain('comparing 3 admissible revisions in 5 candidates',);
        expect(outcome.text,).toBe([
          'The cat naps by the window.',
          'It chirps back at the birds.',
          'It won a ribbon last spring.',
          LINE_4,
        ].join('\n',),);
      },
    },),
    it({
      name: 'REPLACES THE LINE-COUNT REFUSAL (ledger B803): A REVISION THAT ADDS A LINE COMPOSES WITH A REVISION OF '
        + 'ANOTHER LINE, and the log names the added line by where it goes',
      fn: async () => {
        /** Revision that adds a line after the second. */
        const withAddedLine = [LINE_1, LINE_2, 'It naps at noon.', LINE_3, LINE_4,].join('\n',);
        const { outcome, lines, } = await review({
          answers: [FIRST_LINE, withAddedLine, RETAIN, RETAIN,],
          best: 3,
        },);
        expect(slateLine({ lines, },),).toContain('comparing 2 admissible revisions in 4 candidates',);
        expect(outcome.text,).toBe(
          ['The cat naps by the window.', LINE_2, 'It naps at noon.', LINE_3, LINE_4,].join('\n',),
        );
        expect(logged({
          lines,
          parts: [
            'archive composite built',
            '2 revisions',
            'candidate 2 adds 1 line after line 2',
          ],
        },),).toBe(true,);
        expect(logged({
          lines,
          parts: ['has 5 lines',],
        },),).toBe(false,);
      },
    },),
    it({
      name: 'A REVISION THAT REMOVES A LINE COMPOSES WITH A REVISION OF ANOTHER LINE: the removal is one hunk, and '
        + 'the other revision\'s reworded line stands beside it',
      fn: async () => {
        /** Revision that drops the third line. */
        const withoutThird = [LINE_1, LINE_2, LINE_4,].join('\n',);
        /** Revision of the last line. */
        const lastLine = [LINE_1, LINE_2, LINE_3, 'The dog howls at the mailman.',].join('\n',);
        const { outcome, lines, } = await review({
          answers: [withoutThird, lastLine, RETAIN, RETAIN,],
          best: 3,
        },);
        expect(outcome.text,).toBe([LINE_1, LINE_2, 'The dog howls at the mailman.',].join('\n',),);
        expect(logged({
          lines,
          parts: [
            'archive composite built',
            'candidate 1 removes line 3',
            'candidate 2 replaces line 4 with 1 line',
          ],
        },),).toBe(true,);
      },
    },),
    it({
      name: 'A REVISION THAT BOTH REMOVES AND ADDS LINES COMPOSES, each of its hunks keeping its place beside '
        + 'the other revisions\' hunks: a hunk that ends where another begins does not overlap it',
      fn: async () => {
        /** Revision that drops the second line and adds one before the last. */
        const movedAround = [LINE_1, LINE_3, 'The bird sings in the tree.', LINE_4,].join('\n',);
        /** Revision of the first line. */
        const first = FIRST_LINE;
        /** Revision of the last line, which the added line sits before. */
        const last = [LINE_1, LINE_2, LINE_3, 'The dog howls at the mailman.',].join('\n',);
        const { outcome, lines, } = await review({
          answers: [movedAround, first, last, RETAIN,],
          best: 4,
        },);
        expect(slateLine({ lines, },),).toContain('comparing 3 admissible revisions in 5 candidates',);
        expect(outcome.text,).toBe([
          'The cat naps by the window.',
          LINE_3,
          'The bird sings in the tree.',
          'The dog howls at the mailman.',
        ].join('\n',),);
        expect(logged({
          lines,
          parts: [
            'candidate 1 removes line 2 and adds 1 line after line 3',
          ],
        },),).toBe(true,);
      },
    },),
    it({
      name: 'MARKS THE LINES OF THE COMPOSITE EACH CONTRIBUTOR\'S ASSESSMENTS COVER when the revisions add and '
        + 'remove lines: positions in the composite, which no longer match the block\'s',
      fn: async () => {
        /** Revision that adds a line after the first. */
        const withAddedLine = [LINE_1, 'It stretches first.', LINE_2, LINE_3, LINE_4,].join('\n',);
        /** Revision that drops the third line. */
        const withoutThird = [LINE_1, LINE_2, LINE_4,].join('\n',);
        /** The composite of the two. */
        const composite = [LINE_1, 'It stretches first.', LINE_2, LINE_4,].join('\n',);
        const { prompts, } = await review({
          answers: [withAddedLine, withoutThird, RETAIN, RETAIN,],
          best: 3,
        },);
        // The note on the sheet says what a removal's entry covers, since no line of the composite is its own.
        expect(prompts.some(function saysRemoval(prompt,): boolean {
          return prompt.includes('the line that now follows the cut',);
        },),).toBe(true,);
        const sheet = selectSheetOf({ prompts, },);
        const number = numberOf({ sheet, text: composite, },);
        expect(number,).toBeGreaterThan(0,);
        expect(sheet.assessments
          .filter(function onComposite({ candidate, },): boolean {
            return candidate === number;
          },)
          .map(function covered({ coversLines, fromCandidate, },): string {
            return `${String(fromCandidate,)}:${JSON.stringify(coversLines,)}`;
          },),).toEqual(['1:[2]', '2:[4]',],);
      },
    },),
    it({
      name: 'BUILDS NO COMPOSITE WHEN TWO REVISIONS ADD AT THE SAME PLACE: which added line comes first is '
        + 'undetermined, so the overlap rule of ledger B712 holds and the log names the place',
      fn: async () => {
        /** Revision that adds a line after the second. */
        const addsNoon = [LINE_1, LINE_2, 'It naps at noon.', LINE_3, LINE_4,].join('\n',);
        /** Revision that adds another line at the same place. */
        const addsDusk = [LINE_1, LINE_2, 'It naps at dusk.', LINE_3, LINE_4,].join('\n',);
        const { outcome, lines, } = await review({
          answers: [addsNoon, addsDusk, RETAIN, RETAIN,],
          best: 1,
        },);
        expect(slateLine({ lines, },),).toContain('in 3 candidates',);
        expect(logged({
          lines,
          parts: [
            'archive composite not built',
            'overlap',
            'after line 2',
            'candidates 1 and 2',
          ],
        },),).toBe(true,);
        expect(outcome.text,).toBe(addsNoon,);
      },
    },),
    it({
      name: 'BUILDS NO COMPOSITE WHEN AN ADDED LINE FALLS INSIDE LINES ANOTHER REVISION REMOVES: the added line '
        + 'would sit in a gap the other revision closes',
      fn: async () => {
        /** Revision that adds a line between the second and third. */
        const addsBetween = [LINE_1, LINE_2, 'It naps at noon.', LINE_3, LINE_4,].join('\n',);
        /** Revision that drops the second and third lines. */
        const dropsBoth = [LINE_1, LINE_4,].join('\n',);
        const { lines, } = await review({
          answers: [addsBetween, dropsBoth, RETAIN, RETAIN,],
          best: 1,
        },);
        expect(slateLine({ lines, },),).toContain('in 3 candidates',);
        expect(logged({
          lines,
          parts: [
            'archive composite not built',
            'overlap',
            'after line 2',
            'candidates 1 and 2',
          ],
        },),).toBe(true,);
      },
    },),
    it({
      name: 'BUILDS NO COMPOSITE WHEN ONE REVISION REWORDS A LINE ANOTHER REMOVES, and says the log\'s line',
      fn: async () => {
        /** Revision that drops the second line. */
        const withoutSecond = [LINE_1, LINE_3, LINE_4,].join('\n',);
        /** Revision that rewords the second line. */
        const reworded = [LINE_1, 'It sings to the birds.', LINE_3, LINE_4,].join('\n',);
        const { lines, } = await review({
          answers: [withoutSecond, reworded, RETAIN, RETAIN,],
          best: 1,
        },);
        expect(logged({
          lines,
          parts: [
            'archive composite not built',
            'overlap on line 2',
            'candidates 1 and 2',
          ],
        },),).toBe(true,);
      },
    },),
    it({
      name: 'BUILDS NO COMPOSITE WHEN THE REVISION BESIDE AN ADDITION IS A REMOVAL OF THE WHOLE BLOCK: a removal '
        + 'edits no line, it is a verdict on the block, so nothing combines with it',
      fn: async () => {
        /** Revision that adds a line after the second. */
        const addsNoon = [LINE_1, LINE_2, 'It naps at noon.', LINE_3, LINE_4,].join('\n',);
        const { lines, } = await review({
          answers: [addsNoon, '', RETAIN, RETAIN,],
          best: 1,
        },);
        expect(logged({
          lines,
          parts: [
            'archive composite not built',
            'removal',
          ],
        },),).toBe(true,);
      },
    },),
    it({
      name: 'BUILDS NO COMPOSITE OF AN ADDITION BESIDE A REVISION EQUAL TO THE ARCHIVE: the revision that equals '
        + 'the block changes no line, so one contributor is nothing to combine',
      fn: async () => {
        /** Revision that adds a line after the second. */
        const addsNoon = [LINE_1, LINE_2, 'It naps at noon.', LINE_3, LINE_4,].join('\n',);
        const { lines, } = await review({
          answers: [addsNoon, BLOCK, RETAIN, RETAIN,],
          best: 1,
        },);
        expect(slateLine({ lines, },),).toContain('comparing 2 admissible revisions in 2 candidates',);
        expect(logged({
          lines,
          parts: [
            'archive composite not built',
            '1 of 2',
          ],
        },),).toBe(true,);
      },
    },),
    it({
      name: 'COMPOSES ADDED AND REMOVED LINES AROUND LIST MARKERS, QUOTES AND BLANK LINES: a quoted list whose '
        + 'blank quote lines repeat keeps its shape, and a straight apostrophe in an added line is restored to '
        + 'the page\'s own',
      fn: async () => {
        /** Quoted list block with a repeated blank quote line between paragraphs. */
        const list = ['> - first cat', '> - second cat', '>', '> The cat’s nap.', '>', '> The end.',].join('\n',);
        /** Revision that adds a list item and a straight apostrophe. */
        const addsItem = [
          '> - first cat',
          '> - second cat',
          '> - third cat, the owner\'s',
          '>',
          '> The cat’s nap.',
          '>',
          '> The end.',
        ].join('\n',);
        /** Revision that removes the middle paragraph and one blank quote line with it. */
        const dropsMiddle = ['> - first cat', '> - second cat', '>', '> The end.',].join('\n',);
        const { outcome, } = await review({
          answers: [addsItem, dropsMiddle, RETAIN, RETAIN,],
          best: 3,
          blockText: list,
        },);
        expect(outcome.text,).toBe(
          ['> - first cat', '> - second cat', '> - third cat, the owner’s', '>', '> The end.',].join('\n',),
        );
      },
    },),
    it({
      name: 'WITHHOLDS A COMPOSITE WHOSE REMOVALS TOGETHER DROP EVERY MARKER OF A NOTE: each revision removes a '
        + 'line with one of the note\'s two markers, which leaves the note referenced, and together they drop both',
      fn: async () => {
        /** Block citing the page\'s note on two lines among others. */
        const cited = 'Cats[^1] nap.\nDogs bark.\nBirds[^1] sing.\nFish swim.';
        const page = `${cited}\n\n[^1]: A note.`;
        const { outcome, lines, } = await review({
          answers: ['Dogs bark.\nBirds[^1] sing.\nFish swim.', 'Cats[^1] nap.\nDogs bark.\nFish swim.', RETAIN, RETAIN,],
          best: 1,
          blockText: cited,
          targetText: page,
        },);
        expect(slateLine({ lines, },),).toContain('comparing 2 admissible revisions in 3 candidates',);
        expect(logged({
          lines,
          parts: [
            'archive composite not built',
            'footnote',
          ],
        },),).toBe(true,);
        expect(outcome.findings.some(function refusesComposite(finding,): boolean {
          return finding.startsWith('archive-revision-refused (composite(',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'BUILDS NO COMPOSITE WHEN ONE OF THE REVISIONS IS A REMOVAL, which has no lines to match',
      fn: async () => {
        const { lines, } = await review({
          answers: [FIRST_LINE, THIRD_LINE, '', RETAIN,],
          best: 1,
        },);
        expect(logged({
          lines,
          parts: [
            'archive composite not built',
            'removal',
          ],
        },),).toBe(true,);
      },
    },),
    it({
      name: 'READS A TRAILING NEWLINE AS A CHANGE TO THE LAST LINE: a revision that only adds one composes with '
        + 'a revision of another line, and collides with a revision of the last line',
      fn: async () => {
        /** The first revision with a newline after the last line. */
        const withNewline = `${THIRD_LINE}\n`;
        const composed = await review({
          answers: [FIRST_LINE, withNewline, RETAIN, RETAIN,],
          best: 3,
        },);
        expect(composed.outcome.text,).toBe(`${FIRST_AND_THIRD}\n`,);
        /** Revision of the last line. */
        const lastLine = [
          LINE_1,
          LINE_2,
          LINE_3,
          'The dog howls at the mailman.',
        ].join('\n',);
        const collided = await review({
          answers: [lastLine, `${LINE_1}\n${LINE_2}\n${LINE_3}\n${LINE_4}\n`, RETAIN, RETAIN,],
          best: 1,
        },);
        expect(slateLine({ lines: collided.lines, },),).toContain('in 3 candidates',);
        expect(logged({
          lines: collided.lines,
          parts: [
            'archive composite not built',
            'overlap',
            'line 4',
          ],
        },),).toBe(true,);
      },
    },),
    it({
      name: 'COUNTS AN EMPTY LINE AS A LINE: a block with a blank line between two paragraphs composes around it '
        + 'and keeps it',
      fn: async () => {
        /** Two paragraphs with the empty line between them. */
        const twoParagraphs = 'The cat sleeps.\n\nThe dog barks.';
        const { outcome, } = await review({
          answers: ['The cat naps.\n\nThe dog barks.', 'The cat sleeps.\n\nThe dog howls.', RETAIN, RETAIN,],
          best: 3,
          blockText: twoParagraphs,
        },);
        expect(outcome.text,).toBe('The cat naps.\n\nThe dog howls.',);
      },
    },),
    it({
      name: 'KEEPS CARRIAGE RETURNS WITH THEIR LINES: revisions of a CRLF block compose in CRLF, and a revision '
        + 'that only rewrites every ending as a bare line feed changes every line and so collides',
      fn: async () => {
        /** The block with Windows endings. */
        const crlf = BLOCK.replaceAll('\n', '\r\n',);
        const composed = await review({
          answers: [FIRST_LINE.replaceAll('\n', '\r\n',), THIRD_LINE.replaceAll('\n', '\r\n',), RETAIN, RETAIN,],
          best: 3,
          blockText: crlf,
        },);
        expect(composed.outcome.text,).toBe(FIRST_AND_THIRD.replaceAll('\n', '\r\n',),);
        const normalized = await review({
          answers: [FIRST_LINE.replaceAll('\n', '\r\n',), THIRD_LINE, RETAIN, RETAIN,],
          best: 1,
          blockText: crlf,
        },);
        expect(slateLine({ lines: normalized.lines, },),).toContain('in 3 candidates',);
        expect(logged({
          lines: normalized.lines,
          parts: [
            'archive composite not built',
            'overlap',
          ],
        },),).toBe(true,);
      },
    },),
    it({
      name: 'LEAVES OUT A REVISION EQUAL TO THE ARCHIVE: it changes no line, so it is no contributor and the '
        + 'composite is of the others, while one revision beside it leaves nothing to combine',
      fn: async () => {
        const combined = await review({
          answers: [FIRST_LINE, THIRD_LINE, BLOCK, RETAIN,],
          best: 4,
        },);
        // The revision equal to the block is the block's own candidate, so the block is not added again.
        expect(slateLine({ lines: combined.lines, },),).toContain('comparing 3 admissible revisions in 4 candidates',);
        expect(combined.outcome.text,).toBe(FIRST_AND_THIRD,);
        expect(logged({
          lines: combined.lines,
          parts: [`composite(${ROSTER[0]} + ${ROSTER[1]})`,],
        },),).toBe(true,);
        const alone = await review({
          answers: [FIRST_LINE, BLOCK, RETAIN, RETAIN,],
          best: 1,
        },);
        expect(slateLine({ lines: alone.lines, },),).toContain('comparing 2 admissible revisions in 2 candidates',);
        expect(logged({
          lines: alone.lines,
          parts: [
            'archive composite not built',
            '1 of 2',
          ],
        },),).toBe(true,);
      },
    },),
    it({
      name: 'OFFERS NO COMPOSITE OF ONE REVISION and logs nothing about one',
      fn: async () => {
        const { lines, } = await review({
          answers: [FIRST_LINE, RETAIN, RETAIN, RETAIN,],
          best: 1,
        },);
        expect(slateLine({ lines, },),).toContain('comparing 1 admissible revision in 2 candidates',);
        expect(logged({
          lines,
          parts: ['archive composite',],
        },),).toBe(false,);
      },
    },),
    it({
      name: 'WITHHOLDS A COMPOSITE THAT LEAVES A FOOTNOTE DEFECT NEITHER REVISION DOES: each drops one of the '
        + 'note\'s two markers, which leaves the note referenced, and together they drop both',
      fn: async () => {
        /** Block citing the page\'s note on two lines. */
        const cited = 'Cats[^1] nap.\nDogs bark.\nBirds[^1] sing.';
        /** The page, with the note the block cites. */
        const page = `${cited}\n\n[^1]: A note.`;
        const { outcome, lines, } = await review({
          answers: ['Cats nap.\nDogs bark.\nBirds[^1] sing.', 'Cats[^1] nap.\nDogs bark.\nBirds sing.', RETAIN, RETAIN,],
          best: 1,
          blockText: cited,
          targetText: page,
        },);
        expect(slateLine({ lines, },),).toContain('comparing 2 admissible revisions in 3 candidates',);
        expect(logged({
          lines,
          parts: [
            'archive composite not built',
            'footnote',
          ],
        },),).toBe(true,);
        expect(outcome.findings.some(function refusesComposite(finding,): boolean {
          return finding.startsWith('archive-revision-refused (composite(',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'KEEPS A MARKER ON A CHANGED LINE WHEN THE NOTE STAYS REFERENCED: revisions that reword lines '
        + 'holding the marker compose',
      fn: async () => {
        /** Block citing the page\'s note on two lines. */
        const cited = 'Cats[^1] nap.\nDogs bark.\nBirds[^1] sing.';
        const page = `${cited}\n\n[^1]: A note.`;
        const { outcome, } = await review({
          answers: ['Cats[^1] doze.\nDogs bark.\nBirds[^1] sing.', 'Cats[^1] nap.\nDogs bark.\nBirds[^1] chirp.', RETAIN, RETAIN,],
          best: 3,
          blockText: cited,
          targetText: page,
        },);
        expect(outcome.text,).toBe('Cats[^1] doze.\nDogs bark.\nBirds[^1] chirp.',);
      },
    },),
    it({
      name: 'WITHHOLDS A COMPOSITE THAT SHOWS A READER NOTHING though each revision shows something: each '
        + 'blanks the paragraph the other keeps',
      fn: async () => {
        /** Two paragraphs. */
        const twoParagraphs = 'The cat sleeps.\n\nThe dog barks.';
        const { lines, } = await review({
          answers: ['\u3164\n\nThe dog barks.', 'The cat sleeps.\n\n\u3164', RETAIN, RETAIN,],
          best: 1,
          blockText: twoParagraphs,
        },);
        expect(logged({
          lines,
          parts: [
            'archive composite not built',
            'shows a reader nothing',
          ],
        },),).toBe(true,);
      },
    },),
  ],
},);
