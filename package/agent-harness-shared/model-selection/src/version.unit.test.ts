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
          fn: async function testExtractMajorVersion(): Promise<void> {
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
          fn: async function testExtractVersionNumbers(): Promise<void> {
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
          fn: async function testCompareVersions(): Promise<void> {
            expect(compareVersions({
              a: fixtureModel({ provider: 'openai', id: 'gpt-4o', },),
              b: fixtureModel({ provider: 'openai', id: 'gpt-5', },),
            },),)
              .toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'compares every position of vectors that differ in the second number',
          fn: async function testCompareVersionPositions(): Promise<void> {
            expect(compareVersions({
              a: fixtureModel({ provider: 'openai', id: 'gpt-4.1', },),
              b: fixtureModel({ provider: 'openai', id: 'gpt-4.2', },),
            },),)
              .toBeGreaterThan(0,);
            expect(compareVersions({
              a: fixtureModel({ provider: 'openai', id: 'gpt-4.2', },),
              b: fixtureModel({ provider: 'openai', id: 'gpt-4.2', },),
            },),)
              .toBe(0,);
          },
        },),
        it({
          name: 'sorts a shorter vector ahead when the extra positions are zero',
          fn: async function testCompareVersionLengths(): Promise<void> {
            expect(compareVersions({
              a: fixtureModel({ provider: 'openai', id: 'gpt-4', },),
              b: fixtureModel({ provider: 'openai', id: 'gpt-4.0', },),
            },),)
              .toBeLessThan(0,);
            expect(compareVersions({
              a: fixtureModel({ provider: 'openai', id: 'gpt-4.0', },),
              b: fixtureModel({ provider: 'openai', id: 'gpt-4', },),
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
          fn: async function testFindCheapestMajorVersions(): Promise<void> {
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
