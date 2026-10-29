import {
  CONSOLIDATE_GATE_QUORUM,
  type GateBallot,
  type GateShipped,
  isGateChoice,
  settleGateBallots,
  type ConsolidationSettlement,
  type ConsolidationTerminal
} from '@monochromatic-dev/translation-repair-consolidate/ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  CONSOLIDATE_NAMESPACE,
  openNamespacedCache,
} from './slice-cache-namespace.ts';

//region Consolidate cache store
// Resuming a settlement an earlier run already bought for a contested slice.
//
// SEPARATE FROM THE CONTEST STORE, for the reason that store gives about the
// lanes: a settlement is not a contest, it is bought after one has already
// decided what stands, over the two lanes and the standing text together. It
// shares the entry directory and retires with the entry.
//
// THE SHAPE IS CHECKED DOWN TO THE BALLOT, following the contest store, and
// here it matters more than there. A settlement carries `text` that SHIPS: the
// record built from it hands that text to the assembly whenever the terminal
// says a consolidation won. So this store is the one path on which bytes read
// off disk become corpus text in an artifact, and a file that was corrupted,
// hand-edited, or written by a different schema would carry them there with
// nothing else in the way. Refusing it costs one re-asked slice.

/**
 What the stage writes beside a settlement's terminal on its way out of the
 stage.
 */
type WrittenWay = {
  /**
   Rendering the settlement's gate ships, or `not-asked` where the stage
   leaves before the gate and the settlement carries none.
   */
  readonly gate: GateShipped | 'not-asked';

  /**
   What the settlement's `demoted` flag reads on this way out: the wrap
   erased the difference on `wrap-erased-difference` and on no other.
   */
  readonly demoted: boolean;

  /**
   Whether the settlement's `rewrapped` flag may be true on this way out:
   only a consolidation the stage ships, or erases, passes through the wrap.
   */
  readonly mayRewrap: boolean;
};

/**
 Ways a settlement can leave the stage, as the terminal names them, each with
 what the stage writes beside it.

 SPELLED OUT RATHER THAN INFERRED, because this is a stored value: a union
 gaining a member should make an older cache file readable, not silently
 widen what this accepts to whatever the current source happens to say.
 KEYED BY `ConsolidationTerminal` all the same, so the compiler refuses a
 terminal the stage gains until a person has decided here what gate and wrap
 flags its settlement is written with, which is the decision that widens what
 this accepts.

 THE GATE COLUMN IS READ OFF THE STAGE. `settleConsolidation` leaves with no
 gate on five terminals: no standing text, a slate with nothing valid on it,
 and the three ways a judged slate keeps what stands. Every other settlement
 is `gateAndShip`'s, which always records its gate and names the terminal from
 what that gate ships: `gate-kept-standing` where it ships the standing text,
 and where it ships the consolidation, `consolidated` or, once the wrap finds
 the two alike, `wrap-erased-difference`.
 */
const SETTLEMENT_WAYS: Readonly<Record<ConsolidationTerminal, WrittenWay>> = {
  'incumbent-only': {
    gate: 'not-asked',
    demoted: false,
    mayRewrap: false,
  },
  'no-standing-text': {
    gate: 'not-asked',
    demoted: false,
    mayRewrap: false,
  },
  'slate-endorsed-standing': {
    gate: 'not-asked',
    demoted: false,
    mayRewrap: false,
  },
  'slate-unjudged-standing': {
    gate: 'not-asked',
    demoted: false,
    mayRewrap: false,
  },
  'slate-declined-standing': {
    gate: 'not-asked',
    demoted: false,
    mayRewrap: false,
  },
  'gate-kept-standing': {
    gate: 'standing',
    demoted: false,
    mayRewrap: false,
  },
  'wrap-erased-difference': {
    gate: 'consolidated',
    demoted: true,
    mayRewrap: true,
  },
  consolidated: {
    gate: 'consolidated',
    demoted: false,
    mayRewrap: true,
  },
};

/**
 Whether a stored value names a way a settlement can leave the stage.

 @param value - terminal as the cache file carries it

 @returns Whether it is one of the terminals `SETTLEMENT_WAYS` holds

 @example
 ```ts
 const known = isSettlementTerminal('consolidated',);
 ```
 */
function isSettlementTerminal(value: unknown,): value is ConsolidationTerminal {
  return ((typeof value) === 'string')
    && Object.hasOwn(
      SETTLEMENT_WAYS,
      value,
    );
}

/**
 Whether a value is one judge`s gate ballot as this schema writes it.

 @param value - parsed cache entry

 @returns Whether it is a readable ballot

 @example
 ```ts
 const readable = isGateBallot(parsed,);
 ```
 */
