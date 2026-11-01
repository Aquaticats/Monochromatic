/**
 Genuine virtual companion registration from the complete native catalog.
 
 @module
 */
import { getSupportedThinkingLevels, type Api, type Model, } from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  createFastModelRegistration,
  FAST_PROVIDER,
  CODEX_PROVIDER,
  FastModelError,
  PRIORITY_TARGET_PREFIX,
} from '../dist/final/node/index.mjs';
import { caughtFailure, fixtureModel, } from './host-fixture-model.ts';
import { fixtureRegistrationHost as fixtureHost, fixtureHome, } from './host-fixture-session.ts';

//region Native registration: no allowlist or unrelated settings mutation.

await describe({ name: createFastModelRegistration.name, children: [
  it({ name: 'creates native pi virtual companions for every supplied base and preserves advertised metadata', fn: async function verifyVirtualMetadata() {
    /** Disposable storage isolates actual native registration. */
    await using home = await fixtureHome();
    /** Registration-only host avoids plugin-driven lazy synchronization. */
    using host = await fixtureHost({ home, },);
    /** Built synchronizer installs virtual selections through the real SDK. */
    const synchronize = createFastModelRegistration({ pi: host.pi, },);
    synchronize(host.source.provider.getModels(),);
    /** Unrecognized native IDs and text-only capability variations need companions. */
    const models = [fixtureModel({ id: 'unknown-future-model', },),
      fixtureModel({ id: 'custom-text-only', reasoning: false, input: ['text',], contextWindow: 98_765, maxTokens: 432, },),];
    /** Settings control detects any unrelated default or selector-policy change. */
    const settings = host.settings.getSettings();
    synchronize(models,);
    expect(host.runtime.getModels(FAST_PROVIDER,).map(function modelId(model: ForeignBorrowed<Model<Api>>) {
      return model.id;
    },),).toEqual(models.map(function baseId(model: ForeignBorrowed<Model<Api>>) {
      return model.id;
    },),);
    for (const base of models) {
      /** Actual companion must advertise native capabilities without becoming physical. */
      const companion = host.runtime.getModel(FAST_PROVIDER, base.id,);
      expect(companion,).toMatchObject({ api: 'pi-virtual', provider: FAST_PROVIDER, id: base.id,
        name: `${base.name} Fast`, input: base.input, contextWindow: base.contextWindow, maxTokens: base.maxTokens, },);
      if (companion === undefined)
        throw new Error('Native virtual companion was not registered.',);
      expect(getSupportedThinkingLevels(companion,),).toEqual(getSupportedThinkingLevels(base,),);
    }
    expect(host.settings.getSettings(),).toEqual(settings,);
    expect(host.session.model?.provider,).toBe(CODEX_PROVIDER,);
  }, },),
  it({ name: 'deduplicates structural definitions, updates changed capabilities, and removes disappeared companions', fn: async function verifyStructuralUpdates(ctx) {
    /** All registry changes remain in independently disposable state. */
    await using home = await fixtureHome();
    /** Actual SDK virtual registration is observed through host capability spies. */
    using host = await fixtureHost({ home, },);
    /** Registration count distinguishes structural changes from cost-only updates. */
    const register = ctx.sinon.spy(host.pi, 'registerVirtualModel',);
    /** Removal count verifies disappeared aliases are actually unregistered. */
    const unregister = ctx.sinon.spy(host.pi, 'unregisterVirtualModel',);
    /** Built synchronizer owns structural signatures and virtual entries. */
    const synchronize = createFastModelRegistration({ pi: host.pi, },);
    /** Original native metadata anchors the duplicate-registration control. */
    const base = fixtureModel();
    /** Added native identity must produce an additional companion. */
    const added = fixtureModel({ id: 'new-catalog-model', },);
    synchronize([base,],);
    synchronize([{ ...base, input: [...base.input,], cost: { ...base.cost, input: 99, }, },],);
    expect(register,).toHaveBeenCalledTimes(1,);
    /** Changed advertised capabilities require a new structural definition. */
    const changed = { ...base, name: 'Updated name', input: ['text',] as ('text' | 'image')[],
      contextWindow: 123_123, maxTokens: 321, thinkingLevelMap: { high: null, xhigh: null, }, };
    synchronize([changed, added,],);
    expect(register,).toHaveBeenCalledTimes(3,);
    expect(host.runtime.getModel(FAST_PROVIDER, base.id,),).toMatchObject({ name: 'Updated name Fast',
      input: ['text',], contextWindow: 123_123, maxTokens: 321, },);
    synchronize([added,],);
    expect(unregister,).toHaveBeenCalledExactlyOnceWith(FAST_PROVIDER, base.id,);
    expect(host.runtime.getModel(FAST_PROVIDER, base.id,),).toBeUndefined();
    synchronize([],);
    expect(host.runtime.getModels(FAST_PROVIDER,),).toEqual([],);
    expect(unregister,).toHaveBeenCalledTimes(2,);
  }, },),
  ...[false, true,].map(function collisionScenario(collision) {
    return it({ name: collision
      ? 'rejects a base catalog containing another model at the generated physical identity'
      : 'rejects upstream IDs inside the reserved local target namespace', fn: async function verifyReservedIdentity() {
        /** Reserved-identity probes remain within the disposable real registry. */
        await using home = await fixtureHome();
        /** Host exposes native registration without plugin catalog reads. */
        using host = await fixtureHost({ home, },);
        /** Built synchronizer must reject impossible routing identities. */
        const synchronize = createFastModelRegistration({ pi: host.pi, },);
        /** Native identity whose generated target can collide with another base. */
        const base = fixtureModel();
        /** Upstream fixture in the extension-owned namespace must never be routed. */
        const reserved = fixtureModel({ id: `${PRIORITY_TARGET_PREFIX}${base.id}`, },);
        expect(function synchronizeCollision() {
          synchronize(collision ? [base, reserved,] : [reserved,],);
        },).toThrow(FastModelError,);
        expect(function synchronizeCollisionDiagnostic() {
          synchronize(collision ? [base, reserved,] : [reserved,],);
        },).toThrow('internal routing namespace',);
      }, },);
  },),
  it({ name: 'reports an absent physical route rather than falling back to an ordinary model', fn: async function verifyAbsentRoute() {
    /** Missing-target route resolution uses only disposable native host state. */
    await using home = await fixtureHome();
    /** Real runtime can register a virtual entry without its physical route. */
    using host = await fixtureHost({ home, },);
    /** Removed physical target identity must not be substituted with an ordinary model. */
    const absent = fixtureModel({ id: 'removed-routing-target', },);
    /** Built synchronizer installs the deliberately unroutable virtual selection. */
    const synchronize = createFastModelRegistration({ pi: host.pi, },);
    synchronize([absent,],);
    /** Actual native virtual selection triggers the built route callback. */
    const companion = host.runtime.getModel(FAST_PROVIDER, absent.id,);
    if (companion === undefined)
      throw new Error('Fixture virtual model was not registered.',);
    /** Native resolution failure must retain the no-target diagnostic. */
    const error = await caughtFailure(async function resolveAbsentRoute() {
      return await host.runtime.resolveModel(companion, [], { reason: 'direct', thinkingLevel: 'high', },);
    },);
    expect(error,).toBeInstanceOf(FastModelError,);
    expect(error,).toHaveProperty('message',);
    expect(error,).toSatisfy(function missingTargetDiagnostic(value) {
      return (value instanceof FastModelError) && value.message.includes('no current priority target',);
    },);
    expect(host.source.state.calls,).toHaveLength(0,);
  }, },),
], },);

//endregion Native registration
