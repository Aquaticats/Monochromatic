/**
 Unit tests for Advisor provider client wiring.
 
 @module
 */

import {
  type Api,
  type AssistantMessage,
  type Context,
  fauxAssistantMessage,
  fauxProvider,
  type Model,
  type SimpleStreamOptions,
  type ThinkingLevel,
  type Usage,
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

/** Fixture model context window. */
const CONTEXT_WINDOW = 1_000;

/** Fixture max output tokens. */
const MAX_TOKENS = 100;

/** Fixture provider timeout. */
const TIMEOUT_MS = 1_000;

/** Fixture Advisor output token budget. */
const ADVISOR_OUTPUT_TOKENS = 100;

/** Advertised endpoint capacity below the configured request budget. */
const LIMITED_MAX_TOKENS = ADVISOR_OUTPUT_TOKENS - 1;

/** Focused question fixture. */
const FOCUS_QUESTION = 'Which assumption is weakest?';

/** Provider call count expected after one no-text retry. */
const RETRY_PROVIDER_CALL_COUNT = 2;

/**
 Build Advisor model fixture with selected reasoning capabilities.
 
 @param overrides - fixture identity and reasoning overrides
 
 @returns complete Advisor model fixture
 */
function createFixtureModel(
  overrides: Readonly<{
    id: string;
    reasoning: boolean;
    thinkingLevelMap?: Model<Api>['thinkingLevelMap'];
    maxTokens?: number;
  }>,
): Model<Api> {
  return {
    id: overrides.id,
    name: 'Reviewer',
    api: 'faux',
    provider: 'faux-provider',
    baseUrl: 'https://example.invalid',
    reasoning: overrides.reasoning,
    ...(overrides.thinkingLevelMap
      === undefined ? {} : { thinkingLevelMap: overrides.thinkingLevelMap, }),
    input: ['text',],
    cost: {
      input: 1,
      output: 1,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: CONTEXT_WINDOW,
    maxTokens: overrides.maxTokens
      ?? MAX_TOKENS,
  };
}

/** Fixture reasoning cases and highest expected simple-API effort. */
const REASONING_CASES: readonly {
  readonly name: string;
  readonly model: Model<Api>;
  readonly expectedReasoning?: ThinkingLevel;
}[] = [
  {
    name: 'omits reasoning for non-reasoning model',
    model: createFixtureModel({
      id: 'plain-reviewer',
      reasoning: false,
    },),
  },
  {
    name: 'selects high for standard reasoning model',
    model: createFixtureModel({
      id: 'standard-reviewer',
      reasoning: true,
    },),
    expectedReasoning: 'high',
  },
  {
    name: 'selects xhigh when model supports xhigh but not max',
    model: createFixtureModel({
      id: 'xhigh-reviewer',
      reasoning: true,
      thinkingLevelMap: {
        xhigh: 'xhigh',
      },
    },),
    expectedReasoning: 'xhigh',
  },
  {
    name: 'caps max-capable model at xhigh',
    model: createFixtureModel({
      id: 'max-reviewer',
      reasoning: true,
      thinkingLevelMap: {
        xhigh: 'xhigh',
        max: 'max',
      },
    },),
    expectedReasoning: 'xhigh',
  },
  {
    name: 'uses high when max is supported without xhigh',
    model: createFixtureModel({
      id: 'max-only-reviewer',
      reasoning: true,
      thinkingLevelMap: {
        xhigh: null,
        max: 'max',
      },
    },),
    expectedReasoning: 'high',
  },
];

//endregion Constants

//region Fixtures

/** Fixture usage metadata returned by fake provider. */
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

/** Fixture Advisor config. */
const advisorConfig: AdvisorConfig = {
  ...DEFAULT_CONFIG,
  timeoutMs: TIMEOUT_MS,
  maxAdvisorOutputTokens: ADVISOR_OUTPUT_TOKENS,
  source: {
    globalPath: '/home/test/.pi/agent/extensions/pi-advisor.json',
    projectPath: '/repo/.pi/extensions/pi-advisor.json',
    globalLoaded: false,
    projectLoaded: false,
  },
};

/** Fixture Advisor context. */
const advisorContext: AdvisorContext = {
  text: 'serialized evidence',
  maxContextChars: 1_000,
  originalChars: 19,
  finalChars: 19,
  truncated: false,
  includedMessageCount: 1,
  estimatedInputTokens: 10,
};

/** Fixture Advisor model. */
const fixtureModel: Model<Api> = createFixtureModel({
  id: 'reviewer',
  reasoning: false,
},);

/** Fixture assistant message returned by fake provider. */
const assistantMessage: AssistantMessage = {
  role: 'assistant',
  content: [{
    type: 'text',
    text: 'advisor answer',
  },],
  api: 'faux',
  provider: 'faux-provider',
  model: 'reviewer',
  usage,
  stopReason: 'stop',
  timestamp: 0,
};

/** Fixture assistant message without text content. */
const emptyAssistantMessage: AssistantMessage = {
  ...assistantMessage,
  content: [],
};

/** Fixture model-resolution port with auth lookup. */
const modelResolution = {
  async getApiKeyAndHeaders() {
    return {
      ok: true,
      apiKey: 'test-key',
    };
  },
} as unknown as AdvisorModelResolution;

//endregion Fixtures

//region Helpers

/**
 Build fake complete implementation capturing provider contexts.
 
 @param contexts - mutable capture sink for provider contexts
 
 @returns fake complete implementation
 */
function createCapturingCompleteModel(
  {
    contexts,
  }: {
    readonly contexts: Readonly<Context>[];
  },
): CompleteAdvisorModel {
  return async function completeModel(
    {
      model,
      context,
    },
  ) {
    void model;
    contexts.push(context,);
    return assistantMessage;
  };
}

/**
 Build fake complete implementation returning responses in call order.
 
 @param contexts - mutable capture sink for provider contexts
 
 @param responses - provider responses returned in order
 
 @returns fake complete implementation
 */
function createSequencedCompleteModel(
  {
    contexts,
    responses,
    options,
  }: {
    readonly contexts: Readonly<Context>[];
    readonly responses: readonly AssistantMessage[];
    readonly options?: Readonly<SimpleStreamOptions>[];
  },
): CompleteAdvisorModel {
  /**
   Queue of provider responses not yet returned.
   */
  const remainingResponses = [...responses,];
  return async function completeModel(
    {
      model,
      context,
      providerOptions,
    },
  ) {
    void model;
    contexts.push(context,);
    if (options !== undefined) {
      if (providerOptions === undefined)
        throw new Error('provider options were not supplied',);
      options.push(providerOptions,);
    }
    /**
     Response selected for this provider invocation.
     */
    const response = remainingResponses.shift();
    if (response === undefined)
      throw new Error('fake complete was called too many times',);
    return response;
  };
}

//endregion Helpers

await describe({
  name: completeAdvisor.name,
  children: [
    it({
      name: 'clamps the request output tokens to the advertised endpoint capacity',
      fn: async function testProviderBoundaryClampsOutputCapacity() {
        /** Provider contexts recorded by the completion seam. */
        const contexts: Readonly<Context>[] = [];
        /** Provider stream options recorded by the completion seam. */
        const observedOptions: Readonly<SimpleStreamOptions>[] = [];
        /** Fake provider completion recording context and stream options. */
        const completeModel = createSequencedCompleteModel({
          contexts,
          responses: [assistantMessage,],
          options: observedOptions,
        },);
        /** Model whose endpoint advertises less than the configured budget. */
        const limitedModel = createFixtureModel({
          id: 'limited-reviewer',
          reasoning: false,
          maxTokens: LIMITED_MAX_TOKENS,
        },);
        /** Completion against the capacity-limited route. */
        await completeAdvisor({
          ctx: modelResolution,
          model: limitedModel,
          config: advisorConfig,
          advisorContext,
          completeModel,
        },);

        expect(contexts,).toHaveLength(1,);
        expect(observedOptions[0]?.maxTokens,).toBe(LIMITED_MAX_TOKENS,);
      },
    },),
    it({
      name: 'requests the configured budget when the route advertises more output',
      fn: async function testProviderBoundaryKeepsBudgetWithinCapacity() {
        /** Provider contexts recorded by the completion seam. */
        const contexts: Readonly<Context>[] = [];
        /** Provider stream options recorded by the completion seam. */
        const observedOptions: Readonly<SimpleStreamOptions>[] = [];
        /** Fake provider completion recording context and stream options. */
        const completeModel = createSequencedCompleteModel({
          contexts,
          responses: [assistantMessage,],
          options: observedOptions,
        },);
        /** Model whose endpoint advertises more than the configured budget. */
        const roomyModel = createFixtureModel({
          id: 'roomy-reviewer',
          reasoning: false,
        },);
        /** Completion against the roomy route. */
        await completeAdvisor({
          ctx: modelResolution,
          model: roomyModel,
          config: advisorConfig,
          advisorContext,
          completeModel,
        },);

        expect(contexts,).toHaveLength(1,);
        expect(observedOptions[0]?.maxTokens,).toBe(ADVISOR_OUTPUT_TOKENS,);
      },
    },),
    it({
      name: 'requests the configured budget when the route advertises zero capacity',
      fn: async function testProviderBoundaryZeroCapacityUsesBudget() {
        /** Provider contexts recorded by the completion seam. */
        const contexts: Readonly<Context>[] = [];
        /** Provider stream options recorded by the completion seam. */
        const observedOptions: Readonly<SimpleStreamOptions>[] = [];
        /** Fake provider completion recording context and stream options. */
        const completeModel = createSequencedCompleteModel({
          contexts,
          responses: [assistantMessage,],
          options: observedOptions,
        },);
        /** Model whose endpoint advertises a non-positive capacity. */
        const zeroCapacityModel = createFixtureModel({
          id: 'zero-reviewer',
          reasoning: false,
          maxTokens: 0,
        },);
        /** Completion against the zero-capacity route. */
        await completeAdvisor({
          ctx: modelResolution,
          model: zeroCapacityModel,
          config: advisorConfig,
          advisorContext,
          completeModel,
        },);

        expect(contexts,).toHaveLength(1,);
        expect(observedOptions[0]?.maxTokens,).toBe(ADVISOR_OUTPUT_TOKENS,);
      },
    },),
    it({
      name: 'dispatches highest reasoning through registered custom provider',
      fn: async function testRegisteredCustomProvider() {
        /** Custom provider whose API is not part of pi built-ins. */
        const customProvider = fauxProvider({
          api: 'advisor-custom-api',
          provider: 'advisor-custom-provider',
          models: [{
            id: 'custom-reviewer',
            reasoning: true,
          },],
        },);
        /** Highest reasoning captured by registered provider implementation. */
        const capturedReasoning: ThinkingLevel[] = [];
        customProvider.setResponses([
          /*
           * Capture provider options before returning fixture response.
           */
          function customProviderResponse(context, streamOptions,) {
            void context;
            /** Simple provider options received through registered provider seam. */
            const simpleOptions: SimpleStreamOptions = streamOptions
              ?? {};
            /** Highest reasoning supplied to custom provider. */
            const providerReasoning = simpleOptions.reasoning;
            if (providerReasoning === undefined)
              throw new Error('registered provider did not receive reasoning',);
            capturedReasoning[capturedReasoning.length] = providerReasoning;
            return fauxAssistantMessage('registered provider response',);
          },
        ],);
        /** Custom provider model selected for Advisor. */
        const customModel = customProvider.getModel();
        /** Model-resolution port exposing registered custom provider. */
        const customModelResolution = {
          async getApiKeyAndHeaders() {
            return {
              ok: true,
              apiKey: 'test-key',
            };
          },
          getProvider(providerId: string,) {
            expect(providerId,).toBe('advisor-custom-provider',);
            return customProvider.provider;
          },
        } as unknown as AdvisorModelResolution;

        await completeAdvisor({
          ctx: customModelResolution,
          model: customModel,
          config: advisorConfig,
          advisorContext,
        },);

        expect(customProvider.state.callCount,).toBe(1,);
        expect(capturedReasoning,).toEqual(['high',],);
      },
    },),
    ...REASONING_CASES.map(function mapReasoningCase(reasoningCase,) {
      return it({
        name: reasoningCase.name,
        fn: async function testHighestSupportedReasoning() {
          /** Captured provider contexts. */
          const contexts: Readonly<Context>[] = [];
          /** Captured simple provider options. */
          const options: Readonly<SimpleStreamOptions>[] = [];
          /** Fake complete implementation capturing provider options. */
          const completeModel = createSequencedCompleteModel({
            contexts,
            responses: [assistantMessage,],
            options,
          },);

          await completeAdvisor({
            ctx: modelResolution,
            model: reasoningCase.model,
            config: advisorConfig,
            advisorContext,
            completeModel,
          },);

          const [capturedOptions,] = options;
          if (capturedOptions === undefined)
            throw new Error('provider options were not captured',);
          expect(capturedOptions.reasoning,).toBe(
            reasoningCase.expectedReasoning,
          );
        },
      },);
    },),
    it({
      name: 'passes focused question into provider user message',
      fn: async function testFocusedQuestionProviderMessage() {
        /** Captured provider contexts. */
        const contexts: Readonly<Context>[] = [];
        /** Fake complete implementation capturing context. */
        const completeModel = createCapturingCompleteModel({ contexts, });

        await completeAdvisor({
          ctx: modelResolution,
          model: fixtureModel,
          config: advisorConfig,
          advisorContext,
          question: FOCUS_QUESTION,
          completeModel,
        },);

        const [capturedContext,] = contexts;
        if (capturedContext === undefined)
          throw new Error('fake complete was not called',);
        const [message,] = capturedContext.messages;
        if (message?.role !== 'user')
          throw new Error('captured provider message is not user message',);
        if (!Array.isArray(message.content))
          throw new Error('captured provider message has string content',);
        const [firstBlock,] = message.content;
        if (firstBlock?.type !== 'text')
          throw new Error('captured provider content is not text',);

        expect(firstBlock.text,).toContain('## Focus question',);
        expect(firstBlock.text,).toContain(FOCUS_QUESTION,);
        expect(firstBlock.text,).toContain('## Serialized conversation',);
        expect(firstBlock.text,).toContain(advisorContext.text,);
      },
    },),
    it({
      name: 'retries once when provider returns no text',
      fn: async function testNoTextRetry() {
        /** Captured provider contexts. */
        const contexts: Readonly<Context>[] = [];
        /** Captured provider options. */
        const options: Readonly<SimpleStreamOptions>[] = [];
        /** Fake complete implementation returning no text then text. */
        const completeModel = createSequencedCompleteModel({
          contexts,
          responses: [
            emptyAssistantMessage,
            assistantMessage,
          ],
          options,
        },);

        const result = await completeAdvisor({
          ctx: modelResolution,
          model: fixtureModel,
          config: advisorConfig,
          advisorContext,
          completeModel,
        },);

        expect(result,).toEqual(assistantMessage,);
        expect(contexts.length,).toBe(RETRY_PROVIDER_CALL_COUNT,);
        expect(options.length,).toBe(RETRY_PROVIDER_CALL_COUNT,);
        const [firstOptions, retryOptions,] = options;
        if ((firstOptions === undefined) || (retryOptions === undefined))
          throw new Error('provider options were not captured',);
        expect(firstOptions,).not.toBe(retryOptions,);
        expect(firstOptions.signal,).toBe(retryOptions.signal,);
        /** First attempt timeout under shared deadline. */
        const firstTimeoutMs = firstOptions.timeoutMs;
        /** Retry timeout under shared deadline. */
        const retryTimeoutMs = retryOptions.timeoutMs;
        if ((firstTimeoutMs === undefined) || (retryTimeoutMs === undefined))
          throw new Error('provider timeout was not captured',);
        expect(retryTimeoutMs,).toBeLessThanOrEqual(firstTimeoutMs,);
      },
    },),
  ],
},);
