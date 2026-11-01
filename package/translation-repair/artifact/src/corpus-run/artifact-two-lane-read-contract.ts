import type { ArtifactConsistency, } from './artifact-two-lane-consistency.ts';
import type { ArtifactPageAssembly, } from './artifact-two-lane-page-assembly.ts';
import type {
  ArchiveOriginalSpan,
  PreparationIdentity,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ArtifactLaneSelection, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import type { ParsedConsolidation, } from './artifact-two-lane-read-consolidate.ts';
import type {
  ArtifactJsonValue,
  TwoLaneArtifactGeneration,
} from './artifact-two-lane-contract.ts';
import type { ParsedBlockPairing, } from './artifact-two-lane-read-pairing.ts';
import type { ParsedSectionPairing, } from './artifact-two-lane-read-section-pairing.ts';
import type {
  ArtifactComparisonRow,
  ArtifactDeliveryRow,
  ArtifactSliceOutcome,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Artifact version 2 read contract
// What a reader gets back from a version 2 artifact, and what it requires of
// the two fields the schema leaves open.
//
// NOT `SettledArtifact`, which is the WRITER's type. Its two raw lane results
// are typed by the live pipeline shapes, so a reader returning it would promise
// callers that a file written months ago satisfies today's `RepairTranslationResult`.
// It does not, and it never had to: those results are evidence, they grow by
// addition, and version 2 requires only the part of them it checks.
//
// THE EVIDENCE CORE IS THAT PART, and it is why this file exists. Without it
// the meaning of version 2 is whatever the current TypeScript types say, which
// is the same drift the frozen vocabulary and the frozen comparison already
// closed on the other two axes. With it, a version 2 artifact is readable
// forever by a reader that never imports a live pipeline type: the whole raw
// record comes back untouched for whoever wants more, and everything this
// reader CHECKS is named here.
//
// THE TWO LANE STATUSES ARE LITERAL COPIES, not `RepairStatus` or
// `TranslateDocumentResult['status']`. Importing either would let a live union
// gaining a member silently widen what version 2 accepts, which is exactly the
// drift the vocabulary file exists to stop, reintroduced on the read side.

/**
 How a repair run ended, as version 2 froze it.

 @example
 ```ts
 const status: ArtifactRepairStatus = 'blocked-non-translation';
 ```
 */
export type ArtifactRepairStatus =
  | 'repaired'
    | 'unchanged'
    | 'blocked-non-translation';

/**
 Whether a translate run produced a whole translation, as version 2 froze it.

 @example
 ```ts
 const status: ArtifactTranslateStatus = 'unfilled';
 ```
 */
export type ArtifactTranslateStatus =
  | 'complete'
    | 'unfilled';

/**
 One slice as a lane's RAW result describes it.

 The evidence behind one ledger row, parsed out of the open raw record and
 checked against that row by position. Deliberately the same four fields the
 ledger repeats, since the check is whether the two agree; everything else the
 raw row carries stays in the raw record, unread and unrequired.

 @example
 ```ts
 const row: ArtifactEvidenceRow = { sliceIndex: 0, incumbentKind: 'present', ... };
 ```
 */
export type ArtifactEvidenceRow = {
  /**
   Global slice index this row is for.
   */
  readonly sliceIndex: number;

  /**
   Whether the archive holds any wording at this slice.
   */
  readonly incumbentKind: 'present' | 'absent';

  /**
   Archive's own English for it.
   */
  readonly incumbentText: string;

  /**
   What the lane did about it.
   */
  readonly outcome: ArtifactSliceOutcome;
};

/**
 What version 2 requires of the repair lane's raw result.

 @example
 ```ts
 const evidence: ArtifactRepairEvidence = { status: 'unchanged', sliceCount: 2, ... };
 ```
 */
export type ArtifactRepairEvidence = {
  /**
   How the run ended, which the blocked-compatibility check reads.
   */
  readonly status: ArtifactRepairStatus;

  /**
   Slices the preparation produced, which every index here is out of.
   */
  readonly sliceCount: number;

  /**
   Slices the returned document carries a repair for.
   */
  readonly changedSliceIndices: readonly number[];

  /**
   Slices whose repair the assembly guard took back.
   */
  readonly withdrawnSliceIndices: readonly number[];

  /**
   What the lane decided for every prepared slice, in document order.
   */
  readonly sliceTexts: readonly ArtifactEvidenceRow[];
};

/**
 What version 2 requires of the translate lane's raw result.

 Carries two counts the repair result has no equivalent of, and they are here
 because they are CHECKABLE: each equals the length of a list stored beside
 it, so a result whose count and list disagree is caught rather than believed.

 @example
 ```ts
 const evidence: ArtifactTranslateEvidence = { status: 'complete', sliceCount: 2, ... };
 ```
 */
export type ArtifactTranslateEvidence = {
  /**
   Whether the document is a whole translation.
   */
  readonly status: ArtifactTranslateStatus;

  /**
   Slices the preparation produced.
   */
  readonly sliceCount: number;

  /**
   Slices whose accepted text shipped, which the shipped list also names.
   */
  readonly changedSliceCount: number;

  /**
   Slices whose replacement was withdrawn, which the withdrawn list also
   names.
   */
  readonly withdrawnSliceCount: number;

  /**
   Slices the returned document carries a replacement for.
   */
  readonly changedSliceIndices: readonly number[];

  /**
   Slices whose replacement the assembly guard took back.
   */
  readonly withdrawnSliceIndices: readonly number[];

  /**
   What the lane decided for every prepared slice, in document order.
   */
  readonly sliceTexts: readonly ArtifactEvidenceRow[];
};

/**
 One lane as a reader gets it: the whole raw record, what version 2 requires
 of it, and the ledger.

 THE RAW RECORD IS RETURNED rather than discarded once the core is out of it,
 because a reader that wanted a field this version does not check should get
 it from the artifact rather than from a later generation of this parser.

 @example
 ```ts
 const lane: ParsedLane<ArtifactRepairEvidence> = { raw, evidence, delivery, };
 ```
 */
export type ParsedLane<TEvidence,> = {
  /**
   Exactly what the artifact holds under this lane's `result`, unread beyond
   the core and never narrowed to a live pipeline type.
   */
  readonly raw: Readonly<Record<string, unknown>>;

  /**
   The part of that record version 2 requires and checks.
   */
  readonly evidence: TEvidence;

  /**
   One row per prepared slice, saying what this lane's document carries.
   */
  readonly delivery: readonly ArtifactDeliveryRow[];
};

/**
 Archive English an artifact carries, or a positive statement that it carries
 none.

 A TAGGED ABSENCE RATHER THAN AN OPTIONAL STRING, for the reason
 `sample-manifest.ts` gives for the same shape: a file written before the
 field existed cannot claim a value, and reading its silence as the empty
 string would say the entry HAD no English, which is a different and false
 claim. `unrecorded` says the file predates the field; `stored` says what it
 held.

 @example
 ```ts
 const archive: ParsedArchiveText = { kind: 'unrecorded', };
 ```
 */
export type ParsedArchiveText = {
  readonly kind: 'stored';

  /**
   Archive English of the whole entry, verbatim.
   */
  readonly text: string;
} | {
  readonly kind: 'unrecorded';
};

/**
 The slicing both lanes ran over, as a reader gets it.

 @example
 ```ts
 const preparation: ParsedPreparation = { identity, sliceCount: 12, ... };
 ```
 */
export type ParsedPreparation = {
  /**
   Archive English this file carries, or a statement that it carries none.
   */
  readonly archiveText: ParsedArchiveText;

  /**
   Name this slicing gives itself, checked for SYNTAX and nothing more: the
   inputs it hashes are not in the file, so a standalone reader cannot
   recompute it. Verification against a real preparation is a separate entry
   point that takes one.
   */
  readonly identity: PreparationIdentity;

  /**
   Slices it produced.
   */
  readonly sliceCount: number;

  /**
   Present when the file records a standing archive front matter and so no
   metadata slice; absent on every earlier generation and on rendered ones.
   */
  readonly frontMatterAuthority?: 'archive';

  /**
   Present when the file records spans the archive's note sealed as the
   English original; absent on every earlier generation and where nothing
   was sealed.
   */
  readonly archiveOriginalSpans?: readonly ArchiveOriginalSpan[];

  /**
   Original document length in UTF-16 code units.
   */
  readonly sourceChars: number;

  /**
   Archive translation length, on the same measure.
   */
  readonly targetChars: number;

  /**
   Original document length in UTF-8 bytes, which is the band input.
   */
  readonly sourceBytes: number;

  /**
   Section pairs alignment produced.
   */
  readonly alignmentPairCount: number;

  /**
   What alignment observed about the two documents' structure.
   */
  readonly alignmentFindings: readonly string[];

  /**
   Pairing this slicing was built on, or a statement that the file records
   none.
   */
  readonly blockPairing: ParsedBlockPairing;

  /**
   Which decider chose the aligned sections, or a statement that the file
   records none.
   */
  readonly sectionPairing: ParsedSectionPairing;
};

/**
 One version 2 artifact as a reader gets it.

 NO SINGULAR STATUS, output, winner, issue list or change set, which version 1
 had and this generation deliberately does not: both lanes are here, neither
 is the output, and a reader wanting one answer has to say which lane it means.

 @example
 ```ts
 const artifact: ParsedTwoLaneArtifact = parseSettledTwoLaneArtifact({ value, },);
 ```
 */
export type ParsedTwoLaneArtifact = {
  /**
   Schema generation selecting preparation semantics and key vocabulary.
   */
  readonly artifactSchemaVersion: TwoLaneArtifactGeneration;

  /**
   Corpus entry this covers.
   */
  readonly id: string;

  /**
   Repository head when the pass started, as provenance.
   */
  readonly tip: string;

  /**
   Built output that ran, as identity.
   */
  readonly pipelineDigest: PipelineDigest;

  /**
   Corpus commit the two texts were read at.
   */
  readonly corpusSha: string;

  /**
   Model call configuration, whose inside this schema does not describe.
   */
  readonly callConfig: Readonly<Record<string, ArtifactJsonValue>>;

  /**
   Time the entry took, on the writer's monotonic clock.
   */
  readonly durationMs: number;

  /**
   When the artifact was written.
   */
  readonly timestamp: string;

  /**
   Slicing both lanes ran over.
   */
  readonly preparation: ParsedPreparation;

  /**
   The two lanes, neither of them the output.
   */
  readonly lanes: {
    /**
     Lane that mends the archive's English.
     */
    readonly repair: ParsedLane<ArtifactRepairEvidence>;

    /**
     Lane that renders every slice afresh from the original.
     */
    readonly translate: ParsedLane<ArtifactTranslateEvidence>;
  };

  /**
   The two lanes compared, RECOMPUTED from the ledgers by version 2's own
   rules and returned only once it matched the copy the file carries.
   */
  readonly comparison: readonly ArtifactComparisonRow[];

  /**
   Which lane ships, proven to agree with the ballots recorded beside it and
   with the comparison this reader recomputed.
   */
  readonly laneSelection: ArtifactLaneSelection;

  /**
   What the third rendering settled, that the pass chose not to ask, or that
   this artifact was written before the field existed.
   */
  readonly consolidation: ParsedConsolidation;

  /**
   What the page-level assembly guard did to the composed page, empty on
   every artifact written before the guard existed.
   */
  readonly pageAssembly: ArtifactPageAssembly;

  /**
   What the page-level consistency pass did to the settled page, recounted
   from its own ballots, and empty on every artifact written before
   generation fifteen.
   */
  readonly consistency: ArtifactConsistency;
};

//endregion Artifact version 2 read contract
