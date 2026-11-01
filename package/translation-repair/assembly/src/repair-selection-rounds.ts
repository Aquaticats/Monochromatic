import type { CandidateProducer, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  REPAIR_ROUND_STAGES,
  type RepairJudgedRound,
  type RepairRoundStage,
} from './repair-round-record.ts';
import type { SelectionRound, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Repair selection rounds
// Projects the repair lane's recorded rounds into the shape
// `producer-standing.ts` counts, so an EDITOR can be ranked by the same
// instrument that ranks a writer.
//
// This was once believed to need either a schema change or a replay
// path, because a settled artifact exposes neither the envelopes nor the
// issues an editor worked from. Reading the contracts says otherwise:
// `ChunkRepairOutcome.rounds` already carries, per round, the slate judges saw
// with each candidate's producer attached, and every ballot cast over it. That
// is exactly `SelectionRound`, one re-shape away, and both types name the same
// `CandidateProducer` and `SelectionBallot` out of `candidate-select-model.ts`.
//
// SO THE CALIBRATION DRIVES THE LANE LIVE AND READS ITS ROUNDS. Nothing is
// replayed, nothing is stored, and no artifact field changes.
//
// A DECLINED ROUND COUNTS. Judges saw that slate and cast those ballots, and a
// standing counts votes rather than wins: dropping the rounds that failed to
// resolve would keep only the slates judges found easy, and would flatter
// whichever model happened to write in them. A judge that named nothing is
// recorded as `CANDIDATE_NONE`, so it lands in the denominator and gives no
// one a vote, which is the honest reading of an abstention.
//
// STAGES ARE SEPARATED, because they are different jobs. The envelope and
// chunk-patch rounds are the editor's; the refine rounds belong to the
// naturalness lane. Seating one on the other's evidence is the mistake this
// file exists to make impossible.

/**
 Which seat produces the rounds of each stage.

 A RECORD OVER EVERY STAGE, so a stage the repair lane gains cannot be left
 out of both `EDITOR_ROUND_STAGES` and `REFINER_ROUND_STAGES`: the compiler
 refuses it here until a seat is named.
 */
const ROUND_STAGE_SEATS: Readonly<Record<RepairRoundStage, 'editor' | 'refiner'>> = {
  envelope: 'editor',
  'chunk-patch': 'editor',
  refine: 'refiner',
};

/**
 Stages whose rounds an editor produced: every stage `ROUND_STAGE_SEATS`
 gives to an editor, which today are `envelope` and `chunk-patch`. Together
 with {@link REFINER_ROUND_STAGES} it names every stage, once.

 @example
 ```ts
 const editorStages = EDITOR_ROUND_STAGES; // ['envelope', 'chunk-patch']
 ```
 */
export const EDITOR_ROUND_STAGES: readonly RepairRoundStage[] = REPAIR_ROUND_STAGES
  .filter(function isEditors(stage,): boolean {
    return ROUND_STAGE_SEATS[stage] === 'editor';
  },);

/**
 Stages whose rounds a refiner produced: every stage `ROUND_STAGE_SEATS` gives
 to a refiner, which today is `refine`. Together with
 {@link EDITOR_ROUND_STAGES} it names every stage, once.

 @example
 ```ts
 const refinerStages = REFINER_ROUND_STAGES; // ['refine']
 ```
 */
export const REFINER_ROUND_STAGES: readonly RepairRoundStage[] = REPAIR_ROUND_STAGES
  .filter(function isRefiners(stage,): boolean {
    return ROUND_STAGE_SEATS[stage] === 'refiner';
  },);

/**
 Raised when a recorded slate's positions are not the positions judges were
 shown.

 A BALLOT IS A NUMBER, so this is the one assumption the projection cannot
 check later. `SelectionRound.producers` is read positionally, a ballot's
 `best` being a one-based index into it, so a slate whose `index` values are
 not exactly one to its length would silently credit the wrong model.
 */
export class SlatePositionsError extends Error {
  /**
   Declares this message safe to forward: it names positions and counts, never a candidate's wording.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the positions the slate carried.

   @param detail - what the slate claimed, and why that cannot be read

   @example
   ```ts
   throw new SlatePositionsError({ detail: 'positions 1,3 over 2 candidates', },);
   ```
   */
  public constructor(
    { detail, }: { readonly detail: string; },
  ) {
    super(`recorded slate cannot be read positionally: ${detail}`,);
    this.name = 'SlatePositionsError';
  }
}

/**
 Projects one recorded round into the shape a standing counts.

 @param round - recorded round, selected or declined

 @returns Producers in slate order, plus every ballot cast over them

 @throws {@link SlatePositionsError} when the slate's positions are not one
 to its length

 @example
 ```ts
 const projected = selectionRoundOf({ round, },);
 ```
 */
export function selectionRoundOf(
  { round, }: { readonly round: RepairJudgedRound; },
): SelectionRound {
  /**
   Slate in the order judges were shown it.
   */
  const ordered = round
    .slate
    .toSorted(function byPosition(
      left,
      right,
    ): number {
      return left.index - right.index;
    },);

  /**
   Positions the slate claims, which must be one to its length.
   */
  const positions = ordered.map(function toPosition(entry,): number {
    return entry.index;
  },);

  /**
   Positions a slate of this size must carry.
   */
  const expected = ordered.map(function toExpected(
    _entry,
    at,
  ): number {
    return at + 1;
  },);

  if (positions.join(',',) !== expected.join(',',)) {
    throw new SlatePositionsError({
      detail: `slate of ${String(ordered.length,)} ${
        wordForCount({
          count: ordered.length,
          one: 'candidate',
          many: 'candidates',
        },)
      } carries positions `
        + `${positions.join(',',)}, and a ballot names a position by number, so a `
        + 'standing read off it would credit the wrong model',
    },);
  }

  return {
    producers: ordered.map(function toProducer(entry,): CandidateProducer {
      return entry.producer;
    },),
    ballots: round.ballots,
  };
}

/**
 Projects every round one role produced.

 @param rounds - rounds recorded by a chunk repair

 @param stages - stages belonging to the role being ranked

 @returns Rounds in the shape a standing counts

 @throws {@link SlatePositionsError} when any slate's positions are not one
 to its length

 @example
 ```ts
 const rounds = selectionRoundsFor({ rounds: outcome.rounds, stages: EDITOR_ROUND_STAGES, },);
 ```
 */
export function selectionRoundsFor(
  {
    rounds,
    stages,
  }: {
    readonly rounds: readonly RepairJudgedRound[];
    readonly stages: readonly RepairRoundStage[];
  },
): readonly SelectionRound[] {
  return rounds
    .filter(function isRole(round,): boolean {
      return stages.includes(round.stage,);
    },)
    .map(function project(round,): SelectionRound {
      return selectionRoundOf({ round, },);
    },);
}

//endregion Repair selection rounds
