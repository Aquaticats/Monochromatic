/**
 Session-local admission for provider-published limits.
 @module
 */

import type { Api, Model, SimpleStreamOptions, } from '@earendil-works/pi-ai';
import { probeQuota, type QuotaEvidence, type RoutingRegistry, } from './quota-endpoints.ts';

//region One live view per exact member

/** Quota and active attempts associated with one provider and model in a Pi session. */
export type MemberCapacity = {
  /** Whether the next admission needs a fresh provider reading. */
  needsProbe: boolean;
  /** Shared read so overlapping requests do not all poll the quota endpoint. */
  pending?: Promise<void>;
  /** In-flight streams started by this plugin, not provider-wide load. */
  active: number;
  /** Retry/reset timestamp supplied by the provider or its documented guidance. */
  retryAt: number;
  /** Most recent published limits and subscription facts. */
  evidence: QuotaEvidence;
  /** Rejection's observed lower cap and the renewal that ends the exception. */
  overage?: { readonly concurrency: number; readonly periodEnd: number; };
};

/**
 Get or initialize one member's state.
 @param members - session-owned capacity map
 @param label - exact provider and model
 @returns mutable member state used only within the session
 */
export function capacityOf({
  members,
  label,
}: {
  members: Map<string, MemberCapacity>;
  label: string;
},): MemberCapacity {
  const existing = members.get(label,);
  if (existing !== undefined)
    return existing;
  const capacity: MemberCapacity = { needsProbe: true, active: 0, retryAt: 0, evidence: {}, };
  members.set(label, capacity,);
  return capacity;
}

//endregion One live view per exact member

//region Live quota reads and local ceiling

/**
 Use the caller's timeout for an optional quota read without aborting the model request.
 @param options - caller cancellation and transport timing policy
 @returns quota request cancellation
 */
function quotaSignal(options: SimpleStreamOptions | undefined,): AbortSignal | undefined {
  const { signal, timeoutMs, } = options ?? {};
  if (timeoutMs === undefined || !Number.isSafeInteger(timeoutMs,) || timeoutMs <= 0)
    return signal;
  const timeout = AbortSignal.timeout(timeoutMs,);
  return signal === undefined ? timeout : AbortSignal.any([signal, timeout,],);
}

/**
 Refresh quota at first use and on reentry after a refusal or exhausted window.
 @param capacity - state shared by the member's requests
 @param registry - registry used for the quota credential
 @param model - exact physical member
 @param options - probe transport and cancellation
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
  async function refresh(): Promise<void> {
    capacity.needsProbe = false;
    const evidence = await probeQuota({
      registry,
      model,
      fetchQuota: options?.fetch ?? globalThis.fetch,
      signal: quotaSignal(options,),
    },);
    if (options?.signal?.aborted) {
      capacity.needsProbe = true;
      return;
    }
    const now = Date.now();
    capacity.evidence = {
      concurrency: evidence.concurrency ?? capacity.evidence.concurrency,
      overage: evidence.overage ?? capacity.evidence.overage,
      periodEnd: evidence.periodEnd || capacity.evidence.periodEnd,
    };
    capacity.retryAt = Math.max(capacity.retryAt, evidence.retryAt ?? 0,);
    if (capacity.overage !== undefined) {
      if (evidence.overage === false
        || (capacity.overage.periodEnd > 0 && capacity.overage.periodEnd <= now))
        delete capacity.overage;
      else if (capacity.overage.periodEnd === 0 && (evidence.periodEnd ?? 0) > now)
        capacity.overage = { ...capacity.overage, periodEnd: evidence.periodEnd ?? 0, };
    }
  }

  if (capacity.pending !== undefined) {
    await capacity.pending;
    return;
  }
  if (!capacity.needsProbe && capacity.retryAt !== 0 && capacity.retryAt <= Date.now())
    capacity.needsProbe = true;
  if (capacity.overage !== undefined && capacity.overage.periodEnd <= Date.now())
    capacity.needsProbe = true;
  if (!capacity.needsProbe)
    return;
  capacity.pending = refresh();
  try {
    await capacity.pending;
  } finally {
    delete capacity.pending;
  }
}

/** NeuralWatt's exact pay-as-you-go exception, as recorded in the 2026-11-11 refusal. */
const OVERAGE_NOTICE = 'Your subscription is in overage with auto-renew off';

/** Concurrent slots reported by the admission service, not the catalog endpoint. */
const OVERAGE_CONCURRENCY = /Concurrent limit reached for [^:\r\n]+: \d+\/(\d+) slots in use/u;

/**
 Mark a genuine pre-output HTTP 429 and, only for NeuralWatt's named exception, retain its cap.
 @param capacity - admitted member's per-session state
 @param provider - member whose admission service produced the response
 @param errorMessage - terminal refusal text; never logged
 @param retryAt - known retry window, or zero if no timing evidence
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
  capacity.retryAt = retryAt;
  if (provider !== 'neuralwatt' || !errorMessage.includes(OVERAGE_NOTICE,))
    return;
  const concurrency = Number(OVERAGE_CONCURRENCY.exec(errorMessage,)?.[1],);
  if (Number.isSafeInteger(concurrency,) && concurrency > 0) {
    const periodEnd = capacity.evidence.periodEnd ?? 0;
    capacity.overage = { concurrency, periodEnd: periodEnd > Date.now() ? periodEnd : 0, };
  }
}

//endregion Live quota reads and local ceiling
