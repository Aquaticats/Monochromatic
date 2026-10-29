/**
 Operation-owned attempt snapshots; progress and final accounting share this source. @module
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type {
  AdvisorAttemptRecord,
  AdvisorCollectedReview,
  AdvisorOperationEnd,
  AdvisorOperationSnapshot,
} from './operation-types.ts';
import { aggregateAdvisorUsage, } from './operation-usage.ts';

/**
 Ledger logger root.
 */
const l = tagged({ tag: 'advisor/operation-ledger', },);

//region Ledger

/**
 Own immutable published records behind a frozen operation-local capability.
 
 @param timing - original absolute operation boundaries
 
 @returns ledger accessors and locally owned update methods
 
 @example
 ```ts
 const ledger = createAdvisorOperationLedger({ startedAtMs: 0, deadlineAtMs: 1000 });
 ```
 */
export function createAdvisorOperationLedger(timing: {
  readonly startedAtMs: number;
  readonly deadlineAtMs: number;
},): AdvisorOperationLedger {
  /**
   Attempts indexed by local identity.
   */
  const attempts = new Map<number, AdvisorAttemptRecord>();
  /**
   Usable reviews indexed by their successful attempt.
   */
  const reviews = new Map<number, AdvisorCollectedReview>();
  /**
   Provider exclusions discovered by this operation only.
   */
  const blocked = new Set<string>();
  /**
   Optional boundaries owned by this operation, never shared between calls.
   */
  const state: {
    collectionEndsAtMs?: number;
    end?: AdvisorOperationEnd
  } = {};

  /**
   Insert or replace an attempt snapshot while the operation is open.
   
   @param value - detached attempt metadata
   
   @example
   ```ts
   ledger.record(attempt);
   ```
   */
  function record(value: AdvisorAttemptRecord,): void {
    if (state.end !== undefined)
      return;
    attempts.set(
      value.id,
      value,
    );
  }

  /**
   Find an existing attempt without inventing a placeholder for an inconsistent identity.
   
   @param id - operation-local attempt identity
   
   @returns current immutable record
   
   @throws when no such attempt was started
   
   @example
   ```ts
   const current = ledger.attempt(1);
   ```
   */
  function attempt(id: number,): AdvisorAttemptRecord {
    /**
     Latest attempt snapshot.
     */
    const value = attempts.get(id,);
    if (value === undefined)
      throw new Error(`advisor: unknown attempt ${String(id,)}`,);
    return value;
  }

  /**
   Retain original usable text and establish the collection boundary only once.
   
   @param review - original usable text and identity
   
   @param collectionEndsAtMs - proposed first-success cutoff
   
   @example
   ```ts
   ledger.collect({ review, collectionEndsAtMs: 1000 });
   ```
   */
  function collect({
    review,
    collectionEndsAtMs,
  }: {
    readonly review: AdvisorCollectedReview;
    readonly collectionEndsAtMs: number;
  },): void {
    if (state.end !== undefined)
      return;
    reviews.set(
      review.attemptId,
      review,
    );
    state.collectionEndsAtMs ??= Math.min(
      collectionEndsAtMs,
      timing.deadlineAtMs,
    );
  }

  /**
   Exclude a provider after observed credit exhaustion.
   
   @param provider - registered provider identity
   
   @example
   ```ts
   ledger.blockProvider('fixture-provider');
   ```
   */
  function blockProvider(provider: string,): void {
    if (state.end !== undefined)
      return;
    tagged({
      tag: blockProvider.name,
      l,
    },)
      .debug(`excluding provider for this operation: ${provider}`,);
    blocked.add(provider,);
  }

  /**
   Seal records and locally finalize cancelled pending attempts.
   
   @param end - reason scheduling and collection ended
   
   @param now - local cutoff timestamp
   
   @example
   ```ts
   ledger.finish({ end: 'collection', now: 1000 });
   ```
   */
  function finish({
    end,
    now,
  }: {
    readonly end: AdvisorOperationEnd;
    readonly now: number
  },): void {
    if (state.end !== undefined)
      return;
    for (const value of attempts.values()) {
      if ((value.state === 'running') || (value.state === 'preparing')) {
        attempts.set(
          value.id,
          {
            ...value,
            state: 'cancelled',
            endedAtMs: now,
            diagnostic: `${end}: cancellation requested; remote settlement not confirmed`,
            usageIncomplete: true,
          },
        );
      }
    }
    state.end = end;
  }

  /**
   Publish detached containers counting each attempt's available usage once.
   
   @returns snapshot safe to retain across later callbacks
   
   @example
   ```ts
   const progress = ledger.snapshot();
   ```
   */
  function snapshot(): AdvisorOperationSnapshot {
    /**
     Stable attempt order inherited from insertion.
     */
    const records = [...attempts.values(),];
    return {
      ...timing,
      ...state,
      attempts: records,
      reviews: [...reviews.values(),].toSorted(function byAttempt(
        left: AdvisorCollectedReview,
        right: AdvisorCollectedReview,
      ): number {
        return left.attemptId - right.attemptId;
      },),
      blockedProviders: [...blocked,],
      usage: aggregateAdvisorUsage(records.flatMap(function available(value: AdvisorAttemptRecord,): readonly NonNullable<AdvisorAttemptRecord['usage']>[] {
        return value.usage === undefined ? [] : [value.usage,];
      },),),
      usageIncomplete: records.some(function incomplete(value: AdvisorAttemptRecord,): boolean { return value.usageIncomplete; },),
    };
  }

  return Object.freeze({
    record,
    attempt,
    collect,
    blockProvider,
    finish,
    snapshot,
  },);
}

/**
 Operation-local ledger capability.
 */
export type AdvisorOperationLedger = {
  /**
   Replace an attempt's current snapshot.
   */
  readonly record: (value: AdvisorAttemptRecord) => void;
  /**
   Resolve an existing attempt identity.
   */
  readonly attempt: (id: number) => AdvisorAttemptRecord;
  /**
   Retain one usable review and initialize collection timing.
   */
  readonly collect: (options: {
    readonly review: AdvisorCollectedReview;
    readonly collectionEndsAtMs: number
  }) => void;
  /**
   Exclude one provider within this operation.
   */
  readonly blockProvider: (provider: string) => void;
  /**
   Seal the operation and locally cancel pending attempts.
   */
  readonly finish: (options: {
    readonly end: AdvisorOperationEnd;
    readonly now: number
  }) => void;
  /**
   Publish current detached containers.
   */
  readonly snapshot: () => AdvisorOperationSnapshot;
};

//endregion Ledger
