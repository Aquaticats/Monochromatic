//region Repair unheard
// What it means for the repair lane to have heard nobody about a slice, named
// once so every reader asks the same question.
//
// THE DEFECT THIS ENDS, measured on a two-slice fixture whose every critic call
// failed: 48 attempts, quorum unmet 0 of 6 on each slice, and the lane reported
// `decided` at both. `decided` means the lane produced a wording. Here nobody
// was heard, so the archive's wording stood by DEFAULT rather than by anyone's
// choice, which is what `incumbent-fallback` exists to say. Recorded as it was,
// "critics examined this slice and found nothing" and "no critic answered at
// all" were the same row, and a provider outage during a corpus pass produced
// artifacts that read like clean runs.
//
// The signal was already there and nothing read it: the findings carry
// `stage-quorum-unmet (critic 0/6)` per slice, and `heardCriticIds` names who
// answered. This turns that into the per-slice outcome.
//
// WHY TWO STAGES AND NOT ONE. Critics are not the only voice that can produce a
// wording here: the naturalness lane rewrites a slice after the accuracy pass
// settles, and it can do so on a slice no critic ever raised anything about. A
// slice it rewrote HAS a wording somebody produced, so hearing nobody means
// both stages were silent, not just the critics.

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

/**
 What a silent slice may wrongly claim.

 @example
 ```ts
 const claim: UnheardClaim = 'change';
 ```
 */
export type UnheardClaim = 'foreign-wording' | 'change';

/**
 Sentence ending for each claim, fixed text keyed by the literal.
 */
const UNHEARD_CLAIMS: Readonly<Record<UnheardClaim, string>> = {
  'foreign-wording': 'carries a wording that is not the archive\'s, so something produced text no stage was '
    + 'recorded as having produced',
  'change': 'claims a change',
};

/**
 Reports a slice the repair lane heard nobody about that carries a wording
 anyway.

 @example
 ```ts
 throw new RepairUnheardError({ sliceIndex: 3, claim: 'foreign-wording', },);
 ```
 */
export class RepairUnheardError extends Error {
  /**
   Names this error for a caller matching on it.
   */
  public override readonly name = 'RepairUnheardError';

  /**
   Declares this message safe to print whole at a boundary: it names a slice
   index and one of two fixed claims, and quotes nothing.
   */
  readonly messageNamesOnly: true = true;

  /**
   @param sliceIndex - slice nobody spoke about

   @param claim - what the outcome nevertheless claimed

   @example
   ```ts
   new RepairUnheardError({ sliceIndex: 3, claim: 'change', },);
   ```
   */
  public constructor(
    {
      sliceIndex,
      claim,
    }: {
      readonly sliceIndex: number;
      readonly claim: UnheardClaim;
    },
  ) {
    super(`slice ${String(sliceIndex,)} heard no critic and was never refined, and ${UNHEARD_CLAIMS[claim]}`,);
  }
}

/**
 What this reads off one settled slice, which is the whole of what it needs.

 A STRUCTURAL SUBSET of `ChunkRepairOutcome` rather than that type, so this
 file does not depend on the whole repair contract to ask one question of it.

 @example
 ```ts
 const outcome: RepairVoiceRecord = { sliceIndex: 0, repairedText, changed: false, ... };
 ```
 */
export type RepairVoiceRecord = {
  /**
   Slice this settled.
   */
  readonly sliceIndex: number;

  /**
   Wording the lane settled on.
   */
  readonly repairedText: string;

  /**
   Whether the lane reports having changed anything here.
   */
  readonly changed: boolean;

  /**
   Critics that answered on this slice, empty when none did.
   */
  readonly heardCriticIds: readonly RosterModelId[];

  /**
   Whether the naturalness lane rewrote this slice after the accuracy pass.
   */
  readonly refined: boolean;
};

/**
 Whether the lane heard no voice at all about this slice.

 ONE DEFINITION for every reader, because the question is asked in more than
 one place and each spelling of it is a chance for two readers to disagree
 about what silence was.

 @param outcome - what the lane settled for one slice

 @returns Whether both producing stages were silent here

 @example
 ```ts
 const silent = heardNobodyAbout({ outcome, },);
 ```
 */
export function heardNobodyAbout(
  { outcome, }: { readonly outcome: RepairVoiceRecord; },
): boolean {
  /**
   Whether any critic answered here.
   */
  const criticsAnswered = outcome.heardCriticIds
    .length
    > 0;
  return (!criticsAnswered) && (!outcome.refined);
}

/**
 Refuses a slice the lane heard nobody about that carries anything but the
 archive's own wording.

 The counterpart of `assertUnheardKeptIncumbent` on the translate side, and it
 exists for the same reason: a silent stage having produced a wording is a
 contradiction, and one caught here is one that never reaches a ledger, a
 comparison or a rate.

 @param outcome - what the lane settled for one slice

 @param incumbentText - archive's own wording for that slice

 @throws {@link RepairUnheardError} when a slice nobody spoke about carries a
 different wording or claims a change

 @example
 ```ts
 assertUnheardKeptArchive({ outcome, incumbentText, },);
 ```
 */
export function assertUnheardKeptArchive(
  {
    outcome,
    incumbentText,
  }: {
    readonly outcome: RepairVoiceRecord;
    readonly incumbentText: string;
  },
): void {
  if (!heardNobodyAbout({ outcome, },))
    return;

  // Where the contradiction is, for a message that names one slice.
  if (outcome.repairedText !== incumbentText) {
    throw new RepairUnheardError({
      sliceIndex: outcome.sliceIndex,
      claim: 'foreign-wording',
    },);
  }
  if (outcome.changed) {
    throw new RepairUnheardError({
      sliceIndex: outcome.sliceIndex,
      claim: 'change',
    },);
  }
}

//endregion Repair unheard
