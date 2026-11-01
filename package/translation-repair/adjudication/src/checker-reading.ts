import type { ResolutionVerdict, } from './resolution-wire.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { IssueResolutionTally, } from './tally-resolution.ts';

//region Checker reading
// What the resolution checkers said about one issue, kept in full rather than
// summed away.
//
// WHY THE BOOLEAN WAS NOT ENOUGH. `RepairIssueRecord.resolved` is the milestone's
// headline number and the input to candidate selection, and it is a majority
// verdict. A settled artifact carrying only the verdict cannot say whether that
// majority was unanimous or one vote wide, cannot say who dissented, and cannot
// be re-read at another roster width without buying the whole stage again. The
// introduced-defect probe already keeps its tallies and roster sizes per issue
// for exactly this reason; the stage that decides what ships kept less.

/**
 One checker's answer on one issue, beside the fact that sets its weight.

 @example
 ```ts
 const ballot: IssueCheckerBallot = {
   modelId: 'hf:Qwen/Qwen3.8-27B',
   verdict: 'fixed',
   wroteTheText: false,
 };
 ```
 */
export type IssueCheckerBallot = {
  /**
   Checker that answered.
   */
  readonly modelId: RosterModelId;

  /**
   What it said about this issue.
   */
  readonly verdict: ResolutionVerdict;

  /**
   Whether this checker helped write the text it was judging.

   THE ONE WEIGHT INPUT THAT IS NOT IN THE VERDICT. `tallyResolutionChecks`
   chooses each weight per issue from the authorship record, so a reader
   holding the verdicts alone cannot reproduce the tally and a reader holding
   this too can, including at a roster width this run never used.
   */
  readonly wroteTheText: boolean;
};

/**
 Everything the checker round decided about one issue.

 @example
 ```ts
 const reading: IssueCheckerReading = { ballots, configuredCheckers: 3, tally, };
 ```
 */
export type IssueCheckerReading = {
  /**
   Every ballot cast on this issue, one per checker heard.
   */
  readonly ballots: readonly IssueCheckerBallot[];

  /**
   Checkers the run seated, which is not recoverable from the ballots.

   A lost voice leaves no ballot, so two ballots of three seated and two of
   six seated are indistinguishable without this and are very different
   evidence. Same reasoning as `IssueProbeReading.configuredProbers`.
   */
  readonly configuredCheckers: number;

  /**
   Weights behind each answer and what they decided, as this run tallied them.

   STORED RATHER THAN LEFT DERIVABLE so a reader can check its own arithmetic
   against the run's. The ballots and `wroteTheText` are enough to recompute
   it, and a recomputation that disagrees means the weighting changed under a
   settled artifact, which is a thing worth being able to notice.
   */
  readonly tally: IssueResolutionTally;
};

//endregion Checker reading
