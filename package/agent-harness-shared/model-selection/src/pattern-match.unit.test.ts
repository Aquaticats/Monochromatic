/**
 Unit tests for model pattern parsing helpers.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  hasDateSuffix,
  isAlias,
  isThinkingLevel,
  parseModelPattern,
  patternHasGlob,
  splitThinkingSuffix,
} from '../dist/final/node/index.mjs';
import { fixtureModel, } from './test-fixtures.ts';

//region Fixtures

/** Alias model fixture preferred by fuzzy matching. */
const aliasModel = fixtureModel({
  provider: 'anthropic',
  id: 'claude-sonnet-latest',
  name: 'Claude Sonnet Latest',
},);

/** Dated model fixture used to test alias preference. */
const datedModel = fixtureModel({
  provider: 'anthropic',
  id: 'claude-sonnet-20251001',
  name: 'Claude Sonnet Snapshot',
},);

/** Models available to pattern tests. */
const availableModels = [
  datedModel,
  aliasModel,
] as const;

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: parseModelPattern.name,
      children: [
        it({
          name: 'matches fuzzy aliases and carries thinking suffixes',
          fn: async function testFuzzyAliasThinking() {
            const result = parseModelPattern({
              pattern: 'sonnet:high',
              availableModels,
            },);
            expect(result.model,).toBe(aliasModel,);
            expect(result.thinkingLevel,).toBe('high',);
          },
        },),
        it({
          name: 'strips invalid suffixes iteratively',
          fn: async function testInvalidSuffixStripping() {
            const result = parseModelPattern({
              pattern: 'sonnet:not-a-level',
              availableModels,
            },);
            expect(result.model,).toBe(aliasModel,);
            expect(result.thinkingLevel,).toBeUndefined();
          },
        },),
      ],
    },),
    describe({
      name: splitThinkingSuffix.name,
      children: [
        it({
          name: 'splits valid thinking suffixes only',
          fn: async function testSplitThinkingSuffix() {
            expect(splitThinkingSuffix('anthropic/*:xhigh',),).toEqual({
              pattern: 'anthropic/*',
              thinkingLevel: 'xhigh',
            },);
            expect(splitThinkingSuffix('anthropic/*:custom',),).toEqual({
              pattern: 'anthropic/*:custom',
            },);
          },
        },),
      ],
    },),
    describe({
      name: patternHasGlob.name,
      children: [
        it({
          name: 'detects glob tokens',
          fn: async function testPatternHasGlob() {
            expect(patternHasGlob('anthropic/*',),).toBe(true,);
            expect(patternHasGlob('anthropic/claude',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'low-level helpers',
      children: [
        it({
          name: 'detects thinking levels, aliases, and date suffixes',
          fn: async function testLowLevelHelpers() {
            expect(isThinkingLevel('minimal',),).toBe(true,);
            expect(isThinkingLevel('custom',),).toBe(false,);
            expect(hasDateSuffix('claude-20251001',),).toBe(true,);
            expect(hasDateSuffix('claude-2025x001',),).toBe(false,);
            expect(isAlias('claude-sonnet-latest',),).toBe(true,);
            expect(isAlias('claude-sonnet-20251001',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
