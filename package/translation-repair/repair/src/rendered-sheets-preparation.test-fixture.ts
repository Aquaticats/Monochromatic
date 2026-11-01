/**
 The sheets the preparation asks, rendered for `rendered-sheets.test-fixture.ts`
 (ledger X17).

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  IMAGE_READING_PERSPECTIVES,
  imageReadingText,
} from '@monochromatic-dev/translation-repair-slice-reading';
import {
  buildReferenceAttestConfirmMessages,
  buildReferenceAttestMessages,
} from '@monochromatic-dev/translation-repair-reference';
import { buildCandidateSelectMessages, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  buildBlockPairingMessages,
  buildSectionPairingMessages,
} from '@monochromatic-dev/translation-repair-stage';
import {
  ARCHIVE_BLOCK_DECLINE_CONSEQUENCE,
  ARCHIVE_BLOCK_SELECTION_CRITERIA,
  ARCHIVE_BLOCK_SELECTION_TASK,
  archiveBlockSelectionEvidence,
  buildArchiveBlockReviewMessages,
  withArchiveOriginal,
} from '@monochromatic-dev/translation-repair-archive';
import { SEAT_HYPER_ONLY, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

import {
  type RenderedSheet,
  SOURCE,
  ARCHIVE,
  IDENTITY,
  REFERENCES,
  ARCHIVE_QUOTE,
  ASIDE,
  ASIDE_REVISED,
  joined,
} from './rendered-sheets-texts.test-fixture.ts';

//region Preparation sheets

/**
 The sheets the preparation asks: attestation, pairing, the archive review
 and its correction slate, and the picture readers (ledger X17).

 @returns One entry per sheet

 @example
 ```ts
 const sheets = evidenceSheets();
 ```
 */
export function evidenceSheets(): readonly RenderedSheet[] {
  /**
   Archive carrying the unclaimed block.
   */
  const archiveWithAside = `${ARCHIVE}\n\n${ASIDE}`;
  /**
   The correction slate's candidates: the revision, then the block as it stands.
   */
  const archiveCandidates = withArchiveOriginal({
    revisions: [{
      producer: {
        kind: 'model',
        modelId: SEAT_HYPER_ONLY,
      },
      value: ASIDE_REVISED,
      rendered: ASIDE_REVISED,
    },],
    blockText: ASIDE,
  },);
  return [
    {
      name: 'reference attestation',
      text: joined({
        messages: buildReferenceAttestMessages({
          sourceText: SOURCE,
          archiveText: ARCHIVE,
          referenceContext: REFERENCES,
        },),
      },),
    },
    {
      name: 'reference attestation confirmation',
      text: joined({
        messages: buildReferenceAttestConfirmMessages({
          sourceText: SOURCE,
          archiveText: ARCHIVE,
          referenceContext: REFERENCES,
          candidates: [{
            archiveQuote: ARCHIVE_QUOTE,
            reference: 1,
            referenceQuote: 'Mittens had an older sister',
            voices: 1,
            heard: 2,
          },],
        },),
      },),
    },
    {
      name: 'section pairing',
      text: joined({
        messages: buildSectionPairingMessages({
          sourceSections: [{
            index: 0,
            text: SOURCE,
          },],
          targetSections: [{
            index: 0,
            text: ARCHIVE,
          },],
        },),
      },),
    },
    {
      name: 'block pairing',
      text: joined({
        messages: buildBlockPairingMessages({
          sourceBlocks: [{
            index: 0,
            text: SOURCE,
          },],
          targetBlocks: [
            {
              index: 0,
              text: ARCHIVE,
            },
            {
              index: 1,
              text: ASIDE,
            },
          ],
          pictureContext: '- picture 1 (chat.webp): [left] 咪咪：我明天还来。',
        },),
      },),
    },
    {
      name: 'archive block review',
      text: joined({
        messages: buildArchiveBlockReviewMessages({
          sourceText: SOURCE,
          targetText: archiveWithAside,
          blockText: ASIDE,
          priorFindings: [],
          // What preparation passes the review since ledger B28.
          identityContext: IDENTITY,
          referenceContext: REFERENCES,
        },),
      },),
    },
    {
      name: 'archive correction slate',
      text: joined({
        messages: buildCandidateSelectMessages({
          task: ARCHIVE_BLOCK_SELECTION_TASK,
          criteria: ARCHIVE_BLOCK_SELECTION_CRITERIA,
          evidence: archiveBlockSelectionEvidence({
            sourceText: SOURCE,
            targetText: archiveWithAside,
            blockText: ASIDE,
            voices: [{
              modelId: SEAT_HYPER_ONLY,
              value: {
                disposition: 'revise',
                sourceQuote: '',
                replacementText: ASIDE_REVISED,
                assessment: 'The original does not say the kitten was a favourite.',
              },
            },],
            candidates: archiveCandidates,
            priorFindings: [],
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
          },),
          rendered: archiveCandidates.map(function renderedOf(candidate,): string {
            return candidate.rendered;
          },),
          declineConsequence: ARCHIVE_BLOCK_DECLINE_CONSEQUENCE,
          sourceText: SOURCE,
        },),
      },),
    },
    ...IMAGE_READING_PERSPECTIVES.map(function readingSheet(
      perspective,
      at,
    ): RenderedSheet {
      return {
        name: `picture reading ${String(at + 1,)}`,
        text: imageReadingText({ perspective, },),
      };
    },),
  ];
}

//endregion Preparation sheets
