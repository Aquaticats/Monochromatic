import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  buildReviewedFidelityReference,
  type EditorFidelityTrial,
  type ReviewedFidelityReference,
} from '../dist/final/node/index.mjs';
import {
  REVIEW_REFERENCE,
  REVIEW_SOURCE,
  reviewedFixture,
} from './fidelity-reference.test-fixture.ts';

//region Invented editor fidelity fixtures
// One invented reviewed reference, built through the real reference builder so its damaged twins are the ones the
// builders make, and the cards the cases seat. Cat-themed invention; no corpus content.

/**
 The card the cases seat alone.
 */
export const LOW_CARD: RosterModelId = 'claude-sonnet-5-5-effort-low';

/**
 A second card of the same wire model, which a run may bench in its own gather.
 */
export const HIGH_CARD: RosterModelId = 'claude-sonnet-5-5-effort-high';

/**
 The default effort of the same wire model, which the bench runs twice to measure its own spread.
 */
export const MEDIUM_CARD: RosterModelId = 'claude-sonnet-5-5-effort-medium';

/**
 A card of the other wire model, whose prices differ.
 */
export const HAIKU_CARD: RosterModelId = 'claude-haiku-5-5-effort-low';

/**
 The invented reference with its three damaged twins, a deletion, an insertion and an alteration.

 @returns Reference as the reader would hand it

 @example
 ```ts
 const reference = inventedReference();
 ```
 */
export function inventedReference(): ReviewedFidelityReference {
  /**
   Files and specification the invented reference is built from.
   */
  const {
    sourceFile,
    archiveFile,
    spec,
  } = reviewedFixture();
  return buildReviewedFidelityReference({
    sourceFile,
    archiveFile,
    spec,
  },);
}

/**
 Heading of the section holding the reviewed passage, in the original of the headed pages.
 */
export const HEADED_SOURCE_HEADING = '## 正文';

/**
 Heading of the section holding the reviewed passage, in the archive of the headed pages.
 */
export const HEADED_ARCHIVE_HEADING = '## Body';

/**
 Source of the section before the reviewed passage in the headed pages.
 */
export const HEADED_SOURCE_BEFORE = '## 前言\n猫在书店里睡觉。';

/**
 Source of the section after the reviewed passage in the headed pages.
 */
export const HEADED_SOURCE_AFTER = '## 后记\n猫在夜里回家。';

/**
 Archive of the section before the reviewed passage in the headed pages.
 */
export const HEADED_ARCHIVE_BEFORE = '## Preface\nThe cat slept in the bookshop.';

/**
 Archive of the section after the reviewed passage in the headed pages.
 */
export const HEADED_ARCHIVE_AFTER = '## Afterword\nThe cat came home at night.';

/**
 The invented reference on pages of three headed sections, the reviewed passage in the middle one below its heading,
 so that the heading slice and the section after are the slices either side of it.

 @returns Reference with its pages and archive range of the passage moved to match

 @example
 ```ts
 const reference = headedReference();
 ```
 */
export function headedReference(): ReviewedFidelityReference {
  /**
   The invented reference the headed pages are built around.
   */
  const reference = inventedReference();
  /**
   Pages of three sections, aligned by position.
   */
  const pages = {
    sourceFile: `${HEADED_SOURCE_BEFORE}\n\n${HEADED_SOURCE_HEADING}\n${REVIEW_SOURCE}\n\n${HEADED_SOURCE_AFTER}\n`,
    archiveFile: `${HEADED_ARCHIVE_BEFORE}\n\n${HEADED_ARCHIVE_HEADING}\n${REVIEW_REFERENCE}\n\n${HEADED_ARCHIVE_AFTER}\n`,
  };
  /**
   Where the passage begins in the headed archive.
   */
  const startOffset = pages.archiveFile
    .indexOf(REVIEW_REFERENCE,);
  /**
   Provenance of the invented reference, whose archive range moves.
   */
  const { spec, } = reference;
  /**
   Archive range of the invented reference before it moves.
   */
  const { archive, } = spec;
  return {
    ...reference,
    spec: {
      ...spec,
      archive: {
        ...archive,
        startOffset,
        endOffset: startOffset + REVIEW_REFERENCE.length,
      },
    },
    pages,
  };
}

/**
 The region of the damaged text.

 @param trial - trial whose region is read

 @returns Text the envelope holds

 @example
 ```ts
 const region = regionOf({ trial: ALTERATION, },);
 ```
 */
export function regionOf({ trial, }: { readonly trial: EditorFidelityTrial; },): string {
  return trial.damagedText
    .slice(
    trial.region
      .startOffset,
    trial.region
      .endOffset,
    );
}

//endregion Invented editor fidelity fixtures
