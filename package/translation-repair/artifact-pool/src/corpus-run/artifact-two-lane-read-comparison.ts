import { ArtifactParseError, } from '../artifact-guard.ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { compareLanes, } from './artifact-two-lane-comparison.ts';
import { ArtifactComparisonError, } from './artifact-two-lane-comparison-fault.ts';
import { comparisonRowDifferences, } from './artifact-two-lane-row-equality.ts';
import type {
  ArtifactComparisonRow,
  ArtifactDeliveryRow,
} from './artifact-two-lane-vocabulary.ts';

//region Artifact version 2 recorded comparison
// Checking the comparison a version 2 artifact CARRIES against the one its own
// ledgers produce.
//
// The stored comparison is a claim about two ledgers stored beside it, so
// nothing has to trust it: a reader derives its own and refuses a disagreement.
// What makes that worth doing is the derivation being FROZEN. Recomputing with
// the live comparator would agree with itself under any later change to how a
// verdict is decided, while both disagreed with what the artifact meant when it
// was written; `compareLanes` is version 2's own rules over version 2's own
// rows, so a reader running it asks what this file meant rather than what the
// pipeline currently thinks.
//
// AND THE DERIVED COPY IS WHAT COMES BACK, not the recorded one. They are equal
// by the time this returns, so which one a caller gets cannot change an answer;
// returning the derived rows means a caller holds the reading this reader can
// account for.

/**
 Derives the comparison from two ledgers, reporting a refusal as a parse
 failure.

 Separate from the caller so the translation happens once and around the one
 call that can raise the comparison's own error type.

 @param repair - repair lane's ledger

 @param translate - translate lane's ledger

 @param path - dotted path of the recorded comparison

 @returns Comparison version 2's rules derive

 @throws {@link ArtifactParseError} when the two ledgers cannot be compared at
 all, carrying what the comparison said

 @throws Whatever else the comparison raised, unchanged, since only its own
 refusal is a fact about the file

 @example
 ```ts
 const derived = deriveComparison({ repair, translate, path, },);
 ```
 */
function deriveComparison(
  {
    repair,
    translate,
    path,
  }: {
    readonly repair: readonly ArtifactDeliveryRow[];
    readonly translate: readonly ArtifactDeliveryRow[];
    readonly path: string;
  },
): readonly ArtifactComparisonRow[] {
  try {
    return compareLanes({
      repair,
      translate,
    },);
  } catch (error) {
    // ONLY THE COMPARISON'S OWN REFUSAL DESCRIBES THE FILE. Anything else
    // raised in here is a defect in this reader, and rewriting it as an
    // artifact refusal would blame the artifact for it.
    if (!(error instanceof ArtifactComparisonError))
      throw error;

    // TRANSLATED RATHER THAN RETHROWN, so a reader meets one error type from
    // this layer; the sentence is forwarded because its class writes it from
    // names and numbers alone (ledger B34).
    throw new ArtifactParseError({
      path,
      reason: `two ledgers this version can compare (${error.message})`,
    },);
  }
}

/**
 Refuses an artifact whose recorded comparison disagrees with its ledgers.

 @param recorded - comparison the artifact carries

 @param repair - repair lane's ledger

 @param translate - translate lane's ledger

 @param path - dotted path of the recorded comparison

 @returns Comparison derived from the two ledgers, proven equal to the
 recorded one

 @throws {@link ArtifactParseError} when the two ledgers cannot be compared at
 all, when the counts differ, or when any row disagrees

 @example
 ```ts
 const comparison = assertRecordedComparisonMatches({ recorded, repair, translate, path, },);
 ```
 */
export function assertRecordedComparisonMatches(
  {
    recorded,
    repair,
    translate,
    path,
  }: {
    readonly recorded: readonly ArtifactComparisonRow[];
    readonly repair: readonly ArtifactDeliveryRow[];
    readonly translate: readonly ArtifactDeliveryRow[];
    readonly path: string;
  },
): readonly ArtifactComparisonRow[] {
  /**
   What version 2's own rules say about these two ledgers.
   */
  const derived = deriveComparison({
    repair,
    translate,
    path,
  },);
  if (recorded.length !== derived.length) {
    throw new ArtifactParseError({
      path,
      reason: `one row per slice the two ledgers cover, which is ${
        String(derived.length,)
      } here, rather than ${String(recorded.length,)}`,
    },);
  }
  for (const [
    position,
    row,
  ] of derived.entries()) {
    /**
     Row the artifact recorded at the same position; the two hold the same
     number of rows, so there is one.
     */
    const theirs = nonNullishOrThrow(recorded[position],);

    // FIELD BY FIELD rather than by serialized bytes, since these rows came off
    // disk in whatever key order the file wrote them and key order is not part
    // of what a row says.
    /**
     Fields on which the stored row and the derived one disagree. NAMES ONLY:
     the rows carry the archive text and both lanes' output, and this reason
     reaches a marked class whose message every CLI prints.
     */
    const differing = comparisonRowDifferences({
      left: row,
      right: theirs,
    },);
    if (differing.length > 0) {
      throw new ArtifactParseError({
        path: `${path}[${String(position,)}]`,
        reason: `what this version's rules derive for slice ${
          String(row.sliceIndex,)
        } from the ledgers stored beside it; the stored row differs on ${differing.join(', ',)}`,
      },);
    }
  }
  return derived;
}

//endregion Artifact version 2 recorded comparison
