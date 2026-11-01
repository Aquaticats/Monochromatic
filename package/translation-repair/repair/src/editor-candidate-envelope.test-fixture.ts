import type {
  EditableEnvelope,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import { applyPatchOperations, } from '@monochromatic-dev/translation-repair-page-pass';
import type { EditorCandidate, } from '../dist/final/node/index.mjs';

//region Editor candidate envelope
// ONE FIXTURE ENVELOPE AND THE CANDIDATE A MODEL PROPOSES AGAINST IT, for
// cases comparing candidates that differ only in who proposed what.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The candidate-select and editor-ensemble
// tests kept their own copy of this envelope and builder; both now import it
// from here.

/**
 Document the envelope sits inside.
 */
export const TARGET_TEXT = 'The cat naps. The cat hates butterflies. The bowl stays full.';

/**
 Fixture envelope covering one sentence.
 */
export const ENVELOPE: EditableEnvelope = {
  envelopeId: 'envelope/butterflies',
  startOffset: TARGET_TEXT.indexOf('The cat hates butterflies.',),
  endOffset: TARGET_TEXT.indexOf('The cat hates butterflies.',)
    + 'The cat hates butterflies.'.length,
  baseText: 'The cat hates butterflies.',
  baseHash: hashContent({ content: 'The cat hates butterflies.', },),
  issueIds: ['adjudicated/butterflies',],
};

/**
 Builds one editor candidate proposing a replacement for the fixture
 envelope, so tests differ only in who proposed what.

 @param modelId - proposing model

 @param newText - replacement it proposed

 @returns Candidate carrying the gated patch

 @example
 ```ts
 const candidate = candidateFor({ modelId, newText, },);
 ```
 */
export function candidateFor(
  {
    modelId,
    newText,
  }: {
    readonly modelId: RosterModelId;
    readonly newText: string;
  },
): EditorCandidate {
  return {
    modelId,
    patch: applyPatchOperations({
      targetText: TARGET_TEXT,
      envelopes: [ENVELOPE,],
      operations: [
        {
          envelopeId: ENVELOPE.envelopeId,
          baseHash: ENVELOPE.baseHash,
          newText,
        },
      ],
      preservation: { mode: 'skip', },
    },),
  };
}

//endregion Editor candidate envelope
