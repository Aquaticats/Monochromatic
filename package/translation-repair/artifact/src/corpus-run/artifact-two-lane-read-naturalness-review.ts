import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { requireExactKeys, } from '../artifact-exact-guard.ts';
import {
  ArtifactParseError,
  requireArray,
  requireCount,
  requireRecord,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  ArtifactNaturalnessReview,
  ArtifactNaturalnessReviewRound,
} from './artifact-two-lane-consolidate.ts';
import { parseNaturalnessConfirmations, } from './artifact-two-lane-read-naturalness-confirmation.ts';
import {
  assertFinalNaturalnessDigests,
  assertNaturalnessCorrectionChain,
  parseNaturalnessCorrection,
} from './artifact-two-lane-read-naturalness-digest.ts';
import { parseNaturalnessReviewRound, } from './artifact-two-lane-read-naturalness-round.ts';

//region Artifact absolute naturalness review read

/**
 Reads schema-eight or later absolute review and binds final round to final text.

 @param value - unknown review field

 @param path - artifact path

 @param finalText - exact polish text artifact says ships

 @param correctionChainRequired - whether generation requires transition digests

 @param everyBodyBlockReviewed - whether reviewed paragraphs are every body
 block (generation ten) rather than the refinable paragraphs alone

 @param quorumBasisRequired - whether generation stores the effective wider quorum

 @param lastSeatGrace - whether generation decided its rounds with the
 last-seat grace of 2026-10-22 (generation twenty on)

 @returns Cross-validated review audit

 @example
 ```ts
 const review = parseNaturalnessReview({ value, path, finalText, });
 ```
 */
export function parseNaturalnessReview(
  {
    value,
    path,
    finalText,
    correctionChainRequired = false,
    everyBodyBlockReviewed = false,
    quorumBasisRequired = false,
    lastSeatGrace = false,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly finalText: string;
    readonly correctionChainRequired?: boolean;
    readonly everyBodyBlockReviewed?: boolean;
    readonly quorumBasisRequired?: boolean;
    readonly lastSeatGrace?: boolean;
  },
): ArtifactNaturalnessReview {
  /**
   Reads one schema-nine-or-later confirmation round under this review's
   paragraph set.

   @param round - unknown confirmation round and its path

   @returns Exact candidate and paragraph-bound round

   @example
   ```ts
   const parsed = parseConfirmationRound({ value, path, },);
   ```
   */
  function parseConfirmationRound(
    round: {
      readonly value: unknown;
      readonly path: string;
    },
  ): ArtifactNaturalnessReviewRound {
    return parseNaturalnessReviewRound({
      value: round.value,
      path: round.path,
      paragraphDigestsRequired: true,
      everyBodyBlockReviewed,
      quorumBasisRequired,
      lastSeatGrace,
    },);
  }
  /**
   Review under exact schema-eight shape.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'correctionCount',
      ...(correctionChainRequired
        ? [
          'corrections',
          'confirmations',
        ]
        : []),
      'rounds',
    ],
    path,
  },);
  /**
   Correction count under artifact generation policy.
   */
  const correctionCount = requireCount({
    value: record.correctionCount,
    path: `${path}.correctionCount`,
  },);
  if ((!correctionChainRequired) && (correctionCount > 1)) {
    throw new ArtifactParseError({
      path: `${path}.correctionCount`,
      reason: 'zero or one legacy correction',
    },);
  }
  /**
   Decisive initial and post-correction reviews.
   */
  const rounds = requireArray({
    value: record.rounds,
    path: `${path}.rounds`,
  },)
    .map(function readRound(
      entry,
      at,
    ) {
      return parseNaturalnessReviewRound({
        value: entry,
        path: `${path}.rounds[${String(at,)}]`,
        paragraphDigestsRequired: correctionChainRequired,
        everyBodyBlockReviewed,
        quorumBasisRequired,
        lastSeatGrace,
      },);
    },);
  if (rounds.length !== (correctionCount + 1)) {
    throw new ArtifactParseError({
      path: `${path}.rounds`,
      reason: `${String(correctionCount + 1,)} ${
        wordForCount({
          count: correctionCount + 1,
          one: 'round',
          many: 'rounds',
        },)
      } for correction count`,
    },);
  }
  /**
   Digest-bound correction transitions under generation-nine shape.
   */
  const corrections = correctionChainRequired
    ? requireArray({
      value: record.corrections,
      path: `${path}.corrections`,
    },)
      .map(function readCorrection(
        entry,
        at,
      ) {
        return parseNaturalnessCorrection({
          value: entry,
          path: `${path}.corrections[${String(at,)}]`,
        },);
      },)
    : [];
  if (correctionChainRequired && (corrections.length !== correctionCount)) {
    throw new ArtifactParseError({
      path: `${path}.corrections`,
      reason: `${String(correctionCount,)} digest-bound correction transitions`,
    },);
  }
  /**
   Whether artifact carries post-generation-nine acceptance confirmation.
   */
  const confirmationsPresent = correctionChainRequired && ('confirmations' in record);
  /**
   Final review, present because the rounds number one more than the
   correction count, which is never negative.
   */
  const final = nonNullishOrThrow(rounds.at(-1,),);
  /**
   Reviews that each authorized one correction generation.
   */
  const priorRounds = rounds.slice(
    0,
    -1,
  );
  if (priorRounds.some(function nonRejection(round,): boolean {
    return round.verdict !== 'unacceptable';
  },)) {
    throw new ArtifactParseError({
      path: `${path}.rounds`,
      reason: 'unacceptable review before every correction',
    },);
  }
  assertNaturalnessCorrectionChain({
    corrections,
    rounds,
    path,
  },);
  // The final verdict is recorded evidence, whatever it says: the no-loop
  // design demoted review authority, so a rejected or quorumless final round
  // is a valid record rather than a malformed one.
  /**
   Earlier acceptable readings bound to decisive reviewed candidates.
   */
  const confirmations = parseNaturalnessConfirmations({
    value: record.confirmations,
    present: confirmationsPresent,
    rounds,
    final,
    path,
    parseRound: parseConfirmationRound,
  },);
  /**
   Path of final accepted exact-text review.
   */
  const finalPath = `${path}.rounds[${String(rounds.length - 1,)}]`;
  assertFinalNaturalnessDigests({
    final,
    finalText,
    path: finalPath,
    paragraphDigestsRequired: correctionChainRequired,
    everyBodyBlockReviewed,
  },);
  return {
    correctionCount,
    ...(correctionChainRequired ? { corrections, } : {}),
    rounds,
    ...(confirmationsPresent ? { confirmations, } : {}),
  };
}

//endregion Artifact absolute naturalness review read
