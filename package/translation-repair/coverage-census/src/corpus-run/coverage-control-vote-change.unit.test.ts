/**
 Tests that the coverage control counts a case ONLY WHEN THE VOTE MOVED.

 WHAT THE CONTROL IS FOR. It exists to answer one question: can this wire vote
 absence at all? A roster that answers `full` to the undamaged passage and
 `full` again once the rendering it pointed at has been deleted has shown
 exactly nothing, and that is the reading the whole gate was built to refuse.

 WHAT WAS MEASURED. On 2026-08-25, relaxing the comparison that counts a case
 from `absentAfter > absentBefore` to `absentAfter >= absentBefore` failed no
 test in this package. Under that relaxation every damaged case counts as a
 case where the wire noticed, including one whose votes never moved, so a wire
 that cannot see damage reports a HELD control and licenses the null it was
 supposed to invalidate.

 COUNTED ON VOTES, NOT ON THE VERDICT KIND, which is what the field's own
 documentation says: the recorded null is about ballots rather than about how
 they were rolled up.

 NO NETWORK. Three kinds of scripted client stand in for the roster.

 The blind client (`coverageControlClient`, from the shared fixture) answers
 every round with full coverage, quoting a sentence really present, so the
 standing verdict carries, its evidence can be located and cut, and the answer
 after the cut is identical to the answer before it. That is a wire blind to
 damage, spelled out.

 The sheet-reading client (`sheetReadingClient`) answers each round by what
 the sheet carries: full coverage quoting the sentence while the judged
 rendering is on the sheet, none with an empty quote when it is not. One
 condition makes it follow the quoted sentence alone, a wire that sees the
 targeted damage and nothing else; the other makes it follow the whole page,
 a wire that sees any cut.

 The denying client answers every round that the passage is not carried at
 all, so the undamaged verdict never claims the passage.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  coverageControlHolds,
  type CoverageControlRow,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  coverageControlCasesAt,
  coverageControlClient,
} from './coverage-control-cases.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the control under test.
 */
const l = tagged({ tag: 'coverage-control-vote-change-test', },);

//region Fixtures

/**
 Sentence every scripted judge quotes, verbatim from `TRANSLATION`, so
 the standing verdict carries and its evidence can be located and cut.
 */
const QUOTED = 'Whiskers counts the birds outside.';

/**
 Translation the cases ask about, long enough on both sides of the quoted
 sentence that an equally large decoy cut has somewhere to go.
 */
const TRANSLATION = `The kitten dozes on the windowsill in the afternoon sun. ${QUOTED} `
  + 'The tabby sleeps by the radiator until evening.';

/**
 Original passage the cases ask about.
 */
const SOURCE_PASSAGE = '白胡子数着外面的鸟。';

/**
 Roster asked at every round.
 */
const MODEL_IDS = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Three damageable cases, distinguished only by where they sit.
 */
const CASES = coverageControlCasesAt({
  where: ['slice-0', 'slice-1', 'slice-2',],
  sourcePassage: SOURCE_PASSAGE,
  translationText: TRANSLATION,
},);

/**
 Client answering EVERY round the same way, damaged or not: a wire whose votes
 do not move is the one the control has to refuse.
 */
const BLIND_CLIENT = coverageControlClient({ quote: QUOTED, },);

/**
 Translation the quoted sentence fills but for its last two characters, a
 space and one letter: 36 characters holding a sentence of 34, so no window
 clear of the sentence can take a cut of the sentence's own length.
 */
const TIGHT_TRANSLATION = `${QUOTED} x`;

/**
 Three cases with no room for a decoy cut, distinguished only by where they
 sit.
 */
const TIGHT_CASES = coverageControlCasesAt({
  where: ['tight-0', 'tight-1', 'tight-2',],
  sourcePassage: SOURCE_PASSAGE,
  translationText: TIGHT_TRANSLATION,
},);

