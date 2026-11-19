/**
 Session-local admission for provider-published limits.
 @module
 */

import { once, } from 'node:events';
import type {
  Api,
  Model,
  SimpleStreamOptions,
} from '@earendil-works/pi-ai';
import {
  probeQuota,
  QUOTA_UNAVAILABLE,
  type QuotaEvidence,
  type RoutingRegistry,
} from './quota-endpoints.ts';
import { digitsEnd, } from './provider-wire-text.ts';

//region One live view per exact member

/**
 Quota and active attempts associated with one provider and model in a Pi session.
 */
export type MemberCapacity = {
  /**
   Whether the next admission needs a fresh provider reading.
   */
  needsProbe: boolean;
  /**
   Shared read so overlapping requests do not all poll the quota endpoint.
   */
  pending?: Promise<void>;
  /**
   In-flight streams started by this plugin, not provider-wide load.
   */
  active: number;
  /**
   Retry/reset timestamp supplied by the provider or its documented guidance.
   */
  retryAt: number;
  /**
   Most recent published limits and subscription facts.
   */
  evidence: QuotaEvidence;
  /**
   Rejection's observed lower cap and the renewal that ends the exception.
   */
  overage?: {
    readonly concurrency: number;
    readonly periodEnd: number
  };
};

/**
 Get or initialize one member's state.

 @param members - session-owned capacity map

 @param label - exact provider and model

 @returns mutable member state used only within the session

 @example
 ```ts
 const capacity = capacityOf({ members: new Map(), label: 'neuralwatt/model' });
 ```
 */
export function capacityOf({
  members,
  label,
}: {
  members: Map<string, MemberCapacity>;
  label: string;
},): MemberCapacity {
  /**
   Member capacity already shared by other requests in this session.
   */
  const existing = members.get(label,);
  if (existing !== undefined)
    return existing;
  /**
   Member-specific state for rate evidence and local admission.
   */
  const capacity: MemberCapacity = {
    needsProbe: true,
    active: 0,
    retryAt: 0,
    evidence: {},
  };
  members.set(
    label,
    capacity,
  );
  return capacity;
}

//endregion One live view per exact member

//region Live quota reads and local ceiling

/**
 Provisional pre-install bound for optional quota reads on the inference path. Set it from the live endpoint
 check before install; there is no endpoint latency measurement from this offline work.
 */
const QUOTA_READ_TIMEOUT_MS = 2_000;

/**
 Bound the optional quota read independently of inference and below any valid model timeout.

 @param options - caller cancellation and transport timing policy

 @returns quota request cancellation
 */
function quotaSignal(options?: SimpleStreamOptions,): AbortSignal {
  /**
   Caller cancellation and timeout settings used by the quota lookup.
   */
  const {
    signal,
    timeoutMs,
  } = options ?? {};
  /**
   Provisional allocation below the model timeout so an optional read cannot spend a full model timeout.
   */
  const budget = ((timeoutMs === undefined) || (!Number.isFinite(timeoutMs,))
    || (timeoutMs <= 0))
    ? QUOTA_READ_TIMEOUT_MS
    : Math.min(
      QUOTA_READ_TIMEOUT_MS,
      Math.floor(timeoutMs / 2,),
    );
  /**
   Independent, abortable quota deadline.
   */
  const timeout = AbortSignal.timeout(budget,);
  return signal === undefined ? timeout : AbortSignal.any([
    signal,
    timeout,
  ],);
}

/**
 Stop waiting even if credential resolution or an injected transport does not observe the quota signal.
 The signal still reaches the quota operations so cooperative transports can cancel their work.

 @param request - quota lookup and its independent deadline

 @returns evidence only when lookup completes before cancellation
 */
async function readQuotaWithinDeadline(request: Parameters<typeof probeQuota>[0] & { readonly signal: AbortSignal; },): Promise<QuotaEvidence | typeof QUOTA_UNAVAILABLE> {
  /**
   Deadline combined with the owning request's cancellation.
   */
  const { signal, } = request;
  if (signal.aborted)
    return QUOTA_UNAVAILABLE;
  /**
   Listener cleanup for a read that completes before its deadline.
   */
  const stopWaiting = new AbortController();
  /**
   Release the temporary abort listener when either side of the race settles.
   */
  using watch = {
    [Symbol.dispose](): void {
      stopWaiting.abort();
    },
  };
  /**
   Stop the wait once the deadline or the owning request aborts.

   @returns unavailable quota reading
   */
  async function stopOnAbort(): Promise<typeof QUOTA_UNAVAILABLE> {
    await once(
      signal,
      'abort',
      { signal: stopWaiting.signal, },
    );
    return QUOTA_UNAVAILABLE;
  }
  return await Promise.race([
    stopOnAbort(),
    probeQuota(request,),
  ],);
}

