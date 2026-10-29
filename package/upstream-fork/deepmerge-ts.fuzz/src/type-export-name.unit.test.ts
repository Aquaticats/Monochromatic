/**
 Checks that the type tests reference every type the installed deepmerge-ts
 declaration file exports (`./type-export-name.ts`).

 `ObjectType` is left to `./surface-known-defect.unit.test.ts`, which pins
 its missing runtime export.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { fileURLToPath, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  declaredTypeNames,
  EXPORTED_TYPE_NAMES,
} from './type-export-name.ts';

/**
 Exported types covered by other files.
 */
const COVERED_ELSEWHERE: ReadonlySet<string> = new Set(['ObjectType',],);

await describe({
  name: 'deepmerge-ts exported type coverage',
  children: [
    it({
      name: 'the declaration reader finds prefixed and renamed exports but not prose',
      fn: async () => {
        expect(
          declaredTypeNames([
            '/** Use it as a guide. */',
            'export type A<T> = T;',
            'export interface B {',
            'export declare const enum C {',
            'export declare function d(): void;',
            'export {',
            '\tE$1 as E,',
            '};',
          ].join('\n',),),
        )
          .toEqual([
            'A',
            'B',
            'C',
            'E',
          ],);
      },
    },),
    it({
      name: 'every type the installed release exports is referenced by a type test',
      fn: async () => {
        /**
         Declaration file beside the resolved ESM entry.
         */
        const declarationPath = fileURLToPath(import.meta.resolve('deepmerge-ts',),)
          .replace(
          'index.mjs',
          'index.d.mts',
          );
        /**
         Names the release exports, minus those covered elsewhere.
         */
        const exported = declaredTypeNames(await readFile(
          declarationPath,
          'utf8',
        ),)
          .filter(function notElsewhere(name,) {
            return !COVERED_ELSEWHERE.has(name,);
          },);
        expect(exported,).toEqual(EXPORTED_TYPE_NAMES,);
      },
    },),
    it({
      name: 'the covered list is sorted and free of duplicates',
      fn: async () => {
        expect([...new Set(EXPORTED_TYPE_NAMES,),].toSorted(),).toEqual(EXPORTED_TYPE_NAMES,);
      },
    },),
  ],
},);
