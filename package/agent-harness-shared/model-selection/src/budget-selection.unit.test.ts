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
  NO_AUTH,
  NoBudgetModelError,
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
    async resolveAuth({ model, },) {
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
  name: selectBudgetModel.name,
  children: [
    it({
      name: 'selects same-provider candidate with auth',
      fn: async function testSameProviderBudgetSelection() {
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
      fn: async function testAnyProviderBudgetSelection() {
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
      fn: async function testSameProviderSelectionWithoutCostRatioRejection() {
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
      fn: async function testSameProviderSpeedSelection() {
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
      fn: async function testAnyProviderSpeedSelection() {
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
      fn: async function testBudgetSelectionErrors() {
        const noAuth = await captureAsyncError(async function selectWithoutAuth() {
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
