import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';

import type { SettledArtifact, } from '@monochromatic-dev/translation-repair-artifact/ts';
import { contestStandingMayShip, } from '@monochromatic-dev/translation-repair-consolidate/ts';
import type { ArtifactDeliveryRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  type WouldShipReading,
  wouldShipTextPerSlice,
} from '@monochromatic-dev/translation-repair-page-check/ts';

//region Settled tally
// The one line a settled entry prints, once BOTH lanes have run.
//
// The version 1 line reported one lane's status as THE status, because there
// was one lane: `status=repaired issues=4 accepted=3`. Keeping that shape and
// appending translate counts beside it would say, in every log a later reader
// greps, that the repair lane is the run's outcome and the other lane is
// commentary. That is the question this generation exists to leave open.
//
// So the top-level `status` is the PASS's own state, settled or ERROR, which is
// what an operator scanning a run actually wants from it, and every lane
// measurement carries its lane in the key. `selection=` says out loud that
// nobody has picked a winner, matching what the artifact records, so a reader
// who greps the logs and a reader who reads the artifacts learn the same thing.
//
// EVERYTHING IS READ OFF THE ARTIFACT, not recomputed beside it. A log line
// that disagrees with the file it describes is worse than no log line, and the
// only way to be sure is to have one source.

/**
 One slice as the page carries it, beside the archive wording it replaces.
 */
type ShippedSlice = {
  /**
   What the page carries at this slice.
   */
  readonly reading: WouldShipReading;

  /**
   Archive wording at this slice.
   */
  readonly incumbentText: string;
};

/**
 Counts slices whose delivery carries a change.

 @param rows - one lane's delivery ledger, as version 2 records it

 @returns How many slices that lane's document carries a replacement for

 @example
 ```ts
 const changed = changedSlices({ rows: artifact.lanes.repair.delivery, },);
 ```
 */
function changedSlices(
  { rows, }: { readonly rows: readonly ArtifactDeliveryRow[]; },
): number {
  /**
   Rows whose document carries a replacement.
   */
  const changed = rows.filter(function carriesAChange(row,): boolean {
    return row.delivery
      .kind
      === 'replacement-shipped';
  },);
  return changed.length;
}

/**
 Counts the consolidation slices that kept a standing text the contest never
 endorsed, which the stage logs as "remains retryable" and no other field of
 the line carried.

 READ OFF THE RECORD, so it is the slices the stage's own rule names: the
 contest backed no lane and did not endorse the archive
 (`contestStandingMayShip`), and the consolidation did not write fresh wording
 over the standing. A slice that kept the archive for want of any valid
 wording is the defects line's, and is left out. The stage's log lines are a
 little more than this count, since the standing verdict warns before the
 slate is judged and a slice the slate then rewrote has been warned about too
 (XingZ6020: 58 lines, 56 slices in this count).

 @param artifact - settled entry whose contest and consolidation records are read

 @returns Slices whose consolidation kept an unendorsed standing, none when the consolidation did not run

 @example
 ```ts
 const kept = standingsKeptUnendorsed({ artifact, },);
 ```
 */
function standingsKeptUnendorsed(
  { artifact, }: { readonly artifact: SettledArtifact; },
): number {
  /**
   What the stages left on record, either of which may not have run.
   */
  const {
    consolidation,
    laneSelection,
  } = artifact;
  if ((consolidation.kind !== 'settled') || (laneSelection.kind !== 'contested'))
    return 0;

  /**
   Slices the contest ran over, by index.
   */
  const contests = new Map(laneSelection.slices
    .map(function toEntry(contest,) {
      return [
        contest.sliceIndex,
        contest.verdict,
      ] as const;
    },),);
  return consolidation.slices
    .filter(function keptUnendorsed(slice,): boolean {
      if ((slice.terminal === 'consolidated') || (slice.shipped.kind === 'archive'))
        return false;
      /**
       What the contest recorded here, absent for a slice it never ran over.
       */
      const verdict = contests.get(slice.sliceIndex,);
      if (verdict === undefined)
        return false;
      return !contestStandingMayShip({
        // A LANE BACKED BY THE CONTEST IS ENDORSED; the choice is the verdict's lane or neither, as the driver reads it.
        choice: ((verdict.kind === 'lane-won') || (verdict.kind === 'sole-lane')) ? verdict.lane : 'neither',
        verdict,
      },);
    },)
    .length;
}

/**
 Renders the TALLY line for one settled entry.

 @param artifact - what was written for this entry, which supplies every
 number here rather than being recounted beside it

 @returns Single line, no trailing newline

 @example
 ```ts
 console.log(settledTallyLine({ artifact, },),);
 ```
 */
