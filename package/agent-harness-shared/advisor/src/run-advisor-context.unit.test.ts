/**
 Run-level context-boundary tests for Advisor.
 
 @module
 */

import {
  fauxAssistantMessage,
  fauxProvider,
  getCurrentSystemPrompt,
  type Context,
  type SimpleStreamOptions,
} from '@earendil-works/pi-ai';
import type { SessionEntry, } from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type AdvisorConfig,
  type AdvisorDefaultRanking,
  type AdvisorRunContext,
  DEFAULT_CONFIG,
  type RankAdvisorDefaultCandidates,
  runAdvisor,
} from '../dist/final/node/index.mjs';
import { testAdvisorTranscriptSerialization, } from './transcript-test-fixture.ts';

//region Fixtures

/** Retained user message passed through compaction-aware context. */
const retainedEntry: SessionEntry = {
  type: 'message',
  id: 'retained-entry',
  parentId: null,
  timestamp: '2026-07-31T00:00:00.000Z',
  message: {
    role: 'user',
    content: 'retained task evidence',
    timestamp: 0,
  },
};

/** Stale user message exposed only by full branch fixture. */
const staleEntry: SessionEntry = {
  type: 'message',
  id: 'stale-entry',
  parentId: null,
  timestamp: '2026-07-30T00:00:00.000Z',
  message: {
    role: 'user',
    content: 'stale pre-compaction evidence',
    timestamp: 0,
  },
};

/** Canonical loaded project context that must survive compaction-aware selection. */
const PROJECT_CONTEXT_FIXTURE =
  `[{"content":"PX3: Act on authorized repository work.\\n","path":"/repo/AGENTS.md"}]`;

/** Advisor config fixture. */
const advisorConfig: AdvisorConfig = {
  ...DEFAULT_CONFIG,
  timeoutMs: 1_000,
  maxAdvisorOutputTokens: 100,
  source: {
    globalPath: '/home/test/.pi/agent/extensions/pi-advisor.json',
    projectPath: '/repo/.pi/extensions/pi-advisor.json',
    globalLoaded: false,
    projectLoaded: false,
  },
};

/**
 Extract text from provider user message.
 
 @param context - captured provider context
 
 @returns joined user text blocks
 */
function providerUserText(context: Readonly<Context>,): string {
  /** First user message containing serialized Advisor request, after Pi AI 0.87 leading system message. */
  const message = context.messages.find(function isUserMessage(candidate,) {
    return candidate.role === 'user';
  },);
  if ((message === undefined) || (message.role !== 'user'))
    throw new Error('provider user message missing',);
  if (!Array.isArray(message.content,))
    return message.content;
  return message.content
    .filter(function keepText(block,) {
      return block.type === 'text';
    },)
    .map(function textFromBlock(block,) {
      return block.type === 'text' ? block.text : '';
    },)
    .join('\n',);
}

//endregion Fixtures

