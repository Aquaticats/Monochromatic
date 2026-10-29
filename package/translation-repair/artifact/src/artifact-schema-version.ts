import {
  ArtifactParseError,
  requireCount,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  ARTIFACT_SCHEMA_VERSION_V2,
  ARTIFACT_SCHEMA_VERSION_V3,
  ARTIFACT_SCHEMA_VERSION_V4,
  ARTIFACT_SCHEMA_VERSION_V5,
  ARTIFACT_SCHEMA_VERSION_V6,
  ARTIFACT_SCHEMA_VERSION_V7,
  ARTIFACT_SCHEMA_VERSION_V8,
  ARTIFACT_SCHEMA_VERSION_V9,
  ARTIFACT_SCHEMA_VERSION_V10,
  ARTIFACT_SCHEMA_VERSION_V11,
  ARTIFACT_SCHEMA_VERSION_V12,
  ARTIFACT_SCHEMA_VERSION_V13,
  ARTIFACT_SCHEMA_VERSION_V14,
  ARTIFACT_SCHEMA_VERSION_V15,
  ARTIFACT_SCHEMA_VERSION_V16,
  ARTIFACT_SCHEMA_VERSION_V17,
  ARTIFACT_SCHEMA_VERSION_V18,
  ARTIFACT_SCHEMA_VERSION_V19,
  ARTIFACT_SCHEMA_VERSION_V20,
  ARTIFACT_SCHEMA_VERSION_V21,
  ARTIFACT_SCHEMA_VERSION_V22,
} from './corpus-run/artifact-two-lane-contract.ts';

//region Artifact schema version
// What generation a settled artifact belongs to, stated by the writer instead
// of guessed by every reader from which fields happen to be there.
//
// Its own file because the constant is the one thing a WRITER needs from this
// subject and the reading is the one thing a READER needs, and the two are
// imported from opposite ends of the package. Keeping the version history here
// also gives it the same shape the slice cache key's history has, which is the
// only reason that one has stayed honest.