/**
 Client whose vote follows what the sheet shows: full coverage quoting
 `QUOTED` where the sheet still carries the rendering, none with an empty
 quote where it does not, the two never contradicting.

 @param carries - whether a sheet, read as one text, still carries the
 rendering in this client's judgement

 @returns Client answering every coverage round by that judgement

 @example
 ```ts
 const client = sheetReadingClient({ carries: function showsQuote(sheet,) { return sheet.includes(QUOTED,); }, },);
 ```
 */
function sheetReadingClient(
  { carries, }: { readonly carries: (sheet: string) => boolean; },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the coverage control',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,) => {
      /**
       Sheet as one text, where the rendering rides or does not.
       */
      const sheet = request.messages
        .map(function toContent(message,) {
          return messageText({ message, },);
        },)
        .join('\n',);
      /**
       Whether this client reads the sheet as still carrying the rendering.
       */
      const carried = carries(sheet,);
      const scripted: unknown = {
        coverage: carried ? 'full' : 'none',
        copiedSpan: carried ? QUOTED : '',
        assessment: 'fixture',
      };
      if (!request.validate(scripted,))
        return {
          kind: 'schema-mismatch',
          rawText: '',
          detail: 'fixture',
        };
      return {
        kind: 'ok',
        value: scripted as ValueT,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the coverage control',);
    },
  };
}

/**
 Client whose vote follows the quoted rendering on the sheet. A wire that sees
 the damage, and sees nothing else.
 */
const MOVING_CLIENT = sheetReadingClient({
  carries: function showsQuote(sheet,): boolean {
    return sheet.includes(QUOTED,);
  },
},);

/**
 Client that calls the passage covered only while the sheet shows one of the
 two translations whole. A wire answering the damage rather than the
 question: any cut, targeted or not, moves its vote.
 */
const DAMAGE_READING_CLIENT = sheetReadingClient({
  carries: function showsAWholePage(sheet,): boolean {
    return sheet.includes(TRANSLATION,) || sheet.includes(TIGHT_TRANSLATION,);
  },
},);

/**
 Rows the control records for cases alike in everything but where they sit.

 @param where - location label per row, in the order the cases were offered

 @param row - every other field, the same for each of them

 @returns One row per label

 @example
 ```ts
 const rows = rowsAt({ where: ['slice-0', 'slice-1',], row, },);
 ```
 */
function rowsAt(
  {
    where,
    row,
  }: {
    readonly where: readonly string[];
    readonly row: Omit<CoverageControlRow, 'where'>;
  },
): readonly CoverageControlRow[] {
  return where.map(function toRow(oneWhere,): CoverageControlRow {
    return {
      where: oneWhere,
      ...row,
    };
  },);
}

//endregion Fixtures