export function settledTallyLine(
  { artifact, }: { readonly artifact: SettledArtifact; },
): string {
  /**
   Both lanes as the artifact nests them.
   */
  const {
    repair,
    translate,
  } = artifact.lanes;

  /**
   Issues the adjudication accepted, which is a repair-lane measurement and
   is named as one: the translate lane files no issues at all.
   */
  const accepted = repair.result
    .issues
    .filter(function isAccepted(record,): boolean {
      return record.issue
        .status
        === 'accepted';
    },);

  /**
   Accepted issues the checkers confirmed fixed.
   */
  const resolved = accepted.filter(function isResolved(record,): boolean {
    return record.resolved;
  },);

  /**
   Slices where the two documents ended up carrying different words, which is
   the number worth watching: it is how much of this entry the open question
   actually covers.
   */
  const differing = artifact.comparison
    .filter(function documentsDiffer(row,): boolean {
      return row.repairText !== row.translateText;
    },);

  /**
   Archive wording per slice, which each page reading is compared against.
   */
  const incumbents = new Map(artifact.comparison
    .map(function toEntry(row,): readonly [
      number,
      string,
    ] {
      return [
        row.sliceIndex,
        row.incumbentText,
      ];
    },),);

  /**
   Every slice as the page carries it, in comparison-row order.
   */
  const shipped: readonly ShippedSlice[] = wouldShipTextPerSlice({ artifact, },)
    .map(function besideIncumbent(
      {
        sliceIndex,
        reading,
      },
    ): ShippedSlice {
      return {
        reading,
        // Every reading comes from a comparison row, so its slice is keyed.
        incumbentText: nonNullishOrThrow(incumbents.get(sliceIndex,),),
      };
    },);

  /**
   Slices where a document assembled today would carry wording the archive
   did not.

   ADDED BESIDE the two lane counts rather than replacing either, per this
   task's decision 1. `repairChanged` and `translateChanged` say what each
   lane PROPOSED, which stays true however the deciders later ruled; this
   says how much of the entry a reader would actually meet as new. Without
   it a reader gauging how much an entry changed misses the consolidation
   entirely, and on an entry nobody has decided reads two sets of proposals
   as the outcome.

   READ AS THE PAGE CARRIES IT, after the archive's typography (ledger A10):
   counting the reading before typography called a wording that differed from
   the archive only in its quote style a change, and hulicaijia31 logged 34
   changed slices where its page carried 31.

   ZERO IS THE HONEST ANSWER on an undecided entry, and it is meant to be
   read beside `selection=pending-human-decision` on the same line: two
   lanes proposed changes and, as things stand, a document would carry none
   of them. That is the pending decision, stated in the log rather than left
   to inference.
   */
  const pageChanged = shipped.filter(function pageCarriesAChange(
    {
      reading,
      incumbentText,
    }: ShippedSlice,
  ): boolean {
    return (reading.kind === 'wording') && (reading.text !== incumbentText);
  },);

  /**
   Slices where nothing at all would stand, which is neither a change nor a
   retention and would be invisible inside either count.
   */
  const pageSilent = shipped.filter(function pageCarriesNothing({ reading, }: ShippedSlice,): boolean {
    return reading.kind === 'nothing-ships';
  },);

  /**
   Slices whose page carries the original text of a passage nobody translated, under a note (the owner,
   2026-10-23, "Source text + note").

   COUNTED BESIDE `pageSilent` AND NOT INSIDE IT, because the two say opposite things: a silent slice is a
   passage the page does not carry at all, and this is one it carries untranslated. Folding them together
   would have kept a run that ships its whole tail as the original reading as a run that lost the tail,
   and a reader of the line could not tell the gap from the page that names it. A later run replaces these
   once a wording is backed.
   */
  const pageSourceShipped = shipped.filter(function pageCarriesTheOriginal({ reading, }: ShippedSlice,): boolean {
    return reading.kind === 'source-ships';
  },);

  return [
    `TALLY ${artifact.id}`,
    'status=SETTLED',
    `slices=${String(artifact.preparation
      .sliceCount,)}`,
    `repairStatus=${repair.result
      .status}`,
    `repairIssues=${String(repair.result
      .issues
      .length,)}`,
    `repairAccepted=${String(accepted.length,)}`,
    `repairResolved=${String(resolved.length,)}`,
    `repairFindings=${String(repair.result
      .findings
      .length,)}`,
    `repairChanged=${String(changedSlices({ rows: repair.delivery, },),)}`,
    `translateStatus=${translate.result
      .status}`,
    `translateChanged=${String(changedSlices({ rows: translate.delivery, },),)}`,
    `documentsDiffer=${String(differing.length,)}`,
    `pageChanged=${String(pageChanged.length,)}`,
    `pageSilent=${String(pageSilent.length,)}`,
    `pageSourceShipped=${String(pageSourceShipped.length,)}`,
    `standingUnendorsed=${String(standingsKeptUnendorsed({ artifact, },),)}`,
    `alignmentFindings=${String(artifact.preparation
      .alignmentFindings
      .length,)}`,
    `selection=${artifact.laneSelection
      .kind}`,
    `ms=${String(artifact.durationMs,)}`,
  ].join(' ',);
}

//endregion Settled tally
