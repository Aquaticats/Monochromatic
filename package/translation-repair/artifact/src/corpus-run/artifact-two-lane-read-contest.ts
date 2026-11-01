import {
  requireExactKeys,
  requireKeyOf,
} from '../artifact-exact-guard.ts';
import {
  type ArtifactComparisonRow,
  ArtifactParseError,
  requireArray,
  requireCount,
  requireRecord,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  type ArtifactContestSlice,
  type ArtifactLaneSelection,
  contestEligibleIndexes,
  describeContestSlice,
  describeSoleLaneSlice,
  type LaneContestBallot,
  settleEligibleLaneContestBallots,
  soleLaneOf,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';
import { parseContestBallot, } from './artifact-two-lane-read-contest-ballot.ts';
import { parseRecordedTenseCandidates, } from './artifact-two-lane-read-tense.ts';
import { CONTEST_TENSE_LABELS, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import {
  contestEligibilityRequired,
  parseContestEligibility,
} from './artifact-two-lane-read-contest-eligibility.ts';
import { parseContestFloorMark, } from './artifact-two-lane-read-contest-floor.ts';
import { assertContestVerdictMatches, } from './artifact-two-lane-read-contest-verdict.ts';
import type { ArtifactKeyVocabulary, } from '../artifact-key-vocabulary.ts';
import {
  ARTIFACT_SCHEMA_VERSION_V6,
  ARTIFACT_SCHEMA_VERSION_V7,
  artifactGenerationRecordsBenchFloor,
  artifactGenerationRecordsTense,
  artifactGenerationSettlesSoleLane,
  type TwoLaneArtifactGeneration,
} from './artifact-two-lane-contract.ts';

//region Lane contest reading
// Reading the recorded contest, and refusing one that disagrees with either the
// ballots stored beside it or the comparison the reader recomputed.
//
// THE VERDICT IS RECOMPUTED, not trusted. It is a claim about ballots this same
// record carries, so the reader settles those ballots by the stage`s own rule
// and refuses a stored verdict that says something else. This is the treatment
// the recorded lane comparison already gets, for the same reason: a derived
// field nobody re-derives is a field that can quietly become a lie.

/**
 Kinds a recorded selection may carry, keyed by the frozen
 `ArtifactLaneSelection` rather than listed, so the compiler refuses a kind
 the contest record gains that this reader lacks. The order is the order a
 refusal names them in.
 */
const SELECTION_KINDS: Readonly<Record<ArtifactLaneSelection['kind'], true>> = {
  'pending-human-decision': true,
  contested: true,
};

/**
 Reads one slice a single lane applies to, which carries no roster record.

 Refuses any ballot and any usable voice, since a roster was never asked about
 this slice and a record claiming otherwise would be a vote nobody cast. The
 verdict is re-derived from the comparison row alone and must be the one the
 file records.

 @param slice - recorded slice, already proven to carry the expected keys

 @param sliceIndex - slice index the record names, already read

 @param lane - lane the comparison row says applies alone

 @param path - dotted path of that slice

 @returns Slice record, proven to name the lane the row names

 @throws {@link ArtifactParseError} when the slice carries a ballot or a usable
 voice, or when its verdict is not the sole-lane one

 @example
 ```ts
 const slice = parseSoleLaneSlice({ slice, sliceIndex, lane, path, },);
 ```
 */
function parseSoleLaneSlice(
  {
    slice,
    sliceIndex,
    lane,
    path,
  }: {
    readonly slice: Readonly<Record<string, unknown>>;
    readonly sliceIndex: number;
    readonly lane: 'repair' | 'translate';
    readonly path: string;
  },
): ArtifactContestSlice {
  // A SLICE NO JUDGE WAS ASKED ABOUT was never kept for want of a bench either: the mark belongs to a slice the roster
  // would have been asked about, and is refused here rather than dropped unread.
  if (slice.benchBelowFloor !== undefined) {
    throw new ArtifactParseError({
      path: `${path}.benchBelowFloor`,
      reason: 'no floor mark, since no roster was asked about a slice one lane applies to',
    },);
  }
  // A SLICE NO JUDGE WAS ASKED ABOUT shows no sheet, so it has no tense
  // candidates to record; the key is refused rather than dropped unread.
  if (slice.tenseCandidates !== undefined) {
    throw new ArtifactParseError({
      path: `${path}.tenseCandidates`,
      reason: 'no tense candidates, since no roster was asked about a slice one lane applies to',
    },);
  }
  /**
   Ballots this slice carries, which must be none.
   */
  const ballots = requireArray({
    value: slice.ballots,
    path: `${path}.ballots`,
  },);
  if (ballots.length > 0)
    throw new ArtifactParseError({
      path: `${path}.ballots`,
      reason: 'no ballots, since no roster was asked about a slice one lane applies to',
    },);
  /**
   Voices the record claims, which must be none.
   */
  const usable = requireCount({
    value: slice.usable,
    path: `${path}.usable`,
  },);
  if (usable !== 0)
    throw new ArtifactParseError({
      path: `${path}.usable`,
      reason: `0, since no roster was asked about a slice one lane applies to, rather than ${String(usable,)}`,
    },);
  /**
   Verdict the comparison row settles on, which is the one the record must state.
   */
  const derived = describeSoleLaneSlice({
    sliceIndex,
    lane,
  },);
  assertContestVerdictMatches({
    value: slice.verdict,
    derived: derived.verdict,
    path: `${path}.verdict`,
  },);
  return derived;
}

/**
 Reads one contested slice and re-derives its verdict from its own ballots.

 @param value - recorded slice

 @param path - dotted path of that slice

 @param keys - field spellings this artifact's generation uses, so an older
 file is read by its own names rather than by today's

 @param generation - artifact generation deciding eligibility field support

 @param comparison - recomputed lane rows eligibility is checked against

 @returns Slice record, proven to agree with the ballots it carries

 @throws {@link ArtifactParseError} when a field is unreadable, when the usable
 count disagrees with the ballots, or when the stored verdict is not the one
 those ballots settle on

 @example
 ```ts
 const slice = parseContestSlice({ value, path, keys, generation, comparison, },);
 ```
 */
function parseContestSlice(
  {
    value,
    path,
    keys,
    generation,
    comparison,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly keys: ArtifactKeyVocabulary;
    readonly generation: TwoLaneArtifactGeneration;
    readonly comparison: readonly ArtifactComparisonRow[];
  },
): ArtifactContestSlice {
  /**
   Slice as a record, before any field is read.
   */
  const slice = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record: slice,
    allowed: [
      keys.sliceIndex,
      'verdict',
      'ballots',
      'usable',
      ...((generation >= ARTIFACT_SCHEMA_VERSION_V7) ? ['eligibility',] : []),
      // THE TENSE FINDING, optional from generation fifteen alone.
      ...(artifactGenerationRecordsTense({ generation, }) ? ['tenseCandidates',] : []),
      // THE FLOOR MARK, optional from generation twenty-one alone.
      ...(artifactGenerationRecordsBenchFloor({ generation, }) ? ['benchBelowFloor',] : []),
    ],
    path,
  },);

  /**
   Slice index this record answers.
   */
  const sliceIndex = requireCount({
    value: slice[keys.sliceIndex],
    path: `${path}.${keys.sliceIndex}`,
  },);
  /**
   Lane row carrying candidate texts for eligibility recomputation.
   */
  const row = comparison.find(function namesSlice(candidate,): boolean {
    return candidate.sliceIndex === sliceIndex;
  },);
  if (row === undefined) {
    throw new ArtifactParseError({
      path: `${path}.${keys.sliceIndex}`,
      reason: 'an index naming a slice the recomputed comparison holds',
    },);
  }
  /**
   Lane alone applying to this slice, absent where both lanes or neither did.
   */
  const sole = soleLaneOf({ row, },);

  /**
   Whether the generation records a sole-lane verdict at all.
   */
  const recordsSoleLane = artifactGenerationSettlesSoleLane({ generation, },);

  // A RECORD CARRYING ELIGIBILITY IS A ROSTER RECORD, which the writer makes for
  // every syntax-bearing slice whatever lanes apply, so it is read as one.
  /**
   Whether the record is a roster record by carrying eligibility.
   */
  const isRosterRecord = slice.eligibility !== undefined;
  /**
   Whether this slice takes the sole-lane path: its lane alone applies, the
   generation records that verdict, and the record is not a roster record.
   */
  const soleLaneRecorded = recordsSoleLane && (!isRosterRecord);
  if ((sole.kind === 'one-lane') && soleLaneRecorded) {
    return parseSoleLaneSlice({
      slice,
      sliceIndex,
      lane: sole.lane,
      path,
    },);
  }
  /**
   Source-backed syntax eligibility, absent on ordinary and older records.
   */
  const eligibility = (slice.eligibility === undefined)
    ? undefined
    : parseContestEligibility({
      value: slice.eligibility,
      row,
      path: `${path}.eligibility`,
    },);
  /**
   Whether artifact generation requires syntax eligibility evidence.
   */
  const eligibilityRequired = generation >= ARTIFACT_SCHEMA_VERSION_V7;
  if (eligibilityRequired
    && (eligibility === undefined)
    && contestEligibilityRequired({ row, })) {
    throw new ArtifactParseError({
      path: `${path}.eligibility`,
      reason: 'source-backed syntax eligibility record rather than absence',
    },);
  }

  /**
   Ballots this slice carries.
   */
  const ballots = requireArray({
    value: slice.ballots,
    path: `${path}.ballots`,
  },)
    .map(function readOne(
      one,
      position,
    ): LaneContestBallot {
      return parseContestBallot({
        value: one,
        path: `${path}.ballots[${String(position,)}]`,
      },);
    },);

  /**
   Count the record claims for those ballots.
   */
  const usable = requireCount({
    value: slice.usable,
    path: `${path}.usable`,
  },);
  if (usable !== ballots.length) {
    throw new ArtifactParseError({
      path: `${path}.usable`,
      reason: `${String(ballots.length,)}, which is how many ballots this slice carries, rather than ${
        String(usable,)
      }`,
    },);
  }

  /**
   Verdict the stored ballots settle on under the stage`s own rule.
   */
  const derived = describeContestSlice({
    sliceIndex,
    outcome: {
      choice: settleEligibleLaneContestBallots({
        ballots,
        ...((eligibility === undefined) ? {} : { eligibility, }),
      },),
      ballots,
      usable,
      findings: [],
    },
    ...((eligibility === undefined) ? {} : { eligibility, }),
  },);

  assertContestVerdictMatches({
    value: slice.verdict,
    derived: derived.verdict,
    path: `${path}.verdict`,
  },);
  /**
   Tense candidates the slice records as its audit finding, absent on a slice
   whose entry records no death or whose texts have no candidate.
   */
  const tenseCandidates = (slice.tenseCandidates === undefined)
    ? {}
    : {
      tenseCandidates: parseRecordedTenseCandidates({
        value: slice.tenseCandidates,
        path: `${path}.tenseCandidates`,
        labels: CONTEST_TENSE_LABELS,
      },),
    };
  return {
    ...derived,
    ...tenseCandidates,
    ...parseContestFloorMark({
      value: slice.benchBelowFloor,
      ballotCount: ballots.length,
      path: `${path}.benchBelowFloor`,
    },),
  };
}

