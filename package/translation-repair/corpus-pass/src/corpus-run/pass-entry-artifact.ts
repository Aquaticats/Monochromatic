import type { DocumentLanesResult, } from '@monochromatic-dev/translation-repair-translate-lane/ts';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  type ArtifactConsistency,
  type ArtifactConsolidation,
  type ArtifactPageAssembly,
  buildSettledTwoLaneArtifact,
  type SettledArtifact,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import type { ArtifactLaneSelection, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { RUN_CALL_CONFIG, } from '@monochromatic-dev/translation-repair-run/ts';

//region Pass entry artifact
// The artifact one settled entry writes, assembled from what the pass ran,
// split out of `pass-entry.ts` at its line budget.

/**
 Builds the rich artifact for later grading; corpus-derived, hence gitignored.

 Everything derivable is derived inside the builder, so what goes in is the
 preparation and the driver's own result rather than counts taken off them
 here. It refuses a run whose ledgers do not describe that preparation, which
 is why no artifact can name a slicing the lanes never ran over.

 A CONTEST THAT RAN, whatever it found: a document whose two lanes never
 differed records an empty contest rather than the pending kind, since "the
 roster was asked and nothing differed" and "nobody has asked" are different
 facts, and the pending kind now means only the second. A CONSOLIDATION THAT
 RAN is recorded the same way, for the same reason.

 @param entryId - corpus entry this covers

 @param tip - repository head recorded into the artifact

 @param pipelineDigest - identity of the built pipeline

 @param corpusSha - corpus commit the entry was read at, which the artifact
 names so a reader of it knows which pages it repaired

 @param durationMs - time the entry took on `monotonicMs`, both lanes and the contest
 included

 @param prepared - slicing both lanes ran over

 @param lanes - what both lanes returned

 @param contestSlices - what the lane contest decided

 @param consolidateSlices - what the consolidation settled

 @param pageAssembly - what the page-level assembly guard did to the composed
 page

 @param consistency - what the page-level consistency pass did to the settled
 page

 @returns Artifact ready to serialize

 @example
 ```ts
 const artifact = settledEntryArtifact({ entryId, tip, pipelineDigest, corpusSha, durationMs, prepared, lanes, contestSlices, consolidateSlices, pageAssembly, consistency, },);
 ```
 */
export function settledEntryArtifact(
  {
    entryId,
    tip,
    pipelineDigest,
    corpusSha,
    durationMs,
    prepared,
    lanes,
    contestSlices,
    consolidateSlices,
    pageAssembly,
    consistency,
  }: {
    readonly entryId: string;
    readonly tip: string;
    readonly pipelineDigest: PipelineDigest;
    readonly corpusSha: string;
    readonly durationMs: number;
    readonly prepared: PreparedDocumentPair;
    readonly lanes: DocumentLanesResult;
    readonly contestSlices: Extract<ArtifactLaneSelection, { readonly kind: 'contested'; }>['slices'];
    readonly consolidateSlices: Extract<ArtifactConsolidation, { readonly kind: 'settled'; }>['slices'];
    readonly pageAssembly: ArtifactPageAssembly;
    readonly consistency: ArtifactConsistency;
  },
): SettledArtifact {
  return buildSettledTwoLaneArtifact({
    entryId,
    tip,
    pipelineDigest,
    corpusSha,
    callConfig: RUN_CALL_CONFIG,
    durationMs,
    prepared,
    lanes,
    laneSelection: {
      kind: 'contested',
      slices: contestSlices,
    },
    consolidation: {
      kind: 'settled',
      slices: consolidateSlices,
    },
    pageAssembly,
    consistency,
  },);
}

//endregion Pass entry artifact
