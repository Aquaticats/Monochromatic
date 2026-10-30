import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { LaneText, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  tenseCandidateGroups,
  type TenseCandidateGroup,
  type TranslateStageResult,
} from '@monochromatic-dev/translation-repair-translate-stage/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  translateWinnerOf,
  unseatedFinalRound,
} from './consolidate-ineligible-standing.ts';
import type {
  ConsolidationSettlement,
  ConsolidationSubject,
} from './consolidate-settle.ts';
import { unpolishedBaseline, } from './consolidation-polish-skip.ts';

//region Consolidate tense swap
// THE ONE PLACE THE TENSE DETECTOR DECIDES WHAT SHIPS. The owner ruled on 2026-10-11 that the detector is an aid shown
// to judges and never decides (`design-commitments-tense-and-dictionary-entries.md`). XingZ6022 (2026-10-29) shipped a
// life that had ended in the present tense at six slices because the slate heard one judge and kept an eligible
// standing the contest had not endorsed, so nobody read the house tense rule against it; the translate lane held a
// text two judges had chosen at each, and the detector flagged the standing and not that text. Asked what to do, the
// owner chose "Swap only where flagged (Recommended)" (2026-10-29, B1050): where no vote can seat two judges, the
// two-ballot translate text ships instead of the standing, only where the detector flags present tense in the
// standing and does not flag the translate text. The detector decides here and nowhere else.
//
// THE SWAP STOPS AT THE FINAL ROUND. A slate that seats two or more judges decides, as it always did, and so does a
// standing the contest endorsed. A standing the rule refused is the business of `consolidate-settle-unseated.ts`.

/**
 Label the standing carries in the detector's groups.
 */
const STANDING_LABEL = 'STANDING';

/**
 Label the translate lane's text carries in the detector's groups.
 */
const TRANSLATE_LABEL = 'TRANSLATE TEXT';

/**
 Whether the translate lane's winner ships over an eligible standing the contest did not endorse, and what the detector
 made of each text.

 @example
 ```ts
 const keeps: TenseSwap = { kind: 'keeps', };
 ```
 */
export type TenseSwap =
  | {
    /**
     The detector flags the standing and not the translate lane's winner, which two or more judges chose in the lane.
     */
    readonly kind: 'swaps';

    /**
     The translate lane's text that ships.
     */
    readonly winner: LaneText;

    /**
     Present-tense candidates the detector flags in the standing, one or more.
     */
    readonly standingCandidates: number;

    /**
     Ballots that named the translate lane's text in the lane's own round.
     */
    readonly ballots: number;
  }
    | {
    /**
     The standing stays: it is endorsed, refused, unflagged, or the translate text is flagged too or is not on offer.
     */
      readonly kind: 'keeps';
    };

/**
 What a settlement that cannot seat two judges does over an eligible standing the detector may overrule.

 @example
 ```ts
 const exit: UnseatedSwapExit = { kind: 'keeps', };
 ```
 */
export type UnseatedSwapExit =
  | {
    /**
     The translate lane's winner ships.
     */
    readonly kind: 'ships';

    /**
     The settlement that ships it.
     */
    readonly settlement: ConsolidationSettlement;
  }
    | {
    /**
     Nothing is overruled here; the settlement goes on as it was.
     */
      readonly kind: 'keeps';
    };

/**
 Counts the candidates the detector flags in one text of the pair it was asked about.

 @param groups - groups the detector returned, one per text with a candidate

 @param label - label of the text counted

 @returns Candidates flagged in that text, zero where it has none

 @example
 ```ts
 const flagged = candidatesLabelled({ groups, label: 'STANDING', },);
 ```
 */
function candidatesLabelled(
  {
    groups,
    label,
  }: {
    readonly groups: readonly TenseCandidateGroup[];
    readonly label: string;
  },
): number {
  /**
   The text's group, absent where the detector found nothing in it.
   */
  const group = groups.find(function labelled(candidate,): boolean {
    return candidate.label === label;
  },);
  if (group === undefined)
    return 0;
  return group.found
    .length;
}