/**
 First schema generation there has ever been.

 NOT WHAT THE PASS WRITES, which is `ARTIFACT_SCHEMA_VERSION_V22`
 (`corpus-run/artifact-two-lane-contract.ts`, ledger D2) and has been
 a two-lane generation since `settleEntry` moved to one. This was called
 `SETTLED_ARTIFACT_SCHEMA_VERSION` and documented as the generation the pass
 writes, which stopped being true at that move; it is renamed rather than
 re-documented because a caller reaching for what the pass writes reaches for
 the name that says SETTLED, and a guard built on that name would refuse every
 artifact this pipeline produces.

 VERSION HISTORY, and the rule that comes with it: every bump records what
 changed and why a reader could not have worked it out from the fields alone.
 A version that does NOT move on a shape change is the failure this field
 exists to end, so say so here when a field is added compatibly.

 VERSIONS 10 TO 21 are recorded at their constants in
 `corpus-run/artifact-two-lane-contract.ts`, each saying what moved from the one
 before; the history here stops at 9 because the constants moved there.

 VERSION 9, 2026-08-28: absolute review may drive two bounded corrections;
 each transition binds rejected text, canonical findings, and gated text digests.

 VERSION 8, 2026-08-28: every final body polish records independent absolute
 naturalness review bound to exact would-ship text and one bounded correction.

 VERSION 7, 2026-08-28: syntax-bearing lane contest records source-backed
 candidate eligibility, so raw votes for unpublishable candidates are kept
 but excluded from effective verdict.

 VERSION 6, 2026-08-28: final post-consolidation body polish became auditable
 per slice. Generation 5 records no naturalness decision after consolidation.

 VERSION 5, 2026-08-28: visible YAML front matter became explicit slice zero.
 Earlier generations sliced body only, so current preparation cannot rebuild
 their slice count or identity unless generation suppresses metadata slice.

 VERSION 4, 2026-08-24: one key renamed and nothing else. `chunkIndex`
 becomes `sliceIndex` on every per-slice record. It is the same rename version
 3 made to the arrays, finishing the job on the field those arrays hold
 indices INTO, and it moved separately because `sliceIndex` was already taken
 by a different concept and had to be freed first.

 VERSION 3, 2026-08-24: three keys renamed and nothing else. Generation 2
 spelled them `shippedChunkIndices`, `withdrawnChunkIndices` and
 `chunkCritics`, in records that already spelled `sliceCount` and
 `withdrawnSliceCount` the other way about the same things. Generation 3
 spells all of them `slice`. A reader could NOT have worked this out from the
 fields alone: both generations carry the same field COUNT under the same
 parents, so a version 2 file read under generation 3 reports three keys as
 absent rather than as differently spelled. `artifact-key-vocabulary.ts` holds
 the two spellings and the recorded version picks one.

 Version 1 is the first version there has ever been, which makes its own
 history the important part. THREE UNVERSIONED GENERATIONS precede it, and a
 reader meeting one has only field presence to go on:

 -   Before 2026-08-14: no `pipelineDigest`, no `sourceBytes`, and no index
     sets. Every artifact settled to date is one of these, measured over the
     164 artifact files on disk on 2026-08-15.
 -   From 2026-08-14: `pipelineDigest` and `sourceBytes`, still no index sets.
 -   From 2026-08-15: `changedSliceIndices` and `withdrawnSliceIndices`, with
     no `sliceCount` to bound them and no version to announce them.

 The middle two generations are EMPTY populations today, since no pass has run
 since either landed. They are named anyway, because a reader that meets one
 must not read it as the generation before.

 COMPATIBLE ADDITIONS WITHIN VERSION 2, which the VERSION HISTORY rule requires saying
 here rather than only at the field. Each adds an OPTIONAL key to
 `preparation` and each parses to a named absence, so a reader that meets the
 key missing understands the artifact completely and refusing the generation
 would buy nothing:

 -   `archiveText`, the entry's English before either lane ran, so a reader
     needs no checkout pinned to the corpus commit. Decided in
     `doc/decision/artifact-stores-the-archive-text.md`.
 -   `blockPairing`, which original block the roster said each translation
     block renders. Decided in
     `doc/decision/artifact-stores-the-block-pairing.md`.

 BOTH ARE EMPTY POPULATIONS on disk: measured 2026-08-21 over the 11 settled
 artifacts in the two run directories, every one is version 2 and carries
 neither key. So the absence branch of each is the only branch any stored
 artifact exercises today, which is a fact about this corpus rather than about
 the format, and is why both branches are tested rather than only the one a
 fixture happens to reach.

 @example
 ```ts
 const artifact = { artifactSchemaVersion: ARTIFACT_SCHEMA_VERSION_V1, };
 ```
 */
export const ARTIFACT_SCHEMA_VERSION_V1 = 1;

/**
 Generations a reader still understands.

 SEPARATE FROM WHAT THE PASS WRITES, and separate on purpose. The two are the
 same list today and will stop being so the moment a second version exists:
 reading the writer's constant to decide what is READABLE means that bumping
 it turns every artifact of the previous generation into a refusal, which is
 the opposite of what a version is for.

 EVERY BUMP DECIDES THIS EXPLICITLY. Add the outgoing version here when a
 reader still understands it, and leave it out when it genuinely cannot. Once
 this holds more than one entry, whatever reads a versioned field has to
 dispatch per version rather than assume the newest shape.

 THAT MOMENT HAS ARRIVED: version 2 is here and its shape shares almost
 nothing with version 1, which recorded one lane at the top level. Version 1
 stays readable, because refusing a generation a reader still understands is
 the opposite of what a version is for, and the empty population on disk is a
 fact about this corpus rather than about the format. What changes is that
 every reader of a versioned field now has to say which versions it handles:
 `readArtifactChangeSets` answers with ONE singular change set per artifact
 and refuses version 2 outright, since a two-lane artifact has no singular
 anything for it to answer with.

 @example
 ```ts
 const readable = KNOWN_ARTIFACT_SCHEMA_VERSIONS.includes(version,);
 ```
 */
