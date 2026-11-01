/**
 Unit tests for scope pattern resolution.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { resolveModelPatterns, } from '../dist/final/node/index.mjs';
import { fixtureModel, } from './test-fixtures.ts';

//region Fixtures

/** Cheap model fixture. */
const cheapModel = fixtureModel({
  provider: 'cheap',
  id: 'reviewer',
},);

/** Expensive model fixture. */
const expensiveModel = fixtureModel({
  provider: 'expensive',
  id: 'reviewer',
},);

/** Model fixture whose id contains a literal glob token. */
const literalStarModel = fixtureModel({
  provider: 'cheap',
  id: 'review*',
},);

/** Model list used by pattern resolution tests. */
const availableModels = [
  cheapModel,
  expensiveModel,
] as const;

//endregion Fixtures

await describe({
  name: resolveModelPatterns.name,
  children: [
    it({
      name: 'resolves glob and literal patterns in order',
      fn: async function testPatternOrder() {
        const result = resolveModelPatterns({
          patterns: [
            'expensive/*:high',
            'cheap/reviewer',
          ],
          availableModels,
        },);
        expect(result.map(function mapEntry(entry,) {
          return entry.canonicalSlug;
        },),)
          .toEqual([
            'expensive/reviewer',
            'cheap/reviewer',
          ],);
        expect(result[0]?.thinkingLevel,).toBe('high',);
      },
    },),
    it({
      name: 'deduplicates provider/id matches',
      fn: async function testDeduplication() {
        const result = resolveModelPatterns({
          patterns: [
            'cheap/*',
            'cheap/reviewer',
          ],
          availableModels,
        },);
        expect(result,).toHaveLength(1,);
      },
    },),
    it({
      name: 'matches glob patterns case-insensitively against slugs and ids',
      fn: async function testCaseInsensitiveGlobMatching() {
        /**
         Result matched through canonical provider/model slug.
         */
        const canonicalResult = resolveModelPatterns({
          patterns: ['EXPENSIVE/REVIEW*',],
          availableModels,
        },);
        expect(canonicalResult.map(function mapCanonicalEntry(entry,) {
          return entry.canonicalSlug;
        },),)
          .toEqual(['expensive/reviewer',],);

        /**
         Result matched through bare model id fallback.
         */
        const bareIdResult = resolveModelPatterns({
          patterns: ['REVIEW*',],
          availableModels,
        },);
        expect(bareIdResult.map(function mapBareIdEntry(entry,) {
          return entry.canonicalSlug;
        },),)
          .toEqual([
            'cheap/reviewer',
            'expensive/reviewer',
          ],);
      },
    },),
    it({
      name: 'keeps expected glob syntax for model scope patterns',
      fn: async function testExpectedGlobSyntax() {
        /**
         Result matched through brace alternation on provider segment.
         */
        const braceResult = resolveModelPatterns({
          patterns: ['{cheap,expensive}/*',],
          availableModels,
        },);
        expect(braceResult.map(function mapBraceEntry(entry,) {
          return entry.canonicalSlug;
        },),)
          .toEqual([
            'cheap/reviewer',
            'expensive/reviewer',
          ],);

        /**
         Result matched through character class syntax.
         */
        const characterClassResult = resolveModelPatterns({
          patterns: ['cheap/reviewe[!x]',],
          availableModels,
        },);
        expect(characterClassResult.map(function mapCharacterClassEntry(entry,) {
          return entry.canonicalSlug;
        },),)
          .toEqual(['cheap/reviewer',],);

        /**
         Result matched through globstar across provider/id separator.
         */
        const globstarResult = resolveModelPatterns({
          patterns: ['cheap/**',],
          availableModels,
        },);
        expect(globstarResult.map(function mapGlobstarEntry(entry,) {
          return entry.canonicalSlug;
        },),)
          .toEqual(['cheap/reviewer',],);

        /**
         Result proving single-star globs do not cross provider/id separator.
         */
        const slashBoundaryResult = resolveModelPatterns({
          patterns: ['cheap*',],
          availableModels,
        },);
        expect(slashBoundaryResult,).toEqual([],);

        /**
         Result proving escaped glob tokens are treated literally.
         */
        const escapedStarResult = resolveModelPatterns({
          patterns: [String.raw`cheap/review\*`,],
          availableModels: [
            cheapModel,
            literalStarModel,
          ],
        },);
        expect(escapedStarResult.map(function mapEscapedStarEntry(entry,) {
          return entry.canonicalSlug;
        },),)
          .toEqual(['cheap/review*',],);
      },
    },),
  ],
},);
