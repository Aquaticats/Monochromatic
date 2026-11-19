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
   Shared read so overlapping refusals do not all poll the quota endpoint.
   */
  pending?: Promise<void>;
  /**
   In-flight streams started by this plugin, not provider-wide load.
   */
  active: number;
  /**
   When this member's quota endpoint was last asked, or zero before the first read.
   */
  lastReadAt: number;
  /**
   When the concurrency ceiling on record was read, or zero before a reading published one.
   */
  concurrencyReadAt: number;
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
    active: 0,
    lastReadAt: 0,
    concurrencyReadAt: 0,
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
 Minimum wait between two reads of one member, from the endpoint's own published rate.

 NeuralWatt's guide says `/v1/rate_limits` is rate-limited to 2 requests per second per customer and
 asks clients to poll at most once per second
 (https://docs.neuralwatt.com/guides/rate-limits.md, "Checking your current limit").
 */
export const QUOTA_READ_INTERVAL_MS = 1_000;

/**
 Whether this member was read too recently to read again.

 @param capacity - member state holding the last read time

 @returns whether the documented minimum interval has not passed
 */
function isReadTooSoon(capacity: Readonly<{ lastReadAt: number }>,): boolean {
  return (capacity.lastReadAt !== 0)
    && ((Date.now() - capacity.lastReadAt) < QUOTA_READ_INTERVAL_MS);
}

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
 Read one member's quota endpoint at the moment it refuses a request.

 The owner's decision of 2026-11-19 reads the endpoint at refusal, not on the first eligible attempt
 or before returning to the member: quota endpoints carry their own, far higher rate limits than
 inference, so a read costs a refused request nothing it was not already paying, and the reading
 describes the plan as it stands when the member said no. No read happens on admission.

 A read that fails leaves the evidence already held in place, and the next refusal asks again; the
 docs/README state the deadline.

 @param capacity - state shared by the member's requests

 @param registry - registry used for the quota credential

 @param model - exact physical member that refused

 @param options - probe transport and cancellation

 @example
 ```ts
 await readAtRefusal({ capacity, registry, model, options });
 ```
 */
export async function readAtRefusal({
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
  async function read(): Promise<void> {
    /**
     Clears the pending lookup only when its own read has settled.
     */
    using owner = {
      [Symbol.dispose](): void {
        delete capacity.pending;
      },
    };
    capacity.lastReadAt = Date.now();
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
      === true)
      return;
    /**
     Leave existing evidence in place after an unavailable read, but still expire past deadlines.
     */
    const evidence = reading === QUOTA_UNAVAILABLE ? {} : reading;
    /**
     Observation time used to compare provider deadlines.
     */
    const now = Date.now();
    // A reading's own time names when it published the ceiling now on record, which a later failed
    // read leaves in place while moving `lastReadAt` alone.
    if (evidence.concurrency !== undefined)
      capacity.concurrencyReadAt = now;
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
   This refusal's own cancellation, independent of the shared lookup.
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
  if (isReadTooSoon(capacity,)) {
    // The endpoint's own rate limit bounds how often this member may be asked; the next refusal asks
    // again instead of polling it now.
    return;
  }
  capacity.pending = read();
  await waitForCapacity({
    pending: capacity.pending,
    ...cancellation,
  },);
}

/**
 The ceiling a refusal's own number sets, while its period lasts.

 The owner's decision of 2026-11-19 takes the cap a 429 reports, not the lower of it and a published
 ceiling: the refusal describes the account as it actually stands, while a published ceiling is the
 plan's promise. A cap whose renewal time has passed stops governing.

 @param capacity - member state an earlier refusal wrote

 @param now - admission time

 @returns observed cap in force, or NaN when no refusal's cap applies

 @example
 ```ts
 const cap = observedCeiling({ capacity, now: Date.now() });
 ```
 */
export function observedCeiling({
  capacity,
  now,
}: {
  readonly capacity: MemberCapacity;
  readonly now: number;
},): number {
  /**
   Cap an earlier refusal recorded, which may have lapsed.
   */
  const { overage, } = capacity;
  if (overage === undefined)
    return Number.NaN;
  if ((overage.periodEnd > 0) && (overage.periodEnd <= now)) {
    delete capacity.overage;
    return Number.NaN;
  }
  return overage.concurrency;
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
