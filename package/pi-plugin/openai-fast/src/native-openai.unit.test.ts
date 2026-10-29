/**
 Native OpenAI companions share priority routing without replacing either source provider.

 @module
 */
import { writeFile, } from 'node:fs/promises';
import type { Api, Model, } from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  CODEX_PROVIDER,
  FAST_PROVIDER,
  OPENAI_API,
  OPENAI_PROVIDER,
  loadOriginalProvider,
  registerOpenAIFast,
  streamPriority,
  streamSimplePriority,
} from '../dist/final/node/index.mjs';
import { fixtureHttp, nativeResponse, } from './host-fixture-http.ts';
import { fixtureContext, fixtureModel, } from './host-fixture-model.ts';
import { fixtureCredential, HOST_TOKEN, } from './host-fixture-provider.ts';
import { fixtureHome, fixtureHost, requireCompanion, } from './host-fixture-session.ts';
import { stopStreamFixtureTransport, } from './stream-fixture-http.ts';
import { StreamFixtureStopError, } from './stream-fixture.ts';

//region Native provider bootstrap and actual host dispatch.

await describe({ name: '', children: [
  describe({ name: loadOriginalProvider.name, children: [
    it({ name: 'restores configured native OpenAI metadata without credentials or networking', fn: async function configuredOpenAI(ctx) {
      /** Disposable configuration must not consult or mutate real pi authentication. */
      await using home = await fixtureHome();
      /** Unexpected catalog networking must reject before reaching an external endpoint. */
      const fetch = ctx.sinon.stub(globalThis, 'fetch',).callsFake(function unexpectedFetch(): Promise<Response> {
        return Promise.reject(new Error('Native catalog fixture must remain offline.',),);
      },);
      /** Custom native model demonstrates catalog-derived coverage rather than an allowlist. */
      const custom = { ...fixtureModel({ id: 'future-openai-fixture', },), provider: OPENAI_PROVIDER,
        api: OPENAI_API, baseUrl: 'https://api.openai.com/v1', };
      await writeFile(home.modelsPath, JSON.stringify({ providers: { [OPENAI_PROVIDER]: {
        api: OPENAI_API, models: [custom,],
      }, }, },),);
      /** Built loader restores the new source without requesting a login or token refresh. */
      const provider = await loadOriginalProvider({ providerId: OPENAI_PROVIDER, modelsPath: home.modelsPath, },);
      expect(provider.id,).toBe(OPENAI_PROVIDER,);
      expect(provider.auth.oauth?.loginLabel,).toBe('Sign in with ChatGPT',);
      expect(provider.auth.apiKey,).toBeDefined();
      expect(provider.getModels().find(function configuredModel(model: ForeignBorrowed<Model<Api>>) {
        return model.id === custom.id;
      },),).toMatchObject(custom,);
      expect(fetch,).not.toHaveBeenCalled();
    }, },),
  ], },),
  describe({ name: registerOpenAIFast.name, children: [
    it({ name: 'keeps equal native IDs separate, preserves ordinary requests, and delegates new fast requests to native auth and HTTP', fn: async function nativeHost() {
      /** Sessions, settings, and credentials belong exclusively to this disposable host. */
      await using home = await fixtureHome();
      /** Finite HTTP response budgets make cross-provider dispatch or fallback observable. */
      const legacy = fixtureHttp({ responses: [nativeResponse, nativeResponse,], },);
      /** The new source uses its actual native Responses adapter and authentication descriptor. */
      const native = fixtureHttp({ providerId: OPENAI_PROVIDER, responses: [nativeResponse, nativeResponse, nativeResponse,], },);
      /** Register both source-bound adapters through actual extension capabilities. */
      using host = await fixtureHost({ home, source: legacy.source,
        register: function registerBoth({ pi, provider, },) {
          registerOpenAIFast({ pi, provider, },);
          pi.registerProvider(native.source.provider,);
          registerOpenAIFast({ pi, provider: native.source.provider, },);
        }, },);
      await host.credentials.modify(OPENAI_PROVIDER, function seedNative() {
        return Promise.resolve(fixtureCredential(),);
      },);
      await host.runtime.refresh({ allowNetwork: false, },);
      /** Settings are a positive control for unchanged defaults and exact model scopes. */
      const settings = host.settings.getSettings();
      /** Identical upstream IDs must never select another source provider implicitly. */
      const ordinary = host.runtime.getModel(OPENAI_PROVIDER, host.base.id,);
      /** Selectable companion remains virtual while its native source is untouched. */
      const fast = host.runtime.getModel('openai-fast', host.base.id,);
      if ((ordinary === undefined) || (fast === undefined))
        throw new Error('Native OpenAI fixture did not register its ordinary and fast models.',);
      expect(fast,).toMatchObject({ api: 'pi-virtual', provider: 'openai-fast', id: host.base.id, },);
      expect(host.runtime.getRegisteredNativeProvider(OPENAI_PROVIDER,),).toBe(native.source.provider,);
      expect(host.runtime.getRegisteredNativeProvider(CODEX_PROVIDER,),).toBe(legacy.source.provider,);
      await host.session.prompt('Ordinary legacy request.',);
      /** Ordinary native response supplies a control that would detect globally forced priority. */
      const normal = await host.runtime.completeSimple(ordinary, fixtureContext(), { reasoning: 'high', },);
      expect(normal.stopReason,).toBe('stop',);
      await host.session.setModel(fast,);
      await host.session.prompt('Native fast request.',);
      /** Caller API-key overrides and payload replacement pass through the same native path. */
      const explicit = await host.runtime.completeSimple(fast, fixtureContext(), {
        apiKey: 'sk-native-fixture', reasoning: 'high', headers: { 'x-request-fixture': 'caller', },
        onPayload: async function replaceTier(payload: unknown, model: ForeignBorrowed<Model<Api>>) {
          expect(model,).toMatchObject({ provider: OPENAI_PROVIDER, api: OPENAI_API, id: host.base.id, },);
          if (((typeof payload) !== 'object') || (payload === null) || Array.isArray(payload,))
            throw new Error('Native OpenAI fixture expected an object payload.',);
          return await Promise.resolve({ ...payload, service_tier: 'default', },);
        },
      },);
      expect(explicit,).toMatchObject({ stopReason: 'stop', provider: OPENAI_PROVIDER, api: OPENAI_API, model: host.base.id, },);
      expect(native.requests,).toHaveLength(3,);
      expect(native.requests[0]?.payload,).not.toHaveProperty('service_tier',);
      for (const request of native.requests.slice(1,)) {
        expect(request.url,).toBe('https://api.openai.com/v1/responses',);
        expect(request.payload,).toMatchObject({ model: host.base.id, service_tier: 'priority', },);
        expect(request.payload,).toHaveProperty('reasoning.effort', 'high',);
      }
      expect(native.requests[1]?.headers.get('authorization',),).toBe(`Bearer ${HOST_TOKEN}`,);
      expect(native.requests[2]?.headers.get('authorization',),).toBe('Bearer sk-native-fixture',);
      expect(native.requests[2]?.headers.get('x-request-fixture',),).toBe('caller',);
      expect(host.session.messages.at(-1),).toMatchObject({ provider: OPENAI_PROVIDER, api: OPENAI_API, model: host.base.id, stopReason: 'stop', },);
      await host.session.setModel(requireCompanion({ runtime: host.runtime, id: host.base.id, },),);
      await host.session.prompt('Legacy fast request.',);
      expect(legacy.requests,).toHaveLength(2,);
      expect(legacy.requests[0]?.payload,).not.toHaveProperty('service_tier',);
      expect(legacy.requests[1]?.payload,).toMatchObject({ model: host.base.id, service_tier: 'priority', },);
      expect(host.session.model?.provider,).toBe(FAST_PROVIDER,);
      expect(host.settings.getSettings(),).toEqual(settings,);
      expect(host.settings.getDefaultProvider(),).toBe(CODEX_PROVIDER,);
      expect(host.settings.getDefaultModel(),).toBe(host.base.id,);
      expect(host.settings.getEnabledModels(),).toEqual([`${CODEX_PROVIDER}/${host.base.id}`,],);
      expect(host.ctx.scopedModels,).toEqual([{ model: host.base, thinkingLevel: 'high', },],);
      /** Native logout must hide only its own fast namespace while legacy remains configured. */
      await host.runtime.logout(OPENAI_PROVIDER,);
      expect(await host.runtime.getAvailable('openai-fast',),).toEqual([],);
      expect((await host.runtime.getAvailable(FAST_PROVIDER,)).some(function legacyStillAvailable(model: ForeignBorrowed<Model<Api>>) {
        return (model.id === host.base.id) && (model.api === 'pi-virtual');
      },),).toBe(true,);
      /** Fresh native credentials restore availability in the same bound host. */
      await host.credentials.modify(OPENAI_PROVIDER, function restoreNative() {
        return Promise.resolve(fixtureCredential(),);
      },);
      expect((await host.runtime.getAvailable('openai-fast',)).some(function nativeRestored(model: ForeignBorrowed<Model<Api>>) {
        return (model.id === host.base.id) && (model.api === 'pi-virtual');
      },),).toBe(true,);
      expect(await host.credentials.read('openai-fast',),).toBeUndefined();
    }, },),
  ], },),
  describe({ name: 'direct native OpenAI priority streams', children: [streamPriority, streamSimplePriority,].map(function nativeEntryPoint(entryPoint,) {
    return it({ name: `${entryPoint.name} uses native OpenAI preparation before caller cancellation`, fn: async function nativePreparation(ctx) {
        /** Original native identity is required for ChatGPT-specific request preparation. */
      const model = { ...fixtureModel(), provider: OPENAI_PROVIDER, api: OPENAI_API,
        baseUrl: 'https://api.openai.com/v1', } satisfies Model<typeof OPENAI_API>;
        /** Transport observation proves payload cancellation prevents networking. */
      const fetch = ctx.sinon.spy(stopStreamFixtureTransport,);
        /** Native error formatting must preserve the caller's explicit stop reason. */
      const stop = new StreamFixtureStopError();
        /** Both public wrappers must choose the new native adapter when no dispatch is injected. */
      const result = await entryPoint({ model, context: fixtureContext(), options: {
        apiKey: HOST_TOKEN, fetch, maxTokens: 123, temperature: 0.5,
        onPayload: function stopPayload(payload,) {
          expect(payload,).toMatchObject({ model: model.id, service_tier: 'priority', },);
          expect(payload,).not.toHaveProperty('max_output_tokens',);
          expect(payload,).not.toHaveProperty('temperature',);
          throw stop;
        },
      }, },).result();
      expect(result.stopReason,).toBe('error',);
      expect(result.errorMessage,).toContain(stop.message,);
      expect(fetch,).not.toHaveBeenCalled();
    }, },);
  },), },),
], },);

//endregion Native provider bootstrap and actual host dispatch.