/**
 Wait for a lookup without propagating this waiter's deadline to the shared read.

 @param pending - shared member quota read

 @param signal - cancellation or wait deadline for this caller alone
 */
async function waitForCapacity({
  pending,
  signal,
}: {
  readonly pending: Promise<void>;
  readonly signal?: AbortSignal;
},): Promise<void> {
  if (signal === undefined) {
    await pending;
    return;
  }
  if (signal.aborted)
    return;
  /**
   Cancellation for the temporary waiter, not for the shared lookup.
   */
  const stopWaiting = new AbortController();
  /**
   Releases the abort listener when the waiter completes for any reason.
   */
  using watch = {
    [Symbol.dispose](): void {
      stopWaiting.abort();
    },
  };
  await Promise.race([
    pending,
    once(
      signal,
      'abort',
      { signal: stopWaiting.signal, },
    ),
  ],);
}

/**
 Refresh quota at first use and on reentry after a refusal or exhausted window.

 @param capacity - state shared by the member's requests

 @param registry - registry used for the quota credential

 @param model - exact physical member

 @param options - probe transport and cancellation

 @example
 ```ts
 await refreshCapacity({ capacity, registry, model });
 ```
 */
export async function refreshCapacity({
  capacity,
  registry,
  model,
  options,
}: {
  capacity: MemberCapacity;
  registry: RoutingRegistry;
  model: Model<Api>;
  options?: SimpleStreamOptions;
},): Promise<void> {
  /**
   Read and publish fresh evidence, releasing the shared lookup when settled.
   */
  async function refresh(): Promise<void> {
    /**
     Clears the pending lookup only when its own read has settled.
     */
    using owner = {
      [Symbol.dispose](): void {
        delete capacity.pending;
      },
    };
    capacity.needsProbe = false;
    if (capacity.retryAt <= Date.now())
      capacity.retryAt = 0;
    /**
     Fresh quota facts that may supersede previous capacity or retry hints.
     */
    const reading = await readQuotaWithinDeadline({
      registry,
      model,
      fetchQuota: options?.fetch ?? globalThis.fetch,
      signal: quotaSignal(options,),
    },);
    if (options?.signal
      ?.aborted
      === true) {
      capacity.needsProbe = true;
      return;
    }
    if (reading === QUOTA_UNAVAILABLE)
      capacity.needsProbe = true;
    /**
     Leave existing evidence in place after an unavailable read, but still expire past deadlines.
     */
    const evidence = reading === QUOTA_UNAVAILABLE ? {} : reading;
    /**
     Observation time used to compare provider deadlines.
     */
    const now = Date.now();
    capacity.evidence = {
      ...capacity.evidence,
      ...(evidence.concurrency === undefined ? {} : { concurrency: evidence.concurrency, }),
      ...(evidence.overage === undefined ? {} : { overage: evidence.overage, }),
      ...((evidence.periodEnd === undefined) || (evidence.periodEnd === 0) ? {} : { periodEnd: evidence.periodEnd, }),
    };
    capacity.retryAt = Math.max(
      capacity.retryAt,
      evidence.retryAt ?? 0,
    );
    if (capacity.overage !== undefined) {
      if ((evidence.overage === false)
        || ((capacity.overage
          .periodEnd
          > 0) && (capacity.overage
            .periodEnd
            <= now)))
        delete capacity.overage;
      else if ((capacity.overage
        .periodEnd
        === 0) && ((evidence.periodEnd ?? 0) > now))
        capacity.overage = {
          ...capacity.overage,
          periodEnd: evidence.periodEnd ?? 0,
        };
    }
  }

  /**
   This caller's cancellation, independent of the shared lookup.
   */
  const { signal, } = options ?? {};
  /**
   Pass the caller's abort signal only when supplied.
   */
  const cancellation = signal === undefined ? {} : { signal, };
  if (capacity.pending !== undefined) {
    await waitForCapacity({
      pending: capacity.pending,
      signal: quotaSignal(options,),
    },);
    return;
  }
  if ((!capacity.needsProbe) && (capacity.retryAt !== 0)
    && (capacity.retryAt <= Date.now()))
    capacity.needsProbe = true;
  if ((capacity.overage !== undefined) && (capacity.overage
    .periodEnd
    <= Date.now()))
    capacity.needsProbe = true;
  if (!capacity.needsProbe)
    return;
  capacity.pending = refresh();
  await waitForCapacity({
    pending: capacity.pending,
    ...cancellation,
  },);
}

