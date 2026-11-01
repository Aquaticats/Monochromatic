import {
  requireExactKeys,
  requireKeyOf,
  requireOneOf,
} from '../artifact-exact-guard.ts';
import {
  ArtifactParseError,
  requireArray,
  requireCount,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type {
  ConsistencyWinner,
  NoWinnerReason,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import {
  artifactGenerationCountsBySpan,
  type TwoLaneArtifactGeneration,
} from './artifact-two-lane-contract.ts';
import {
  type ArtifactConsistencyVerdict,
  CONSISTENCY_NOT_ASKED_REASONS,
} from './artifact-two-lane-consistency.ts';
import { parseConsistencyRunOff, } from './artifact-two-lane-read-consistency-carry.ts';
import { parseConsistencyBallot, } from './artifact-two-lane-read-consistency-parts.ts';

//region Artifact consistency verdict reading
// Reading what was asked of the bench about one group and what it answered,
// key by key. What the verdict says about the group's places and ballots is
// the coherence check's question (`artifact-two-lane-read-consistency-coherence.ts`).

/**
 Every reason a group can have no winner, keyed by the union so the compiler
 refuses a member the record lacks.
 */
const NO_WINNER_REASONS: Readonly<Record<NoWinnerReason, true>> = {
  'no-quorum': true,
  'no-majority': true,
  'chosen-unlocated': true,
  'run-off-no-quorum': true,
  'run-off-tied': true,
};

/**
 Reasons only generation sixteen gives, after the bench was asked again among
 the finalists.
 */
const RUN_OFF_REASONS: ReadonlySet<NoWinnerReason> = new Set<NoWinnerReason>([
  'run-off-no-quorum',
  'run-off-tied',
],);

/**
 Reads the winner of a group the bench was asked about.

 @param value - recorded winner

 @param path - where it sits, for the parse error

 @param generation - generation the section was written under, which says
 whether a run-off reason exists

 @returns The winner

 @throws {@link ArtifactParseError} when it is the wrong shape or kind, or gives
 a run-off reason in a generation with no run-off

 @example
 ```ts
 const winner = parseConsistencyWinner({ value: record.winner, path: `${path}.winner`, generation: 16, },);
 ```
 */
function parseConsistencyWinner(
  {
    value,
    path,
    generation,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly generation: TwoLaneArtifactGeneration;
  },
): ConsistencyWinner {
  /**
   The winner as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  /**
   Whether a place won.
   */
  const kind = requireOneOf({
    value: record.kind,
    allowed: [
      'chosen',
      'none',
    ],
    path: `${path}.kind`,
  },);
  if (kind === 'chosen') {
    requireExactKeys({
      record,
      allowed: [
        'kind',
        'label',
      ],
      path,
    },);
    return {
      kind,
      label: requireString({
        value: record.label,
        path: `${path}.label`,
      },),
    };
  }
  requireExactKeys({
    record,
    allowed: [
      'kind',
      'reason',
    ],
    path,
  },);
  /**
   Why no place won.
   */
  const reason = requireKeyOf({
    value: record.reason,
    record: NO_WINNER_REASONS,
    path: `${path}.reason`,
  },);
  if (RUN_OFF_REASONS.has(reason,) && (!artifactGenerationCountsBySpan({ generation, },))) {
    throw new ArtifactParseError({
      path: `${path}.reason`,
      reason: 'a reason generation fifteen gives, which runs off no tie',
    },);
  }
  return {
    kind,
    reason,
  };
}

/**
 Reads what was asked of the bench about a group.

 @param value - recorded verdict

 @param path - where it sits, for the parse error

 @param generation - generation the section was written under, which says
 whether a verdict may carry a run-off

 @returns The verdict

 @throws {@link ArtifactParseError} when it is the wrong shape or kind, or
 carries a key or a reason of the other generation

 @example
 ```ts
 const verdict = parseConsistencyVerdict({ value: record.verdict, path: `${path}.verdict`, generation: 16, },);
 ```
 */
export function parseConsistencyVerdict(
  {
    value,
    path,
    generation,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly generation: TwoLaneArtifactGeneration;
  },
): ArtifactConsistencyVerdict {
  /**
   The verdict as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  /**
   Whether the bench was asked, and how that went.
   */
  const kind = requireOneOf({
    value: record.kind,
    allowed: [
      'not-asked',
      'failed',
      'asked',
    ],
    path: `${path}.kind`,
  },);
  if (kind === 'not-asked') {
    requireExactKeys({
      record,
      allowed: [
        'kind',
        'reason',
      ],
      path,
    },);
    return {
      kind,
      reason: requireKeyOf({
        value: record.reason,
        record: CONSISTENCY_NOT_ASKED_REASONS,
        path: `${path}.reason`,
      },),
    };
  }
  if (kind === 'failed') {
    requireExactKeys({
      record,
      allowed: [
        'kind',
        'detail',
      ],
      path,
    },);
    return {
      kind,
      detail: requireString({
        value: record.detail,
        path: `${path}.detail`,
      },),
    };
  }
  /**
   Whether the section has the sixteenth shape, whose verdict may carry a run-off.
   */
  const spanned = artifactGenerationCountsBySpan({ generation, },);
  requireExactKeys({
    record,
    allowed: spanned
      ? [
        'kind',
        'ballots',
        'usable',
        'quorum',
        'winner',
        'runOff',
      ]
      : [
        'kind',
        'ballots',
        'usable',
        'quorum',
        'winner',
      ],
    path,
  },);
  return {
    kind,
    ballots: requireArray({
      value: record.ballots,
      path: `${path}.ballots`,
    },)
      .map(function toBallot(
        entry,
        at,
      ) {
        return parseConsistencyBallot({
          value: entry,
          path: `${path}.ballots[${String(at,)}]`,
        },);
      },),
    usable: requireCount({
      value: record.usable,
      path: `${path}.usable`,
    },),
    quorum: requireCount({
      value: record.quorum,
      path: `${path}.quorum`,
    },),
    winner: parseConsistencyWinner({
      value: record.winner,
      path: `${path}.winner`,
      generation,
    },),
    ...((record.runOff === undefined)
      ? {}
      : {
        runOff: parseConsistencyRunOff({
          value: record.runOff,
          path: `${path}.runOff`,
        },),
      }),
  };
}

//endregion Artifact consistency verdict reading
