import {
  requireExactKeys,
  requireIndexPairList,
} from '../artifact-exact-guard.ts';
import {
  ArtifactParseError,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { ArtifactSectionCorrespondence, } from './artifact-two-lane-contract.ts';

//region Artifact version 2 section pairing read
// Reading which decider chose a settled preparation's aligned sections.
//
// ITS OWN FILE beside `artifact-two-lane-read-pairing.ts`, which reads the
// block pairing WITHIN each aligned section and has its own file for the same
// reason: one subject per sibling, and the whole-preparation reader has no
// room left under the file-length limit.
//
// THREE ANSWERS, NOT TWO. The block pairing reader names one absence, because
// there absence means one thing. Here the deterministic aligner deciding the
// sections is the ordinary production case, so an artifact written after the
// field says so explicitly, and only a file written before the field is
// reported as recording nothing.
//
// EVERY REFUSAL MIRRORS AN INVARIANT THE PRODUCER ALREADY HOLDS. `agreePairs`
// keeps a section pairing strictly increasing on both sides and the wire reader
// bounds every index against the section counts, so a stored pairing that runs
// backwards, stands still, or names more pairs than the preparation aligned
// was not written by this pipeline.

/**
 What an artifact says about how its aligned sections were decided.

 @example
 ```ts
 const pairing: ParsedSectionPairing = { kind: 'deterministic', };
 ```
 */
export type ParsedSectionPairing = {
  /**
   The deterministic aligner chose the sections; no pairing was supplied.
   */
  readonly kind: 'deterministic';
} | {
  /**
   A supplied pairing chose them, and these are its pairs.
   */
  readonly kind: 'supplied';

  /**
   Correspondences it committed to, in document order.
   */
  readonly pairs: readonly ArtifactSectionCorrespondence[];
} | {
  /**
   Artifact names no decider, which means it was written before the field
   existed: a rebuild from it can only assume the deterministic aligner.
   */
  readonly kind: 'unrecorded';
};

/**
 Refuses a pairing no section round could have agreed.

 STRICT ON BOTH SIDES, unlike the block pairing's check: a block pairing
 carries splits and merges, but a section pairing is one target per source
 and `agreePairs` drops any target that does not advance, so standing still
 on either side is a shape the producer never emits.

 @param pairs - pairs as recorded

 @param path - dotted path for error messages

 @throws {@link ArtifactParseError} naming the first position that breaks
 the order the producer guarantees

 @example
 ```ts
 assertPairsClimb({ pairs, path, },);
 ```
 */
function assertPairsClimb(
  {
    pairs,
    path,
  }: {
    readonly pairs: readonly ArtifactSectionCorrespondence[];
    readonly path: string;
  },
): void {
  for (const [at, pair,] of pairs.entries()) {
    /**
     Pair before this one, absent at the first position.
     */
    const previous = pairs[at - 1];
    if (previous === undefined)
      continue;
    if ((pair.source <= previous.source) || (pair.target <= previous.target))
      throw new ArtifactParseError({
        path: `${path}[${String(at,)}]`,
        reason: `a pair strictly after ${String(previous.source,)},${
          String(previous.target,)
        } on both sides, since a section pairing names one translation section per original and keeps document order`,
      },);
  }
}

/**
 Reads which decider chose a preparation's aligned sections, or its absence.

 @param value - `sectionPairing` as the artifact carries it, possibly absent

 @param alignmentPairCount - aligned sections this preparation reports, which
 a supplied pairing cannot exceed: every pair it names becomes one aligned
 section, and insertions only add to the count

 @param path - dotted path for error messages

 @returns Decider it records, with the pairs where one was supplied, or a
 named absence

 @throws {@link ArtifactParseError} when the record is the wrong shape, names
 a decider this version does not know, carries pairs beside the deterministic
 decider or none beside the supplied one, names more pairs than sections were
 aligned, or records a pairing no section round could have agreed

 @example
 ```ts
 const pairing = parseSectionPairing({ value: record.sectionPairing, alignmentPairCount, path, },);
 ```
 */
export function parseSectionPairing(
  {
    value,
    alignmentPairCount,
    path,
  }: {
    readonly value: unknown;
    readonly alignmentPairCount: number;
    readonly path: string;
  },
): ParsedSectionPairing {
  if (value === undefined)
    return { kind: 'unrecorded', };

  /**
   Decider record as written.
   */
  const record = requireRecord({
    value,
    path,
  },);

  /**
   Which decider it names.
   */
  const kind = requireString({
    value: record.kind,
    path: `${path}.kind`,
  },);
  if (kind === 'deterministic') {
    requireExactKeys({
      record,
      allowed: ['kind',],
      path,
    },);
    return { kind: 'deterministic', };
  }
  if (kind !== 'supplied')
    throw new ArtifactParseError({
      path: `${path}.kind`,
      reason: `deterministic or supplied, which are the deciders this version names, rather than ${kind}`,
    },);
  requireExactKeys({
    record,
    allowed: [
      'kind',
      'pairs',
    ],
    path,
  },);

  /**
   Pairs the supplied pairing committed to.
   */
  const pairs = requireIndexPairList({
    value: record.pairs,
    path: `${path}.pairs`,
  },);

  // BOUNDED BY THE COUNT SITTING BESIDE IT, the way the block pairing's
  // section indices are: each supplied pair becomes exactly one aligned
  // section, so a pairing longer than the alignment describes some other pair
  // of documents.
  if (pairs.length > alignmentPairCount)
    throw new ArtifactParseError({
      path: `${path}.pairs`,
      reason: `at most ${String(alignmentPairCount,)} ${
        wordForCount({
          count: alignmentPairCount,
          one: 'pair',
          many: 'pairs',
        },)
      }, which is how many sections this preparation aligned`,
    },);
  assertPairsClimb({
    pairs,
    path: `${path}.pairs`,
  },);
  return {
    kind: 'supplied',
    pairs,
  };
}

//endregion Artifact version 2 section pairing read