await describe({
  name: coverageControlHolds.name,
  // ONE AT A TIME: the no-decoy case diverts the process-wide `console.log`
  // (ledger B79), and every case here prints through it.
  concurrency: 1,
  children: [
    it({
      name: 'REFUSES to hold when deleting the rendering changed no vote, which is the reading the '
        + 'gate exists to invalidate: a wire answering the same way to the damaged page as to the '
        + 'undamaged one has shown the absence vote unreachable, not the passage covered',
      fn: async () => {
        /**
         What the control made of a roster whose votes never move.
         */
        const control = await coverageControlHolds({
          client: BLIND_CLIENT,
          cases: CASES,
          modelIds: [...MODEL_IDS,],
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        // Every case was damageable, so all three reached the after round and
        // the length of `control.rows` is taken over three rows rather than
        // over none.
        expect(control.rows,).toHaveLength(3,);
        expect(control.refusals,).toStrictEqual([],);

        // The vote is identical before and after. Nothing moved, so nothing was
        // noticed, and a count of three here would mean the control credits a
        // wire for answering at all rather than for answering differently.
        expect(control.sawAbsenceOnTarget,).toBe(0,);
        expect(control.held,).toBe(false,);
      },
    },),

    it({
      name: 'RECORDS A REFUSAL for a case whose undamaged verdict never claimed the passage is '
        + 'carried, since only a carried case can have its rendering deleted',
      fn: async () => {
        /**
         Client answering that the passage is not carried at all.
         */
        const denying: ModelClient = {
          chatText: async () => {
            throw new Error('chatText unused by the coverage control',);
          },
          chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,) => {
            /**
             Reply claiming the passage is absent.
             */
            const scripted: unknown = {
              coverage: 'none',
              copiedSpan: '',
              assessment: 'fixture',
            };
            if (!request.validate(scripted,))
              return {
                kind: 'schema-mismatch',
                rawText: '',
                detail: 'fixture',
              };
            return {
              kind: 'ok',
              value: scripted as ValueT,
              rawText: JSON.stringify(scripted,),
            };
          },
          quotas: async () => {
            throw new Error('quotas unused by the coverage control',);
          },
        };
        const control = await coverageControlHolds({
          client: denying,
          cases: CASES,
          modelIds: [...MODEL_IDS,],
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(control,).toEqual({
          held: false,
          sawAbsenceOnTarget: 0,
          sawAbsenceOnDecoy: 0,
          decoysTaken: 0,
          rows: [],
          refusals: ['slice-0', 'slice-1', 'slice-2',].map(function toRefusal(where,) {
            return {
              where,
              reason: 'not-carried',
              verdict: 'absent',
              absent: 3,
              offeredSpans: 0,
            };
          },),
        },);
      },
    },),

    it({
      name: 'RECORDS A cut-left-nothing REFUSAL when the spans the roster anchored on are the whole '
        + 'translation, since nothing is left to ask it about',
      fn: async () => {
        /**
         Cases whose anchored span is the whole translation.
         */
        const whole = coverageControlCasesAt({
          where: ['envelope/whole',],
          sourcePassage: SOURCE_PASSAGE,
          translationText: TRANSLATION,
        },);
        const control = await coverageControlHolds({
          client: coverageControlClient({ quote: TRANSLATION, },),
          cases: whole,
          modelIds: [...MODEL_IDS,],
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(control,).toEqual({
          held: false,
          sawAbsenceOnTarget: 0,
          sawAbsenceOnDecoy: 0,
          decoysTaken: 0,
          rows: [],
          refusals: [
            {
              where: 'envelope/whole',
              reason: 'cut-left-nothing',
              verdict: 'carried',
              absent: 0,
              offeredSpans: 3,
            },
          ],
        },);
      },
    },),

    it({
      name: 'RECORDS A ROW whose decoy reads no-room when no window clear of the anchored span can take '
        + 'a cut of the same size',
      fn: async () => {
        const control = await coverageControlHolds({
          client: BLIND_CLIENT,
          cases: coverageControlCasesAt({
            where: ['envelope/tight',],
            sourcePassage: SOURCE_PASSAGE,
            translationText: TIGHT_TRANSLATION,
          },),
          modelIds: [...MODEL_IDS,],
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(control,).toEqual({
          held: false,
          sawAbsenceOnTarget: 0,
          sawAbsenceOnDecoy: 0,
          decoysTaken: 0,
          rows: rowsAt({
            where: ['envelope/tight',],
            row: {
              before: 'carried',
              after: 'split',
              absentBefore: 0,
              absentAfter: 0,
              decoy: 'no-room',
              absentAfterDecoy: 0,
              decoyAt: -1,
              removedSpans: 3,
              removedChars: 34,
            },
          },),
          refusals: [],
        },);
      },
    },),

    it({
      name: 'REFUSES to hold when no case had room for a decoy cut, and prints why, since a control that '
        + 'never ran its decoy half has not shown the roster quiet on an unrelated cut',
      fn: async (ctx,) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        const control = await coverageControlHolds({
          client: MOVING_CLIENT,
          cases: TIGHT_CASES,
          modelIds: [...MODEL_IDS,],
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        // The targeted cut moved every vote, so only the decoy half is
        // missing: this case fails on that half and on nothing else.
        expect(control,).toEqual({
          held: false,
          sawAbsenceOnTarget: 3,
          sawAbsenceOnDecoy: 0,
          decoysTaken: 0,
          rows: rowsAt({
            where: ['tight-0', 'tight-1', 'tight-2',],
            row: {
              before: 'carried',
              after: 'absent',
              absentBefore: 0,
              absentAfter: 3,
              decoy: 'no-room',
              absentAfterDecoy: 0,
              decoyAt: -1,
              removedSpans: 3,
              removedChars: 34,
            },
          },),
          refusals: [],
        },);
        expect(printed.lines,).toEqual([
          ...['tight-0', 'tight-1', 'tight-2',].map(function toLine(where,) {
            return `COVERAGE control ${where}: carried -> absent, absence votes 0 -> 3, cut 3 spans of 34 `
              + 'chars; DECOY of the same size at -1: no-room, absence votes 0';
          },),
          'COVERAGE control CANNOT HOLD: a decoy cut was taken on none of 3 damaged cases, no page having '
            + 'room for one clear of the spans the roster anchored on, so nothing shows whether an unrelated '
            + 'cut of the same size moves the vote too',
        ],);
      },
    },),

    it({
      name: 'REFUSES to hold when the one decoy cut taken moved the vote, counting the cases with no room '
        + 'for one as no decoy at all rather than as decoys that stayed quiet',
      fn: async () => {
        const control = await coverageControlHolds({
          client: DAMAGE_READING_CLIENT,
          cases: [
            ...coverageControlCasesAt({
              where: ['slice-0',],
              sourcePassage: SOURCE_PASSAGE,
              translationText: TRANSLATION,
            },),
            ...TIGHT_CASES.slice(
              0,
              2,
            ),
          ],
          modelIds: [...MODEL_IDS,],
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        // One decoy was taken and it moved every vote, so the decoy half has
        // shown the wire answering the damage; the two rows with no room add
        // nothing to either side of that.
        expect(control,).toEqual({
          held: false,
          sawAbsenceOnTarget: 3,
          sawAbsenceOnDecoy: 1,
          decoysTaken: 1,
          rows: [
            ...rowsAt({
              where: ['slice-0',],
              row: {
                before: 'carried',
                after: 'absent',
                absentBefore: 0,
                absentAfter: 3,
                decoy: 'absent',
                absentAfterDecoy: 3,
                decoyAt: 105,
                removedSpans: 3,
                removedChars: 34,
              },
            },),
            ...rowsAt({
              where: ['tight-0', 'tight-1',],
              row: {
                before: 'carried',
                after: 'absent',
                absentBefore: 0,
                absentAfter: 3,
                decoy: 'no-room',
                absentAfterDecoy: 0,
                decoyAt: -1,
                removedSpans: 3,
                removedChars: 34,
              },
            },),
          ],
          refusals: [],
        },);
      },
    },),

    it({
      name: 'HOLDS when the absence vote moves with the targeted cut and never with the decoy one',
      fn: async () => {
        const control = await coverageControlHolds({
          client: MOVING_CLIENT,
          cases: CASES,
          modelIds: [...MODEL_IDS,],
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(control,).toEqual({
          held: true,
          sawAbsenceOnTarget: 3,
          sawAbsenceOnDecoy: 0,
          decoysTaken: 3,
          rows: rowsAt({
            where: ['slice-0', 'slice-1', 'slice-2',],
            row: {
              before: 'carried',
              after: 'absent',
              absentBefore: 0,
              absentAfter: 3,
              decoy: 'carried',
              absentAfterDecoy: 0,
              decoyAt: 105,
              removedSpans: 3,
              removedChars: 34,
            },
          },),
          refusals: [],
        },);
      },
    },),
  ],
},);