function isGateBallot(value: unknown,): value is GateBallot {
  if (!isJsonRecord(value,))
    return false;

  /**
   Renderings this judge called unsupported, before any is known to name one.
   */
  const { unsupported, } = value;

  /**
   Renderings this judge called incomplete, before any is known to name one.
   */
  const { dropped, } = value;
  return isGateChoice(value.choice,)
    && Array.isArray(unsupported,)
    && unsupported.every(function namesRendering(one,): boolean {
      return isGateChoice(one,);
    },)
    && Array.isArray(value.unsupportedRaw,)
    && Array.isArray(dropped,)
    && dropped.every(function namesRendering(one,): boolean {
      return isGateChoice(one,);
    },)
    && Array.isArray(value.droppedRaw,)
    && ((typeof value.reason) === 'string');
}

/**
 Whether a value is one proposal`s structural verdict.

 @param value - parsed cache entry

 @returns Whether it is a readable verdict

 @example
 ```ts
 const readable = isProposalVerdict(parsed,);
 ```
 */
function isProposalVerdict(value: unknown,): boolean {
  return isJsonRecord(value,)
    && ((typeof value.modelId) === 'string')
    && ((value.kind === 'valid') || (value.kind === 'invalid'))
    && Array.isArray(value.findings,);
}

/**
 Whether a value is what the validity floor made of a slate.

 @param value - parsed cache entry

 @returns Whether it is a readable floor

 @example
 ```ts
 const readable = isSlateFloor(parsed,);
 ```
 */
function isSlateFloor(value: unknown,): boolean {
  if (!isJsonRecord(value,))
    return false;
  if (value.kind === 'proposals')
    return Array.isArray(value.validModelIds,);
  return (value.kind === 'incumbent-only') && Array.isArray(value.refusedModelIds,);
}

/**
 Whether a value is what the gate settled, or nothing at all.

 ABSENT IS VALID and is not the same as empty. A slice the floor stopped never
 reached the gate, so its settlement carries no gate; a gate that ran and
 heard nobody carries one with no ballots, which the quorum check refuses as a
 gate short of its quorum. The terminal tells absence from that, and a store that
 required the key would refuse every floored slice. Whether
 absence is right for the settlement in hand is `gateFitsTerminal`'s question.

 THE CHOICE IS RECOMPUTED FROM THE BALLOTS. The stage settles `choice` with
 `settleGateBallots` over the ballots it stores and rewrites only `ships`
 afterwards, so a stored choice its own ballots do not give was not written by
 the stage. `choice` and `ships` are not compared: a forfeit standing ships
 the consolidation over a standing or neither choice.

 A GATE THAT HEARD FEWER VOICES THAN THE QUORUM IS REFUSED. The stage returns
 one when too few gate voices answer, and `consolidationWorthResuming` never
 persists it, because a thin panel is the hour and not the slice. A file
 holding one was not written by the stage, and resuming it would freeze the
 slice on that thin panel where a re-asked gate could hear the quorum.
 Refusing it costs one re-asked gate.

 @param value - parsed cache entry

 @returns Whether it is a readable gate outcome or absent

 @example
 ```ts
 const readable = isGateOutcomeOrAbsent(parsed.gate,);
 ```
 */
function isGateOutcomeOrAbsent(value: unknown,): boolean {
  if (value === undefined)
    return true;
  if (!isJsonRecord(value,))
    return false;

  /**
   Ballots the file carries, before any of them is known to be one.
   */
  const { ballots, } = value;
  return isGateChoice(value.choice,)
    && ((value.ships === 'consolidated') || (value.ships === 'standing'))
    && Array.isArray(ballots,)
    && ballots.every(isGateBallot,)
    && (value.choice === settleGateBallots({ ballots, },))
    && ((typeof value.usable) === 'number')
    && (value.usable === ballots.length)
    && (value.usable >= CONSOLIDATE_GATE_QUORUM)
    && Array.isArray(value.findings,);
}

/**
 Whether a gate outcome is the one the stage writes beside a terminal.

 THE TERMINAL AND THE GATE ARE ONE FACT WRITTEN TWICE, so a record where they
 disagree is a record the stage did not write, and this store exists to
 refuse those. The record built from a settlement hands its `text` to the
 assembly whenever the terminal reads `consolidated`: until 2026-10-05 a file
 corrupted or hand-edited into that terminal with no gate beside it was
 resumed, shipped its text, and left an artifact saying no gate was asked.
 Refusing it costs one re-asked slice.

 CALLED ONLY ON A GATE `isGateOutcomeOrAbsent` HAS PASSED, so a gate that is
 present is a record whose `ships` names one of the two renderings.

 @param gate - gate field of a parsed cache entry, its shape already checked

 @param written - what the stage writes for a gate beside the entry's terminal

 @returns Whether the gate is absent where the stage asks none, and ships the
 rendering the terminal names where it asks one

 @example
 ```ts
 const fits = gateFitsTerminal({ gate: parsed.gate, written: 'not-asked', },);
 ```
 */
