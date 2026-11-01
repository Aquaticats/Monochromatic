import { join, } from 'node:path';

import {
  createNeuralWattClient,
  createNeuralWattLedger,
  type NeuralWattClient,
} from '../dist/final/node/index.mjs';
import type {
  TransportExchange,
  TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream';
import { handClock, } from './neuralwatt-ledger.test-fixture.ts';
import {
  LISTED_MODELS,
  modelsReply,
} from './neuralwatt-models-listing.test-fixture.ts';
import { SEAT_HYPER_OPENROUTER_UNMEASURED, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region NeuralWatt scripted client test support
// A NEURALWATT CLIENT OVER A SCRIPTED TRANSPORT AND A LEDGER FILE IN A CASE'S OWN DIRECTORY, for the cases about the
// modules that sit behind the client: the quota session and the settlement. Neither is on the package index, so the
// cases reach them the way production does, through a call.
//
// TEST SUPPORT, NOT PACKAGE SOURCE: no case calls NeuralWatt or reads a key, the clock is the case's own, and the
// stream shape is the one the documentation prints (`https://docs.neuralwatt.com/energy-methodology.md`, fetched
// 2026-10-20).

/**
 Host every case points the client at; no case reaches it.
 */
export const BASE_URL = 'https://neuralwatt.invalid/v1';

/**
 Abort signal every call here carries.
 */
export const SIGNAL: AbortSignal = new AbortController().signal;

/**
 The energy comment the page prints: 0.000015 kWh, which is 0.015 Wh.
 */
export const ENERGY_COMMENT = ': energy {"energy_kwh": 0.000015, "energy_joules": 54.0, "attribution_ratio": 0.25}\n\n';

/**
 The cost comment of a free request.
 */
export const FREE_COST_COMMENT = ': cost {"request_cost_usd": 0, "cache_savings_usd": 0.0}\n\n';

/**
 The cost comment of a free-energy request that carries a nominal cost above zero, as the first keyed run's did
 (0.000107 USD for a request the dashboard counted wholly against the free energy).
 */
export const NOMINAL_COST_COMMENT = ': cost {"request_cost_usd": 0.00001}\n\n';

/**
 kWh the plan in every quota reply here includes.
 */
const QUOTA_INCLUDED_KWH = 20;

/**
 The cap, in USD, of the key in every allowance here.
 */
const KEY_LIMIT_USD = 5;

/**
 Reply a transport gives where the script has run out, which no case means to reach.
 */
const NO_SCRIPT_REPLY: TransportReply = {
  status: 500,
  bodyText: 'no script',
};

/**
 Quota reply of an account with no subscription, the field present and null.
 */
export const NO_SUBSCRIPTION_REPLY: TransportReply = {
  status: 200,
  bodyText: JSON.stringify({
    snapshot_at: '2026-10-20T20:00:00Z',
    subscription: null,
  },),
};

/**
 Quota reply whose subscription leaves the charged figure out, as null.
 */
export const UNREPORTED_QUOTA_REPLY: TransportReply = {
  status: 200,
  bodyText: JSON.stringify({
    snapshot_at: '2026-10-20T20:00:00Z',
    subscription: {
      plan: 'max',
      current_period_start: '2026-10-11T05:05:25Z',
      kwh_included: 20,
      kwh_used: null,
      kwh_remaining: 20,
      in_overage: false,
    },
  },),
};

/**
 Whole stream an endpoint answers with.

 @param comments - comment lines placed before the sentinel, each ending in a blank line

 @param done - whether the sentinel closes the stream

 @returns Body text

 @example
 ```ts
 const bodyText = streamOf({ comments: ENERGY_COMMENT, },);
 ```
 */
export function streamOf(
  {
    comments = '',
    done = true,
  }: {
    readonly comments?: string;
    readonly done?: boolean;
  },
): string {
  return [
    `data: ${JSON.stringify({
      id: 'chatcmpl-whiskers',
      object: 'chat.completion.chunk',
      choices: [{
        index: 0,
        delta: {
          role: 'assistant',
          content: '{"spot":"sunbeam"}',
        },
        finish_reason: null,
      },],
    },)}\n\n`,
    `data: ${JSON.stringify({
      id: 'chatcmpl-whiskers',
      object: 'chat.completion.chunk',
      choices: [],
      usage: {
        prompt_tokens: 90,
        completion_tokens: 10,
        total_tokens: 100,
      },
    },)}\n\n`,
    comments,
    done ? 'data: [DONE]\n\n' : '',
  ].join('',);
}

/**
 Quota reply with a given charged energy.

 @param kwhUsed - charged kWh this period

 @param periodStart - start of the billing period the reading belongs to

 @param periodStartAbsent - whether the reply carries the field as null instead

 @param keyAllowance - what `key.allowance` holds, as the reply would carry it: null for an uncapped key

 @returns Success reply

 @example
 ```ts
 const reply = quotaReply({ kwhUsed: 13.9, },);
 ```
 */
export function quotaReply(
  {
    kwhUsed,
    periodStart = '2026-10-11T05:05:25Z',
    periodStartAbsent = false,
    keyAllowance = null,
  }: {
    readonly kwhUsed: number;
    readonly periodStart?: string;
    readonly periodStartAbsent?: boolean;
    readonly keyAllowance?: unknown;
  },
): TransportReply {
  return {
    status: 200,
    bodyText: JSON.stringify({
      snapshot_at: '2026-10-20T20:00:00Z',
      subscription: {
        plan: 'max',
        current_period_start: periodStartAbsent ? null : periodStart,
        kwh_included: QUOTA_INCLUDED_KWH,
        kwh_used: kwhUsed,
        kwh_remaining: QUOTA_INCLUDED_KWH - kwhUsed,
        in_overage: false,
      },
      key: {
        name: 'whiskers',
        allowance: keyAllowance,
      },
    },),
  };
}

/**
 A per-key allowance as the quota schema states it (`KeyAllowance`).

 @param remainingUsd - what is left of the cap

 @param blocked - whether further requests are refused

 @param resetAt - when the cap's window ends, or null

 @returns The object the reply carries under `key.allowance`

 @example
 ```ts
 const allowance = keyAllowanceOf({ remainingUsd: 0, blocked: true, resetAt: '2026-10-21T00:00:00Z', },);
 ```
 */
export function keyAllowanceOf(
  {
    remainingUsd,
    blocked,
    resetAt,
  }: {
    readonly remainingUsd: number;
    readonly blocked: boolean;
    readonly resetAt?: string;
  },
): Readonly<Record<string, unknown>> {
  return {
    limit_usd: KEY_LIMIT_USD,
    period: 'daily',
    spent_usd: KEY_LIMIT_USD - remainingUsd,
    remaining_usd: remainingUsd,
    blocked,
    reset_at: resetAt ?? null,
  };
}

/**
 One `products[]` row of the by-model usage reply, with the fields these cases read.
 */
type UsageRow = {
  readonly requested_model: string;
  readonly served_model_id: string;
  readonly requests: number;
  readonly energy_kwh: number;
};

/**
 What the by-model route does on a read: answers with a body, drops the connection, or builds its answer from the
 request it was asked.
 */
type UsageScript = Error | TransportReply | ((exchange: TransportExchange,) => TransportReply);

/**
 Spells an instant the way the by-model reply echoes its period: `+00:00`, the fraction six digits where there is one.

 @param instant - ISO 8601 instant

 @returns The echo spelling

 @example
 ```ts
 echoSpelling({ instant: '2026-10-20T23:15:00.741Z', },); // '2026-10-20T23:15:00.741000+00:00'
 ```
 */
function echoSpelling({ instant, }: { readonly instant: string; },): string {
  /**
   The instant in the one spelling `toISOString` has.
   */
  const iso = new Date(instant,).toISOString();
  return iso.endsWith('.000Z',) ? iso.replace(
    '.000Z',
    '+00:00',
  ) : iso.replace(
    'Z',
    '000+00:00',
  );
}

/**
 A by-model reply for the given rows whose `period` echoes the window the request asked for, as the route does when it
 honours its input.

 @param rows - the `products[]` rows

 @param echo - a period to echo instead of the asked one: what the route answers when it falls back to its default
 window

 @returns Script entry building the reply from the request

 @example
 ```ts
 const usage = [usageReply({ rows: [{ requested_model: 'glm-5.3-flash', served_model_id: 'zai-org/GLM-5.3-Flash', requests: 9, energy_kwh: 0.002811, },], },),];
 ```
 */
export function usageReply(
  {
    rows,
    echo,
  }: {
    readonly rows: readonly UsageRow[];
    readonly echo?: {
      readonly start: string;
      readonly end: string
    };
  },
): UsageScript {
  return function build(exchange,): TransportReply {
    /**
     Query the request carried.
     */
    const { searchParams, } = new URL(exchange.url,);
    return {
      status: 200,
      bodyText: JSON.stringify({
        period: echo ?? {
          start: echoSpelling({ instant: searchParams.get('start_date',) ?? '', },),
          end: echoSpelling({ instant: searchParams.get('end_date',) ?? '', },),
        },
        models: [],
        products: rows,
      },),
    };
  };
}

/**
 One covered `products[]` row.

 @param requestedModel - the name the call used

 @param kwh - energy consumed, in kWh

 @param servedModelId - the backend model that served it, as the route spells it

 @returns The row

 @example
 ```ts
 const row = usageRowOf({ requestedModel: 'glm-5.3-flash', kwh: 0.002811, },);
 ```
 */
export function usageRowOf({
  requestedModel,
  kwh,
  servedModelId = `served/${requestedModel}`,
}: {
  readonly requestedModel: string;
  readonly kwh: number;
  readonly servedModelId?: string
},): UsageRow {
  return {
    requested_model: requestedModel,
    served_model_id: servedModelId,
    requests: 1,
    energy_kwh: kwh,
  };
}

/**
 Builds a client over a scripted transport and a ledger file in a given
 directory.

 @param dir - directory the ledger file lives in

 @param dailyWh - the day's allowance

 @param chat - replies to chat completions, in order, the last repeating

 @param quota - replies to quota reads, in order, the last repeating; an error stands for a read the connection dropped

 @param models - replies to listing reads, in order, the last repeating; the listing of 2026-10-20 where none is given

 @returns Client, ledger, and what the transport saw

 @example
 ```ts
 const { client, ledger, } = scriptedClient({ dir, },);
 ```
 */
export function scriptedClient(
  {
    dir,
    dailyWh = 550,
    chat = [{
      status: 200,
      bodyText: streamOf({ comments: ENERGY_COMMENT + FREE_COST_COMMENT, }),
    },],
    quota = [quotaReply({ kwhUsed: 13.9023, }),],
    usage = [usageReply({ rows: [], },),],
    models = [modelsReply({ entries: LISTED_MODELS, },),],
    clockStart = '2026-10-20T12:00:00.000Z',
    dayStartSetting,
  }: {
    readonly dir: string;
    readonly dailyWh?: number;
    readonly chat?: readonly TransportReply[];
    readonly quota?: readonly (Error | TransportReply)[];
    readonly usage?: readonly UsageScript[];
    readonly models?: readonly (Error | TransportReply)[];
    readonly clockStart?: string;
    readonly dayStartSetting?: { readonly msOfDay: number; };
  },
): {
  readonly client: NeuralWattClient;
  readonly ledger: ReturnType<typeof createNeuralWattLedger>;
  readonly exchanges: readonly TransportExchange[];
  readonly clock: ReturnType<typeof handClock>;
} {
  /**
   Every exchange the transport was handed.
   */
  const exchanges: TransportExchange[] = [];

  /**
   Exchanges of each kind seen so far, for scripting by position.
   */
  const counts = {
    chat: 0,
    quota: 0,
    usage: 0,
    models: 0,
  };

  /**
   Clock the ledger and the client read, which a case moves by hand.
   */
  const clock = handClock({ start: clockStart, },);

  /**
   Ledger the client spends against, on a clock the case does not move.
   */
  const ledger = createNeuralWattLedger({
    path: join(
      dir,
      'neuralwatt-energy.jsonl',
    ),
    dailyWh,
    now: clock.now,
  },);
  return {
    exchanges,
    ledger,
    clock,
    client: createNeuralWattClient({
      apiKey: 'test-key',
      ledger,
      baseUrl: BASE_URL,
      now: clock.now,
      ...((dayStartSetting === undefined) ? {} : { dayStartSetting, }),
      transport: function transport(exchange,): Promise<TransportReply> {
        exchanges.push(exchange,);
        if ((exchange.method === 'GET')
          && exchange.url
            .includes('/usage/by-model',)) {
          counts.usage += 1;

          /**
           What this by-model read is scripted to get.
           */
          const scriptedUsage = usage[Math.min(
            counts.usage - 1,
            usage.length - 1,
          )] ?? NO_SCRIPT_REPLY;
          if (Error.isError(scriptedUsage,))
            return Promise.reject(scriptedUsage,);
          return Promise.resolve((typeof scriptedUsage) === 'function'
            ? (scriptedUsage as (exchange: TransportExchange,) => TransportReply)(exchange,)
            : scriptedUsage as TransportReply,);
        }
        if ((exchange.method === 'GET')
          && exchange.url
            .endsWith('/models',)) {
          counts.models += 1;

          /**
           What this listing read is scripted to get.
           */
          const listed = models[Math.min(
            counts.models - 1,
            models.length - 1,
          )] ?? NO_SCRIPT_REPLY;
          return Error.isError(listed,) ? Promise.reject(listed,) : Promise.resolve(listed,);
        }
        if (exchange.method === 'GET') {
          counts.quota += 1;

          /**
           What this quota read is scripted to get.
           */
          const scripted = quota[Math.min(
            counts.quota - 1,
            quota.length - 1,
          )] ?? NO_SCRIPT_REPLY;
          return Error.isError(scripted,) ? Promise.reject(scripted,) : Promise.resolve(scripted,);
        }
        counts.chat += 1;
        return Promise.resolve(
          chat[Math.min(
            counts.chat - 1,
            chat.length - 1,
          )] ?? NO_SCRIPT_REPLY,
        );
      },
      retryPolicy: {
        limit: 0,
        baseMs: 1,
      },
    },),
  };
}

/**
 Chat exchanges among those seen.

 @param exchanges - everything the transport saw

 @returns The POSTs

 @example
 ```ts
 const posts = chatExchangesOf({ exchanges, },);
 ```
 */
export function chatExchangesOf(
  { exchanges, }: { readonly exchanges: readonly TransportExchange[]; },
): readonly TransportExchange[] {
  return exchanges.filter(function isPost(exchange,): boolean {
    return exchange.method === 'POST';
  },);
}

/**
 Quota exchanges among those seen.

 @param exchanges - everything the transport saw

 @returns The GETs

 @example
 ```ts
 const reads = quotaExchangesOf({ exchanges, },);
 ```
 */
export function quotaExchangesOf(
  { exchanges, }: { readonly exchanges: readonly TransportExchange[]; },
): readonly TransportExchange[] {
  return exchanges.filter(function isQuotaGet(exchange,): boolean {
    return (exchange.method === 'GET')
      && exchange.url
        .endsWith('/quota',);
  },);
}

/**
 Listing exchanges among those seen.

 @param exchanges - everything the transport saw

 @returns The GETs

 @example
 ```ts
 const reads = modelsExchangesOf({ exchanges, },);
 ```
 */
export function modelsExchangesOf(
  { exchanges, }: { readonly exchanges: readonly TransportExchange[]; },
): readonly TransportExchange[] {
  return exchanges.filter(function isModelsGet(exchange,): boolean {
    return (exchange.method === 'GET')
      && exchange.url
        .endsWith('/models',);
  },);
}

/**
 By-model usage exchanges among those seen.

 @param exchanges - everything the transport saw

 @returns The GETs of the by-model route

 @example
 ```ts
 const reads = usageExchangesOf({ exchanges, },);
 ```
 */
export function usageExchangesOf(
  { exchanges, }: { readonly exchanges: readonly TransportExchange[]; },
): readonly TransportExchange[] {
  return exchanges.filter(function isUsageGet(exchange,): boolean {
    return (exchange.method === 'GET')
      && exchange.url
        .includes('/usage/by-model',);
  },);
}

/**
 Asks the DeepSeek seat one short question.

 @param client - client under test

 @returns The reply

 @example
 ```ts
 const reply = await ask({ client, },);
 ```
 */
export async function ask(
  { client, }: { readonly client: NeuralWattClient; },
): ReturnType<NeuralWattClient['chatText']> {
  return await client.chatText({
    modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
    messages: [{
      role: 'user',
      content: 'Where does the cat sleep?',
    },],
    signal: SIGNAL,
  },);
}

//endregion NeuralWatt scripted client test support
