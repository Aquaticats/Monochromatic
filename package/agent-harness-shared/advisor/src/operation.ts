/**
 Serial failure recovery and opt-in overlap with bounded post-success collection. @module
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  nextAdvisorCandidate,
  NO_ADVISOR_CANDIDATE,
} from './operation-candidates.ts';
import {
  ADVISOR_CLOCK_BOUNDARY,
  createAdvisorCancellation,
  waitForAdvisorEvent,
} from './operation-clock.ts';
import { createAdvisorOperationLedger, } from './operation-ledger.ts';
import {
  observeAdvisorAttempt,
  type AdvisorAttemptCompletion,
  type AdvisorDispatch,
  type AdvisorOperationCandidate,
} from './operation-attempt.ts';
import type {
  AdvisorAttemptRecord,
  AdvisorOperationEnd,
  AdvisorOperationSnapshot,
} from './operation-types.ts';

/**
 Scheduler logger root.
 */
const l = tagged({ tag: 'advisor/operation', },);

/**
 Inputs kept independent of Pi host selection and rendering for deterministic verification.
 */
export type AdvisorOperationOptions = {
  /**
   Candidates in existing default ranking, or one exact explicit endpoint.
   */
  readonly candidates: readonly AdvisorOperationCandidate[];
  /**
   Original start before preparation.
   */
  readonly startedAtMs: number;
  /**
   Absolute operation limit.
   */
  readonly deadlineAtMs: number;
  /**
   Omitted disables overlap without disabling serial failure recovery.
   */
  readonly hedgeDelayMs?: number;
  /**
   Post-success collection allowance.
   */
  readonly collectionGraceMs: number;
  /**
   Caller cancellation, independent of branch failures.
   */
  readonly signal?: ForeignHostCapability<AbortSignal>;
  /**
   Actual provider dispatch with request-boundary observers.
   */
  readonly complete: AdvisorDispatch;
  /**
   Bounded state-transition updates derived from the ledger.
   */
  readonly onUpdate?: (operation: AdvisorOperationSnapshot) => void;
  /**
   Injected clock for deterministic scheduling tests.
   */
  readonly now?: () => number;
  /**
   Injected wait boundary for deterministic scheduling tests.
   */
  readonly wait?: typeof waitForAdvisorEvent<AdvisorAttemptCompletion | typeof ADVISOR_CLOCK_BOUNDARY>;
};

/**
 Active attempt retained until its ledger state settles.
 */
type ActiveAdvisorAttempt = {
  /**
   Endpoint and evidence metrics.
   */
  readonly candidate: AdvisorOperationCandidate;
  /**
   Observed provider settlement.
   */
  readonly promise: Promise<AdvisorAttemptCompletion>;
};

/**
 Run default recovery or an exact candidate under one operation-owned cancellation lifetime.
 
 @param options - ranked candidates, provider boundary, and timing policy
 
 @returns sealed operation including usable reviews and every attempt
 
 @mutates options - provider, clock, progress, and cancellation capabilities are invoked
 
 @example
 ```ts
 const operation = await runAdvisorOperation({ candidates, startedAtMs, deadlineAtMs, collectionGraceMs, complete });
 ```
 */
