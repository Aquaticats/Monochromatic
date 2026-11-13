/**
 Tests for the wrapper that hands this provider's passing token-rate limit to pi
 as an answer pi's own automatic retry takes.

 Each case builds the stream the provider implementation would have produced,
 forwards it through the wrapper, and reads what a caller sees:
 every event as it arrives, and the message the stream settled on.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createAssistantMessageEventStream,
  isRetryableAssistantError,
  type Api,
  type AssistantMessage,
  type AssistantMessageEvent,
  type AssistantMessageEventStream,
  type Model,
} from '@earendil-works/pi-ai';
import type {
  Logger,
} from '@monochromatic-dev/logger-core/ts';
import {
  QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_RETRIED_ERROR_MESSAGE,
  streamWithPassingTokenRateLimitRetried,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Discards one log message.
 */
function discardLogMessage(): void {}

/**
 Silent logger these tests run under.
 */
function fixtureLogger(): Logger {
  return {
    debug: discardLogMessage,
    error: discardLogMessage,
    fatal: discardLogMessage,
    flush: function flush(): Promise<void> {
      return Promise.resolve();
    },
    info: discardLogMessage,
    trace: discardLogMessage,
    warn: discardLogMessage,
  };
}

/**
 The answer fifteen review runs ended on 2026-11-13, as pi wrote it,
 with its request id replaced by a placeholder.
 */
const RECORDED_PASSING_LIMIT_ANSWER = `429: {"message":"Allocated quota exceeded, please increase your quota limit. For details, see: https://www.alibabacloud.com/help/en/model-studio/error-code#token-limit","id":"00000000-0000-4000-8000-000000000000","type":"insufficient_quota","code":"insufficient_quota"}`;

/**
 The model the answers belong to, projected the way this package serves it.
 */
const FIXTURE_MODEL: Model<Api> = {
  id: 'qwen4-max-preview',
  name: 'Qwen4 Max Preview',
  api: 'openai-completions',
  provider: 'qwen-token-plan-individual',
  baseUrl: 'https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1',
  input: ['text',],
  cost: {
    input: 0,
    output: 0,
    cacheRead: 0,
    cacheWrite: 0,
  },
  contextWindow: 1_000_000,
  maxTokens: 393_216,
  reasoning: true,
};

/**
 One message a stream shows while it runs, and one it settles on.

 @param stopReason - the reason the message ended for

 @param errorMessage - text the message carries, for an error

 @param text - reply the message carries, for a reply

 @returns a complete assistant message
 */
function assistantMessageFixture({
  stopReason,
  errorMessage,
  text,
}: {
  readonly stopReason: AssistantMessage['stopReason'];
  readonly errorMessage?: string;
  readonly text?: string;
},): AssistantMessage {
  return {
    role: 'assistant',
    content: (text === undefined) ? [] : [{ type: 'text', text, },],
    api: 'openai-completions',
    provider: 'qwen-token-plan-individual',
    model: 'qwen4-max-preview',
    usage: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
      totalTokens: 0,
      cost: {
        input: 0,
        output: 0,
        cacheRead: 0,
        cacheWrite: 0,
        total: 0,
      },
    },
    stopReason,
    ...(errorMessage === undefined ? {} : { errorMessage, }),
    timestamp: Date.now(),
  };
}

/**
 The stream a provider implementation would have produced, filled with the
 events one answer consists of.

 @param events - events to push, in order

 @returns the stream the provider implementation hands on
 */
function providerStreamOf(
  events: readonly AssistantMessageEvent[],
): AssistantMessageEventStream {
  const inner = createAssistantMessageEventStream();
  for (const event of events)
    inner.push(event,);
  inner.end();
  return inner;
}

/**
 Forward one provider stream through the wrapper under test.

 @param inner - the provider's own stream

 @returns the stream the caller drains
 */
function forwardedStreamOf(
  inner: AssistantMessageEventStream,
): AssistantMessageEventStream {
  return streamWithPassingTokenRateLimitRetried({
    inner,
    model: FIXTURE_MODEL,
    logger: fixtureLogger(),
  },);
}

/**
 Drain one stream, so its events and its result can be read together.

 @param stream - stream to drain

 @returns every event the caller saw, in order
 */
async function drainedEventsOf(
  stream: AssistantMessageEventStream,
): Promise<AssistantMessageEvent[]> {
  const events: AssistantMessageEvent[] = [];
  for await (const event of stream)
    events.push(event,);
  return events;
}

/**
 The stream one answer of the provider's consists of.

 @param message - the message the answer settles on

 @returns the events pi's openai-completions stream pushes for it
 */
function answerEventsOf(message: AssistantMessage,): AssistantMessageEvent[] {
  return [
    {
      type: 'start',
      partial: message,
    },
    {
      type: 'error',
      reason: message.stopReason,
      error: message,
    },
  ];
}

//endregion Fixtures

//region Forwarding one passing limit

