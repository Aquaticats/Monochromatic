import {
  requireExactKeys,
  requireOneOf,
} from '../artifact-exact-guard.ts';
import {
  requireArray,
  requireRecord,
  requireString,
  requireStringList,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  ARCHIVE_VERDICTS,
  CANDIDATE_NAMES,
  type LaneChoice,
  type LaneContestBallot,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';

//region Lane contest ballot reading
// Reading one judge`s ballot back out of a settled artifact.
//
// SPLIT FROM THE SLICE READER on the line budget, along the seam the wire
// module already draws: a ballot is a shape a judge produced, and a slice is
// what the roster made of a set of them.

/**
 Reads one list of candidate names a judge wrote.

 @param value - recorded list

 @param path - dotted path of that list

 @returns Candidate names it carries

 @throws {@link ArtifactParseError} when any entry names no candidate

 @example
 ```ts
 const named = parseChoiceList({ value, path, },);
 ```
 */
function parseChoiceList(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): readonly LaneChoice[] {
  return requireArray({
    value,
    path,
  },)
    .map(function readOne(
      one,
      position,
    ): LaneChoice {
      return requireOneOf({
        value: one,
        allowed: CANDIDATE_NAMES,
        path: `${path}[${String(position,)}]`,
      },);
    },);
}

/**
 Reads one judge`s ballot.

 @param value - recorded ballot

 @param path - dotted path of that ballot

 @returns Ballot as the judge left it

 @throws {@link ArtifactParseError} when any field is missing or unreadable

 @example
 ```ts
 const ballot = parseContestBallot({ value, path, },);
 ```
 */
export function parseContestBallot(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): LaneContestBallot {
  /**
   Ballot as a record, before any field is read.
   */
  const ballot = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record: ballot,
    allowed: [
      'choice',
      'unsupported',
      'unsupportedRaw',
      'dropped',
      'droppedRaw',
      'reason',
      'archive',
      'modelId',
    ],
    path,
  },);
  /**
   Seat that cast this ballot, present only on a ballot recorded with one.

   ABSENT ON AN OLDER ARTIFACT rather than refused, since a ballot settled
   before seats were recorded (hulicaijia26 and earlier) still carries a
   verdict the reader recomputes, and the seat never enters that verdict.
   */
  const seat = (ballot.modelId === undefined)
    ? {}
    : {
      modelId: requireString({
        value: ballot.modelId,
        path: `${path}.modelId`,
      },),
    };
  /**
   Archive verdict this ballot carries, present only when it recorded one.

   SPREAD RATHER THAN SET TO UNDEFINED, because the property is optional
   under `exactOptionalPropertyTypes`. A ballot written before the question
   existed carries no such key, and reading it back as absent is what makes
   the verdict derived from it agree with the one recorded beside it.
   */
  const archive = (ballot.archive === undefined)
    ? {}
    : {
      archive: requireOneOf({
        value: ballot.archive,
        allowed: ARCHIVE_VERDICTS,
        path: `${path}.archive`,
      },),
    };

  return {
    ...archive,
    ...seat,
    choice: requireOneOf({
      value: ballot.choice,
      allowed: CANDIDATE_NAMES,
      path: `${path}.choice`,
    },),
    unsupported: parseChoiceList({
      value: ballot.unsupported,
      path: `${path}.unsupported`,
    },),
    unsupportedRaw: requireStringList({
      value: ballot.unsupportedRaw,
      path: `${path}.unsupportedRaw`,
    },),
    dropped: parseChoiceList({
      value: ballot.dropped,
      path: `${path}.dropped`,
    },),
    droppedRaw: requireStringList({
      value: ballot.droppedRaw,
      path: `${path}.droppedRaw`,
    },),
    reason: requireString({
      value: ballot.reason,
      path: `${path}.reason`,
    },),
  };
}

//endregion Lane contest ballot reading
