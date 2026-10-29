import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { SliceValidation, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Consolidation validity floor
// REFUSES A SLATE CARRYING NO STRUCTURALLY VALID PROPOSAL, before the gate is
// asked which rendering is more faithful.
//
// The policy is not chosen here, it is inherited. `translate-produce.ts` says
// invalid renderings go back to their authors and THE DISTINCT SURVIVORS
// become a slate with the incumbent among them, so a candidate that failed
// validation is not a slate member. It separately exempts the incumbent from
// the validity check, because a check that could drop it would be a check that
// could delete the archive. Together those give one answer: when nothing
// survives, the slate is the incumbent alone.
//
// MEASURED, NOT ANTICIPATED. `Zha_Ke#1` finished the repair round with five
// candidates and zero valid ones, in both runs of the band pair, and a
// consolidation shipped at both. The page there is two blocks; two candidates
// produced one block, one produced the wrong element kind, and the rest did
// not parse as MDX. The guard refused all five and the slice shipped one
// anyway. `doc/planning/the-third-rendering.md` records it.
//
// WHY BEFORE THE GATE RATHER THAN AFTER: the gate answers which of two
// renderings is more faithful to the original, and that question has no
// meaning when one of them is structurally not the page it would be written
// into. Asking it anyway buys ballots about a candidate that cannot ship.

/**
 One proposal's identity beside what the structural guard made of it.

 @example
 ```ts
 const checked: ProposalValidity = { modelId: 'hf:cat/Cat-A', validation: { kind: 'valid', pageGrammar: 'strict', }, };
 ```
 */
export type ProposalValidity = {
  /**
   Voice that wrote this proposal, carried so a refusal can name who was
   refused rather than only how many were.
   */
  readonly modelId: string;

  /**
   What the structural guard returned for it, after any repair round.
   */
  readonly validation: SliceValidation;
};

/**
 What a slate amounts to once the invalid proposals are not in it.

 @example
 ```ts
 const floor: SlateFloor = { kind: 'incumbent-only', refusedModelIds: ['hf:cat/Cat-A'], };
 ```
 */
export type SlateFloor =
  | {
    readonly kind: 'proposals';

    /**
     Voices whose proposal survived validation, in the order given.
     */
    readonly validModelIds: readonly string[];
  }
    | {
      readonly kind: 'incumbent-only';

    /**
     Every voice whose proposal was refused, so the record says who wrote a
     candidate that could not ship rather than only that none could.
     */
      readonly refusedModelIds: readonly string[];
    };

/**
 Decides whether a consolidation slate has anything the gate can be asked about.

 AN EMPTY ROSTER READS AS INCUMBENT-ONLY rather than as an error. A stage that
 bought no voices at all and a stage whose every voice was refused both leave
 the standing text as the only thing that can ship, and the caller that has to
 act on either does the same thing. What separates them is the refused list,
 which is empty in the first case and named in the second.

 @param validity - each proposal's identity and structural verdict, after any
 repair round has had its turn

 @param l - stage logger

 @returns Whether proposals survive, and who they belong to

 @example
 ```ts
 const floor = floorConsolidateSlate({ validity, l, },);
 ```
 */
export function floorConsolidateSlate(
  {
    validity,
    l,
  }: {
    readonly validity: readonly ProposalValidity[];
    readonly l: Logger;
  },
): SlateFloor {
  /**
   Voices whose proposal the structural guard passed.
   */
  const survived = validity.filter(function isValid({ validation, },): boolean {
    return validation.kind === 'valid';
  },);
  /**
   Voices withheld beside those survivors, named in the log because the
   artifact alone records them (ledger E5).
   */
  const withheld = validity
    .filter(function isRefused({ validation, },): boolean {
      return validation.kind !== 'valid';
    },)
    .map(function toModelId(checked,): string {
      return checked.modelId;
    },);

  if ((survived.length > 0) && (withheld.length > 0)) {
    l.info(
      `consolidation slate: withheld ${withheld.join(', ',)}, whose proposals did not pass the structural guard; ${
        String(survived.length,)
      } remain for the judges`,
    );
  }

  if (survived.length > 0)
    return {
      kind: 'proposals',
      validModelIds: survived.map(function toModelId(checked,): string {
        return checked.modelId;
      },),
    };

  /**
   Everyone the guard refused, which is everyone when nothing survived.
   */
  const refusedModelIds = validity.map(function toModelId(checked,): string {
    return checked.modelId;
  },);

  if (refusedModelIds.length > 0)
    l.warn(
      `consolidation slate: all ${
        String(refusedModelIds.length,)
      } proposals failed the structural guard, so the slice keeps its standing text and no gate is asked`,
    );

  return {
    kind: 'incumbent-only',
    refusedModelIds,
  };
}

//endregion Consolidation validity floor
