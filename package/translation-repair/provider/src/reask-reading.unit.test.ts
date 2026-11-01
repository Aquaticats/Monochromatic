/**
 Tests for the rule a re-ask reads its alternates by: the reading of the meters taken now, or the rule that withholds
 every model with a hold the run applies at seating time (ledger B1330).

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  warnLinesDuring,
} from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import {
  HYPER_WET_ONLY,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/dryness-readings.test-fixture.ts';
import { rulePastReading, } from '../dist/final/node/index.mjs';

//region Reading for a re-ask tests

/**
 Logger the rule writes a failed read to.
 */
const l = contextRoot({ tag: 'reask-reading-test', },);

await describe({
  name: rulePastReading.name,
  children: [
    it({
      name: 'APPLIES EVERY HOLD WHERE THE CALLER READS NO DRYNESS, and applies the reading it is handed where it '
        + 'reads one',
      fn: async () => {
        const { signal, } = new AbortController();
        expect(await rulePastReading({ readDry: undefined, signal, l, },),).toEqual({ applied: 'always', },);
        expect(await rulePastReading({ readDry: async () => HYPER_WET_ONLY, signal, l, },),).toEqual({
          applied: 'in-reading',
          dry: HYPER_WET_ONLY,
        },);
      },
    },),

    it({
      name: 'APPLIES EVERY HOLD AND SAYS SO WHERE THE READ FAILS, and hands the failure back when the caller aborted',
      fn: async () => {
        const { signal, } = new AbortController();
        const { warned, result, } = await warnLinesDuring({
          run: async () =>
            rulePastReading({
              readDry: async () => {
                throw new Error('the meters did not answer',);
              },
              signal,
              l,
            },),
        },);
        expect(result,).toEqual({ applied: 'always', },);
        expect(warned.some(function namesTheRule(line,): boolean {
          return line.includes('could not be read',) && line.includes('leaving out every model',);
        },),).toBe(true,);

        const aborting = new AbortController();
        aborting.abort();
        await expect(rulePastReading({
          readDry: async () => {
            throw new Error('aborted',);
          },
          signal: aborting.signal,
          l,
        },),).rejects.toThrow('aborted',);
      },
    },),
  ],
},);

//endregion Reading for a re-ask tests
