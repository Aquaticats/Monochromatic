/**
 Read provider-published quota at routing decision points without exporting Pi credentials.
 @module
 */

import type {
  Api,
  FetchFunction,
  Model,
} from '@earendil-works/pi-ai';
import type { ModelRegistry, } from '@earendil-works/pi-coding-agent';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  field,
  finiteNumber,
  resetTime,
  syntheticReset,
} from './rate-limit-evidence.ts';

/**
 Registry operations the dispatcher and its probes need.
 */
export type RoutingRegistry = Pick<ModelRegistry, 'find' | 'getApiKeyAndHeaders' | 'streamSimple'>;

/** A supported provider's read failed or could not be attempted; try again on reentry. */
export const QUOTA_UNAVAILABLE: unique symbol = Symbol('Quota reading unavailable',);

/**
 Logger whose diagnostics contain only provider identity and parsed capacity.
 */
const l = tagged({ tag: 'rate-limit-fallback', },);

/**
 Synthetic's account quota origin and endpoint.
 */
const SYNTHETIC_QUOTA = 'https://api.synthetic.new/v2/quotas';

/**
 NeuralWatt's per-model rate-limit endpoint, including the flex lane.
 */
const NEURALWATT_LIMITS = 'https://api.neuralwatt.com/v1/rate_limits';

/**
 NeuralWatt's subscription and renewal metadata endpoint.
 */
const NEURALWATT_QUOTA = 'https://api.neuralwatt.com/v1/quota';

/**
 Only the provider-published facts that affect routing.
 */
export type QuotaEvidence = {
  /**
   When a reported exhausted window can first be checked again.
   */
  readonly retryAt?: number;
  /**
   Published ceiling for the requested model and lane, not free slots.
   */
  readonly concurrency?: number;
  /**
   Whether the known pay-as-you-go exception is currently active.
   */
  readonly overage?: boolean;
  /**
   End of the provider's current subscription period.
   */
  readonly periodEnd?: number;
};

/**
 Fetch one official endpoint without letting redirects, errors or body samples disclose credentials.

 @param url - fixed provider endpoint

 @param bearer - registry-resolved authorization value

 @param fetchQuota - request transport

 @param signal - probe cancellation

 @returns parsed quota body; unavailable evidence is undefined
 */
