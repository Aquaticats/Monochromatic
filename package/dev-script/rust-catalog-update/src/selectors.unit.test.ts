/**
 Unit tests for pnpm-style exclude selectors.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  isExcluded,
  matchesPattern,
  parseSelector,
  pruneSelectors,
} from './selectors.ts';

await describe({
  name: parseSelector.name,
  children: [
    it({
      name: 'reads a bare name, a name with one version, and a version list with stray spaces',
      fn: async () => {
        expect(parseSelector(' ring ',),).toEqual({ pattern: 'ring', versions: [], },);
        expect(parseSelector('zbus@5.19.0',),).toEqual({ pattern: 'zbus', versions: ['5.19.0',], },);
        expect(parseSelector('webpack@ 4.47.0 ||5.102.1 || ',),).toEqual({ pattern: 'webpack', versions: ['4.47.0', '5.102.1',], },);
      },
    },),
  ],
},);

await describe({
  name: matchesPattern.name,
  children: [
    it({
      name: 'matches exact names, a lone star, prefix, suffix, and inner wildcards',
      fn: async () => {
        expect(matchesPattern({ pattern: 'tokio', name: 'tokio', },),).toBe(true,);
        expect(matchesPattern({ pattern: 'tokio', name: 'tokio-util', },),).toBe(false,);
        expect(matchesPattern({ pattern: '*', name: 'anything', },),).toBe(true,);
        expect(matchesPattern({ pattern: '**', name: 'x', },),).toBe(true,);
        expect(matchesPattern({ pattern: 'tokio*', name: 'tokio-util', },),).toBe(true,);
        expect(matchesPattern({ pattern: '*-sys', name: 'openssl-sys', },),).toBe(true,);
        expect(matchesPattern({ pattern: 'a*b*c', name: 'axxbyyc', },),).toBe(true,);
        expect(matchesPattern({ pattern: 'a*b*c', name: 'axxcyyb', },),).toBe(false,);
      },
    },),
    it({
      name: 'rejects names shorter than the fixed prefix and suffix together',
      fn: async () => {
        expect(matchesPattern({ pattern: 'ab*ba', name: 'aba', },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: isExcluded.name,
  children: [
    it({
      name: 'excludes every version for a name selector and only listed versions for a version selector',
      fn: async () => {
        expect(isExcluded({ selectors: ['ring',], name: 'ring', version: '0.17.8', },),).toBe(true,);
        expect(isExcluded({ selectors: ['zbus@5.19.0',], name: 'zbus', version: '5.19.0', },),).toBe(true,);
        expect(isExcluded({ selectors: ['zbus@5.19.0',], name: 'zbus', version: '5.19.1', },),).toBe(false,);
        expect(isExcluded({ selectors: [], name: 'zbus', version: '5.19.0', },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: pruneSelectors.name,
  children: [
    it({
      name: 'keeps patterns, drops unlocked names, and narrows version lists to locked versions',
      fn: async () => {
        expect(pruneSelectors({
          selectors: ['tokio*', 'gone', 'ring', 'zbus@5.19.0 || 5.19.1', 'old@1.0.0',],
          locked: new Set(['ring@0.17.8', 'zbus@5.19.1',],),
        },),).toEqual(['tokio*', 'ring', 'zbus@5.19.1',],);
      },
    },),
  ],
},);
