import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  PreparationIdentity,
  PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { IdentifiedDeliveryLedger, } from '@monochromatic-dev/translation-repair-lane-contest/ts';

//region Artifact version 2 verification
// That a lane's ledger describes THIS preparation, checked rather than assumed.
//
// The comparison refuses two ledgers naming different preparations, and that
// refusal says nothing about whether either names the RIGHT one: two ledgers
// built over some other slicing agree with each other perfectly. The builder is
// the only boundary holding all three, so a caller that passes the wrong
// preparation would otherwise get a clean artifact whose recorded identity
// names a slicing the lanes never ran over, which is this generation's own
// defect class one level up.
//
// TWO CHECKS, because neither implies the other:
//
//  -   THE NAME. Each ledger carries the identity of the slicing it was built
//      over, stamped by the driver that built it, and that identity covers what
//      a row cannot show: both whole documents, every slice's offsets on both
//      sides, the line-structure flag, and whether an identity block was in
//      play. Two preparations can agree on every field a ledger exposes and
//      still differ in all of those.
//  -   THE ROWS. An equal name is a hash claim; the four per-slice facts are
//      the ones this artifact actually joins on, and checking them turns a
//      collision or a mis-stamped ledger into a refusal here rather than into
//      rows filed under a slicing they do not describe.
//
// `buildSliceDelivery` already refuses three of the four against the slices IT
// was handed. This refuses them against the slices the ARTIFACT is about, which
// is a different claim whenever the two preparations are not the same object.

/**
 Reports a ledger that does not describe the preparation it is filed under.

 @example
 ```ts
 throw new ArtifactPreparationMismatchError({ message: 'repair ledger names slice 4 at position 3', },);
 ```
 */
export class ArtifactPreparationMismatchError extends Error {
  /**
   Names this error for a caller matching on it.
   */
  public override readonly name = 'ArtifactPreparationMismatchError';

  /**
   @param message - what disagreed, naming lane and slice

   @example
   ```ts
   new ArtifactPreparationMismatchError({ message: 'translate ledger has 3 rows for 4 slices', },);
   ```
   */
  public constructor({ message, }: { readonly message: string; },) {
    super(message,);
  }
}

/**
 Refuses a ledger that does not describe the given preparation.

 Checks the name the ledger's builder stamped on it, then the four per-slice
 facts a row carries over from preparation: which slice it is, the original it
 renders, whether the archive holds any wording there, and what that wording
 is. A row disagreeing on any of them was built over different slices, however
 well it lines up with the other lane's.

 @param prepared - slicing this artifact is about

 @param expected - name that slicing gives itself, computed by the caller so
 it is not recomputed once per lane

 @param ledger - one lane's ledger, one row per slice in position order,
 carrying the identity of the slicing it was built over

 @param lane - which lane, so a message names the side at fault

 @throws {@link ArtifactPreparationMismatchError} when the name, the count or
 any per-slice fact disagrees

 @example
 ```ts
 assertLedgerDescribesPreparation({ prepared, expected, ledger: lanes.repairDelivery, lane: 'repair', },);
 ```
 */
export function assertLedgerDescribesPreparation(
  {
    prepared,
    expected,
    ledger,
    lane,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly expected: PreparationIdentity;
    readonly ledger: IdentifiedDeliveryLedger;
    readonly lane: 'repair' | 'translate';
  },
): void {
  /**
   Rows the ledger filed, under the name it was built over.
   */
  const {
    preparationIdentity: named,
    records,
  } = ledger;
  if (named !== expected) {
    throw new ArtifactPreparationMismatchError({
      message: `${lane} ledger was built over ${named} and is being filed under ${expected}, `
        + 'so it describes a different slicing of some pair of documents',
    },);
  }

  /**
   Slices the ledger has to cover, one row each.
   */
  const { slices, } = prepared;
  if (records.length !== slices.length) {
    throw new ArtifactPreparationMismatchError({
      message: `${lane} ledger has ${String(records.length,)} ${
        wordForCount({
          count: records.length,
          one: 'row',
          many: 'rows',
        },)
      } for a preparation of ${
        String(slices.length,)
      } ${
        wordForCount({
          count: slices.length,
          one: 'slice',
          many: 'slices',
        },)
      }, so it was built over a different slicing`,
    },);
  }
  for (const [
    position,
    slice,
  ] of slices.entries()) {
    /**
     Row filed for this position. The row count was just held to the slice
     count, so every position has one; a refusal for a missing row stood here
     and no ledger could reach it (ledger T8).
     */
    const record = nonNullishOrThrow(records[position],);

    /**
     What the preparation says this position holds.
     */
    const {
      sliceIndex,
      text: incumbentText,
    } = slice.target;
    if (record.sliceIndex !== sliceIndex) {
      throw new ArtifactPreparationMismatchError({
        message: `${lane} ledger names slice ${String(record.sliceIndex,)} at position ${
          String(position,)
        }, where the preparation has slice ${String(sliceIndex,)}`,
      },);
    }

    /**
     Original the preparation pairs with that archive wording.
     */
    const { text: sourceText, } = slice.source;
    if (record.sourceText !== sourceText) {
      throw new ArtifactPreparationMismatchError({
        message: `${lane} ledger's slice ${
          String(record.sliceIndex,)
        } renders an original the preparation does not carry there`,
      },);
    }

    /**
     What the preparation says about the archive at this slice.
     */
    const incumbentKind = isInsertionChunk(slice.target,) ? 'absent' : 'present';
    if (record.incumbentKind !== incumbentKind) {
      throw new ArtifactPreparationMismatchError({
        message: `${lane} ledger's slice ${String(record.sliceIndex,)} calls the archive wording ${
          record.incumbentKind
        } where the preparation calls it ${incumbentKind}`,
      },);
    }
    if (record.incumbentText !== incumbentText) {
      throw new ArtifactPreparationMismatchError({
        message: `${lane} ledger's slice ${
          String(record.sliceIndex,)
        } carries archive wording the preparation does not have there`,
      },);
    }
  }
}