export const KNOWN_ARTIFACT_SCHEMA_VERSIONS: readonly number[] = [
  ARTIFACT_SCHEMA_VERSION_V1,
  ARTIFACT_SCHEMA_VERSION_V2,
  ARTIFACT_SCHEMA_VERSION_V3,
  ARTIFACT_SCHEMA_VERSION_V4,
  ARTIFACT_SCHEMA_VERSION_V5,
  ARTIFACT_SCHEMA_VERSION_V6,
  ARTIFACT_SCHEMA_VERSION_V7,
  ARTIFACT_SCHEMA_VERSION_V8,
  ARTIFACT_SCHEMA_VERSION_V9,
  ARTIFACT_SCHEMA_VERSION_V10,
  ARTIFACT_SCHEMA_VERSION_V11,
  ARTIFACT_SCHEMA_VERSION_V12,
  ARTIFACT_SCHEMA_VERSION_V13,
  ARTIFACT_SCHEMA_VERSION_V14,
  ARTIFACT_SCHEMA_VERSION_V15,
  ARTIFACT_SCHEMA_VERSION_V16,
  ARTIFACT_SCHEMA_VERSION_V17,
  ARTIFACT_SCHEMA_VERSION_V18,
  ARTIFACT_SCHEMA_VERSION_V19,
  ARTIFACT_SCHEMA_VERSION_V20,
  ARTIFACT_SCHEMA_VERSION_V21,
  ARTIFACT_SCHEMA_VERSION_V22,
];

/**
 What an artifact says about which generation it belongs to.

 A UNION RATHER THAN AN OPTIONAL NUMBER, deliberately. Absence is the whole
 difficulty this field was added to remove, and an optional field hands the
 difficulty straight back: a consumer writes `version ?? 0` and an unversioned
 artifact becomes a versioned one nobody notices.

 @example
 ```ts
 const reading: ArtifactSchemaReading = { kind: 'unversioned', };
 ```
 */
export type ArtifactSchemaReading = {
  /**
   Artifact names a generation this reader knows.
   */
  readonly kind: 'versioned';

  /**
   Generation it names.
   */
  readonly version: number;
} | {
  /**
   Artifact predates the field, so its generation is legible only from which
   fields it carries.
   */
  readonly kind: 'unversioned';
};

/**
 Reads an artifact's schema generation.

 AN UNKNOWN VERSION IS REFUSED, not tolerated. A reader meeting a generation
 written after it was compiled knows exactly one thing about that artifact:
 that it does not know its shape. Carrying on would parse a field whose
 meaning may have moved, which is how a measurement instrument reports a
 number that is wrong rather than missing.

 @param artifact - artifact record, freshly parsed

 @param path - dotted path for error messages

 @returns Generation it names, or a named absence

 @throws {@link ArtifactParseError} when the field is present but is not a
 count, or names a generation this reader does not know

 @example
 ```ts
 const reading = readArtifactSchemaVersion({ artifact, path: id, },);
 ```
 */
export function readArtifactSchemaVersion(
  {
    artifact,
    path,
  }: {
    readonly artifact: Readonly<Record<string, unknown>>;
    readonly path: string;
  },
): ArtifactSchemaReading {
  if (
    !Object.hasOwn(
      artifact,
      'artifactSchemaVersion',
    )
  ) return { kind: 'unversioned', };

  /**
   Generation the artifact names, checked as a count first so a string or a
   fraction is refused here rather than compared numerically.
   */
  const version = requireCount({
    value: artifact.artifactSchemaVersion,
    path: `${path}.artifactSchemaVersion`,
  },);
  if (!KNOWN_ARTIFACT_SCHEMA_VERSIONS.includes(version,))
    throw new ArtifactParseError({
      path: `${path}.artifactSchemaVersion`,
      reason: `one of schema versions ${
        KNOWN_ARTIFACT_SCHEMA_VERSIONS.join(', ',)
      }, which are the ones this reader knows`,
    },);

  return {
    kind: 'versioned',
    version,
  };
}

//endregion Artifact schema version
