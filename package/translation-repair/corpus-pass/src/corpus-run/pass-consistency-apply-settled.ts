import type {
  AppliedEdits,
  ConsistencyEdit,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import type { ConsistencyKeptReason, } from '@monochromatic-dev/translation-repair-artifact/ts';
import type { LeftOut, } from './pass-consistency-apply-worked.ts';
import type { SliceEdits, } from './pass-consistency-apply-record.ts';
import type { ConsistencyFloorVerdict, } from './pass-consistency-floors.ts';
import type { ConsistencySliceFacts, } from './pass-consistency-slices.ts';

//region Pass consistency apply settled
// WHAT A SLICE COMES TO WHEN THE PLAN AND THE FLOORS HAVE AGREED, and how the floors are asked. These
// shapes are shared by the plan (`pass-consistency-apply-plan.ts`), which settles the page, and by the
// step that puts back the gains the check dropped too early (`pass-consistency-apply-readmit.ts`).
//
// THE FLOORS are the checks every slice with a replacement made passes (`pass-consistency-floors.ts`):
// the original's facts, the declared names, the line structure and the disputed wordings. They judge a
// slice whole, and a slice they refuse keeps the wording it has now. A PLAN IS SETTLED when the floors
// have said what they say of each slice with its replacements, the check of the apostrophes has run on
// the page the passing slices make, and no later step would change either.

/**
 What apostrophe-carrying code splices came to once the page was settled: why a splice that was left out stays out.
 */
export type GainReason =
  | {
    /**
     Another quotation mark of the final page could pair with the apostrophe the splice carries in, so it would read as a
     quotation mark there.
     */
    readonly kind: 'pairs';
  }
    | {
    /**
     The splice's own apostrophe would be proven on the final page, but making it would unprove the apostrophe a splice that
     is made carries in.
     */
      readonly kind: 'unproves';
    }
    | {
    /**
     The floors refuse the slice with the splice made.
     */
      readonly kind: 'floors';

    /**
     What the floors found.
     */
      readonly findings: readonly string[];
    }
    | {
    /**
     The splice's words overlap the words of a replacement that is made.
     */
      readonly kind: 'overlaps';
    }
    | {
    /**
     The floors refused the other replacements of the slice, which keeps its wording whole.
     */
      readonly kind: 'refused';
    };

/**
 What the apply stage does with one slice that carries English and has replacements.
 */
export type SlicePlan = SliceEdits & {
  /**
   What the pass knows of the slice.
   */
  readonly facts: ConsistencySliceFacts;

  /**
   The wording the slice carries now.
   */
  readonly currentText: string;

  /**
   The replacements that change the words another group located and that group does not replace, which are not made.
   */
  readonly leftOut: LeftOut;

  /**
   The replacements to make, in priority order: those not left out, less the code splices whose carried apostrophe is not
   proven on the page.
   */
  readonly toMake: readonly ConsistencyEdit[];

  /**
   The code splices left out because the apostrophe they carry in would not be one on the page.
   */
  readonly unprovenGains: readonly ConsistencyEdit[];
};

/**
 One slice of the settled page: its plan, the slice with the plan made, and what the floors made of it.
 */
export type SettledSlice = {
  /**
   What the slice makes and leaves out. For a slice the floors refused it is the plan as it stood when they refused it.
   */
  readonly plan: SlicePlan;

  /**
   The slice with the plan's replacements made, none when the plan makes none.
   */
  readonly made?: AppliedEdits;

  /**
   What the floors made of the slice with the plan made, none when the plan makes none.
   */
  readonly verdict?: ConsistencyFloorVerdict;

  /**
   For each code splice the plan left out, by name, the reason that holds on the final page.
   */
  readonly gainReasons: ReadonlyMap<string, GainReason>;
};

/**
 What the floors are asked about a slice.
 */
export type FloorsAsked = {
  /**
   The slice's plan.
   */
  readonly plan: SlicePlan;

  /**
   The slice with the plan's replacements made.
   */
  readonly candidateText: string;
};

/**
 The index of the slice a settled slice stands for.

 @param slice - the settled slice

 @returns The index of the slice, as the artifact names it

 @example
 ```ts
 const index = sliceIndexOf({ slice, },);
 ```
 */
export function sliceIndexOf({ slice, }: { readonly slice: SettledSlice; },): number {
  /**
   The plan of the slice.
   */
  const { plan, } = slice;
  /**
   The facts of the slice the plan is for.
   */
  const { facts, } = plan;
  return facts.sliceIndex;
}

/**
 The question the floors are asked about a slice with a set of replacements made: the slice and the replacements, which is
 what makes the same question the same.

 @param sliceIndex - the slice

 @param edits - the replacements made in it, in priority order

 @returns The question as a key for what the floors said

 @example
 ```ts
 const question = floorsQuestionOf({ sliceIndex: 1, edits, },); // '1 group 1 OCCURRENCE 2'
 ```
 */
export function floorsQuestionOf(
  {
    sliceIndex,
    edits,
  }: {
    readonly sliceIndex: number;
    readonly edits: readonly ConsistencyEdit[];
  },
): string {
  /**
   The names of the replacements, in priority order.
   */
  const ids = edits.map(function idOf({ id, },): string {
    return id;
  },);
  return `${String(sliceIndex,)} ${ids.join(' | ',)}`;
}

/**
 What holds on the final page of a splice that was left out, in the words of a reader of the log.

 @param reason - why the final page does not carry the splice

 @returns The reason as a clause that follows "because"

 @example
 ```ts
 const clause = becauseOf({ reason: { kind: 'overlaps', }, },);
 ```
 */
function becauseOf({ reason, }: { readonly reason: GainReason; },): string {
  if (reason.kind === 'pairs') {
    return 'the apostrophe it carries in would read as a quotation mark there, since another quotation mark of the page '
      + 'could pair with it';
  }
  if (reason.kind === 'unproves') {
    return 'the apostrophe it carries in would make the apostrophe that another replacement carries in lose its proof';
  }
  if (reason.kind === 'floors') {
    /**
     What the floors found.
     */
    const { findings, } = reason;
    return `the floors refuse the slice with it: ${findings.join(' | ',)}`;
  }
  if (reason.kind === 'overlaps')
    return 'its words overlap the words of another replacement that is made';
  return 'the floors refused the other replacements of its slice, which keeps its wording whole';
}

/**
 The reason the record keeps of a place whose code splice was left out, for each reason the final page gives.

 @param reason - why the final page does not carry the splice

 @returns The record's name for the reason, one for each kind

 @example
 ```ts
 const kept = keptReasonOfGain({ reason: { kind: 'pairs', }, },); // 'gain-pairs'
 ```
 */
export function keptReasonOfGain({ reason, }: { readonly reason: GainReason; },): ConsistencyKeptReason {
  if (reason.kind === 'pairs')
    return 'gain-pairs';
  if (reason.kind === 'unproves')
    return 'gain-unproves';
  if (reason.kind === 'floors')
    return 'gain-floors';
  if (reason.kind === 'overlaps')
    return 'gain-overlaps';
  return 'gain-refused';
}

/**
 What is reported of a code splice that was left out, in the words of the reason that holds on the final page.

 @param sliceIndex - the slice the splice was addressed to

 @param id - name of the splice

 @param reason - why the final page does not carry it

 @returns The finding, which ends with the place keeping its words

 @example
 ```ts
 const finding = gainFindingOf({ sliceIndex: 2, id: 'group 1 OCCURRENCE 2', reason: { kind: 'pairs', }, },);
 ```
 */
export function gainFindingOf(
  {
    sliceIndex,
    id,
    reason,
  }: {
    readonly sliceIndex: number;
    readonly id: string;
    readonly reason: GainReason;
  },
): string {
  return `slice ${String(sliceIndex,)}: the consistency replacement of ${id} was left out because ${
    becauseOf({ reason, },)
  }; the place keeps its words`;
}

//endregion Pass consistency apply settled