/**
 Refuses a lane result counting slices the preparation does not have.

 The ledgers are checked row by row and the RAW RESULTS beside them are not:
 a structurally valid driver result could pair one lane's result with the
 other's ledger, or with a result from another entry entirely. Both lanes
 report the slice count their preparation produced, which is the one field
 cheap enough to check here and enough to refuse a grossly mismatched pairing.

 NOT a proof that the result and the ledger beside it came from one run. That
 needs the result's document re-spliced from the ledger's rows, which the
 driver is better placed to do than the artifact writer.

 @param prepared - slicing this artifact is about

 @param sliceCount - count the lane's own result reports

 @param lane - which lane, so a message names the side at fault

 @throws {@link ArtifactPreparationMismatchError} when the counts differ

 @example
 ```ts
 assertResultCountsPreparation({ prepared, sliceCount: lanes.repair.sliceCount, lane: 'repair', },);
 ```
 */
export function assertResultCountsPreparation(
  {
    prepared,
    sliceCount,
    lane,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly sliceCount: number;
    readonly lane: 'repair' | 'translate';
  },
): void {
  /**
   Slices the preparation produced, which both lanes report a count of.
   */
  const preparedSliceCount = prepared.slices
    .length;
  if (sliceCount !== preparedSliceCount) {
    throw new ArtifactPreparationMismatchError({
      message: `${lane} result counts ${String(sliceCount,)} ${
        wordForCount({
          count: sliceCount,
          one: 'slice',
          many: 'slices',
        },)
      } where the preparation has ${
        String(preparedSliceCount,)
      }, so the result describes a different run`,
    },);
  }
}

/**
 Refuses a run whose alignment findings differ from its preparation's.

 The artifact records these ONCE, on the preparation, and the driver reports
 them too. Two derivations of one fact reaching the same boundary is the
 moment to check them rather than to pick one, which is what recording either
 silently amounts to.

 @param prepared - slicing this artifact is about

 @param reported - findings the lane driver returned

 @throws {@link ArtifactPreparationMismatchError} when the two lists differ in
 length or in any entry

 @example
 ```ts
 assertFindingsDescribePreparation({ prepared, reported: lanes.alignmentFindings, },);
 ```
 */
export function assertFindingsDescribePreparation(
  {
    prepared,
    reported,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly reported: readonly string[];
  },
): void {
  /**
   Findings the preparation itself observed.
   */
  const { alignmentFindings, } = prepared;
  if (reported.length !== alignmentFindings.length) {
    throw new ArtifactPreparationMismatchError({
      message: `run reports ${String(reported.length,)} alignment ${
        wordForCount({
          count: reported.length,
          one: 'finding',
          many: 'findings',
        },)
      } for a preparation with ${
        String(alignmentFindings.length,)
      }`,
    },);
  }

  for (const [
    position,
    finding,
  ] of reported.entries()) {
    if (finding !== alignmentFindings[position]) {
      throw new ArtifactPreparationMismatchError({
        message: `run's alignment finding ${
          String(position,)
        } is not what the preparation observed there, so the two describe different documents`,
      },);
    }
  }
}

//endregion Artifact version 2 verification