await describe({
  name: 'passing token-rate limit forwarding',
  children: [
    it({
      name: 'hands pi an answer its own retry takes, with the provider answer kept',
      fn: async () => {
        /**
         The error message the provider's stream settles on.
         */
        const message = assistantMessageFixture({
          stopReason: 'error',
          errorMessage: RECORDED_PASSING_LIMIT_ANSWER,
        },);
        const stream = forwardedStreamOf(providerStreamOf(answerEventsOf(message,),),);
        const events = await drainedEventsOf(stream,);
        expect(events.map(function typeOf(event,) {
          return event.type;
        },),).toEqual(['start', 'error',],);
        /**
         The terminal event the caller saw.
         */
        const terminal = events.at(-1,);
        if (terminal?.type !== 'error')
          throw new Error('the forwarded stream did not end on an error event',);
        expect(terminal.error.errorMessage,).toBe(
          QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_RETRIED_ERROR_MESSAGE,
        );
        expect(terminal.error.diagnostics?.[0]?.error?.message,).toBe(
          RECORDED_PASSING_LIMIT_ANSWER,
        );
        expect(isRetryableAssistantError(terminal.error,),).toBe(true,);
        expect(await stream.result(),).toBe(terminal.error,);
      },
    },),
    it({
      name: 'forwards every other event untouched',
      fn: async () => {
        /**
         Text the reply carries.
         */
        const reply = assistantMessageFixture({
          stopReason: 'stop',
          text: 'Fixture reply.',
        },);
        const stream = forwardedStreamOf(providerStreamOf([
          {
            type: 'start',
            partial: reply,
          },
          {
            type: 'text_delta',
            contentIndex: 0,
            delta: 'Fixture reply.',
            partial: reply,
          },
          {
            type: 'done',
            reason: 'stop',
            message: reply,
          },
        ],),);
        const events = await drainedEventsOf(stream,);
        expect(events.map(function typeOf(event,) {
          return event.type;
        },),).toEqual(['start', 'text_delta', 'done',],);
        // The guard this wrapper pushes after the provider's stream has ended
        // is dropped once an answer of the provider's arrived, so the reply
        // stands rather than being replaced by the guard.
        expect(await stream.result(),).toBe(reply,);
      },
    },),
    it({
      name: 'leaves an error that is not the passing limit untouched',
      fn: async () => {
        for (const errorMessage of [
          '401: {"error":{"message":"Invalid API-key provided.","type":"invalid_api_key"}}',
          '402: {"message":"Payment required."}',
          '429: {"message":"Free allocated quota exceeded.","type":"insufficient_quota","code":"insufficient_quota"}',
        ]) {
          /**
           The error message the provider's stream settles on.
           */
          const message = assistantMessageFixture({
            stopReason: 'error',
            errorMessage,
          },);
          const events = await drainedEventsOf(
            forwardedStreamOf(providerStreamOf(answerEventsOf(message,),),),
          );
          /**
           The terminal event the caller saw.
           */
          const terminal = events.at(-1,);
          if (terminal?.type !== 'error')
            throw new Error('the forwarded stream did not end on an error event',);
          expect(terminal.error,).toBe(message,);
          expect(terminal.error.diagnostics,).toBe(undefined,);
          expect(isRetryableAssistantError(terminal.error,),).toBe(false,);
        }
      },
    },),
  ],
},);

//endregion Forwarding one passing limit

//region A stream that reports nothing

await describe({
  name: 'passing token-rate limit forwarding of a stream that reports nothing',
  children: [
    it({
      name: 'reports an error rather than leaving the caller waiting',
      fn: async () => {
        /**
         A stream with a message shown and no answer of its own.
         */
        const inner = providerStreamOf([{
          type: 'start',
          partial: assistantMessageFixture({ stopReason: 'pending', },),
        },],);
        const events = await drainedEventsOf(forwardedStreamOf(inner,),);
        /**
         The terminal event the caller saw.
         */
        const terminal = events.at(-1,);
        if (terminal?.type !== 'error')
          throw new Error('a stream that reported nothing did not end on an error event',);
        expect(terminal.error.stopReason,).toBe('error',);
        expect(terminal.error.errorMessage,).toContain('ended before',);
        expect(terminal.error.model,).toBe(FIXTURE_MODEL.id,);
      },
    },),
    it({
      name: 'reports an error for a stream that showed nothing at all',
      fn: async () => {
        const events = await drainedEventsOf(forwardedStreamOf(providerStreamOf([],),),);
        expect(events,).toHaveLength(1,);
        /**
         The terminal event the caller saw.
         */
        const terminal = events.at(-1,);
        if (terminal?.type !== 'error')
          throw new Error('an empty stream did not end on an error event',);
        expect(terminal.error.content,).toEqual([],);
        expect(terminal.error.provider,).toBe(FIXTURE_MODEL.provider,);
      },
    },),
  ],
},);

//endregion A stream that reports nothing
