/**
 Tests for the rewrite the rotation sweep makes of `benchRotation` (ledger B701): both shapes the function
 appears in, the cases it must refuse, and the answers a rewritten module gives.

 Fixtures are cat-themed invention.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  allInInputOrder,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

import { saltedBenchRotation, } from './rotation-salt-source.test-fixture.ts';

/**
 The function as the minified bundle spells it, with a stand-in body that answers the length of the prompt.
 */
const BUNDLED = 'function benchRotation({messages,size}){if(size<=0)return 0;return messages.length%size}export{benchRotation};';

/**
 The function as the source spells it.
 */
const SOURCED = 'export function benchRotation({messages,size}){if(size<=0)return 0;return messages.length%size}';

/**
 Loads a rewritten module as a module of its own and asks it.

 @param source - rewritten module text that exports `benchRotation`

 @param size - bench size

 @returns Rotation for two messages

 @example
 ```ts
 const start = await rotationOf({ source: BUNDLED, size: 5, },);
 ```
 */
async function rotationOf(
  {
    source,
    size,
  }: {
    readonly source: string;
    readonly size: number;
  },
): Promise<number> {
  /**
   The module, loaded from a data URL.
   */
  const loaded: unknown = await import(`data:text/javascript,${encodeURIComponent(source,)}`);
  if ((!isJsonRecord(loaded,)) || ((typeof loaded.benchRotation) !== 'function'))
    throw new Error('the rewritten module exports no benchRotation',);
  /**
   The rotation the module answers.
   */
  const answer: unknown = Reflect.apply(
    loaded.benchRotation,
    undefined,
    [
      {
        messages: ['purr', 'knead',],
        size,
      },
    ],
  );
  if ((typeof answer) !== 'number')
    throw new Error(`benchRotation answered ${String(answer,)}, not a number`,);
  return answer;
}

await describe({
  name: saltedBenchRotation.name,
  concurrency: 1,
  children: [
    it({
      name: 'shifts the bundle form by the salt, modulo the bench',
      fn: async () => {
        // The stand-in answers 2 % 5 = 2; salts 0, 1, 3 and 4 move it to 2, 3, 0 and 1.
        /**
         The rotation under each salt, asked side by side.
         */
        const answers = await allInInputOrder({
          members: [0, 1, 3, 4,].map(function askUnder(salt,): Promise<number> {
            return rotationOf({
              source: saltedBenchRotation({ source: BUNDLED, salt, },),
              size: 5,
            },);
          },),
        },);
        expect(answers,).toStrictEqual([2, 3, 0, 1,],);
      },
    },),
    it({
      name: 'keeps the answer of an empty bench at zero whatever the salt',
      fn: async () => {
        expect(
          await rotationOf({
            source: saltedBenchRotation({ source: BUNDLED, salt: 7, },),
            size: 0,
          },),
        ).toBe(0,);
      },
    },),
    it({
      name: 'keeps the export keyword of the source form on the wrapper',
      fn: async () => {
        /**
         The rewritten source form.
         */
        const rewritten = saltedBenchRotation({ source: SOURCED, salt: 2, },);
        expect(rewritten,).toContain('function benchRotationUnsalted(',);
        expect(rewritten,).toContain('export function benchRotation(args){',);
        expect(rewritten,).not.toContain('export function benchRotationUnsalted(',);
      },
    },),
    it({
      name: 'leaves a module that declares no benchRotation alone',
      fn: async () => {
        /**
         A module that only names the function.
         */
        const source = 'export const purr = benchRotation;';
        expect(saltedBenchRotation({ source, salt: 2, },),).toBe(source,);
      },
    },),
    it({
      name: 'refuses a module that declares it twice, since the shape is then unknown',
      fn: async () => {
        /**
         The refusal.
         */
        const refusal = caught(function twice(): void {
          saltedBenchRotation({ source: `${BUNDLED}${BUNDLED}`, salt: 1, },);
        },);
        expect(String(refusal,),).toContain('declares benchRotation 2 times',);
      },
    },),
    it({
      name: 'refuses a salt that is not a whole number from zero',
      fn: async () => {
        for (const salt of [-1, 1.5, Number.NaN,]) {
          expect(caught(function odd(): void {
            saltedBenchRotation({ source: BUNDLED, salt, },);
          },),).toBeDefined();
        }
      },
    },),
  ],
},);
