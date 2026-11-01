import {
  type ArtifactDeliveryRow,
  ArtifactParseError,
  outcomesEqual,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  assertDeliveryCoherent,
  DeliveryCoherenceError,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  assertWordingCoherent,
  WordingCoherenceError,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { ArtifactEvidenceRow, } from './artifact-two-lane-read-contract.ts';

//region Artifact version 2 row relations
// What has to hold between a lane's LEDGER and the raw result recorded beside
// it, row by row.
//
// The two are separate derivations of one run. The ledger is what a reader
// compares, because the writer checked it against that lane's own document;
// the raw result is the evidence behind it. Nothing in the file forces them to
// describe the same run, so a reader that took either on trust could report a
// slice the other contradicts.
//
// BY POSITION, never joined on `sliceIndex`. Both lists are in document order
// by contract, so position is the join, and joining on the index instead would
// accept a result whose rows are in some other order while reporting that
// everything matched.
//
// THE COHERENCE CHECKS ARE HERE TOO, and they are a different question from
// parsing: exact parsing says every word in a row is one this version knows,
// and these say the words in one row can be true together. A row saying its
// document carries a shipped replacement while its lane reports having had
// nothing to do there parses cleanly and cannot have happened.

/**
 Says how two disagreeing outcomes differ, in terms that distinguish them.

 NAMING THE KINDS IS NOT ENOUGH when they agree. Two outcomes can carry the
 same member and still disagree, since one member holds wording, and a message
 built from the kinds alone would read `decided rather than decided` and send
 its reader looking for a difference it refused to state. Phrased as what the
 member carries rather than as accepted wording specifically, so it stays true
 of the next member that gains a payload.

 @param raw - member the raw result names

 @param recorded - member the ledger names

 @returns Phrase naming the difference

 @example
 ```ts
 const said = describeOutcomeDisagreement({ raw: 'decided', recorded: 'decided', },);
 ```
 */
function describeOutcomeDisagreement(
  {
    raw,
    recorded,
  }: {
    readonly raw: string;
    readonly recorded: string;
  },
): string {
  return (raw === recorded)
    ? `both name ${raw}, and they differ in what that member carries`
    : `${raw} rather than ${recorded}`;
}

/**
 Refuses a lane whose raw evidence and ledger describe different runs.

 @param evidence - what the lane's raw result says per slice

 @param ledger - what the lane's delivery ledger says per slice

 @param path - dotted path of the lane, for error messages

 @throws {@link ArtifactParseError} when the two lists differ in length, or
 disagree at any position about which slice it is, what the archive holds, or
 what the lane decided

 @example
 ```ts
 assertEvidenceMatchesLedger({ evidence, ledger, path: 'lanes.repair', },);
 ```
 */
export function assertEvidenceMatchesLedger(
  {
    evidence,
    ledger,
    path,
  }: {
    readonly evidence: readonly ArtifactEvidenceRow[];
    readonly ledger: readonly ArtifactDeliveryRow[];
    readonly path: string;
  },
): void {
  if (evidence.length !== ledger.length) {
    throw new ArtifactParseError({
      path,
      reason: `one row per slice in both, and this lane records ${
        String(evidence.length,)
      } raw ${
        wordForCount({
          count: evidence.length,
          one: 'slice',
          many: 'slices',
        },)
      } against ${String(ledger.length,)} ledger ${
        wordForCount({
          count: ledger.length,
          one: 'row',
          many: 'rows',
        },)
      }`,
    },);
  }
  for (const [
    position,
    mine,
  ] of evidence.entries()) {
    /**
     Ledger row at the same POSITION, which is where a ledger built from this
     result holds the same slice; the two hold the same number of rows, so
     there is one.
     */
    const theirs = nonNullishOrThrow(ledger[position],);
    if (mine.sliceIndex !== theirs.sliceIndex) {
      throw new ArtifactParseError({
        path: `${path}.delivery[${String(position,)}].sliceIndex`,
        reason: `slice ${String(mine.sliceIndex,)}, which the raw result names at this position, `
          + `rather than slice ${String(theirs.sliceIndex,)}`,
      },);
    }
    if (mine.incumbentKind !== theirs.incumbentKind) {
      throw new ArtifactParseError({
        path: `${path}.delivery[${String(position,)}].incumbentKind`,
        reason: `${mine.incumbentKind}, as the raw result says of slice ${
          String(mine.sliceIndex,)
        }, rather than ${theirs.incumbentKind}`,
      },);
    }
    if (mine.incumbentText !== theirs.incumbentText) {
      throw new ArtifactParseError({
        path: `${path}.delivery[${String(position,)}].incumbentText`,
        reason: `the archive wording the raw result records for slice ${String(mine.sliceIndex,)}`,
      },);
    }
    if (!outcomesEqual({
      left: mine.outcome,
      right: theirs.outcome,
    },)) {
      /**
       How these two outcomes differ.
       */
      const disagreement = describeOutcomeDisagreement({
        raw: mine.outcome
          .kind,
        recorded: theirs.outcome
          .kind,
      },);

      throw new ArtifactParseError({
        path: `${path}.delivery[${String(position,)}].outcome`,
        reason: `the outcome the raw result records for slice ${
          String(mine.sliceIndex,)
        }, which is ${disagreement}`,
      },);
    }
  }
}

/**
 Refuses a ledger whose rows are not in document order.

 NOT COVERED BY ANY OTHER CHECK HERE, and the reason is the same one that
 makes every check here cheap: they all join BY POSITION. Two ledgers carrying
 the same permutation agree with each other, permuted evidence agrees with its
 permuted ledger, the row count still matches the preparation, and the index
 sets are sets. A ledger of rows naming slices 20 then 10 passes all of that
 and hands a consumer zipping it against the preparation the wrong slice's
 wording, at every row.

 STRICTLY INCREASING, which is the property the writer actually has and the
 weakest one that anchors a positional read. A first version of this checked
 DISTINCTNESS only, on the reasoning that the writer renumbers slices by
 design so a reader must not assume `0` to `length - 1`. That
 reasoning is sound and does not reach this far: renumbering produces GAPS,
 and gaps are still increasing. Distinct-but-permuted was accepted, which is
 the defect this replaces. Measured before choosing: `prepareDocumentPair`
 stamps strictly increasing target indices on every fixture tried, at three
 slice budgets, including a one-sided pair.

 @param ledger - rows to check

 @param path - dotted path of the lane, for error messages

 @throws {@link ArtifactParseError} at the first row that does not advance,
 naming both slices and the position

 @example
 ```ts
 assertSlicesOrdered({ ledger, path: 'lanes.repair', },);
 ```
 */
export function assertSlicesOrdered(
  {
    ledger,
    path,
  }: {
    readonly ledger: readonly ArtifactDeliveryRow[];
    readonly path: string;
  },
): void {
  for (const [
    position,
    row,
  ] of ledger.entries()) {
    /**
     Row before this one, absent at the first position.
     */
    const previous = ledger[position - 1];
    if (previous === undefined)
      continue;

    if (row.sliceIndex <= previous.sliceIndex) {
      throw new ArtifactParseError({
        path: `${path}.delivery[${String(position,)}].sliceIndex`,
        reason: `a slice after ${
          String(previous.sliceIndex,)
        }, which the row before this one names, since a ledger is stated in document order and every `
          + `check here joins by position; this row names ${
            String(row.sliceIndex,)
          }, so the rows are ${
            (row.sliceIndex === previous.sliceIndex) ? 'a repeat' : 'out of order'
          }`,
      },);
    }
  }
}

/**
 Refuses a ledger row whose two axes cannot both be true.

 DELEGATED to the same assertions the writing pipeline runs, rather than
 restated here. They encode which pairs of outcome and delivery can occur
 together, that rule belongs to the shape rather than to either side of it,
 and a second copy would drift from the first exactly when it mattered.

 @param ledger - rows to check

 @param path - dotted path of the lane, for error messages

 @throws {@link ArtifactParseError} carrying the failing row's position and
 whatever the coherence rule said, so a reader never meets an error type from
 the pipeline's internals

 @example
 ```ts
 assertRowsCoherent({ ledger, path: 'lanes.repair', },);
 ```
 */
export function assertRowsCoherent(
  {
    ledger,
    path,
  }: {
    readonly ledger: readonly ArtifactDeliveryRow[];
    readonly path: string;
  },
): void {
  for (const [
    position,
    row,
  ] of ledger.entries()) {
    try {
      // BOTH, because they cover different pairs: one relates what the lane did
      // to what the archive holds, and the other relates what the document
      // carries to both.
      assertWordingCoherent({ wording: row, },);
      assertDeliveryCoherent({ record: row, },);
    } catch (error) {
      /**
       Whether one of the two rules refused this row, as against something
       else failing inside the same `try`.
       */
      const refusedTheRow = (error instanceof WordingCoherenceError)
        || (error instanceof DeliveryCoherenceError);

      // ONLY A COHERENCE REFUSAL DESCRIBES THE FILE. Anything else raised in
      // here is a defect in this reader, and rewriting it as an artifact
      // refusal would blame the artifact for it: an operator reading that
      // message archives a run that was fine and never sees the real fault.
      if (!refusedTheRow)
        throw error;

      throw new ArtifactParseError({
        path: `${path}.delivery[${String(position,)}]`,
        reason: `a row whose outcome and delivery can both be true: ${
          caughtValueText(error,)
        }`,
      },);
    }
  }
}

//endregion Artifact version 2 row relations
