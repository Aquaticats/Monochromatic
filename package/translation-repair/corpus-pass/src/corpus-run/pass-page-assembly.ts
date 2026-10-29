import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { DocumentLanesResult, } from '@monochromatic-dev/translation-repair-translate-lane/ts';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ArtifactLaneSelection, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  type ArtifactConsistency,
  type ArtifactConsolidation,
  NO_PAGE_ASSEMBLY,
  type SettledArtifact,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import { guardPageAssembly, } from '@monochromatic-dev/translation-repair-page-check/ts';
import { settledEntryArtifact, } from './pass-entry-artifact.ts';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Pass page assembly
// The artifact is composed twice: once with no page assembly, so the page it
// would ship can be read, and once with what the guard made of that page. Both
// compositions are pure, and the second is the one written.

/**
 Composes the settled artifact with the page-level guard's outcome recorded.

 @param entryId - corpus entry

 @param tip - pipeline tip

 @param pipelineDigest - digest of the pipeline that ran

 @param corpusSha - corpus commit the entry was read at, which the artifact
 names

 @param durationMs - time the entry took, on `monotonicMs`

 @param prepared - preparation both lanes ran over

 @param lanes - both lane results and their ledgers

 @param contestSlices - what the contest decided per slice

 @param consolidateSlices - what the third rendering settled per slice

 @param consistency - what the page-level consistency pass did to the settled
 page, which the guard reads the page through

 @param targetText - archive text the page's replacements address

 @param l - logger the guard's findings reach

 @returns Artifact to write

 @example
 ```ts
 const artifact = settledPageArtifact({ entryId, tip, pipelineDigest, corpusSha, durationMs, prepared, lanes, contestSlices, consolidateSlices, consistency, targetText, l, },);
 ```
 */
export function settledPageArtifact(
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
    consistency,
    targetText,
    l,
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
    readonly consistency: ArtifactConsistency;
    readonly targetText: string;
    readonly l: Logger;
  },
): SettledArtifact {
  /**
   The artifact as the stages composed it, before the guard.
   */
  const composed = settledEntryArtifact({
    entryId,
    tip,
    pipelineDigest,
    corpusSha,
    durationMs,
    prepared,
    lanes,
    contestSlices,
    consolidateSlices,
    pageAssembly: NO_PAGE_ASSEMBLY,
    consistency,
  },);
  /**
   What the guard made of the page that artifact would ship.
   */
  const pageAssembly = guardPageAssembly({
    artifact: composed,
    slices: prepared.slices,
    sourceText: prepared.sourceText,
    targetText,
    archiveOriginalSpans: prepared.archiveOriginalSpans ?? [],
  },);
  for (const finding of pageAssembly.findings)
    l.warn(`page assembly: ${finding}`,);
  /**
   Slices the passes or the guard rewrote, trims among them.
   */
  const trimmedCount = pageAssembly.trimmed
    .length;
  /**
   Slices the guard took back.
   */
  const withdrawnCount = pageAssembly.withdrawn
    .length;
  if ((trimmedCount > 0) || (withdrawnCount > 0)) {
    l.warn(
      `page assembly rewrote ${String(trimmedCount,)} ${
        wordForCount({
          count: trimmedCount,
          one: 'slice',
          many: 'slices',
        },)
      } and withdrew ${
        String(withdrawnCount,)
      }; the findings say why`,
    );
  }
  return settledEntryArtifact({
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
  },);
}

//endregion Pass page assembly
