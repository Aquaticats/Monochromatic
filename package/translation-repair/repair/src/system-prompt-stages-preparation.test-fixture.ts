/**
 The stages the preparation asks, each handed a subject, for the guard that holds every stage's system prompt to the
 same text on every call (R4 of the second review of the Claude route).

 Fixtures are cat-themed and dog-themed invention; no corpus content appears here.

 @module
 */

import { buildCandidateSelectMessages, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  ARCHIVE_BLOCK_DECLINE_CONSEQUENCE,
  ARCHIVE_BLOCK_SELECTION_CRITERIA,
  ARCHIVE_BLOCK_SELECTION_TASK,
  archiveBlockSelectionEvidence,
  buildArchiveBlockReviewMessages,
  withArchiveOriginal,
} from '@monochromatic-dev/translation-repair-archive';
import {
  buildReferenceAttestConfirmMessages,
  buildReferenceAttestMessages,
} from '@monochromatic-dev/translation-repair-reference';
import {
  buildBlockPairingMessages,
  buildSectionPairingMessages,
} from '@monochromatic-dev/translation-repair-stage';
import { SEAT_HYPER_ONLY, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import type {
  PromptMessages,
  PromptStage,
  PromptSubject,
} from './system-prompt-subject.test-fixture.ts';

//region Preparation stages

/**
 The archive correction slate over a subject's unclaimed block.

 @param subject - invented subject

 @returns Slate's messages

 @example
 ```ts
 const messages = archiveCorrectionSlateOf({ subject: CAT_SUBJECT, },);
 ```
 */
function archiveCorrectionSlateOf(
  { subject, }: { readonly subject: PromptSubject; },
): PromptMessages {
  /**
   Archive carrying the unclaimed block.
   */
  const archiveWithAside = `${subject.archive}\n\n${subject.aside}`;
  /**
   The slate's candidates: the revision, then the block as it stands.
   */
  const candidates = withArchiveOriginal({
    revisions: [{
      producer: {
        kind: 'model',
        modelId: SEAT_HYPER_ONLY,
      },
      value: subject.asideRevised,
      rendered: subject.asideRevised,
    },],
    blockText: subject.aside,
  },);
  return buildCandidateSelectMessages({
    task: ARCHIVE_BLOCK_SELECTION_TASK,
    criteria: ARCHIVE_BLOCK_SELECTION_CRITERIA,
    evidence: archiveBlockSelectionEvidence({
      sourceText: subject.source,
      targetText: archiveWithAside,
      blockText: subject.aside,
      voices: [{
        modelId: SEAT_HYPER_ONLY,
        value: {
          disposition: 'revise',
          sourceQuote: '',
          replacementText: subject.asideRevised,
          assessment: `The original does not say the ${subject.name} was a favourite.`,
        },
      },],
      candidates,
      priorFindings: [],
      identityContext: subject.identity,
      referenceContext: subject.references,
    },),
    rendered: candidates.map(function renderedOf(candidate,): string {
      return candidate.rendered;
    },),
    declineConsequence: ARCHIVE_BLOCK_DECLINE_CONSEQUENCE,
    sourceText: subject.source,
  },);
}

/**
 The stages the preparation asks, each rendered from a subject.
 */
export const PREPARATION_STAGES: readonly PromptStage[] = [
  {
    name: 'reference attestation',
    render: function renderReferenceAttestation({ subject, },) {
      return buildReferenceAttestMessages({
        sourceText: subject.source,
        archiveText: subject.archive,
        referenceContext: subject.references,
      },);
    },
  },
  {
    name: 'reference attestation confirmation',
    render: function renderReferenceAttestationConfirmation({ subject, },) {
      return buildReferenceAttestConfirmMessages({
        sourceText: subject.source,
        archiveText: subject.archive,
        referenceContext: subject.references,
        candidates: [{
          archiveQuote: subject.archiveQuote,
          reference: 1,
          referenceQuote: `${subject.name} had an older sister`,
          voices: 1,
          heard: 2,
        },],
      },);
    },
  },
  {
    name: 'section pairing',
    render: function renderSectionPairing({ subject, },) {
      return buildSectionPairingMessages({
        sourceSections: [{
          index: 0,
          text: subject.source,
        },],
        targetSections: [{
          index: 0,
          text: subject.archive,
        },],
      },);
    },
  },
  {
    name: 'block pairing',
    render: function renderBlockPairing({ subject, },) {
      return buildBlockPairingMessages({
        sourceBlocks: [{
          index: 0,
          text: subject.source,
        },],
        targetBlocks: [
          {
            index: 0,
            text: subject.archive,
          },
          {
            index: 1,
            text: subject.aside,
          },
        ],
        pictureContext: '',
      },);
    },
  },
  {
    name: 'block pairing, pictures read',
    render: function renderBlockPairingPicturesRead({ subject, },) {
      return buildBlockPairingMessages({
        sourceBlocks: [{
          index: 0,
          text: subject.source,
        },],
        targetBlocks: [
          {
            index: 0,
            text: subject.archive,
          },
          {
            index: 1,
            text: subject.aside,
          },
        ],
        pictureContext: subject.pictureContext,
      },);
    },
  },
  {
    name: 'archive block review',
    render: function renderArchiveBlockReview({ subject, },) {
      return buildArchiveBlockReviewMessages({
        sourceText: subject.source,
        targetText: `${subject.archive}\n\n${subject.aside}`,
        blockText: subject.aside,
        priorFindings: [],
        identityContext: subject.identity,
        referenceContext: subject.references,
      },);
    },
  },
  {
    name: 'archive correction slate',
    render: archiveCorrectionSlateOf,
  },
];

//endregion Preparation stages
