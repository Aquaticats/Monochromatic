/**
 Tests for the provider's own stream path when this provider answers with its
 passing token-rate limit:
 the message pi settles on has to be one pi's own automatic retry takes,
 and every other answer has to reach pi exactly as the provider sent it.

 Each test drives `provider.stream` or `provider.streamSimple` with pi's
 injectable transport, so what is asserted is the message pi would settle on
 rather than a translation read in isolation.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  isRetryableAssistantError,
  normalizeContext,
  type ApiKeyAuth,
  type AssistantMessage,
  type AssistantMessageEvent,
  type OpenAICompletionsOptions,
} from '@earendil-works/pi-ai';
import {
  QWEN_TOKEN_PLAN_DEFAULT_CONTEXT_WINDOW,
  QWEN_TOKEN_PLAN_DEFAULT_MAX_TOKENS,
  type QwenTokenPlanResolvedModelFacts,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';
import type {
  Logger,
} from '@monochromatic-dev/logger-core/ts';
import {
  createQwenTokenPlanIndividualProvider,
  projectQwenTokenPlanIndividualModel,
  QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_RETRIED_ERROR_MESSAGE,
  type QwenTokenPlanIndividualPiModel,
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
 Api-key auth stub; these tests hand the key in the request options.
 */
function fixtureApiKeyAuth(): NonNullable<ApiKeyAuth> {
  return {
    name: 'fixture key',
    resolve: function resolve(): Promise<undefined> {
      return Promise.resolve(undefined);
    },
  };
}

/**
 The message fifteen review runs ended on 2026-11-13,
 with its request id replaced by a placeholder.
 */
const RECORDED_PASSING_LIMIT_BODY = {
  message:
    'Allocated quota exceeded, please increase your quota limit. For details, see: https://www.alibabacloud.com/help/en/model-studio/error-code#token-limit',
  id: '00000000-0000-4000-8000-000000000000',
  type: 'insufficient_quota',
  code: 'insufficient_quota',
};

/**
 The model every request in this file names.
 */
const FIXTURE_FACTS: QwenTokenPlanResolvedModelFacts = {
  id: 'qwen4-max-preview',
  name: 'Qwen4 Max Preview',
  kind: 'chat',
  contextWindow: QWEN_TOKEN_PLAN_DEFAULT_CONTEXT_WINDOW,
  maxTokens: QWEN_TOKEN_PLAN_DEFAULT_MAX_TOKENS,
  input: ['text',],
  reasoning: true,
  compat: {
    thinkingFormat: 'qwen',
    supportsDeveloperRole: false,
    supportsStore: false,
    supportsStrictMode: true,
    supportsReasoningEffort: false,
  },
};

/**
 One provider answer that failed, as the transport delivers it.

 @param status - status the provider answered with

 @param body - body the provider sent

 @returns the response pi's transport hands its SDK
 */
function failedAnswerResponse({
  status,
  body,
}: {
  readonly status: number;
  readonly body: unknown
},): Response {
  return Response.json(
    body,
    {
      status,
      headers: { 'content-type': 'application/json', },
    },
  );
}

/**
 Build a completed chat-completions SSE response the stream can drain.

 @param text - reply the answer carries

 @returns a finite SSE response with one text delta and one stop chunk
 */
function replyResponse({
  text,
}: {
  readonly text: string
},): Response {
  /**
   Chunk lines the response body carries.
   */
  const chunkLines = [
    JSON.stringify({
      choices: [{
        index: 0,
        delta: {
          role: 'assistant',
          content: text,
        },
        finish_reason: null,
      },],
    },),
    JSON.stringify({
      choices: [{
        index: 0,
        delta: {},
        finish_reason: 'stop',
      },],
    },),
  ].map(function toChunkLine(payload,) {
    return `data: ${payload}\n\n`;
  },);
  return new Response(
    `${chunkLines.join('',)}data: [DONE]\n\n`,
    {
      status: 200,
      headers: { 'content-type': 'text/event-stream', },
    },
  );
}

/**
 Drain one provider stream.

 @param model - projected model under test

 @param options - dispatch options this answer varies

 @param simple - whether to drive `streamSimple` instead of `stream`

 @returns the message the stream settled on
 */
async function settledMessageOf({
  model,
  fetch,
  simple = false,
}: {
  readonly model: QwenTokenPlanIndividualPiModel;
  readonly fetch: OpenAICompletionsOptions['fetch'];
  readonly simple?: boolean
},): Promise<AssistantMessage> {
  const { provider, } = createQwenTokenPlanIndividualProvider({
    logger: fixtureLogger(),
    apiKeyAuth: fixtureApiKeyAuth(),
  },);
  /**
   Transcript every request in this file asks about.
   */
  const context = normalizeContext({
    systemPrompt: 'You are a fixture assistant.',
    messages: [{
      role: 'user',
      content: 'Fixture question.',
      timestamp: Date.now(),
    },],
  },);
  /**
   Options every request in this file carries beside its transport.
   */
  const requestOptions = {
    apiKey: 'fixture-key',
    signal: new AbortController().signal,
    ...(fetch === undefined ? {} : { fetch, }),
  };
  /**
   Stream the provider hands pi.
   */
  const events = simple
    ? provider.streamSimple(
      model,
      context,
      requestOptions,
    )
    : provider.stream(
      model,
      context,
      requestOptions,
    );
  /**
   Events drained so the stream settles before its message is read.
   */
  const drained: AssistantMessageEvent[] = [];
  for await (const event of events)
    drained.push(event,);
  return await events.result();
}

