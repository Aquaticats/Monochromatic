/**
 Expectations a scenario attaches to its run:
 every attempt exited 0,
 or ended as one of the accepted design's outcomes.

 @module
 */

import type { RunObservation, } from './invariant-model-fixture.ts';
import type { AttemptRecord, } from './ledger-fixture.ts';
import { isBranchKeptChanging, } from './scenario-outcome-fixture.ts';

/**
 One expectation of a run.
 */
type Expectations = RunObservation['expectations'];

//region Expectation helpers

/**
 Expectation that every listed attempt exited 0.

 @param name - expectation name

 @param attempts - attempts the accepted design lets land

 @returns expectation record

 @example
 ```ts
 allSucceeded({ name: 'all-commits-succeed', attempts });
 ```
 */
export function allSucceeded({
  name,
  attempts,
}: Readonly<{
  name: string;
  attempts: readonly AttemptRecord[];
}>,): Expectations[number] {
  /**
   Attempts that did not exit 0.
   */
  const failed = attempts.filter(function nonZero(attempt,) {
    return attempt.outcome
      .exitCode
      !== 0;
  },);
  return {
    name,
    holds: failed.length === 0,
    detail: failed.map(function describe(attempt,) {
      /**
       Git explains "nothing to commit" on its standard output, while stderr then holds only the queue notice.
       */
      const standardOutput = attempt.outcome
        .stdout
        .trim()
        .split('\n',)
        .slice(-2,)
        .join(' | ',);
      return `${attempt.label} exited ${String(attempt.outcome
        .exitCode,)}: ${attempt.outcome
          .stderr
          .trim()
          .split('\n',)
          .slice(-2,)
          .join(' | ',)}; stdout: ${standardOutput}`;
    },)
      .join('; ',),
  };
}

/**
 Whether an attempt failed with the finding of a commit whose captured change overlaps what landed before it:
 exit 1, the `concurrent-commit/replay-conflict` core finding, and one of the attempt's own paths named in it.

 @param attempt - finished attempt

 @returns whether the attempt ended as a reported replay conflict

 @example
 ```ts
 isReportedConflict(attempt);
 ```
 */
export function isReportedConflict(attempt: Pick<AttemptRecord, 'outcome' | 'selectedPaths'>,): boolean {
  return (attempt.outcome
    .exitCode
    === 1)
    && attempt.outcome
      .stderr
      .includes('"code":"concurrent-commit/replay-conflict"',)
    && attempt.selectedPaths
      .some(function named(path,) {
        return attempt.outcome
          .stderr
          .includes(path,);
      },);
}

/**
 Whether an attempt ended because commits that landed while it was prepared already hold everything it changes:
 exit 1 and the `concurrent-commit/already-committed` core finding, as native `git commit` says "nothing to commit"
 for a path whose bytes an earlier commit took.

 @param attempt - finished attempt

 @returns whether the attempt ended with nothing left to commit

 @example
 ```ts
 isAlreadyCommitted(attempt);
 ```
 */
export function isAlreadyCommitted(attempt: Pick<AttemptRecord, 'outcome'>,): boolean {
  return (attempt.outcome
    .exitCode
    === 1)
    && attempt.outcome
      .stderr
      .includes('"code":"concurrent-commit/already-committed"',);
}

/**
 Expectation that every listed attempt landed its captured bytes merged onto the branch (exit 0),
 failed as a reported replay conflict,
 found its captured bytes already on the branch,
 or ended after the service's bounded retries, which are the accepted design's outcomes.

 @param name - expectation name

 @param attempts - attempts the accepted design lets land or conflict

 @returns expectation record

 @example
 ```ts
 landedOrConflicted({ name: 'all-commits-succeed', attempts });
 ```
 */
export function landedOrConflicted({
  name,
  attempts,
}: Readonly<{
  name: string;
  attempts: readonly AttemptRecord[];
}>,): Expectations[number] {
  return allSucceeded({
    name,
    attempts: attempts.filter(function neither(attempt,) {
      return ![
        isReportedConflict,
        isAlreadyCommitted,
        isBranchKeptChanging,
      ].some(function matches(outcome,) {
        return outcome(attempt,);
      },);
    },),
  },);
}

//endregion Expectation helpers
