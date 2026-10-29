/**
 Original native OAuth ownership and genuine virtual catalog refresh lifecycle.
 
 @module
 */
import type { Api, Model, AssistantMessage, } from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { CODEX_PROVIDER, FAST_PROVIDER, isPriorityTarget, PRIORITY_TARGET_PREFIX, registerOpenAIFast, } from '../dist/final/node/index.mjs';
import { fixtureContext, fixtureModel, } from './host-fixture-model.ts';
import { fixtureCredential, HOST_TOKEN, } from './host-fixture-provider.ts';
import { fixtureHome, fixtureHost, requireCompanion, } from './host-fixture-session.ts';

//region OAuth and catalog: no extra login or stale compatibility allowlist.

await describe({ name: registerOpenAIFast.name, children: [
  it({ name: 'serializes synthetic OAuth refresh, reuses the stored credential, and rejects dispatch after logout', fn: async function verifyOriginalOAuth() {
    /** Expired synthetic OAuth is stored only in independently disposable state. */
    await using home = await fixtureHome();
    /** Real native credential lock owns refresh across concurrent priority requests. */
    using host = await fixtureHost({ home, expired: true, },);
    /** User-visible virtual model must route to original native authentication. */
    const companion = requireCompanion({ runtime: host.runtime, id: host.base.id, },);
    expect(host.source.state.oauthRefreshes,).toBe(0,);
    /** Entry barrier proves the native refresh is held before any transport dispatch. */
    const entered = Promise.withResolvers<void>();
    /** Release barrier lets assertions inspect serialization before refresh completes. */
    const released = Promise.withResolvers<void>();
    host.source.state.refreshGate = async function refreshGate(): Promise<void> {
      entered.resolve();
      await released.promise;
    };
    /** First native request enters the original provider's credential refresh lock. */
    const first = host.runtime.completeSimple(companion, fixtureContext(), { reasoning: 'high', },);
    /** Concurrent request must share that refresh rather than create another login. */
    const second = host.runtime.completeSimple(companion, fixtureContext(), { reasoning: 'xhigh', },);
    {
      /** Refresh work is released and drained even if in-flight assertions fail. */
      await using refreshCompletion = {
        [Symbol.asyncDispose]: async function dispose(): Promise<void> {
          released.resolve();
          await Promise.all([first, second,],);
        },
      };
      await Promise.race([entered.promise, Promise.all([first, second,],),],);
      expect(host.source.state.oauthRefreshes,).toBe(1,);
      expect(host.source.state.calls,).toHaveLength(0,);
    }
    /** Settled native replies are inspected only after cleanup releases the gate. */
    const results = await Promise.all([first, second,],);
    expect(results.every(function successfulReply(message: ForeignBorrowed<AssistantMessage>) {
      return message.stopReason === 'stop';
    },),).toBe(true,);
    /** Later native request must reuse the refreshed original credential. */
    const later = await host.runtime.completeSimple(companion, fixtureContext(), { reasoning: 'high', },);
    expect(later.stopReason,).toBe('stop',);
    expect(host.source.state.oauthRefreshes,).toBe(1,);
    expect(host.source.state.calls,).toHaveLength(3,);
    expect(await host.credentials.read(CODEX_PROVIDER,),).toMatchObject({ type: 'oauth', refresh: 'fixture-rotated-refresh', access: HOST_TOKEN, },);
    expect(await host.credentials.list(),).toEqual([{ providerId: CODEX_PROVIDER, type: 'oauth', },],);
    expect(await host.credentials.read(FAST_PROVIDER,),).toBeUndefined();
    for (const call of host.source.state.calls)
      expect(call.options,).toHaveProperty('apiKey', HOST_TOKEN,);
    await host.runtime.logout(CODEX_PROVIDER,);
    /** Availability follows the original logout rather than a cached adapter-ready flag. */
    expect(await host.runtime.getAvailable(FAST_PROVIDER,),).toEqual([],);
    expect((await host.runtime.getAvailable()).some(function orphanedFast(model: ForeignBorrowed<Model<Api>>) {
      return model.provider === FAST_PROVIDER;
    },),).toBe(false,);
    expect(host.ctx.modelRegistry.getAvailable().some(function cachedFast(model: ForeignBorrowed<Model<Api>>) {
      return model.provider === FAST_PROVIDER;
    },),).toBe(false,);
    /** Logged-out priority request must fail without dispatch or ordinary fallback. */
    const denied = await host.runtime.completeSimple(companion, fixtureContext(), { reasoning: 'high', },);
    expect(denied.stopReason,).toBe('error',);
    expect(denied.errorMessage,).toContain('which has no credentials',);
    expect(host.source.state.calls,).toHaveLength(3,);
    expect(host.source.state.oauthRefreshes,).toBe(1,);
    expect(await host.credentials.list(),).toEqual([],);
    /** Reconfiguration becomes visible in the same bound session without copied fast credentials. */
    await host.credentials.modify(CODEX_PROVIDER, function restoreLogin() {
      return Promise.resolve(fixtureCredential(),);
    },);
    expect((await host.runtime.getAvailable(FAST_PROVIDER,)).some(function restoredFast(model: ForeignBorrowed<Model<Api>>) {
      return (model.id === host.base.id) && (model.api === 'pi-virtual');
    },),).toBe(true,);
    await host.runtime.refresh({ allowNetwork: false, },);
    expect(host.ctx.modelRegistry.getAvailable().some(function visibleAgain(model: ForeignBorrowed<Model<Api>>) {
      return (model.provider === FAST_PROVIDER) && (model.id === host.base.id);
    },),).toBe(true,);
    expect(await host.credentials.read(FAST_PROVIDER,),).toBeUndefined();
  }, },),
  it({ name: 'adds, updates, and removes virtual companions when the actual native source catalog refreshes', fn: async function verifyNativeCatalogRefresh() {
    /** Native runtime refresh and all catalog writes remain in disposable storage. */
    await using home = await fixtureHome();
    /** Actual host lazy synchronization must track original native model changes. */
    using host = await fixtureHost({ home, },);
    /** Changed original capabilities must replace the virtual advertised definition. */
    const changed = fixtureModel({ name: 'Refreshed fixture', contextWindow: 444_444, maxTokens: 22_222, input: ['text',], },);
    /** New native identity must gain a genuine virtual companion without an allowlist. */
    const added = fixtureModel({ id: 'new-native-model', },);
    host.source.state.nextModels = [changed, added,];
    /** Real native refresh publishes changes under the original provider ID. */
    const updated = await host.runtime.refresh({ providers: [CODEX_PROVIDER,], allowNetwork: false, },);
    expect(updated.errors.size,).toBe(0,);
    expect(requireCompanion({ runtime: host.runtime, id: changed.id, },),).toMatchObject({ name: 'Refreshed fixture Fast',
      contextWindow: 444_444, maxTokens: 22_222, input: ['text',], },);
    expect(requireCompanion({ runtime: host.runtime, id: added.id, },).api,).toBe('pi-virtual',);
    host.source.state.nextModels = [added,];
    /** Source disappearance must remove both visible companion and hidden target. */
    const removed = await host.runtime.refresh({ providers: [CODEX_PROVIDER,], allowNetwork: false, },);
    expect(removed.errors.size,).toBe(0,);
    expect(host.runtime.getModel(FAST_PROVIDER, changed.id,),).toBeUndefined();
    expect(host.runtime.getModel(FAST_PROVIDER, `${PRIORITY_TARGET_PREFIX}${changed.id}`,),).toBeUndefined();
    /** Complete catalog retains exactly the new virtual and its local physical target. */
    const current = host.runtime.getModels(FAST_PROVIDER,);
    expect(current.map(function modelId(model: ForeignBorrowed<Model<Api>>) {
      return model.id;
    },),).toEqual([`${PRIORITY_TARGET_PREFIX}${added.id}`, added.id,],);
    expect(current.filter(function selectedVirtual(model: ForeignBorrowed<Model<Api>>) {
      return model.api === 'pi-virtual';
    },).map(function virtualId(model: ForeignBorrowed<Model<Api>>) {
      return model.id;
    },),).toEqual([added.id,],);
    expect((await host.runtime.getAvailable(FAST_PROVIDER,)).some(function physicalTarget(model: ForeignBorrowed<Model<Api>>) {
      return isPriorityTarget(model,);
    },),).toBe(false,);
  }, },),
], },);

//endregion OAuth and catalog
