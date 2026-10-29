import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { LaneText, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { TranslateStageResult, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import {
  keepTheArchive,
  nothingValidShips,
  shipTranslateWinner,
  translateWinnerOf,
  unseatedFinalRound,
} from './consolidate-ineligible-standing.ts';
import type {
  ConsolidationSettlement,
  ConsolidationSubject,
  ConsolidationTerminal,
} from './consolidate-settle.ts';
import type { ProposalVerdict, } from './consolidate-settle-context.ts';
import type { SlateFloor, } from './consolidate-validity-floor.ts';

//region Consolidate settle, final round unseated
// WHAT A SETTLEMENT DOES WHERE THE FINAL ROUND CANNOT SEAT TWO JUDGES, split out of `consolidate-settle.ts` at its line
// cap. The owner's answer of 2026-10-29 ("Ship the two-ballot text (Recommended)"): when the final round can't seat two
// judges, the translate lane's winner ships if two judges chose it and it passes the publication rule. One judge alone
// never decides (2026-10-25), a slice with no wording that passes keeps the archive (2026-09-27, "Keep archive,
// ship"), and a disputed archive is never a candidate or a fallback (2026-09-27).

/**
 Settles an exit that bought no slate and offered no lane text: the standing stays, and where the rule refused it no
 wording passed, so the archive keeps the slice (owner, 2026-09-27, "Keep archive, ship").

 @param terminal - how the settlement leaves the stage

 @param standingEligible - whether the standing passed the deterministic gate

 @param standingText - wording in place when the stage began

 @param floor - what the validity floor made of the slate

 @param verdicts - every voice's verdict without its text

 @param producedFindings - what gathering recorded

 @param subject - slice, whose syntax role decides the polish not-run reason

 @param sliceIndex - prepared position of the slice, for the log

 @param l - stage logger

 @returns Settlement keeping the standing, marked archive-kept where the rule refused it

 @example
 ```ts
 const settlement = settleWithoutSlate({ terminal: 'incumbent-only', standingEligible, standingText, floor, verdicts, producedFindings, subject, sliceIndex, l, },);
 ```
 */
export function settleWithoutSlate(
  {
    terminal,
    standingEligible,
    standingText,
    floor,
    verdicts,
    producedFindings,
    subject,
    sliceIndex,
    l,
  }: {
    readonly terminal: ConsolidationTerminal;
    readonly standingEligible: boolean;
    readonly standingText: string;
    readonly floor: SlateFloor;
    readonly verdicts: readonly ProposalVerdict[];
    readonly producedFindings: readonly string[];
    readonly subject: ConsolidationSubject;
    readonly sliceIndex: number;
    readonly l: Logger;
  },
): ConsolidationSettlement {
  /**
   The settlement before the archive is marked.
   */
  const settlement: ConsolidationSettlement = {
    terminal,
    text: standingText,
    floor,
    verdicts,
    rewrapped: false,
    demoted: false,
    findings: producedFindings,
  };
  if (!nothingValidShips({
    standingEligible,
    terminal,
  },))
    return settlement;
  return keepTheArchive({
    settlement,
    subject,
    sliceIndex,
    reason: 'no-wording-passed',
    l,
  },);
}

/**
 What a settlement does before it buys a slate.
 */
export type BeforeSlateExit =
  | {
    /**
     The translate lane's winner ships and no slate is bought.
     */
    readonly kind: 'ships';

    /**
     The settlement that ships it.
     */
    readonly settlement: ConsolidationSettlement;
  }
    | {
    /**
     Nothing is decided yet; the slate is asked.
     */
      readonly kind: 'asks';
    };

/**
 Ships the translate lane's winner before any slate is bought, where fewer slate judges are reachable than a winner
 needs over a standing the rule refused.

 @param standingEligible - whether the standing passed the deterministic gate

 @param laneTexts - lane texts the slate offers, whose translate entry passed the rule

 @param translateWinnerBallots - ballots that named the translate lane's text in the lane's own round

 @param slateReachable - slate judges a wet provider would serve, absent where the caller did not count

 @param standingText - wording in place when the stage began, the refused standing here

 @param floor - what the validity floor made of the slate

 @param verdicts - every voice's verdict without its text

 @param producedFindings - what gathering recorded

 @param subject - slice, whose syntax role decides the polish not-run reason

 @param sliceIndex - prepared position of the slice, for the log

 @param l - stage logger

 @returns Settlement that ships the winner, or that the slate is asked

 @example
 ```ts
 const exit = shipWinnerBeforeSlate({ standingEligible: false, laneTexts, translateWinnerBallots: 2, slateReachable: 1, standingText, floor, verdicts, producedFindings, subject, sliceIndex, l, },);
 ```
 */
export function shipWinnerBeforeSlate(
  {
    standingEligible,
    laneTexts,
    translateWinnerBallots,
    slateReachable,
    standingText,
    floor,
    verdicts,
    producedFindings,
    subject,
    sliceIndex,
    l,
  }: {
    readonly standingEligible: boolean;
    readonly laneTexts: readonly LaneText[];
    readonly translateWinnerBallots: number;
    readonly slateReachable?: number;
    readonly standingText: string;
    readonly floor: SlateFloor;
    readonly verdicts: readonly ProposalVerdict[];
    readonly producedFindings: readonly string[];
    readonly subject: ConsolidationSubject;
    readonly sliceIndex: number;
    readonly l: Logger;
  },
): BeforeSlateExit {
  /**
   The translate lane's two-ballot text, where the offer holds it.
   */
  const winner = translateWinnerOf({
    laneTexts,
    ballots: translateWinnerBallots,
  },);
  if (standingEligible)
    return { kind: 'asks', };
  if (winner.kind === 'none')
    return { kind: 'asks', };
  if (slateReachable === undefined)
    return { kind: 'asks', };
  if (!unseatedFinalRound({ heard: slateReachable, },))
    return { kind: 'asks', };
  return {
    kind: 'ships',
    settlement: shipTranslateWinner({
      settlement: {
        terminal: 'incumbent-only',
        text: standingText,
        floor,
        verdicts,
        rewrapped: false,
        demoted: false,
        findings: producedFindings,
      },
      winner: winner.laneText,
      subject,
      sliceIndex,
      l,
    },),
  };
}

/**
 Settles a slate that ended on the standing the rule refused: the translate lane's winner ships where the slate round
 heard fewer than two judges and the lane holds a two-ballot text, else the archive keeps the slice with the reason.

 @param settlement - what the slate settled, whose text is the refused standing

 @param decided - what the slate judges settled

 @param laneTexts - lane texts the slate offered, whose translate entry passed the rule

 @param translateWinnerBallots - ballots that named the translate lane's text in the lane's own round

 @param subject - slice, whose syntax role decides the polish not-run reason

 @param sliceIndex - prepared position of the slice, for the log

 @param l - stage logger

 @returns Settlement shipping the winner, or marked archive-kept with its reason

 @example
 ```ts
 return settleRefusedStanding({ settlement, decided, laneTexts, translateWinnerBallots, subject, sliceIndex, l, },);
 ```
 */
export function settleRefusedStanding(
  {
    settlement,
    decided,
    laneTexts,
    translateWinnerBallots,
    subject,
    sliceIndex,
    l,
  }: {
    readonly settlement: ConsolidationSettlement;
    readonly decided: TranslateStageResult;
    readonly laneTexts: readonly LaneText[];
    readonly translateWinnerBallots: number;
    readonly subject: ConsolidationSubject;
    readonly sliceIndex: number;
    readonly l: Logger;
  },
): ConsolidationSettlement {
  /**
   The translate lane's two-ballot text, where the offer holds it.
   */
  const winner = translateWinnerOf({
    laneTexts,
    ballots: translateWinnerBallots,
  },);
  /**
   Judges the slate round heard.
   */
  const { ballots: heard, } = decided.tally;
  /**
   Whether the slate round could not seat two judges.
   */
  const unseated = unseatedFinalRound({ heard, },);
  if (unseated && (winner.kind === 'offered')) {
    return shipTranslateWinner({
      settlement,
      winner: winner.laneText,
      subject,
      sliceIndex,
      l,
    },);
  }
  return keepTheArchive({
    settlement,
    subject,
    sliceIndex,
    reason: unseated ? 'final-round-unseated' : 'slate-declined',
    l,
  },);
}

//endregion Consolidate settle, final round unseated
