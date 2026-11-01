import type { ChunkRepairOutcome, } from './repair-contract.ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';

//region Repair replacements
// The repair lane's answer to "what text goes where", which is the only thing
// assembly needs from it.
//
// Kept out of `splice-slices.ts` so assembly stays lane-neutral: this is where
// the repair lane's notion of a change, an outcome carrying `changed` and
// `repairedText`, becomes the neutral instruction to write text over a span.

/**
 Turns repair outcomes into the replacements assembly applies.

 Outcomes that changed nothing are dropped rather than passed with their
 original text: writing a slice back over itself is a no-op that still reads,
 in every later diff and count, as a slice this lane touched.

 @param outcomes - per-slice repair outcomes in any order

 @returns One replacement per changed slice

 @example
 ```ts
 const replacements = repairReplacements({ outcomes, },);
 ```
 */
export function repairReplacements(
  { outcomes, }: { readonly outcomes: readonly ChunkRepairOutcome[]; },
): readonly SliceReplacement[] {
  return outcomes
    .filter(function isChanged(outcome,): boolean {
      return outcome.changed;
    },)
    .map(function toReplacement(outcome,): SliceReplacement {
      return {
        sliceIndex: outcome.sliceIndex,
        replacementText: outcome.repairedText,
      };
    },);
}

//endregion Repair replacements
