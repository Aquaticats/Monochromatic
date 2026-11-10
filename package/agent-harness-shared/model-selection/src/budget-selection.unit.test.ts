/**
 Unit tests for budget-model strategy selection.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type BudgetModel,
  findFastestCandidate,
  firstAuthenticatedCandidate,
  NO_AUTH,
  NO_CANDIDATE,
  NoBudgetModelError,
  scoreModelSpeed,
  selectBudgetModel,
  type BudgetModelAuth,
} from '../dist/final/node/index.mjs';
import {
  captureAsyncError,
  fixtureModel,
  fixtureSlug,
} from './test-fixtures.ts';

//region Fixtures

/** Same-provider near-active input token price. */
const NEAR_ACTIVE_INPUT_COST = 9;

/** Active model fixture. */
const activeModel = fixtureModel({
  provider: 'openai',
  id: 'gpt-4o',
  inputCost: 10,
},);

/** Same-provider budget model fixture. */
const sameProviderModel = fixtureModel({
  provider: 'openai',
  id: 'gpt-4o-mini',
  inputCost: 1,
},);

/** Same-provider model close to active-model price. */
const nearActiveCostModel = fixtureModel({
  provider: 'openai',
  id: 'gpt-4o-near-active',
  inputCost: NEAR_ACTIVE_INPUT_COST,
},);

/** Same-provider model with explicit speed signal and higher price. */
const sameProviderHighspeedModel = fixtureModel({
  provider: 'openai',
  id: 'gpt-4o-highspeed',
  inputCost: NEAR_ACTIVE_INPUT_COST,
},);

/** Any-provider budget model fixture. */
const anyProviderModel = fixtureModel({
  provider: 'anthropic',
  id: 'claude-4-haiku',
  inputCost: 0.25,
},);

/** Any-provider model with explicit speed signal and higher price. */
const anyProviderHighspeedModel = fixtureModel({
  provider: 'moonshotai',
  id: 'kimi-k2.7-code-highspeed',
  inputCost: NEAR_ACTIVE_INPUT_COST,
},);

/** Model id with no version token, which major-version filtering drops. */
const versionlessModel = fixtureModel({
  provider: 'openai',
  id: 'faux-model',
},);

/** All budget test models. */
const allModels = [
  activeModel,
  sameProviderModel,
  anyProviderModel,
] as const;

/** All budget test models plus explicit speed fixtures. */
const allModelsWithSpeed = [
  activeModel,
  sameProviderModel,
  sameProviderHighspeedModel,
  anyProviderModel,
  anyProviderHighspeedModel,
] as const;

/** Auth fixture. */
const auth: BudgetModelAuth = { apiKey: 'test-key', };

/**
 Build authenticated slug callbacks for budget tests.
 
 @param authenticatedSlugs - slugs that should resolve auth
 
 @returns auth callbacks for shared budget selection
 */
function authCallbacks(
  authenticatedSlugs: readonly string[],
): Pick<Parameters<typeof selectBudgetModel>[0], 'resolveAuth' | 'hasConfiguredAuth'> {
  /** Authenticated slug set. */
  const slugs = new Set(authenticatedSlugs,);
  return {
    async resolveAuth({ model, },): Promise<BudgetModelAuth | typeof NO_AUTH> {
      return slugs.has(fixtureSlug(model,),)
        ? auth
        : NO_AUTH;
    },
    hasConfiguredAuth({ model, },) {
      return slugs.has(fixtureSlug(model,),);
    },
  };
}

//endregion Fixtures

await describe({
  name: firstAuthenticatedCandidate.name,
  children: [
    it({
      name: 'walks ranked candidates in order and stops at the first authenticated one',
      fn: async function testFirstAuthenticated(): Promise<void> {
        /**
         Slugs the walk asked the resolver about, in order.
         */
        const resolvedSlugs: string[] = [];
        const selected = await firstAuthenticatedCandidate({
          candidates: [
            activeModel,
            sameProviderModel,
            anyProviderModel,
          ],
          resolveAuth: async function resolveLaterCandidate({ model, },): Promise<BudgetModelAuth | typeof NO_AUTH> {
            resolvedSlugs.push(fixtureSlug(model,),);
            return model === anyProviderModel
              ? auth
              : NO_AUTH;
          },
        },);
        expect(selected,).toEqual({
          model: anyProviderModel,
          auth,
        },);
        expect(resolvedSlugs,).toEqual([
          fixtureSlug(activeModel,),
          fixtureSlug(sameProviderModel,),
          fixtureSlug(anyProviderModel,),
        ],);
      },
    },),
    it({
      name: 'returns NO_AUTH when no candidate has configured auth',
      fn: async function testNoAuthenticatedCandidate(): Promise<void> {
        const selected = await firstAuthenticatedCandidate({
          candidates: [
            activeModel,
            sameProviderModel,
          ],
          resolveAuth: async function resolveNoCandidate(): Promise<typeof NO_AUTH> {
            return NO_AUTH;
          },
        },);
        expect(selected,).toBe(NO_AUTH,);
      },
    },),
  ],
},);

