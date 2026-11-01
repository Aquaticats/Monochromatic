/**
 Unit tests for argv scope parsing.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  NO_ARGV_MODELS,
  parseArgvModelPatterns,
} from '../dist/final/node/index.mjs';

await describe({
  name: parseArgvModelPatterns.name,
  children: [
    it({
      name: 'parses separated and inline --models arguments',
      fn: async function testArgvModels() {
        expect(parseArgvModelPatterns({
          argv: [
            'pi',
            '--models',
            'cheap/*, expensive/reviewer ',
          ],
        },),)
          .toEqual([
            'cheap/*',
            'expensive/reviewer',
          ],);
        expect(parseArgvModelPatterns({ argv: ['--models=cheap/*',], },),)
          .toEqual(['cheap/*',],);
      },
    },),
    it({
      name: 'returns NO_ARGV_MODELS when --models is absent',
      fn: async function testAbsentArgvModels() {
        expect(parseArgvModelPatterns({ argv: ['pi',], },),).toBe(NO_ARGV_MODELS,);
      },
    },),
  ],
},);
