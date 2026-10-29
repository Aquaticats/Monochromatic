import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { gateConsolidatedSlice, } from './consolidate-gate-stage.ts';
import type { GateBallot, } from './consolidate-gate-wire.ts';
import {
  keepTheArchive,
  nothingValidShips,
  shipPastForfeitStanding,
  slatePickBallotsOf,
  unseatedFinalRound,
} from './consolidate-ineligible-standing.ts';
import type {
  ConsolidationSettlement,
  ConsolidationSubject,
  ConsolidationTerminal,
} from './consolidate-settle.ts';
import type {
  ProposalVerdict,
  SettlementIdentity,
} from './consolidate-settle-context.ts';
import type { SlateFloor, } from './consolidate-validity-floor.ts';
import { wrapConsolidation, } from './consolidate-wrap.ts';
import { applyFinalPolish, } from './consolidation-polish-apply.ts';
import type { ConsolidationPolishConfig, } from './consolidation-polish.ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { TranslateStageResult, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Consolidate settle gate
// THE SETTLEMENT'S LAST TWO STEPS, gate what won the slate and wrap what
// ships, split out of `consolidate-settle.ts` at the line cap on 2026-09-04.
// The order and the reasons are that file's; this one only carries them out.

/**
 What the gate held against the consolidation: the reason of every ballot
 that did not choose it or named it unsupported or dropping content, once
 each (owner, 2026-09-27, the fourteenth addendum of
 `doc/decision/translation-repair-ineligible-standing.md`).

 @param ballots - every usable gate ballot

 @returns Objecting reasons in ballot order, empty when no ballot objected

 @example
 ```ts
 gateObjectionsOf({ ballots, },);
 ```
 */
export function gateObjectionsOf(
  { ballots, }: { readonly ballots: readonly GateBallot[]; },
): readonly string[] {
  /**
   Reasons of the ballots that objected, trimmed.
   */
  const reasons = ballots
    .filter(function objects(ballot,): boolean {
      /**
       What the ballot chose and what it named.
       */
      const {
        choice,
        unsupported,
        dropped,
      } = ballot;
      return (choice !== 'consolidated')
        || unsupported.includes('consolidated',)
        || dropped.includes('consolidated',);
    },)
    .map(function reasonOf(ballot,): string {
      /**
       Why the ballot objected.
       */
      const { reason, } = ballot;
      return reason.trim();
    },)
    // A reason showing a reader nothing states nothing, invisible characters
    // `trim()` keeps among it (ledger B40).
    .filter(function stated(reason,): boolean {
      return !rendersAsNothing({ text: reason, },);
    },);
  return [...new Set(reasons,),];
}

/**
 Gates the consolidation the judges chose, wraps what ships, and polishes it.

 @param client - provider client the gate borrows

 @param judgeModelIds - voices seated for the gate

 @param subject - slice in the archive's terms

 @param decided - what the slate judges settled, a fresh consolidation

 @param standingText - wording the consolidation has to beat

 @param lineStructured - whether structural rule forbids merged lines

 @param floor - what the validity floor made of the slate

 @param verdicts - every voice's verdict without its text

 @param sliceIndex - prepared position used by records and refusals

 @param polishConfig - final body polish roles and document guard facts

 @param standingMayShip - whether unchanged baseline has prior endorsement

 @param standingEligible - whether the standing passed the deterministic
 gate; over an ineligible standing a gate that heard two or more answering
 judges ships the proposal the slate chose, whether it preferred the standing
 or settled on neither, recorded as a finding (classes fifty-four and one
 hundred eighty-five), and a gate that heard fewer ships it only when two or
 more slate ballots chose it (owner, 2026-10-25, "Ship the slate's wording")

 @param standingRefusal - why the deterministic gate refused the standing,
 shown to the gate judges so keeping it is not taken for the safe choice

 @param standingFlawedByMajority - whether a strict majority of the contest
 ballots called the standing flawed, so an undecided gate ships the slate's
 choice over it (class one hundred seventy-seven; every ballot until the
 owner's answer of 2026-10-11, "Majority, like the run-off")

 @param identity - front matter identity as the gate takes it

 @param signal - cancellation for the whole settlement

 @param perCallTimeoutMs - bound on any single exchange

 @param l - stage logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns What ships, and every round that decided it; a gate keeping a
 standing the deterministic rule refused leaves the archive kept (owner,
 2026-09-27, "Keep archive, ship")

 @example
 ```ts
 const settled = await gateAndShip({ client, judgeModelIds, subject, decided, standingText, lineStructured, floor, verdicts, sliceIndex, standingMayShip, standingEligible, identity, signal, perCallTimeoutMs, l, stageSettings, },);
 ```
 */
