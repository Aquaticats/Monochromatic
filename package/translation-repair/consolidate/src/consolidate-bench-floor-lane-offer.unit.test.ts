/**
 Guards the owner's answer of 2026-10-30 to B1053, "Skip when below floor (Recommended)": an eligible standing the
 contest did not endorse buys no writer and no slate whenever fewer slate judges are reachable than a winner needs, with
 the lane texts on the offer or not. One judge alone never decides (2026-10-25), so such a slate can only keep the
 standing. Where the swap of B1050 fires (the tense detector flags the standing and not the two-ballot translate text)
 it still swaps; the skip covers the rest.

 XingZ6022 (2026-10-29) had a consolidation slate round at 75 slices, 91 rounds in all, and every one heard 1 judge of
 7 (`select round: 1/7 heard`).

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { LaneText, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  buyConsolidationSlice,
  type ConsolidationSettlement,
  consolidationWorthResuming,
} from '../dist/final/node/index.mjs';
import {
  capturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  SCRIPTED_STAGE_SETTINGS,
} from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Invented original: the cat was a tabby with a white chin.
 */
const SOURCE = '猫猫是一只下巴雪白的虎斑猫。';

/**
 Name the translation declares for its subject, which the detector reads as a named subject beside a verb.
 */
const SUBJECT_NAMES = ['Mittens',] as const;

/**
 A standing the detector does not flag: the sentence told in the past.
 */
const PAST_STANDING = 'Mittens was a tabby with a white chin.';

/**
 A standing the detector flags: a copula in the present directly after the subject's name.
 */
const FLAGGED_STANDING = 'Mittens is a tabby with a white chin.';

/**
 The repair lane's text, which differs from the standing in wording only.
 */
const REPAIR_TEXT = 'Mittens was a tabby with a pale chin.';

/**
 The translate lane's text, which the detector does not flag.
 */
const TRANSLATE_TEXT = 'Mittens had been a tabby with a white chin.';

/**
 Writers of the cases.
 */
const WRITERS = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Slate judges seated, as many as the bench of XingZ6022 held.
 */
const JUDGES = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_VISION,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_ONLY,
] as const satisfies readonly RosterModelId[];

/**
 Both lane texts as the lane offer hands them over once they passed the rule.
 */
const BOTH_LANES: readonly LaneText[] = [
  {
    lane: 'repair',
    text: REPAIR_TEXT,
  },
  {
    lane: 'translate',
    text: TRANSLATE_TEXT,
  },
];

/**
 The line the skip writes where no lane text is on the offer, which names the slice, the 1 of 7 reachable and the floor
 of 2 a winner needs.
 */
const SKIP_LINE = '[consolidate-bench-floor-lane-offer-test] slice 7: 1 of 7 slate judges reachable, below the floor of 2; '
  + 'no slate is bought and the standing is kept';

/**
 The line the skip writes where two lane texts are on the offer.
 */
const SKIP_LINE_WITH_OFFER = `${SKIP_LINE}; 2 lane texts on the offer go unjudged`;

/**
 What a purchase did, and how many calls the client was asked.
 */
type Bought =
  | {
    /**
     The purchase ended in a settlement.
     */
    readonly kind: 'settled';

    /**
     The settlement.
     */
    readonly settled: ConsolidationSettlement;

    /**
     Calls of any kind the client was asked.
     */
    readonly calls: number;

    /**
     Lines the stages logged.
     */
    readonly lines: readonly string[];
  }
    | {
    /**
     The purchase ended in an error because every producer refused.
     */
      readonly kind: 'interrupted';

    /**
     Calls of any kind the client was asked.
     */
      readonly calls: number;

    /**
     Lines the stages logged.
     */
      readonly lines: readonly string[];
    };

/**
 Buys one slice over an eligible standing against a client that refuses every call and counts them.

 @param standing - wording in place
 @param laneTexts - lane texts on the offer
 @param translateWinnerBallots - ballots that named the translate lane's text in its own round
 @param slateReachable - slate judges a wet provider would serve, absent where the caller did not count
 @param lifeEnded - whether the entry records a death, true when absent
 @param standingMayShip - whether the contest endorsed the standing, false when absent

 @returns Settlement, calls and log lines

 @example
 ```ts
 const bought = await buyOver({ standing: PAST_STANDING, laneTexts: BOTH_LANES, translateWinnerBallots: 2, slateReachable: 1, });
 ```
 */
