/**
 Unit tests for budget-model override resolution.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type BudgetModel,
  NO_AUTH,
  NO_OVERRIDE_MODEL,
  NoBudgetModelError,
  type ReadonlyModel,
  resolveBudgetModelOverride,
  type BudgetModelAuth,
} from '../dist/final/node/index.mjs';
import {
  captureAsyncError,
  fixtureModel,
} from './test-fixtures.ts';

//region Fixtures

/** Override model fixture. */
const overrideModel = fixtureModel({
  provider: 'openai',
  id: 'gpt-4o-mini',
},);

/** Override auth fixture. */
const auth: BudgetModelAuth = { apiKey: 'test-key', };

/**
 Find fixture model by provider/id.
 
 @param provider - provider slug
 
 @param modelId - model id
 
 @returns fixture model, or {@link NO_OVERRIDE_MODEL} when unmatched
 */
function findModel(
  {
    provider,
    modelId,
  }: {
    readonly provider: string;
    readonly modelId: string;
  },
): ReadonlyModel | typeof NO_OVERRIDE_MODEL {
  return (provider === overrideModel.provider) && (modelId === overrideModel.id)
    ? overrideModel
    : NO_OVERRIDE_MODEL;
}

/**
 Resolve fixture auth.
 
 @returns fixture auth
 */
async function resolveAuth(): Promise<BudgetModelAuth> {
  return auth;
}

//endregion Fixtures

await describe({
  name: resolveBudgetModelOverride.name,
  children: [
    it({
      name: 'resolves string override through lookup and auth callback',
      fn: async function testStringOverride(): Promise<void> {
        const selected = await resolveBudgetModelOverride({
          override: 'openai/gpt-4o-mini',
          findModel,
          resolveAuth,
        },);
        expect(selected.model,).toBe(overrideModel,);
        expect(selected.auth,).toBe(auth,);
      },
    },),
    it({
      name: 'uses inline auth for structured override',
      fn: async function testStructuredOverride(): Promise<void> {
        const selected = await resolveBudgetModelOverride({
          override: {
            model: 'openai/gpt-4o-mini',
            auth,
          },
          findModel,
          resolveAuth,
        },);
        expect(selected.auth,).toBe(auth,);
      },
    },),
    it({
      name: 'throws for malformed and missing overrides',
      fn: async function testOverrideErrors(): Promise<void> {
        const malformed = await captureAsyncError(async function resolveMalformedOverride(): Promise<BudgetModel> {
          return await resolveBudgetModelOverride({
            override: 'gpt-4o-mini',
            findModel,
            resolveAuth,
          },);
        },);
        const missing = await captureAsyncError(async function resolveMissingOverride(): Promise<BudgetModel> {
          return await resolveBudgetModelOverride({
            override: 'openai/missing',
            findModel,
            resolveAuth,
          },);
        },);
        expect(malformed,).toBeInstanceOf(NoBudgetModelError,);
        expect((malformed as Error).message,).toContain('not a provider/model slug',);
        expect(missing,).toBeInstanceOf(NoBudgetModelError,);
        expect((missing as Error).message,).toContain('not found in registry',);
      },
    },),
    it({
      name: 'throws when the override model has no configured auth',
      fn: async function testOverrideNoAuth(): Promise<void> {
        /**
         Override resolution failure for a registry model without auth.
         */
        const error = await captureAsyncError(async function resolveUnauthenticatedOverride(): Promise<BudgetModel> {
          return await resolveBudgetModelOverride({
            override: 'openai/gpt-4o-mini',
            findModel,
            resolveAuth: async function resolveNoAuth(): Promise<typeof NO_AUTH> {
              return NO_AUTH;
            },
          },);
        },);
        expect(error,).toBeInstanceOf(NoBudgetModelError,);
        expect((error as Error).message,).toContain('no API key for model override "openai/gpt-4o-mini"',);
      },
    },),
  ],
},);
