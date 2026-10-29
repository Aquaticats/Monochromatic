/**
 Unit tests for exact model matching.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  findExactModelReferenceMatch,
  NO_EXACT_MATCH,
} from '../dist/final/node/index.mjs';
import { fixtureModel, } from './test-fixtures.ts';

//region Fixtures

/** First exact-match fixture. */
const openaiModel = fixtureModel({
  provider: 'openai',
  id: 'gpt-5.5',
  name: 'GPT 5.5',
},);

/** Second exact-match fixture. */
const anthropicModel = fixtureModel({
  provider: 'anthropic',
  id: 'claude-sonnet-4-5',
  name: 'Claude Sonnet',
},);

/** Models available to exact-match tests. */
const availableModels = [
  openaiModel,
  anthropicModel,
] as const;

//endregion Fixtures

await describe({
  name: findExactModelReferenceMatch.name,
  children: [
    it({
      name: 'matches canonical and provider/model references case-insensitively',
      fn: async function testCanonicalReference() {
        expect(findExactModelReferenceMatch({
          modelReference: 'OPENAI/GPT-5.5',
          availableModels,
        },),)
          .toBe(openaiModel,);
      },
    },),
    it({
      name: 'matches unique bare ids',
      fn: async function testBareIdReference() {
        expect(findExactModelReferenceMatch({
          modelReference: 'claude-sonnet-4-5',
          availableModels,
        },),)
          .toBe(anthropicModel,);
      },
    },),
    it({
      name: 'returns NO_EXACT_MATCH for empty, unknown, and ambiguous references',
      fn: async function testAbsentReferences() {
        expect(findExactModelReferenceMatch({
          modelReference: '',
          availableModels,
        },),)
          .toBe(NO_EXACT_MATCH,);
        expect(findExactModelReferenceMatch({
          modelReference: 'missing',
          availableModels,
        },),)
          .toBe(NO_EXACT_MATCH,);
        expect(findExactModelReferenceMatch({
          modelReference: 'gpt-5.5',
          availableModels: [
            openaiModel,
            fixtureModel({ provider: 'azure', id: 'gpt-5.5', },),
          ],
        },),)
          .toBe(NO_EXACT_MATCH,);
      },
    },),
  ],
},);
