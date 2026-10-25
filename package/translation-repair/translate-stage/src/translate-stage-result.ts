import type {
  Candidate,
  CandidateProducer,
  CandidateWeight,
  SelectionBallot,
  SelectionTally,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import type {
  RunoffBasis,
  TranslateCandidateValue,
  TranslateOrigin,
  TranslateSlateEntry,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { TieBasis, } from './translate-runoff-tie.ts';

//region Translate stage result
// What one slice's round DECIDED, apart from the code that decides it.
//
// Split from `translate-stage.ts` when that file reached its line budget, and
// along the seam that was already there: this is the record every later reader
// joins to, while the stage is the fan-out, the slate and the judging that
// produce it. A reader of an artifact needs this file and none of that one.

/**
 How a slice's shipped text was decided.

 Kept apart from the origin because "the incumbent shipped" and "the judges
 chose the incumbent" are different facts, and only the second is evidence
 about the incumbent. A tie, a lost round or an empty slate all ship the
 incumbent too, and counting those as wins would report the archive as
 vindicated by exactly the rounds that examined nothing.

 `unfloored` IS THE ONE DECISION NO ROUND MADE: the floor could compare
 nothing written for the slice, so no candidate could pass it, and the
 stage asked nobody (ledger B43). Its absence reason of the same name is
 the same fact where the archive has no wording.

 @example
 ```ts
 const decision: TranslateDecision = 'judged';
 ```
 */
export type TranslateDecision =
  | 'judged'
    | 'sole-candidate'
    | 'declined-indecision'
    | 'declined-rejection'
    | 'no-candidate-backed'
    | 'no-candidate'
    | 'no-voice-heard'
    | 'unfloored';

/**
 Everything the translate stage decided for one slice.

 @example
 ```ts
 const { text, origin, decision, } = await runTranslateStage({ ... },);
 ```
 */
export type TranslateStageResult = {
  /**
   Text that ships for this slice.
   */
  readonly text: string;

  /**
   Whether that text was already there.
   */
  readonly origin: TranslateOrigin;

  /**
   Who produced it.
   */
  readonly producer: CandidateProducer;

  /**
   How it was decided.
   */
  readonly decision: TranslateDecision;

  /**
   Ballot weight the winner drew, zero when no round decided it. A weight
   rather than a count because a judge voting for its own work counts for
   less; see `SELF_VOTE_WEIGHT`.
   */
  readonly voteWeight: number;

  /**
   What the judging round counted; zeros when none ran.
   */
  readonly tally: SelectionTally;

  /**
   Every ballot cast, so a replaced human translation carries the reasons it
   was replaced for rather than leaving them in a log.
   */
  readonly ballots: readonly SelectionBallot[];

  /**
   Translators whose reply arrived and validated.
   */
  readonly heardTranslators: number;

  /**
   Distinct proposals the judges saw, incumbent included.
   */
  readonly candidateCount: number;

  /**
   Voice loss, blank replies, incumbent matches and fallbacks, in
   scorecard-stable wording.
   */
  readonly findings: readonly string[];

  /**
   Candidates in the order the judges saw them, which is what makes a stored
   ballot readable: ballots name a position, and the slate is rotated per
   slice.

   The ASSEMBLED rotated order, whether or not judges were called. A slice
   with one distinct proposal ships it without a round and still records the
   slate, because what was on the ballot is the same question either way and
   `decision` already says whether anyone voted. Empty only when the slice had
   no candidates at all.
   */
  readonly slate: readonly TranslateSlateEntry[];

  /**
   Position the judges chose, or {@link NOT_ON_SLATE} when they chose nothing.
   */
  readonly selectedIndex: number;

  /**
   Candidates a tied round backed when fewer than the slate, so a challenge
   round can be a run-off over them where the incumbent was kept (class one
   hundred six, 2026-09-24). The same narrowing `TranslateAbsenceError`
   carries where there is no incumbent to keep (class fifty-three); absent
   where the round decided, rejected, or backed the whole slate.
   */
  readonly runoffFinalists?: readonly Candidate<TranslateCandidateValue>[];

  /**
   Why those are the finalists, present exactly when `runoffFinalists` is:
   the candidates tied at the top, or the leader and its runner-up where one
   candidate led and one judge alone named it (ledger B806).
   */
  readonly runoffBasis?: RunoffBasis;

  /**
   How a slate declined in its challenge round shipped anyway over wording
   that cannot ship (owner, 2026-09-27, "Preference + polish"): the
   preference that chose among the candidates two or more judges named (owner,
   2026-10-25, "Two judges must name it"), and the reasons of the ballots that
   did not back it. Absent wherever the judges chose, a tie was broken, or no
   candidate drew two ballots and the archive wording stayed.

   READ ONLY FOR THE CONSOLIDATION SLATE, whose reasons the final polish
   checks against the ORIGINAL (`consolidate-settle-gate.ts`). The translate
   lane's record carries them too and nothing reads them. Neither carries a
   decision seat's reason, a distribution rather than an objection (ledger
   B125).
   */
  readonly shippedPastDecline?: {
    readonly basis: TieBasis;
    readonly objections: readonly string[];
  };

  /**
   Position of the text that actually shipped, which differs from the
   selection on every fallback and is {@link NOT_ON_SLATE} when the shipped
   text was never a candidate, as a blank incumbent never is.
   */
  readonly shippedIndex: number;

  /**
   What each position drew, so a decline says by how much and against what.
   */
  readonly perCandidate: readonly CandidateWeight[];
};

/**
 Tally of a stage that never reached the judges.

 @example
 ```ts
 const tally = EMPTY_TALLY;
 ```
 */
export const EMPTY_TALLY: SelectionTally = {
  judgesAvailable: 0,
  ballots: 0,
  abstentions: 0,
  selfVotes: 0,
};

//endregion Translate stage result