async function buyOver(
  {
    standing,
    laneTexts,
    translateWinnerBallots,
    slateReachable,
    lifeEnded = true,
    standingMayShip = false,
  }: {
    readonly standing: string;
    readonly laneTexts: readonly LaneText[];
    readonly translateWinnerBallots: number;
    readonly slateReachable?: number;
    readonly lifeEnded?: boolean;
    readonly standingMayShip?: boolean;
  },
): Promise<Bought> {
  /**
   Calls made so far, of any kind.
   */
  const calls = { count: 0, };
  /**
   Client that refuses every call.
   */
  const client: ModelClient = {
    chatText: async () => {
      calls.count += 1;
      throw new Error('scripted refusal',);
    },
    chatJson: async <ValueT,>(_request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      calls.count += 1;
      throw new Error('scripted refusal',);
    },
    quotas: async () => {
      throw new Error('quotas unused by the consolidation',);
    },
  };
  const lines: string[] = [];
  try {
    /**
     The settlement the purchase ended in.
     */
    const settled = await buyConsolidationSlice({
      client,
      roster: WRITERS,
      judgeModelIds: JUDGES,
      subject: {
        sourceText: SOURCE,
        incumbentText: standing,
        repairText: standing,
        translateText: TRANSLATE_TEXT,
        ballots: [],
        lineStructured: false,
        ...(lifeEnded ? { lifeEnded: true, subjectNames: [...SUBJECT_NAMES,], } : {}),
      },
      standingText: standing,
      lineStructured: false,
      sliceIndex: 7,
      standingMayShip,
      standingEligible: true,
      laneTexts,
      translateWinnerBallots,
      ...((slateReachable === undefined) ? {} : { slateReachable, }),
      signal: AbortSignal.timeout(HANG_STOP_MS,),
      perCallTimeoutMs: HANG_STOP_MS,
      l: tagged({ tag: 'consolidate-bench-floor-lane-offer-test', l: capturingLogger({ messages: lines, },), },),
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },);
    return {
      kind: 'settled',
      settled,
      calls: calls.count,
      lines,
    };
  }
  catch (error) {
    // A purchase whose producers all refuse ends in the stage's interruption, which is what buying looks like here.
    expect(Error.isError(error,),).toBe(true,);
    return {
      kind: 'interrupted',
      calls: calls.count,
      lines,
    };
  }
}

/**
 What a case reads off a purchase.

 @param bought - what the purchase did

 @returns Fields the owner's rule decides

 @example
 ```ts
 expect(readBought({ bought, },),).toEqual({ calls: 0, });
 ```
 */
function readBought(
  { bought, }: { readonly bought: Bought; },
): Readonly<Record<string, unknown>> {
  if (bought.kind === 'interrupted') {
    return {
      calls: bought.calls,
      ended: 'an error',
    };
  }
  /**
   The settlement the purchase ended in.
   */
  const { settled, } = bought;
  return {
    calls: bought.calls,
    ended: 'a settlement',
    terminal: settled.terminal,
    text: settled.text,
    benchBelowFloor: settled.benchBelowFloor,
    archiveKept: settled.archiveKept,
    translateWinnerShipped: settled.translateWinnerShipped,
    translateWinnerReason: settled.translateWinnerReason,
    resumable: consolidationWorthResuming({ settlement: settled, standingMayShip: false, },),
    floorFinding: settled.findings.includes(
      'stage-bench-below-floor (consolidate-slate reachable 1 of 7, floor 2, no call made)',
    ),
  };
}

/**
 What a purchase that skipped the writers and the slate and kept the standing reads as.

 @param standing - wording that stays

 @returns Fields of a marked, unstored settlement that kept the standing and asked nothing

 @example
 ```ts
 expect(readBought({ bought, },),).toEqual(skippedKeepingStanding({ standing: PAST_STANDING, },),);
 ```
 */
