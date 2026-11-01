import {
  ArtifactParseError,
  requireArray,
  requireCount,
  requireFinite,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  requireKeyOf,
  requireOneOf,
} from '../artifact-exact-guard.ts';
import type {
  CandidateWeight,
  SelectionBallot,
  SelectionDisposition,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  REPAIR_ROUND_STAGES,
  type RepairJudgedRound,
  type RepairSlateEntry,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import { UnhandledMemberInvariantError, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { requireProducer, } from './artifact-producer-read.ts';
import {
  requireBallot,
  requireCandidateWeight,
} from './artifact-vote-read.ts';

//region Artifact rounds read
// Reads judged repair rounds back out of an artifact's raw lane result.
//
// WHY THIS IS NOT THE VERSION 2 PARSER'S JOB. `ParsedLane` deliberately hands
// back the lane's `result` as an unread record and checks only the core version
// 2 requires, saying in its own note that a reader wanting a field this version
// does not check should take it from the artifact. Rounds are such a field.
// Taking them means checking them here, because nothing else has.
//
// IT REFUSES AN ARTIFACT FROM AN OLDER ROSTER, by way of
// `artifact-producer-read.ts`, which names a departed model rather than reading
// it as current. That refusal is the point; the note there says why.
//
// A ROUND IS ALL-OR-NOTHING. Half a slate is worse evidence than none, because
// a standing divides by the ballots it saw and a dropped candidate moves every
// share on the round.
//
// A RESULT WITH NO `chunks` AT ALL IS ITS OWN ANSWER, not a malformed one. The
// repair lane recorded rounds only from a later build, so an artifact settled
// before that carries a complete, correct result that simply predates the field.
// Found by running the reader over the archives: 22 of 41 artifacts were absent
// this field and every one of them records `status: repaired`. Reporting those
// as parse failures would say 22 records are broken when none is, and would
// understate how much evidence a reader could hope to find.

/**
 Outcomes a recorded round can name, keyed by the kind of
 `RepairJudgedRound` rather than listed, so the compiler refuses an outcome
 the lane gains that this reader lacks. The order is the order a refusal names
 them in.
 */
const ROUND_KINDS: Readonly<Record<RepairJudgedRound['kind'], true>> = {
  selected: true,
  declined: true,
  adopted: true,
};

/**
 Reasons a round can decide nothing, keyed by `SelectionDisposition` for the
 reason `ROUND_KINDS` gives.
 */
const ROUND_DISPOSITIONS: Readonly<Record<SelectionDisposition, true>> = {
  indecision: true,
  rejection: true,
};

/**
 Reads one slate position.

 @param value - entry as recorded

 @param path - dotted path for error messages

 @returns Position with its provenance

 @example
 ```ts
 const entry = requireSlateEntry({ value, path, },);
 ```
 */
function requireSlateEntry(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): RepairSlateEntry {
  /**
   Entry as a record.
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
    rendered: requireString({
      value: record.rendered,
      path: `${path}.rendered`,
    },),
    hash: requireString({
      value: record.hash,
      path: `${path}.hash`,
    },),
    producer: requireProducer({
      value: record.producer,
      path: `${path}.producer`,
    },),
  };
}

/**
 Reads one judged round.

 @param value - round as recorded

 @param path - dotted path for error messages

 @returns Round in the shape the projection reads

 @throws {@link ArtifactParseError} when a field is missing or mistyped

 @throws {@link OffRosterModelError} when it names a departed model

 @throws {@link UnhandledMemberInvariantError} when a member of `RepairJudgedRound` kinds has no branch here, which the compiler rules out, after the kind has been read

 @example
 ```ts
 const round = requireJudgedRound({ value, path, },);
 ```
 */
function requireJudgedRound(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): RepairJudgedRound {
  /**
   Round as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);

  /**
   Stage that ran it, checked against the three the lane can name.
   */
  const stage = requireOneOf({
    value: record.stage,
    path: `${path}.stage`,
    allowed: REPAIR_ROUND_STAGES,
  },);

  /**
   Which of the three outcomes this round recorded, read before the vote
   fields because an adopted round has none.
   */
  const kind = requireKeyOf({
    value: record.kind,
    path: `${path}.kind`,
    record: ROUND_KINDS,
  },);

  /**
   Envelope the round decided.
   */
  const envelopeId = requireString({
    value: record.envelopeId,
    path: `${path}.envelopeId`,
  },);

  /**
   Slate as written, every entry checked.
   */
  const slate = requireArray({
    value: record.slate,
    path: `${path}.slate`,
  },)
    .map(function one(
      entry,
      index,
    ): RepairSlateEntry {
      return requireSlateEntry({
        value: entry,
        path: `${path}.slate[${String(index,)}]`,
      },);
    },);

  /**
   Tally as a record, whose four counts are read individually.
   */
  const tally = requireRecord({
    value: record.tally,
    path: `${path}.tally`,
  },);

  /**
   Everything both outcomes record, read once.

   SPLIT FROM THE PER-OUTCOME BRANCHES because the two outcomes agree on six fields
   and differ on two, and reading the six twice is how one of the copies
   drifts.
   */
  const common = {
    stage,
    envelopeId,
    slate,
    ballots: requireArray({
      value: record.ballots,
      path: `${path}.ballots`,
    },)
      .map(function one(
        entry,
        index,
      ): SelectionBallot {
        return requireBallot({
          value: entry,
          path: `${path}.ballots[${String(index,)}]`,
        },);
      },),
    tally: {
      judgesAvailable: requireCount({
        value: tally.judgesAvailable,
        path: `${path}.tally.judgesAvailable`,
      },),
      ballots: requireCount({
        value: tally.ballots,
        path: `${path}.tally.ballots`,
      },),
      abstentions: requireCount({
        value: tally.abstentions,
        path: `${path}.tally.abstentions`,
      },),
      selfVotes: requireCount({
        value: tally.selfVotes,
        path: `${path}.tally.selfVotes`,
      },),
    },
    perCandidate: requireArray({
      value: record.perCandidate,
      path: `${path}.perCandidate`,
    },)
      .map(function one(
        entry,
        index,
      ): CandidateWeight {
        return requireCandidateWeight({
          value: entry,
          path: `${path}.perCandidate[${String(index,)}]`,
        },);
      },),
  };

  // THE TWO OUTCOMES CARRY DIFFERENT FIELDS, and reading a declined round's
  // `reason` off a selected one would find nothing. Branching here is what
  // makes the returned round the same shape the lane wrote, rather than a
  // partial one every later reader has to re-check.
  // AN ADOPTED ROUND HELD NO VOTE: its vote fields are present and empty, so
  // they are read like any round's, and only the winner and the reason are
  // its own.
  if (kind === 'adopted') {
    return {
      kind: 'adopted',
      ...common,
      selectedIndex: requireCount({
        value: record.selectedIndex,
        path: `${path}.selectedIndex`,
      },),
      reason: requireString({
        value: record.reason,
        path: `${path}.reason`,
      },),
    };
  }
  if (kind === 'declined')
    return {
      kind: 'declined',
      ...common,
      reason: requireString({
        value: record.reason,
        path: `${path}.reason`,
      },),
      disposition: requireKeyOf({
        value: record.disposition,
        path: `${path}.disposition`,
        record: ROUND_DISPOSITIONS,
      },),
    };

  if (kind === 'selected') {
    return {
      kind: 'selected',
      ...common,
      selectedIndex: requireCount({
        value: record.selectedIndex,
        path: `${path}.selectedIndex`,
      },),
      voteWeight: requireFinite({
        value: record.voteWeight,
        path: `${path}.voteWeight`,
      },),
    };
  }
  throw new UnhandledMemberInvariantError({
    what: 'recorded round outcome',
    member: kind,
  },);
}

/**
 Raised when a repair result predates rounds being recorded at all.

 SEPARATE FROM A PARSE FAILURE, and this is the whole point of the class. Such
 a result is complete and correct for the build that wrote it; it just cannot
 answer a question that build was never asked. A reader counting these apart
 from malformed ones reports a schema generation rather than a defect.

 @example
 ```ts
 throw new RoundsNotRecordedError({ path: 'Whiskerfold.lanes.repair.result', },);
 ```
 */
export class RoundsNotRecordedError extends Error {
  /**
   Declares this message safe to forward: it names an artifact path and nothing inside it.
   */
  readonly messageNamesOnly: true = true;

  /**
   @param path - where in the artifact the absent field would sit
   */
  public constructor(
    { path, }: { readonly path: string; },
  ) {
    super(
      `${path}.chunks is absent, so this repair result was written before the lane recorded `
        + 'rounds and carries no ballots to read. It is an earlier shape, not a malformed record',
    );
    this.name = 'RoundsNotRecordedError';
  }
}

/**
 Reads every round every chunk of one raw repair result recorded.

 GROUPED BY CHUNK rather than flattened, because a standing drawn almost
 entirely from one chunk reads the same as one drawn evenly across many, and
 only the grouping tells them apart.

 @param raw - lane result exactly as the artifact holds it

 @param path - dotted path for error messages

 @returns One list of rounds per chunk, in chunk order

 @throws {@link ArtifactParseError} when the result or a round is malformed

 @throws {@link OffRosterModelError} when any record names a departed model

 @example
 ```ts
 const perChunk = readRepairRounds({ raw, path: 'lanes.repair.result', },);
 ```
 */
export function readRepairRounds(
  {
    raw,
    path,
  }: {
    readonly raw: Readonly<Record<string, unknown>>;
    readonly path: string;
  },
): readonly (readonly RepairJudgedRound[])[] {
  if (!('chunks' in raw))
    throw new RoundsNotRecordedError({ path, },);

  return requireArray({
    value: raw.chunks,
    path: `${path}.chunks`,
  },)
    .map(function one(
      chunk,
      index,
    ): readonly RepairJudgedRound[] {
    /**
     Where this chunk sits, for every message below it.
     */
      const at = `${path}.chunks[${String(index,)}]`;

    /**
     Chunk as a record.
     */
      const record = requireRecord({
        value: chunk,
        path: at,
      },);

      return requireArray({
        value: record.rounds,
        path: `${at}.rounds`,
      },)
        .map(function toRound(
          round,
          roundIndex,
        ): RepairJudgedRound {
          return requireJudgedRound({
            value: round,
            path: `${at}.rounds[${String(roundIndex,)}]`,
          },);
        },);
    },);
}

//endregion Artifact rounds read
