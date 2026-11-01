import type { DocumentLanesResult, } from '@monochromatic-dev/translation-repair-translate-lane/ts';
import {
  type ArtifactComparisonRow,
  type ArtifactDeliveryRow,
  compareLanes,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { toArtifactRow, } from './artifact-two-lane-project.ts';

//region Artifact version 2 derivation
// Both ledgers as version 2 rows, and what version 2`s own rules make of them.
//
// ONE DEFINITION FOR TWO CALLERS. The builder writes these rows and this
// comparison into the artifact; the contest driver needs the same comparison to
// know which slices are worth asking a roster about. Deriving it twice would
// let the two drift, and the drift would show up as a settled artifact whose
// contest the reader refuses for answering the wrong slices, which is a true
// refusal of a run that was never wrong about anything.

/**
 Both ledgers projected into version 2, beside the comparison they derive.

 @example
 ```ts
 const projected: ProjectedLanes = projectLanes({ lanes, },);
 ```
 */
export type ProjectedLanes = {
  /**
   Each lane`s ledger, rebuilt field by field as version 2 rows.
   */
  readonly delivery: Readonly<Record<'repair' | 'translate', readonly ArtifactDeliveryRow[]>>;

  /**
   What version 2`s frozen rules make of those two ledgers.
   */
  readonly comparison: readonly ArtifactComparisonRow[];
};

/**
 Projects both ledgers and derives the comparison version 2 records.

 PROJECTED rather than assigned, because assignment freezes only half of what
 the frozen vocabulary claims: a live union that gains a MEMBER fails to
 assign, and a live row that gains a FIELD assigns cleanly and then gets
 serialized, into artifacts the version 2 parser refuses for carrying keys the
 schema does not name.

 @param lanes - what both drivers returned

 @returns Version 2 rows and the comparison derived from them

 @example
 ```ts
 const { delivery, comparison, } = projectLanes({ lanes, },);
 ```
 */
export function projectLanes(
  { lanes, }: { readonly lanes: DocumentLanesResult; },
): ProjectedLanes {
  /**
   Both ledgers rebuilt as version 2 rows.
   */
  const delivery: Readonly<Record<'repair' | 'translate', readonly ArtifactDeliveryRow[]>> = {
    repair: lanes.repairDelivery
      .records
      .map(function projectRepair(record,): ArtifactDeliveryRow {
        return toArtifactRow({ record, },);
      },),
    translate: lanes.translateDelivery
      .records
      .map(function projectTranslate(record,): ArtifactDeliveryRow {
        return toArtifactRow({ record, },);
      },),
  };
  return {
    delivery,

    // THE FROZEN RULES, over the rows the artifact carries. Deriving the
    // persisted comparison from the live comparator alone would let a later
    // change to how a verdict is decided reinterpret every artifact on disk,
    // under an unchanged version number and with nothing in the file recording
    // which rules produced it.
    comparison: compareLanes({
      repair: delivery.repair,
      translate: delivery.translate,
    },),
  };
}

//endregion Artifact version 2 derivation
