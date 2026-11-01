import {
  ArtifactParseError,
  requireArray,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { ArtifactNaturalnessReviewRound, } from './artifact-two-lane-consolidate.ts';

//region Artifact naturalness confirmation read

/**
 Parser for one exact candidate review round.

 @example
 ```ts
 const parser: RoundParser = ({ value, path, }) => parseRound({ value, path, });
 ```
 */
type RoundParser = (input: {
  readonly value: unknown;
  readonly path: string;
}) => ArtifactNaturalnessReviewRound;

/**
 A confirmation beside the decisive review of its exact candidate.
 */
type ConfirmationMatch = {
  /**
   Earlier acceptable reading.
   */
  readonly confirmation: ArtifactNaturalnessReviewRound;

  /**
   Decisive review of the same candidate and paragraphs.
   */
  readonly decisive: ArtifactNaturalnessReviewRound;
};

/**
 Reads and binds optional acceptance confirmations added within schema nine.

 Absence remains readable for historical schema-nine artifacts.
 Presence opts into repeated-acceptance invariant and must confirm final text.

 @param value - unknown confirmation array

 @param present - whether artifact explicitly carries confirmation key

 @param rounds - decisive candidate reviews in correction order

 @param final - last of those reviews, which authorizes publication

 @param path - artifact review path

 @param parseRound - exact schema-nine round parser

 @returns Earlier acceptable same-candidate reviews

 @example
 ```ts
 const confirmations = parseNaturalnessConfirmations({ value, present: true, rounds, final, path, parseRound, });
 ```
 */
export function parseNaturalnessConfirmations(
  {
    value,
    present,
    rounds,
    final,
    path,
    parseRound,
  }: {
    readonly value: unknown;
    readonly present: boolean;
    readonly rounds: readonly ArtifactNaturalnessReviewRound[];
    readonly final: ArtifactNaturalnessReviewRound;
    readonly path: string;
    readonly parseRound: RoundParser;
  },
): readonly ArtifactNaturalnessReviewRound[] {
  if (!present)
    return [];

  /**
   Earlier acceptable readings over exact decisive-round candidates.
   */
  const confirmations = requireArray({
    value,
    path: `${path}.confirmations`,
  },)
    .map(function readConfirmation(
      entry,
      at,
    ): ArtifactNaturalnessReviewRound {
      return parseRound({
        value: entry,
        path: `${path}.confirmations[${String(at,)}]`,
      },);
    },);
  if (confirmations.some(function rejectedConfirmation(round,): boolean {
    return round.verdict !== 'acceptable';
  },)) {
    throw new ArtifactParseError({
      path: `${path}.confirmations`,
      reason: 'acceptable earlier readings before decisive same-candidate review',
    },);
  }

  /**
   Candidate identities carrying one earlier acceptable reading at most.
   */
  const confirmationDigests = confirmations.map(function digestOf(round,): string {
    return round.candidateDigest;
  },);
  if (new Set(confirmationDigests,).size !== confirmationDigests.length) {
    throw new ArtifactParseError({
      path: `${path}.confirmations`,
      reason: 'at most one acceptance confirmation per reviewed candidate',
    },);
  }
  /**
   Each confirmation beside the decisive review of its exact candidate,
   refusing one that matches none and one that matches more than one.
   */
  const matches = confirmations.map(function matchOf(
    confirmation,
    at,
  ): ConfirmationMatch {
    /**
     Decisive reviews of the same candidate and paragraphs. More than one
     means a correction chain returned to a candidate it had reviewed, and a
     confirmation that fits two rounds names neither.
     */
    const sameCandidates = rounds.filter(function sameCandidate(round,): boolean {
      return (round.candidateDigest === confirmation.candidateDigest)
        && (round.candidateText === confirmation.candidateText)
        && (JSON.stringify(round.paragraphDigests,)
          === JSON.stringify(confirmation.paragraphDigests,));
    },);
    /**
     The first of them, the only one when the confirmation is sound.
     */
    const [decisive,] = sameCandidates;
    if (decisive === undefined) {
      throw new ArtifactParseError({
        path: `${path}.confirmations`,
        reason: 'exact candidate and paragraph identities of one decisive review round',
      },);
    }
    if (sameCandidates.length > 1) {
      throw new ArtifactParseError({
        path: `${path}.confirmations[${String(at,)}]`,
        reason: 'a confirmation that fits one decisive review round, not several',
      },);
    }
    return {
      confirmation,
      decisive,
    };
  },);
  /**
   Decisive-round position of each confirmation.
   */
  const confirmedRoundIndexes = matches.map(function positionOf({ decisive, },): number {
    return rounds.indexOf(decisive,);
  },);
  if (confirmedRoundIndexes.some(function outOfOrder(
    index,
    at,
  ): boolean {
    /**
     Prior confirmation's decisive position, none before the first.
     */
    const previous = confirmedRoundIndexes[at - 1];
    return (previous !== undefined) && (index <= previous);
  },)) {
    throw new ArtifactParseError({
      path: `${path}.confirmations`,
      reason: 'same candidate order as decisive review rounds',
    },);
  }
  if (matches.some(function differentRoster({
    confirmation,
    decisive,
  },): boolean {
    /**
     Requested reviewer identities in stable roster order.
     */
    const confirmationRoster = confirmation
      .seats
      .map(function modelIdOf(seat,): string {
        return seat.modelId;
      },);
    /**
     Decisive review's requested identities in same order.
     */
    const decisiveRoster = decisive
      .seats
      .map(function modelIdOf(seat,): string {
        return seat.modelId;
      },);
    return (JSON.stringify(confirmationRoster,) !== JSON.stringify(decisiveRoster,))
      || (confirmation.quorumOver !== decisive.quorumOver);
  },)) {
    throw new ArtifactParseError({
      path: `${path}.confirmations`,
      reason: 'same requested reviewer roster and quorum basis as decisive review',
    },);
  }

  // The repeated-acceptance invariant binds only an accepted final: a
  // rejected or quorumless final round is recorded evidence under the
  // no-loop design and legitimately carries no earlier acceptable reading.
  if ((final.verdict === 'acceptable')
    && (!confirmationDigests.includes(final.candidateDigest,)))
    {
    throw new ArtifactParseError({
      path: `${path}.confirmations`,
      reason: 'earlier acceptable reading of exact final candidate',
    },);
  }
  return confirmations;
}

//endregion Artifact naturalness confirmation read