/**
 NeuralWatt's exact pay-as-you-go exception, as recorded in the 2026-11-11 refusal.
 */
const OVERAGE_NOTICE = 'Your subscription is in overage with auto-renew off';

/**
 Introduction to the reported concurrency counts.
 */
const OVERAGE_HEAD = 'Concurrent limit reached for ';

/**
 Scan an admission error for the first well-formed concurrent-slots report.

 @param value - terminal error without changes to spacing

 @returns reported ceiling, or NaN when the counts are absent
 */
function describedConcurrency(value: string,): number {
  for (
    let head = value.indexOf(OVERAGE_HEAD,);
    head !== (-1);
  ) {
    /**
     Beginning of the model field following the introduction.
     */
    const modelStart = head + OVERAGE_HEAD.length;
    /**
     Exclusive end of the model field before a colon or line ending.
     */
    const next = { offset: modelStart, };
    while ((next.offset < value.length) && (value[next.offset] !== ':')
      && (value[next.offset] !== '\r')
      && (value[next.offset] !== '\n'))
      next.offset += 1;
    if ((next.offset > modelStart) && value.startsWith(
      ': ',
      next.offset,
    )) {
      /**
       End of the current-slot count, before the slash.
       */
      const inUseStart = next.offset + ': '.length;
      /**
       Slash position when the in-use count is valid.
       */
      const inUseEnd = digitsEnd({
        value,
        offset: inUseStart,
      },);
      if ((inUseEnd !== inUseStart) && (value[inUseEnd] === '/')) {
        /**
         Span of the ceiling count, followed by the known units.
         */
        const ceilingStart = inUseEnd + 1;
        /**
         End of the ceiling count before the required units.
         */
        const ceilingEnd = digitsEnd({
          value,
          offset: ceilingStart,
        },);
        if ((ceilingEnd !== ceilingStart) && value.startsWith(
          ' slots in use',
          ceilingEnd,
        ))
          return Number(value.slice(
            ceilingStart,
            ceilingEnd,
          ),);
      }
    }
    // Another introduction in this model field would have the same terminator and counts.
    head = value.indexOf(
      OVERAGE_HEAD,
      next.offset + 1,
    );
  }
  return Number.NaN;
}

/**
 Mark a genuine pre-output HTTP 429 and, only for NeuralWatt's named exception, retain its cap.

 @param capacity - admitted member's per-session state

 @param provider - member whose admission service produced the response

 @param errorMessage - terminal refusal text; never logged

 @param retryAt - known retry window, or zero if no timing evidence

 @example
 ```ts
 noteRateLimit({ capacity, provider: 'synthetic', errorMessage: '429 limited', retryAt: 0 });
 ```
 */
export function noteRateLimit({
  capacity,
  provider,
  errorMessage,
  retryAt,
}: {
  capacity: MemberCapacity;
  provider: string;
  errorMessage: string;
  retryAt: number;
},): void {
  capacity.needsProbe = true;
  capacity.retryAt = Math.max(
    capacity.retryAt,
    retryAt,
  );
  if ((provider !== 'neuralwatt') || (!errorMessage.includes(OVERAGE_NOTICE,)))
    return;
  /**
   Provider-dependent simultaneous request ceiling, not available slots.
   */
  const concurrency = describedConcurrency(errorMessage,);
  if (Number.isSafeInteger(concurrency,) && (concurrency > 0)) {
    /**
     Last known subscription boundary for an observed overage cap.
     */
    const periodEnd = capacity.evidence
      .periodEnd
      ?? 0;
    capacity.overage = {
      concurrency,
      periodEnd: periodEnd > Date.now() ? periodEnd : 0,
    };
  }
}

//endregion Live quota reads and local ceiling
