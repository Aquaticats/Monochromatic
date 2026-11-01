import {
  requireExactKeys,
  requireKeyOf,
} from '../artifact-exact-guard.ts';
import {
  requireArray,
  requireCount,
  requireRecord,
  requireString,
  requireStringList,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  type ArtifactConsistencyCandidate,
  type ArtifactConsistencyCarry,
  type ArtifactConsistencyCarryDecision,
  type ArtifactConsistencyChoiceBallot,
  type ArtifactConsistencyRunOff,
  CONSISTENCY_CANDIDATE_KINDS,
  CONSISTENCY_CARRY_DECISION_KINDS,
  CONSISTENCY_CARRY_NO_DECISION_REASONS,
} from './artifact-two-lane-consistency.ts';
import { parseConsistencyChoiceBallot, } from './artifact-two-lane-read-consistency-parts.ts';

//region Artifact consistency carry reading
// Reading the records of the sixteenth shape that the first round does not
// hold: the run-off among the finalists, and the carry of the
// chosen rendering to a receiving place (the candidates, the ballots that chose
// among them and what they decided), every key checked. What a carry says
// about its own ballots is recounted in
// `artifact-two-lane-read-consistency-carry-recount.ts`, and what it says about
// the group's ballots, renderings and outcomes is the coherence check's question
// (`artifact-two-lane-read-consistency-span.ts`).

/**
 Reads the ballots of a run-off or of a carry, each a seat's reason and choice.

 @param value - recorded ballots

 @param path - where they sit, for the parse error

 @returns The ballots, in the order recorded

 @throws {@link ArtifactParseError} when they are not a list of choices

 @example
 ```ts
 const ballots = parseChoiceBallots({ value: record.ballots, path: `${path}.ballots`, },);
 ```
 */
function parseChoiceBallots(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactConsistencyChoiceBallot[] {
  return requireArray({
    value,
    path,
  },)
    .map(function toBallot(
      entry,
      at,
    ) {
      return parseConsistencyChoiceBallot({
        value: entry,
        path: `${path}[${String(at,)}]`,
      },);
    },);
}

/**
 Reads the run-off of a group.

 @param value - recorded run-off

 @param path - where it sits, for the parse error

 @returns The run-off

 @throws {@link ArtifactParseError} when it is the wrong shape

 @example
 ```ts
 const runOff = parseConsistencyRunOff({ value: record.runOff, path: `${path}.runOff`, },);
 ```
 */
export function parseConsistencyRunOff(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactConsistencyRunOff {
  /**
   The run-off as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'finalists',
      'ballots',
      'usable',
      'quorum',
    ],
    path,
  },);
  return {
    finalists: requireStringList({
      value: record.finalists,
      path: `${path}.finalists`,
    },),
    ballots: parseChoiceBallots({
      value: record.ballots,
      path: `${path}.ballots`,
    },),
    usable: requireCount({
      value: record.usable,
      path: `${path}.usable`,
    },),
    quorum: requireCount({
      value: record.quorum,
      path: `${path}.quorum`,
    },),
  };
}

/**
 Reads one candidate of a carry.

 @param value - recorded candidate

 @param path - where it sits, for the parse error

 @returns The candidate

 @throws {@link ArtifactParseError} when it is the wrong shape

 @example
 ```ts
 const candidate = parseCandidate({ value, path: `${path}.candidates[0]`, },);
 ```
 */
function parseCandidate(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactConsistencyCandidate {
  /**
   The candidate as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'label',
      'kind',
      'writers',
      'text',
    ],
    path,
  },);
  return {
    label: requireString({
      value: record.label,
      path: `${path}.label`,
    },),
    kind: requireKeyOf({
      value: record.kind,
      record: CONSISTENCY_CANDIDATE_KINDS,
      path: `${path}.kind`,
    },),
    writers: requireStringList({
      value: record.writers,
      path: `${path}.writers`,
    },),
    text: requireString({
      value: record.text,
      path: `${path}.text`,
    },),
  };
}

/**
 Reads what a carry decided.

 @param value - recorded decision

 @param path - where it sits, for the parse error

 @returns The decision

 @throws {@link ArtifactParseError} when it is the wrong shape or kind

 @example
 ```ts
 const decision = parseDecision({ value: record.decision, path: `${path}.decision`, },);
 ```
 */
function parseDecision(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactConsistencyCarryDecision {
  /**
   The decision as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  /**
   How the carry ended.
   */
  const kind = requireKeyOf({
    value: record.kind,
    record: CONSISTENCY_CARRY_DECISION_KINDS,
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
      record: CONSISTENCY_CARRY_NO_DECISION_REASONS,
      path: `${path}.reason`,
    },),
  };
}

/**
 Reads one carry.

 @param value - recorded carry

 @param path - where it sits, for the parse error

 @returns The carry

 @throws {@link ArtifactParseError} when it is the wrong shape

 @example
 ```ts
 const carry = parseConsistencyCarry({ value, path: `${path}.carries[0]`, },);
 ```
 */
export function parseConsistencyCarry(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactConsistencyCarry {
  /**
   The carry as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'label',
      'candidates',
      'ballots',
      'usable',
      'quorum',
      'runOff',
      'decision',
    ],
    path,
  },);
  return {
    label: requireString({
      value: record.label,
      path: `${path}.label`,
    },),
    candidates: requireArray({
      value: record.candidates,
      path: `${path}.candidates`,
    },)
      .map(function toCandidate(
        entry,
        at,
      ) {
        return parseCandidate({
          value: entry,
          path: `${path}.candidates[${String(at,)}]`,
        },);
      },),
    ballots: parseChoiceBallots({
      value: record.ballots,
      path: `${path}.ballots`,
    },),
    usable: requireCount({
      value: record.usable,
      path: `${path}.usable`,
    },),
    quorum: requireCount({
      value: record.quorum,
      path: `${path}.quorum`,
    },),
    ...((record.runOff === undefined)
      ? {}
      : {
        runOff: parseConsistencyRunOff({
          value: record.runOff,
          path: `${path}.runOff`,
        },),
      }),
    decision: parseDecision({
      value: record.decision,
      path: `${path}.decision`,
    },),
  };
}

//endregion Artifact consistency carry reading
