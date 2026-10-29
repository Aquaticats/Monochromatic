/**
 Guards the owner's answer of 2026-10-29, "Ship the two-ballot text (Recommended)": when the final round of a
 consolidation cannot seat two judges, the translate lane's winner ships if two judges chose it and it passes the
 publication rule. One judge alone never decides (2026-10-25), a slice with no wording that passes the rule keeps the
 archive (2026-09-27, "Keep archive, ship"), and a disputed archive is never a candidate or a fallback (2026-09-27).

 XingZ6022 (2026-10-29) kept the archive at sixteen slices on a one-judge slate although the translate lane held a text
 two judges had chosen that passed the rule, and at slices 67 to 70 the archive carries an invented quotation. Its
 consolidation of those slices bought 65 calls and 10.679 Wh for an outcome the one-judge slate could not change.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import type { LaneText, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  buyConsolidationSlice,
  type ConsolidationSettlement,
  consolidationWorthResuming,
  settleConsolidation,
  standingKeptUnendorsed,
} from '../dist/final/node/index.mjs';
import {
  capturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the stages write through, whose output is not under test.
 */
const l = tagged({ tag: 'translate-winner-ships-test', },);

/**
 Invented original: the cat dozed on the sill, tail by the radiator.
 */
const SOURCE = '猫猫在窗台上打盹，尾巴垂在暖气片旁边。';

/**
 The archive's rendering, which the repair lane's panel disputed: it adds a purr the original never states.
 */
const ARCHIVE = 'The cat dozed on the windowsill and purred for you.';

/**
 What the translate lane would ship, which two of its judges chose.
 */
const TRANSLATE_TEXT = 'The cat naps on the sill, tail beside the radiator.';

/**
 Writers and slate judges of the cases.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 The translate lane's text as the lane offer hands it over once it passed the rule.
 */
const TRANSLATE_LANE: LaneText = {
  lane: 'translate',
  text: TRANSLATE_TEXT,
};

/**
 Counts of what a scripted client was asked.
 */
type Asked = {
  /**
   Every call of any kind.
   */
  calls: number;

  /**
   Refiner sheets, which only a polish round sends.
   */
  refinerSheets: number;
};

/**
 Client under which every slate judge names no candidate.

 @param asked - where the calls are counted

 @returns Client over every stage these cases reach

 @example
 ```ts
 const client = rejectingClient({ asked: { calls: 0, refinerSheets: 0, }, });
 ```
 */
function rejectingClient({ asked, }: { readonly asked: Asked; },): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by these stages',);
    },
    quotas: async () => {
      throw new Error('quotas unused by these stages',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      asked.calls += 1;
      /**
       Stage the request belongs to, by its reply schema.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';
      // The sheet is read so a fixture reply can never depend on a message nobody built.
      expect(request.messages.map(function textOf(message,): string {
        return messageText({ message, },);
      },).join('\n',).length,).toBeGreaterThan(0,);
      if (schema === 'refine_report')
        asked.refinerSheets += 1;
      /**
       Reply for the stage.
       */
      const value: unknown = (schema === 'candidate_ballot')
        ? {
          best: 0,
          assessment: 'No candidate renders the original without an addition.',
        }
        : (schema === 'refine_report')
          ? { rewrites: [], }
          : (schema === 'absolute_naturalness_review')
            ? {
              isAcceptable: true,
              findings: [],
              assessment: 'publication-ready',
            }
            : undefined;
      if (value === undefined)
        throw new Error(`no fixture reply for ${schema}`,);
      if (!request.validate(value,))
        throw new Error(`fixture ${schema} reply failed validation`,);
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
  };
}

/**
 Settles a slice whose standing the rule refused, with a stated slate and lane offer.

 @param judgeCount - slate judges seated, who all name no candidate

 @param laneTexts - lane texts the offer put on the slate

 @param translateWinnerBallots - ballots that named the translate lane's text in its own round, absent when not counted

 @param standingEligible - whether the standing passed the deterministic gate

 @param asked - where the calls are counted

 @returns Settlement

 @example
 ```ts
 const settled = await settleOver({ judgeCount: 1, laneTexts: [TRANSLATE_LANE], translateWinnerBallots: 2, standingEligible: false, asked, });
 ```
 */
