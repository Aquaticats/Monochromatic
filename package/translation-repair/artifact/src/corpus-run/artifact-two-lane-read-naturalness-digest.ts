import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { requireExactKeys, } from '../artifact-exact-guard.ts';
import { isLowerHexDigit, } from '@monochromatic-dev/translation-repair-markup/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  ArtifactParseError,
  requireArray,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  finalPolishParagraphs,
  reviewParagraphsOf,
} from '@monochromatic-dev/translation-repair-consolidate/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type {
  ArtifactNaturalnessCorrection,
  ArtifactNaturalnessReviewRound,
} from './artifact-two-lane-consolidate.ts';

//region Artifact naturalness digest chain

/**
 Paragraphs a generation showed its reviewers, re-derived from the text.

 @param text - reviewed candidate text

 @param everyBodyBlockReviewed - whether the writing generation showed every
 body block (generation ten) rather than the refinable paragraphs alone

 @returns Paragraph texts in display order

 @example
 ```ts
 const paragraphs = reviewedParagraphsOf({ text, everyBodyBlockReviewed: true, },);
 ```
 */
function reviewedParagraphsOf(
  {
    text,
    everyBodyBlockReviewed,
  }: {
    readonly text: string;
    readonly everyBodyBlockReviewed: boolean;
  },
): readonly string[] {
  return everyBodyBlockReviewed
    ? reviewParagraphsOf({ text, },)
    : finalPolishParagraphs({ text, },);
}

/**
 Character length of lowercase hexadecimal SHA-256 digest.
 */
const SHA256_HEX_LENGTH = 64;

/**
 Checks lowercase hexadecimal SHA-256 shape without regular expression.

 @param value - candidate digest

 @returns Whether exact ASCII digest shape matches
 */
function isLowerHexDigest(
  { value, }: { readonly value: string; },
): boolean {
  if (value.length !== SHA256_HEX_LENGTH)
    return false;
  for (let index = 0; index < value.length; index += 1) {
    if (!isLowerHexDigit({ character: value.charAt(index,), },))
      return false;
  }
  return true;
}

/**
 Requires one lowercase hexadecimal SHA-256 digest.

 @param value - unknown digest

 @param path - artifact path

 @returns Validated digest

 @example
 ```ts
 const digest = requireNaturalnessDigest({ value, path, });
 ```
 */
export function requireNaturalnessDigest(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): string {
  /**
   String before digest-shape validation.
   */
  const digest = requireString({
    value,
    path,
  },);
  if (!isLowerHexDigest({ value: digest, })) {
    throw new ArtifactParseError({
      path,
      reason: 'lowercase hexadecimal SHA-256 digest',
    },);
  }
  return digest;
}

/**
 Reads reviewed paragraph identity digests.

 @param value - unknown digest array

 @param path - artifact path

 @param paragraphCount - exact number of reviewed paragraphs

 @returns Validated paragraph digests

 @example
 ```ts
 const digests = parseParagraphDigests({ value, path, paragraphCount, });
 ```
 */
export function parseParagraphDigests(
  {
    value,
    path,
    paragraphCount,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly paragraphCount: number;
  },
): readonly string[] {
  /**
   Unknown rows parsed as exact digests.
   */
  const digests = requireArray({
    value,
    path,
  },)
    .map(function readDigest(
      entry,
      at,
    ): string {
      return requireNaturalnessDigest({
        value: entry,
        path: `${path}[${String(at,)}]`,
      },);
    },);
  if (digests.length !== paragraphCount) {
    throw new ArtifactParseError({
      path,
      reason: `${String(paragraphCount,)} reviewed paragraph ${
        wordForCount({
          count: paragraphCount,
          one: 'digest',
          many: 'digests',
        },)
      }`,
    },);
  }
  return digests;
}

/**
 Reads one generation-nine digest-bound correction transition.

 @param value - unknown transition

 @param path - artifact path

 @returns Validated transition digests

 @example
 ```ts
 const correction = parseNaturalnessCorrection({ value, path, });
 ```
 */