/**
 Reads whether the translate lane's winner ships over an eligible standing the contest did not endorse, because the
 detector flags the standing and does not flag that winner.

 @param standingEligible - whether the standing passed the deterministic gate; a refused one is settled elsewhere

 @param standingMayShip - whether the contest endorsed the standing; an endorsed one is never overruled here

 @param standingText - wording in place when the stage began

 @param laneTexts - lane texts the slate offers, whose translate entry passed the rule

 @param translateWinnerBallots - ballots that named the translate lane's text in the lane's own round

 @param subject - slice, whose recorded death and declared names the detector reads

 @returns Swap with its counts, or that the standing stays

 @example
 ```ts
 const swap = presentTenseSwapOf({ standingEligible: true, standingMayShip: false, standingText, laneTexts, translateWinnerBallots: 2, subject, },);
 ```
 */
export function presentTenseSwapOf(
  {
    standingEligible,
    standingMayShip,
    standingText,
    laneTexts,
    translateWinnerBallots,
    subject,
  }: {
    readonly standingEligible: boolean;
    readonly standingMayShip: boolean;
    readonly standingText: string;
    readonly laneTexts: readonly LaneText[];
    readonly translateWinnerBallots: number;
    readonly subject: Pick<ConsolidationSubject, 'lifeEnded' | 'subjectNames'>;
  },
): TenseSwap {
  if ((!standingEligible) || standingMayShip)
    return { kind: 'keeps', };
  // THE DETECTOR READS NOTHING WHERE THE ENTRY RECORDS NO DEATH, so a page about a living person is never swapped.
  if (subject.lifeEnded !== true)
    return { kind: 'keeps', };
  /**
   The translate lane's two-ballot text, where the offer holds it.
   */
  const winner = translateWinnerOf({
    laneTexts,
    ballots: translateWinnerBallots,
  },);
  if (winner.kind === 'none')
    return { kind: 'keeps', };
  /**
   The translate lane's text, read off the winner.
   */
  const { laneText, } = winner;
  /**
   What the detector flags in the pair, one group for each text that has a candidate.
   */
  const groups = tenseCandidateGroups({
    lifeEnded: true,
    ...((subject.subjectNames === undefined) ? {} : { subjectNames: subject.subjectNames, }),
    texts: [
      {
        label: STANDING_LABEL,
        text: standingText,
      },
      {
        label: TRANSLATE_LABEL,
        text: laneText.text,
      },
    ],
  },);
  /**
   Candidates in the standing.
   */
  const standingCandidates = candidatesLabelled({
    groups,
    label: STANDING_LABEL,
  },);
  /**
   Candidates in the translate lane's text.
   */
  const translateCandidates = candidatesLabelled({
    groups,
    label: TRANSLATE_LABEL,
  },);
  if ((standingCandidates === 0) || (translateCandidates > 0))
    return { kind: 'keeps', };
  return {
    kind: 'swaps',
    winner: laneText,
    standingCandidates,
    ballots: translateWinnerBallots,
  };
}

/**
 Ships the translate lane's winner over an eligible standing the detector flags (owner, 2026-10-29, "Swap only where
 flagged (Recommended)"), and says in the log and the findings which slice, what the detector made of each text, and
 the rule.

 THE SETTLEMENT KEEPS ITS TERMINAL, as the other translate-winner shipments do, and the mark, the reason and the
 finding say what ships. No polish runs over the winner, since no judge here heard it, and nothing of it is stored for
 a later run (`consolidationWorthResuming`).

 @param settlement - what the stage settled, before any polish

 @param swap - reading that overruled the standing

 @param subject - slice, whose syntax role decides the not-run reason the final naturalness check requires

 @param sliceIndex - prepared position of the slice, for the log

 @param l - stage logger, told which slice swapped and why

 @returns Settlement marked translate-winner-shipped for the present-tense standing, carrying the winner as its text

 @example
 ```ts
 return shipTranslateWinnerOverPresentTense({ settlement, swap, subject, sliceIndex, l, },);
 ```
 */