await describe({
  name: runAdvisor.name,
  children: [
    it({
      name: 'clamps the request output budget to the route advertised capacity',
      fn: async function testOutputCapacityClampsRequest() {
        /** Faux provider whose endpoint advertises less than the configured budget. */
        const providerFixture = fauxProvider({
          api: 'faux',
          provider: 'limited-provider',
          models: [{
            id: 'limited-reviewer',
            reasoning: false,
            maxTokens: advisorConfig.maxAdvisorOutputTokens - 1,
          },],
        },);
        /** Stream options observed at dispatch, captured by the response factory. */
        const observedOptions: Readonly<SimpleStreamOptions>[] = [];
        providerFixture.setResponses([
          function recordOptions(
            _context,
            options,
          ) {
            if (options !== undefined)
              observedOptions.push(options,);
            return fauxAssistantMessage('clamped advisor answer',);
          },
        ],);
        /** Host run context whose only scoped model advertises less output than the budget. */
        const ctx = {
          cwd: '/repo',
          scopedModels: [providerFixture.getModel(),],
          modelRegistry: {
            async getApiKeyAndHeaders() {
              return {
                ok: true,
                apiKey: 'test-key',
              };
            },
            getProvider() {
              return providerFixture.provider;
            },
          },
          transcript: {
            ...testAdvisorTranscriptSerialization,
            buildContextEntries() {
              return [];
            },
          },
        } as unknown as AdvisorRunContext;
        /** Operation completing against the clamped request. */
        const result = await runAdvisor({
          ctx,
          config: advisorConfig,
        },);

        expect(result.text,).toContain('clamped advisor answer',);
        expect(providerFixture.state.callCount,).toBe(1,);
        expect(observedOptions[0]?.maxTokens,).toBe(advisorConfig.maxAdvisorOutputTokens - 1,);
      },
    },),
    it({
      name: 'clamps the request for an explicit model below the configured budget',
      fn: async function testExplicitOutputCapacityClampsRequest() {
        /** Faux provider exposing full-capacity and limited endpoints in one scope. */
        const providerFixture = fauxProvider({
          api: 'faux',
          provider: 'mixed-provider',
          models: [
            {
              id: 'eligible-reviewer',
              reasoning: false,
              maxTokens: advisorConfig.maxAdvisorOutputTokens,
            },
            {
              id: 'limited-reviewer',
              reasoning: false,
              maxTokens: advisorConfig.maxAdvisorOutputTokens - 1,
            },
          ],
        },);
        /** Stream options observed at dispatch, captured by the response factory. */
        const observedOptions: Readonly<SimpleStreamOptions>[] = [];
        providerFixture.setResponses([
          function recordOptions(
            _context,
            options,
          ) {
            if (options !== undefined)
              observedOptions.push(options,);
            return fauxAssistantMessage('explicit clamped answer',);
          },
        ],);
        /** Host run context exposing both endpoints through live scope. */
        const ctx = {
          cwd: '/repo',
          scopedModels: providerFixture.models,
          modelRegistry: {
            getAll() {
              return providerFixture.models;
            },
            async getApiKeyAndHeaders() {
              return {
                ok: true,
                apiKey: 'test-key',
              };
            },
            getProvider() {
              return providerFixture.provider;
            },
          },
          transcript: {
            ...testAdvisorTranscriptSerialization,
            buildContextEntries() {
              return [];
            },
          },
        } as unknown as AdvisorRunContext;
        /** Explicit operation completing against the clamped request. */
        const result = await runAdvisor({
          ctx,
          config: advisorConfig,
          requestedSlug: 'mixed-provider/limited-reviewer',
        },);

        expect(result.text,).toContain('explicit clamped answer',);
        expect(providerFixture.state.callCount,).toBe(1,);
        expect(observedOptions[0]?.maxTokens,).toBe(advisorConfig.maxAdvisorOutputTokens - 1,);
      },
    },),
    it({
      name: 'uses compaction-aware session entries instead of full branch',
      fn: async function testCompactionAwareBoundary() {
        /** Faux provider and selected model. */
        const providerFixture = fauxProvider({
          api: 'faux',
          provider: 'faux-provider',
          models: [{
            id: 'reviewer',
            reasoning: false,
            maxTokens: advisorConfig.maxAdvisorOutputTokens,
          },],
        },);
        /** Provider contexts captured by response callback. */
        const providerContexts: Context[] = [];
        providerFixture.setResponses([
          function providerResponse(context,) {
            providerContexts.push(context,);
            return fauxAssistantMessage('advisor answer',);
          },
        ],);
        /** Compaction-aware context method calls. */
        const contextCalls: string[] = [];
        /** Full-branch method calls, which must remain empty. */
        const branchCalls: string[] = [];
        /** Host run context exposing both session methods. */
        const ctx = {
          cwd: '/repo',
          scopedModels: [providerFixture.getModel(),],
          modelRegistry: {
            getAll() {
              return providerFixture.models;
            },
            async getApiKeyAndHeaders() {
              return {
                ok: true,
                apiKey: 'test-key',
              };
            },
            getProvider() {
              return providerFixture.provider;
            },
          },
          transcript: {
            ...testAdvisorTranscriptSerialization,
            buildContextEntries() {
              contextCalls.push('buildContextEntries',);
              return [retainedEntry,];
            },
            getBranch() {
              branchCalls.push('getBranch',);
              return [staleEntry, retainedEntry,];
            },
          },
        } as unknown as AdvisorRunContext;

        await runAdvisor({
          ctx,
          config: advisorConfig,
          projectContext: PROJECT_CONTEXT_FIXTURE,
          requestedSlug: 'faux-provider/reviewer',
        },);

        expect(contextCalls,).toEqual(['buildContextEntries',],);
        expect(branchCalls,).toEqual([],);
        const [providerContext,] = providerContexts;
        if (providerContext === undefined)
          throw new Error('provider context not captured',);
        const requestText = providerUserText(providerContext,);
        expect(requestText,).toContain('retained task evidence',);
        expect(requestText,).not.toContain('stale pre-compaction evidence',);
        expect(getCurrentSystemPrompt(providerContext.messages,),).toContain('/repo/AGENTS.md',);
        expect(getCurrentSystemPrompt(providerContext.messages,),).toContain('PX3: Act on authorized repository work.',);
      },
    },),
    it({
      name: 'dispatches the policy-ranked model for default calls',
      fn: async function testPolicyRankedDispatch() {
        /** First scoped provider and model. */
        const firstProviderFixture = fauxProvider({
          api: 'faux',
          provider: 'first-provider',
          models: [{
            id: 'reviewer',
            reasoning: false,
            maxTokens: advisorConfig.maxAdvisorOutputTokens,
          },],
        },);
        /** Second scoped provider and model, ranked first by the policy. */
        const secondProviderFixture = fauxProvider({
          api: 'faux',
          provider: 'second-provider',
          models: [{
            id: 'reviewer',
            reasoning: false,
            maxTokens: advisorConfig.maxAdvisorOutputTokens,
          },],
        },);
        secondProviderFixture.setResponses([
          function secondProviderResponse() {
            return fauxAssistantMessage('advisor answer',);
          },
        ],);
        firstProviderFixture.setResponses([
          function firstProviderResponse() {
            return fauxAssistantMessage('advisor answer',);
          },
        ],);
        /** Host run context exposing both scoped models. */
        const ctx = {
          cwd: '/repo',
          scopedModels: [
            firstProviderFixture.getModel(),
            secondProviderFixture.getModel(),
          ],
          modelRegistry: {
            getAll() {
              return [
                ...firstProviderFixture.models,
                ...secondProviderFixture.models,
              ];
            },
            async getApiKeyAndHeaders() {
              return {
                ok: true,
                apiKey: 'test-key',
              };
            },
            getProvider(provider: string,) {
              return provider === 'first-provider'
                ? firstProviderFixture.provider
                : secondProviderFixture.provider;
            },
          },
          transcript: {
            ...testAdvisorTranscriptSerialization,
            buildContextEntries() {
              return [retainedEntry,];
            },
          },
        } as unknown as AdvisorRunContext;
        /**
         External ranking policy reversing the scope order.
         
         @param options - eligible candidates and requested output budget
         
         @returns policy ranking with the scope order reversed
         
         @example
         ```typescript
         reversedPolicy({ candidates: [], maxOutputTokens: 32 });
         ```
         */
        function reversedPolicy(
          {
            candidates,
            maxOutputTokens,
          }: Parameters<RankAdvisorDefaultCandidates>[0],
        ): AdvisorDefaultRanking {
          return {
            ranked: candidates
              .map(function mapScopedModel(candidate,) {
                return candidate.scopedModel;
              },)
              .toReversed(),
            reason: `reversed scope order for ${String(maxOutputTokens,)} output tokens`,
          };
        }
        const result = await runAdvisor({
          ctx,
          config: advisorConfig,
          rankDefaultCandidates: reversedPolicy,
        },);
        expect(result.details
          .selectedSlug,).toBe('second-provider/reviewer',);
      },
    },),
  ],
},);
