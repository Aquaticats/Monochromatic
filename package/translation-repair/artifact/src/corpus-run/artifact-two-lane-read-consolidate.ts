import { requireExactKeys, } from '../artifact-exact-guard.ts';
import {
  ArtifactParseError,
  requireArray,
  requireRecord,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type {
  ArtifactConsolidateSlice,
  ArtifactConsolidation,
} from './artifact-two-lane-consolidate.ts';
import type { ArtifactLaneSelection, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import { parseConsolidateSlice, } from './artifact-two-lane-read-consolidate-slice.ts';

import type { ArtifactKeyVocabulary, } from '../artifact-key-vocabulary.ts';

//region Artifact version 2 consolidation read
// Reading what the third rendering settled over one document.
//
// ITS OWN FILE for the reason the pairing read is: the whole-artifact reader
// has no room under the file-length limit, and this is a subject of its own,
// with an absence carrying two meanings and a payload that decides what a
// reader ships. Its leaf shapes sit in `artifact-two-lane-read-consolidate-parts.ts`.
//
// EVERY REFUSAL HERE MIRRORS AN INVARIANT THE PRODUCER ALREADY HOLDS, which is
// the only kind worth enforcing on a stored record: `describeConsolidateSlice`
// writes one record per contested slice, in comparison-row order, and carries
// text on exactly the consolidated terminal. The first two are checked against
// the contest the same artifact records, the way the contest itself is checked
// against the comparison; the third is checked per slice.

/**
 What an artifact says about the third rendering over its document.

 THREE STATES, NOT TWO. `not-run` is a pass that chose not to ask; `unrecorded`
 is an artifact written before the field existed, which is every artifact
 settled before this landed. Collapsing them would let a census of how often
 the stage declines count the whole earlier archive as declines.

 @example
 ```ts
 const consolidation: ParsedConsolidation = { kind: 'unrecorded', };
 ```
 */
export type ParsedConsolidation =
  | ArtifactConsolidation
    | {
    /**
     Artifact names no consolidation, which for every artifact settled to
     date means it was written before the field existed.
     */
      readonly kind: 'unrecorded';
    };

/**
 Refuses a settled stage that does not answer exactly the slices the contest
 settled, in the contest's order.

 MIRRORS `assertContestCoversEligible`, and for the same reason: the driver
 walks the comparison rows and writes one record at every row the contest
 answered, so a record missing from that set, or naming a slice outside it,
 or out of its order, was not written by this pipeline, and a consumer keying
 by `sliceIndex` would otherwise read a missing slice as left with what the
 contest settled. Measured before this was written: all 28 artifacts on this
 machine carrying the field, across schema versions 2, 3 and 4, name exactly
 their contest's slices in its order, one of them naming none at all.

 @param slices - records the stage carries

 @param laneSelection - contest the same artifact records, already proven to
 cover the eligible slices; a pending selection settled nothing

 @param path - dotted path of the recorded slices

 @throws {@link ArtifactParseError} naming both lists when they differ

 @example
 ```ts
 assertConsolidationCoversContest({ slices, laneSelection, path, },);
 ```
 */
function assertConsolidationCoversContest(
  {
    slices,
    laneSelection,
    path,
  }: {
    readonly slices: readonly ArtifactConsolidateSlice[];
    readonly laneSelection: ArtifactLaneSelection;
    readonly path: string;
  },
): void {
  /**
   Slices the contest settled, in its order; none where nobody has asked.
   */
  const contested = (laneSelection.kind === 'contested')
    ? laneSelection.slices
      .map(function nameIt(slice,): number {
        return slice.sliceIndex;
      },)
      .join(',',)
    : '';

  /**
   Slices the stage answers.
   */
  const answered = slices
    .map(function nameIt(slice,): number {
      return slice.sliceIndex;
    },)
    .join(',',);
  if (answered !== contested) {
    throw new ArtifactParseError({
      path,
      reason: `slices [${contested}], one per slice the contest settled and in its order, rather than [${answered}]`,
    },);
  }
}

/**
 Reads what the consolidation settled over one document.

 A DUPLICATE SLICE IS REFUSED, following the contest reader for the reason
 it gives: the driver writes one record per contested slice, so two records
 naming one slice are two different answers to the same question, and a
 consumer keying by `sliceIndex` would silently keep whichever it read last.
 A settled stage is then held to exactly the contest's slices, in its order.

 @param value - consolidation field as the artifact carries it

 @param laneSelection - contest the same artifact records, which names the
 slices a settled stage must answer

 @param path - dotted path for error messages

 @param keys - field spellings this artifact's generation uses, so an older
 file is read by its own names rather than by today's

 @param polishRequired - whether generation records final body polish

 @param reviewRequired - whether polish records absolute naturalness review

 @param correctionChainRequired - whether review records digest-bound corrections

 @param everyBodyBlockReviewed - whether reviewed paragraphs include all body blocks

 @param quorumBasisRequired - whether review rounds retain their wider quorum

 @param lastSeatGrace - whether review rounds were decided with the last-seat
 grace of 2026-10-22 (generation twenty on)

 @param tenseRecorded - whether the generation records the tense candidates of the shipped wording

 @param benchFloorRecorded - whether the generation records a slice kept for want of a slate bench

 @param translateWinnerRecorded - whether the generation records why the archive keeps a slice and the shipped kind of
 a translate lane winner

 @returns What the stage settled, that it did not run, or that this artifact
 predates the field

 @throws {@link ArtifactParseError} when the field is the wrong shape, two
 records name one slice, or the records are not the contest's slices in its
 order

 @example
 ```ts
 const consolidation = parseConsolidation({ value: artifact.consolidation, laneSelection, path, keys, },);
 ```
 */
export function parseConsolidation(
  {
    value,
    laneSelection,
    path,
    keys,
    polishRequired = false,
    reviewRequired = false,
    correctionChainRequired = false,
    everyBodyBlockReviewed = false,
    quorumBasisRequired = false,
    lastSeatGrace = false,
    tenseRecorded = false,
    benchFloorRecorded = false,
    translateWinnerRecorded = false,
  }: {
    readonly value: unknown;
    readonly laneSelection: ArtifactLaneSelection;
    readonly path: string;
    readonly keys: ArtifactKeyVocabulary;
    readonly polishRequired?: boolean;
    readonly reviewRequired?: boolean;
    readonly correctionChainRequired?: boolean;
    readonly everyBodyBlockReviewed?: boolean;
    readonly quorumBasisRequired?: boolean;
    readonly lastSeatGrace?: boolean;
    readonly tenseRecorded?: boolean;
    readonly benchFloorRecorded?: boolean;
    readonly translateWinnerRecorded?: boolean;
  },
): ParsedConsolidation {
  if (value === undefined) {
    if (polishRequired) {
      throw new ArtifactParseError({
        path,
        reason: 'recorded consolidation with generation-six polish decisions',
      },);
    }
    return { kind: 'unrecorded', };
  }

  /**
   Consolidation field as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  if (record.kind === 'not-run') {
    requireExactKeys({
      record,
      allowed: ['kind',],
      path,
    },);
    return { kind: 'not-run', };
  }
  if (record.kind !== 'settled') {
    throw new ArtifactParseError({
      path: `${path}.kind`,
      reason: 'one of settled, not-run',
    },);
  }
  requireExactKeys({
    record,
    allowed: [
      'kind',
      'slices',
    ],
    path,
  },);

  /**
   Every consolidated slice, in the order the driver wrote them.
   */
  const slices = requireArray({
    value: record.slices,
    path: `${path}.slices`,
  },)
    .map(function readOne(
      entry,
      at,
    ) {
      return parseConsolidateSlice({
        value: entry,
        path: `${path}.slices[${String(at,)}]`,
        keys,
        polishRequired,
        reviewRequired,
        correctionChainRequired,
        everyBodyBlockReviewed,
        quorumBasisRequired,
        lastSeatGrace,
        tenseRecorded,
        benchFloorRecorded,
        translateWinnerRecorded,
      },);
    },);

  /**
   Slices already named, so a second record naming one is refused where it
   appears rather than after the whole list has been read.
   */
  const named = new Set<number>();
  for (const slice of slices) {
    if (named.has(slice.sliceIndex,)) {
      throw new ArtifactParseError({
        path: `${path}.slices`,
        reason: `one record per slice; slice ${String(slice.sliceIndex,)} appears more than once`,
      },);
    }
    named.add(slice.sliceIndex,);
  }
  assertConsolidationCoversContest({
    slices,
    laneSelection,
    path: `${path}.slices`,
  },);
  return {
    kind: 'settled',
    slices,
  };
}

//endregion Artifact version 2 consolidation read
