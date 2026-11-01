import {
  ArtifactParseError,
  assertRecordedComparisonMatches,
  requireArray,
  requireCount,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  requireArtifactJsonRecord,
  requireExactKeys,
} from '../artifact-exact-guard.ts';
import {
  assertPreparationIdentity,
  isPreparationIdentityShaped,
  PREPARATION_IDENTITY_SHAPE,
  type PreparationIdentity,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  ARTIFACT_SCHEMA_VERSION_V13,
  artifactGenerationReadingRequirements,
  isTwoLaneArtifactGeneration,
  TWO_LANE_GENERATIONS,
} from './artifact-two-lane-contract.ts';
import {
  keyVocabularyOf,
  type ArtifactKeyVocabulary,
} from '../artifact-key-vocabulary.ts';
import { parseLaneSelection, } from './artifact-two-lane-read-contest.ts';
import {
  requireArchiveAuthority,
  requireArchiveOriginalSpans,
  requireDigest,
} from './artifact-two-lane-read-fields.ts';
import type {
  ParsedTwoLaneArtifact,
  ParsedPreparation,
} from './artifact-two-lane-read-contract.ts';
import { parseLanes, } from './artifact-two-lane-read-lanes.ts';
import { parseConsolidation, } from './artifact-two-lane-read-consolidate.ts';
import { parsePageAssembly, } from './artifact-two-lane-page-assembly.ts';
import { parseConsistency, } from './artifact-two-lane-read-consistency.ts';
import { SETTLED_ARTIFACT_KEYS, } from './artifact-two-lane-read-keys.ts';
import { parseBlockPairing, } from './artifact-two-lane-read-pairing.ts';
import { parseSectionPairing, } from './artifact-two-lane-read-section-pairing.ts';
import { parseComparisonRow, } from './artifact-two-lane-read-rows.ts';

//region Artifact version 2 reading
// Reading one whole version 2 artifact, and refusing everything it cannot
// account for.
//
// WHAT THIS IS NOT is a verification that the run happened as recorded. The
// artifact stores measurements of the two documents rather than the documents,
// so the preparation identity is checked for SYNTAX here and nothing more; a
// reader holding the corpus checkout and the matching pipeline can rebuild the
// preparation and check the rest through a separate entry point that takes one.
// Saying so out loud is the point: a syntax check read as a verification is
// exactly the kind of absence this generation exists to stop recording.
//
// WHAT IT DOES DO is refuse every internal contradiction a file alone can show:
// a key this version does not name, a union member it cannot project, a ledger
// its own raw result disagrees with, an index set its rows do not produce, a
// status the deliveries could not have come from, a row whose two axes cannot
// both be true, and a recorded comparison the ledgers do not derive.

/**
 Reads the branded identity a preparation gives itself.

 @param value - recorded identity

 @param path - dotted path for error message

 @returns Identity, narrowed by the same check a fresh one passes

 @throws {@link ArtifactParseError} when the value is not a string, or not
 shaped like an identity

 @example
 ```ts
 const identity = requireIdentity({ value: preparation.identity, path, },);
 ```
 */
function requireIdentity(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): PreparationIdentity {
  /**
   Recorded string, before it is known to be an identity.
   */
  const held = requireString({
    value,
    path,
  },);

  // REFUSED BY SHAPE, in words this file writes, before the assertion narrows.
  // Forwarding the assertion's message instead quoted the recorded value into
  // the refusal (ledger B34).
  if (!isPreparationIdentityShaped({ value: held, },)) {
    throw new ArtifactParseError({
      path,
      reason: `a preparation identity, ${PREPARATION_IDENTITY_SHAPE}`,
    },);
  }
  assertPreparationIdentity(held,);
  return held;
}

/**
 Reads the slicing both lanes ran over.

 @param value - preparation JSON

 @param path - dotted path for error messages

 @returns Preparation as a reader gets it

 @throws {@link ArtifactParseError} when it carries a key this version does
 not name, or any field is the wrong shape

 @example
 ```ts
 const preparation = parsePreparation({ value: artifact.preparation, path, },);
 ```
 */
