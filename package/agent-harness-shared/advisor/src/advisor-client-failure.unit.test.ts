/**
 Failure-state and deadline tests for Advisor provider completion.
 
 @module
 */

import type {
  Api,
  AssistantMessage,
  Model,
  SimpleStreamOptions,
  StopReason,
  Usage,
} from '@earendil-works/pi-ai';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type AdvisorConfig,
  type AdvisorContext,
  type AdvisorModelResolution,
  completeAdvisor,
  type CompleteAdvisorModel,
  DEFAULT_CONFIG,
} from '../dist/final/node/index.mjs';

//region Constants

/** Fixture provider timeout. */
const TIMEOUT_MS = 1_000;

/** Fixture output token budget. */
const OUTPUT_TOKENS = 100;

/** Fixture model context window. */
const CONTEXT_WINDOW = 1_000;

/** Expected provider attempts after one empty successful response. */
const RETRY_ATTEMPTS = 2;

//endregion Constants

//region Fixtures

/** Zeroed usage fixture. */
const usage: Usage = {
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
};

/** Advisor model fixture. */
const fixtureModel: Model<Api> = {
  id: 'reviewer',
  name: 'Reviewer',
  api: 'faux',
  provider: 'faux-provider',
  baseUrl: 'https://example.invalid',
  reasoning: false,
  input: ['text',],
  cost: {
    input: 1,
    output: 1,
    cacheRead: 0,
    cacheWrite: 0,
  },
  contextWindow: CONTEXT_WINDOW,
  maxTokens: OUTPUT_TOKENS,
};

/** Advisor context fixture. */
const advisorContext: AdvisorContext = {
  text: 'serialized evidence',
  maxContextChars: CONTEXT_WINDOW,
  originalChars: 19,
  finalChars: 19,
  truncated: false,
  includedMessageCount: 1,
  estimatedInputTokens: 10,
};

/** Model-resolution port fixture with successful auth. */
const modelResolution = {
  async getApiKeyAndHeaders() {
    return {
      ok: true,
      apiKey: 'test-key',
    };
  },
} as unknown as AdvisorModelResolution;

/**
 Build Advisor config with selected timeout.
 
 @param timeoutMs - total operation deadline
 
 @returns complete Advisor config fixture
 */
function advisorConfig(timeoutMs: number,): AdvisorConfig {
  return {
    ...DEFAULT_CONFIG,
    timeoutMs,
    maxAdvisorOutputTokens: OUTPUT_TOKENS,
    source: {
      globalPath: '/home/test/.pi/agent/extensions/pi-advisor.json',
      projectPath: '/repo/.pi/extensions/pi-advisor.json',
      globalLoaded: false,
      projectLoaded: false,
    },
  };
}

/**
 Build terminal provider response fixture.
 
 @param stopReason - provider terminal state
 
 @param text - optional user-visible text
 
 @param errorMessage - optional provider diagnostic
 
 @returns assistant response fixture
 */
function assistantResponse(
  {
    stopReason,
    text,
    errorMessage,
  }: {
    readonly stopReason: StopReason;
    readonly text?: string;
    readonly errorMessage?: string;
  },
): AssistantMessage {
  return {
    role: 'assistant',
    content: text === undefined
      ? []
      : [{
        type: 'text',
        text,
      },],
    api: 'faux',
    provider: fixtureModel.provider,
    model: fixtureModel.id,
    usage,
    stopReason,
    ...(errorMessage === undefined ? {} : { errorMessage, }),
    timestamp: 0,
  };
}

/**
 Capture error from asynchronous Advisor completion.
 
 @param action - completion expected to fail
 
 @returns caught Error instance
 */
async function captureError(
  action: () => Promise<unknown>,
): Promise<Error> {
  try {
    await action();
  }
  catch (error) {
    if (Error.isError(error,))
      return error;
    throw new Error(
      `expected Error, received ${String(error,)}`,
      { cause: error, },
    );
  }
  throw new Error('expected completion to fail',);
}

/**
 Complete fixture Advisor request through supplied provider seam.
 
 @param completeModel - provider completion seam
 
 @param timeoutMs - total operation deadline
 
 @param signal - optional caller cancellation signal
 
 @returns terminal successful response
 */