export async function gateAndShip(
  {
    client,
    judgeModelIds,
    subject,
    decided,
    standingText,
    lineStructured,
    floor,
    verdicts,
    sliceIndex,
    polishConfig,
    standingMayShip,
    standingEligible,
    standingRefusal,
    standingFlawedByMajority = false,
    identity,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly judgeModelIds: readonly RosterModelId[];
    readonly subject: ConsolidationSubject;
    readonly decided: TranslateStageResult;
    readonly standingText: string;
    readonly lineStructured: boolean;
    readonly floor: SlateFloor;
    readonly verdicts: readonly ProposalVerdict[];
    readonly sliceIndex: number;
    readonly polishConfig?: ConsolidationPolishConfig;
    readonly standingMayShip: boolean;
    readonly standingEligible: boolean;
    readonly standingRefusal?: string;
    readonly standingFlawedByMajority?: boolean;
    readonly identity: SettlementIdentity;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ConsolidationSettlement> {
  /**
   What the gate made of the consolidation that won the slate.
   */
  const gate = await gateConsolidatedSlice({
    client,
    modelIds: judgeModelIds,
    subject: {
      sourceText: subject.sourceText,
      incumbentText: subject.incumbentText,
      consolidatedText: decided.text,
      standingText,
      lineStructured,
      ...((subject.syntax === undefined) ? {} : { syntax: subject.syntax, }),
      ...((standingRefusal === undefined) ? {} : { standingRefusal, }),
      ...identity,
    },
    signal,
    exchangeTimeoutMs: perCallTimeoutMs,
    l,
    stageSettings,
  },);

  /**
   Gate outcome as it ships: a neither verdict over a forfeit standing, or
   any preference for an ineligible one, resolved toward the slate's choice.
   */
  const gated = shipPastForfeitStanding({
    outcome: gate,
    standingEligible,
    standingFlawedByMajority,
    slatePickBallots: slatePickBallotsOf({ decided, },),
    l,
  },);

  /**
   What ships once the semantic wrap has been applied and demotion re-derived.
   */
  const wrapped = wrapConsolidation({
    outcome: gated,
    consolidatedText: decided.text,
    standingText,
    lineStructured,
    l,
  },);

  /**
   Which of the three ways this slice could keep its standing text it took,
   kept apart because they answer different questions about the roster.
   */
  const terminal: ConsolidationTerminal = (wrapped.ships === 'consolidated')
    ? 'consolidated'
    : (wrapped.demoted ? 'wrap-erased-difference' : 'gate-kept-standing');

  /**
   The settlement the gate ends in.
   */
  const settlement: ConsolidationSettlement = {
    terminal,
    text: wrapped.text,
    floor,
    verdicts,
    decided,
    gate: gated,
    rewrapped: wrapped.rewrapped,
    demoted: wrapped.demoted,

    // The judged round already carries the produce half's findings, so
    // adding them again here would report one voice loss twice.
    findings: [
      ...decided.findings,
      ...gated.findings,
    ],
  };
  // A GATE LEAVING THE REFUSED STANDING IN PLACE leaves no wording the rule
  // passed, so the archive keeps the slice (owner, 2026-09-27, "Keep archive,
  // ship"); `shipPastForfeitStanding` has already turned every gate verdict it
  // can into the slate's choice, so only a wrap that erased the difference
  // reaches here.
  if (nothingValidShips({
    standingEligible,
    terminal,
  },)) {
    return keepTheArchive({
      settlement,
      subject,
      sliceIndex,
      // The gate is the final round here: one that heard fewer than two judges could not seat them.
      reason: unseatedFinalRound({ heard: gated.usable, },) ? 'final-round-unseated' : 'slate-declined',
      l,
    },);
  }

  /**
   How the slate shipped past two declines, absent where it chose.
   */
  const { shippedPastDecline, } = decided;
  return await applyFinalPolish({
    client,
    settlement,
    subject,
    lineStructured,
    sliceIndex,
    ...((polishConfig === undefined) ? {} : { polishConfig, }),
    eligible: (terminal === 'consolidated') || standingMayShip,
    // OVER AN INELIGIBLE STANDING every gate verdict ships the consolidation,
    // so what the gate held against it goes to the polish, and so do the
    // reasons of a slate that shipped it by preference past two declines
    // (owner, 2026-09-27, fourteenth and fifteenth addenda).
    objections: [
      ...((shippedPastDecline === undefined)
        ? []
        : [{
          origin: 'consolidation slate',
          objections: shippedPastDecline.objections,
        },] as const),
      ...(standingEligible
        ? []
        : [{
          origin: 'consolidation gate',
          objections: gateObjectionsOf({ ballots: gated.ballots, },),
        },] as const),
    ],
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);
}

//endregion Consolidate settle gate
