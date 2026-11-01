import type { FidelityDamageKind, } from '@monochromatic-dev/translation-repair-repair/ts';
import { FidelityReferenceError, } from './fidelity-reference-error.ts';
import { REVIEWED_FIDELITY_REFERENCES, } from './fidelity-reference-manifest.ts';
import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';
import { selectReviewedFidelitySpecs, } from './fidelity-reference-select.ts';

//region Reviewed calibration requests
// Usage checks precede all corpus and provider activity, including metadata-only preflight.

/**
 Validates the fixed calibration request without reading files or creating a model client.

 @param specs - reviewed manifest, defaulting to the checked-in set

 @param corpusSha - configured corpus revision, checked without reading any files

 @param onlyEntryIds - explicit reviewed population filter

 @param damageKinds - requested reviewed defect families

 @param judgeModelIds - distinct identities that will judge the comparisons

 @param cap - nonnegative finite trial bound; zero means preflight only

 @param withContext - refused until context is independently reviewed with the fixtures

 @returns Owned selected specifications

 @throws {@link FidelityReferenceError} for unreviewed inputs or a correction author judging their own fixture

 @example
 ```ts
 const specs = reviewedFidelityRequest({ corpusSha, onlyEntryIds: [], damageKinds, judgeModelIds, cap: 0, withContext: false });
 ```
 */
export function reviewedFidelityRequest({
  specs = REVIEWED_FIDELITY_REFERENCES,
  onlyEntryIds,
  corpusSha,
  damageKinds,
  judgeModelIds,
  cap,
  withContext,
}: {
  readonly specs?: readonly FidelityReferenceSpec[];
  readonly onlyEntryIds: readonly string[];
  readonly corpusSha: string;
  readonly damageKinds: readonly FidelityDamageKind[];
  readonly judgeModelIds: readonly string[];
  readonly cap: number;
  readonly withContext: boolean;
},): readonly FidelityReferenceSpec[] {
  if (withContext)
    throw new FidelityReferenceError({
      referenceId: 'unreviewed context',
      operation: 'request',
    },);
  if ((!Number.isSafeInteger(cap,)) || (cap < 0))
    throw new FidelityReferenceError({
      referenceId: 'trial cap',
      operation: 'request',
    },);
  if ((judgeModelIds.length === 0) || (new Set(judgeModelIds,).size !== judgeModelIds.length)
    || judgeModelIds.some(function blankIdentity(modelId,): boolean {
      return modelId.trim() === '';
    },))
    throw new FidelityReferenceError({
      referenceId: 'judge roster',
      operation: 'request',
    },);
  if ((damageKinds.length === 0) || (new Set(damageKinds,).size !== damageKinds.length))
    throw new FidelityReferenceError({
      referenceId: 'damage selection',
      operation: 'request',
    },);
  /**
   Only reviewed entries may be requested, even during zero-call preflight.
   */
  const selected = selectReviewedFidelitySpecs({
    specs,
    onlyEntryIds,
  },);
  /**
   Every requested family needs reviewed evidence somewhere in the selected population.
   An available deletion does not authorize silently omitting a requested alteration.
   */
  const unavailable = damageKinds.filter(function missing(kind,): boolean {
    return !selected.some(function supportsRequestedDamage(spec,): boolean {
      return spec.damages
        .some(function requested(damage,): boolean {
          return damage.kind === kind;
        },);
    },);
  },);
  if (unavailable.length > 0) {
    throw new FidelityReferenceError({
      referenceId: `damage selection (${unavailable.join(', ',)})`,
      operation: 'request',
    },);
  }
  for (const spec of selected) {
    if (spec.corpusSha !== corpusSha)
      throw new FidelityReferenceError({
        referenceId: spec.id,
        operation: 'pin',
      },);
    for (const edit of spec.edits) {
      if (judgeModelIds.some(function authoredCorrection(modelId,): boolean {
        return (modelId === edit.author) || modelId.endsWith(`/${edit.author}`,);
      },)) {
        throw new FidelityReferenceError({
          referenceId: spec.id,
          operation: 'request',
        },);
      }
    }
  }
  return selected;
}

//endregion Reviewed calibration requests
