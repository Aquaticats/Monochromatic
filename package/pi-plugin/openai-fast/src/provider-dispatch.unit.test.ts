/**
 Keyless priority adapter dispatch versus explicit ordinary delegation.
 
 @module
 */
import type { Provider, SimpleStreamOptions, OpenAICodexResponsesOptions, } from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import {
  createPriorityProvider,
  FAST_PROVIDER,
  FastModelError,
  priorityTarget,
} from '../dist/final/node/index.mjs';
import { fixtureContext, fixtureModel, } from './host-fixture-model.ts';
import { fixtureOverlay, } from './host-fixture-overlay.ts';
import { fixtureProvider, HOST_TOKEN, } from './host-fixture-provider.ts';

//region Dispatch: ordinary invocations delegate; priority uses injected full dispatch.

await describe({ name: createPriorityProvider.name, children: [
  ...['full', 'simple',].map(function ordinaryScenario(kind) {
    return it({ name: `delegates explicitly invoked ordinary ${kind} streaming unchanged`, fn: async function verifyOrdinaryDelegation() {
      /** Synthetic native provider records exact ordinary request references. */
      const source = fixtureProvider();
      /** Overlay dispatch is captured independently from ordinary delegation. */
      const { overlay, dispatched, } = fixtureOverlay(source.provider,);
      /** Ordinary native identity must never be translated into a target. */
      const model = fixtureModel();
      /** Transcript reference must reach ordinary native dispatch unchanged. */
      const context = fixtureContext();
      /** Caller-owned native options retain explicit ordinary tier and headers. */
      const options: OpenAICodexResponsesOptions & SimpleStreamOptions = { apiKey: HOST_TOKEN, transport: 'auto',
        headers: { 'x-request': 'ordinary', }, serviceTier: 'default', reasoning: 'high', reasoningEffort: 'high', };
      /** Actual native stream completion is awaited before checking delegation. */
      const events = kind === 'full' ? overlay.stream(model, context, options,) : overlay.streamSimple(model, context, options,);
      /** Finite response preserves ordinary upstream identity. */
      const result = await events.result();
      expect(source.state.calls,).toHaveLength(1,);
      expect(source.state.calls[0]?.kind,).toBe(kind,);
      expect(source.state.calls[0]?.model,).toBe(model,);
      expect(source.state.calls[0]?.context,).toBe(context,);
      expect(source.state.calls[0]?.options,).toBe(options,);
      expect(options.serviceTier,).toBe('default',);
      expect(dispatched,).toHaveLength(0,);
      expect(result.model,).toBe(model.id,);
    }, },);
  },),
  ...['full', 'simple',].map(function priorityScenario(kind) {
    return it({ name: `routes keyless priority ${kind} through injected full dispatch, preserving caller headers`, fn: async function verifyKeylessPriority() {
      /** Native provider supplies current original metadata but is not delegated to. */
      const source = fixtureProvider();
      /** Injected full dispatch observes priority options without adapter credentials. */
      const { overlay, dispatched, } = fixtureOverlay(source.provider,);
      /** Current original must be resolved at dispatch time. */
      const [base,] = source.provider.getModels();
      if (base === undefined)
        throw new Error('Fixture base model is absent.',);
      /** Normalized transcript retains reference identity through the adapter. */
      const context = fixtureContext();
      /** Caller owns header precedence, reasoning, transport, and session identity. */
      const options = { transport: 'auto' as const, reasoning: 'xhigh' as const,
        reasoningEffort: 'xhigh' as const, sessionId: 'fixture-session', headers: { 'X-Conflict': 'caller-wins', }, };
      /** Local target is translated before original-registry dispatch. */
      const target = priorityTarget(base,);
      /** Full and simple priority paths must converge on full injected dispatch. */
      const events = kind === 'full' ? overlay.stream(target, context, options,) : overlay.streamSimple(target, context, options,);
      /** Actual response remains canonical native Codex history. */
      const result = await events.result();
      expect(dispatched,).toHaveLength(1,);
      expect(dispatched[0]?.kind,).toBe('full',);
      expect(dispatched[0]?.model,).toBe(base,);
      expect(dispatched[0]?.context,).toBe(context,);
      expect(dispatched[0]?.options,).toMatchObject({ transport: 'auto', reasoningEffort: 'xhigh',
        serviceTier: 'priority', sessionId: 'fixture-session', headers: options.headers, },);
      expect(dispatched[0]?.options?.headers,).toBe(options.headers,);
      expect(dispatched[0]?.options?.apiKey,).toBeUndefined();
      expect(dispatched[0]?.options?.onPayload,).toBeTypeOf('function',);
      expect(source.state.calls,).toHaveLength(0,);
      expect(options,).not.toHaveProperty('serviceTier',);
      expect(result,).toMatchObject({ provider: 'openai-codex', api: 'openai-codex-responses', model: base.id, },);
    }, },);
  },),
  ...['full', 'simple',].flatMap(function dispatchFailureScenarios(kind) {
    return ['missing', 'unsupported',].map(function originalFailureScenario(reason) {
      return it({ name: `rejects ${kind} target when its original is ${reason}, without dispatch or fallback`, fn: async function verifyNoFallback() {
        /** Source catalog can disappear or change API independently of the target. */
        const source = fixtureProvider();
        /** Dispatch observations must stay empty when lookup rejects the target. */
        const { overlay, dispatched, } = fixtureOverlay(source.provider,);
        /** Target captures the identity before its original becomes invalid. */
        const target = priorityTarget(fixtureModel(),);
        source.state.models = reason === 'missing' ? [] : [{ ...fixtureModel(), api: 'openai-responses', },];
        expect(function dispatchInvalidTarget() {
          return kind === 'full' ? overlay.stream(target, fixtureContext(),) : overlay.streamSimple(target, fixtureContext(),);
        },).toThrow(FastModelError,);
        expect(dispatched,).toHaveLength(0,);
        expect(source.state.calls,).toHaveLength(0,);
        await Promise.resolve();
      }, },);
    },);
  },),
  it({ name: 'owns no original auth, headers, endpoint, mixed operations, or deferred operations', fn: async function verifyAdapterOwnership() {
    /** Static native provider exposes ownership without refresh capability. */
    const source = fixtureProvider({ dynamic: false, },);
    /** Distinct rejected operation would expose an accidentally copied native method. */
    const failure = new Error('Fixture operation reached.',);
    /** Mixed native operations must remain available only on the original provider. */
    const provider: Provider = { ...source.provider,
      generateImages: function generateImages() {
        return Promise.reject(failure,);
      },
      classify: function classify() {
        return Promise.reject(failure,);
      },
      fetchDeferred: function fetchDeferred() {
        throw failure;
      },
      cancelDeferred: function cancelDeferred() {
        return Promise.reject(failure,);
      }, };
    /** Keyless overlay must not inherit native ownership capabilities. */
    const { overlay, } = fixtureOverlay(provider,);
    expect(overlay.id,).toBe(FAST_PROVIDER,);
    expect(overlay.auth,).not.toBe(provider.auth,);
    expect(overlay.auth.oauth,).toBeUndefined();
    expect(overlay.headers,).toBeUndefined();
    expect(overlay.baseUrl,).toBeUndefined();
    for (const key of ['generateImages', 'classify', 'fetchDeferred', 'cancelDeferred',] as const) {
      expect(overlay[key],).toBeUndefined();
      expect(provider[key],).toBeTypeOf('function',);
    }
    /** Auth resolves only routing metadata, never synthetic subscription refresh. */
    const auth = await overlay.auth.apiKey?.resolve({ ctx: {
      env: function env() {
        return Promise.resolve(undefined,);
      },
      fileExists: function fileExists() {
        return Promise.resolve(false,);
      },
    }, signal: new AbortController().signal, },);
    expect(auth,).toEqual({ auth: {}, source: 'routes-to-original-provider', },);
    expect(source.state.oauthRefreshes,).toBe(0,);
  }, },),
], },);

//endregion Dispatch
