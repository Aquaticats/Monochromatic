import {
  ArtifactParseError,
  requireBoolean,
  requireCount,
  requireFinite,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type {
  CandidateWeight,
  SelectionBallot,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { requireRosterModelId, } from './artifact-producer-read.ts';

//region Artifact vote read
// Reads the two records a round keeps ABOUT VOTING: one ballot a judge cast, and
// what one slate position drew from all of them.
//
// Split out of `artifact-rounds-read.ts` when reading the whole round pushed that
// file past its line budget, and along the seam that was already there: nothing
// here knows what a round is, a stage is, or which outcome the round reached. A
// caller reading only ballots needs this file and none of that one.
//
// NEITHER READS A COUNT WHERE A WEIGHT BELONGS. A judge voting on its own work
// counts for half, so both a ballot's weight and a candidate's summed weight are
// routinely fractional, and a count guard would refuse records that are entirely
// well formed.

/**
 Reads what one slate position drew.

 WEIGHTS ARE READ AS FINITE, NOT AS COUNTS. A judge voting on its own work
 counts for half, so a candidate's summed weight is routinely fractional and
 a count guard would refuse a round that is entirely well formed.

 @param value - one entry of `perCandidate`, unread

 @param path - where in the artifact this sits, for the refusal

 @returns Counts and weight for that position

 @throws {@link ArtifactParseError} when a field is missing or mistyped

 @example
 ```ts
 const drawn = requireCandidateWeight({ value, path, },);
 ```

 @internal
 */
export function requireCandidateWeight(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): CandidateWeight {
  /**
   Position as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);

  return {
    index: requireCount({
      value: record.index,
      path: `${path}.index`,
    },),
    ballots: requireCount({
      value: record.ballots,
      path: `${path}.ballots`,
    },),
    fullVotes: requireCount({
      value: record.fullVotes,
      path: `${path}.fullVotes`,
    },),
    selfVotes: requireCount({
      value: record.selfVotes,
      path: `${path}.selfVotes`,
    },),
    weight: requireFinite({
      value: record.weight,
      path: `${path}.weight`,
    },),
  };
}

/**
 Reads one ballot.

 BEST AND WEIGHT ARE FINITE NUMBERS, NOT COUNTS. A judge naming no candidate
 records a sentinel index, and a judge naming its own writing records a
 fractional weight, so the count guard would refuse both.

 @param value - ballot as recorded

 @param path - dotted path for error messages

 @returns Ballot as the tally reads it

 @example
 ```ts
 const ballot = requireBallot({ value, path, },);
 ```

 @internal
 */
export function requireBallot(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): SelectionBallot {
  /**
   Ballot as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);

  return {
    modelId: requireRosterModelId({
      value: record.modelId,
      path: `${path}.modelId`,
    },),
    best: requireFinite({
      value: record.best,
      path: `${path}.best`,
    },),
    reason: requireString({
      value: record.reason,
      path: `${path}.reason`,
    },),
    weight: requireFinite({
      value: record.weight,
      path: `${path}.weight`,
    },),
    selfVote: requireBoolean({
      value: record.selfVote,
      path: `${path}.selfVote`,
    },),
  };
}

//endregion Artifact vote read