async function completeFixture(
  {
    completeModel,
    timeoutMs = TIMEOUT_MS,
    signal,
  }: {
    readonly completeModel: CompleteAdvisorModel;
    readonly timeoutMs?: number;
    readonly signal?: AbortSignal;
  },
): Promise<AssistantMessage> {
  return await completeAdvisor({
    ctx: modelResolution,
    model: fixtureModel,
    config: advisorConfig(timeoutMs,),
    advisorContext,
    completeModel,
    ...(signal === undefined ? {} : { signal, }),
  },);
}

//endregion Fixtures

await describe({
  name: completeAdvisor.name,
  children: [
    it({
      name: 'fails provider error once with original diagnostic',
      fn: async function testProviderErrorNoRetry() {
        /** Count of provider attempts. */
        const attempts: number[] = [];
        /**
         Provider seam returning terminal error.
         
         @returns terminal provider error
         */
        async function completeModel(): Promise<AssistantMessage> {
          attempts.push(1,);
          return assistantResponse({
            stopReason: 'error',
            errorMessage: 'fixture provider failure',
          },);
        }

        const error = await captureError(async function completeErrorResponse() {
          await completeFixture({ completeModel, });
        },);

        expect(attempts,).toHaveLength(1,);
        expect(error.message,).toContain('faux-provider/reviewer',);
        expect(error.message,).toContain('attempt 1',);
        expect(error.message,).toContain('fixture provider failure',);
      },
    },),
    it({
      name: 'classifies caller cancellation without retry',
      fn: async function testCallerCancellation() {
        /** Caller-owned abort controller. */
        const controller = new AbortController();
        /** Count of provider attempts. */
        const attempts: number[] = [];
        /**
         Provider seam aborting caller during request.
         
         @returns terminal aborted response
         */
        async function completeModel(): Promise<AssistantMessage> {
          attempts.push(1,);
          controller.abort();
          return assistantResponse({
            stopReason: 'aborted',
            errorMessage: 'request aborted',
          },);
        }

        const error = await captureError(async function completeCancelledResponse() {
          await completeFixture({
            completeModel,
            signal: controller.signal,
          },);
        },);

        expect(attempts,).toHaveLength(1,);
        expect(error.message,).toContain('call cancelled',);
        expect(error.message,).toContain('attempt 1',);
      },
    },),
    it({
      name: 'classifies shared deadline expiry without retry',
      fn: async function testDeadlineExpiry(testContext) {
        /** Test-local wall clock advances only after provider dispatch. */
        const clock = testContext.sinon.stub(Date, 'now',).returns(0,);
        /** Count of provider attempts. */
        const attempts: number[] = [];
        /**
         Provider seam waiting beyond configured deadline.
         
         @returns terminal aborted response after deadline
         */
        async function completeModel(): Promise<AssistantMessage> {
          attempts.push(1,);
          clock.returns(TIMEOUT_MS + 1,);
          return assistantResponse({
            stopReason: 'aborted',
            errorMessage: 'request aborted',
          },);
        }

        const error = await captureError(async function completeTimedOutResponse() {
          await completeFixture({
            completeModel,
            timeoutMs: TIMEOUT_MS,
          },);
        },);

        expect(attempts,).toHaveLength(1,);
        expect(error.message,).toContain(`timed out after ${TIMEOUT_MS}ms`,);
        expect(error.message,).toContain('attempt 1',);
      },
    },),
    it({
      name: 'classifies provider abort without retry',
      fn: async function testProviderAbort() {
        /** Count of provider attempts. */
        const attempts: number[] = [];
        /**
         Provider seam returning independent abort.
         
         @returns terminal provider abort
         */
        async function completeModel(): Promise<AssistantMessage> {
          attempts.push(1,);
          return assistantResponse({
            stopReason: 'aborted',
            errorMessage: 'upstream cancelled request',
          },);
        }

        const error = await captureError(async function completeAbortedResponse() {
          await completeFixture({ completeModel, });
        },);

        expect(attempts,).toHaveLength(1,);
        expect(error.message,).toContain('provider aborted',);
        expect(error.message,).toContain('upstream cancelled request',);
      },
    },),
    it({
      name: 'retries successful empty response under shared deadline',
      fn: async function testEmptyResponseRetry() {
        /** Provider options captured by attempt. */
        const capturedOptions: SimpleStreamOptions[] = [];
        /** Responses returned in attempt order. */
        const responses = [
          assistantResponse({ stopReason: 'stop', }),
          assistantResponse({ stopReason: 'stop', text: 'advisor answer', }),
        ];
        /**
         Provider seam returning queued responses.
         
         @param providerOptions - current provider attempt options
         
         @returns response at current attempt index
         */
        async function completeModel(
          { providerOptions, }: Parameters<CompleteAdvisorModel>[0],
        ): Promise<AssistantMessage> {
          if (providerOptions === undefined)
            throw new Error('provider options missing',);
          capturedOptions.push(providerOptions,);
          /**
           Response at current attempt index.
           */
          const response = responses.at(capturedOptions.length - 1,);
          if (response === undefined)
            throw new Error('provider called too many times',);
          return response;
        }

        const response = await completeFixture({ completeModel, });

        expect(response.content,).toEqual([{
          type: 'text',
          text: 'advisor answer',
        },],);
        expect(capturedOptions,).toHaveLength(RETRY_ATTEMPTS,);
        const [firstOptions, secondOptions,] = capturedOptions;
        if ((firstOptions === undefined) || (secondOptions === undefined))
          throw new Error('attempt options missing',);
        expect(firstOptions.signal,).toBe(secondOptions.signal,);
        /** First attempt timeout under shared deadline. */
        const firstTimeoutMs = firstOptions.timeoutMs;
        /** Second attempt timeout under shared deadline. */
        const secondTimeoutMs = secondOptions.timeoutMs;
        if ((firstTimeoutMs === undefined) || (secondTimeoutMs === undefined))
          throw new Error('attempt timeout missing',);
        expect(secondTimeoutMs,).toBeLessThanOrEqual(firstTimeoutMs,);
      },
    },),
    it({
      name: 'fails after two successful empty responses',
      fn: async function testEmptyResponseExhaustion() {
        /** Count of provider attempts. */
        const attempts: number[] = [];
        /**
         Provider seam returning successful empty response.
         
         @returns terminal empty response
         */
        async function completeModel(): Promise<AssistantMessage> {
          attempts.push(1,);
          return assistantResponse({ stopReason: 'stop', });
        }

        const error = await captureError(async function completeEmptyResponses() {
          await completeFixture({ completeModel, });
        },);

        expect(attempts,).toHaveLength(RETRY_ATTEMPTS,);
        expect(error.message,).toContain('returned no text after 2 attempts',);
      },
    },),
    it({
      name: 'rejects unexpected tool use without retry',
      fn: async function testUnexpectedToolUse() {
        /** Count of provider attempts. */
        const attempts: number[] = [];
        /**
         Provider seam returning tool-use terminal state.
         
         @returns terminal tool-use response
         */
        async function completeModel(): Promise<AssistantMessage> {
          attempts.push(1,);
          return assistantResponse({ stopReason: 'toolUse', });
        }

        const error = await captureError(async function completeToolUseResponse() {
          await completeFixture({ completeModel, });
        },);

        expect(attempts,).toHaveLength(1,);
        expect(error.message,).toContain('requested unavailable tool use',);
      },
    },),
    ...(['stop', 'length',] as const).map(function mapSuccessfulStopReason(
      stopReason,
    ) {
      return it({
        name: `returns text for ${stopReason} terminal response`,
        fn: async function testSuccessfulTextResponse() {
          /**
           Provider seam returning text.
           
           @returns successful terminal response
           */
          async function completeModel(): Promise<AssistantMessage> {
            return assistantResponse({
              stopReason,
              text: 'advisor answer',
            },);
          }

          const response = await completeFixture({ completeModel, });

          expect(response.stopReason,).toBe(stopReason,);
        },
      },);
    },),
  ],
},);
