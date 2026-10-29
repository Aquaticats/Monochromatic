import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { AppliedEnvelopeReading, } from './chunk-measure.ts';
import {
  type DeclaredNameRefusalReport,
  declaredNameRefusalReport,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import type {
  EditableEnvelope,
  RepairDocument,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  describeChunkSettlement,
  type SettledChunk,
  settleChunkFromChecks,
} from './repair-chunk-verdict.ts';
import type { EditorStageResult, } from './repair-editor-stage.ts';
import type { IssueResolutionTally, } from '@monochromatic-dev/translation-repair-adjudication/ts';

//region Chunk settlement
// SPLIT OUT OF `repair-chunk.ts` for the file-length cap when the ledger's L3
// strip landed there. The settlement is what runs once the patch that ships
// is known: the verdict over its checker tallies, the declared-name refusal
// it owes, and the line an operator watches a run by.

/**
 Settlement of the shipped patch beside the refusal it owes.
 */
export type ShippedSettlement = SettledChunk & {
  /**
   What the declared-name refusal owes the slice's record and findings.
   */
  readonly refusal: DeclaredNameRefusalReport;
};

/**
 Settles a chunk on the patch that ships, logging its refusal and its
 settlement line.

 @param sliceIndex - slice being settled

 @param declaredNames - name forms the patch may not drop

 @param declaredSubjectForms - the declared forms among them that name the subject, which may stand in for one another

 @param targetText - archive wording of the chunk

 @param shipped - editor result as it ships, after any strip

 @param appliedEnvelopes - what the shipped patch's envelopes bought

 @param tallies - checker verdicts on the shipped patch, by issue id

 @param envelopes - envelopes the patch was written against

 @param targetDocument - archive parsed, the grammar baseline

 @param acceptedCount - issues the panel accepted

 @param unenvelopedCount - accepted issues no envelope could serve

 @param l - chunk logger

 @returns The settlement and the refusal

 @example
 ```ts
 const settled = settleShippedPatch({ sliceIndex, declaredNames, targetText, shipped, appliedEnvelopes, tallies, envelopes, targetDocument, acceptedCount, unenvelopedCount, l, },);
 ```
 */
export function settleShippedPatch(
  {
    sliceIndex,
    declaredNames,
    declaredSubjectForms,
    targetText,
    shipped,
    appliedEnvelopes,
    tallies,
    envelopes,
    targetDocument,
    acceptedCount,
    unenvelopedCount,
    l,
  }: {
    readonly sliceIndex: number;
    readonly declaredNames: readonly string[];
    readonly declaredSubjectForms?: readonly string[];
    readonly targetText: string;
    readonly shipped: EditorStageResult;
    readonly appliedEnvelopes: AppliedEnvelopeReading;
    readonly tallies: Readonly<Record<string, IssueResolutionTally>>;
    readonly envelopes: readonly EditableEnvelope[];
    readonly targetDocument: RepairDocument;
    readonly acceptedCount: number;
    readonly unenvelopedCount: number;
    readonly l: Logger;
  },
): ShippedSettlement {
  /**
   Which candidate won, whether the returned text moved at all, and which
   issues the checkers confirmed.

   Several verdicts rather than one: a patch whose envelope operations cancel
   can win selection and write no byte, and a patch that drops a declared name
   is refused whatever it won. See `settleChunkFromChecks`.
   */
  const settled = settleChunkFromChecks({
    sliceIndex,
    declaredNames,
    ...((declaredSubjectForms === undefined) ? {} : { declaredSubjectForms, }),
    incumbentText: targetText,
    patchedText: shipped.patch
      .patchedText,
    appliedOperations: shipped.patch
      .applied,
    creditableIssues: appliedEnvelopes.creditableIssues,
    tallies,
    envelopes,
    targetDocument,
  },);
  /**
   What the declared-name refusal owes this slice's record and findings.
   */
  const refusal = declaredNameRefusalReport({
    sliceIndex,
    dropped: settled.droppedDeclaredNames,
  },);
  for (const finding of refusal.findings)
    l.warn(finding,);
  l.info(describeChunkSettlement({
    sliceIndex,
    changed: settled.changed,
    patchSelected: settled.patchSelected,
    refused: settled.droppedDeclaredNames
      .length
      > 0,
    ...((settled.patchLostOn === undefined) ? {} : { patchLostOn: settled.patchLostOn, }),
    resolvedCount: settled.resolvedIssueIds
      .length,
    creditableCount: appliedEnvelopes.creditableIssues
      .length,
    acceptedCount,
    unenvelopedCount,
  },),);
  return {
    ...settled,
    refusal,
  };
}

//endregion Chunk settlement
