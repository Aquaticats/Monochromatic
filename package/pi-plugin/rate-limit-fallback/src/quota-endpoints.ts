/**
 Read provider-published quota at routing decision points without exporting Pi credentials.
 @module
 */

import type { Api, FetchFunction, Model, } from '@earendil-works/pi-ai';
import type { ModelRegistry, } from '@earendil-works/pi-coding-agent';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { field, finiteNumber, resetTime, syntheticReset, } from './rate-limit-evidence.ts';

/** Registry operations the dispatcher and its probes need. */
export type RoutingRegistry = Pick<ModelRegistry, 'find' | 'getApiKeyAndHeaders' | 'streamSimple'>;

/** Logger whose diagnostics contain only provider identity and parsed capacity. */
const l = tagged({ tag: 'rate-limit-fallback', },);

/** Synthetic's account quota origin and endpoint. */
const SYNTHETIC_QUOTA = 'https://api.synthetic.new/v2/quotas';

/** NeuralWatt's per-model rate-limit endpoint, including the flex lane. */
const NEURALWATT_LIMITS = 'https://api.neuralwatt.com/v1/rate_limits';

/** NeuralWatt's subscription and renewal metadata endpoint. */
const NEURALWATT_QUOTA = 'https://api.neuralwatt.com/v1/quota';

/** Only the provider-published facts that affect routing. */
export type QuotaEvidence = {
  /** When a reported exhausted window can first be checked again. */
  readonly retryAt?: number | undefined;
  /** Published ceiling for the requested model and lane, not free slots. */
  readonly concurrency?: number | undefined;
  /** Whether the known pay-as-you-go exception is currently active. */
  readonly overage?: boolean | undefined;
  /** End of the provider's current subscription period. */
  readonly periodEnd?: number | undefined;
};

/**
 Fetch one official endpoint without letting redirects, errors or body samples disclose credentials.
 @param url - fixed provider endpoint
 @param key - registry-resolved bearer credential
 @param fetchQuota - request transport
 @param signal - probe cancellation
 @returns parsed quota body; unavailable evidence is undefined
 */
async function requestQuota({
  url,
  key,
  fetchQuota,
  signal,
}: {
  url: string | URL;
  key: string;
  fetchQuota: FetchFunction;
  signal: AbortSignal | undefined;
},): Promise<unknown> {
  const rl = tagged({ tag: requestQuota.name, l, },);
  try {
    const response = await fetchQuota(url, {
      headers: { authorization: `Bearer ${key}`, },
      redirect: 'error',
      ...(signal === undefined ? {} : { signal, }),
    },);
    if (!response.ok) {
      await response.body?.cancel();
      rl.debug(`Quota endpoint HTTP ${response.status}`,);
      return undefined;
    }
    return await response.json();
  } catch (error) {
    if (error !== undefined && !signal?.aborted)
      rl.debug('Quota response unavailable',);
    return undefined;
  }
}

/**
 Interpret only the requested lane: NeuralWatt documents a separate flex ceiling.
 @param limits - one model's rate-limit response
 @param quota - account quota response
 @returns qualified ceiling and subscription state
 */
function neuralWattEvidence({ limits, quota, }: { limits: unknown; quota: unknown; },): QuotaEvidence {
  const models = field({ value: limits, key: 'models', },);
  const row: unknown = Array.isArray(models,) && models.length === 1 ? models[0] : undefined;
  const concurrency = finiteNumber(field({ value: row, key: 'flex_floor', },),);
  const subscription = field({ value: quota, key: 'subscription', },);
  const inOverage = field({ value: subscription, key: 'in_overage', },);
  const autoRenew = field({ value: subscription, key: 'auto_renew', },);
  return {
    concurrency: concurrency !== undefined && Number.isSafeInteger(concurrency,) && concurrency >= 0
      ? concurrency
      : undefined,
    overage: typeof inOverage === 'boolean' && typeof autoRenew === 'boolean'
      ? inOverage && !autoRenew
      : undefined,
    periodEnd: resetTime(field({ value: subscription, key: 'current_period_end', },),),
  };
}

/**
 Check quota only when it belongs to the configured model's origin.
 @param registry - sole credential resolver
 @param model - physical model found in Pi
 @param fetchQuota - request transport, injectable by the consumer
 @param signal - parent request cancellation and its timeout, when supplied
 @returns usable limit facts, or an empty object when no evidence is available
 */
export async function probeQuota({
  registry,
  model,
  fetchQuota,
  signal,
}: {
  registry: RoutingRegistry;
  model: Model<Api>;
  fetchQuota: FetchFunction;
  signal: AbortSignal | undefined;
},): Promise<QuotaEvidence> {
  const rl = tagged({ tag: probeQuota.name, l, },);
  const { provider, } = model;
  if (provider !== 'synthetic' && provider !== 'neuralwatt')
    return {};
  const url = provider === 'synthetic' ? SYNTHETIC_QUOTA : NEURALWATT_LIMITS;
  try {
    const origin = new URL(url,).origin;
    if (URL.parse(model.baseUrl,)?.origin !== origin)
      return {};
    const auth = await registry.getApiKeyAndHeaders(model,);
    signal?.throwIfAborted();
    if (!auth.ok || auth.apiKey === undefined || auth.apiKey === ''
      || URL.parse(auth.baseUrl ?? model.baseUrl,)?.origin !== origin)
      return {};
    const request = { key: auth.apiKey, fetchQuota, signal, };
    if (provider === 'synthetic')
      return { retryAt: syntheticReset(await requestQuota({ url, ...request, },),), };
    const limitsUrl = new URL(url,);
    limitsUrl.searchParams.set('model', model.id,);
    const [limits, quota,] = await Promise.all([
      requestQuota({ url: limitsUrl, ...request, },),
      requestQuota({ url: NEURALWATT_QUOTA, ...request, },),
    ],);
    return neuralWattEvidence({ limits, quota, },);
  } catch (error) {
    // Do not log raw credential-resolution errors.
    if (error !== undefined && !signal?.aborted)
      rl.debug(`${provider} registry auth for quota read unavailable`,);
    return {};
  }
}
