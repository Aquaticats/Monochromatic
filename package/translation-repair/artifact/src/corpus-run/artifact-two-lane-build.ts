import type { DocumentLanesResult, } from '@monochromatic-dev/translation-repair-translate-lane/ts';
import {
  type ArtifactLaneSelection,
  compareDocumentLanes,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  preparationIdentity,
  type PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { sourceBytesOf, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { ArtifactConsolidation, } from './artifact-two-lane-consolidate.ts';
import type { ArtifactConsistency, } from './artifact-two-lane-consistency.ts';
import type { ArtifactPageAssembly, } from './artifact-two-lane-page-assembly.ts';
import { projectLanes, } from './artifact-two-lane-derive.ts';
import {
  ARTIFACT_SCHEMA_VERSION_V22,
  type ArtifactJsonValue,
  type ArtifactSectionAlignment,
  type ArtifactSectionCorrespondence,
  type SettledArtifact,
} from './artifact-two-lane-contract.ts';
import { assertDerivationsAgree, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  toArtifactComparisonRow,
} from './artifact-two-lane-project.ts';
import {
  assertFindingsDescribePreparation,
  assertLedgerDescribesPreparation,
  assertResultCountsPreparation,
} from './artifact-two-lane-verify.ts';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Artifact version 2 build
// The one place a two-lane artifact is assembled.
//
// EVERYTHING DERIVABLE IS DERIVED HERE rather than passed in beside what it
// was derived from. Version 1 took a status and two counts as parameters next
// to the result they described, so a caller could state a status the result
// contradicted, and the fields a reader trusts most were the ones least tied to
// what actually ran. The counts moved onto the result in `e4f857c83`; this
// takes the same rule further, since the preparation identity and the whole
// lane comparison are also things nobody should be able to supply.
//
// So the parameters are exactly what cannot be computed from the run: which
// entry it was, which commit and pipeline produced it, which corpus commit the
// texts came from, how the models were called, and how long it took.

/**
 Records which decider chose the aligned sections.

 DERIVED FROM PRESENCE rather than carried as a third state on the
 preparation, because on the preparation absence already means exactly one
 thing: `prepareDocumentPair` ran its deterministic aligner. The artifact
 cannot afford the same spelling, since there absence also means "written
 before the field", so the builder says it out loud.

 @param prepared - preparation the artifact records

 @returns Deterministic when no pairing was supplied, else the pairs consumed

 @example
 ```ts
 const alignment = sectionAlignmentOf({ prepared, },);
 ```
 */
function sectionAlignmentOf(
  { prepared, }: { readonly prepared: PreparedDocumentPair; },
): ArtifactSectionAlignment {
  if (prepared.sectionPairing === undefined)
    return { kind: 'deterministic', };
  return {
    kind: 'supplied',
    pairs: prepared.sectionPairing
      .map(function frozen(pair,): ArtifactSectionCorrespondence {
        return {
          source: pair.source,
          target: pair.target,
        };
      },),
  };
}

/**
 Assembles one settled entry's version 2 artifact.

 @param entryId - corpus entry this covers

 @param tip - repository head when the pass started, as provenance

 @param pipelineDigest - built output that ran, as identity

 @param corpusSha - corpus commit the two texts were read at

 @param callConfig - model call configuration this run used

 @param durationMs - time the entry took on `monotonicMs`, both lanes included

 @param prepared - preparation both lanes ran over, which supplies the
 identity and every measurement rather than being measured by the caller

 @param lanes - what both lanes returned, with the ledgers derived from them

 @param laneSelection - which lane ships, or that nobody has asked

 @param consolidation - what the third rendering settled, or that it never ran

 @param pageAssembly - what the page-level assembly guard did to the composed
 page

 @param consistency - what the page-level consistency pass did to the settled
 page

 @returns Artifact ready to serialize

 @throws {@link LaneComparisonError} when the two ledgers cannot be compared,
 which is a defect in the run rather than in this artifact: an entry whose
 lanes disagree about their own preparation has nothing worth writing

 @throws {@link ArtifactPreparationMismatchError} when either ledger, or the
 run's alignment findings, describe a preparation other than the one passed

 @example
 ```ts
 const artifact = buildSettledTwoLaneArtifact({ entryId, tip, pipelineDigest, ... },);
 ```
 */
export function buildSettledTwoLaneArtifact(
  {
    entryId,
    tip,
    pipelineDigest,
    corpusSha,
    callConfig,
    durationMs,
    prepared,
    lanes,
    laneSelection,
    consolidation,
    pageAssembly,
    consistency,
  }: {
    readonly entryId: string;
    readonly tip: string;
    readonly pipelineDigest: PipelineDigest;
    readonly corpusSha: string;
    readonly callConfig: Readonly<Record<string, ArtifactJsonValue>>;
    readonly durationMs: number;
    readonly prepared: PreparedDocumentPair;
    readonly lanes: DocumentLanesResult;

    /**
     Which lane ships, which the caller states rather than the builder
     assuming. A pass that has not run the contest says so out loud here, so
     the pending state is a decision somebody made rather than a default
     nobody chose.
     */
    readonly laneSelection: ArtifactLaneSelection;

    /**
     What the consolidation settled, stated by the caller for the same reason
     `laneSelection` is: a pass that never asked for a third rendering says so
     out loud rather than leaving an absence the reader has to interpret.
     */
    readonly consolidation: ArtifactConsolidation;

    /**
     What the page-level guard did to the page these sections compose.
     */
    readonly pageAssembly: ArtifactPageAssembly;

    /**
     What the page-level consistency pass did to the page these sections compose.
     */
    readonly consistency: ArtifactConsistency;
  },
): SettledArtifact {
  /**
   Name this slicing gives itself, computed from the preparation rather than
   taken from a caller, so the identity an artifact records is the identity of
   the thing the artifact describes.
   */
  const identity = preparationIdentity({ prepared, },);

  // BOTH LEDGERS AGAINST THE PREPARATION, before anything is derived from
  // either, and INDEPENDENTLY of the name they carry. Each was stamped by the
  // driver that built it; this recomputes what the name should be and refuses a
  // ledger whose own name disagrees. Re-stamping them here instead would make
  // the comparison's unequal-identity refusal unfireable, since two ledgers
  // wearing one applied name always agree, including two from some other
  // preparation entirely.
  assertLedgerDescribesPreparation({
    prepared,
    expected: identity,
    ledger: lanes.repairDelivery,
    lane: 'repair',
  },);
  assertLedgerDescribesPreparation({
    prepared,
    expected: identity,
    ledger: lanes.translateDelivery,
    lane: 'translate',
  },);

  // The raw results are recorded beside those ledgers and are not checked by
  // them: a structurally valid driver result could carry one lane's result and
  // the other's rows. The slice count is what each result says about its own
  // preparation, and is cheap enough to check on the way past.
  assertResultCountsPreparation({
    prepared,
    sliceCount: lanes.repair
      .sliceCount,
    lane: 'repair',
  },);
  assertResultCountsPreparation({
    prepared,
    sliceCount: lanes.translate
      .sliceCount,
    lane: 'translate',
  },);

  // And the one fact the preparation and the driver BOTH report, checked here
  // rather than resolved by preferring a source: recording either silently is
  // picking a winner between two claims nobody compared.
  assertFindingsDescribePreparation({
    prepared,
    reported: lanes.alignmentFindings,
  },);

  /**
   The two lanes compared, derived here rather than accepted as a parameter.

   A comparison supplied beside the ledgers it describes could disagree with
   them, and a reader has no way to tell which of the two to believe. Derived,
   there is only one answer, and a reader that recomputes it is checking this
   code rather than adjudicating between two stored claims.

   The ledgers go in AS THEY CAME, names included, so the comparison's own
   refusal is doing work rather than reading back what this function wrote.
   */
  const comparison = compareDocumentLanes({
    repair: lanes.repairDelivery,
    translate: lanes.translateDelivery,
  },);

  /**
   Both ledgers as version 2 rows, beside the comparison they derive.

   SHARED WITH THE CONTEST DRIVER, which needs the same comparison to know
   which slices are worth asking a roster about. Two derivations here would
   let the writer and the driver disagree about which slices those are.
   */
  const {
    delivery,
    comparison: frozen,
  } = projectLanes({ lanes, },);

  // AND THE TWO DERIVATIONS AGAINST EACH OTHER. The live comparator is what
  // refuses ledger pairs that cannot be compared at all, and the frozen rules
  // are what version 2 MEANS; today they agree, and this is what makes the day
  // they stop a stopped corpus pass rather than a silent change of meaning.
  assertDerivationsAgree({
    frozen,
    live: comparison.slices
      .map(function projectRow(row,) {
        return toArtifactComparisonRow({ row, },);
      },),
  },);
  return {
    artifactSchemaVersion: ARTIFACT_SCHEMA_VERSION_V22,
    id: entryId,
    tip,
    pipelineDigest,
    corpusSha,
    callConfig,
    durationMs,
    timestamp: new Date().toISOString(),
    preparation: {
      identity,

      // The archive as it stood before either lane ran, which is what every
      // later reading of "what changed" is a change FROM. Stored whole because
      // it costs under one percent of the file and removes the reader's
      // dependency on a checkout pinned to the right corpus commit.
      archiveText: prepared.targetText,
      sliceCount: prepared.slices
        .length,
      ...((prepared.frontMatterAuthority === undefined)
        ? {}
        : { frontMatterAuthority: prepared.frontMatterAuthority, }),
      ...((prepared.archiveOriginalSpans === undefined)
        ? {}
        : { archiveOriginalSpans: [ ...prepared.archiveOriginalSpans, ], }),

      // CHARACTER counts, named that way on purpose: they are UTF-16 code unit
      // lengths for reading an entry's size off a log line, and they are NOT
      // what band classification takes. Feeding them into it classifies large
      // pages as small, which has already produced one wrong band census.
      sourceChars: prepared.sourceText
        .length,
      targetChars: prepared.targetText
        .length,

      // The band input, recorded so analysis over a directory has the RIGHT
      // number nearest to hand rather than the tempting wrong one.
      sourceBytes: sourceBytesOf({ text: prepared.sourceText, },),
      alignmentPairCount: prepared.alignmentPairCount,

      // Read off the PREPARATION, which this artifact says these describe. The
      // driver reports the same list, and picking one of two claims is what
      // `assertFindingsDescribePreparation` exists to stop: with the two
      // checked equal, this reads from the side the field is filed under.
      alignmentFindings: [...prepared.alignmentFindings,],

      // THE PAIRING THE SLICING WAS BUILT ON, taken off the preparation for the
      // same reason the findings are: preparation echoes back the map it
      // consumed, so this records the decision itself rather than a second copy
      // that could describe a different run.
      //
      // OMITTED RATHER THAN EMPTIED when no roster was asked. An empty list is
      // the answer "asked, agreed nothing", and writing it where nobody was
      // asked would put a claim in the file that nobody made.
      ...((prepared.blockPairing === undefined)
        ? {}
        : { blockPairing: [...prepared.blockPairing,], }),

      // WHICH DECIDER CHOSE THE SECTIONS, always written. Unlike the block
      // pairing, the deterministic path is the ordinary case here, so an
      // omitted field would not tell "the aligner decided" from "written
      // before the field", and a rebuild has to know which.
      sectionPairing: sectionAlignmentOf({ prepared, },),
    },
    lanes: {
      repair: {
        result: lanes.repair,
        delivery: delivery.repair,
      },
      translate: {
        result: lanes.translate,
        delivery: delivery.translate,
      },
    },
    comparison: frozen,

    // WHAT THE CALLER SAID, said out loud. Which lane ships is answered by the
    // roster or by nobody, and an artifact that left the field out would make
    // "not decided yet" indistinguishable from "written before anyone asked".
    laneSelection,

    // THE THIRD RENDERING, likewise stated. A pass that ran only the two lanes
    // records that it did not ask, so a census counting consolidated slices is
    // not counting a field's absence as a stage that found nothing.
    consolidation,

    // WHAT THE PAGE-LEVEL GUARD DID to the page `lanes`, `laneSelection` and
    // `consolidation` compose, applied first by every reader composing that page.
    pageAssembly,

    // WHAT THE PAGE-LEVEL CONSISTENCY PASS DID to the settled page, applied
    // below the page assembly and above the polish when a slice's text is read.
    consistency,
  };
}

//endregion Artifact version 2 build
