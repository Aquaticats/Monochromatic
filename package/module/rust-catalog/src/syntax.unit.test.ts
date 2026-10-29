/**
 Unit tests for the built crate-name and version-requirement validators.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  floorVersion,
  isBoundedRequirement,
  isCrateName,
  isFloorRequirement,
  isVersion,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'syntax',
  children: [
    it({
      name: isCrateName.name,
      fn: async () => {
        expect(['anyhow', 'tracing-subscriber', 'ra_ap_hir', 'a', 'a'.repeat(64,),].map(isCrateName,),).toEqual([
          true,
          true,
          true,
          true,
          true,
        ],);
        expect(['', '9lives', '_x', '-x', 'a'.repeat(65,), 'a b', 'a.b', '../x', 'a\nb', 'é',].map(isCrateName,),).toEqual(
          [false, false, false, false, false, false, false, false, false, false,],
        );
      },
    },),

    it({
      name: isVersion.name,
      fn: async () => {
        expect(['1', '0.4', '1.0.104', '1.0.0-alpha.5', '1.0.0+build.7', '1.0.0-rc.1+b',].map(isVersion,),).toEqual([
          true,
          true,
          true,
          true,
          true,
          true,
        ],);
        expect(['', '1.x', '01', '1.02', '1.2.3.4', '1.0.0-', '1.0.0+', 'v1', '1..2', '1.0.0-a b',].map(isVersion,),).toEqual([
          false,
          false,
          false,
          false,
          false,
          false,
          false,
          false,
          false,
          false,
        ],);
      },
    },),

    it({
      name: isFloorRequirement.name,
      fn: async () => {
        expect(['>=1', '>=1.0.104', '>= 0.4',].map(isFloorRequirement,),).toEqual([true, true, true,],);
        expect(['1', '>1', '>=1, <2', '>=', '<=1', '*',].map(isFloorRequirement,),).toEqual([
          false,
          false,
          false,
          false,
          false,
          false,
        ],);
      },
    },),

    it({
      name: isBoundedRequirement.name,
      fn: async () => {
        expect(['0.58', '^0.58', '~0.58.1', '=1.18.1',].map(isBoundedRequirement,),).toEqual([true, true, true, true,],);
        expect(['>=0.58', '*', '<1', '0.58, <1', '',].map(isBoundedRequirement,),).toEqual([false, false, false, false, false,],);
      },
    },),

    it({
      name: floorVersion.name,
      fn: async () => {
        expect(floorVersion('>=1.0.104',),).toBe('1.0.104',);
        try {
          floorVersion('1.0.104',);
        } catch (error) {
          expect(error,).toBeInstanceOf(TypeError,);
          return;
        }
        throw new Error('expected a TypeError',);
      },
    },),
  ],
},);
