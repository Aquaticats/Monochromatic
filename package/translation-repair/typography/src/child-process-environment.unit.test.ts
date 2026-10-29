/**
 Tests for the environment every child process of production code is started
 with: the parent's, with every variable whose name ends in `_API_KEY` and
 every setting of the package stated as absent. The proof from a child's side
 is in `production-children-environment.unit.test.ts`, which starts the real
 children of each site.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  childEnvironment,
  isCredentialName,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'child process environment',
  children: [
    it({
      name: 'STATES a name ending in _API_KEY as absent wherever it stands in the object, with or without a prefix',
      fn: async () => {
        /**
         What the helper returned, name by name in the parent's order.
         */
        const child = childEnvironment({
          parent: {
            WHISKER_API_KEY: 'a cat naps',
            PATH: '/cat/bin',
            TRANSLATION_REPAIR_OPENROUTER_API_KEY: 'a kitten naps',
            HOME: '/cat/home',
            EXA_API_KEY: 'a tabby naps',
          },
        },);
        expect(Object.entries(child,),).toEqual([
          ['WHISKER_API_KEY', undefined,],
          ['PATH', '/cat/bin',],
          ['TRANSLATION_REPAIR_OPENROUTER_API_KEY', undefined,],
          ['HOME', '/cat/home',],
          ['EXA_API_KEY', undefined,],
        ],);
      },
    },),
    it({
      name: 'STATES the two direct providers\' keys as absent: DEEPSEEK_API_KEY, which has no package prefix, and '
        + 'the Xiaomi key',
      fn: async () => {
        const child = childEnvironment({
          parent: {
            DEEPSEEK_API_KEY: 'a lynx naps',
            PATH: '/cat/bin',
            TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY: 'a bobcat naps',
          },
        },);
        expect(Object.entries(child,),).toEqual([
          ['DEEPSEEK_API_KEY', undefined,],
          ['PATH', '/cat/bin',],
          ['TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY', undefined,],
        ],);
      },
    },),
    it({
      name: 'KEEPS a name that only contains the text, ends otherwise, or spells it in another case',
      fn: async () => {
        /**
         Names none of which ends in the exact suffix.
         */
        const parent = {
          WHISKER_API_KEY_PATH: '/cat/key',
          API_KEY: 'a calico naps',
          MY_API_KEYS: 'a ginger naps',
          whisker_api_key: 'a tom naps',
          WHISKER_API_KEY_: 'a queen naps',
        };
        expect(childEnvironment({ parent, },),).toEqual(parent,);
      },
    },),
    it({
      name: 'STATES every setting of the package as absent and keeps a name that only starts alike',
      fn: async () => {
        /**
         What the helper returned, name by name in the parent's order.
         */
        const child = childEnvironment({
          parent: {
            TRANSLATION_REPAIR_RUNS_DIR: '/cat/runs',
            TRANSLATION_REPAIR_CORPUS_COMMIT: 'whiskers',
            TRANSLATION_REPAIRS_NOTE: 'a kitten plays',
            TRANSLATION_REPAIR: 'a tabby plays',
            XTRANSLATION_REPAIR_NOTE: 'a calico plays',
          },
        },);
        expect(Object.entries(child,),).toEqual([
          ['TRANSLATION_REPAIR_RUNS_DIR', undefined,],
          ['TRANSLATION_REPAIR_CORPUS_COMMIT', undefined,],
          ['TRANSLATION_REPAIRS_NOTE', 'a kitten plays',],
          ['TRANSLATION_REPAIR', 'a tabby plays',],
          ['XTRANSLATION_REPAIR_NOTE', 'a calico plays',],
        ],);
      },
    },),
    it({
      name: 'RETURNS an empty object for an empty environment',
      fn: async () => {
        expect(childEnvironment({ parent: {}, },),).toEqual({},);
      },
    },),
    it({
      name: 'KEEPS a name the parent already holds as undefined, a key among them, as an own name with no value',
      fn: async () => {
        /**
         What the helper returned.
         */
        const child = childEnvironment({ parent: { PLAIN_NOTE: undefined, WHISKER_API_KEY: undefined, }, },);
        expect(Object.entries(child,),).toEqual([
          ['PLAIN_NOTE', undefined,],
          ['WHISKER_API_KEY', undefined,],
        ],);
      },
    },),
    it({
      name: 'STATES a removed name as an own name with the value undefined, which a merge over the process '
        + 'environment overwrites and a name left out would not',
      fn: async () => {
        /**
         What the helper returned.
         */
        const child = childEnvironment({ parent: { WHISKER_API_KEY: 'a cat naps', }, },);
        expect(Object.hasOwn(
          child,
          'WHISKER_API_KEY',
        ),).toBe(true,);
        expect(child.WHISKER_API_KEY,).toBe(undefined,);
      },
    },),
    it({
      name: 'NEVER changes the parent or returns it, so the key stays in the parent and not in the copy',
      fn: async () => {
        /**
         The parent, frozen so a write would throw.
         */
        const parent = Object.freeze({ WHISKER_API_KEY: 'a cat naps', PATH: '/cat/bin', },);
        /**
         What the helper returned.
         */
        const child = childEnvironment({ parent, },);
        expect(child === parent,).toBe(false,);
        expect(parent,).toEqual({ WHISKER_API_KEY: 'a cat naps', PATH: '/cat/bin', },);
      },
    },),
    it({
      name: 'READS a name as a credential only where it ends in _API_KEY',
      fn: async () => {
        expect([
          'WHISKER_API_KEY',
          'TRANSLATION_REPAIR_SYNTHETIC_API_KEY',
          'WHISKER_API_KEY_PATH',
          'API_KEY',
          'whisker_api_key',
          '',
        ].map(function credential(name,): boolean {
          return isCredentialName({ name, },);
        },),).toEqual([
          true,
          true,
          false,
          false,
          false,
          false,
        ],);
      },
    },),
  ],
},);
