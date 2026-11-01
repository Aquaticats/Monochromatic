import { isSuccessStatus, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  exchangeFailureLogText,
  type ModelTransport,
  ProviderHttpError,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  aliasNamesOf,
  type NeuralWattAliasMap,
} from './neuralwatt-aliases.ts';
import {
  neuralWattModelsUrlFor,
  neuralWattUsageByModelUrlFor,
} from './neuralwatt-catalog.ts';
import { coveredWhOf, } from './neuralwatt-usage.ts';
import {
  exchangeWithRetry,
  type RetryPolicy,
} from './transient-retry.ts';

//region NeuralWatt account read
// ONE READ OF THE ACCOUNT'S COVERED ENERGY for a window, over the transport seam. It throws on any failure, so the
// day session (`neuralwatt-day-session.ts`) decides what an outage means: the last figure for the same day stands,
// else the ledger counts alone, with one warning.

/**
 Reads the covered energy the whole account used in a window.

 @param headers - request headers, auth included

 @param baseUrl - API base address

 @param transport - HTTP the read goes over

 @param retryPolicy - transient-retry pacing

 @param asked - the window: the day's start to now

 @param aliases - the tracking aliases whose calls count too

 @param signal - abort signal for the read

 @returns Watt-hours the covered models' rows add up to

 @throws {@link ProviderHttpError} on a non-success status

 @throws {@link import('./neuralwatt-usage.ts').NeuralWattUsageShapeError} on a body that is not the documented
 shape or answers for another window than the one asked

 @example
 ```ts
 const wh = await readAccountCoveredWh({ headers, baseUrl, transport, retryPolicy, asked, aliases, signal, },);
 ```
 */
export async function readAccountCoveredWh(
  {
    headers,
    baseUrl,
    transport,
    retryPolicy,
    asked,
    aliases,
    signal,
  }: {
    readonly headers: Readonly<Record<string, string>>;
    readonly baseUrl: string;
    readonly transport: ModelTransport;
    readonly retryPolicy: RetryPolicy;
    readonly asked: {
      readonly start: Date;
      readonly end: Date;
    };
    readonly aliases: NeuralWattAliasMap;
    readonly signal: AbortSignal;
  },
): Promise<number> {
  /**
   Reply from the by-model route.
   */
  const reply = await exchangeWithRetry({
    provider: 'neuralwatt',
    transport,
    exchange: {
      url: neuralWattUsageByModelUrlFor({
        baseUrl,
        ...asked,
      },),
      label: 'usage-by-model',
      method: 'GET',
      headers,
      signal,
    },
    policy: retryPolicy,
  },);
  if (!isSuccessStatus({ status: reply.status, },))
    throw new ProviderHttpError({
      provider: 'neuralwatt',
      status: reply.status,
      bodyText: reply.bodyText,
      summary: `NeuralWatt /usage/by-model returned HTTP ${String(reply.status,)}:`,
    },);
  return coveredWhOf({
    bodyText: reply.bodyText,
    asked,
    aliases,
  },);
}

/**
 Reads which names are tracking aliases of a covered model.

 @param headers - request headers, auth included

 @param baseUrl - API base address

 @param transport - HTTP the read goes over

 @param retryPolicy - transient-retry pacing

 @param signal - abort signal for the read

 @returns The aliases, or why the listing gave none

 @throws {@link ProviderHttpError} on a non-success status

 @example
 ```ts
 const aliases = await readAliasNames({ headers, baseUrl, transport, retryPolicy, signal, },);
 ```
 */
async function readAliasNames(
  {
    headers,
    baseUrl,
    transport,
    retryPolicy,
    signal,
  }: {
    readonly headers: Readonly<Record<string, string>>;
    readonly baseUrl: string;
    readonly transport: ModelTransport;
    readonly retryPolicy: RetryPolicy;
    readonly signal: AbortSignal;
  },
): Promise<NeuralWattAliasMap> {
  /**
   Reply from the models route.
   */
  const reply = await exchangeWithRetry({
    provider: 'neuralwatt',
    transport,
    exchange: {
      url: neuralWattModelsUrlFor({ baseUrl, },),
      label: 'models',
      method: 'GET',
      headers,
      signal,
    },
    policy: retryPolicy,
  },);
  if (!isSuccessStatus({ status: reply.status, },))
    throw new ProviderHttpError({
      provider: 'neuralwatt',
      status: reply.status,
      bodyText: reply.bodyText,
      summary: `NeuralWatt /models returned HTTP ${String(reply.status,)}:`,
    },);
  return aliasNamesOf({ bodyText: reply.bodyText, },);
}

/**
 Reads which names are tracking aliases of a covered model, and says why not where the listing cannot be used.

 Never throws for a failed read, so the count can go on by the served model of each row; an abort still throws.

 @param headers - request headers, auth included

 @param baseUrl - API base address

 @param transport - HTTP the read goes over

 @param retryPolicy - transient-retry pacing

 @param signal - abort signal for the read

 @returns The aliases, or why the listing gave none

 @example
 ```ts
 const aliases = await readAliasMap({ headers, baseUrl, transport, retryPolicy, signal, },);
 ```
 */
export async function readAliasMap(
  {
    headers,
    baseUrl,
    transport,
    retryPolicy,
    signal,
  }: {
    readonly headers: Readonly<Record<string, string>>;
    readonly baseUrl: string;
    readonly transport: ModelTransport;
    readonly retryPolicy: RetryPolicy;
    readonly signal: AbortSignal;
  },
): Promise<NeuralWattAliasMap> {
  try {
    return await readAliasNames({
      headers,
      baseUrl,
      transport,
      retryPolicy,
      signal,
    },);
  }
  catch (error) {
    signal.throwIfAborted();
    return {
      known: false,
      why: exchangeFailureLogText({ error, },),
    };
  }
}

//endregion NeuralWatt account read
