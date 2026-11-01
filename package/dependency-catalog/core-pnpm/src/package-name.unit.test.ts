/**
 Unit tests for the built npm package-name validator.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isValidPackageName,
} from '../dist/final/node/index.mjs';

/**
 Valid package names used by the workspace catalog.
 */
const VALID_NAMES: readonly string[] = [
  'oxlint',
  'opentype.js',
  'lezer-json5',
  '@anthropic-ai/sdk',
  '@types/node',
  '@total-typescript/ts-reset',
];

/**
 Crafted or malformed names that must never become catalog keys.
 */
const INVALID_NAMES: readonly string[] = [
  '__proto__',
  '',
  '@',
  '@scope',
  '@scope/',
  '@/name',
  'a/b/c',
  '.leading-dot',
  '_leading-underscore',
  'Upper',
  'has space',
];

await describe({
  name: isValidPackageName.name,
  children: [
    //region Valid names

    ...VALID_NAMES.map(function buildValidCase(name,): ReturnType<typeof it> {
      return it({
        name: `accepts ${name}`,
        fn: async () => {
          expect(isValidPackageName(name,),).toBe(true,);
        },
      },);
    },),

    //endregion Valid names

    //region Invalid names

    ...INVALID_NAMES.map(function buildInvalidCase(name,): ReturnType<typeof it> {
      return it({
        name: `rejects ${JSON.stringify(name,)}`,
        fn: async () => {
          expect(isValidPackageName(name,),).toBe(false,);
        },
      },);
    },),

    //endregion Invalid names
  ],
},);