function gateFitsTerminal(
  {
    gate,
    written,
  }: {
    readonly gate: unknown;
    readonly written: WrittenWay['gate'];
  },
): boolean {
  if (written === 'not-asked')
    return gate === undefined;
  return isJsonRecord(gate,) && (gate.ships === written);
}

/**
 Whether the wrap flags are the ones the stage writes beside a terminal.

 THE FLAGS ARE REPORTED, NOT RE-DERIVED. The artifact states `demoted` and
 `rewrapped` as facts about the slice, and each is a function of the way out:
 the wrap erases a difference only on `wrap-erased-difference`, and a wrap
 reaches a settlement only where a consolidation ships or is erased. A record
 where they disagree with its terminal is a record the stage did not write.

 @param written - what the stage writes beside the entry's terminal

 @param rewrapped - `rewrapped` field of a parsed cache entry, unchecked

 @param demoted - `demoted` field of a parsed cache entry, unchecked

 @returns Whether both are booleans and agree with the terminal

 @example
 ```ts
 const fits = wrapFlagsFitTerminal({ written, rewrapped: parsed.rewrapped, demoted: parsed.demoted, },);
 ```
 */
function wrapFlagsFitTerminal(
  {
    written,
    rewrapped,
    demoted,
  }: {
    readonly written: WrittenWay;
    readonly rewrapped: unknown;
    readonly demoted: unknown;
  },
): boolean {
  if (demoted !== written.demoted)
    return false;
  if (rewrapped === false)
    return true;
  return (rewrapped === true) && written.mayRewrap;
}

/**
 Whether a value is a settlement as this schema writes it.

 `decided` IS NOT CHECKED BEYOND BEING A RECORD. It is the translate stage`s
 own result, checked by that stage when it was produced, and nothing this
 store feeds reads more than its `decision`. Re-deriving its whole shape here
 would duplicate a contract that lives elsewhere and would refuse valid files
 whenever that contract gained a field.

 @param value - parsed cache entry

 @returns Whether it is this schema`s settlement

 @example
 ```ts
 if (isConsolidationSettlement(parsed,)) resumed.set(key, parsed,);
 ```
 */
function isConsolidationSettlement(value: unknown,): value is ConsolidationSettlement {
  if (!isJsonRecord(value,))
    return false;

  /**
   Judged round the file carries, absent where no slate reached the judges.
   */
  const { decided, } = value;

  /**
   Verdicts the file carries, before any of them is known to be one.
   */
  const { verdicts, } = value;

  /**
   Terminal the file names, before it is known to be one this schema writes.
   */
  const { terminal, } = value;

  /**
   Findings the file carries, before any of them is known to be a string.
   */
  const { findings, } = value;

  // A terminal that is no way this stage can leave is no settlement.
  if (!isSettlementTerminal(terminal,))
    return false;

  /**
   What the stage writes beside that terminal.
   */
  const named = SETTLEMENT_WAYS[terminal];
  return ((typeof value.text) === 'string')
    && isSlateFloor(value.floor,)
    && Array.isArray(verdicts,)
    && verdicts.every(isProposalVerdict,)
    && ((decided === undefined) || isJsonRecord(decided,))
    && isGateOutcomeOrAbsent(value.gate,)
    && gateFitsTerminal({
      gate: value.gate,
      written: named.gate,
    },)
    && wrapFlagsFitTerminal({
      written: named,
      rewrapped: value.rewrapped,
      demoted: value.demoted,
    },)
    // NEVER PERSISTED: `consolidationWorthResuming` refuses an archive-kept
    // settlement, and its mark is what makes the archive ship.
    && (value.archiveKept === undefined)
    // NEVER PERSISTED either: a settlement bought on a bench below the floor asked no judge, so a copy resumed
    // later would keep a standing nobody judged.
    && (value.benchBelowFloor === undefined)
    // NEVER PERSISTED either: a settlement that shipped the translate lane's winner because the final round could not
    // seat two judges is the hour (owner, 2026-10-29), and a copy resumed later would ship it without asking a bench
    // that could seat two.
    && (value.translateWinnerShipped === undefined)
    && Array.isArray(findings,)
    && findings.every(function isText(finding,): boolean {
      return (typeof finding) === 'string';
    },);
}

/**
 Opens the per-entry store of settlements already bought.

 @param dir - per-entry slice-cache directory

 @param generation - pipeline this run belongs to

 @returns Cache of settlements, keyed by slice hash

 @example
 ```ts
 const cache = await openConsolidateCache({ dir: entryCacheDir, generation: pipelineDigest, },);
 ```
 */
export async function openConsolidateCache(
  {
    dir,
    generation,
  }: {
    readonly dir: string;
    readonly generation: string;
  },
): Promise<SliceCache<ConsolidationSettlement>> {
  return await openNamespacedCache({
    dir,
    generation,
    namespace: CONSOLIDATE_NAMESPACE,
    isValue: isConsolidationSettlement,
  },);
}

//endregion Consolidate cache store
