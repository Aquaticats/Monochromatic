/**
 Outcomes of a commit attempt that the accepted design allows besides landing, told apart by their exit code and message.

 @module
 */

import type { AttemptRecord, } from './ledger-fixture.ts';

/**
 Whether an attempt ended because the branch kept moving while it was prepared:
 exit 1 and the plain message of the bounded retry (`SPEC.md` "Retry after a moved branch"), with nothing landed.

 @param attempt - finished attempt

 @returns whether the attempt ended after the service's bounded retries

 @example
 ```ts
 isBranchKeptChanging(attempt);
 ```
 */
export function isBranchKeptChanging(attempt: Pick<AttemptRecord, 'outcome'>,): boolean {
  return (attempt.outcome
    .exitCode
    === 1)
    && attempt.outcome
      .stderr
      .includes(' times while this commit was being prepared, so it was not made.',);
}

/**
 Whether an attempt ended because `git gc --prune=now` removed the content it had prepared before the service landed it:
 exit 1 and the plain message that names the prune, which Git's documentation warns about for a repository other
 processes write to.

 @param attempt - finished attempt

 @returns whether the attempt ended because its content was pruned

 @example
 ```ts
 isPrunedBeforeLanding(attempt);
 ```
 */
export function isPrunedBeforeLanding(attempt: Pick<AttemptRecord, 'outcome'>,): boolean {
  return (attempt.outcome
    .exitCode
    === 1)
    && attempt.outcome
      .stderr
      .includes('were removed from the repository while the commit was being made, so it was not made.',);
}
