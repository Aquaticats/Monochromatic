import {
  type ArtifactDeliveryRow,
  ArtifactParseError,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type {
  ArtifactRepairEvidence,
  ArtifactTranslateEvidence,
} from './artifact-two-lane-read-contract.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Artifact version 2 set relations
// What has to hold between a lane's INDEX SETS, its counts, its status, and the
// ledger rows those describe.
//
// Each set is a second statement of something the ledger already says, and the
// ledger is the statement the writer checked against the document. So these are
// derived from the rows and compared, IN ORDER: both contracts say the sets are
// in document order, so an equal-length list in another order is a lane whose
// two derivations disagree, and a set read as a set would have thrown that away
// along with any repeated index.
//
// THE BLOCKED STATUS IS NOT RECOMPUTED, and cannot be. A blocked run and an
// unblocked one produce the same ledger whenever no slice decided anything
// different from the archive, so the derivation is not invertible and a reader
// claiming to recompute it would be refusing valid artifacts. What is checkable
// is COMPATIBILITY, which is what this file states.

/**
 Slices whose ledger row says the document carries a replacement.

 @param ledger - rows to read

 @returns Indices in document order

 @example
 ```ts
 const shipped = shippedIndicesOf({ ledger, },);
 ```
 */
function shippedIndicesOf(
  { ledger, }: { readonly ledger: readonly ArtifactDeliveryRow[]; },
): readonly number[] {
  return ledger.filter(function isShipped(row,): boolean {
    /**
     How this row's document came to carry what it carries.
     */
    const { delivery, } = row;
    return delivery.kind === 'replacement-shipped';
  },)
    .map(function toIndex(row,): number {
      return row.sliceIndex;
    },);
}

/**
 Slices the ASSEMBLY GUARD took a replacement back at.

 The whole-document refusal is deliberately not counted here. Both are
 withdrawals and they are different events: assembly ran and rejected this
 slice, against a document that was never assembled at all. A lane's withdrawn
 set names the first, so counting the second would make every blocked run look
 like a document the guard tore apart.

 @param ledger - rows to read

 @returns Indices in document order

 @example
 ```ts
 const withdrawn = guardWithdrawnIndicesOf({ ledger, },);
 ```
 */
function guardWithdrawnIndicesOf(
  { ledger, }: { readonly ledger: readonly ArtifactDeliveryRow[]; },
): readonly number[] {
  return ledger.filter(function isGuardWithdrawal(row,): boolean {
    /**
     How this row's document came to carry what it carries.
     */
    const { delivery, } = row;
    if (delivery.kind !== 'replacement-withdrawn')
      return false;
    return delivery.reason === 'assembly-integrity';
  },)
    .map(function toIndex(row,): number {
      return row.sliceIndex;
    },);
}

/**
 Refuses a list that does not match the one the rows produce.

 @param recorded - list the raw result carries

 @param derived - list the ledger rows produce

 @param path - dotted path of the recorded list

 @throws {@link ArtifactParseError} naming the first position they differ at,
 or the two lengths

 @example
 ```ts
 assertListMatches({ recorded, derived, path: 'lanes.repair.result.changedSliceIndices', },);
 ```
 */
function assertListMatches(
  {
    recorded,
    derived,
    path,
  }: {
    readonly recorded: readonly number[];
    readonly derived: readonly number[];
    readonly path: string;
  },
): void {
  if (recorded.length !== derived.length) {
    throw new ArtifactParseError({
      path,
      reason: `${String(derived.length,)} ${
        wordForCount({
          count: derived.length,
          one: 'slice',
          many: 'slices',
        },)
      }, which is what this lane's ledger rows say, rather than ${
        String(recorded.length,)
      }`,
    },);
  }
  for (const [
    position,
    index,
  ] of derived.entries()) {
    if (recorded[position] !== index) {
      throw new ArtifactParseError({
        path: `${path}[${String(position,)}]`,
        reason: `slice ${String(index,)}, which is what the ledger says in this position`,
      },);
    }
  }
}

/**
 Refuses a lane whose index sets disagree with its own ledger.

 @param evidence - the lane's recorded lists

 @param ledger - rows those lists describe

 @param path - dotted path of the lane's raw result

 @throws {@link ArtifactParseError} when either list differs from the one the
 rows produce

 @example
 ```ts
 assertIndexSetsMatchLedger({ evidence, ledger, path: 'lanes.repair.result', },);
 ```
 */
export function assertIndexSetsMatchLedger(
  {
    evidence,
    ledger,
    path,
  }: {
    readonly evidence: ArtifactRepairEvidence | ArtifactTranslateEvidence;
    readonly ledger: readonly ArtifactDeliveryRow[];
    readonly path: string;
  },
): void {
  assertListMatches({
    recorded: evidence.changedSliceIndices,
    derived: shippedIndicesOf({ ledger, },),
    path: `${path}.changedSliceIndices`,
  },);
  assertListMatches({
    recorded: evidence.withdrawnSliceIndices,
    derived: guardWithdrawnIndicesOf({ ledger, },),
    path: `${path}.withdrawnSliceIndices`,
  },);
  if (evidence.sliceCount !== ledger.length) {
    throw new ArtifactParseError({
      path: `${path}.sliceCount`,
      reason: `${String(ledger.length,)} ${
        wordForCount({
          count: ledger.length,
          one: 'slice',
          many: 'slices',
        },)
      }, which is how many rows this lane's ledger holds`,
    },);
  }
}

/**
 Refuses a repair lane whose deliveries could not have come from a run of the
 status it claims.

 ONE DIRECTION ONLY, which is what makes this a compatibility check rather
 than a recomputation: a blocked run whose slices all agreed with the archive
 produces no blocked withdrawal at all, and is a perfectly ordinary artifact.

 @param evidence - the lane's recorded status

 @param ledger - rows that status describes

 @param path - dotted path of the lane's raw result

 @throws {@link ArtifactParseError} when a blocked run carries a shipped
 replacement or a guard withdrawal, or an unblocked one carries a withdrawal
 naming the whole-document refusal

 @example
 ```ts
 assertBlockedCompatible({ evidence, ledger, path: 'lanes.repair.result', },);
 ```
 */
export function assertBlockedCompatible(
  {
    evidence,
    ledger,
    path,
  }: {
    readonly evidence: ArtifactRepairEvidence;
    readonly ledger: readonly ArtifactDeliveryRow[];
    readonly path: string;
  },
): void {
  /**
   Whether the run refused the whole document before assembling anything.
   */
  const blocked = evidence.status === 'blocked-non-translation';

  /**
   First row whose delivery contradicts that status, or nothing.
   */
  const contradiction = ledger.find(function contradicts(row,): boolean {
    /**
     How this row's document came to carry what it carries.
     */
    const { delivery, } = row;
    if (delivery.kind === 'replacement-shipped') {
      // Nothing was assembled by a blocked run, so nothing of it shipped.
      return blocked;
    }
    if (delivery.kind !== 'replacement-withdrawn')
      return false;

    // BOTH WITHDRAWALS ARE WRONG SOMEWHERE, and each in the opposite run: the
    // guard only ever ran on a document that was assembled, and the
    // whole-document refusal only ever happens to one that was not.
    return blocked
      ? (delivery.reason === 'assembly-integrity')
      : (delivery.reason === 'blocked-non-translation');
  },);
  if (contradiction !== undefined) {
    throw new ArtifactParseError({
      path: `${path}.status`,
      reason: blocked
        ? `a status this ledger could hold: slice ${
          String(contradiction.sliceIndex,)
        } reports assembly having run, which a blocked run never reaches`
        : `a status this ledger could hold: slice ${
          String(contradiction.sliceIndex,)
        } reports a withdrawal by whole-document refusal, which only a blocked run produces`,
    },);
  }
}

/**
 Refuses a translate lane whose counts or status disagree with what it
 recorded per slice.

 @param evidence - the lane's counts, status and lists

 @param path - dotted path of the lane's raw result

 @throws {@link ArtifactParseError} when either count differs from the list
 beside it, or the status disagrees with whether any slice went unfilled

 @example
 ```ts
 assertTranslateCountsAgree({ evidence, path: 'lanes.translate.result', },);
 ```
 */
export function assertTranslateCountsAgree(
  {
    evidence,
    path,
  }: {
    readonly evidence: ArtifactTranslateEvidence;
    readonly path: string;
  },
): void {
  /**
   How many slices this lane names as shipped.
   */
  const shippedCount = evidence.changedSliceIndices
    .length;

  /**
   How many it names as withdrawn.
   */
  const withdrawnCount = evidence.withdrawnSliceIndices
    .length;
  if (evidence.changedSliceCount !== shippedCount) {
    throw new ArtifactParseError({
      path: `${path}.changedSliceCount`,
      reason: `${String(shippedCount,)}, which is how many slices this lane names as shipped`,
    },);
  }
  if (evidence.withdrawnSliceCount !== withdrawnCount) {
    throw new ArtifactParseError({
      path: `${path}.withdrawnSliceCount`,
      reason: `${String(withdrawnCount,)}, which is how many slices this lane names as withdrawn`,
    },);
  }

  /**
   Whether any slice was reached and could not be filled, which is the ONLY
   thing this status reports: a slice left alone for any other reason keeps
   the archive's wording, and the document is whole either way.
   */
  const anyUnfilled = evidence.sliceTexts
    .some(function isUnfilled(row,): boolean {
      /**
       What the lane did about this slice.
       */
      const { outcome, } = row;
      return outcome.kind === 'unfilled';
    },);
  if (anyUnfilled !== (evidence.status === 'unfilled')) {
    throw new ArtifactParseError({
      path: `${path}.status`,
      reason: anyUnfilled
        ? 'unfilled, since this lane records a slice it reached and could not fill'
        : 'complete, since this lane records no slice it reached and could not fill',
    },);
  }
}

//endregion Artifact version 2 set relations
