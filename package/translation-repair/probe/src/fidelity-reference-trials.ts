import type { FidelityDamageKind, } from '@monochromatic-dev/translation-repair-repair/ts';
import { FidelityReferenceError, } from './fidelity-reference-error.ts';
import type {
  FidelityReferenceSpec,
  ReviewedFidelityReference,
} from './fidelity-reference-model.ts';
import type {
  FidelityDirection,
  FidelityTrial,
} from '@monochromatic-dev/translation-repair-run/ts';

//region Fixed reviewed trial matrix
// Positions and directions are preplanned, never generated from judge feedback.

/**
 Existing two-direction, two-position matrix.
 */
const ARRANGEMENTS: readonly {
  /**
   Which conceptual side is the reviewed reference.
   */
  readonly direction: FidelityDirection;
  /**
   Which ballot position carries that reference.
   */
  readonly cleanFirst: boolean;
}[] = [
  {
    direction: 'preserve',
    cleanFirst: true,
  },
  {
    direction: 'preserve',
    cleanFirst: false,
  },
  {
    direction: 'replace',
    cleanFirst: true,
  },
  {
    direction: 'replace',
    cleanFirst: false,
  },
];

/**
 One trial beside its reviewed provenance, not a claimed production slice index.

 @example
 ```ts
 const result = await runFidelityTrial({ trial: row.trial, ... });
 ```
 */
export type ReviewedFidelityTrial = {
  /**
   Manifest that established reference and damage validity.
   */
  readonly spec: FidelityReferenceSpec;
  /**
   Builder's reviewed changed-content count.
   */
  readonly changedChars: number;
  /**
   Safe structural description of the intentional delta.
   */
  readonly damageDetail: string;
  /**
   Exact comparison consumed by the existing selector.
   */
  readonly trial: FidelityTrial;
};

/**
 Expands reviewed variants into the existing position/direction matrix.

 @param references - materialized and hash-verified reviewed inputs

 @param damageKinds - requested supported families

 @returns Fixed matrix in reference, damage and arrangement order

 @throws {@link FidelityReferenceError} when selection cannot produce a reviewed comparison

 @example
 ```ts
 const trials = reviewedFidelityTrials({ references, damageKinds: ['deletion', 'insertion'] });
 ```
 */
export function reviewedFidelityTrials({
  references,
  damageKinds,
}: {
  readonly references: readonly ReviewedFidelityReference[];
  readonly damageKinds: readonly FidelityDamageKind[];
},): readonly ReviewedFidelityTrial[] {
  if ((damageKinds.length === 0) || (new Set(damageKinds,).size !== damageKinds.length))
    throw new FidelityReferenceError({
      referenceId: 'damage selection',
      operation: 'request',
    },);
  /**
   Every row exists before a model is asked.
   */
  const rows = references.flatMap(function referenceRows(reference,): readonly ReviewedFidelityTrial[] {
    return reference.damages
      .filter(function requested(damage,): boolean {
        return damageKinds.includes(damage.damageKind,);
      },)
      .flatMap(function arrangements(damage,): readonly ReviewedFidelityTrial[] {
        return ARRANGEMENTS.map(function arranged(arrangement,): ReviewedFidelityTrial {
          return {
            spec: reference.spec,
            changedChars: damage.changedChars,
            damageDetail: damage.damageDetail,
            trial: {
              trialId: `${reference.spec
                .id}/${damage.damageKind}`,
              direction: arrangement.direction,
              damageKind: damage.damageKind,
              sourceText: reference.sourceText,
              contextText: '',
              cleanText: reference.referenceText,
              damagedText: damage.damagedText,
              cleanFirst: arrangement.cleanFirst,
            },
          };
        },);
      },);
  },);
  /**
   A nonempty prefix is not evidence that every requested defect family was materialized.
   */
  const unavailable = damageKinds.filter(function missing(kind,): boolean {
    return !rows.some(function represents(row,): boolean {
      return row.trial
        .damageKind
        === kind;
    },);
  },);
  if (unavailable.length > 0)
    throw new FidelityReferenceError({
      referenceId: `damage selection (${unavailable.join(', ',)})`,
      operation: 'request',
    },);
  return rows;
}

//endregion Fixed reviewed trial matrix
