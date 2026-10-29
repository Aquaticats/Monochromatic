/** Built Advisor dispatch, scope, and exact-model integration through real faux providers. @module */
import { fauxAssistantMessage, fauxProvider, } from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { AdvisorOperationError, type AdvisorRunContext, DEFAULT_CONFIG, runAdvisor, } from '../dist/final/node/index.mjs';
import { testAdvisorTranscriptSerialization, } from './transcript-test-fixture.ts';

/** Runtime config does not read the user's real home. */
const config = { ...DEFAULT_CONFIG, timeoutMs: 1_000, maxAdvisorOutputTokens: 100,
  source: { globalPath: '/fixture/global', projectPath: '/fixture/project', globalLoaded: false, projectLoaded: false }, };

/** Build provider and host fixtures with an authoritative getter-backed live scope. */
function hostFixture() {
  const first = fauxProvider({ provider: 'first', api: 'faux-first', models: [
    { id: 'a', maxTokens: 100, cost: { input: 10, output: 10, cacheRead: 0, cacheWrite: 0 }, },
    { id: 'b', maxTokens: 100, cost: { input: 9, output: 9, cacheRead: 0, cacheWrite: 0 }, },
  ], },);
  const other = fauxProvider({ provider: 'other', api: 'faux-other', models: [
    { id: 'c', maxTokens: 100, cost: { input: 1, output: 1, cacheRead: 0, cacheWrite: 0 }, },
  ], },);
  const all = [...first.models, ...other.models,];
  const state = { live: all as unknown, auth: (): void => {}, branchReads: 0, };
  const ctx = {
    cwd: '/fixture', getScopedModels: (): unknown => state.live,
    transcript: { ...testAdvisorTranscriptSerialization, buildContextEntries: () => { state.branchReads += 1;
      return []; }, },
    modelRegistry: {
      getAvailable: () => all,
      getProvider: (id: string) => id === 'first' ? first.provider : id === 'other' ? other.provider : undefined,
      getApiKeyAndHeaders: async () => { state.auth();
        return { ok: true, apiKey: 'fixture', }; },
      find: (provider: string, id: string) => all.find(model => (model.provider === provider) && (model.id === id)),
    },
  } as unknown as AdvisorRunContext;
  return { first, other, state, ctx, };
}

await describe({ name: '', children: [
  it({ name: 'default recovery blocks every exhausted-provider model and serializes evidence only once', fn: async (): Promise<void> => {
    const fixture = hostFixture();
    fixture.first.setResponses([fauxAssistantMessage('', { stopReason: 'error', errorMessage: '402: {"message":"out of credits","type":"billing_error"}', },),],);
    fixture.other.setResponses([fauxAssistantMessage('usable review',),],);
    const result = await runAdvisor({ ctx: fixture.ctx, config, },);
    expect(result.text,).toBe('usable review',);
    expect(fixture.first.state.callCount,).toBe(1,);
    expect(fixture.other.state.callCount,).toBe(1,);
    expect(fixture.state.branchReads,).toBe(1,);
    expect(result.details.operation?.blockedProviders,).toEqual(['first',],);
    expect(result.details.usage,).toEqual(result.details.operation?.usage,);
  }, },),
  it({ name: 'explicit requests do not switch models even when overlap is enabled', fn: async (): Promise<void> => {
    const fixture = hostFixture();
    fixture.first.setResponses([fauxAssistantMessage('', { stopReason: 'error', errorMessage: 'out of credits', },),],);
    fixture.other.setResponses([fauxAssistantMessage('must not dispatch',),],);
    let caught: unknown;
    try {
      await runAdvisor({ ctx: fixture.ctx, config: { ...config, hedgingEnabled: true, hedgeDelayMs: 1, }, requestedSlug: 'first/a', },);
    }
    catch (error) { caught = error; }
    expect(caught,).toBeInstanceOf(AdvisorOperationError,);
    expect(fixture.first.state.callCount,).toBe(1,);
    expect(fixture.other.state.callCount,).toBe(0,);
  }, },),
  ...['raw', 'wrapped', 'malformed',].map(shape => it({ name: `getter-backed ${shape} scope changes during auth prevent stale dispatch`, fn: async (): Promise<void> => {
    const fixture = hostFixture();
    fixture.first.setResponses([fauxAssistantMessage('must not dispatch',),],);
    fixture.state.auth = () => {
      fixture.state.live = shape === 'malformed' ? [{}] : shape === 'raw' ? fixture.other.models : fixture.other.models.map(model => ({ model, thinkingLevel: 'high', }));
    };
    let caught: unknown;
    try {
      await runAdvisor({ ctx: fixture.ctx, config, requestedSlug: 'first/a', },);
    }
    catch (error) {
      caught = error;
    }
    expect(caught,).toBeInstanceOf(AdvisorOperationError,);
    expect((caught as Error).message,).toContain('left the live scope before dispatch',);
    expect(fixture.first.state.callCount,).toBe(0,);
    expect(fixture.other.state.callCount,).toBe(0,);
  }, },),),
  it({ name: 'clearing Pi model cycling scope does not revoke the selected endpoint', fn: async (): Promise<void> => {
    const fixture = hostFixture();
    fixture.first.setResponses([fauxAssistantMessage('review after clearing cycle list',),],);
    fixture.state.auth = (): void => {
      fixture.state.live = [];
    };
    const result = await runAdvisor({ ctx: fixture.ctx, config, requestedSlug: 'first/a', },);
    expect(result.text,).toBe('review after clearing cycle list',);
    expect(fixture.first.state.callCount,).toBe(1,);
    expect(fixture.other.state.callCount,).toBe(0,);
  }, },),
  it({ name: 'already-cancelled caller invokes neither scope nor provider', fn: async (): Promise<void> => {
    const fixture = hostFixture();
    const controller = new AbortController();
    controller.abort();
    let caught: unknown;
    try {
      await runAdvisor({ ctx: fixture.ctx, config, signal: controller.signal, },);
    }
    catch (error) {
      caught = error;
    }
    expect(caught,).toBeInstanceOf(AdvisorOperationError,);
    expect(fixture.state.branchReads,).toBe(0,);
    expect(fixture.first.state.callCount,).toBe(0,);
  }, },),
], },);
