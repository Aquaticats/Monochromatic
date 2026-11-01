import type { ChunkRepairOutcome, } from '@monochromatic-dev/translation-repair-assembly/ts';
import { silentStagesOfRecord, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Repair cache gate
// WHY A SETTLED SLICE MAY NOT BE CACHED, said as reasons rather than as a
// boolean, so the driver's warn line names what happened.
//
// Two reasons, one old and one newer. No critic heard was always
// refused; a stage that fell short of quorum was not, because it settles on
// an ordinary "unchanged" exit with critics heard and only a finding says
// otherwise. Cached, that finding travelled with a record every later run
// resumed as "examined and found nothing to change".
//
// THE SILENCE IS READ FROM THE OUTCOME'S DATA FIRST (`silentStages`, then the plain findings), because a stage that
// locates its findings by place hides them from a reader of where a finding begins: XingZ6018 slice 85, whose panel
// heard nobody on five packets, was cached as examined for that reason.

/**
 Reasons a settled slice must not be cached, empty when it may be.

 @param outcome - settlement the driver is about to persist

 @returns Reasons in the words the warn line prints

 @example
 ```ts
 const refusals = cacheRefusalsOf({ outcome, },);
 if (refusals.length === 0) await cache.persist(...);
 ```
 */
export function cacheRefusalsOf(
  { outcome, }: { readonly outcome: ChunkRepairOutcome; },
): readonly string[] {
  if (outcome.heardCritics === 0)
    return ['no critic was heard',];
  return silentStagesOfRecord({ record: outcome, },)
    .map(function toReason(silent,): string {
      return `a stage heard fewer than quorum (${silent})`;
    },);
}

/**
 The warn line a refused settlement prints.

 SAYS ONLY WHAT IS TRUE OF A REFUSAL: nothing is persisted, and a later run
 buys the slice again. Whether the slice's text changed is the settlement's
 own fact, so it is said from the outcome and not assumed. The line said the
 slice "ships unchanged" for every reason, and zheermao21 slice 3 changed (an
 accuracy patch won and its repaired title reached the page) while its probe
 closed at 1 of 3.

 @param sliceIndex - slice the refusal concerns

 @param refusals - reasons from {@link cacheRefusalsOf}, never empty

 @param changed - whether the settled text differs from the archive's

 @returns Line naming the slice, every reason and what follows

 @example
 ```ts
 l.warn(cacheRefusalLine({ sliceIndex: 3, refusals, changed: outcome.changed, },),);
 ```
 */
export function cacheRefusalLine(
  {
    sliceIndex,
    refusals,
    changed,
  }: {
    readonly sliceIndex: number;
    readonly refusals: readonly string[];
    readonly changed: boolean;
  },
): string {
  return `chunk ${String(sliceIndex,)}: ${refusals.join('; ',)}, so this settlement is NOT cached and a later run `
    + `buys the slice again; the slice's text ${changed ? 'changed' : 'did not change'} in this run`;
}

//endregion Repair cache gate
