import type { PatchOperation, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  type Candidate,
  type CandidateProducer,
  mergeProducers,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { EditableEnvelope, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { EditorCandidate, } from './editor-selection-result.ts';

//region Editor proposals
// The distinct replacements offered for ONE envelope, with every model that
// offered each of them.
//
// Provenance is the whole point of this file. Selection discounts a judge's
// ballot for text that judge produced, and it decides that by asking which
// models a candidate is credited to. Dropping a duplicate proposal rather than
// merging its author into the survivor therefore did not just lose a name: it
// let every model after the first vote at FULL weight for its own words, on
// exactly the envelopes where the ensemble agreed.

/**
 Every distinct replacement proposed for one envelope.

 @param candidates - editor outcomes, in roster order

 @param envelope - envelope being decided

 @returns Distinct proposals, each credited to every model that wrote it

 @example
 ```ts
 const proposals = collectEnvelopeProposals({ candidates, envelope, },);
 ```
 */
export function collectEnvelopeProposals(
  {
    candidates,
    envelope,
  }: {
    readonly candidates: readonly EditorCandidate[];
    readonly envelope: EditableEnvelope;
  },
): readonly Candidate<PatchOperation>[] {
  /**
   Proposals kept so far, first writer of each text holding its position.
   */
  const proposals: Candidate<PatchOperation>[] = [];
  for (const candidate of candidates) {
    /**
     This model's applied operation for this envelope, when it has one.
     */
    const operation = candidate.patch
      .applied
      .find(function forEnvelope(op,) {
        return op.envelopeId === envelope.envelopeId;
      },);
    if (operation === undefined)
      continue;

    /**
     This model's stake in whatever it just proposed.
     */
    const producer: CandidateProducer = {
      kind: 'model',
      modelId: candidate.modelId,
    };

    /**
     Earlier proposal carrying the same replacement, when one exists.
     */
    const twin = proposals.find(function sameText(existing,): boolean {
      return existing.value
        .newText
        === operation.newText;
    },);
    if (twin === undefined) {
      proposals.push({
        producer,
        value: operation,
        rendered: operation.newText,
      },);
      continue;
    }
    proposals.splice(
      proposals.indexOf(twin,),
      1,
      {
        ...twin,
        producer: mergeProducers({
          left: twin.producer,
          right: producer,
        },),
      },
    );
  }

  return proposals;
}

//endregion Editor proposals