function parsePreparation(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ParsedPreparation {
  /**
   Preparation as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'identity',
      'archiveText',
      'sliceCount',
      'frontMatterAuthority',
      'archiveOriginalSpans',
      'sourceChars',
      'targetChars',
      'sourceBytes',
      'alignmentPairCount',
      'alignmentFindings',
      'blockPairing',
      'sectionPairing',
    ],
    path,
  },);

  /**
   Aligned sections this preparation reports, read before the rest because the
   pairing's section indices are bounded by it.
   */
  const alignmentPairCount = requireCount({
    value: record.alignmentPairCount,
    path: `${path}.alignmentPairCount`,
  },);
  return {
    identity: requireIdentity({
      value: record.identity,
      path: `${path}.identity`,
    },),

    // ABSENT AND EMPTY ARE DIFFERENT ANSWERS. A file written before this field
    // existed carries no archive text and cannot claim one; an entry whose
    // English really is empty would carry the empty string. Reading absence as
    // '' would turn "nobody recorded it" into "the entry had none", which is a
    // claim about the corpus rather than about the file.
    archiveText: ((record.archiveText) === undefined)
      ? { kind: 'unrecorded' as const, }
      : {
        kind: 'stored' as const,
        text: requireString({
          value: record.archiveText,
          path: `${path}.archiveText`,
        },),
      },
    sliceCount: requireCount({
      value: record.sliceCount,
      path: `${path}.sliceCount`,
    },),

    // ABSENT MEANS RENDERED. A file written before the archive's front matter
    // stood carries no authority, and so does one whose lanes rendered slice
    // zero; only a standing archive is recorded, so the one value the field
    // can carry is 'archive'.
    ...((record.frontMatterAuthority === undefined)
      ? {}
      : {
        frontMatterAuthority: requireArchiveAuthority({
          value: record.frontMatterAuthority,
          path: `${path}.frontMatterAuthority`,
        },),
      }),

    // ABSENT MEANS NOTHING SEALED, on a file from before generation twelve
    // and on one whose archive carries no such note alike; a recorded list is
    // never empty, so absence is the one way to say it.
    ...((record.archiveOriginalSpans === undefined)
      ? {}
      : {
        archiveOriginalSpans: requireArchiveOriginalSpans({
          value: record.archiveOriginalSpans,
          path: `${path}.archiveOriginalSpans`,
        },),
      }),
    sourceChars: requireCount({
      value: record.sourceChars,
      path: `${path}.sourceChars`,
    },),
    targetChars: requireCount({
      value: record.targetChars,
      path: `${path}.targetChars`,
    },),
    sourceBytes: requireCount({
      value: record.sourceBytes,
      path: `${path}.sourceBytes`,
    },),
    alignmentPairCount,

    // ABSENT AND EMPTY ARE DIFFERENT ANSWERS here for the second time in this
    // record, and for a second reason. A file written before the field carries
    // no pairing; an entry whose roster was asked about every section and
    // committed to nothing carries the empty list. Reading absence as `[]`
    // would turn "nobody was asked" into "asked and agreed nothing".
    blockPairing: parseBlockPairing({
      value: record.blockPairing,
      alignmentPairCount,
      path: `${path}.blockPairing`,
    },),
    sectionPairing: parseSectionPairing({
      value: record.sectionPairing,
      alignmentPairCount,
      path: `${path}.sectionPairing`,
    },),
    alignmentFindings: requireArray({
      value: record.alignmentFindings,
      path: `${path}.alignmentFindings`,
    },)
      .map(function readFinding(
        finding,
        position,
      ): string {
        return requireString({
          value: finding,
          path: `${path}.alignmentFindings[${String(position,)}]`,
        },);
      },),
  };
}

/**
 Reads one two-lane artifact, of any generation that wrote the shape.

 NAMED FOR THE FAMILY, not for one integer. Generations 2, 3 and 4 record the
 same two lanes, the same comparison and the same lane selection, and differ
 only in how the slice-index and change-set keys are spelled;
 `artifact-key-vocabulary.ts` holds that difference and the recorded version
 picks the spelling, so no artifact is ever tried under another's names.

 IT REFUSES EVERY OTHER GENERATION INCLUDING VERSION 1: generic dispatch has
 already happened by the time this is called, so a version 1 artifact arriving
 here is a caller reading the wrong file rather than an old artifact needing
 tolerance.

 @param value - artifact JSON, freshly parsed and still untyped

 @returns Everything the artifact records, with its comparison recomputed

 @throws {@link ArtifactParseError} when the artifact belongs to no
 generation of this shape, when any field is missing or the wrong shape, when
 it carries a key this shape does not name, or when any two of its parts
 contradict each other

 @example
 ```ts
 const artifact = parseSettledTwoLaneArtifact({ value: parseRunJson({ text, from, },), },);
 ```
 */
export function parseSettledTwoLaneArtifact(
  { value, }: { readonly value: unknown; },
): ParsedTwoLaneArtifact {
  /**
   Artifact as a record.
   */
  const artifact = requireRecord({
    value,
    path: 'artifact',
  },);
  /**
   Generation this artifact records, read as a count first so a string or a
   fraction is refused as a malformed version rather than compared against
   two numbers and reported as the wrong generation.
   */
  const version = requireCount({
    value: artifact.artifactSchemaVersion,
    path: 'artifact.artifactSchemaVersion',
  },);
  if (!isTwoLaneArtifactGeneration(version,)) {
    throw new ArtifactParseError({
      path: 'artifact.artifactSchemaVersion',
      reason: `${
        TWO_LANE_GENERATIONS
          .map(String,)
          .join(', ',)
      }, since this reader describes that two-lane shape only and dispatch has already chosen it`,
    },);
  }

  /**
   Known two-lane generation after membership guard.
   */
  const generation = version;
  /**
   Spelling this artifact's own generation gave the three renamed keys.
   */
  const keys: ArtifactKeyVocabulary = keyVocabularyOf({ version, },);
  /**
   Whether this generation records the mark of an outcome the select bench never bought a candidate for.
   */
  const { benchFloorRecorded, } = artifactGenerationReadingRequirements({ generation, },);

  /**
   Entry id, which every nested path is reported under.
   */
  const id = requireString({
    value: artifact.id,
    path: 'artifact.id',
  },);
  requireExactKeys({
    record: artifact,
    allowed: SETTLED_ARTIFACT_KEYS,
    path: id,
  },);

  /**
   Slicing both lanes ran over.
   */
  const preparation = parsePreparation({
    value: artifact.preparation,
    path: `${id}.preparation`,
  },);

  /**
   Both lanes, each checked against its own raw result and its own ledger.
   */
  const lanes = parseLanes({
    value: artifact.lanes,
    preparation,
    path: `${id}.lanes`,
    keys,
    benchFloorRecorded,
  },);

  /**
   The two lanes compared, RECOMPUTED from the ledgers and returned only once
   it matched the copy the file carries. Named here rather than built inline,
   because the contest is checked against it: a selection may only answer the
   slices this comparison says the two lanes left differently worded.
   */
  const comparison = assertRecordedComparisonMatches({
    recorded: requireArray({
      value: artifact.comparison,
      path: `${id}.comparison`,
    },)
      .map(function readRow(
        row,
        position,
      ) {
        return parseComparisonRow({
          value: row,
          path: `${id}.comparison[${String(position,)}]`,
          keys,
          benchFloorRecorded,
        },);
      },),
    repair: lanes.repair
      .delivery,
    translate: lanes.translate
      .delivery,
    path: `${id}.comparison`,
  },);

  /**
   Which lane ships, checked against both the ballots recorded beside it and
   the comparison, so a selection answering the wrong slices is refused
   rather than read. Named here because the consolidation is checked against
   it: the third rendering answers exactly the slices the contest settled.
   */
  const laneSelection = parseLaneSelection({
    value: artifact.laneSelection,
    comparison,
    path: `${id}.laneSelection`,
    keys,
    generation,
  },);
  return {
    artifactSchemaVersion: generation,
    id,
    tip: requireString({
      value: artifact.tip,
      path: `${id}.tip`,
    },),
    pipelineDigest: requireDigest({
      value: artifact.pipelineDigest,
      path: `${id}.pipelineDigest`,
    },),
    corpusSha: requireString({
      value: artifact.corpusSha,
      path: `${id}.corpusSha`,
    },),
    callConfig: requireArtifactJsonRecord({
      value: artifact.callConfig,
      path: `${id}.callConfig`,
    },),
    durationMs: requireCount({
      value: artifact.durationMs,
      path: `${id}.durationMs`,
    },),
    timestamp: requireString({
      value: artifact.timestamp,
      path: `${id}.timestamp`,
    },),
    preparation,
    lanes,
    comparison,
    laneSelection,

    // WHAT THE THIRD RENDERING SETTLED, absent on every artifact written before
    // the field existed. That absence is NAMED rather than defaulted: a reader
    // counting how often the stage declined must not count the whole earlier
    // archive as declines. A settled stage is held to the contest's slices.
    consolidation: parseConsolidation({
      value: artifact.consolidation,
      laneSelection,
      path: `${id}.consolidation`,
      keys,
      ...artifactGenerationReadingRequirements({ generation, }),
    },),

    // WHAT THE PAGE-LEVEL GUARD DID, written from generation thirteen on and
    // empty before it, since no guard ran over those pages.
    pageAssembly: parsePageAssembly({
      value: artifact.pageAssembly,
      path: `${id}.pageAssembly`,
      required: generation
        >= ARTIFACT_SCHEMA_VERSION_V13,
    },),

    // WHAT THE PAGE-LEVEL CONSISTENCY PASS DID, written from generation fifteen
    // on and refused in an older file, which no such pass ever ran over; the
    // generation also says which shape the section has (fifteen's or sixteen's).
    consistency: parseConsistency({
      value: artifact.consistency,
      path: `${id}.consistency`,
      generation,
      sliceIndexes: new Set(comparison.map(function indexOf(row,): number {
        return row.sliceIndex;
      },),),
    },),
  };
}

//endregion Artifact version 2 reading
