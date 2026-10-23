/**
 Tests for reading every reader's settled exchange of one picture: a reader that failed outright costs its own reading
 and nothing else, and the refusal that ends the run is raised instead.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { EveryProviderDryError, } from '@monochromatic-dev/translation-repair-provider';
import { modelDarkRefusal, } from '@monochromatic-dev/translation-repair-provider/ts/model-dark-refusal.test-fixture.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  containFailedReaders,
  type LabelledReading,
} from '../dist/final/node/index.mjs';

/**
 Readers in roster order.
 */
const READERS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
];

/**
 What the first reader made of the picture.
 */
const READ: LabelledReading = {
  modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
  reading: { kind: 'read', text: 'Name: Mittens', },
};

await describe({
  name: containFailedReaders.name,
  children: [
    it({
      name: 'PASSES A FULFILLED READING THROUGH and records a rejected reader as unavailable by its roster position, '
        + 'with one line naming the picture, the model and why it failed',
      fn: async () => {
        /**
         Lines the contain step wrote.
         */
        const lines: string[] = [];
        expect(containFailedReaders({
          settled: [
            { status: 'fulfilled', value: READ, },
            { status: 'rejected', reason: new Error('looped on a runaway reasoning channel',), },
          ],
          readerModelIds: READERS,
          assetName: 'noticeboard.webp',
          signal: new AbortController().signal,
          l: levelCapturingLogger({ lines, },),
        },),).toStrictEqual([
          READ,
          {
            modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            reading: { kind: 'unavailable', reason: 'reader-failed', },
          },
        ],);
        expect(lines,).toEqual([
          `warn noticeboard.webp: ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER} failed outright, so it contributes no reading `
            + '(refused by Error)',
        ],);
      },
    },),

    it({
      name: 'SAYS THE CALLER STOPPED THE READING for a failure under the caller\'s stop, and writes nothing for the '
        + 'caller\'s own reason',
      fn: async () => {
        /**
         Lines the contain step wrote.
         */
        const lines: string[] = [];
        /**
         Why the caller stopped.
         */
        const reason = new Error('the owner called the cats in',);
        /**
         The caller's steering, already stopped.
         */
        const steering = new AbortController();
        steering.abort(reason,);
        containFailedReaders({
          settled: [
            { status: 'rejected', reason: new Error('refused',), },
            { status: 'rejected', reason, },
          ],
          readerModelIds: READERS,
          assetName: 'noticeboard.webp',
          signal: steering.signal,
          l: levelCapturingLogger({ lines, },),
        },);
        expect(lines,).toEqual([
          `warn noticeboard.webp: ${SEAT_SYNTHETIC_VISION_WITHHELD} failed outright (refused by Error), `
            + 'and the caller stopped the reading',
        ],);
      },
    },),

    it({
      name: 'RAISES THE EVERY-PROVIDER-DRY REFUSAL, unchanged, whichever reader met it, and writes no failed-reader line',
      fn: async () => {
        /**
         Lines the contain step wrote.
         */
        const lines: string[] = [];
        /**
         The refusal the router raised for the second reader.
         */
        const dry = new EveryProviderDryError({ measured: 'every meter dry', },);
        expect(() => containFailedReaders({
          settled: [
            { status: 'rejected', reason: new Error('refused',), },
            { status: 'rejected', reason: dry, },
          ],
          readerModelIds: READERS,
          assetName: 'noticeboard.webp',
          signal: new AbortController().signal,
          l: levelCapturingLogger({ lines, },),
        },),).toThrow(EveryProviderDryError,);
        expect(lines,).toEqual([],);
      },
    },),

    it({
      name: 'CONTAINS A MODEL-DARK REFUSAL like any other failed reader, writing its failed-reader line and raising '
        + 'nothing, since a reader whose own providers are dry costs the picture one reading and not the run',
      fn: async () => {
        /**
         Lines the contain step wrote.
         */
        const lines: string[] = [];
        expect(() => containFailedReaders({
          settled: [
            { status: 'rejected', reason: new Error('refused',), },
            { status: 'rejected', reason: modelDarkRefusal({ modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, },), },
          ],
          readerModelIds: READERS,
          assetName: 'noticeboard.webp',
          signal: new AbortController().signal,
          l: levelCapturingLogger({ lines, },),
        },),).not.toThrow();
        expect(lines.length,).toBe(2,);
      },
    },),

    it({
      name: 'REFUSES A SETTLEMENT THE ROSTER CANNOT NAME, which no roster mapped one ask per slot can produce',
      fn: async () => {
        expect(() =>
          containFailedReaders({
            settled: [{ status: 'rejected', reason: new Error('refused',), },],
            readerModelIds: [],
            assetName: 'noticeboard.webp',
            signal: new AbortController().signal,
            l: levelCapturingLogger({ lines: [], },),
          },)
        ).toThrow('could not name the one at index 0',);
      },
    },),
  ],
},);