function skippedKeepingStanding({ standing, }: { readonly standing: string; },): Readonly<Record<string, unknown>> {
  return {
    calls: 0,
    ended: 'a settlement',
    terminal: 'incumbent-only',
    text: standing,
    benchBelowFloor: true,
    archiveKept: undefined,
    translateWinnerShipped: undefined,
    translateWinnerReason: undefined,
    resumable: false,
    floorFinding: true,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: `${buyConsolidationSlice.name} over an unendorsed eligible standing with lane texts on the offer`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'BUYS NO WRITER AND NO SLATE with 1 of 7 slate judges reachable and both lane texts on the offer '
            + '(the translate text two-ballot, the detector flagging neither text), keeps the standing, marks the '
            + 'settlement and does not keep it for a later run (B1053: 75 slices of XingZ6022)',
          fn: async () => {
            const bought = await buyOver({
              standing: PAST_STANDING,
              laneTexts: BOTH_LANES,
              translateWinnerBallots: 2,
              slateReachable: 1,
            },);
            expect(readBought({ bought, },),).toEqual(skippedKeepingStanding({ standing: PAST_STANDING, },),);
          },
        },),
        it({
          name: 'BUYS NO WRITER AND NO SLATE with 1 of 7 slate judges reachable and a repair text alone on the '
            + 'offer, the translate lane having drawn no ballot',
          fn: async () => {
            const bought = await buyOver({
              standing: PAST_STANDING,
              laneTexts: [BOTH_LANES[0] as LaneText,],
              translateWinnerBallots: 0,
              slateReachable: 1,
            },);
            expect(readBought({ bought, },),).toEqual(skippedKeepingStanding({ standing: PAST_STANDING, },),);
          },
        },),
        it({
          name: 'SAYS THE SLICE, THE REACHABLE COUNT AND THE NEEDED COUNT in the one skip line, and how many lane '
            + 'texts on the offer go unjudged',
          fn: async () => {
            const bought = await buyOver({
              standing: PAST_STANDING,
              laneTexts: BOTH_LANES,
              translateWinnerBallots: 2,
              slateReachable: 1,
            },);
            expect(bought.lines.filter(function namesTheFloor(line,): boolean {
              return line.includes('below the floor',);
            },),).toEqual([SKIP_LINE_WITH_OFFER,],);
          },
        },),
        it({
          name: 'STILL SWAPS where the tense detector flags the standing and not the two-ballot translate text, '
            + 'buying nothing and keeping nothing for a later run (B1050)',
          fn: async () => {
            const bought = await buyOver({
              standing: FLAGGED_STANDING,
              laneTexts: BOTH_LANES,
              translateWinnerBallots: 2,
              slateReachable: 1,
            },);
            expect(readBought({ bought, },),).toEqual({
              calls: 0,
              ended: 'a settlement',
              terminal: 'incumbent-only',
              text: TRANSLATE_TEXT,
              benchBelowFloor: true,
              archiveKept: undefined,
              translateWinnerShipped: true,
              translateWinnerReason: 'present-tense-standing',
              resumable: false,
              floorFinding: true,
            },);
          },
        },),
        it({
          name: 'STILL BUYS once the slate bench meets the floor, and when the caller did not count, with the same '
            + 'offer and the same unendorsed standing',
          fn: async () => {
            expect({
              metAsked: (await buyOver({
                standing: PAST_STANDING,
                laneTexts: BOTH_LANES,
                translateWinnerBallots: 2,
                slateReachable: 2,
              },)).calls > 0,
              uncountedAsked: (await buyOver({
                standing: PAST_STANDING,
                laneTexts: BOTH_LANES,
                translateWinnerBallots: 2,
              },)).calls > 0,
            },).toEqual({
              metAsked: true,
              uncountedAsked: true,
            },);
          },
        },),
        it({
          name: 'KEEPS THE EMPTY-OFFER SKIP AS IT WAS: no lane text on the offer, 1 of 7 reachable, nothing bought, '
            + 'and the line without the clause about lane texts',
          fn: async () => {
            const bought = await buyOver({
              standing: PAST_STANDING,
              laneTexts: [],
              translateWinnerBallots: 0,
              slateReachable: 1,
            },);
            expect(readBought({ bought, },),).toEqual(skippedKeepingStanding({ standing: PAST_STANDING, },),);
            expect(bought.lines.filter(function namesTheFloor(line,): boolean {
              return line.includes('below the floor',);
            },),).toEqual([SKIP_LINE,],);
          },
        },),
      ],
    },),
  ],
},);