export function parseNaturalnessCorrection(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactNaturalnessCorrection {
  /**
   Transition under exact generation-nine shape.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'inputDigest',
      'findingsDigest',
      'gatedTextDigest',
    ],
    path,
  },);
  return {
    inputDigest: requireNaturalnessDigest({
      value: record.inputDigest,
      path: `${path}.inputDigest`,
    },),
    findingsDigest: requireNaturalnessDigest({
      value: record.findingsDigest,
      path: `${path}.findingsDigest`,
    },),
    gatedTextDigest: requireNaturalnessDigest({
      value: record.gatedTextDigest,
      path: `${path}.gatedTextDigest`,
    },),
  };
}

/**
 Verifies correction transitions against adjacent exact review rounds.

 @param corrections - stored digest transitions

 @param rounds - stored exact candidate reviews

 @param path - artifact review path

 @example
 ```ts
 assertNaturalnessCorrectionChain({ corrections, rounds, path, });
 ```
 */
export function assertNaturalnessCorrectionChain(
  {
    corrections,
    rounds,
    path,
  }: {
    readonly corrections: readonly ArtifactNaturalnessCorrection[];
    readonly rounds: readonly ArtifactNaturalnessReviewRound[];
    readonly path: string;
  },
): void {
  corrections.forEach(function verifyTransition(
    correction,
    index,
  ): void {
    // The review reader holds one round more than it holds transitions
    // (`parseNaturalnessReview` refuses any other count), so both rounds
    // around each transition exist.
    /**
     Rejected review feeding this correction.
     */
    const input = nonNullishOrThrow(rounds[index],);
    /**
     Review over exact gated correction output.
     */
    const output = nonNullishOrThrow(rounds[index + 1],);
    /**
     Canonical digest of findings correction received.
     */
    const findingsDigest = hashContent({
      content: JSON.stringify(input.findings,),
    },);
    if ((correction.inputDigest !== input.candidateDigest)
      || (correction.findingsDigest !== findingsDigest)
      || (correction.gatedTextDigest !== output.candidateDigest)) {
      throw new ArtifactParseError({
        path: `${path}.corrections[${String(index,)}]`,
        reason: 'digest chain from rejected review through canonical findings to next gated text',
      },);
    }
  },);
}

/**
 Verifies one reviewed candidate and paragraph identity list from exact text.

 @param candidateText - exact reviewed candidate

 @param candidateDigest - stored candidate digest

 @param paragraphCount - stored correctable paragraph count

 @param paragraphDigests - stored correctable paragraph identities

 @param path - review round path

 @example
 ```ts
 assertReviewedCandidateDigests({ candidateText, candidateDigest, paragraphCount, paragraphDigests, path, });
 ```
 */
export function assertReviewedCandidateDigests(
  {
    candidateText,
    candidateDigest,
    paragraphCount,
    paragraphDigests,
    path,
    everyBodyBlockReviewed = false,
  }: {
    readonly candidateText: string;
    readonly candidateDigest: string;
    readonly paragraphCount: number;
    readonly paragraphDigests: readonly string[];
    readonly path: string;
    readonly everyBodyBlockReviewed?: boolean;
  },
): void {
  /**
   Exact reviewed paragraphs re-derived from candidate text, under the set
   the writing generation showed its reviewers.
   */
  const paragraphs = reviewedParagraphsOf({
    text: candidateText,
    everyBodyBlockReviewed,
  },);
  /**
   Digests independently re-derived from candidate paragraph text.
   */
  const derivedParagraphDigests = paragraphs
    .map(function digestParagraph(paragraph,): string {
      return hashContent({ content: paragraph, },);
    },);
  if (paragraphCount !== paragraphs.length) {
    throw new ArtifactParseError({
      path: `${path}.paragraphCount`,
      reason: 'structurally correctable paragraph count of reviewed candidate text',
    },);
  }
  if (JSON.stringify(paragraphDigests,) !== JSON.stringify(derivedParagraphDigests,)) {
    throw new ArtifactParseError({
      path: `${path}.paragraphDigests`,
      reason: 'SHA-256 digest of every structurally correctable reviewed paragraph',
    },);
  }
  if (candidateDigest !== hashContent({ content: candidateText, },)) {
    throw new ArtifactParseError({
      path: `${path}.candidateDigest`,
      reason: 'SHA-256 of exact reviewed candidate text',
    },);
  }
}

/**
 Verifies final candidate and paragraph digests against exact final text.

 Where the generation records paragraph identities, the round reader
 (`parseNaturalnessReviewRound`) has already checked the final round's
 candidate digest, paragraph count and paragraph digests against its own
 candidate text, so only that text is compared with the text that ships.
 Before 2026-09-30 (UTC) this also refused a final round with no paragraph
 digests, which that reader never returns, and checked the digests again
 against the same text (ledger T8).

 @param final - final review, read by `parseNaturalnessReviewRound` under the
 same generation flags

 @param finalText - exact polish text artifact says ships

 @param path - final review round path

 @param paragraphDigestsRequired - whether generation records paragraph identities

 @example
 ```ts
 assertFinalNaturalnessDigests({ final, finalText, path, paragraphDigestsRequired: true, });
 ```
 */
export function assertFinalNaturalnessDigests(
  {
    final,
    finalText,
    path,
    paragraphDigestsRequired,
    everyBodyBlockReviewed = false,
  }: {
    readonly final: ArtifactNaturalnessReviewRound;
    readonly finalText: string;
    readonly path: string;
    readonly paragraphDigestsRequired: boolean;
    readonly everyBodyBlockReviewed?: boolean;
  },
): void {
  if (paragraphDigestsRequired) {
    if (final.candidateText !== finalText) {
      throw new ArtifactParseError({
        path: `${path}.candidateText`,
        reason: 'exact final polish text',
      },);
    }
    return;
  }
  /**
   Exact reviewed paragraphs from legacy final text.
   */
  const paragraphs = reviewedParagraphsOf({
    text: finalText,
    everyBodyBlockReviewed,
  },);
  if (final.paragraphCount !== paragraphs.length) {
    throw new ArtifactParseError({
      path: `${path}.paragraphCount`,
      reason: 'structurally correctable paragraph count of final polish text',
    },);
  }
  if (final.candidateDigest !== hashContent({ content: finalText, },)) {
    throw new ArtifactParseError({
      path: `${path}.candidateDigest`,
      reason: 'SHA-256 of final polish text',
    },);
  }
}

//endregion Artifact naturalness digest chain
