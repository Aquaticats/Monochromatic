/**
 Shared fixtures for the Claude client tests: Messages streams as the wire sends them, built the way the API
 reference documents them. Nothing here reaches a provider, and every key a case passes in is invented.

 @module
 */

import type {
  ModelTransport,
  TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream';
import {
  createClaudeClient,
  type ModelClient,
} from '../dist/final/node/index.mjs';
import {
  anthropicBlockDelta,
  anthropicBlockStart,
  anthropicFrameOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/anthropic-frames.test-fixture.ts';

/**
 Usage a stream reports, in the Messages protocol's own fields.

 @example
 ```ts
 const usage: ClaudeStreamUsage = { input: 100, cacheWrite: 0, cacheRead: 0, output: 10, };
 ```
 */
type ClaudeStreamUsage = {
  /**
   Prompt tokens past the last cache breakpoint.
   */
  readonly input: number;

  /**
   Prompt tokens written to the cache.
   */
  readonly cacheWrite: number;

  /**
   Prompt tokens read from the cache.
   */
  readonly cacheRead: number;

  /**
   Completion tokens, thinking included.
   */
  readonly output: number;
};

/**
 Usage the cases use unless they state their own: 100 plain prompt tokens and 10 completion tokens.
 */
export const PLAIN_USAGE: ClaudeStreamUsage = {
  input: 100,
  cacheWrite: 0,
  cacheRead: 0,
  output: 10,
};

/**
 Body of a whole Messages stream: a thinking block, a text block, and the terminator.

 @param text - the answer the text block carries, empty for none

 @param stopReason - why the model stopped

 @param stopDetails - the `stop_details` object the closing frame carries, absent for none

 @param usage - counts the stream reports

 @returns Stream text

 @example
 ```ts
 const body = claudeStreamOf({ text: '{"spot":"sunbeam"}', stopReason: 'end_turn', usage: PLAIN_USAGE, },);
 ```
 */
export function claudeStreamOf(
  {
    text,
    stopReason,
    stopDetails,
    usage,
  }: {
    readonly text: string;
    readonly stopReason: string;
    readonly stopDetails?: Readonly<Record<string, unknown>>;
    readonly usage: ClaudeStreamUsage;
  },
): string {
  return [
    anthropicFrameOf({
      body: {
        type: 'message_start',
        message: {
          id: 'msg_cat',
          type: 'message',
          role: 'assistant',
          content: [],
          model: 'claude-haiku-5-5',
          stop_reason: null,
          usage: {
            input_tokens: usage.input,
            cache_creation_input_tokens: usage.cacheWrite,
            cache_read_input_tokens: usage.cacheRead,
            output_tokens: 1,
          },
        },
      },
    },),
    anthropicBlockStart({
      index: 0,
      type: 'thinking',
    },),
    anthropicBlockDelta({
      index: 0,
      deltaType: 'thinking_delta',
      field: 'thinking',
      text: 'The cat naps.',
    },),
    anthropicFrameOf({ body: {
      type: 'content_block_stop',
      index: 0,
    }, },),
    (text === '')
      ? ''
      : [
        anthropicBlockStart({
          index: 1,
          type: 'text',
        },),
        anthropicBlockDelta({
          index: 1,
          deltaType: 'text_delta',
          field: 'text',
          text,
        },),
        anthropicFrameOf({ body: {
          type: 'content_block_stop',
          index: 1,
        }, },),
      ].join('',),
    anthropicFrameOf({
      body: {
        type: 'message_delta',
        delta: {
          stop_reason: stopReason,
          stop_sequence: null,
          ...((stopDetails === undefined) ? {} : { stop_details: stopDetails, }),
        },
        usage: { output_tokens: usage.output, },
      },
    },),
    anthropicFrameOf({ body: { type: 'message_stop', }, },),
  ].join('',);
}

/**
 Key every case here sends. Invented.
 */
export const CLAUDE_CAT_KEY = 'sk-ant-cat-key-whisker-9f3c7a-Qx4d';

/**
 The quota read of a Claude client, which has no quota view, so it rejects.
 */
function quotasUnused(): Promise<never> {
  return Promise.reject(new Error('quotas unused',),);
}

/**
 A model client over a Claude client that sends through the given transport, for the cases that ask a stage or a probe
 about a Claude seat.

 @param transport - transport the Claude client sends through

 @returns Client whose `quotas` throws, since a Claude client has no quota view and the callers here read none

 @example
 ```ts
 const client = claudeModelClientOver({ transport, },);
 ```
 */
export function claudeModelClientOver({ transport, }: { readonly transport: ModelTransport; },): ModelClient {
  /**
   Claude client whose calls the case reads.
   */
  const {
    chatText,
    chatJson,
  } = createClaudeClient({
    apiKey: CLAUDE_CAT_KEY,
    transport,
  },);
  return {
    chatText,
    chatJson,
    quotas: quotasUnused,
  };
}

/**
 A success reply that carries response headers, typed as the status-and-body pair the shared script helper takes so
 the extra member passes through it unchanged.

 @param bodyText - the whole stream

 @param headers - response headers by lower-case name

 @returns The reply

 @example
 ```ts
 const reply = headedReply({ bodyText, headers: { 'anthropic-ratelimit-requests-limit': '1000', }, },);
 ```
 */
export function headedReply(
  {
    bodyText,
    headers,
  }: {
    readonly bodyText: string;
    readonly headers: Readonly<Record<string, string>>;
  },
): {
  readonly status: number;
  readonly bodyText: string;
} {
  /**
   The reply as the transport seam types it.
   */
  const reply: TransportReply = {
    status: 200,
    bodyText,
    headers,
  };
  return reply;
}
