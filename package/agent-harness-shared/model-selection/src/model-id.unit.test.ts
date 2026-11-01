/**
 Unit tests for model-id helpers.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  allowedSlugs,
  canonicalSlug,
  getModelIdLeaf,
  MALFORMED_SLUG,
  parseProviderModelSlug,
  resolveRequestedModel,
  type EffectiveModelScope,
} from '../dist/final/node/index.mjs';
import {
  captureError,
  fixtureModel,
} from './test-fixtures.ts';

//region Fixtures

/** Cheap fixture model. */
const cheapModel = fixtureModel({
  provider: 'cheap',
  id: 'reviewer',
  name: 'Reviewer Cheap',
},);

/** Expensive fixture model sharing a bare id with cheapModel. */
const expensiveModel = fixtureModel({
  provider: 'expensive',
  id: 'reviewer',
  name: 'Reviewer Expensive',
},);

/** Registry-only model used for out-of-scope checks. */
const thirdModel = fixtureModel({
  provider: 'third',
  id: 'reviewer',
},);

/** Effective scope fixture. */
const scope: EffectiveModelScope = {
  source: 'available',
  entries: [
    {
      model: cheapModel,
      canonicalSlug: 'cheap/reviewer',
    },
    {
      model: expensiveModel,
      canonicalSlug: 'expensive/reviewer',
    },
  ],
};

/** Registry fixture exposing scoped plus out-of-scope models. */
const modelRegistry = {
  getAll() {
    return [
      cheapModel,
      expensiveModel,
      thirdModel,
    ];
  },
};

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: canonicalSlug.name,
      children: [
        it({
          name: 'formats provider and id',
          fn: async function testCanonicalSlug() {
            expect(canonicalSlug(cheapModel,),).toBe('cheap/reviewer',);
          },
        },),
      ],
    },),
    describe({
      name: parseProviderModelSlug.name,
      children: [
        it({
          name: 'parses provider/model slugs',
          fn: async function testParseProviderSlug() {
            expect(parseProviderModelSlug('openai/gpt-5.5',),).toEqual({
              provider: 'openai',
              modelId: 'gpt-5.5',
            },);
          },
        },),
        it({
          name: 'returns MALFORMED_SLUG for malformed slugs',
          fn: async function testMalformedProviderSlug() {
            expect(parseProviderModelSlug('gpt-5.5',),).toBe(MALFORMED_SLUG,);
          },
        },),
      ],
    },),
    describe({
      name: getModelIdLeaf.name,
      children: [
        it({
          name: 'returns final slash-delimited segment',
          fn: async function testModelIdLeaf() {
            expect(getModelIdLeaf({ modelId: 'synthetic/hf:org/model', },),).toBe('model',);
          },
        },),
      ],
    },),
    describe({
      name: allowedSlugs.name,
      children: [
        it({
          name: 'formats scoped slugs and empty scopes',
          fn: async function testAllowedSlugs() {
            expect(allowedSlugs(scope,),).toBe('cheap/reviewer, expensive/reviewer',);
            expect(allowedSlugs({ source: 'available', entries: [], },),).toBe('none',);
          },
        },),
      ],
    },),
    describe({
      name: resolveRequestedModel.name,
      children: [
        it({
          name: 'accepts canonical scoped slug',
          fn: async function testCanonicalResolution() {
            const result = resolveRequestedModel({
              scope,
              requestedSlug: 'expensive/reviewer',
              modelRegistry,
              errorPrefix: 'advisor',
            },);
            expect(result.selected.canonicalSlug,).toBe('expensive/reviewer',);
          },
        },),
        it({
          name: 'rejects ambiguous bare id',
          fn: async function testAmbiguousBareId() {
            const error = captureError(function resolveAmbiguousBareId() {
              return resolveRequestedModel({
                scope,
                requestedSlug: 'reviewer',
                modelRegistry,
                errorPrefix: 'advisor',
              },);
            },);
            expect(error,).toBeInstanceOf(Error,);
            expect((error as Error).message,).toContain('ambiguous in scoped models',);
          },
        },),
        it({
          name: 'distinguishes out-of-scope and unknown slugs',
          fn: async function testOutOfScopeAndUnknown() {
            const outOfScope = captureError(function resolveOutOfScope() {
              return resolveRequestedModel({
                scope,
                requestedSlug: 'third/reviewer',
                modelRegistry,
                errorPrefix: 'advisor',
              },);
            },);
            const unknown = captureError(function resolveUnknown() {
              return resolveRequestedModel({
                scope,
                requestedSlug: 'missing/reviewer',
                modelRegistry,
                errorPrefix: 'advisor',
              },);
            },);
            expect((outOfScope as Error).message,).toContain('is not in scoped models',);
            expect((unknown as Error).message,).toContain('was not found in scoped models',);
          },
        },),
      ],
    },),
  ],
},);
