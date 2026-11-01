import type { ArtifactContestVerdict, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  type WouldShipSource,
  wouldShipTextPerSlice,
} from './would-ship-text.ts';

//region Final selection completeness
// An archive standing after contest without endorsement is provenance, not
// final semantic approval; the publisher records that as a finding beside the
// tally rather than refusing the page.

/**
 Reports whether contest explicitly endorsed archive fallback.

 @param verdict - contest result for slice

 @returns Whether archive won explicit semantic endorsement

 @example
 ```ts
 const endorsed = archiveWasEndorsed({ verdict: { kind: 'settled-neither', archive: 'endorsed', }, });
 ```
 */
function archiveWasEndorsed(
  { verdict, }: { readonly verdict: ArtifactContestVerdict; },
): boolean {
  return (verdict.kind === 'settled-neither') && (verdict.archive === 'endorsed');
}

/**
 Reports slices whose final selection keeps archive wording without contest
 endorsement, as findings rather than a refusal.

 The contest verdict is recorded evidence, never withholding authority:
 the page ships and the reading judges the recorded non-endorsements
 (doc/planning/translation-repair-no-loop-design.md).

 @param artifact - comparison and deciding stages used by final assembly

 @returns One finding per slice standing without endorsement, empty when none

 @throws Error when the artifact records its contest as other than contested
 or its consolidation as other than settled, which no persisted artifact does:
 `settledEntryArtifact` writes both kinds as literals

 @example
 ```ts
 const findings = finalSelectionFindings({ artifact, });
 ```
 */
export function finalSelectionFindings(
  { artifact, }: { readonly artifact: WouldShipSource; },
): readonly string[] {
  /**
   Final reading by slice index.
   */
  const readings = new Map(wouldShipTextPerSlice({ artifact, },)
    .map(function nameReading(slice,) {
      return [
        slice.sliceIndex,
        slice.reading,
      ] as const;
    },),);

  /**
   Lane-selection stage this artifact carries.
   */
  const { laneSelection, } = artifact;

  // LOUD RATHER THAN READ AS NOTHING TO REPORT. `persistSettledEntry` is the
  // one caller, and the pass hands it only what `settledEntryArtifact` writes,
  // a contested selection. An artifact whose contest never ran stands on no
  // verdict, so reading it as having no unendorsed slice would pass it as
  // reviewed.
  if (laneSelection.kind !== 'contested')
    throw new Error(
      `unreachable: the artifact handed to persistence records its contest as ${laneSelection.kind}, though `
        + 'settledEntryArtifact, its one writer, records it as contested',
    );

  /**
   Consolidation and optional polish records final reading derives from.
   */
  const { consolidation, } = artifact;

  // LOUD FOR THE CONSOLIDATION TOO, for the same reason. Polish wording exists
  // only inside a settled consolidation (`would-ship-text.ts` reads polish only
  // there), and `settledEntryArtifact` writes the kind as `settled`, so a
  // consolidation that never ran stands on no polish verdict at all, and
  // reading it as having none would pass an unreviewed polish as reviewed.
  // `assertFinalNaturalnessComplete` refuses the same state out loud.
  if (consolidation.kind !== 'settled')
    throw new Error(
      `unreachable: the artifact handed to persistence records its consolidation as ${consolidation.kind}, though `
        + 'settledEntryArtifact, its one writer, records it as settled',
    );

  /**
   Contest records, one per slice the contest ran over.
   */
  const contests = laneSelection.slices;

  /**
   Consolidation records the final reading derives from.
   */
  const consolidatedSlices = consolidation.slices;

  /**
   Slices where archive would stand despite no contest endorsement.
   */
  const unresolved = contests.flatMap(function unresolvedArchive(contest,): readonly number[] {
    /**
     Final wording source after consolidation.
     */
    const reading = readings.get(contest.sliceIndex,);
    if (reading?.kind !== 'wording')
      return [];
    if (reading.decidedBy === 'polish') {
      /**
       Initial consolidation result polish rewrote, when stage recorded one.
       */
      const consolidated = consolidatedSlices
        .find(function namesSlice(slice,): boolean {
          return slice.sliceIndex === contest.sliceIndex;
        },);
      if (consolidated?.terminal === 'consolidated')
        return [];
      /**
       Contest verdict deciding whether baseline had prior endorsement.
       */
      const { verdict, } = contest;
      if ((verdict.kind === 'lane-won') || (verdict.kind === 'sole-lane'))
        return [];
      if (archiveWasEndorsed({ verdict, },))
        return [];
      return [contest.sliceIndex,];
    }
    if (reading.decidedBy !== 'archive')
      return [];
    if (archiveWasEndorsed({ verdict: contest.verdict, },))
      return [];
    return [contest.sliceIndex,];
  },);

  return unresolved.map(function toFinding(sliceIndex,): string {
    return `final-selection-unendorsed (slice ${String(sliceIndex,)}):`
      + ' archive wording stands without contest endorsement, recorded as evidence';
  },);
}

//endregion Final selection completeness