/**
 Refuses a contest that does not answer exactly the slices where the two lanes
 left different wording.

 @param slices - records the contest carries

 @param comparison - rows the reader recomputed from both ledgers

 @param path - dotted path of the recorded slices

 @throws {@link ArtifactParseError} when the answered slices are not the
 eligible ones, in eligible order

 @example
 ```ts
 assertContestCoversEligible({ slices, comparison, path, },);
 ```
 */
function assertContestCoversEligible(
  {
    slices,
    comparison,
    path,
  }: {
    readonly slices: readonly ArtifactContestSlice[];
    readonly comparison: readonly ArtifactComparisonRow[];
    readonly path: string;
  },
): void {
  /**
   Slices a contest may answer, named by the comparison this reader derived.
   */
  const eligible = contestEligibleIndexes({ comparison, },)
    .join(',',);

  /**
   Slices it does answer.
   */
  const answered = slices
    .map(function nameIt(slice,): number {
      return slice.sliceIndex;
    },)
    .join(',',);
  if (answered !== eligible) {
    throw new ArtifactParseError({
      path,
      reason: `slices [${eligible}], which are the ones where the two lanes differ, rather than [${answered}]`,
    },);
  }
}

/**
 Reads which lane ships, and refuses a contest that does not cover exactly the
 slices where the two lanes left different wording.

 @param value - recorded selection

 @param comparison - rows the reader recomputed from both ledgers

 @param path - dotted path of the recorded selection

 @param keys - field spellings this artifact's generation uses, so an older
 file is read by its own names rather than by today's

 @param generation - artifact generation deciding eligibility field support

 @returns Selection, proven to agree with the ballots and the comparison

 @throws {@link ArtifactParseError} when the kind is unknown, when any slice is
 unreadable, or when the slices covered are not the eligible ones

 @example
 ```ts
 const selection = parseLaneSelection({ value, comparison, path, keys, },);
 ```
 */