async function settleOver(
  {
    judgeCount,
    laneTexts,
    translateWinnerBallots,
    standingEligible,
    asked,
  }: {
    readonly judgeCount: number;
    readonly laneTexts: readonly LaneText[];
    readonly translateWinnerBallots?: number;
    readonly standingEligible: boolean;
    readonly asked: Asked;
  },
): Promise<ConsolidationSettlement> {
  return await settleConsolidation({
    client: rejectingClient({ asked, },),
    roster: ROSTER,
    judgeModelIds: ROSTER.slice(0, judgeCount,),
    subject: {
      sourceText: SOURCE,
      incumbentText: ARCHIVE,
    },
    // NO WRITER IS HEARD: the slate is the lane text alone, so no proposal is wrapped through the lint binary, whose
    // pinned version this tree's shared binary does not match until the typography round lands.
    voices: [],
    validity: [],
    producedFindings: [],
    standingText: ARCHIVE,
    lineStructured: false,
    sliceIndex: 4,
    polishConfig: {
      refinerModelIds: [ROSTER[0],],
      judgeModelIds: ROSTER,
      gateModelIds: ROSTER,
      declaredNames: [],
      definitions: '',
    },
    standingMayShip: false,
    standingEligible,
    standingRefusal: 'the archive rendering was disputed by the repair lane',
    laneTexts,
    ...((translateWinnerBallots === undefined) ? {} : { translateWinnerBallots, }),
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

/**
 What a case reads off a settlement.

 @param settled - the settlement

 @returns The fields the owner's rule decides

 @example
 ```ts
 const read = readSettlement({ settled, });
 ```
 */
function readSettlement(
  { settled, }: { readonly settled: ConsolidationSettlement; },
): Readonly<Record<string, unknown>> {
  /**
   The settlement as a loose record, so a field a build lacks reads as absent.
   */
  const loose = settled as unknown as Readonly<Record<string, unknown>>;
  return {
    terminal: settled.terminal,
    text: settled.text,
    archiveKept: settled.archiveKept,
    translateWinnerShipped: loose.translateWinnerShipped,
    archiveReason: loose.archiveReason,
    named: settled.findings.some(function namesIt(finding,): boolean {
      return finding.startsWith('translate-winner-ships',);
    },),
  };
}

/**
 Counts the calls a purchase makes over a refused standing with the translate text on offer, whatever the purchase
 then does about the refusals.

 @param slateReachable - slate judges a wet provider would serve

 @param translateWinnerBallots - ballots that named the translate lane's text in its own round

 @returns Calls made

 @example
 ```ts
 const made = await callsOver({ slateReachable: 2, translateWinnerBallots: 2, });
 ```
 */
async function callsOver(
  {
    slateReachable,
    translateWinnerBallots,
  }: {
    readonly slateReachable: number;
    readonly translateWinnerBallots: number;
  },
): Promise<number> {
  const asked: Asked = {
    calls: 0,
    refinerSheets: 0,
  };
  try {
    await buyConsolidationSlice({
      client: rejectingClient({ asked, },),
      roster: ROSTER,
      judgeModelIds: ROSTER,
      subject: {
        sourceText: SOURCE,
        incumbentText: ARCHIVE,
        repairText: ARCHIVE,
        translateText: TRANSLATE_TEXT,
        ballots: [],
        lineStructured: false,
      },
      standingText: ARCHIVE,
      lineStructured: false,
      sliceIndex: 7,
      standingMayShip: false,
      standingEligible: false,
      laneTexts: [TRANSLATE_LANE,],
      slateReachable,
      translateWinnerBallots,
      signal: AbortSignal.timeout(HANG_STOP_MS,),
      perCallTimeoutMs: HANG_STOP_MS,
      l,
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },);
  }
  catch (error) {
    // A purchase whose producers all refuse ends in the stage's interruption, which is not what this counts.
    expect(Error.isError(error,),).toBe(true,);
  }
  return asked.calls;
}

await describe({
  name: '',
  children: [
    describe({
      name: 'the translate lane winner ships where the final round cannot seat two judges (owner, 2026-10-29)',
      children: [
        it({
          name: 'SHIPS THE TRANSLATE TEXT over a refused standing when one slate judge is seated, runs no polish over '
            + 'it, and names the path in the findings',
          fn: async () => {
            const asked: Asked = {
              calls: 0,
              refinerSheets: 0,
            };
            const settled = await settleOver({
              judgeCount: 1,
              laneTexts: [TRANSLATE_LANE,],
              translateWinnerBallots: 2,
              standingEligible: false,
              asked,
            },);
            expect({
              ...readSettlement({ settled, },),
              refinerSheets: asked.refinerSheets,
            },).toEqual({
              terminal: 'slate-declined-standing',
              text: TRANSLATE_TEXT,
              archiveKept: undefined,
              translateWinnerShipped: true,
              archiveReason: undefined,
              named: true,
              refinerSheets: 0,
            },);
          },
        },),
        it({
          name: 'SHIPS THE TRANSLATE TEXT where no slate judge is seated at all',
          fn: async () => {
            const settled = await settleOver({
              judgeCount: 0,
              laneTexts: [TRANSLATE_LANE,],
              translateWinnerBallots: 2,
              standingEligible: false,
              asked: {
                calls: 0,
                refinerSheets: 0,
              },
            },);
            expect(readSettlement({ settled, },).translateWinnerShipped,).toBe(true,);
            expect(settled.text,).toBe(TRANSLATE_TEXT,);
          },
        },),
        it({
          name: 'KEEPS THE ARCHIVE where the translate text drew one ballot, since one judge alone never decides, '
            + 'and says the final round could not seat two judges',
          fn: async () => {
            const settled = await settleOver({
              judgeCount: 1,
              laneTexts: [TRANSLATE_LANE,],
              translateWinnerBallots: 1,
              standingEligible: false,
              asked: {
                calls: 0,
                refinerSheets: 0,
              },
            },);
            expect(readSettlement({ settled, },),).toMatchObject({
              archiveKept: true,
              translateWinnerShipped: undefined,
              archiveReason: 'final-round-unseated',
              named: false,
            },);
          },
        },),
        it({
          name: 'KEEPS THE ARCHIVE where the translate text failed the rule, so the offer holds no translate text '
            + 'however many judges chose it',
          fn: async () => {
            const settled = await settleOver({
              judgeCount: 1,
              laneTexts: [],
              translateWinnerBallots: 2,
              standingEligible: false,
              asked: {
                calls: 0,
                refinerSheets: 0,
              },
            },);
            expect(readSettlement({ settled, },),).toMatchObject({
              archiveKept: true,
              translateWinnerShipped: undefined,
              named: false,
            },);
          },
        },),
        it({
          name: 'KEEPS THE ARCHIVE as before where the slate seats two judges and they decline, and says the slate '
            + 'declined',
          fn: async () => {
            const settled = await settleOver({
              judgeCount: 2,
              laneTexts: [TRANSLATE_LANE,],
              translateWinnerBallots: 2,
              standingEligible: false,
              asked: {
                calls: 0,
                refinerSheets: 0,
              },
            },);
            expect(readSettlement({ settled, },),).toMatchObject({
              terminal: 'slate-declined-standing',
              archiveKept: true,
              translateWinnerShipped: undefined,
              archiveReason: 'slate-declined',
            },);
          },
        },),
        it({
          name: 'LEAVES AN ELIGIBLE STANDING ALONE: its slate decides as before and nothing is shipped from the lane',
          fn: async () => {
            const settled = await settleOver({
              judgeCount: 1,
              laneTexts: [TRANSLATE_LANE,],
              translateWinnerBallots: 2,
              standingEligible: true,
              asked: {
                calls: 0,
                refinerSheets: 0,
              },
            },);
            expect(readSettlement({ settled, },),).toMatchObject({
              archiveKept: undefined,
              translateWinnerShipped: undefined,
              named: false,
            },);
            expect(settled.text,).toBe(ARCHIVE,);
          },
        },),
        it({
          name: 'KEEPS THE ARCHIVE AND SAYS NO WORDING PASSED where the offer holds no lane text and the floor '
            + 'refused every proposal',
          fn: async () => {
            const settled = await settleConsolidation({
              client: rejectingClient({
                asked: {
                  calls: 0,
                  refinerSheets: 0,
                },
              },),
              roster: ROSTER,
              judgeModelIds: ROSTER.slice(0, 1,),
              subject: {
                sourceText: SOURCE,
                incumbentText: ARCHIVE,
              },
              voices: [],
              validity: [],
              producedFindings: [],
              standingText: ARCHIVE,
              lineStructured: false,
              sliceIndex: 4,
              standingMayShip: false,
              standingEligible: false,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(readSettlement({ settled, },),).toMatchObject({
              terminal: 'incumbent-only',
              archiveKept: true,
              archiveReason: 'no-wording-passed',
            },);
          },
        },),
      ],
    },),
    describe({
      name: `${buyConsolidationSlice.name} over an unseated slate with a translate winner`,
      children: [
        it({
          name: 'BUYS NOTHING when fewer slate judges are reachable than a winner needs and the translate lane holds '
            + 'a two-ballot text, ships that text, and never keeps the settlement for a later run (XingZ6022: 65 '
            + 'calls, 10.679 Wh)',
          fn: async () => {
            const asked: Asked = {
              calls: 0,
              refinerSheets: 0,
            };
            const lines: string[] = [];
            const settled = await buyConsolidationSlice({
              client: rejectingClient({ asked, },),
              roster: ROSTER,
              judgeModelIds: ROSTER,
              subject: {
                sourceText: SOURCE,
                incumbentText: ARCHIVE,
                repairText: ARCHIVE,
                translateText: TRANSLATE_TEXT,
                ballots: [],
                lineStructured: false,
              },
              standingText: ARCHIVE,
              lineStructured: false,
              sliceIndex: 7,
              standingMayShip: false,
              standingEligible: false,
              laneTexts: [TRANSLATE_LANE,],
              slateReachable: 1,
              translateWinnerBallots: 2,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l: tagged({
                tag: 'translate-winner-ships-test',
                l: capturingLogger({ messages: lines, },),
              },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect({
              calls: asked.calls,
              ...readSettlement({ settled, },),
              resumable: consolidationWorthResuming({ settlement: settled, standingMayShip: false, },),
              unendorsed: standingKeptUnendorsed({ settlement: settled, standingMayShip: false, },),
              said: lines.some(function namesIt(line,): boolean {
                return line.includes('translate lane winner',) && line.includes('ballots',);
              },),
            },).toEqual({
              calls: 0,
              terminal: 'incumbent-only',
              text: TRANSLATE_TEXT,
              archiveKept: undefined,
              translateWinnerShipped: true,
              archiveReason: undefined,
              named: true,
              resumable: false,
              unendorsed: false,
              said: true,
            },);
          },
        },),
        it({
          name: 'ASKS AS BEFORE where the translate text drew one ballot, and where two slate judges are reachable',
          fn: async () => {
            expect({
              oneBallotAsked: (await callsOver({ slateReachable: 1, translateWinnerBallots: 1, },)) > 0,
              twoJudgesAsked: (await callsOver({ slateReachable: 2, translateWinnerBallots: 2, },)) > 0,
            },).toEqual({
              oneBallotAsked: true,
              twoJudgesAsked: true,
            },);
          },
        },),
      ],
    },),
  ],
},);