/**
 The projected model every request in this file names.

 @returns the model the provider serves for these tests
 */
function fixtureModel(): QwenTokenPlanIndividualPiModel {
  return projectQwenTokenPlanIndividualModel({ facts: FIXTURE_FACTS, },);
}

//endregion Fixtures

//region The provider's own stream path

await describe({
  name: 'provider stream of a passing token-rate limit',
  children: [
    it({
      name: 'settles on an answer pi retries, and the retry reaches the reply',
      fn: async () => {
        /**
         Answers the transport gives, in order.
         */
        const answers: Response[] = [
          failedAnswerResponse({
            status: 429,
            body: RECORDED_PASSING_LIMIT_BODY,
          },),
          replyResponse({ text: 'Fixture reply.', }),
        ];
        /**
         Transport one attempt walks, taking the next answer each time.
         */
        function nextAnswer(): Promise<Response> {
          const answer = answers.shift();
          if (answer === undefined)
            throw new Error('the fixture transport was asked for more answers than it holds',);
          return Promise.resolve(answer,);
        }
        /**
         What pi settles on for the first attempt.
         */
        const first = await settledMessageOf({
          model: fixtureModel(),
          fetch: function takeNextAnswer(): Promise<Response> {
            return nextAnswer();
          },
        },);
        expect(first.stopReason,).toBe('error',);
        expect(isRetryableAssistantError(first,),).toBe(true,);
        expect(first.errorMessage,).toBe(
          QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_RETRIED_ERROR_MESSAGE,
        );
        expect(first.diagnostics?.[0]?.error?.message,).toBe(
          `429: ${JSON.stringify(RECORDED_PASSING_LIMIT_BODY,)}`,
        );
        /**
         What pi settles on for the second attempt.
         */
        const second = await settledMessageOf({
          model: fixtureModel(),
          fetch: function takeNextAnswer(): Promise<Response> {
            return nextAnswer();
          },
        },);
        expect(second.stopReason,).toBe('stop',);
        expect(second.content,).toEqual([{ type: 'text', text: 'Fixture reply.', },],);
      },
    },),
    it({
      name: 'streamSimple settles on the same retried answer',
      fn: async () => {
        /**
         What pi settles on for a simple stream.
         */
        const message = await settledMessageOf({
          model: fixtureModel(),
          simple: true,
          fetch: function answerWithThePassingLimit(): Promise<Response> {
            return Promise.resolve(failedAnswerResponse({
              status: 429,
              body: RECORDED_PASSING_LIMIT_BODY,
            },),);
          },
        },);
        expect(isRetryableAssistantError(message,),).toBe(true,);
        expect(message.errorMessage,).toBe(
          QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_RETRIED_ERROR_MESSAGE,
        );
      },
    },),
    it({
      name: 'leaves every answer that must stay final as the provider sent it',
      fn: async () => {
        const answers = [
          {
            status: 401,
            body: {
              error: {
                message: 'Invalid API-key provided.',
                type: 'invalid_api_key',
              },
            },
            expected: `401: {"message":"Invalid API-key provided.","type":"invalid_api_key"}`,
          },
          {
            status: 402,
            body: {
              message: 'Payment required.',
              type: 'insufficient_quota',
              code: 'insufficient_quota',
            },
            expected: `402: {"message":"Payment required.","type":"insufficient_quota","code":"insufficient_quota"}`,
          },
          {
            status: 429,
            body: {
              message: 'Free allocated quota exceeded.',
              type: 'insufficient_quota',
              code: 'insufficient_quota',
            },
            expected: `429: {"message":"Free allocated quota exceeded.","type":"insufficient_quota","code":"insufficient_quota"}`,
          },
          {
            status: 429,
            body: {
              message: 'Too many requests.',
              type: 'Throttling.RateQuota',
              code: 'Throttling.RateQuota',
            },
            expected: `429: {"message":"Too many requests.","type":"Throttling.RateQuota","code":"Throttling.RateQuota"}`,
          },
        ];
        await Promise.all(answers.map(async function settlesAsSent(answer,) {
          /**
           What pi settles on for this answer.
           */
          const message = await settledMessageOf({
            model: fixtureModel(),
            fetch: function answerWithOneFinalAnswer(): Promise<Response> {
              return Promise.resolve(failedAnswerResponse(answer,),);
            },
          },);
          expect(message.errorMessage,).toBe(answer.expected,);
          expect(message.diagnostics,).toBe(undefined,);
        },),);
      },
    },),
  ],
},);

//endregion The provider's own stream path
