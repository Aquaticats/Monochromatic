import type {
  ReadCandidate,
  ReadRound,
} from './ledger-parse.ts';

//region Ledger read
// READS THE JUDGED-CONTEST LEDGER BACK, and joins each ballot to the model it
// was about.
//
// THE OTHER HALF OF `candidate-ledger.ts`. Writing this half is what a writer
// needs to be trusted: the spend line looked finished until its reader was
// written, and writing the reader found two real defects in the writer.
//
// THE JOIN IS THE WHOLE POINT. A ballot names a POSITION, not a model. Nothing
// before this could say which model a judge was talking about when it explained
// why it did not pick something, which is exactly the evidence a roster
// question needs. `candidates[best - 1].producers` is that join, and it is
// one-based because the slate the judges saw was.
//
// A BALLOT NAMING NOTHING AND A BALLOT NAMING A CANDIDATE THAT DOES NOT EXIST
// ARE DIFFERENT, and neither is dropped. `SelectionBallot` says an index past
// the end is recorded rather than corrected, because a judge naming a candidate
// that is not there IS the finding. A reader that silently skipped either would
// report a cleaner contest than the one that happened.
//
// A MODEL'S OWN BALLOT FOR ITS OWN CANDIDATE IS NOT EVIDENCE ABOUT IT, and is
// counted separately, matching what `producer-standing.ts` does with the same
// rounds.

/**
 Index a ballot carries when its judge named no candidate at all.
 */
const CANDIDATE_NONE = 0;

/**
 One ballot beside the slate it was cast over.

 Named rather than inferred so the pair is readonly all the way down: the
 counts taken over it only ever read.

 @example
 ```ts
 const entry: BallotInContest = { ballot, candidates, };
 ```
 */
type BallotInContest = {
  /**
   Ballot as recorded.
   */
  readonly ballot: ReadRound['ballots'][number];

  /**
   Slate it was cast over, so a position can be resolved to its authors.
   */
  readonly candidates: readonly ReadCandidate[];
};

/**
 What one model did across a set of contests.

 @example
 ```ts
 const work: ModelWork = { model: 'minimax-m3', candidates: 12, wins: 3, votes: 9, ballots: 84, selfVotes: 2, };
 ```
 */
export type ModelWork = {
  /**
   Model as the ledger recorded it.
   */
  readonly model: string;

  /**
   Candidates it had a hand in, joint ones included.
   */
  readonly candidates: number;

  /**
   Contests where a candidate it wrote was chosen.
   */
  readonly wins: number;

  /**
   Ballots from judges with no stake that named its candidate.
   */
  readonly votes: number;

  /**
   Ballots from judges with no stake cast over its candidates.
   */
  readonly ballots: number;

  /**
   Occasions it named a candidate it helped write.
   */
  readonly selfVotes: number;
};

/**
 What a set of contests came to.

 @example
 ```ts
 const summary = summariseLedger({ rounds, },);
 ```
 */
export type LedgerSummary = {
  /**
   One entry per model, by share of the ballots cast over its work.
   */
  readonly models: readonly ModelWork[];

  /**
   Contests read.
   */
  readonly rounds: number;

  /**
   Ballots whose judge named nothing, which is an abstention.
   */
  readonly abstentions: number;

  /**
   Ballots naming a position the slate did not have.

   KEPT SEPARATE FROM ABSTENTIONS, because a judge naming a candidate that is
   not there is a defect in the judge and an abstention is not.
   */
  readonly namedMissing: number;
};

/**
 One candidate a model wrote, with what judges said about it.

 @example
 ```ts
 const shown: CandidateReading = { task: 'render this passage', rendered: '...', won: false, remarks: [], };
 ```
 */
export type CandidateReading = {
  /**
   What the judges were deciding.
   */
  readonly task: string;

  /**
   Exactly what this model put in front of them.
   */
  readonly rendered: string;

  /**
   Whether it was chosen.
   */
  readonly won: boolean;

  /**
   Every disinterested judge's stated reason for naming this candidate.
   */
  readonly remarks: readonly string[];
};

/**
 Names the models behind the candidate a ballot picked.

 @param candidates - slate as the judges saw it

 @param best - one-based position the ballot named

 @returns Models behind it, empty where the ballot named a position the slate
 did not have

 @throws Error where the ballot named nothing, which the caller sets apart
 before asking since an empty answer would count it as a position the slate
 lacked

 @example
 ```ts
 const named = producersNamed({ candidates, best: 2, },);
 ```
 */
function producersNamed(
  {
    candidates,
    best,
  }: {
    readonly candidates: readonly ReadCandidate[];
    readonly best: number;
  },
): readonly string[] {
  if (best === CANDIDATE_NONE)
    throw new Error(
      'unreachable: the ballot named nothing, which summariseLedger sets apart before asking who is behind a '
        + 'position, so an empty answer here would count an abstention as a position the slate lacked',
    );

  /**
   Candidate at that position, absent where the ballot overran the slate.
   */
  const at = candidates[best - 1];

  return (at === undefined) ? [] : at.producers;
}

