/**
 Credential-free effective native Codex catalog discovery.
 
 @module
 */
import { readFile, writeFile, } from 'node:fs/promises';
import type { Api, Model, Credential, } from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  CODEX_API,
  CODEX_PROVIDER,
  createCatalogCredentials,
  FAST_PROVIDER,
  FastModelError,
  loadCodexProvider,
  PRIORITY_TARGET_PREFIX,
} from '../dist/final/node/index.mjs';
import { caughtFailure, fixtureModel, } from './host-fixture-model.ts';
import { fixtureHome, } from './host-fixture-session.ts';

//region Bootstrap: configured metadata requires neither auth nor remote refresh.

await describe({ name: '', children: [
  describe({ name: createCatalogCredentials.name, children: [
    it({ name: 'reads no secrets, lists no credentials, and refuses modification without invoking its callback', fn: async function verifyReadOnlyCredentials() {
      /** Read-only credential facade must never reveal or change secrets. */
      const credentials = createCatalogCredentials();
      /** Proposed modification must never be invoked by the read-only facade. */
      const modificationState = { invoked: false, };
      expect(await credentials.read(CODEX_PROVIDER,),).toBeUndefined();
      expect(await credentials.read(FAST_PROVIDER,),).toBeUndefined();
      expect(await credentials.list(),).toEqual([],);
      /** Actual modification failure retains its remediation diagnostic. */
      const modified = await caughtFailure(async function modifyCredentials() {
        await credentials.modify(CODEX_PROVIDER, function proposedWriter(): Promise<Credential> {
          modificationState.invoked = true;
          return Promise.resolve({ type: 'api_key', key: 'never-stored-fixture', },);
        },);
      },);
      /** Actual deletion failure must identify the forbidden operation. */
      const deleted = await caughtFailure(async function deleteCredentials() {
        await credentials.delete(CODEX_PROVIDER,);
      },);
      expect(modified,).toBeInstanceOf(FastModelError,);
      expect(modified,).toHaveProperty('message',);
      expect(modified,).toSatisfy(function modificationDiagnostic(value) {
        return (value instanceof FastModelError) && value.message.includes('change credentials',);
      },);
      expect(deleted,).toBeInstanceOf(FastModelError,);
      expect(deleted,).toHaveProperty('message',);
      expect(deleted,).toSatisfy(function deletionDiagnostic(value) {
        return (value instanceof FastModelError) && value.message.includes('delete credentials',);
      },);
      expect(modificationState.invoked,).toBe(false,);
      expect(await credentials.list(),).toEqual([],);
    }, },),
  ], },),
  describe({ name: loadCodexProvider.name, children: [
    it({ name: 'includes custom configured models and overrides with their native capabilities and unchanged config', fn: async function verifyConfiguredCatalog(ctx) {
      /** Disposable paths isolate all config discovery from real pi state. */
      await using home = await fixtureHome();
      /** Custom native metadata proves discovery does not apply an allowlist. */
      const original = fixtureModel({ id: 'configured-custom-codex', name: 'Custom configured Codex',
        contextWindow: 654_321, maxTokens: 12_345, input: ['text',],
        headers: { 'x-configured-model': 'custom-fixture', },
        thinkingLevelMap: { high: 'high', xhigh: 'xhigh', low: null, }, },);
      /** Any unexpected network request rejects through fetch's Promise contract. */
      const fetch = ctx.sinon.stub(globalThis, 'fetch',).callsFake(function unexpectedFetch(): Promise<Response> {
        return Promise.reject(new Error('Catalog bootstrap must not make HTTP requests.',),);
      },);
      /** Native catalog without config provides the override control. */
      const baseline = await loadCodexProvider({ modelsPath: home.modelsPath, },);
      /** Existing native entry is overridden without replacing the source catalog. */
      const [overridden,] = baseline.getModels();
      if (overridden === undefined)
        throw new Error('Installed native Codex catalog is empty.',);
      /** Exact JSON is retained to prove discovery does not mutate configuration. */
      const serialized = JSON.stringify({ providers: { [CODEX_PROVIDER]: { api: CODEX_API,
        models: [original,], modelOverrides: { [overridden.id]: { name: 'Override fixture', contextWindow: 777_777, }, }, }, }, },);
      await writeFile(home.modelsPath, serialized,);
      /** Effective provider must combine original, configured, and overridden metadata. */
      const provider = await loadCodexProvider({ modelsPath: home.modelsPath, },);
      /** Custom entry is resolved by its exact upstream identity. */
      const model = provider.getModels().find(function matchesCustom(entry: ForeignBorrowed<Model<Api>>) {
        return entry.id === original.id;
      },);
      expect(provider.id,).toBe(CODEX_PROVIDER,);
      expect(provider.auth.oauth?.isSubscription,).toBe(true,);
      expect(model,).toMatchObject({ id: original.id, name: original.name, api: CODEX_API,
        contextWindow: 654_321, maxTokens: 12_345, input: ['text',], cost: original.cost,
        thinkingLevelMap: original.thinkingLevelMap, },);
      expect(provider.getModels().find(function matchesOverride(entry: ForeignBorrowed<Model<Api>>) {
        return entry.id === overridden.id;
      },),).toMatchObject({ name: 'Override fixture', contextWindow: 777_777, },);
      expect(provider.getModels().map(function modelId(entry: ForeignBorrowed<Model<Api>>) {
        return entry.id;
      },).toSorted(),).toEqual([...baseline.getModels().map(function baselineId(entry: ForeignBorrowed<Model<Api>>) {
        return entry.id;
      },), original.id,].toSorted(),);
      expect(provider.getModels().some(function internalTarget(entry: ForeignBorrowed<Model<Api>>) {
        return entry.id.startsWith(PRIORITY_TARGET_PREFIX,);
      },),).toBe(false,);
      expect(await readFile(home.modelsPath, 'utf8',),).toBe(serialized,);
      expect(fetch,).not.toHaveBeenCalled();
    }, },),
    it({ name: 'hydrates a newer disposable cached catalog without credentials or a network refresh', fn: async function verifyCachedCatalog(ctx) {
      /** Cache discovery uses disposable paths and no real credentials. */
      await using home = await fixtureHome();
      /** Newer synthetic model makes cache hydration observable. */
      const cached = fixtureModel({ id: 'cached-native-codex', contextWindow: 543_210, },);
      /** Exact cached bytes are retained to detect unintended persistence writes. */
      const cache = JSON.stringify({ [CODEX_PROVIDER]: { models: [cached,], lastModified: Number.MAX_SAFE_INTEGER, checkedAt: 0, }, },);
      await writeFile(home.modelsStorePath, cache,);
      /** Network access is rejected through the external fetch completion contract. */
      const fetch = ctx.sinon.stub(globalThis, 'fetch',).callsFake(function unexpectedFetch(): Promise<Response> {
        return Promise.reject(new Error('Cached bootstrap must not request remote catalogs.',),);
      },);
      /** Effective native catalog must hydrate the locally newer cached entry. */
      const provider = await loadCodexProvider({ modelsPath: home.modelsPath, },);
      expect(provider.getModels(),).toContainEqual(cached,);
      expect(await readFile(home.modelsStorePath, 'utf8',),).toBe(cache,);
      expect(fetch,).not.toHaveBeenCalled();
    }, },),
    it({ name: 'loads the complete native static catalog from a missing disposable config without a login', fn: async function verifyMissingConfig() {
      /** Missing configuration is genuine but isolated from real pi state. */
      await using home = await fixtureHome();
      /** Bootstrap must discover the complete installed native catalog. */
      const provider = await loadCodexProvider({ modelsPath: home.modelsPath, },);
      /** Native models remain unique and use the original Codex transport. */
      const models = provider.getModels();
      expect(models.length,).toBeGreaterThan(0,);
      expect(models.every(function nativeModel(model: ForeignBorrowed<Model<Api>>) {
        return (model.api === CODEX_API) && (model.provider === CODEX_PROVIDER);
      },),).toBe(true,);
      expect(new Set(models.map(function modelId(model: ForeignBorrowed<Model<Api>>) {
        return model.id;
      },),).size,).toBe(models.length,);
    }, },),
    ...['{', JSON.stringify({ providers: { [CODEX_PROVIDER]: { models: [{ id: 'invalid-config', contextWindow: 'wrong-type', },], }, }, },),]
      .map(function invalidConfigScenario(invalid, index) {
        return it({ name: `rejects invalid disposable configuration ${index} with a remediation error`, fn: async function verifyInvalidConfig() {
          /** Invalid fixture configuration must never touch the user's model settings. */
          await using home = await fixtureHome();
          await writeFile(home.modelsPath, invalid,);
          /** Actual loader rejection must retain the corrective configuration diagnostic. */
          const error = await caughtFailure(async function loadInvalidCatalog() {
            return await loadCodexProvider({ modelsPath: home.modelsPath, },);
          },);
          expect(error,).toBeInstanceOf(FastModelError,);
          expect(error,).toHaveProperty('message',);
          expect(error,).toSatisfy(function remediationDiagnostic(value) {
            return (value instanceof FastModelError) && value.message.includes('Correct the model configuration',);
          },);
          expect(await readFile(home.modelsPath, 'utf8',),).toBe(invalid,);
        }, },);
      },),
  ], },),
], },);

//endregion Bootstrap