export async function runAdvisorOperation(options: ForeignHostCapability<AdvisorOperationOptions>,): Promise<AdvisorOperationSnapshot> {
  /**
   Clock and wait seams share the same time domain.
   */
  const {
    now = Date.now,
    wait = waitForAdvisorEvent,
  } = options;
  /**
   Single source of truth for attempts, credit exclusions, and collection.
   */
  const ledger = createAdvisorOperationLedger({
    startedAtMs: options.startedAtMs,
    deadlineAtMs: options.deadlineAtMs,
  },);
  /**
   At most two active reviewer attempts are retained.
   */
  const active = new Map<number, ActiveAdvisorAttempt>();
  /**
   Transition wakeups keep scheduling responsive to asynchronous preparation and provider outcomes.
   */
  const wake = { current: Promise.withResolvers<typeof ADVISOR_CLOCK_BOUNDARY>(), };
  /**
   Scope exit requests cancellation even if local orchestration throws.
   */
  using cancellation = createAdvisorCancellation();
  /**
   Caller abort also reaches providers immediately, not only on the next scheduler turn.
   */
  const signal = options.signal === undefined ? cancellation.controller
    .signal : AbortSignal.any([
      options.signal,
      cancellation.controller
        .signal,
    ],);
  /**
   Operation-tagged logger.
   */
  const innerL = tagged({
    tag: runAdvisorOperation.name,
    l,
  },);

  /**
   Notify from ledger transitions, never from individual streamed usage deltas.
   */
  function notify(): void {
    wake.current
      .resolve(ADVISOR_CLOCK_BOUNDARY,);
    try {
      options.onUpdate?.(ledger.snapshot(),);
    }
    catch (error) {
      innerL.warn(`Advisor progress callback failed: ${caughtValueText(error,)}`,);
    }
  }

  /**
   Start one owned attempt; preparation is visible without claiming a provider request already occurred.
   
   @param candidate - selected endpoint and evidence metrics
   */
  function start(candidate: AdvisorOperationCandidate,): void {
    /**
     Existing records establish bounded recovery count and unique identity.
     */
    const records = ledger.snapshot()
      .attempts;
    /**
     New operation-local attempt identity.
     */
    const id = records.length + 1;
    ledger.record({
      ...candidate,
      id,
      attempt: records.filter(function sameModel(record: AdvisorAttemptRecord,): boolean { return record.model === candidate.model; },)
        .length
        + 1,
      state: 'preparing',
      startedAtMs: now(),
      usageIncomplete: true,
    },);
    innerL.debug(`preparing ${candidate.model}; attemptId=${String(id,)}`,);
    active.set(
      id,
      {
        candidate,
        promise: observeAdvisorAttempt({
          id,
          candidate,
          ledger,
          complete: options.complete,
          signal,
          now,
          graceMs: options.collectionGraceMs,
          notify,
        },),
      },
    );
    notify();
  }

  /**
   Seal first, request cancellation, then publish the terminal snapshot.
   
   @param end - local termination reason
   
   @returns finalized operation
   */
  function finish(end: AdvisorOperationEnd,): AdvisorOperationSnapshot {
    ledger.finish({
      end,
      now: now(),
    },);
    cancellation.controller
      .abort();
    innerL.debug(`operation ended: ${end}`,);
    notify();
    return ledger.snapshot();
  }

  /**
   Select an unused eligible model using current provider exclusions.
   
   @returns next endpoint or exhausted-candidate sentinel
   */
  function selectNextCandidate(): AdvisorOperationCandidate | typeof NO_ADVISOR_CANDIDATE {
    /**
     Exclusions and dispatched identities are read just in time.
     */
    const current = ledger.snapshot();
    return nextAdvisorCandidate({
      candidates: options.candidates,
      attemptedModels: current.attempts
        .map(function modelIdentity(record: AdvisorAttemptRecord,): string { return record.model; },),
      blockedProviders: current.blockedProviders,
      runningProviders: [...active.values(),].map(function providerIdentity(value: ActiveAdvisorAttempt,): string { return value.candidate
        .provider; },),
    },);
  }

  while ((!signal.aborted) && (now() < options.deadlineAtMs)) {
    wake.current = Promise.withResolvers<typeof ADVISOR_CLOCK_BOUNDARY>();
    /**
     Snapshot used to decide cancellation before scheduling new work.
     */
    const current = ledger.snapshot();
    if ((current.collectionEndsAtMs !== undefined) && (now() >= current.collectionEndsAtMs))
      return finish('collection',);

    // Reconcile all already-settled records before deciding whether another request is needed.
    for (const [id, entry,] of active) {
      /**
       Latest record for this active promise.
       */
      const record = ledger.attempt(id,);
      if ((record.state === 'running') || (record.state === 'preparing'))
        continue;
      active.delete(id,);
      if ((record.state === 'empty') && (record.attempt < 2)
        && (ledger.snapshot()
          .reviews
          .length
          === 0)
        && (!ledger.snapshot()
          .blockedProviders
          .includes(entry.candidate
            .provider,)))
        start(entry.candidate,);
    }
    if (current.reviews
      .length
      > 0) {
      if (active.size === 0)
        return finish('complete',);
    }
    else {
      /**
       At least one viable reviewer is needed even with overlap disabled.
       */
      const candidate = selectNextCandidate();
      if (active.size === 0) {
        if (candidate === NO_ADVISOR_CANDIDATE)
          return finish('exhausted',);
        start(candidate,);
      }
      /**
       The first logical reviewer call anchors the delay, including authentication.
       */
      const firstStartedAt = ledger.snapshot()
        .attempts
        .at(0,)
        ?.startedAtMs;
      if ((options.hedgeDelayMs !== undefined) && (firstStartedAt !== undefined)
        && (now() >= (firstStartedAt
          + options.hedgeDelayMs))
        && (active.size < 2)) {
        /**
         Recompute after a possible serial replacement was started.
         */
        const alternate = selectNextCandidate();
        if (alternate !== NO_ADVISOR_CANDIDATE)
          start(alternate,);
      }
    }
    /**
     Current collection or deadline cutoff.
     */
    const boundary = ledger.snapshot()
      .collectionEndsAtMs
      ?? options.deadlineAtMs;
    /**
     Logical call start is available even when authentication is stalled.
     */
    const firstStartedAt = ledger.snapshot()
      .attempts
      .at(0,)
      ?.startedAtMs;
    /**
     Hedge wakeup exists only while unused candidates and a concurrency slot remain.
     */
    const hedgeAt = (current.reviews
      .length
      === 0) && (options.hedgeDelayMs !== undefined)
      && (firstStartedAt !== undefined)
      && (active.size < 2)
      && (selectNextCandidate() !== NO_ADVISOR_CANDIDATE)
      ? firstStartedAt + options.hedgeDelayMs : boundary;
    // oxlint-disable-next-line no-await-in-loop -- Each provider outcome determines the next dispatch and cutoff; parallel waits would use stale operation state.
    await wait({
      pending: [
        ...[...active.values(),].map(function pending(entry: ActiveAdvisorAttempt,): Promise<AdvisorAttemptCompletion> { return entry.promise; },),
        wake.current
          .promise,
      ],
      untilMs: Math.min(
        boundary,
        hedgeAt,
      ),
      signal,
      now,
    },);
  }
  return finish(signal.aborted ? 'caller' : 'deadline',);
}
