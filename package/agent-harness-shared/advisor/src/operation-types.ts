/**
 Advisor operation records shared by scheduling, accounting, and rendering. @module
 */
import type { Usage, } from '@earendil-works/pi-ai';
import type { ReadonlyDeep, } from 'type-fest';

//region Records

/**
 Local attempt lifecycle, independent of remote billing settlement.
 */
export type AdvisorAttemptState = 'preparing' | 'running' | 'succeeded' | 'empty' | 'failed' | 'cancelled';

/**
 Why an operation stopped scheduling or collecting provider responses.
 */
export type AdvisorOperationEnd = 'complete' | 'exhausted' | 'deadline' | 'collection' | 'caller';

/**
 Snapshot of one attempt; later updates replace records rather than mutate published snapshots.
 */
export type AdvisorAttemptRecord = {
  /**
   Operation-local identity, also determining stable display order.
   */
  readonly id: number;
  /**
   Exact endpoint dispatched or being prepared.
   */
  readonly model: string;
  /**
   Registered provider identity used for call-local exclusion.
   */
  readonly provider: string;
  /**
   Attempt number on this endpoint.
   */
  readonly attempt: number;
  /**
   Current locally observed state.
   */
  readonly state: AdvisorAttemptState;
  /**
   Preparation start, included in operation deadline.
   */
  readonly startedAtMs: number;
  /**
   Actual provider invocation time, absent when preparation failed.
   */
  readonly dispatchedAtMs?: number;
  /**
   Local finalization time, not proof of remote cancellation.
   */
  readonly endedAtMs?: number;
  /**
   Requested reasoning level, absent for non-reasoning models.
   */
  readonly reasoning?: string;
  /**
   Serialized evidence size for this endpoint.
   */
  readonly contextChars: number;
  /**
   Input estimate for this endpoint's request.
   */
  readonly estimatedInputTokens: number;
  /**
   Whether this endpoint received truncated evidence.
   */
  readonly truncated: boolean;
  /**
   Provider terminal state, when actually received.
   */
  readonly stopReason?: string;
  /**
   Failure or local cancellation explanation.
   */
  readonly diagnostic?: string;
  /**
   Latest available usage snapshot; never sum streaming snapshots.
   */
  readonly usage?: ReadonlyDeep<Usage>;
  /**
   Whether usage may omit work after the last observed snapshot.
   */
  readonly usageIncomplete: boolean;
};

/**
 Successful review retained without synthesizing or selecting a winner.
 */
export type AdvisorCollectedReview = {
  /**
   Ledger identity for the successful attempt.
   */
  readonly attemptId: number;
  /**
   Canonical model label included with review text.
   */
  readonly model: string;
  /**
   Original visible provider text.
   */
  readonly text: string;
  /**
   Whether output exhausted its response budget.
   */
  readonly lengthLimited: boolean;
};

/**
 Operation snapshot published to progress, final output, and durable diagnostics.
 */
export type AdvisorOperationSnapshot = {
  /**
   Local start before scope and context preparation.
   */
  readonly startedAtMs: number;
  /**
   Original absolute operation limit.
   */
  readonly deadlineAtMs: number;
  /**
   Post-success collection limit, when a usable review exists.
   */
  readonly collectionEndsAtMs?: number;
  /**
   Immutable attempt snapshots in dispatch order.
   */
  readonly attempts: readonly AdvisorAttemptRecord[];
  /**
   Completed usable reviews in dispatch order.
   */
  readonly reviews: readonly AdvisorCollectedReview[];
  /**
   Providers excluded after observed exhausted-credit responses.
   */
  readonly blockedProviders: readonly string[];
  /**
   Aggregate of latest available usage per attempt.
   */
  readonly usage: ReadonlyDeep<Usage>;
  /**
   Whether the aggregate is potentially incomplete.
   */
  readonly usageIncomplete: boolean;
  /**
   Absent while running; locally final once present.
   */
  readonly end?: AdvisorOperationEnd;
};

//endregion Records
