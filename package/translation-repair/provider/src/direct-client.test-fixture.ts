/**
 Shared fixtures for the DeepSeek and Xiaomi client tests: recorded transports and the stream shapes the owner's grants
 of 2026-10-22 were probed with. Nothing here reaches a provider, and every key a case passes in is invented.

 @module
 */

import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type {
  ModelTransport,
  TransportExchange,
  TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ModelCaller, } from '../dist/final/node/index.mjs';

/**
 One chat completion chunk as the endpoint sends it.

 @param delta - delta fields for the single choice

 @returns Event line, newline-terminated

 @example
 ```ts
 const raw = chunkOf({ delta: { content: '{"spot":', }, },);
 ```
 */
function chunkOf({ delta, }: { readonly delta: Readonly<Record<string, unknown>>; },): string {
  return `data: ${JSON.stringify({
    id: 'chatcmpl-6f1d0c4e',
    object: 'chat.completion.chunk',
    created: 1_788_811_674,
    system_fingerprint: 'fp_cat_build_7',
    choices: [{
      index: 0,
      delta,
      finish_reason: null,
    },],
  },)}\n\n`;
}

/**
 Usage chunk with DeepSeek's cache fields: 40 of 100 prompt tokens hit.
 */
export const DEEPSEEK_USAGE_CHUNK: string = `data: ${JSON.stringify({
  id: 'chatcmpl-6f1d0c4e',
  object: 'chat.completion.chunk',
  system_fingerprint: 'fp_cat_build_7',
  choices: [],
  usage: {
    prompt_tokens: 100,
    completion_tokens: 10,
    total_tokens: 110,
    prompt_cache_hit_tokens: 40,
    prompt_cache_miss_tokens: 60,
  },
},)}\n\n`;

/**
 Usage chunk with the cached count Xiaomi reports: 50 of 100 prompt tokens.
 */
export const XIAOMI_USAGE_CHUNK: string = `data: ${JSON.stringify({
  id: 'chatcmpl-9b2a7710',
  object: 'chat.completion.chunk',
  choices: [],
  usage: {
    prompt_tokens: 100,
    completion_tokens: 10,
    total_tokens: 110,
    prompt_tokens_details: { cached_tokens: 50, },
  },
},)}\n\n`;

/**
 Body of a whole stream, comment lines and reasoning included.

 @param usageChunk - the closing usage chunk

 @param done - whether the stream ends on its `[DONE]` sentinel

 @returns Stream text

 @example
 ```ts
 const body = streamOf({ usageChunk: DEEPSEEK_USAGE_CHUNK, done: true, },);
 ```
 */
export function streamOf({
  usageChunk,
  done,
}: {
  readonly usageChunk: string;
  readonly done: boolean
},): string {
  return [
    ': keep-alive\n\n',
    chunkOf({ delta: {
      role: 'assistant',
      content: '',
      reasoning_content: 'The cat is thinking of naps.',
    }, },),
    chunkOf({ delta: { content: '{"spot":', }, },),
    ': keep-alive\n\n',
    chunkOf({ delta: { content: '"sunbeam"}', }, },),
    usageChunk,
    done ? 'data: [DONE]\n\n' : '',
  ].join('',);
}

/**
 Abort signal every call here carries.
 */
export const SIGNAL: AbortSignal = new AbortController().signal;

/**
 Retry pacing the cases use: one retry after a millisecond.
 */
export const ONE_QUICK_RETRY = {
  limit: 1,
  baseMs: 1,
} as const;

/**
 Response schema a stage would hand over.
 */
export const NAP_SCHEMA = {
  type: 'json_schema',
  json_schema: {
    name: 'nap_spot',
    strict: true,
    schema: {
      type: 'object',
      required: ['spot',],
      properties: { spot: { type: 'string', }, },
    },
  },
} as const;

/**
 Transport answering from a script, recording what it was sent.

 @param replies - replies in order; the last repeats

 @returns Transport and the exchanges it saw

 @example
 ```ts
 const { transport, exchanges, } = scripted({ replies: [{ status: 200, bodyText: '', },], },);
 ```
 */
export function scripted(
  { replies, }: { readonly replies: readonly {
    readonly status: number;
    readonly bodyText: string
  }[]; },
): {
  readonly exchanges: TransportExchange[];
  readonly transport: ModelTransport
} {
  /**
   Every exchange the transport was handed.
   */
  const exchanges: TransportExchange[] = [];
  return {
    exchanges,
    transport: function transport(exchange: TransportExchange,): Promise<TransportReply> {
      exchanges.push(exchange,);
      /**
       The scripted reply for this turn, the last one once the script runs out.
       */
      const reply = replies[Math.min(
        exchanges.length - 1,
        replies.length - 1,
      )];
      if (reply === undefined)
        return Promise.reject(new Error('the script holds no reply',),);
      return Promise.resolve(reply,);
    },
  };
}

/**
 Asks the one invented question of the cat over a client.

 @param client - client under test

 @param modelId - roster model addressed

 @returns The pending reply

 @example
 ```ts
 const reply = await askCat({ client, modelId: SEAT_OPENROUTER_ONLY_CHECKER, },);
 ```
 */
export function askCat(
  {
    client,
    modelId,
  }: {
    readonly client: Pick<ModelCaller, 'chatText'>;
    readonly modelId: RosterModelId
  },
): ReturnType<ModelCaller['chatText']> {
  return client.chatText({
    modelId,
    messages: [{
      role: 'user',
      content: 'Where does the cat sleep?',
    },],
    signal: SIGNAL,
  },);
}

/**
 Most links of a cause chain the reader follows, so a cyclic chain ends.
 */
const CAUSE_CHAIN_LIMIT = 16;

/**
 Every text a thrown error carries: its message and the message of each error on its `cause` chain.

 @param error - caught value, of unknown type by construction

 @returns The texts joined, so one assertion covers the whole chain

 @example
 ```ts
 expect(errorChainText({ error: failure, },),).not.toContain('Qx4d',);
 ```
 */
export function errorChainText({ error, }: { readonly error: unknown; },): string {
  /**
   The texts met so far, outermost first.
   */
  const texts: string[] = [];
  for (
    let link: unknown = error;
    (texts.length < CAUSE_CHAIN_LIMIT) && (Error.isError(link,) || ((typeof link) === 'string'));
    link = Error.isError(link,) ? link.cause : undefined
  )
    texts.push(Error.isError(link,) ? link.message : (link as string),);
  return texts.join('\n',);
}

/**
 Body of the first exchange, parsed.

 @param exchanges - what the transport saw

 @returns The JSON body of the first exchange

 @example
 ```ts
 const body = firstBodyOf({ exchanges, },);
 ```
 */
export function firstBodyOf(
  { exchanges, }: { readonly exchanges: readonly TransportExchange[]; },
): Record<string, unknown> {
  /**
   The first exchange the transport saw.
   */
  const [first,] = exchanges;
  if (first === undefined)
    throw new Error('nothing was sent',);
  /**
   The body as parsed, of unknown shape until checked.
   */
  const parsed: unknown = JSON.parse(first.bodyJson ?? '{}',);
  if (!isJsonRecord(parsed,))
    throw new Error('the body was not a JSON object',);
  return parsed;
}