async function requestQuota({
  url,
  bearer,
  fetchQuota,
  signal,
}: {
  url: string | URL;
  bearer: string;
  fetchQuota: FetchFunction;
  signal?: AbortSignal;
},): Promise<unknown> {
  /**
   Logger siblings can correlate with this quota request without reading its authorization.
   */
  const rl = tagged({
    tag: requestQuota.name,
    l,
  },);
  try {
    /**
     Headers and response status, before a successful body is consumed.
     */
    const response = await fetchQuota(
      url,
      {
        headers: { authorization: bearer, },
        redirect: 'error',
        ...(signal === undefined ? {} : { signal, }),
      },
    );
    if (!response.ok) {
      await response.body
        ?.cancel();
      rl.debug(`Quota endpoint HTTP ${response.status}`,);
      return undefined;
    }
    return await response.json();
  } catch (error) {
    if ((error !== undefined) && (signal?.aborted !== true))
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
function neuralWattEvidence({
  limits,
  quota,
}: {
  limits: unknown;
  quota: unknown
},): QuotaEvidence {
  /**
   Candidate responses from the model-scoped limits lookup.
   */
  const models = field({
    value: limits,
    key: 'models',
  },);
  /**
   Single result row for a model-scoped NeuralWatt lookup.
   */
  const row: unknown = Array.isArray(models,) && (models.length === 1) ? models[0] : undefined;
  /**
   Provider-reported ceiling for the low-priority lane.
   */
  const concurrency = finiteNumber(field({
    value: row,
    key: 'flex_floor',
  },),);
  /**
   Subscription data used for the pay-as-you-go exception.
   */
  const subscription = field({
    value: quota,
    key: 'subscription',
  },);
  /**
   Whether included subscription energy was exhausted.
   */
  const inOverage = field({
    value: subscription,
    key: 'in_overage',
  },);
  /**
   Whether the account has auto-renew enabled.
   */
  const autoRenew = field({
    value: subscription,
    key: 'auto_renew',
  },);
  return {
    ...(Number.isSafeInteger(concurrency,) && (concurrency >= 0) ? { concurrency, } : {}),
    ...(((typeof inOverage) === 'boolean') && ((typeof autoRenew) === 'boolean')
      ? { overage: inOverage && (!autoRenew), }
      : {}),
    periodEnd: resetTime(field({
      value: subscription,
      key: 'current_period_end',
    },),),
  };
}

/**
 Check quota only when it belongs to the configured model's origin.

 @param registry - sole credential resolver

 @param model - physical model found in Pi

 @param fetchQuota - request transport, injectable by the consumer

 @param signal - quota deadline and parent request cancellation

 @returns usable limit facts, empty facts when no quota contract applies, or `QUOTA_UNAVAILABLE` for a failed read

 @example
 ```ts
 const evidence = await probeQuota({ registry, model, fetchQuota });
 ```
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
  signal?: AbortSignal;
},): Promise<QuotaEvidence | typeof QUOTA_UNAVAILABLE> {
  /**
   Probe diagnostics, excluding credentials and raw provider responses.
   */
  const rl = tagged({
    tag: probeQuota.name,
    l,
  },);
  /**
   Provider identity used to select an official quota contract.
   */
  const { provider, } = model;
  if ((provider !== 'synthetic') && (provider !== 'neuralwatt'))
    return {};
  /**
   Official endpoint against which authentication is checked.
   */
  const url = provider === 'synthetic' ? SYNTHETIC_QUOTA : NEURALWATT_LIMITS;
  try {
    /**
     Trusted destination on which the registry credential may be used.
     */
    const {origin} = new URL(url,);
    if (URL.parse(model.baseUrl,)
      ?.origin
      !== origin)
      return QUOTA_UNAVAILABLE;
    /**
     Member credentials and endpoint overrides as resolved by Pi.
     */
    const auth = await registry.getApiKeyAndHeaders(model,);
    signal?.throwIfAborted();
    if ((!auth.ok) || (URL.parse(auth.baseUrl ?? model.baseUrl,)
      ?.origin
      !== origin))
      return QUOTA_UNAVAILABLE;
    /**
     An authorization override, including explicit suppression.
     */
    const explicit = Object.entries(auth.headers ?? {},)
      .find(function isAuthorization([header,]: readonly [
        string,
        unknown
      ],) {
        return header.toLowerCase() === 'authorization';
      },);
    /**
     Bearer value permitted by the quota endpoints.
     */
    const bearer = explicit === undefined
      ? ((auth.apiKey === undefined) || (auth.apiKey === '') ? '' : `Bearer ${auth.apiKey}`)
      : (explicit[1] ?? '');
    if ((!bearer.toLowerCase()
      .startsWith('bearer ',)) || (bearer.trimEnd()
        .length
        <= 'Bearer '.length))
      return QUOTA_UNAVAILABLE;
    /**
     Transport and authorized header shared by the member's quota GETs.
     */
    const request = {
      bearer,
      fetchQuota,
      ...(signal === undefined ? {} : { signal, }),
    };
    if (provider === 'synthetic') {
      /** Response body when the quota transport and JSON read succeeded. */
      const quota = await requestQuota({
        url,
        ...request,
      },);
      return quota === undefined ? QUOTA_UNAVAILABLE : { retryAt: syntheticReset(quota,), };
    }
    /**
     Rate-limit lookup scoped to the exact model and lane.
     */
    const limitsUrl = new URL(url,);
    limitsUrl.searchParams
      .set(
      'model',
      model.id,
      );
    /**
     Independently read model ceiling and account renewal information.
     */
    const [limits, quota,] = await Promise.all([
      requestQuota({
        url: limitsUrl,
        ...request,
      },),
      requestQuota({
        url: NEURALWATT_QUOTA,
        ...request,
      },),
    ],);
    return (limits === undefined) && (quota === undefined) ? QUOTA_UNAVAILABLE : neuralWattEvidence({
      limits,
      quota,
    },);
  } catch (error) {
    // Do not log raw credential-resolution errors.
    if ((error !== undefined) && (signal?.aborted !== true))
      rl.debug(`${provider} registry auth for quota read unavailable`,);
    return QUOTA_UNAVAILABLE;
  }
}
