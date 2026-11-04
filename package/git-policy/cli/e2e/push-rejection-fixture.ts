/**
 Whether the refusal of an attempt's auto-push reached the person:
 on the attempt's own stderr,
 or on the stderr of a later wrapped command that shows the service's failure notice.

 @module
 */

import type {
  AttemptRecord,
  AuxiliaryRecord,
} from './ledger-fixture.ts';

//region Constants

/**
 Git's reason for refusing a push that does not fast-forward the remote branch.
 */
const NON_FAST_FORWARD = 'non-fast-forward';

/**
 Note the wrapper prints after every failed auto-push (`PUSH_FAILED_NOTE` in `src/native/push/auto_push.rs`).
 */
const PUSH_FAILED_NOTE = 'auto-push to origin failed';

/**
 Line the wrapper prints instead of that note when its reconcile of a stale push cannot publish, for example after a
 conflicting merge (`RECONCILE_FAILED_PREFIX` in `src/native/push/auto_push_reconcile.rs`).
 */
const RECONCILE_FAILED_LINE = 'auto-push: not published:';

//endregion Constants

//region Surfacing

/**
 Whether an attempt's stderr shows Git's non-fast-forward refusal together with one of the two ways the wrapper says
 the commit was not published.

 @param stderr - complete stderr of the attempt

 @returns whether the rejection was surfaced
 */
function surfacesPushRejection(stderr: string,): boolean {
  return stderr.includes(NON_FAST_FORWARD,)
    && [
      PUSH_FAILED_NOTE,
      RECONCILE_FAILED_LINE,
    ].some(function mentioned(note,) {
      return stderr.includes(note,);
    },);
}

/**
 Whether the rejection of an attempt's push was surfaced to the person:
 on the attempt's own stderr,
 or, because the service pushes after the commit returned,
 on the stderr of a later wrapped command in the same repository.
 A notice is not matched to the attempt's own commit:
 it names the commit of the refused push, and the same refusal leaves every commit beneath that one unpublished.

 @param attempt - attempt whose commit landed

 @param attempts - every recorded attempt

 @param auxiliaries - every recorded non-commit command

 @returns whether the rejection was surfaced

 @example
 ```ts
 rejectionReachedPerson({ attempt, attempts, auxiliaries });
 ```
 */
export function rejectionReachedPerson({
  attempt,
  attempts,
  auxiliaries,
}: Readonly<{
  attempt: AttemptRecord;
  attempts: readonly AttemptRecord[];
  auxiliaries: readonly AuxiliaryRecord[];
}>,): boolean {
  if (surfacesPushRejection(attempt.outcome
    .stderr,))
    return true;
  /**
   Stderr of every wrapped command that started after the attempt.
   */
  const later: readonly string[] = [
    ...attempts
      .filter(function startedLater(other,) {
        return (other.startedAt > attempt.startedAt) && (other.mode !== 'foreign');
      },)
      .map(function stderrOf(other,) {
        return other.outcome
          .stderr;
      },),
    ...auxiliaries
      .filter(function wrapped(auxiliary,) {
        return auxiliary.wrapper;
      },)
      .map(function stderrOf(auxiliary,) {
        return auxiliary.outcome
          .stderr;
      },),
  ];
  return later.some(surfacesPushRejection,);
}

//endregion Surfacing
