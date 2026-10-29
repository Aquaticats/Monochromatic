import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { LaneText, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { TranslateStageResult, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';
import { nothingValidShips, } from './consolidate-ineligible-standing.ts';
import type {
  ConsolidationSettlement,
  ConsolidationSubject,
  ConsolidationTerminal,
} from './consolidate-settle.ts';
import type { ProposalVerdict, } from './consolidate-settle-context.ts';
import { settleRefusedStanding, } from './consolidate-settle-unseated.ts';
import { swapAfterUnseatedSlate, } from './consolidate-tense-swap.ts';
import type { SlateFloor, } from './consolidate-validity-floor.ts';
import { applyFinalPolish, } from './consolidation-polish-apply.ts';
import type { ConsolidationPolishConfig, } from './consolidation-polish.ts';

//region Consolidate settle, standing kept after the slate
// WHAT A SETTLEMENT DOES WHEN THE SLATE JUDGES LEFT THE STANDING IN PLACE, split out of `consolidate-settle.ts` at its
// line cap. A standing the rule refused gives way to the translate lane's two-ballot text where the slate heard fewer
// than two judges, else the archive keeps the slice (owner, 2026-09-27 and 2026-10-29). An eligible standing the
// contest did not endorse gives way to that text where the slate heard fewer than two judges and the tense detector
// flags the standing and not the text (owner, 2026-10-29, "Swap only where flagged"). Anything else goes to the final
// polish, which runs only over a standing the contest endorsed.

/**
 Settles a slate whose judges kept the standing text, by the owner's rulings for a final round that cannot seat two
 judges and then by the final polish.

 @param client - provider client the polish borrows

 @param decided - what the slate judges settled

 @param terminal - how the slate ended

 @param standingText - wording in place when the stage began

 @param floor - what the validity floor made of the slate

 @param verdicts - every voice's verdict without its text

 @param standingEligible - whether the standing passed the deterministic gate

 @param standingMayShip - whether the contest endorsed the standing

 @param laneTexts - lane texts the slate offered, whose translate entry passed the rule

 @param translateWinnerBallots - ballots that named the translate lane's text in the lane's own round

 @param subject - slice in the archive's terms

 @param lineStructured - whether structural rule forbids merged lines

 @param sliceIndex - prepared position of the slice, for the log

 @param polishConfig - final body polish roles and guard facts

 @param signal - cancellation for the whole settlement

 @param perCallTimeoutMs - bound on any single exchange

 @param l - stage logger

 @param stageSettings - straggler windows and contest ledger the entry file read for this launch

 @returns Settlement keeping the standing, shipping the translate lane's winner, or keeping the archive with its reason

 @throws Whatever the polish throws

 @example
 ```ts
 return await settleKeptStanding({ client, decided, terminal, standingText, floor, verdicts, standingEligible, standingMayShip, laneTexts, translateWinnerBallots, subject, lineStructured, sliceIndex, signal, perCallTimeoutMs, l, stageSettings, },);
 ```
 */
export async function settleKeptStanding(
  {
    client,
    decided,
    terminal,
    standingText,
    floor,
    verdicts,
    standingEligible,
    standingMayShip,
    laneTexts,
    translateWinnerBallots,
    subject,
    lineStructured,
    sliceIndex,
    polishConfig,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly decided: TranslateStageResult;
    readonly terminal: ConsolidationTerminal;
    readonly standingText: string;
    readonly floor: SlateFloor;
    readonly verdicts: readonly ProposalVerdict[];
    readonly standingEligible: boolean;
    readonly standingMayShip: boolean;
    readonly laneTexts: readonly LaneText[];
    readonly translateWinnerBallots: number;
    readonly subject: ConsolidationSubject;
    readonly lineStructured: boolean;
    readonly sliceIndex: number;
    readonly polishConfig?: ConsolidationPolishConfig;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ConsolidationSettlement> {
  /**
   Logger tagged with this function.
   */
  const sl = tagged({
    tag: settleKeptStanding.name,
    l,
  },);
  /**
   The settlement this exit ends in.
   */
  const settlement: ConsolidationSettlement = {
    terminal,
    text: standingText,
    floor,
    verdicts,
    decided,
    rewrapped: false,
    demoted: false,
    findings: decided.findings,
  };
  if (nothingValidShips({
    standingEligible,
    terminal: settlement.terminal,
  },)) {
    return settleRefusedStanding({
      settlement,
      decided,
      laneTexts,
      translateWinnerBallots,
      subject,
      sliceIndex,
      l: sl,
    },);
  }
  /**
   Whether the translate lane's winner ships over the standing the slate kept.
   */
  const swapped = swapAfterUnseatedSlate({
    settlement,
    decided,
    standingEligible,
    standingMayShip,
    laneTexts,
    translateWinnerBallots,
    subject,
    sliceIndex,
    l: sl,
  },);
  if (swapped.kind === 'ships')
    return swapped.settlement;
  return await applyFinalPolish({
    client,
    settlement,
    subject,
    lineStructured,
    sliceIndex,
    ...((polishConfig === undefined) ? {} : { polishConfig, }),
    eligible: standingMayShip,
    signal,
    perCallTimeoutMs,
    l: sl,
    stageSettings,
  },);
}

//endregion Consolidate settle, standing kept after the slate