/**
 Folds one contest into a running per-model tally.

 @param tally - running counts, mutated in place

 @param round - contest to fold in

 @example
 ```ts
 foldRound({ tally, round, },);
 ```
 */
function foldRound(
  {
    tally,
    round,
  }: {
    readonly tally: Map<string, ModelWork>;
    readonly round: ReadRound;
  },
): void {
  for (const candidate of round.candidates) {
    /**
     Judges with no hand in this candidate, who are the only ones whose
     opinion of it is evidence.
     */
    const disinterested = round
      .ballots
      .filter(function noStake(ballot,): boolean {
        return !candidate
          .producers
          .includes(ballot.modelId,);
      },);

    /**
     Those that named it.
     */
    const naming = disinterested.filter(function named(ballot,): boolean {
      return ballot.best === candidate.index;
    },);

    /**
     Ballots from its own authors that named it.
     */
    const own = round
      .ballots
      .filter(function stake(ballot,): boolean {
        return candidate
          .producers
          .includes(ballot.modelId,)
          && (ballot.best === candidate.index);
      },);

    for (const model of candidate.producers) {
      /**
       This model's counts before this candidate.
       */
      const running = tally.get(model,) ?? {
        model,
        candidates: 0,
        wins: 0,
        votes: 0,
        ballots: 0,
        selfVotes: 0,
      };

      tally.set(
        model,
        {
          model,
          candidates: running.candidates + 1,
          wins: running.wins + ((round.selectedIndex === candidate.index) ? 1 : 0),
          votes: running.votes + naming.length,
          ballots: running.ballots + disinterested.length,
          selfVotes: running.selfVotes + own.length,
        },
      );
    }
  }
}

/**
 Totals what every model did across a set of contests.

 @param rounds - contests as the ledger recorded them

 @returns Per-model counts, plus the two ballot faults kept apart

 @example
 ```ts
 const summary = summariseLedger({ rounds, },);
 ```
 */
export function summariseLedger(
  { rounds, }: { readonly rounds: readonly ReadRound[]; },
): LedgerSummary {
  /**
   Running counts, keyed by model.
   */
  const tally = new Map<string, ModelWork>();

  for (const round of rounds) {
    foldRound({
      tally,
      round,
    },);
  }

  /**
   Every ballot across every contest, so the two faults can be counted once.
   */
  const everyBallot = rounds.flatMap(function ballotsOf(round,): readonly BallotInContest[] {
    return round
      .ballots
      .map(function paired(ballot,): BallotInContest {
        return {
          ballot,
          candidates: round.candidates,
        };
      },);
  },);

  /**
   Ballots whose judge named nothing at all.
   */
  const abstained = everyBallot.filter(function named(entry,): boolean {
    /**
     Position this judge named.
     */
    const { best, } = entry.ballot;

    return best === CANDIDATE_NONE;
  },);

  /**
   Ballots naming a position the slate they were cast over did not hold.
   */
  const overran = everyBallot.filter(function missing(entry,): boolean {
    /**
     Position this judge named.
     */
    const { best, } = entry.ballot;

    if (best === CANDIDATE_NONE)
      return false;

    /**
     Models behind that position, empty when the slate has nothing there.
     */
    const behind = producersNamed({
      candidates: entry.candidates,
      best,
    },);

    return behind.length === 0;
  },);

  /**
   Seats in the order they are reported, by how much each wrote.
   */
  const models = [...tally.values(),].toSorted(function mostWorkFirst(
    left,
    right,
  ): number {
    return right.candidates - left.candidates;
  },);

  return {
    models,
    rounds: rounds.length,
    abstentions: abstained.length,
    namedMissing: overran.length,
  };
}

/**
 Pulls out everything one model wrote, with what judges said about each piece.

 THE QUESTION THIS ANSWERS is whether a seat's low standing comes from writing
 something wrong or from writing something unremarkable. Only the text and the
 reasons can tell those apart.

 @param rounds - contests as the ledger recorded them

 @param model - seat to read

 @returns Its candidates, in the order they were judged

 @example
 ```ts
 const written = workOfModel({ rounds, model: 'minimax-m3', },);
 ```
 */
export function workOfModel(
  {
    rounds,
    model,
  }: {
    readonly rounds: readonly ReadRound[];
    readonly model: string;
  },
): readonly CandidateReading[] {
  return rounds.flatMap(function inRound(round,): readonly CandidateReading[] {
    return round
      .candidates
      .filter(function wroteIt(candidate,): boolean {
        return candidate
          .producers
          .includes(model,);
      },)
      .map(function reading(candidate,): CandidateReading {
        return {
          task: round.task,
          rendered: candidate.rendered,
          won: round.selectedIndex === candidate.index,
          remarks: round
            .ballots
            .filter(function about(ballot,): boolean {
              /**
               Whether this judge named this candidate.
               */
              const chose = ballot.best === candidate.index;

              /**
               Whether this judge had a hand in writing it.
               */
              const wrote = candidate
                .producers
                .includes(ballot.modelId,);

              return chose && (!wrote);
            },)
            .map(function said(ballot,): string {
              return `${ballot.modelId}: ${ballot.reason}`;
            },),
        };
      },);
  },);
}

//endregion Ledger read
