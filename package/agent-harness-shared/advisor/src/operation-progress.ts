/**
 Bounded progress text derived only from operation metadata, never review or reasoning content. @module
 */
import type {
  AdvisorAttemptRecord,
  AdvisorOperationSnapshot,
} from './operation-types.ts';

/**
 Describe current attempt states without exposing serialized evidence or provider payloads.
 
 @param operation - ledger snapshot
 
 @param now - current time for elapsed and remaining durations
 
 @returns compact transition summary
 
 @example
 ```ts
 const text = formatAdvisorProgress({ operation, now: Date.now() });
 ```
 */
export function formatAdvisorProgress({
  operation,
  now,
}: {
  readonly operation: AdvisorOperationSnapshot;
  readonly now: number;
},): string {
  /**
   Each attempt retains its own context and reasoning metadata.
   */
  const attempts = operation.attempts
    .map(function formatAttempt(attempt: AdvisorAttemptRecord,): string {
      return `${attempt.model} attempt ${String(attempt.attempt,)}: ${attempt.state}; ${String(attempt.contextChars,)} chars; reasoning=${attempt.reasoning ?? 'off'}`;
    },)
    .join('\n',);
  /**
   Local collection timing never implies remote provider termination.
   */
  const collection = operation.collectionEndsAtMs === undefined ? '' : `; collection remaining ${String(Math.max(
    0,
    operation.collectionEndsAtMs - now,
  ),)}ms`;
  return `Advisor ${operation.end ?? (operation.reviews
    .length
    > 0 ? 'collecting' : 'running')}: ${String(operation.reviews
      .length,)} usable reviews; elapsed ${String(Math.max(
    0,
    now - operation.startedAtMs,
  ),)}ms; deadline remaining ${String(Math.max(
    0,
    operation.deadlineAtMs - now,
  ),)}ms${collection}\n${attempts}${operation.blockedProviders
    .length
    === 0 ? '' : `\nCredit-exhausted providers excluded for this call: ${operation.blockedProviders
      .join(', ',)}`}`;
}