await describe({
  name: selectBudgetModel.name,
  children: [
    it({
      name: 'selects same-provider candidate with auth',
      fn: async function testSameProviderBudgetSelection(): Promise<void> {
        const selected = await selectBudgetModel({
          activeModel,
          allModels,
          strategy: 'same-provider',
          majorVersions: 1,
          ...authCallbacks([fixtureSlug(sameProviderModel,),],),
        },);
        expect(selected.model,).toBe(sameProviderModel,);
      },
    },),
    it({
      name: 'selects any-provider candidate with auth',
      fn: async function testAnyProviderBudgetSelection(): Promise<void> {
        const selected = await selectBudgetModel({
          activeModel,
          allModels,
          strategy: 'any-provider',
          majorVersions: 1,
          ...authCallbacks([fixtureSlug(anyProviderModel,),],),
        },);
        expect(selected.model,).toBe(anyProviderModel,);
      },
    },),
    it({
      name: 'selects near-active-cost same-provider candidate',
      fn: async function testSameProviderSelectionWithoutCostRatioRejection(): Promise<void> {
        const selected = await selectBudgetModel({
          activeModel,
          allModels: [
            activeModel,
            nearActiveCostModel,
          ],
          strategy: 'same-provider',
          majorVersions: 1,
          ...authCallbacks([fixtureSlug(nearActiveCostModel,),],),
        },);
        expect(selected.model,).toBe(nearActiveCostModel,);
      },
    },),
    it({
      name: 'selects speed-named same-provider candidate before cheaper model',
      fn: async function testSameProviderSpeedSelection(): Promise<void> {
        const selected = await selectBudgetModel({
          activeModel,
          allModels: allModelsWithSpeed,
          strategy: 'same-provider',
          majorVersions: 1,
          ...authCallbacks([
            fixtureSlug(sameProviderModel,),
            fixtureSlug(sameProviderHighspeedModel,),
          ],),
        },);
        expect(selected.model,).toBe(sameProviderHighspeedModel,);
      },
    },),
    it({
      name: 'selects speed-named any-provider candidate before cheaper model',
      fn: async function testAnyProviderSpeedSelection(): Promise<void> {
        const selected = await selectBudgetModel({
          activeModel,
          allModels: allModelsWithSpeed,
          strategy: 'any-provider',
          majorVersions: 1,
          ...authCallbacks([
            fixtureSlug(anyProviderModel,),
            fixtureSlug(anyProviderHighspeedModel,),
          ],),
        },);
        expect(selected.model,).toBe(anyProviderHighspeedModel,);
      },
    },),
    it({
      name: 'throws no-auth error shape',
      fn: async function testBudgetSelectionErrors(): Promise<void> {
        const noAuth = await captureAsyncError(async function selectWithoutAuth(): Promise<BudgetModel> {
          return await selectBudgetModel({
            activeModel,
            allModels,
            strategy: 'same-provider',
            majorVersions: 1,
            ...authCallbacks([],),
          },);
        },);
        expect(noAuth,).toBeInstanceOf(NoBudgetModelError,);
        expect((noAuth as Error).message,).toContain('no API key available',);
      },
    },),
  ],
},);

await describe({
  name: findFastestCandidate.name,
  children: [
    it({
      name: 'returns NO_CANDIDATE when the registry has no model',
      fn: async function testEmptyRegistry(): Promise<void> {
        /**
         Models the auth predicate was asked about.
         */
        const askedModels: unknown[] = [];
        /**
         Sentinel result for an empty registry.
         */
        const candidate = findFastestCandidate({
          allModels: [],
          majorVersions: 1,
          hasConfiguredAuth: function reportAuthForEmptyRegistry({ model, },): boolean {
            askedModels.push(model,);
            return true;
          },
        },);
        expect(candidate,).toBe(NO_CANDIDATE,);
        expect(askedModels,).toEqual([],);
      },
    },),
    it({
      name: 'returns NO_CANDIDATE when major-version filtering drops every model',
      fn: async function testVersionlessRegistry(): Promise<void> {
        /**
         Models the auth predicate was asked about.
         */
        const askedModels: unknown[] = [];
        /**
         Sentinel result for a registry whose ids carry no version token.
         */
        const candidate = findFastestCandidate({
          allModels: [versionlessModel,],
          majorVersions: 1,
          hasConfiguredAuth: function reportAuthForVersionlessRegistry({ model, },): boolean {
            askedModels.push(model,);
            return true;
          },
        },);
        expect(candidate,).toBe(NO_CANDIDATE,);
        expect(askedModels,).toEqual([],);
      },
    },),
    it({
      name: 'returns the fastest provider head with its configured-auth flag',
      fn: async function testFastestProviderHead(): Promise<void> {
        /**
         Models the auth predicate was asked about.
         */
        const askedModels: unknown[] = [];
        /**
         Fastest cross-provider candidate metadata.
         */
        const candidate = findFastestCandidate({
          allModels,
          majorVersions: 1,
          hasConfiguredAuth: function reportConfiguredAuth({ model, },): boolean {
            askedModels.push(model,);
            return true;
          },
        },);
        expect(candidate,).not.toBe(NO_CANDIDATE,);
        expect(candidate,).toEqual({
          provider: sameProviderModel.provider,
          modelId: sameProviderModel.id,
          speedScore: scoreModelSpeed(sameProviderModel,),
          costInput: sameProviderModel.cost.input,
          costOutput: sameProviderModel.cost.output,
          hasApiKey: true,
        },);
        expect(askedModels,).toEqual([sameProviderModel,],);
      },
    },),
  ],
},);