export function parseLaneSelection(
  {
    value,
    comparison,
    path,
    keys,
    generation = ARTIFACT_SCHEMA_VERSION_V6,
  }: {
    readonly value: unknown;
    readonly comparison: readonly ArtifactComparisonRow[];
    readonly path: string;
    readonly keys: ArtifactKeyVocabulary;
    readonly generation?: TwoLaneArtifactGeneration;
  },
): ArtifactLaneSelection {
  /**
   Selection as a record, before its kind is known.
   */
  const selection = requireRecord({
    value,
    path,
  },);

  /**
   Kind it claims, which decides what else it may carry.
   */
  const kind = requireKeyOf({
    value: selection.kind,
    record: SELECTION_KINDS,
    path: `${path}.kind`,
  },);
  if (kind === 'pending-human-decision') {
    requireExactKeys({
      record: selection,
      allowed: ['kind',],
      path,
    },);
    return { kind, };
  }
  requireExactKeys({
    record: selection,
    allowed: [
      'kind',
      'slices',
    ],
    path,
  },);

  /**
   One record per slice the contest answered.
   */
  const slices = requireArray({
    value: selection.slices,
    path: `${path}.slices`,
  },)
    .map(function readOne(
      one,
      position,
    ): ArtifactContestSlice {
      return parseContestSlice({
        value: one,
        path: `${path}.slices[${String(position,)}]`,
        keys,
        generation,
        comparison,
      },);
    },);
  assertContestCoversEligible({
    slices,
    comparison,
    path: `${path}.slices`,
  },);
  return {
    kind,
    slices,
  };
}

//endregion Lane contest reading
