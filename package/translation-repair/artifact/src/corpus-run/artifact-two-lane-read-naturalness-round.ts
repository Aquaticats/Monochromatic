import { requireExactKeys, } from '../artifact-exact-guard.ts';
import { validNaturalnessQuorum, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import {
  ArtifactParseError,
  requireArray,
  requireCount,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  ArtifactNaturalnessFinding,
  ArtifactNaturalnessReviewRound,
} from './artifact-two-lane-consolidate.ts';
import {
  assertReviewedCandidateDigests,
  parseParagraphDigests,
  requireNaturalnessDigest,
} from './artifact-two-lane-read-naturalness-digest.ts';
import {
  naturalnessVerdictOf,
  parseNaturalnessFindings,
  parseNaturalnessReviewSeat,
  sameNaturalnessFindings,
  uniqueNaturalnessFindings,
} from './artifact-two-lane-read-naturalness-seat.ts';

//region Artifact absolute naturalness round read

/**
 Reads one candidate-bound absolute review round.

 @param value - unknown round

 @param path - artifact path

 @param paragraphDigestsRequired - whether generation binds reviewed paragraph identities

 @param everyBodyBlockReviewed - whether reviewed paragraphs are every body
 block (generation ten) rather than the refinable paragraphs alone

 @param quorumBasisRequired - whether generation records its wider quorum basis

 @param lastSeatGrace - whether generation decided its rounds with the
 last-seat grace of 2026-10-22 (generation twenty on)

 @returns Cross-validated review round

 @example
 ```ts
 const round = parseNaturalnessReviewRound({ value, path, paragraphDigestsRequired: true, });
 ```
 */
export function parseNaturalnessReviewRound(
  {
    value,
    path,
    paragraphDigestsRequired,
    everyBodyBlockReviewed = false,
    quorumBasisRequired = false,
    lastSeatGrace = false,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly paragraphDigestsRequired: boolean;
    readonly everyBodyBlockReviewed?: boolean;
    readonly quorumBasisRequired?: boolean;
    readonly lastSeatGrace?: boolean;
  },
): ArtifactNaturalnessReviewRound {
  /**
   Round under exact schema-eight shape.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      ...(quorumBasisRequired
        ? [
          'quorumOver',
          'unreachable',
        ]
        : []),
      'candidateDigest',
      ...(paragraphDigestsRequired ? ['candidateText',] : []),
      'paragraphCount',
      ...(paragraphDigestsRequired ? ['paragraphDigests',] : []),
      'seats',
      'usable',
      'verdict',
      'findings',
    ],
    path,
  },);
  /**
   Accounted seats in roster order.
   */
  const seats = requireArray({
    value: record.seats,
    path: `${path}.seats`,
  },)
    .map(function readSeat(
      entry,
      at,
    ) {
      return parseNaturalnessReviewSeat({
        value: entry,
        path: `${path}.seats[${String(at,)}]`,
      },);
    },);
  /**
   Structurally correctable paragraphs reviewer was shown.
   */
  const paragraphCount = requireCount({
    value: record.paragraphCount,
    path: `${path}.paragraphCount`,
  },);
  /**
   Reviewed paragraph identities under generation-nine shape.
   */
  const paragraphDigests = paragraphDigestsRequired
    ? parseParagraphDigests({
      value: record.paragraphDigests,
      path: `${path}.paragraphDigests`,
      paragraphCount,
    },)
    : [];
  if (seats.some(function outOfRange(seat,): boolean {
    /**
     Findings this seat attached to reviewed paragraphs.
     */
    const { findings: seatFindings, } = seat;
    return seatFindings.some(function missingParagraph(finding,): boolean {
      return finding.paragraph > paragraphCount;
    },);
  },)) {
    throw new ArtifactParseError({
      path: `${path}.seats`,
      reason: 'findings naming existing reviewed paragraph',
    },);
  }
  /**
   Model ids proving one status per seat.
   */
  const modelIds = seats.map(function modelIdOf(seat,): string {
    return seat.modelId;
  },);
  if (new Set(modelIds,).size !== modelIds.length) {
    throw new ArtifactParseError({
      path: `${path}.seats`,
      reason: 'one status per unique reviewer model id',
    },);
  }
  /**
   Usable count recomputed from seats.
   */
  const usable = seats.filter(function usableSeat(seat,): boolean {
    return seat.status !== 'unusable';
  },)
    .length;
  /**
   Stored usable count before equality check.
   */
  const storedUsable = requireCount({
    value: record.usable,
    path: `${path}.usable`,
  },);
  if (storedUsable !== usable) {
    throw new ArtifactParseError({
      path: `${path}.usable`,
      reason: `${String(usable,)} derived from seat statuses`,
    },);
  }
  /**
   Aggregate findings recomputed in roster order.
   */
  const findings = uniqueNaturalnessFindings({
    findings: seats.flatMap(function rejected(
      seat,
    ): readonly ArtifactNaturalnessFinding[] {
      return (seat.status === 'unacceptable') ? seat.findings : [];
    },),
  },);
  /**
   Stored aggregate findings.
   */
  const storedFindings = parseNaturalnessFindings({
    value: record.findings,
    path: `${path}.findings`,
  },);
  if (!sameNaturalnessFindings({
    left: findings,
    right: storedFindings,
  },)) {
    throw new ArtifactParseError({
      path: `${path}.findings`,
      reason: 'deduplicated unacceptable-seat findings in roster order',
    },);
  }
  /**
   Explicit wider basis in current records, seat-count convention in legacy records.
   */
  const quorumOver = quorumBasisRequired
    ? requireCount({
      value: record.quorumOver,
      path: `${path}.quorumOver`,
    },)
    : seats.length;
  if (!validNaturalnessQuorum({
    quorumOver,
    seatCount: seats.length,
  },)) {
    throw new ArtifactParseError({
      path: `${path}.quorumOver`,
      reason: 'safe integer quorum basis covering every recorded seat',
    },);
  }
  /**
   Bench seats the review counted out of reach, recorded since ledger E3 and
   none in a record written before; never more than the bench.
   */
  const unreachable = (quorumBasisRequired && (record.unreachable !== undefined))
    ? requireCount({
      value: record.unreachable,
      path: `${path}.unreachable`,
    },)
    : 0;
  if (unreachable > quorumOver) {
    throw new ArtifactParseError({
      path: `${path}.unreachable`,
      reason: `at most the ${String(quorumOver,)} ${
        wordForCount({
          count: quorumOver,
          one: 'seat',
          many: 'seats',
        },)
      } of the bench`,
    },);
  }
  /**
   Verdict recomputed using the same basis as the producing stage.
   */
  const verdict = naturalnessVerdictOf({
    seats,
    quorumOver,
    unreachable,
    lastSeatGrace,
  },);
  if (record.verdict !== verdict) {
    throw new ArtifactParseError({
      path: `${path}.verdict`,
      reason: `${verdict}, derived from seat statuses and quorum`,
    },);
  }
  /**
   Candidate digest under lowercase SHA-256 shape.
   */
  const candidateDigest = requireNaturalnessDigest({
    value: record.candidateDigest,
    path: `${path}.candidateDigest`,
  },);
  /**
   Exact reviewed candidate under generation-nine shape.
   */
  const candidateText = paragraphDigestsRequired
    ? requireString({
      value: record.candidateText,
      path: `${path}.candidateText`,
    },)
    : '';
  if (paragraphDigestsRequired) {
    assertReviewedCandidateDigests({
      candidateText,
      candidateDigest,
      paragraphCount,
      paragraphDigests,
      path,
      everyBodyBlockReviewed,
    },);
  }
  return {
    ...(quorumBasisRequired ? { quorumOver, } : {}),
    ...((record.unreachable === undefined) ? {} : { unreachable, }),
    candidateDigest,
    ...(paragraphDigestsRequired ? { candidateText, } : {}),
    paragraphCount,
    ...(paragraphDigestsRequired ? { paragraphDigests, } : {}),
    seats,
    usable,
    verdict,
    findings,
  };
}

//endregion Artifact absolute naturalness round read
