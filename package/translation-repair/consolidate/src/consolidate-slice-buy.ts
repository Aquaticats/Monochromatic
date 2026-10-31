import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { ConsolidationPolishConfig, } from './consolidation-polish.ts';
import { produceConsolidations, } from './consolidate-produce.ts';
import {
  translateWinnerOf,
  unseatedFinalRound,
} from './consolidate-ineligible-standing.ts';
import {
  type ConsolidationSettlement,
  type ConsolidationSubject,
  settleConsolidation,
} from './consolidate-settle.ts';
import { presentTenseSwapOf, } from './consolidate-tense-swap.ts';
import type { ConsolidateSubject, } from './consolidate-wire.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  floorReach,
  type LaneText,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  MIN_SELECTION_BALLOTS,
  TranslationRepairInterruptedError,
  unflooredFinding,
} from '@monochromatic-dev/translation-repair-translate-stage/ts';
import {
  benchBelowFloorStageFinding,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Consolidate slice buy

/**
 Inputs shared by initial and continued consolidation attempts.
 */
type ConsolidationBuyInput = {
  readonly client: ModelClient;
  readonly roster: readonly RosterModelId[];
  readonly judgeModelIds?: readonly RosterModelId[];
  readonly subject: ConsolidateSubject & ConsolidationSubject;
  readonly standingText: string;
  readonly lineStructured: boolean;
  readonly sliceIndex: number;
  readonly polishConfig?: ConsolidationPolishConfig;
  readonly standingMayShip: boolean;
  readonly standingEligible: boolean;
  /**
   Why the deterministic gate refused the standing, for the gate sheet
   (class fifty-six, 2026-09-18).
   */
  readonly standingRefusal?: string;
  readonly standingFindings?: readonly string[];
  /**
   Lane texts the slate offers beside the proposals (class forty,
   2026-09-17).
   */
  readonly laneTexts?: readonly LaneText[];
  /**
   Whether a strict majority of the contest ballots called the archive flawed,
   so a tied slate over an eligible standing is run off (class one hundred six)
   and an undecided gate over it ships the slate's choice (class one hundred
   seventy-seven); both widened to the majority on 2026-10-11.
   */
  readonly runoffOverStanding?: boolean;
  /**
   Slate judges a wet provider would serve right now, absent where the caller
   did not count; fewer than the ballots a winner needs buy no slate for an
   eligible standing that has no lane text on the offer or that the contest did
   not endorse (owner, 2026-10-30, "Skip when below floor (Recommended)").
   */
  readonly slateReachable?: number;
  /**
   Ballots that named the translate lane's text in the lane's own round, absent where the caller did not count. With
   two or more, and fewer slate judges reachable than a winner needs, that text ships over a refused standing and no
   slate is bought (owner, 2026-10-29, "Ship the two-ballot text").
   */
  readonly translateWinnerBallots?: number;
  readonly signal: AbortSignal;
  readonly perCallTimeoutMs: number;
  readonly l: Logger;
  readonly stageSettings: StageSettings;
};
// Fresh model work for one consolidation question. Resume, twin reuse, cache
// eligibility and ordered document aggregation remain driver responsibilities.

/**
 Whether the tense detector overrules an eligible standing the contest did not endorse in favour of the translate
 lane's two-ballot text, which reads the texts and so is asked only where the answer is used.

 @param input - what the swap reads, as `presentTenseSwapOf` takes it

 @returns Whether the translate lane's text would ship over the standing

 @example
 ```ts
 const fires = tenseSwapFires({ standingEligible: true, standingMayShip: false, standingText, laneTexts, translateWinnerBallots: 2, subject, },);
 ```
 */
function tenseSwapFires(input: Parameters<typeof presentTenseSwapOf>[0],): boolean {
  /**
   What the detector made of the standing and the translate text.
   */
  const { kind, } = presentTenseSwapOf(input,);
  return kind === 'swaps';
}

/**
 Buys and settles one third-rendering slate, or settles without asking
 producers where there is no standing text or the floor can compare nothing
 (ledger B45).

 @param client - provider client borrowed by every round

 @param roster - voices producing, and judging and gating when no narrower
 judge roster is given

 @param judgeModelIds - voices judging the slate and gating the winner; the
 producers' roster by default

 @param subject - slice and both lane candidates as every round sees them

 @param standingText - wording this consolidation must beat

 @param lineStructured - whether structural rule forbids merged lines

 @param sliceIndex - index used by logs and final polish records

 @param polishConfig - final body polish roles and guard facts

 @param standingMayShip - whether unchanged baseline has prior endorsement

 @param standingFindings - what reading the standing recorded, the
 incumbent's replacement of an ineligible standing among them

 @param standingEligible - whether the baseline passed the deterministic
 publication gate; a baseline that did not is withheld from the slate

 @param standingRefusal - why the deterministic gate refused the baseline,
 shown to the gate judges

 @param slateReachable - slate judges a wet provider would serve, which an
 eligible standing keeps itself on, unasked, when they are fewer than the
 ballots a winner needs and the offer holds no lane text or the contest did not endorse the standing (XingZ6020: 54
 slices, 164 calls; XingZ6022: 75 slices with lane texts on the offer)

 @param translateWinnerBallots - ballots that named the translate lane's text in the lane's own round; two or more,
 beside fewer reachable slate judges than a winner needs, ship that text over a refused standing and buy no slate
 (XingZ6022: 65 calls and 10.679 Wh at sixteen slices that could not decide)

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - driver logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Complete settlement for this question

 @throws Whatever producer, judging, gate, or caller abort throws

 @example
 ```ts
 const settlement = await buyConsolidationAttempt({
   client,
   roster,
   subject,
   standingText,
   lineStructured,
   sliceIndex,
   signal,
   perCallTimeoutMs,
   l,
   stageSettings,
 },);
 ```

 @internal
 */
async function buyConsolidationAttempt(
  {
    client,
    roster,
    judgeModelIds = roster,
    subject,
    standingText,
    lineStructured,
    sliceIndex,
    polishConfig,
    standingMayShip,
    standingEligible,
    standingRefusal,
    standingFindings = [],
    laneTexts = [],
    runoffOverStanding = false,
    slateReachable,
    translateWinnerBallots = 0,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<ConsolidationBuyInput>,
): Promise<ConsolidationSettlement> {
  /**
   What every settlement of this question is given whoever wrote its slate,
   so a settlement with no writer asked keeps the terminal, floor and
   findings a written one would have. ONE SET for all three exits, built
   on every call: a second copy for the exits that ask no writer left its
   optional arms unrun there (ledger B45).
   */
  const question = {
    client,
    roster,
    judgeModelIds,
    subject,
    standingText,
    lineStructured,
    sliceIndex,
    ...((polishConfig === undefined) ? {} : { polishConfig, }),
    standingMayShip,
    standingEligible,
    ...((standingRefusal === undefined) ? {} : { standingRefusal, }),
    laneTexts,
    runoffOverStanding,
    translateWinnerBallots,
    ...((slateReachable === undefined) ? {} : { slateReachable, }),
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  };

  /**
   Whether the floor can pass anything a writer here could write, read as it
   reads every proposal (`consolidate-produce.ts`): against the original and
   the archive's own page.
   */
  const reach = floorReach({
    sourceText: subject.sourceText,
    pageText: subject.incumbentText,
    ...((subject.syntax === undefined) ? {} : { syntax: subject.syntax, }),
  },);
  // NOTHING IS BOUGHT WHERE NOTHING BOUGHT COULD PASS (ledger B45): the
  // settlement refuses the standing and every lane text on the same reading
  // and keeps the archive, so writers asked here were paid for proposals the
  // floor would refuse too, as the translate stage's were before ledger B43.
  if (reach.kind === 'blind') {
    l.warn(
      `slice ${String(sliceIndex,)}: the floor can compare nothing here (${reach.detail}), `
        + 'so no writer is asked',
    );
    return await settleConsolidation({
      ...question,
      voices: [],
      validity: [],
      producedFindings: [
        ...standingFindings,
        unflooredFinding({
          stage: 'consolidate',
          detail: reach.detail,
        },),
      ],
    },);
  }

  // NO STANDING TEXT BUYS NO SLATE. The producers write from the standing
  // text, so with none they are not asked; a lane text the rule admits still
  // reaches the slate judges through the settlement (class eighty-seven,
  // XingZ623 slice 89, 2026-09-22).
  if (standingText === '') {
    l.info((laneTexts.length === 0)
      ? `slice ${String(sliceIndex,)}: no standing text to consolidate against, so no slate is bought`
      : `slice ${String(sliceIndex,)}: no standing text to consolidate against; the lane texts alone go to the slate judges`,);
    return await settleConsolidation({
      ...question,
      voices: [],
      validity: [],
      producedFindings: standingFindings,
    },);
  }

  // A SLATE BENCH BELOW THE FLOOR BUYS NO SLATE OVER A REFUSED STANDING WHEN THE TRANSLATE LANE HOLDS A TWO-BALLOT TEXT.
  // Fewer reachable judges than the ballots a winner needs can only name a winner alone, which the selection declines
  // (owner, 2026-10-25, "one judge alone never decides"), so the writers and the judges would be paid for a slate that
  // cannot decide. The text two judges chose in the lane and the rule admitted ships instead (owner, 2026-10-29,
  // "Ship the two-ballot text"; XingZ6022: 65 calls and 10.679 Wh at sixteen slices). A refused standing with no such
  // text is still asked, since its settlement says whether the final round seated two judges after all. Never cached.
  //
  // SO DOES AN ELIGIBLE STANDING THE CONTEST DID NOT ENDORSE where the tense detector flags it and does not flag the
  // translate text (owner, 2026-10-29, "Swap only where flagged"): the slate could not overrule that swap either. The
  // detector is read only once the bench is short and the winner is offered, because nothing else uses its answer.
  /**
   Whether fewer slate judges are reachable than the ballots a winner needs, so no slate this purchase could buy would
   seat two judges.
   */
  const benchShort = (slateReachable !== undefined) && unseatedFinalRound({ heard: slateReachable, },);
  /**
   The translate lane's two-ballot text, where the offer holds it.
   */
  const translateWinner = translateWinnerOf({
    laneTexts,
    ballots: translateWinnerBallots,
  },);
  /**
   Whether the translate winner ships here with no slate bought: over a refused standing, or over an unendorsed one the
   tense detector overrules, read as the settlement reads it.
   */
  const winnerShipsUnasked = benchShort
    && (translateWinner.kind === 'offered')
    && ((!standingEligible) || tenseSwapFires({
      standingEligible,
      standingMayShip,
      standingText,
      laneTexts,
      translateWinnerBallots,
      subject,
    },));
  if (winnerShipsUnasked) {
    /**
     The translate winner shipped on the incumbent-only exit, before the mark.
     */
    const shipped = await settleConsolidation({
      ...question,
      voices: [],
      validity: [],
      producedFindings: [
        ...standingFindings,
        benchBelowFloorStageFinding({
          stage: 'consolidate-slate',
          reachable: slateReachable,
          benchSize: judgeModelIds.length,
          floor: MIN_SELECTION_BALLOTS,
        },),
      ],
    },);
    return {
      ...shipped,
      benchBelowFloor: true,
    };
  }

  // A SLATE BENCH BELOW THE FLOOR BUYS NO SLATE FOR AN ELIGIBLE STANDING. Fewer reachable judges than the ballots a
  // winner needs can only name a winner alone, which the selection declines, so the writers and the judges would be
  // paid for a slate that settles on the standing it started from (XingZ6020: 54 slices, 164 calls). The settlement
  // is marked and never cached, so a later pass asks the slice again, on whatever bench it then has, only while the
  // entry is unsettled (a pass selects no entry that carries an artifact or a decline record).
  //
  // THAT HOLDS WITH LANE TEXTS ON THE OFFER WHEN THE CONTEST DID NOT ENDORSE THE STANDING (owner, 2026-10-30, "Skip when
  // below floor (Recommended)", B1053): XingZ6022 bought a slate at 75 slices, every round hearing 1 judge of 7, because
  // the skip waited for an empty offer. The swap above has already taken the slices it fires on. An endorsed standing
  // with lane texts on the offer is still asked, since the offer is empty for one in practice and nothing was ruled
  // about it.
  /**
   Whether the slate bench reads below the floor for a standing that may be kept unasked.
   */
  const benchBelowFloor = benchShort
    && standingEligible
    && ((laneTexts.length === 0) || (!standingMayShip));
  if (benchBelowFloor) {
    /**
     The noun for the slate judges seated, which the line counts.
     */
    const judgeWord = wordForCount({
      count: judgeModelIds.length,
      one: 'judge',
      many: 'judges',
    },);
    /**
     The clause saying how many lane texts on the offer go to no judge, absent where the offer is empty.
     */
    const offerClause = (laneTexts.length === 0)
      ? []
      : [
        `${String(laneTexts.length,)} ${
          wordForCount({
            count: laneTexts.length,
            one: 'lane text',
            many: 'lane texts',
          },)
        } on the offer go unjudged`,
      ];
    /**
     The line saying why no slate is bought.
     */
    const skipped = [
      `slice ${String(sliceIndex,)}: ${String(slateReachable,)} of ${String(judgeModelIds.length,)} slate `
        + `${judgeWord} reachable, below the floor `
        + `of ${String(MIN_SELECTION_BALLOTS,)}`,
      'no slate is bought and the standing is kept',
      ...offerClause,
    ].join('; ',);
    l.warn(skipped,);
    /**
     The standing kept on the incumbent-only exit, before the mark. The lane texts are withheld from the settlement
     because no judge is reachable to hear them, and a settlement that holds some goes on to ask the slate judges.
     */
    const kept = await settleConsolidation({
      ...question,
      laneTexts: [],
      voices: [],
      validity: [],
      producedFindings: [
        ...standingFindings,
        benchBelowFloorStageFinding({
          stage: 'consolidate-slate',
          reachable: slateReachable,
          benchSize: judgeModelIds.length,
          floor: MIN_SELECTION_BALLOTS,
        },),
      ],
    },);
    return {
      ...kept,
      benchBelowFloor: true,
    };
  }

  /**
   Slate produced once and judged once for this question.
   */
  const produced = await produceConsolidations({
    client,
    roster,
    subject,
    standingText,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);
  // SILENT PRODUCERS ARE THE HOUR, not the passage, lane texts or none: a
  // slate the judges would be as silent about must not settle as unjudged
  // and persist (`PAUSES provider-silent unsafe standing` in the driver
  // tests), so the lane texts of class forty change nothing here.
  if ((!standingMayShip) && (produced.voices
    .length
    === 0)) {
    throw new TranslationRepairInterruptedError({
      reason: 'provider-unavailable',
      findings: produced.findings,
    },);
  }

  return await settleConsolidation({
    ...question,
    voices: produced.voices,
    validity: produced.validity,
    // THE VERDICT'S FINDINGS RIDE WITH THE PRODUCERS', so the judges read
    // them on the slate and the settlement carries them once.
    producedFindings: [
      ...standingFindings,
      ...produced.findings,
    ],
  },);
}

/**
 Whether the single attempt kept a standing the lane contest never
 endorsed, which then ships with the non-endorsement recorded as a finding.

 Until 2026-09-29 this was `consolidationNeedsRecovery`, which read as
 asking for another strategy; the stage-local recovery it once started
 ended with the single attempt (1ba8f713a), and its evidence builder went
 on 2026-09-29 (ledger B30).

 @param settlement - what the attempt settled

 @param standingMayShip - whether the contest endorsed the standing

 @returns True when the standing lacks endorsement and the attempt did not
 consolidate

 @example
 ```ts
 const unendorsed = standingKeptUnendorsed({ settlement, standingMayShip: false, },);
 ```
 */
export function standingKeptUnendorsed(
  {
    settlement,
    standingMayShip,
  }: {
    readonly settlement: ConsolidationSettlement;
    readonly standingMayShip: boolean;
  },
): boolean {
  // A SHIPPED TRANSLATE WINNER KEPT NO STANDING: its text replaced the refused one, so no non-endorsement ships.
  return (!standingMayShip)
    && (settlement.terminal !== 'consolidated')
    && (settlement.translateWinnerShipped !== true);
}

/**
 Buys one consolidation and settles it in a single attempt.

 SINGLE ATTEMPT BY DESIGN: when the standing baseline lacks contest
 endorsement and the judged round still keeps it, the settlement returns
 as it is with the non-endorsement recorded as a finding, because the
 standing text is the only wording the deterministic gate has passed and
 quality machinery may not withhold the entry over it
 (doc/planning/translation-repair-no-loop-design.md). Zero produced voices
 under a barred standing remain the bounded provider error inside the
 attempt.

 @param input - stage clients, candidates, standing policy, and operation bounds

 @returns Complete settlement for this question

 @throws {@link TranslationRepairInterruptedError} when a barred standing hears no producer voice

 @example
 ```ts
 const settlement = await buyConsolidationSlice(input);
 ```
 */
export async function buyConsolidationSlice(
  input: ForeignBorrowed<ConsolidationBuyInput>,
): Promise<ConsolidationSettlement> {
  /**
   Whether archive or lane standing may already ship.
   */
  const { standingMayShip, } = input;
  /**
   Settlement from the single attempt.
   */
  const settlement = await buyConsolidationAttempt(input,);
  if (!standingKeptUnendorsed({
    settlement,
    standingMayShip,
  }))
    return settlement;
  input.l
    .warn(
    `slice ${String(input.sliceIndex,)}: standing lacks contest endorsement and the single `
      + `consolidation attempt kept it (${settlement.terminal}); shipping with the finding recorded`,
    );
  return {
    ...settlement,
    findings: [
      ...settlement.findings,
      `consolidation-standing-unendorsed (terminal ${settlement.terminal}):`
        + ' the standing text ships with contest non-endorsement recorded as evidence',
    ],
  };
}

//endregion Consolidate slice buy
