import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { CONSOLIDATE_GATE_QUORUM, } from './consolidate-gate-stage.ts';
import type {
  ConsolidationSettlement,
  ConsolidationTerminal,
} from './consolidate-settle.ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { TranslateDecision, } from '@monochromatic-dev/translation-repair-translate-stage/ts';

//region Consolidation persistence
// Cache eligibility belongs apart from document orchestration: it decides which
// settlements a warm run may treat as terminal, including whether unchanged
// baseline had prior approval.

/**
 Judge decisions second panel might change: a DELIBERATE SUBSET of
 `TranslateDecision`.

 A slate settles as `slate-declined-standing`, the one terminal this list is
 read for, on `declined-indecision`, `declined-rejection` or
 `no-candidate-backed`. LEFT OUT is `no-candidate-backed`, the reason
 recorded once the retry is spent, so the slate has already had its second
 panel and a third would not change what it settled. The other decisions are
 not listed because no slate settles as `slate-declined-standing` on them.

 `no-voice-heard` IS KEPT APART, in `UNHEARD_DECISION`, because it settles as
 `slate-unjudged-standing`, a terminal kept without a gate.
 */
const UNSETTLED_DECISIONS: readonly TranslateDecision[] = [
  'declined-indecision',
  'declined-rejection',
];

/**
 Decision recording that no voice was heard on the slate at all: the hour, not
 the slice. A later run could hear one, so a settlement carrying it is not kept
 although its terminal (`slate-unjudged-standing`) is kept for the decisions
 that left nothing to ask (a sole candidate, no candidate).
 */
const UNHEARD_DECISION: TranslateDecision = 'no-voice-heard';

/**
 Terminals settled enough to keep without reading judged round: a DELIBERATE
 SUBSET of `ConsolidationTerminal`.

 LEFT OUT are `gate-kept-standing`, `wrap-erased-difference` and
 `consolidated`, which always carry a gate and are decided on its quorum
 before this list is read, and `slate-declined-standing`, which is decided by
 what its judges settled on. A terminal the stage gains is therefore not kept
 across runs until it is added here.
 */
const SETTLED_WITHOUT_A_GATE: readonly ConsolidationTerminal[] = [
  'incumbent-only',
  'no-standing-text',
  'slate-endorsed-standing',
  'slate-unjudged-standing',
];

/**
 Whether settlement is worth keeping across runs.

 SETTLED VERDICTS ONLY. Thin panel is transient provider fact. Settlement
 retaining unendorsed baseline is nonterminal even when its own panel reached
 quorum, because final-selection guard will refuse same result.

 @param settlement - what stage settled

 @param standingMayShip - whether standing baseline has prior endorsement

 @returns Whether to persist it

 @example
 ```ts
 const keep = consolidationWorthResuming({ settlement, standingMayShip: true, });
 ```
 */
export function consolidationWorthResuming(
  {
    settlement,
    standingMayShip,
  }: {
    readonly settlement: ConsolidationSettlement;
    readonly standingMayShip: boolean;
  },
): boolean {
  // AN ARCHIVE-KEPT SETTLEMENT IS NEVER KEPT (owner, 2026-09-27, "Keep
  // archive, ship"). Its mark is what makes the archive ship; a copy resumed
  // without it would ship the refused standing its text holds. Refused here
  // outright rather than left to `baselineSettled`, which reaches the same
  // answer only while an ineligible standing is never endorsed.
  if (settlement.archiveKept === true)
    return false;
  // A SETTLEMENT BOUGHT ON A BENCH BELOW THE FLOOR IS THE HOUR, not the slice: no judge was asked, and a later
  // pass on a healthier bench must ask the slice rather than resume a standing nobody judged.
  if (settlement.benchBelowFloor === true)
    return false;
  // A SHIPPED TRANSLATE WINNER IS THE HOUR TOO (owner, 2026-10-29): the final round could not seat two judges, and a
  // copy resumed on a healthier bench would ship the winner without asking the slate that bench could seat.
  if (settlement.translateWinnerShipped === true)
    return false;
  /**
   Post-consolidation polish, absent before final candidate.
   */
  const { polish, } = settlement;
  /**
   Whether post-consolidation polish replaced otherwise unsafe baseline.
   */
  const polishSettled = (polish !== undefined)
    && (polish.kind === 'settled')
    && polish.changed;
  /**
   Whether unchanged standing baseline is terminal for this question.
   */
  const baselineSettled = standingMayShip
    ? true
    : (settlement.terminal === 'consolidated') || polishSettled;
  if (!baselineSettled)
    return false;
  /**
   What gate settled, absent where slice never reached it.
   */
  const { gate, } = settlement;
  if (gate !== undefined)
    return gate.usable >= CONSOLIDATE_GATE_QUORUM;

  /**
   How slice left stage.
   */
  const { terminal, } = settlement;
  // A SLATE NOBODY WAS HEARD ON IS THE HOUR, not the slice. The stage reaches
  // it only for a standing that shows nothing beside lane texts that show
  // nothing, which the lane offer never hands it, so nothing writes it today;
  // the rule is here so that a path that ever does is not frozen as settled.
  /**
   What judges decided, absent when no decision reached.
   */
  const { decided, } = settlement;
  if (decided?.decision === UNHEARD_DECISION)
    return false;
  if (SETTLED_WITHOUT_A_GATE.some(function matches(settled,): boolean {
    return settled === terminal;
  },))
    return true;

  if (terminal !== 'slate-declined-standing')
    return false;

  if (decided === undefined)
    return false;

  return !UNSETTLED_DECISIONS.some(function matches(unsettled,): boolean {
    return unsettled === decided.decision;
  },);
}

/**
 Persists bought consolidation only while caller remains live and settlement
 is stable enough for warm run.

 @param key - exact consolidation question

 @param settlement - complete stage answer

 @param cache - consolidation persistence boundary

 @param standingMayShip - whether unchanged baseline may become final output

 @param signal - caller abort checked before write

 @returns Whether settlement persisted and may be reused

 @throws Whatever caller abort reason or persistence throws

 @example
 ```ts
 await persistConsolidationSettlement({ key, settlement, cache, standingMayShip: true, signal, });
 ```
 */
export async function persistConsolidationSettlement(
  {
    key,
    settlement,
    cache,
    standingMayShip,
    signal,
  }: ForeignBorrowed<{
    readonly key: string;
    readonly settlement: ConsolidationSettlement;
    readonly cache: SliceCache<ConsolidationSettlement>;
    readonly standingMayShip: boolean;
    readonly signal: AbortSignal;
  }>,
): Promise<boolean> {
  signal.throwIfAborted();
  if (!consolidationWorthResuming({
    settlement,
    standingMayShip,
  },))
    return false;
  await cache.persist({
    key,
    serialized: JSON.stringify(settlement,),
  },);
  return true;
}

//endregion Consolidation persistence
