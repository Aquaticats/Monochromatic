import {
  ArtifactParseError,
  requireArray,
  requireCount,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { computeDrawDigest, } from './sample-draw-identity.ts';
import type { GradingCandidate, } from '@monochromatic-dev/translation-repair-house-rule/ts';

//region Sample manifest
// What sat at each sheet position, recorded WHEN THE DRAW HAPPENS.
//
// Without this the join is not merely awkward, it is impossible. Sheet items
// are numbered positions; probe telemetry and checker verdicts are keyed by
// issue id; and the sheets deliberately print no issue id, because a 64-hex
// string on a grading sheet is noise a human has to read past. Re-running the
// draw does not recover the mapping either: the draw is deterministic in its
// seed but not in its POOL, and the pool grows with every entry that settles,
// so a draw taken at fifteen entries stops reproducing as soon as the sixteenth
// lands.
//
// So the mapping exists only in the instant the sheets are written, and if it
// is not written down then, every measurement that needs to join a human grade
// to a machine verdict is lost with it. That includes the whole reason the
// introduced-defect probe records anything.
//
// Identifiers only, no corpus text: this file lives beside the sheets, outside
// git, under the same rule they do.

/**
 One drawn item's identity at its sheet position.

 @example
 ```ts
 const item: SampleManifestItem = { position: 1, entryId: 'Kitten', issueId: 'adjudicated/nap', };
 ```
 */
export type SampleManifestItem = {
  /**
   One-based sheet position, identical on both sheets.
   */
  readonly position: number;

  /**
   Corpus entry the issue came from.
   */
  readonly entryId: string;

  /**
   Adjudicated issue at this position, the key every machine verdict uses.
   */
  readonly issueId: string;
};

/**
 Which built pipeline settled the entries a sample was drawn from.

 A TAGGED ABSENCE rather than an optional string, for the reason this whole
 generation runs on: a manifest written before this field existed cannot claim
 a generation, and reading its silence as any particular one would attribute a
 sample to a pipeline nobody checked. The gap is exactly this, that
 `EligibleEntries` already carries the selection and the digests while the
 manifest wrote neither.

 @example
 ```ts
 const generation: SampleGeneration = { kind: 'recorded', digest, entries: 15, };
 ```
 */
export type SampleGeneration = {
  readonly kind: 'recorded';

  /**
   Digest of the built output that settled every entry in the pool.

   ONE DIGEST FOR THE POOL, not one per entry, because a pool holding two
   generations is refused before a draw can reach it. If that ever stops being
   true this field is the thing that has to grow, and a reader comparing it
   against an artifact will notice before a rate does.
   */
  readonly digest: string;

  /**
   How many entries the pool offered, which is not how many the sample took.

   Kept because a sample of fifty issues drawn from fifteen entries and one
   drawn from ninety are different evidence, and the items alone cannot say
   which, since one entry contributes many issues.
   */
  readonly entries: number;
} | {
  readonly kind: 'unrecorded';

  /**
   Why, so a reader can tell an old manifest from a draw that could not
   determine its pool.
   */
  readonly reason: string;
};

/**
 Everything needed to join a graded sheet back to the run that produced it.

 @example
 ```ts
 const manifest: SampleManifest = buildSampleManifest({ sample, seed, corpusSha, },);
 ```
 */
export type SampleManifest = {
  /**
   Draw seed, so a manifest cannot be paired with another draw's sheets.
   */
  readonly seed: string;

  /**
   Corpus commit the entries were read at.
   */
  readonly corpusSha: string;

  /**
   Fingerprint of this exact draw, absent on manifests written before the
   binding existed.

   Optional rather than defaulted to an empty string, because absence and a
   value are genuinely different states and the scorers act differently on
   them. A manifest drawn before this field existed can still be scored, under
   the weaker seed-and-pin check and a printed note saying so; a manifest that
   carries a digest disagreeing with its own items is malformed and refused.
   */
  readonly drawDigest?: string;

  /**
   Pipeline that settled the entries this sample was drawn from.

   DELIBERATELY OUTSIDE `drawDigest`. That fingerprint binds seed, corpus pin
   and items, and every sheet already drawn is bound by it; folding a new field
   into it would change the digest of manifests whose sheets are already
   graded, and every one of those bindings would break at once. So the
   generation sits beside it and is checked separately.
   */
  readonly generation: SampleGeneration;

  /**
   Items in sheet order.
   */
  readonly items: readonly SampleManifestItem[];
};

/**
 Records what sat at each sheet position.

 @param sample - drawn candidates, in the order both sheets render them

 @param seed - draw seed

 @param corpusSha - pinned corpus commit

 @returns Manifest to write beside the sheets

 @example
 ```ts
 const manifest = buildSampleManifest({ sample, seed, corpusSha, generation, },);
 ```
 */
export function buildSampleManifest(
  {
    sample,
    seed,
    corpusSha,
    generation,
  }: {
    readonly sample: readonly GradingCandidate[];
    readonly seed: string;
    readonly corpusSha: string;
    readonly generation: SampleGeneration;
  },
): SampleManifest {
  /**
   Items in sheet order, which is the order the digest is taken over.
   */
  const items: readonly SampleManifestItem[] = sample.map(function toItem(
    candidate,
    index,
  ): SampleManifestItem {
    return {
      position: index + 1,
      entryId: candidate.entryId,
      issueId: candidate.issueId,
    };
  },);

  return {
    seed,
    corpusSha,
    drawDigest: computeDrawDigest({
      seed,
      corpusSha,
      items,
    },),
    generation,
    items,
  };
}

/**
 Reads a manifest's generation, naming its absence rather than guessing one.

 @param manifest - manifest as a record

 @returns Recorded generation, or why there is none

 @throws {@link ArtifactParseError} when a present generation is malformed,
 since a half-written one is worse than none: it would be read as evidence

 @example
 ```ts
 const generation = readGeneration({ manifest, },);
 ```
 */
function readGeneration(
  { manifest, }: { readonly manifest: Readonly<Record<string, unknown>>; },
): SampleGeneration {
  if (manifest.generation === undefined)
    return {
      kind: 'unrecorded',
      reason: 'manifest predates the generation field',
    };

  /**
   Generation as a record.
   */
  const generation = requireRecord({
    value: manifest.generation,
    path: 'manifest.generation',
  },);

  if (generation.kind === 'unrecorded')
    return {
      kind: 'unrecorded',
      reason: requireString({
        value: generation.reason,
        path: 'manifest.generation.reason',
      },),
    };

  return {
    kind: 'recorded',
    digest: requireString({
      value: generation.digest,
      path: 'manifest.generation.digest',
    },),
    entries: requireCount({
      value: generation.entries,
      path: 'manifest.generation.entries',
    },),
  };
}

/**
 Reads a manifest back, throwing rather than skipping a malformed item.

 Strict for the reason `artifact-read.ts` is strict: a dropped item shifts
 every later position silently, which turns a join into a mislabelling rather
 than a gap anyone would notice.

 @param value - parsed manifest JSON

 @returns Manifest as written

 @throws {@link ArtifactParseError} when any field is malformed

 @example
 ```ts
 const manifest = parseSampleManifest({ value: parseRunJson({ text, from, },), },);
 ```
 */
export function parseSampleManifest(
  { value, }: { readonly value: unknown; },
): SampleManifest {
  /**
   Manifest as a record.
   */
  const manifest = requireRecord({
    value,
    path: 'manifest',
  },);

  /**
   Draw seed as written.
   */
  const seed = requireString({
    value: manifest.seed,
    path: 'manifest.seed',
  },);

  /**
   Corpus commit as written.
   */
  const corpusSha = requireString({
    value: manifest.corpusSha,
    path: 'manifest.corpusSha',
  },);

  /**
   Items in the order they were written, which is the order they are scored.
   */
  const items = requireArray({
    value: manifest.items,
    path: 'manifest.items',
  },)
    .map(function toItem(
      entry,
      index,
    ): SampleManifestItem {
      /**
       Item as a record.
       */
      const item = requireRecord({
        value: entry,
        path: `manifest.items[${String(index,)}]`,
      },);

      /**
       Position as written, before it is checked against where it sits.
       */
      const position = requireCount({
        value: item.position,
        path: `manifest.items[${String(index,)}].position`,
      },);

      // Both scorers read grades off the sheet by ARRAY ORDER and then take
      // the issue id from the item at the same index, so a manifest whose
      // recorded position disagrees with where the item sits describes one
      // join while the code performs another. `requireCount` alone admits
      // zero and admits any ordering, so nothing else here would notice.
      if (position !== (index + 1))
        throw new ArtifactParseError({
          path: `manifest.items[${String(index,)}].position`,
          reason: `${String(index + 1,)}, matching where the item sits, not ${
            String(position,)
          }`,
        },);

      return {
        position,
        entryId: requireString({
          value: item.entryId,
          path: `manifest.items[${String(index,)}].entryId`,
        },),
        issueId: requireString({
          value: item.issueId,
          path: `manifest.items[${String(index,)}].issueId`,
        },),
      };
    },);

  /**
   Which pipeline settled the pool, or a named absence.
   */
  const generation = readGeneration({ manifest, },);

  if (manifest.drawDigest === undefined)
    return {
      seed,
      corpusSha,
      generation,
      items,
    };

  /**
   Digest as the manifest declares it.
   */
  const declared = requireString({
    value: manifest.drawDigest,
    path: 'manifest.drawDigest',
  },);

  /**
   Digest these items actually produce.
   */
  const recomputed = computeDrawDigest({
    seed,
    corpusSha,
    items,
  },);

  // Recomputed rather than trusted. A stored digest that is never checked
  // against its own contents proves only that two files carry the same string:
  // edit the items and leave the digest alone, and a sheet carrying the stale
  // digest still matches. The digest has to be a fact about the items.
  if (declared !== recomputed)
    throw new ArtifactParseError({
      path: 'manifest.drawDigest',
      reason: `${recomputed}, the digest of the items recorded beside it, not ${declared}`,
    },);

  return {
    seed,
    corpusSha,
    drawDigest: declared,
    generation,
    items,
  };
}

//endregion Sample manifest
