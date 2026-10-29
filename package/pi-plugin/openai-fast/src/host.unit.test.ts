/**
 Actual native host selection and pre-session dispatch controls.
 
 @module
 */
import type { Api, Model, } from '@earendil-works/pi-ai';
import type { ModelRuntime, } from '@earendil-works/pi-coding-agent';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import type { ForeignBorrowed, ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  CODEX_API,
  CODEX_PROVIDER,
  FAST_PROVIDER,
  isPriorityTarget,
  PRIORITY_TARGET_PREFIX,
  registerOpenAIFast,
} from '../dist/final/node/index.mjs';
import { fixtureContext, fixtureModel, } from './host-fixture-model.ts';
import { fixtureProvider, } from './host-fixture-provider.ts';
import { fixtureHome, fixtureHost, requireCompanion, } from './host-fixture-session.ts';

//region Actual host: visible entries are virtual, never physical targets.

await describe({ name: registerOpenAIFast.name, children: [
  it({ name: 'registers genuine virtual entries without UI, defaults, enabledModels, or scope changes', fn: async function verifyHostSelection() {
    /** Actual SDK registration uses independently disposable storage. */
    await using home = await fixtureHome();
    /** Unrecognized future ID and non-reasoning model must both gain companions. */
    const source = fixtureProvider({ models: [fixtureModel(), fixtureModel({ id: 'unknown-future-native', reasoning: false, input: ['text',], },),], },);
    /** Real host supplies native selectors, settings, and registry capabilities. */
    using host = await fixtureHost({ home, source, },);
    expect(host.ctx.hasUI,).toBe(false,);
    expect(host.session.model,).toMatchObject({ provider: CODEX_PROVIDER, id: host.base.id, api: CODEX_API, },);
    expect(host.settings.getDefaultProvider(),).toBe(CODEX_PROVIDER,);
    expect(host.settings.getDefaultModel(),).toBe(host.base.id,);
    expect(host.settings.getEnabledModels(),).toEqual([`${CODEX_PROVIDER}/${host.base.id}`,],);
    expect(host.ctx.scopedModels,).toHaveLength(1,);
    expect(host.ctx.scopedModels[0],).toMatchObject({ model: host.base, thinkingLevel: 'high', },);
    expect(host.pi.getCommands(),).toEqual([],);
    expect(host.pi.getAllTools().filter(function nonBuiltin(tool) {
      return tool.sourceInfo.source !== 'builtin';
    },),).toEqual([],);
    /** Complete catalog contains native virtual selections and their physical routing targets. */
    const catalog = host.runtime.getModels(FAST_PROVIDER,);
    /** Selector entries are distinguished by the host's actual virtual API marker. */
    const virtuals = catalog.filter(function selectedVirtual(model: ForeignBorrowed<Model<Api>>) {
      return model.api === 'pi-virtual';
    },);
    /** Target inventory remains exact even though targets are hidden from filtered availability. */
    const targets = catalog.filter(function targetModel(model: ForeignBorrowed<Model<Api>>) {
      return isPriorityTarget(model,);
    },);
    expect(targets.map(function targetId(model: ForeignBorrowed<Model<Api>>) {
      return model.id;
    },),).toEqual(source.state.models.map(function expectedTarget(model: ForeignBorrowed<Model<Api>>) {
      return `${PRIORITY_TARGET_PREFIX}${model.id}`;
    },),);
    expect(catalog,).toHaveLength(virtuals.length + targets.length,);
    expect(virtuals.map(function modelId(model: ForeignBorrowed<Model<Api>>) {
      return model.id;
    },),).toEqual(source.state.models.map(function sourceId(model: ForeignBorrowed<Model<Api>>) {
      return model.id;
    },),);
    expect(virtuals.every(function virtualModel(model: ForeignBorrowed<Model<Api>>) {
      return model.api === 'pi-virtual';
    },),).toBe(true,);
    expect(host.runtime.getProvider(CODEX_PROVIDER,),).toBe(host.original,);
    expect(host.runtime.getRegisteredNativeProvider(CODEX_PROVIDER,),).toBe(source.provider,);
    /** Native availability must include virtuals but hide every physical target. */
    const available = await host.runtime.getAvailable(FAST_PROVIDER,);
    expect(available.some(function availableTarget(model: ForeignBorrowed<Model<Api>>) {
      return isPriorityTarget(model,);
    },),).toBe(false,);
    expect(host.ctx.modelRegistry.getAvailable().some(function registryTarget(model: ForeignBorrowed<Model<Api>>) {
      return isPriorityTarget(model,);
    },),).toBe(false,);
    expect((await host.runtime.getAllAvailable(FAST_PROVIDER,)).some(function physicalTarget(model: ForeignBorrowed<{
      readonly id: string;
    }>) { return isPriorityTarget(model,); },),).toBe(false,);
    /** Route work retains fixture ownership until every sibling settles, even on assertion failure. */
    const routes = source.state.models.map(async function verifyBaseRoute(base: ForeignBorrowed<Model<Api>>) {
      /** Hidden target preserves native capabilities without original model headers. */
      const target = host.ctx.modelRegistry.getModelOfType('chat', FAST_PROVIDER, `${PRIORITY_TARGET_PREFIX}${base.id}`,);
      expect(target,).toMatchObject({ api: CODEX_API, provider: FAST_PROVIDER, contextWindow: base.contextWindow, },);
      expect(target,).not.toHaveProperty('headers',);
      /** Genuine virtual entry routes to exactly its own hidden target. */
      const companion = requireCompanion({ runtime: host.runtime, id: base.id, },);
      expect(available,).toContainEqual(companion,);
      /** Native routing retains high reasoning only when the original supports it. */
      const route = await host.runtime.resolveModel(companion, [], { reason: 'direct', thinkingLevel: 'high', },);
      expect(route.model,).toEqual(target,);
      expect(route.thinkingLevel,).toBe(base.reasoning ? 'high' : 'off',);
    },);
    {
      /** Pending siblings are drained before host and home disposal if any route fails. */
      await using routeCompletion = {
        [Symbol.asyncDispose]: async function dispose(): Promise<void> {
          await Promise.allSettled(routes,);
        },
      };
      await Promise.all(routes,);
    }
    expect(host.source.state.calls,).toHaveLength(0,);
  }, },),
  it({ name: 'rejects fast dispatch before session_start instead of using bootstrap credentials or transport', fn: async function verifyPreStartDispatch() {
    /** Pre-start probe uses only disposable real host services. */
    await using home = await fixtureHome();
    /** Bootstrap native transport must never receive the premature dispatch. */
    const source = fixtureProvider();
    /** Actual SDK adapter is installed before the session-start registry binding. */
    using host = await fixtureHost({ home, source, beforeStart: async function beforeStart(runtime: ForeignHostCapability<ModelRuntime>) {
      /** Internal target exists before lifecycle binding but cannot infer request auth. */
      const target = runtime.getModel(FAST_PROVIDER, `${PRIORITY_TARGET_PREFIX}gpt-host-fixture`,);
      if (target === undefined)
        throw new Error('Actual SDK did not install the adapter before lifecycle binding.',);
      /** Real native completion must report the explicit initialization diagnostic. */
      const message = await runtime.complete(target, fixtureContext(),);
      expect(message.stopReason,).toBe('error',);
      expect(message.errorMessage,).toContain('initialized pi session',);
      expect(source.state.calls,).toHaveLength(0,);
    }, },);
    expect(host.session.model?.provider,).toBe(CODEX_PROVIDER,);
  }, },),
], },);

//endregion Actual host