export function shipTranslateWinnerOverPresentTense(
  {
    settlement,
    swap,
    subject,
    sliceIndex,
    l,
  }: {
    readonly settlement: ConsolidationSettlement;
    readonly swap: Extract<TenseSwap, { readonly kind: 'swaps'; }>;
    readonly subject: Pick<ConsolidationSubject, 'syntax'>;
    readonly sliceIndex: number;
    readonly l: Logger;
  },
): ConsolidationSettlement {
  /**
   Logger tagged with this function.
   */
  const sl = tagged({
    tag: shipTranslateWinnerOverPresentTense.name,
    l,
  },);
  /**
   The translate lane's text that ships.
   */
  const { winner, } = swap;
  /**
   The standing's count of candidates, with its noun.
   */
  const standingCount = `${String(swap.standingCandidates,)} ${
    wordForCount({
      count: swap.standingCandidates,
      one: 'candidate',
      many: 'candidates',
    },)
  }`;
  /**
   The ballots that chose the winner in the lane, with their noun.
   */
  const ballotCount = `${String(swap.ballots,)} ${
    wordForCount({
      count: swap.ballots,
      one: 'ballot',
      many: 'ballots',
    },)
  }`;
  /**
   The two counts and the rule, in the words of the log line and the finding. The winner is flagged nowhere, which is
   what lets it ship.
   */
  const why = 'the final round could not seat two judges over an eligible standing the contest did not endorse; the '
    + `tense detector flags present-tense candidates (standing: ${standingCount}; translate text: 0 candidates), so `
    + `the translate lane winner, which ${ballotCount} chose in the lane, ships (owner, 2026-10-29, "Swap only where `
    + 'flagged")';
  sl.warn(`slice ${String(sliceIndex,)}: ${why}`,);
  return {
    ...settlement,
    text: winner.text,
    translateWinnerShipped: true,
    translateWinnerReason: 'present-tense-standing',
    findings: [
      ...settlement.findings,
      `translate-winner-ships-over-present-tense-standing: ${why}`,
    ],
    polish: unpolishedBaseline(subject,),
  };
}

/**
 Settles a slate that heard fewer than two judges over an eligible standing the contest did not endorse: the translate
 lane's winner ships where the detector flags the standing and not the winner, else nothing is overruled.

 @param settlement - what the slate settled, whose text is the standing

 @param decided - what the slate judges settled

 @param standingEligible - whether the standing passed the deterministic gate

 @param standingMayShip - whether the contest endorsed the standing

 @param laneTexts - lane texts the slate offered, whose translate entry passed the rule

 @param translateWinnerBallots - ballots that named the translate lane's text in the lane's own round

 @param subject - slice, whose recorded death and declared names the detector reads

 @param sliceIndex - prepared position of the slice, for the log

 @param l - stage logger

 @returns Settlement shipping the winner, or that the settlement goes on

 @example
 ```ts
 const exit = swapAfterUnseatedSlate({ settlement, decided, standingEligible, standingMayShip, laneTexts, translateWinnerBallots, subject, sliceIndex, l, },);
 ```
 */
export function swapAfterUnseatedSlate(
  {
    settlement,
    decided,
    standingEligible,
    standingMayShip,
    laneTexts,
    translateWinnerBallots,
    subject,
    sliceIndex,
    l,
  }: {
    readonly settlement: ConsolidationSettlement;
    readonly decided: Pick<TranslateStageResult, 'tally'>;
    readonly standingEligible: boolean;
    readonly standingMayShip: boolean;
    readonly laneTexts: readonly LaneText[];
    readonly translateWinnerBallots: number;
    readonly subject: Pick<ConsolidationSubject, 'lifeEnded' | 'subjectNames' | 'syntax'>;
    readonly sliceIndex: number;
    readonly l: Logger;
  },
): UnseatedSwapExit {
  /**
   Judges the slate round heard.
   */
  const { ballots: heard, } = decided.tally;
  // A SLATE THAT SEATED TWO OR MORE JUDGES DECIDED, and the vote stands whatever the detector says.
  if (!unseatedFinalRound({ heard, },))
    return { kind: 'keeps', };
  /**
   What the detector makes of the standing and the winner.
   */
  const swap = presentTenseSwapOf({
    standingEligible,
    standingMayShip,
    standingText: settlement.text,
    laneTexts,
    translateWinnerBallots,
    subject,
  },);
  if (swap.kind === 'keeps')
    return { kind: 'keeps', };
  return {
    kind: 'ships',
    settlement: shipTranslateWinnerOverPresentTense({
      settlement,
      swap,
      subject,
      sliceIndex,
      l,
    },),
  };
}

//endregion Consolidate tense swap
