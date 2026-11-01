/**
 Unit tests for version helpers.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  compareVersions,
  extractMajorVersion,
  extractVersionNumbers,
  findCheapestInMajorVersions,
  NO_MAJOR_VERSION,
} from '../dist/final/node/index.mjs';
import { fixtureModel, } from './test-fixtures.ts';

await describe({
  name: '',
  children: [
    describe({
      name: extractMajorVersion.name,
      children: [
        it({
          name: 'extracts first non-date major version',
          fn: async function testExtractMajorVersion() {
            expect(extractMajorVersion('gpt-4o-mini',),).toBe(4,);
            expect(extractMajorVersion('model-20240101-v2',),).toBe(2,);
            expect(extractMajorVersion('embedding-model',),).toBe(NO_MAJOR_VERSION,);
          },
        },),
      ],
    },),
    describe({
      name: extractVersionNumbers.name,
      children: [
        it({
          name: 'extracts version vectors and skips dates',
          fn: async function testExtractVersionNumbers() {
            expect(extractVersionNumbers('claude-3.5-sonnet',),).toEqual([3, 5,],);
            expect(extractVersionNumbers('model-20240101-v2',),).toEqual([2,],);
          },
        },),
      ],
    },),
    describe({
      name: compareVersions.name,
      children: [
        it({
          name: 'orders higher versions before lower versions',
          fn: async function testCompareVersions() {
            expect(compareVersions({
              a: fixtureModel({ provider: 'openai', id: 'gpt-4o', },),
              b: fixtureModel({ provider: 'openai', id: 'gpt-5', },),
            },),)
              .toBeGreaterThan(0,);
          },
        },),
      ],
    },),
    describe({
      name: findCheapestInMajorVersions.name,
      children: [
        it({
          name: 'filters newest major versions and sorts by cost',
          fn: async function testFindCheapestMajorVersions() {
            const models = [
              fixtureModel({ provider: 'openai', id: 'gpt-3.5-turbo', inputCost: 1, },),
              fixtureModel({ provider: 'openai', id: 'gpt-4o', inputCost: 2, },),
              fixtureModel({ provider: 'openai', id: 'gpt-4o-mini', inputCost: 0.1, },),
            ];
            const result = findCheapestInMajorVersions({
              models,
              majorVersions: 1,
            },);
            expect(result.map(function mapModel(model,) {
              return model.id;
            },),)
              .toEqual([
                'gpt-4o-mini',
                'gpt-4o',
              ],);
          },
        },),
      ],
    },),
  ],
},);
