/**
 Hidden adapter availability and live native catalog lifecycle.
 
 @module
 */
import type { Api, AnyModel, Credential, Model, Provider, } from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { createPriorityProvider, priorityTarget, } from '../dist/final/node/index.mjs';
import { caughtFailure, fixtureModel, } from './host-fixture-model.ts';
import { fixtureOverlay, fixtureRefresh, } from './host-fixture-overlay.ts';
import { fixtureProvider, } from './host-fixture-provider.ts';

//region Catalog: only physical chat targets, never mixed native operations.

await describe({ name: '', children: [
  describe({ name: 'keyless availability and live catalog', children: [
    ...[false, true,].map(function allModelsScenario(allModels) {
      return it({ name: `lists targets only with source getAllModels ${String(allModels,)}`, fn: async function verifyTargetAvailability() {
        /** Native source varies whether it supplies a mixed-model accessor. */
        const source = fixtureProvider({ allModels, },);
        /** Built overlay derives only physical routing targets. */
        const { overlay, } = fixtureOverlay(source.provider,);
        /** Expected targets preserve the complete original native chat catalog. */
        const targets = source.provider.getModels().map(function targetForModel(model: ForeignBorrowed<Model<Api>>) {
          return priorityTarget(model,);
        },);
        expect(overlay.getModels(),).toEqual(targets,);
        expect(overlay.getAllModels?.(),).toEqual(targets,);
        expect(overlay.filterModels?.(targets, undefined,),).toEqual([],);
        expect(overlay.filterAllModels?.(targets, undefined,),).toEqual([],);
        await Promise.resolve();
      }, },);
    },),
    it({ name: 'ignores source virtual, image, and classifier entries without applying or copying native availability policy', fn: async function verifyChatOnlyCatalog() {
      /** Synthetic native source exposes mixed operation and availability controls. */
      const source = fixtureProvider();
      /** Genuine native chat base is the only valid physical target source. */
      const base = fixtureModel();
      /** Existing source virtual entry must not become another physical base. */
      const virtual = { ...fixtureModel({ id: 'existing-native-virtual', },), api: 'pi-virtual', };
      /** Dedicated image model must stay only in the original mixed catalog. */
      const image: AnyModel = { ...base, type: 'image', api: 'openrouter-images', output: ['image',], };
      /** Dedicated classifier must stay only in the original mixed catalog. */
      const classifier: AnyModel = { ...base, type: 'classifier', api: 'typesafe-system-one', };
      /** Original availability filters must never be invoked by adapter reads. */
      const filterState = { filtered: 0, };
      /** Source callbacks mirror the actual foreign mutable model boundaries. */
      const provider: Provider = { ...source.provider,
        getModels: function getModels() {
          return [base, virtual,];
        },
        getAllModels: function getAllModels() {
          return [base, virtual, image, classifier,];
        },
        filterModels: function filterModels(models: ForeignBorrowed<readonly Model<Api>[]>) {
          filterState.filtered += 1;
          return models;
        },
        filterAllModels: function filterAllModels(models: ForeignBorrowed<readonly AnyModel[]>) {
          filterState.filtered += 1;
          return models;
        }, };
      /** Synthetic credential makes accidental copying of native policy observable. */
      const credential: Credential = { type: 'api_key', key: 'synthetic-filter-key', };
      /** Catalog observations must contain only the untouched native chat base. */
      const { overlay, catalogs, } = fixtureOverlay(provider,);
      expect(overlay.getModels(),).toEqual([priorityTarget(base,),],);
      expect(catalogs.at(-1),).toEqual([base,],);
      expect(overlay.getAllModels?.(),).toEqual([priorityTarget(base,),],);
      expect(overlay.filterModels?.([base, virtual,], credential,),).toEqual([],);
      expect(overlay.filterAllModels?.([base, image, classifier,], credential,),).toEqual([],);
      expect(filterState.filtered,).toBe(0,);
      expect(provider.getAllModels?.(),).toEqual([base, virtual, image, classifier,],);
      expect(provider.auth,).toBe(source.provider.auth,);
      await Promise.resolve();
    }, },),
    it({ name: 'uses the live source callback for addition, capability changes, and removal without all-registry enumeration', fn: async function verifyLiveCatalog() {
      /** Bootstrap source remains distinct from the current native provider. */
      const bootstrap = fixtureProvider();
      /** Live metadata may differ from bootstrap without another provider refresh. */
      const live = fixtureProvider({ models: [fixtureModel({ id: 'live-model', contextWindow: 654_321, },),], },);
      /** Original catalog publications are retained to detect every source update. */
      const catalogs: (readonly unknown[])[] = [];
      /** Native source reads must occur once per adapter catalog read. */
      const catalogState = { reads: 0, };
      /** Built adapter reads only the live source, never the whole registry. */
      const adapter = createPriorityProvider({ provider: bootstrap.provider,
        isConfigured: function isConfigured() { return Promise.resolve(true,); },
        getProvider: function getProvider() { catalogState.reads += 1;
          return live.provider; },
        lookup: function lookup(id) { return live.state.models.find(function matchesModel(model: ForeignBorrowed<Model<Api>>) {
          return model.id === id;
        },); },
        dispatch: function dispatch() {
          throw new Error('Catalog read must not dispatch.',);
        },
        onCatalog: function onCatalog(models) {
          catalogs.push(models,);
        }, },);
      expect(adapter.getModels(),).toEqual(live.state.models.map(function currentTarget(model: ForeignBorrowed<Model<Api>>) {
        return priorityTarget(model,);
      },),);
      live.state.models = [fixtureModel({ id: 'new-live-model', contextWindow: 987_654, },),];
      expect(adapter.getAllModels?.(),).toEqual(live.state.models.map(function refreshedTarget(model: ForeignBorrowed<Model<Api>>) {
        return priorityTarget(model,);
      },),);
      expect(catalogState.reads,).toBe(2,);
      expect(catalogs,).toEqual([[fixtureModel({ id: 'live-model', contextWindow: 654_321, },),], live.state.models,],);
      expect(bootstrap.state.refreshes,).toBe(0,);
      expect(live.state.refreshes,).toBe(0,);
      await Promise.resolve();
    }, },),
  ], },),
  //endregion Catalog

  //region Refresh: synchronize source metadata without refreshing another provider ID.
  describe({ name: 'adapter refresh lifecycle', children: [
    ...[false, true,].map(function dynamicScenario(dynamic) {
      return it({ name: `synchronizes a source with refresh support ${String(dynamic,)} without invoking its refresh`, fn: async function verifySourceSynchronization() {
        /** Native refresh capability is independently varied by this scenario. */
        const source = fixtureProvider({ dynamic, },);
        /** Catalog observations detect synchronization without source refresh. */
        const { overlay, catalogs, } = fixtureOverlay(source.provider,);
        source.state.models = [fixtureModel({ id: 'new-model', },),];
        await overlay.refreshModels?.(fixtureRefresh(),);
        expect(source.state.refreshes,).toBe(0,);
        expect(catalogs,).toEqual([source.state.models,],);
      }, },);
    },),
    it({ name: 'does not read or synchronize its source after cancellation', fn: async function verifyCancelledRefresh() {
      /** Native provider observations must remain untouched after cancellation. */
      const source = fixtureProvider();
      /** Captured catalog callbacks prove no cancelled synchronization occurs. */
      const { overlay, catalogs, } = fixtureOverlay(source.provider,);
      /** Test-owned controller is cancelled before refresh begins. */
      const controller = new AbortController();
      controller.abort();
      await overlay.refreshModels?.(fixtureRefresh(controller.signal,),);
      expect(catalogs,).toEqual([],);
      expect(source.state.refreshes,).toBe(0,);
    }, },),
    it({ name: 'propagates source catalog read failure without fallback dispatch', fn: async function verifyCatalogFailure() {
      /** Native request history must remain empty after failed catalog reading. */
      const source = fixtureProvider();
      /** Distinct failure value proves rejection identity is preserved. */
      const failure = new Error('Synthetic catalog read failure.',);
      /** Failing foreign provider accessor replaces only this fixture's source. */
      const { overlay, catalogs, } = fixtureOverlay({ ...source.provider, getModels: function getModels() {
        throw failure;
      }, },);
      expect(await caughtFailure(async function refreshFailedCatalog() {
        await overlay.refreshModels?.(fixtureRefresh(),);
      },),).toBe(failure,);
      expect(catalogs,).toEqual([],);
      expect(source.state.calls,).toEqual([],);
    }, },),
  ], },),
  //endregion Refresh
], },);
